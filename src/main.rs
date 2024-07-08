#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
extern crate chrono;
extern crate cron;

use std::path::PathBuf;

use auto_launch::AutoLaunch;
use eframe::{run_native, NativeOptions};
use notifier::{load_file_and_deserialise, notifier_gui::Notifier};

struct AppDetails {
  path: PathBuf,
  name: String,
}

fn get_app_name() -> color_eyre::eyre::Result<AppDetails> {
  let path = std::env::current_exe().unwrap();
  let name = String::from(path.file_name().unwrap().to_str().unwrap());
  Ok(AppDetails { path, name })
}

fn enable_auto_launch() -> color_eyre::eyre::Result<()> {
  if cfg!(debug_assertions) {
    return Ok(());
  }
  let app_details = get_app_name()?;
  let auto: AutoLaunch = AutoLaunch::new(
    app_details.name.as_str(),
    &app_details.path.as_os_str().to_string_lossy(),
    &[] as &[&str],
  );
  if !auto.is_enabled()? {
    auto.enable()?;
  }
  Ok(())
}

fn main() -> color_eyre::eyre::Result<()> {
  let res = enable_auto_launch();
  color_eyre::install()?;
  if res.is_err() {
    return res;
  }
  match home::home_dir() {
    Some(path) => {
      let config_dir = path.join(".config");
      if !config_dir.exists() {
        std::fs::create_dir_all(&config_dir)?;
      }
      #[cfg(not(debug_assertions))]
      let file_path = config_dir.join("notifier.yaml");
      #[cfg(debug_assertions)]
      let file_path = std::path::Path::new("notifier.yaml").to_path_buf();

      if file_path.exists() {
        let notifications = load_file_and_deserialise(&file_path)?;
        let options = NativeOptions::default();
        let s = run_native(
          "Notifier",
          options,
          Box::new(|cc| {
            Ok(Box::new(Notifier::new_with_data(
              cc,
              notifications,
              file_path,
            )))
          }),
        );
        if let Err(e) = s {
          eprintln!("Error: {:?}", e);
        }
      } else {
        let options = NativeOptions::default();
        if !config_dir.exists() {
          std::fs::create_dir_all(config_dir)?;
        }
        let s = run_native(
          "Notifier",
          options,
          Box::new(|cc| Ok(Box::new(Notifier::new(cc, file_path)))),
        );
        if let Err(e) = s {
          eprintln!("Error: {:?}", e);
        }
      }
    }
    None => println!("Impossible to get your home dir!"),
  }
  Ok(())
}
