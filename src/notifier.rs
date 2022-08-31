use eframe::App;

use crate::yaml::Notifications;

#[derive(Default)]
pub struct Notifier {
  notifications: Notifications
}

impl Notifier {
  pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
    Self::default()
  }

  pub fn new_with_data(_cc: &eframe::CreationContext<'_>, notify: Notifications) -> Self {
    Self { notifications: notify }
  }
}

impl App for Notifier {
  fn update(&mut self, ctx: &eframe::egui::Context, frame: &mut eframe::Frame) {

  }
}
