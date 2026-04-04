use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use log::warn;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::util::friendly_io_msg;

#[allow(clippy::empty_structs_with_brackets)]
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectSettings {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectFiles {
  pub jp: String,
  pub en: String,
}

fn default_version() -> u32 {
  1
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Project {
  #[serde(default = "default_version")]
  pub version: u32,
  pub name: String,
  pub files: ProjectFiles,
  #[serde(default)]
  pub confirmed_lines: BTreeSet<usize>,
  #[serde(default)]
  pub settings: ProjectSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecentProject {
  pub name: String,
  pub id: String,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct RecentIds {
  pub ids: Vec<String>,
}

impl RecentIds {
  const MAX_RECENT: usize = 10;

  pub fn add(&mut self, id: &str) {
    self.ids.retain(|i| i != id);
    self.ids.insert(0, id.to_string());
    self.ids.truncate(Self::MAX_RECENT);
  }
}

fn projects_dir(app_data: &Path) -> PathBuf {
  app_data.join("projects")
}

fn recent_path(app_data: &Path) -> PathBuf {
  app_data.join("recent.json")
}

pub fn project_path(app_data: &Path, id: &str) -> PathBuf {
  projects_dir(app_data).join(format!("{}.json", id))
}

pub fn read_project(app_data: &Path, id: &str) -> Result<Project, String> {
  let path = project_path(app_data, id);
  let content = std::fs::read_to_string(&path).map_err(|e| friendly_io_msg("", &path, &e))?;
  serde_json::from_str(&content).map_err(|e| format!("Invalid project file: {}", e))
}

pub fn update_project(
  app_data: &Path,
  id: &str,
  name: &str,
  files: ProjectFiles,
  settings: ProjectSettings,
) -> Result<(), String> {
  let path = project_path(app_data, id);
  let content = std::fs::read_to_string(&path).map_err(|e| friendly_io_msg("", &path, &e))?;
  let mut project: Project = serde_json::from_str(&content).map_err(|e| format!("Invalid project file: {}", e))?;
  project.name = name.to_string();
  project.files = files;
  project.settings = settings;
  save_project(&path, &project)
}

pub fn create_project(app_data: &Path, name: &str, files: ProjectFiles) -> Result<(String, Project, PathBuf), String> {
  let id = Uuid::new_v4().to_string();
  let path = project_path(app_data, &id);

  let project = Project {
    version: 1,
    name: name.to_string(),
    files,
    confirmed_lines: BTreeSet::new(),
    settings: ProjectSettings::default(),
  };

  let json = serde_json::to_string(&project).map_err(|e| format!("serialize error: {}", e))?;
  std::fs::write(&path, json).map_err(|e| friendly_io_msg("", &path, &e))?;

  let mut recent = load_recent_ids(app_data);
  recent.add(&id);
  save_recent_ids(app_data, &recent);

  Ok((id, project, path))
}

pub fn open_project(app_data: &Path, id: &str) -> Result<(Project, PathBuf), String> {
  let path = project_path(app_data, id);
  let content = std::fs::read_to_string(&path).map_err(|e| friendly_io_msg("", &path, &e))?;
  let project: Project = serde_json::from_str(&content).map_err(|e| format!("Invalid project file: {}", e))?;

  let mut recent = load_recent_ids(app_data);
  recent.add(id);
  save_recent_ids(app_data, &recent);

  Ok((project, path))
}

pub fn save_project(path: &Path, project: &Project) -> Result<(), String> {
  let json = serde_json::to_string(project).map_err(|e| format!("serialize error: {}", e))?;
  std::fs::write(path, json).map_err(|e| friendly_io_msg("", path, &e))
}

fn load_recent_ids(app_data: &Path) -> RecentIds {
  let path = recent_path(app_data);
  if !path.exists() {
    return RecentIds::default();
  }
  std::fs::read_to_string(&path)
    .ok()
    .and_then(|s| serde_json::from_str(&s).ok())
    .unwrap_or_default()
}

fn save_recent_ids(app_data: &Path, recent: &RecentIds) {
  let path = recent_path(app_data);
  if let Ok(json) = serde_json::to_string(recent) {
    if let Err(e) = std::fs::write(&path, json) {
      warn!("Failed to save recent projects to '{}': {}", path.display(), e);
    }
  }
}

pub fn list_recent(app_data: &Path) -> Vec<RecentProject> {
  let mut recent = load_recent_ids(app_data);
  let mut result = Vec::new();
  let mut pruned = false;

  recent.ids.retain(|id| {
    let path = project_path(app_data, id);
    match std::fs::read_to_string(&path) {
      Ok(content) => match serde_json::from_str::<Project>(&content) {
        Ok(proj) => {
          result.push(RecentProject {
            name: proj.name,
            id: id.clone(),
          });
          true
        }
        Err(e) => {
          warn!("Pruning recent project {}: corrupt file: {}", id, e);
          pruned = true;
          false
        }
      },
      Err(e) => {
        warn!("Pruning recent project {}: {}", id, e);
        pruned = true;
        false
      }
    }
  });

  if pruned {
    save_recent_ids(app_data, &recent);
  }

  result
}

pub fn list_all(app_data: &Path) -> Vec<RecentProject> {
  let dir = projects_dir(app_data);
  let entries = match std::fs::read_dir(&dir) {
    Ok(e) => e,
    Err(_) => return Vec::new(),
  };

  let mut result = Vec::new();
  for entry in entries.flatten() {
    let path = entry.path();
    if path.extension().and_then(|e| e.to_str()) != Some("json") {
      continue;
    }
    let id = match path.file_stem().and_then(|s| s.to_str()) {
      Some(s) => s.to_string(),
      None => continue,
    };
    let content = match std::fs::read_to_string(&path) {
      Ok(c) => c,
      Err(e) => {
        warn!("Skipping project '{}': {}", path.display(), e);
        continue;
      }
    };
    let proj: Project = match serde_json::from_str(&content) {
      Ok(p) => p,
      Err(e) => {
        warn!("Skipping project '{}': corrupt file: {}", path.display(), e);
        continue;
      }
    };
    result.push(RecentProject { name: proj.name, id });
  }

  result.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
  result
}

pub fn remove_from_recent(app_data: &Path, id: &str) {
  let mut recent = load_recent_ids(app_data);
  recent.ids.retain(|i| i != id);
  save_recent_ids(app_data, &recent);
}

pub fn delete_project(app_data: &Path, id: &str) -> Result<(), String> {
  remove_from_recent(app_data, id);
  let path = project_path(app_data, id);
  if path.exists() {
    std::fs::remove_file(&path).map_err(|e| friendly_io_msg("", &path, &e))?;
  }
  Ok(())
}
