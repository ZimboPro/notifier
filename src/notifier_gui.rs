use std::{
  path::PathBuf,
  str::FromStr,
  sync::mpsc::{Receiver, Sender},
  thread,
};

use chrono::Local;

use cron::Schedule;
use eframe::{
  egui::{Button, CentralPanel, Context, RichText, ScrollArea, Ui, Window},
  App,
};

use crate::{
  check_cron,
  job_scheduler::{Job, JobScheduler},
  load_file_and_deserialise, save_contents, NotificationDetails, Notifications,
};

#[derive(Debug)]
pub struct Alarm {
  start_time: chrono::Local,
  duration: chrono::Duration,
  end_time: chrono::Local,
}

pub struct Notifier {
  notifications: Notifications,
  notification_detail: NotificationDetails,
  selected_index: Option<usize>,
  path: PathBuf,
  add: bool,
  alarms: Vec<Alarm>,
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
      add: false,
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
      add: false,
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

      let b = Button::new("Save");
      let cron = Schedule::from_str(self.notification_detail.cron.as_str());
      let valid = !self.notification_detail.label.is_empty() && cron.is_ok();
      let b = ui.add_enabled(valid, b);
      if b.enabled() && b.clicked() {
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
              self.add = false;
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

  fn render_card(&mut self, ui: &mut Ui) {
    ScrollArea::vertical().show(ui, |ui| {
      let mut remove = false;
      let mut add = false;
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
            add = true;
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
      if add {
        self.add = true;
        self.notification_detail = self.notifications.notifications[selected_index].clone();
        self.selected_index = Some(selected_index);
      }
    });
  }
}

impl App for Notifier {
  fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
    CentralPanel::default().show(ctx, |ui| {
      if self.notifications.notifications.is_empty() {
        self.render_add_notification(ctx);
      } else {
        self.render_card(ui);
        let btn = ui.button("Add");
        if btn.clicked() {
          self.add = true;
        }
        if self.add {
          self.render_add_notification(ctx);
        }
      }
    });
  }
}
