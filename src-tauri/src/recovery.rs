use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use log::{debug, warn};
use serde::{Deserialize, Serialize};

use crate::util::friendly_io_msg;

fn default_version() -> u32 {
  1
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecoveryEntry {
  pub en_text: Option<String>,
  #[serde(default)]
  pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecoveryData {
  #[serde(default = "default_version")]
  pub version: u32,
  pub project_id: String,
  pub entries: BTreeMap<usize, RecoveryEntry>,
  pub confirmed_lines: BTreeSet<usize>,
  pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecoveryInfo {
  pub timestamp: u64,
  pub entry_count: usize,
  pub confirmed_line_count: usize,
}

fn recovery_dir(app_data: &Path) -> PathBuf {
  app_data.join("recovery")
}

fn recovery_path(app_data: &Path, id: &str) -> PathBuf {
  recovery_dir(app_data).join(format!("{}.recovery.json", id))
}

pub fn ensure_dir(app_data: &Path) -> Result<(), String> {
  let dir = recovery_dir(app_data);
  std::fs::create_dir_all(&dir).map_err(|e| friendly_io_msg("recovery directory", &dir, &e))
}

pub fn write(
  app_data: &Path,
  project_id: &str,
  entries: &BTreeMap<usize, RecoveryEntry>,
  confirmed_lines: &BTreeSet<usize>,
) -> Result<(), String> {
  let path = recovery_path(app_data, project_id);
  let timestamp = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .map(|d| d.as_millis() as u64)
    .unwrap_or(0);

  let data = RecoveryData {
    version: 1,
    project_id: project_id.to_string(),
    entries: entries.clone(),
    confirmed_lines: confirmed_lines.clone(),
    timestamp,
  };

  let json = serde_json::to_string(&data).map_err(|e| format!("serialize error: {}", e))?;

  let tmp_path = path.with_extension("tmp");
  std::fs::write(&tmp_path, &json).map_err(|e| friendly_io_msg("recovery file", &tmp_path, &e))?;
  std::fs::rename(&tmp_path, &path).map_err(|e| friendly_io_msg("recovery file", &path, &e))?;

  debug!(
    "Recovery file written for project {} ({} changed entries)",
    project_id,
    entries.len()
  );
  Ok(())
}

pub fn check(app_data: &Path, id: &str) -> Option<RecoveryInfo> {
  let path = recovery_path(app_data, id);
  if !path.exists() {
    return None;
  }
  let content = std::fs::read_to_string(&path).ok()?;
  let data: RecoveryData = serde_json::from_str(&content).ok()?;
  Some(RecoveryInfo {
    timestamp: data.timestamp,
    entry_count: data.entries.len(),
    confirmed_line_count: data.confirmed_lines.len(),
  })
}

pub fn load(app_data: &Path, id: &str) -> Result<RecoveryData, String> {
  let path = recovery_path(app_data, id);
  let content = std::fs::read_to_string(&path).map_err(|e| friendly_io_msg("recovery file", &path, &e))?;
  serde_json::from_str(&content).map_err(|e| format!("Invalid recovery file: {}", e))
}

pub fn delete(app_data: &Path, id: &str) {
  let path = recovery_path(app_data, id);
  if path.exists() {
    if let Err(e) = std::fs::remove_file(&path) {
      warn!("Failed to delete recovery file '{}': {}", path.display(), e);
    } else {
      debug!("Recovery file deleted for project {}", id);
    }
  }
}
