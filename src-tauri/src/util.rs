use std::io;
use std::path::Path;

use log::error;

pub fn log_err(e: String) -> String {
  error!("{}", e);
  e
}

pub fn friendly_io_msg(subject: &str, path: &Path, e: &io::Error) -> String {
  let p = path.display();
  let subj = if subject.is_empty() { "File" } else { subject };
  match e.kind() {
    io::ErrorKind::NotFound => format!("{} '{}' not found.", subj, p),
    io::ErrorKind::PermissionDenied => format!("Permission denied for '{}'.", p),
    io::ErrorKind::AlreadyExists => format!("{} '{}' already exists.", subj, p),
    _ => format!("Could not access '{}': {}.", p, e.kind()),
  }
}
