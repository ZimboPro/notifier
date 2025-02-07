use std::{
  path::PathBuf,
  str::FromStr,
  sync::mpsc::{Receiver, Sender},
  thread,
};

use chrono::{DateTime, Datelike, Duration, Local, TimeZone, Timelike};

use cron::Schedule;
use eframe::{
  egui::{Button, CentralPanel, Context, RichText, ScrollArea, SidePanel, Slider, Ui, Window},
  App,
};

use crate::{
  check_cron,
  job_scheduler::{Job, JobScheduler},
  load_file_and_deserialise, save_contents, NotificationDetails, Notifications,
};

#[derive(Debug, PartialEq)]
enum TimeType {
  Duration,
  Time,
}

#[derive(Debug)]
pub struct Alarm {
  start_time: DateTime<Local>,
  duration: chrono::Duration,
  end_time: DateTime<Local>,
  message: Option<String>,
}

#[derive(Debug, Default)]
struct AlarmInput {
  hour: i32,
  min: i32,
  message: String,
}

pub struct Notifier {
  notifications: Notifications,
  notification_detail: NotificationDetails,
  selected_index: Option<usize>,
  path: PathBuf,
  add_notification: bool,
  add_alarm: bool,
  alarm: AlarmInput,
  alarms: Vec<Alarm>,
  time_type: TimeType,
  tx: Sender<()>,
}

fn thread_and_notifications(rx: Receiver<()>, notifications: Notifications, path: PathBuf) {
  thread::spawn(move || {
    let mut schedules = JobScheduler::new();
    let mut notifications = notifications;
    loop {
      if !notifications.notifications.is_empty() {
        for notify in notifications.notifications.iter_mut() {
          if notify.job_id.is_none() {
            let cron = notify.cron.as_str();
            if check_cron(cron) {
              let schedule: Schedule = cron.parse().unwrap();
              let uuid = schedules.add(Job::new(schedule, notify.label.clone()));
              notify.job_id = Some(uuid);
            }
          }
        }
        schedules.tick_with_system_time();
      }
      if let Ok(_) = rx.try_recv() {
        schedules.remove_all();
        if let Ok(n) = load_file_and_deserialise(&path) {
          notifications = n;
        }
      }
      thread::sleep(std::time::Duration::from_secs(10));
    }
  });
}

impl Notifier {
  pub fn new(_cc: &eframe::CreationContext<'_>, path: PathBuf) -> Self {
    let (tx, rx) = std::sync::mpsc::channel::<()>();
    thread_and_notifications(rx, Notifications::default(), path.clone());
    Self {
      path,
      tx,
      notifications: Notifications::default(),
      notification_detail: NotificationDetails::default(),
      selected_index: None,
      add_notification: false,
      time_type: TimeType::Time,
      alarm: AlarmInput::default(),
      add_alarm: false,
      alarms: Vec::new(),
    }
  }

  pub fn new_with_data(
    _cc: &eframe::CreationContext<'_>,
    notify: Notifications,
    path: PathBuf,
  ) -> Self {
    let (tx, rx) = std::sync::mpsc::channel::<()>();
    thread_and_notifications(rx, notify.clone(), path.clone());
    Self {
      notifications: notify,
      notification_detail: NotificationDetails::default(),
      selected_index: None,
      path,
      add_notification: false,
      alarm: AlarmInput::default(),
      add_alarm: false,
      time_type: TimeType::Time,
      alarms: Vec::new(),
      tx,
    }
  }

  fn render_add_notification(&mut self, ctx: &Context) {
    Window::new("Add a new notification").show(ctx, |ui| {
      ui.label("Add a new notification configuration");
      ui.horizontal_top(|ui| {
        ui.label("Label:");
        ui.text_edit_singleline(&mut self.notification_detail.label);
      });
      ui.horizontal_top(|ui| {
        ui.label("Cron:");
        ui.text_edit_singleline(&mut self.notification_detail.cron);
      });
      ui.label("e.g. {{sec}}   {{min}}   {{hour}}   {{day of month}}   {{month}}   {{day of week}}   {{year}}");
      ui.hyperlink_to("Cron details", "https://crates.io/crates/job_scheduler");

      let save_btn = Button::new("Save");
      let cancel_btn = ui.button("Cancel");
      if cancel_btn.clicked() {
        self.notification_detail = NotificationDetails::default();
              self.add_notification = false;
              self.selected_index = None;
      }
      let cron = Schedule::from_str(self.notification_detail.cron.as_str());
      let valid = !self.notification_detail.label.is_empty() && cron.is_ok();
      let save_btn = ui.add_enabled(valid, save_btn);
      if save_btn.enabled() && save_btn.clicked() {
        if let Some(index) = self.selected_index {
          self.notifications.notifications[index].label = self.notification_detail.label.clone();
          self.notifications.notifications[index].cron = self.notification_detail.cron.clone();
        } else {
          self.notification_detail.level = "Info".to_string();
          self.notifications.notifications.push(self.notification_detail.clone());
        }
        let result = save_contents(&self.path, &self.notifications);
        match result {
          Ok(()) => {
            // Some form of a toast or notification for success
            if let Err(e) = self.tx.send(()) {
              eprintln!("Error sending message: {:?}", e);
            }
            self.notification_detail = NotificationDetails::default();
              self.add_notification = false;
              self.selected_index = None;
            },
            Err(err) => {
              // Some form of a toast or notification for failure
              eprintln!("Error saving to {}", self.path.display());
              eprintln!("Error saving the notifications: {}", err);
            }
        };
      }
    });
  }

