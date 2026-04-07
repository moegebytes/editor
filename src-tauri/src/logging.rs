use std::fs;
use std::path::Path;

use log::error;
use simplelog::{CombinedLogger, Config, LevelFilter, SimpleLogger, WriteLogger};

const MAX_LOG_FILES: usize = 10;

pub fn init(config_dir: &Path) {
  let logs_dir = config_dir.join("logs");
  let _ = fs::create_dir_all(&logs_dir);

  let timestamp = time_format::now()
    .and_then(|t| time_format::strftime_utc("%Y-%m-%dT%H-%M-%S", t))
    .unwrap_or_else(|_| "unknown".to_string());
  let log_path = logs_dir.join(format!("yona-{}.log", timestamp));

  let console_level = if cfg!(debug_assertions) {
    LevelFilter::Trace
  } else {
    LevelFilter::Info
  };

  let file_logger = fs::File::create(&log_path)
    .ok()
    .map(|file| WriteLogger::new(LevelFilter::Debug, Config::default(), file));

  let mut loggers: Vec<Box<dyn simplelog::SharedLogger>> = vec![SimpleLogger::new(console_level, Config::default())];
  if let Some(fl) = file_logger {
    loggers.push(fl);
  }

  let _ = CombinedLogger::init(loggers);
  install_panic_hook();
  prune_old_logs(&logs_dir);
}

fn install_panic_hook() {
  let default_hook = std::panic::take_hook();
  std::panic::set_hook(Box::new(move |info| {
    let payload = if let Some(s) = info.payload().downcast_ref::<&str>() {
      s.to_string()
    } else if let Some(s) = info.payload().downcast_ref::<String>() {
      s.clone()
    } else {
      "unknown panic".to_string()
    };

    let location = info
      .location()
      .map(|l| format!(" at {}:{}:{}", l.file(), l.line(), l.column()))
      .unwrap_or_default();

    let backtrace = std::backtrace::Backtrace::force_capture();
    error!("PANIC{}: {}\n{}", location, payload, backtrace);

    default_hook(info);
  }));
}

fn prune_old_logs(logs_dir: &Path) {
  let mut logs: Vec<_> = fs::read_dir(logs_dir)
    .into_iter()
    .flatten()
    .flatten()
    .filter(|e| {
      e.path()
        .file_name()
        .and_then(|n| n.to_str())
        .is_some_and(|n| n.starts_with("yona-") && n.ends_with(".log"))
    })
    .collect();

  if logs.len() <= MAX_LOG_FILES {
    return;
  }

  logs.sort_by_key(|e| e.file_name());
  let to_remove = logs.len() - MAX_LOG_FILES;
  for entry in &logs[..to_remove] {
    let _ = fs::remove_file(entry.path());
  }
}
