use std::path::{Path, PathBuf};

use log::warn;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
  #[serde(default)]
  pub auto_confirm_on_enter: bool,
  #[serde(default)]
  pub partial_search: bool,
}

#[allow(clippy::derivable_impls)]
impl Default for AppSettings {
  fn default() -> Self {
    Self {
      auto_confirm_on_enter: false,
      partial_search: false,
    }
  }
}

fn settings_path(app_data: &Path) -> PathBuf {
  app_data.join("settings.json")
}

pub fn load(app_data: &Path) -> AppSettings {
  let path = settings_path(app_data);
  if !path.exists() {
    return AppSettings::default();
  }
  std::fs::read_to_string(&path)
    .ok()
    .and_then(|s| serde_json::from_str(&s).ok())
    .unwrap_or_default()
}

pub fn save(app_data: &Path, settings: &AppSettings) -> Result<(), String> {
  let path = settings_path(app_data);
  let json = serde_json::to_string(settings).map_err(|e| format!("serialize error: {}", e))?;
  std::fs::write(&path, json).map_err(|e| {
    let msg = crate::util::friendly_io_msg("", &path, &e);
    warn!("{}", msg);
    msg
  })
}
