extern crate yaml_rust;
use yaml_rust::{YamlLoader};

extern crate cron;
extern crate chrono;

use cron::Schedule;
use std::str::FromStr;

use notify_rust::Notification;
use std::fs;

extern crate job_scheduler;
use job_scheduler::{JobScheduler, Job};
use std::time::Duration;

#[cfg(all(unix, not(target_os = "macos")))]
static SOUND: &str = "message-new-instant";

#[cfg(target_os = "windows")]
static SOUND: &'static str = "Mail";

fn check_cron(cron_str: &str) -> bool {
  let cron = Schedule::from_str(cron_str);
  let variables = cron_str.split(' ').count();
  if variables != 7 {
    println!("Cron '{}' is invalid: There needs to be 7 variables", cron_str);
    println!("e.g. {{sec}}   {{min}}   {{hour}}   {{day of month}}   {{month}}   {{day of week}}   {{year}}");
    println!("See https://crates.io/crates/job_scheduler for more details");
    return false;
  }
  match cron {
      Ok(_) => {

        true
      }
      Err(err) => {
        println!("Cron {} is invalid: {}", cron_str, err.to_string());
        false
      }
  }
}

fn load_yaml_and_schedule(content: String) {
  let docs = YamlLoader::load_from_str(&content).unwrap();
  let config = &docs[0];
  if config["notifications"].is_array() {
    let notifications = config["notifications"].as_vec().unwrap();
    let mut schedules = JobScheduler::new();
    let mut scheduled = false;
    for notification in notifications.iter() {
      let label = notification["label"].as_str().unwrap();
      let cron = notification["cron"].as_str().unwrap();
      if check_cron(&cron) {
        scheduled = true;
        schedules.add(Job::new( cron.parse().unwrap(), || {
          #[cfg(all(unix, not(target_os = "macos")))]
          Notification::new()
            .body(label)
            .sound_name(SOUND)
            .show()
            .unwrap()
            .wait_for_action(|action| match action {
              _ => ()
            });
            #[cfg(target_os = "macos")]
          Notification::new()
            .body(label)
            .show()
            .unwrap()
            .wait_for_action(|action| match action {
              _ => ()
            });
          #[cfg(target_os = "windows")]
          Notification::new()
            .body(label)
            .sound_name(SOUND)
            .show()
            .unwrap();
        }));
      }
    }
    if scheduled {
      loop {
        schedules.tick();
        std::thread::sleep(Duration::from_millis(500));
      }
    } else {
      println!("No jobs scheduled");
    }
  } else {
    println!("'notifications' is not in the configuration or is not an array");
  }
}

fn load_file(path: String) {
  let config_content = fs::read_to_string(path);
  match config_content {
      Ok(content) => {
        load_yaml_and_schedule(content);
      }
      Err(e) => {
        println!("Err: {}", e.to_string());
      }
  }
}

fn main() {
  match home::home_dir() {
    Some(path) => {
      let file_path = path.join(".config/notifier.yaml");
      if file_path.exists() {
        load_file(file_path.to_str().unwrap().to_owned());
      } else {
        println!("'{}' doesn't exist", file_path.to_str().unwrap());
      }
    },
    None => println!("Impossible to get your home dir!"),
  }
}