  fn render_add_alarm(&mut self, ctx: &Context) {
    Window::new("Add Alarm").show(ctx, |ui| {
      ui.label("Add a alarm");
      ui.add(Slider::new(&mut self.alarm.hour, 0..=23).text("Hour"));
      ui.add(Slider::new(&mut self.alarm.min, 0..=59).text("Minute"));
      ui.label("Message (Optional):");
      ui.text_edit_singleline(&mut self.alarm.message);
      ui.radio_value(&mut self.time_type, TimeType::Time, "Alarm");
      ui.radio_value(&mut self.time_type, TimeType::Duration, "Timer");
      let btn = ui.button("Save");
      if btn.clicked() {
        match self.time_type {
          TimeType::Duration => {
            let dur =
              Duration::hours(self.alarm.hour as i64) + Duration::minutes(self.alarm.min as i64);
            let alarm = Alarm {
              start_time: Local::now(),
              duration: dur,
              end_time: Local::now() + dur,
              message: if self.alarm.message.trim().is_empty() {
                None
              } else {
                Some(self.alarm.message.clone())
              },
            };
            self.alarms.push(alarm);
            self.alarm = AlarmInput::default();
            self.add_alarm = false;
          }
          TimeType::Time => {
            let now = Local::now();
            let alarm_time = Local
              .with_ymd_and_hms(
                now.year(),
                now.month(),
                now.day(),
                self.alarm.hour as u32,
                self.alarm.min as u32,
                0,
              )
              .unwrap();
            let dur = alarm_time - now;
            let alarm = Alarm {
              start_time: now,
              duration: dur,
              end_time: alarm_time,
              message: if self.alarm.message.trim().is_empty() {
                None
              } else {
                Some(self.alarm.message.clone())
              },
            };
            self.alarms.push(alarm);
            self.alarm = AlarmInput::default();
            self.add_alarm = false;
          }
        }
      }
      let cancel_btn = ui.button("Cancel");
      if cancel_btn.clicked() {
        self.alarm = Default::default();
        self.add_alarm = false;
      }
    });
  }

  fn render_card(&mut self, ui: &mut Ui) {
    ScrollArea::vertical().show(ui, |ui| {
      let mut remove = false;
      let mut edit = false;
      let mut selected_index = 0;
      for (index, notification) in self.notifications.notifications.iter().enumerate() {
        ui.add_space(10.);
        ui.horizontal_top(|ui| {
          let label = RichText::new(notification.label.as_str()).size(20.);
          ui.label(label);
          let btn = ui.button("Remove");
          if btn.clicked() {
            remove = true;
            selected_index = index;
          }
          let btn = ui.button("Edit");
          if btn.clicked() {
            edit = true;
            selected_index = index;
          }
        });
        ui.label(notification.cron.as_str());
        ui.horizontal_top(|ui| {
          ui.label("Next notification at: ");
          let cron = Schedule::from_str(notification.cron.as_str());
          match cron {
            Ok(job) => {
              let mut upcoming = job.upcoming(Local::now().timezone());
              ui.label(upcoming.next().unwrap_or_else(Local::now).to_string());
            }
            Err(err) => {
              ui.label(format!("Error: {}", err));
            }
          }
        });
        ui.add_space(10.);
        ui.separator();
      }
      if remove {
        self.notifications.notifications.remove(selected_index);
        if let Err(err) = save_contents(&self.path, &self.notifications) {
          eprintln!("Error: {}", err);
        } else {
          if let Err(e) = self.tx.send(()) {
            eprintln!("Error sending message: {:?}", e);
          }
        }
      }
      if edit {
        self.add_notification = true;
        self.notification_detail = self.notifications.notifications[selected_index].clone();
        self.selected_index = Some(selected_index);
      }
    });
  }
}

impl App for Notifier {
  fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
    if !self.alarms.is_empty() {
      SidePanel::right("alarms").show(ctx, |ui| {
        let mut remove = false;
        let mut selected_index = 0;
        let mut edit = false;
        for (index, alarm) in self.alarms.iter().enumerate() {
          ui.add_space(10.);
          ui.horizontal_top(|ui| {
            let label =
              RichText::new(alarm.message.clone().unwrap_or("Alarm".to_string())).size(20.);
            ui.label(label);
            let btn = ui.button("Remove");
            if btn.clicked() {
              remove = true;
              selected_index = index;
            }
            let btn = ui.button("Edit");
            if btn.clicked() {
              edit = true;
              selected_index = index;
            }
          });
          ui.label(format!(
            "{:02}:{:02}",
            alarm.end_time.hour(),
            alarm.end_time.minute(),
          ));

          ui.add_space(10.);
          ui.separator();
        }
      });
    }
    CentralPanel::default().show(ctx, |ui| {
      if self.notifications.notifications.is_empty() && self.alarms.is_empty() {
        self.render_add_notification(ctx);
      } else {
        self.render_card(ui);
        let btn = ui.button("Add Notification");
        if btn.clicked() {
          self.add_notification = true;
        }
        let btn = ui.button("Add Alarm");
        if btn.clicked() {
          self.add_alarm = true;
        }
        if self.add_notification {
          self.render_add_notification(ctx);
        }
        if self.add_alarm {
          self.render_add_alarm(ctx);
        }
      }
    });
  }
}
