use std::io;
use std::path::Path;

use log::error;

pub fn log_err(e: String) -> String {
  error!("{}", e);
  e
}

pub fn to_katakana(s: &str) -> String {
  s.chars()
    .map(|c| match c {
      '\u{3041}'..='\u{3096}' => char::from_u32(c as u32 + 0x60).unwrap_or(c),
      _ => c,
    })
    .collect()
}

pub fn to_hiragana(s: &str) -> String {
  s.chars()
    .map(|c| match c {
      '\u{30A1}'..='\u{30F6}' => char::from_u32(c as u32 - 0x60).unwrap_or(c),
      _ => c,
    })
    .collect()
}

pub fn kana_variants(query: &str) -> Vec<String> {
  let katakana = to_katakana(query);
  let hiragana = to_hiragana(query);
  let mut variants = Vec::with_capacity(2);
  if katakana != query {
    variants.push(katakana);
  }
  if hiragana != query {
    variants.push(hiragana);
  }
  variants
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
