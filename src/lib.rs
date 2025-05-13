mod job_scheduler;
pub mod notifier_gui;
mod yaml;
use std::{path::PathBuf, str::FromStr};

use cron::Schedule;
use rswifi::{Result, WiFiInterface, WifiUtil};
use thiserror::Error;
pub use yaml::{NotificationDetails, Notifications};
pub use yaml::{load_file_and_deserialise, save_contents};

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
  let variables = cron_str.split(' ').filter(|x| !x.is_empty()).count();
  if variables != 7 {
    println!(
      "Cron '{}' is invalid: There needs to be 7 variables",
      cron_str
    );
    println!(
      "e.g. {{sec}}   {{min}}   {{hour}}   {{day of month}}   {{month}}   {{day of week}}   {{year}}"
    );
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

pub fn get_wifi() -> Vec<String> {
  match WifiUtil::new() {
    Ok(util) => {
      if let Ok(ifaces) = util.interfaces() {
        let iface = ifaces.first().unwrap();
        let mut wifi_list = Vec::new();
        if iface.scan().is_ok() {
          if let Ok(results) = iface.scan_results() {
            for ap in results {
              wifi_list.push(ap.ssid().to_string());
            }
          } else {
            println!("Error getting scan results");
          }
        }
        wifi_list
      } else {
        println!("Error getting interfaces");
        vec![]
      }
    }
    Err(_) => vec![],
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_check_cron() {
    assert_eq!(check_cron("0 0 * * * * *"), true);
    assert_eq!(check_cron("0 0 * * * *"), false);
  }

  #[test]
  fn test_cron_multiple_spaces() {
    assert_eq!(check_cron("0 0 * * * * * *"), false);
    assert_eq!(check_cron("0 0 * * * *"), false);
    assert_eq!(check_cron("0 0 * * * * * "), true);
    assert_eq!(check_cron("0 0 * * * *  * "), true);
  }
}
