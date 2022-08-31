use std::{path::PathBuf, fs};

use color_eyre::eyre;
use serde::{Serialize, Deserialize};

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Notifications {
    pub notifications: Vec<NotificationDetails>
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NotificationDetails {
    pub label: String,
    pub cron: String,
    pub level: String
}

pub fn load_file_and_deserialise(path: &PathBuf) -> eyre::Result<Notifications> {
  let config_content = fs::read_to_string(path)?;
  let notifications: Notifications = serde_yaml::from_str(&config_content)?;
  Ok(notifications)
}

pub fn save_contents(path: &PathBuf, notify: &Notifications) -> eyre::Result<()> {
  fs::write(path, serde_yaml::to_string(notify)?);
  Ok(())
}
