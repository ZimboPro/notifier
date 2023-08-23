#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
extern crate chrono;
extern crate cron;

use std::{path::PathBuf, str::FromStr};

use auto_launch::AutoLaunch;
use eframe::{run_native, NativeOptions};
use notifier::Notifier;
use notify_rust::Notification;
use yaml::{load_file_and_deserialise, Notifications};

mod job_scheduler;
use clap::Parser;
use job_scheduler::{Job, JobScheduler, Schedule};
use std::time::Duration;

mod notifier;
mod yaml;

// http://0pointer.de/public/sound-naming-spec.html
#[cfg(all(unix, not(target_os = "macos")))]
static SOUND: &str = "dialog-information";

// https://allenbenz.github.io/winrt-notification/0_5_0/winrt_notification/enum.Sound.html
#[cfg(target_os = "windows")]
static SOUND: &str = "Reminder";

fn check_cron(cron_str: &str) -> bool {
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

fn schedule_notifications(notifications: Notifications) {
  if !notifications.notifications.is_empty() {
    let mut schedules = JobScheduler::new();
    let mut scheduled = false;
    for notify in notifications.notifications.iter() {
      let label = notify.label.as_str();
      let cron = notify.cron.as_str();
      if check_cron(cron) {
        scheduled = true;
        let schedule: Schedule = cron.parse().unwrap();
        schedules.add(Job::new(schedule, || {
          #[cfg(all(unix, not(target_os = "macos")))]
          Notification::new()
            .body(label)
            .sound_name(SOUND)
            .show()
            .unwrap()
            .wait_for_action(|action| match action {
              _ => (),
            });
          #[cfg(target_os = "macos")]
          Notification::new()
            .body(label)
            .show()
            .unwrap()
            .wait_for_action(|action| match action {
              _ => (),
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
        schedules.tick_with_system_time();
        std::thread::sleep(Duration::from_millis(500));
      }
    } else {
      println!("No jobs scheduled");
    }
  } else {
    println!("No jobs scheduled");
  }
}

#[derive(Parser, Debug)]
struct Args {
  /// Name of the person to greet
  #[clap(short, long)]
  gui: bool,
}

struct AppDetails {
  path: PathBuf,
  name: String,
}

fn get_app_name() -> color_eyre::eyre::Result<AppDetails> {
  let path = std::env::current_exe().unwrap();
  let name = String::from(path.file_name().unwrap().to_str().unwrap());
  Ok(AppDetails { path, name })
}

fn main() -> color_eyre::eyre::Result<()> {
  let app_details = get_app_name()?;
  let auto = AutoLaunch::new(
    app_details.name.as_str(),
    &app_details.path.as_os_str().to_string_lossy().to_owned(),
    &[] as &[&str],
  );
  auto.enable()?;
  color_eyre::install()?;
  let args = Args::parse();
  match home::home_dir() {
    Some(path) => {
      let config_dir = path.join(".config");
      let file_path = config_dir.join("notifier.yaml");
      if file_path.exists() {
        let notifications = load_file_and_deserialise(&file_path)?;
        if args.gui {
          let options = NativeOptions::default();
          run_native(
            "Notifier",
            options,
            Box::new(|cc| Box::new(Notifier::new_with_data(cc, notifications, file_path))),
          );
        } else {
          schedule_notifications(notifications);
        }
      } else if args.gui {
        let options = NativeOptions::default();
        if !config_dir.exists() {
          std::fs::create_dir_all(config_dir)?;
        }
        run_native(
          "Notifier",
          options,
          Box::new(|cc| Box::new(Notifier::new(cc, file_path))),
        );
      } else {
        println!("'{}' doesn't exist", file_path.to_str().unwrap());
      }
    }
    None => println!("Impossible to get your home dir!"),
  }
  Ok(())
}
