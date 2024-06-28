use std::{fs, path::PathBuf};

use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Default, Clone)]
pub struct Notifications {
  pub notifications: Vec<NotificationDetails>,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Default, Clone, Hash)]
pub struct NotificationDetails {
  pub label: String,
  pub cron: String,
  pub level: String,
  #[serde(skip)]
  pub job_id: Option<Uuid>,
}

#[derive(Debug, Error)]
pub enum YamlErrors {
  #[error("Error trying to read the config file")]
  CouldNotReadConfigFile,
  #[error("Error trying to deserialize the config file")]
  CouldNotDeserializeFile,
  #[error("Error trying to save to the config file")]
  CouldNotSaveToFile,
}

pub fn load_contents(path: &PathBuf) -> Result<String, YamlErrors> {
  std::fs::read_to_string(path).map_err(|_| YamlErrors::CouldNotReadConfigFile)
}

pub fn load_file_and_deserialise(path: &PathBuf) -> Result<Notifications, YamlErrors> {
  let config_content = load_contents(path)?;
  if config_content.trim().is_empty() {
    return Ok(Notifications::default());
  }
  let notifications: Notifications =
    serde_yaml::from_str(&config_content).map_err(|_| YamlErrors::CouldNotDeserializeFile)?;
  Ok(notifications)
}

pub fn save_contents(path: &PathBuf, notify: &Notifications) -> Result<(), YamlErrors> {
  fs::write(
    path,
    serde_yaml::to_string(notify).map_err(|_| YamlErrors::CouldNotSaveToFile)?,
  )
  .map_err(|_| YamlErrors::CouldNotSaveToFile)?;
  Ok(())
}
