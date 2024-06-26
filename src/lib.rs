mod job_scheduler;
mod yaml;
use std::{path::PathBuf, str::FromStr};

use cron::Schedule;
use thiserror::Error;
pub use yaml::{load_file_and_deserialise, save_contents};
pub use yaml::{NotificationDetails, Notifications};

#[derive(Debug, Error)]
pub enum Errors {
  #[error("Could not find file")]
  CouldNotFindFile,
  #[error("Error trying to get the home directory path")]
  CouldNotFindHomeDir,
  #[error("Error trying to create the config directory path")]
  CouldNotCreateConfigDir,
  #[error("Error creating the notification: {0}")]
  NotificationError(String),
}

pub fn get_config_path() -> Result<PathBuf, Errors> {
  match home::home_dir() {
    Some(home_dir) => {
      let config_dir = home_dir.join(".config");
      let config_path = config_dir.join("notifier.yaml");
      if !config_dir.is_dir() {
        std::fs::create_dir_all(config_dir).map_err(|_| Errors::CouldNotCreateConfigDir)?;
      }
      Ok(config_path)
    }
    None => Err(Errors::CouldNotFindHomeDir),
  }
}

pub fn check_cron(cron_str: &str) -> bool {
  let cron = Schedule::from_str(cron_str);
  let variables = cron_str.split(' ').count();
  if variables != 7 {
    println!(
      "Cron '{}' is invalid: There needs to be 7 variables",
      cron_str
    );
    println!("e.g. {{sec}}   {{min}}   {{hour}}   {{day of month}}   {{month}}   {{day of week}}   {{year}}");
    println!("See https://crates.io/crates/job_scheduler for more details");
    return false;
  }
  match cron {
    Ok(_) => true,
    Err(err) => {
      println!("Cron {} is invalid: {}", cron_str, err);
      false
    }
  }
}
