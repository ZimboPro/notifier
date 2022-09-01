use std::{path::PathBuf, str::FromStr};

use chrono::Local;

use eframe::{App, egui::{CentralPanel, Ui, Window, Context, Button, RichText}};

use crate::yaml::{Notifications, NotificationDetails, save_contents};
use crate::cron::Schedule;

#[derive(Default)]
pub struct Notifier {
  notifications: Notifications,
  notification_detail: NotificationDetails,
  path: PathBuf,
  add: bool
}

impl Notifier {
  pub fn new(_cc: &eframe::CreationContext<'_>, path: PathBuf) -> Self {
    Self { path, ..Self::default() }
  }

  pub fn new_with_data(_cc: &eframe::CreationContext<'_>, notify: Notifications, path: PathBuf) -> Self {
    Self {
      notifications: notify,
      notification_detail: NotificationDetails::default(),
      path,
      add: false
    }
  }

  fn render_add_notification(&mut self, ctx: & Context) {
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
        self.notification_detail.level = "Info".to_string();
        self.notifications.notifications.push(self.notification_detail.clone());
        self.notification_detail = NotificationDetails::default();
        let result = save_contents(&self.path, &self.notifications);
        match result {
            Ok(()) => {
              // Some form of a toast or notification
              self.add = false;
            },
            Err(err) => {
              // Some form of a toast or notification
              println!("Error saving the notifications: {}", err);
            }
        };
      }
    });
  }

  fn render_card(&mut self, ui: &mut Ui) {
    let mut remove = false;
    let mut del = 0;
    for (index, noti) in self.notifications.notifications.iter().enumerate() {
        ui.add_space(10.);
        ui.horizontal_top(|ui| {
          let label = RichText::new(noti.label.as_str()).size(20.);
          ui.label(label);
          let resp = ui.button("Remove");
          if resp.clicked() {
            remove = true;
            del = index;
          }
        });
        ui.label(noti.cron.as_str());
        ui.horizontal_top(|ui| {
          ui.label("Next notification at: ");
          let cron = Schedule::from_str(noti.cron.as_str());
          match cron {
              Ok(job) => {
                let mut upcoming = job.upcoming(Local::now().timezone());
                ui.label(upcoming.next().unwrap_or_else(Local::now).to_string());
              },
              Err(err) => {
                ui.label(format!("Error: {}", err));
              }
          }
        });
        ui.add_space(10.);
        ui.separator();
    }
    if remove {
      self.notifications.notifications.remove(del);
      if let Err(err) = save_contents(&self.path, &self.notifications) {
        println!("Error: {}", err);
      }
    }
  }
}

impl App for Notifier {
  fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
    CentralPanel::default().show(ctx, |ui| {
      if self.notifications.notifications.is_empty() {
        self.render_add_notification(ctx);
      } else {
        self.render_card(ui);
        let resp = ui.button("Add");
        if resp.clicked() {
          self.add = true;
        }
        if self.add {
          self.render_add_notification(ctx);
        }
      }
    });
  }
}
