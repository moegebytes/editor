use std::path::{Path, PathBuf};
use std::sync::Mutex;

use log::{error, info};
use serde::Serialize;
use tauri::{AppHandle, Manager, State};
use tauri_plugin_opener::OpenerExt;

use crate::core::FlatEntry;
use crate::dictionary::{DictDb, KanjiEntry, LookupResult};
use crate::project::{self, Project, ProjectFiles, ProjectSettings, RecentProject};
use crate::wiktionary::{WiktCache, WiktResult};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectWithEntries {
  pub id: String,
  #[serde(flatten)]
  pub project: Project,
  pub entries: Vec<FlatEntry>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectInfo {
  pub name: String,
  pub files: ProjectFiles,
  pub settings: ProjectSettings,
}

pub struct DictState(pub Mutex<Option<DictDb>>);
pub struct ProjectState(pub Mutex<Option<(String, Project, PathBuf)>>);
pub struct WiktState(pub Mutex<Option<WiktCache>>);

impl DictState {
  fn with_db<T>(&self, f: impl FnOnce(&DictDb) -> Result<T, String>) -> Result<T, String> {
    let lock = self.0.lock().map_err(|e| e.to_string())?;
    let db = lock.as_ref().ok_or("Dictionary not initialized")?;
    f(db)
  }
}

impl WiktState {
  fn with_cache<T>(&self, f: impl FnOnce(&WiktCache) -> Result<T, String>) -> Result<T, String> {
    let lock = self.0.lock().map_err(|e| e.to_string())?;
    let cache = lock.as_ref().ok_or("Wiktionary lookup not available")?;
    f(cache)
  }
}

impl ProjectState {
  fn with_ref<T>(&self, f: impl FnOnce(&Project, &Path) -> Result<T, String>) -> Result<T, String> {
    let lock = self.0.lock().map_err(|e| e.to_string())?;
    let (_, proj, path) = lock.as_ref().ok_or("No project open")?;
    f(proj, path)
  }

  fn with_mut<T>(&self, f: impl FnOnce(&mut Project, &Path) -> Result<T, String>) -> Result<T, String> {
    let mut lock = self.0.lock().map_err(|e| e.to_string())?;
    let (_, proj, path) = lock.as_mut().ok_or("No project open")?;
    f(proj, path)
  }

  fn set(&self, id: String, proj: Project, path: PathBuf) -> Result<(), String> {
    let mut lock = self.0.lock().map_err(|e| e.to_string())?;
    *lock = Some((id, proj, path));
    Ok(())
  }

  fn update_if_open(&self, id: &str, name: &str, files: &ProjectFiles, settings: &ProjectSettings) {
    if let Ok(mut lock) = self.0.lock() {
      if let Some((open_id, proj, _)) = lock.as_mut() {
        if open_id == id {
          proj.name = name.to_string();
          proj.files = files.clone();
          proj.settings = settings.clone();
        }
      }
    }
  }
}

fn map_strings_err(e: &crate::strings::StringsError, label: &str) -> String {
  match e {
    crate::strings::StringsError::Io { path, source, .. }
    | crate::strings::StringsError::ResolvePath { path, source, .. } => {
      crate::util::friendly_io_msg(label, path, source)
    }
    _ => e.to_string(),
  }
}

fn pair_files(files: &ProjectFiles) -> Result<Vec<FlatEntry>, String> {
  let jp_path = Path::new(&files.jp);
  let en_path = Path::new(&files.en);
  let jp = crate::strings::parse_strings(jp_path)
    .map_err(|e| crate::util::log_err(map_strings_err(&e, "Japanese file")))?;
  let en = crate::strings::parse_strings(en_path)
    .map_err(|e| crate::util::log_err(map_strings_err(&e, "English file")))?;
  Ok(crate::core::pair_files(&jp, &en))
}

#[tauri::command]
pub fn save_translation(entries: Vec<FlatEntry>, state: State<'_, ProjectState>) -> Result<(), String> {
  let path = state.with_ref(|proj, _| Ok(proj.files.en.clone()))?;
  info!("Saving translation to '{}'", path);
  let reconstructed = crate::core::reconstruct_entries(&entries);
  crate::strings::write_strings(&reconstructed, Path::new(&path))
    .map_err(|e| crate::util::log_err(e.to_string()))
}

#[tauri::command]
pub fn lookup_word(query: String, state: State<'_, DictState>) -> Result<LookupResult, String> {
  state.with_db(|db| db.lookup_word(&query).map_err(|e| {
    error!("Dictionary lookup failed for '{}': {}", query, e);
    e.to_string()
  }))
}

#[tauri::command]
pub fn lookup_kanji(ch: String, state: State<'_, DictState>) -> Result<Option<KanjiEntry>, String> {
  let c = ch.chars().next().ok_or("Empty character")?;
  state.with_db(|db| db.lookup_kanji(c).map_err(|e| {
    error!("Kanji lookup failed for '{}': {}", ch, e);
    e.to_string()
  }))
}

#[tauri::command]
pub fn lookup_wiktionary(term: String, state: State<'_, WiktState>) -> Result<WiktResult, String> {
  state.with_cache(|cache| cache.lookup(&term).map_err(|e| {
    error!("Wiktionary lookup failed for '{}': {}", term, e);
    e.to_string()
  }))
}

#[tauri::command]
pub fn clear_wiktionary_cache(state: State<'_, WiktState>) -> Result<(), String> {
  state.with_cache(|cache| cache.clear_cache().map_err(|e| {
    error!("Failed to clear Wiktionary cache: {}", e);
    e.to_string()
  }))
}

fn app_config_dir(app: &AppHandle) -> Result<PathBuf, String> {
  app.path().app_config_dir().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_project(
  app: AppHandle,
  name: String,
  files: ProjectFiles,
  state: State<'_, ProjectState>,
) -> Result<ProjectWithEntries, String> {
  info!("Creating project '{}' (jp='{}', en='{}')", name, files.jp, files.en);
  let entries = pair_files(&files)?;
  let data_dir = app_config_dir(&app)?;
  let (id, proj, path) = project::create_project(&data_dir, &name, files).map_err(crate::util::log_err)?;
  info!("Project created with id {}", id);
  state.set(id.clone(), proj.clone(), path)?;
  Ok(ProjectWithEntries { id, project: proj, entries })
}

#[tauri::command]
pub fn open_project(app: AppHandle, id: String, state: State<'_, ProjectState>) -> Result<ProjectWithEntries, String> {
  info!("Opening project {}", id);
  let data_dir = app_config_dir(&app)?;
  let (proj, path) = project::open_project(&data_dir, &id).map_err(crate::util::log_err)?;
  let entries = pair_files(&proj.files)?;
  state.set(id.clone(), proj.clone(), path)?;
  Ok(ProjectWithEntries { id, project: proj, entries })
}

#[tauri::command]
pub fn save_project(state: State<'_, ProjectState>) -> Result<(), String> {
  info!("Saving project state");
  state.with_ref(|proj, path| project::save_project(path, proj).map_err(crate::util::log_err))
}

#[tauri::command]
pub fn confirm_line(index: usize, state: State<'_, ProjectState>) -> Result<(), String> {
  state.with_mut(|proj, _| {
    proj.confirmed_lines.insert(index);
    Ok(())
  })
}

#[tauri::command]
pub fn unconfirm_line(index: usize, state: State<'_, ProjectState>) -> Result<(), String> {
  state.with_mut(|proj, _| {
    proj.confirmed_lines.remove(&index);
    Ok(())
  })
}

#[tauri::command]
pub fn list_recent_projects(app: AppHandle) -> Result<Vec<RecentProject>, String> {
  let data_dir = app_config_dir(&app)?;
  Ok(project::list_recent(&data_dir))
}

#[tauri::command]
pub fn list_all_projects(app: AppHandle) -> Result<Vec<RecentProject>, String> {
  let data_dir = app_config_dir(&app)?;
  Ok(project::list_all(&data_dir))
}

#[tauri::command]
pub fn remove_recent_project(app: AppHandle, id: String) -> Result<(), String> {
  let data_dir = app_config_dir(&app)?;
  project::remove_from_recent(&data_dir, &id);
  Ok(())
}

#[tauri::command]
pub fn delete_project(app: AppHandle, id: String) -> Result<(), String> {
  info!("Deleting project {}", id);
  let data_dir = app_config_dir(&app)?;
  project::delete_project(&data_dir, &id).map_err(crate::util::log_err)
}

#[tauri::command]
pub fn export_project(dest_path: String, state: State<'_, ProjectState>) -> Result<(), String> {
  state.with_ref(|proj, _| {
    let mut exported = proj.clone();
    exported.files = ProjectFiles {
      jp: String::new(),
      en: String::new(),
    };
    let json =
      serde_json::to_string(&exported).map_err(|e| format!("serialize error: {}", e))?;
    std::fs::write(&dest_path, &json)
      .map_err(|e| crate::util::log_err(crate::util::friendly_io_msg("", Path::new(&dest_path), &e)))
  })
}

#[tauri::command]
pub fn get_project_info(app: AppHandle, id: String) -> Result<ProjectInfo, String> {
  let data_dir = app_config_dir(&app)?;
  let proj = project::read_project(&data_dir, &id).map_err(crate::util::log_err)?;
  Ok(ProjectInfo {
    name: proj.name,
    files: proj.files,
    settings: proj.settings,
  })
}

#[tauri::command]
pub fn update_project(
  app: AppHandle,
  id: String,
  name: String,
  files: ProjectFiles,
  settings: ProjectSettings,
  state: State<'_, ProjectState>,
) -> Result<(), String> {
  info!("Updating project {}", id);
  let data_dir = app_config_dir(&app)?;
  project::update_project(&data_dir, &id, &name, files.clone(), settings.clone())
    .map_err(crate::util::log_err)?;
  state.update_if_open(&id, &name, &files, &settings);
  Ok(())
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportPreview {
  pub name: String,
  pub confirmed_count: usize,
}

fn read_import_file(source_path: &str) -> Result<Project, String> {
  let path = Path::new(source_path);
  let content = std::fs::read_to_string(path)
    .map_err(|e| crate::util::log_err(crate::util::friendly_io_msg("", path, &e)))?;
  serde_json::from_str::<Project>(&content)
    .map_err(|e| crate::util::log_err(format!("Invalid project file: {}", e)))
}

#[tauri::command]
pub fn preview_import(source_path: String) -> Result<ImportPreview, String> {
  let proj = read_import_file(&source_path)?;
  Ok(ImportPreview {
    name: proj.name,
    confirmed_count: proj.confirmed_lines.len(),
  })
}

#[tauri::command]
pub fn import_project(
  app: AppHandle,
  source_path: String,
  name: String,
  files: ProjectFiles,
  state: State<'_, ProjectState>,
) -> Result<ProjectWithEntries, String> {
  let imported = read_import_file(&source_path)?;
  let entries = pair_files(&files)?;
  let data_dir = app_config_dir(&app)?;
  let (id, mut proj, path) = project::create_project(&data_dir, &name, files)
    .map_err(crate::util::log_err)?;
  proj.confirmed_lines = imported.confirmed_lines;
  proj.settings = imported.settings;
  project::save_project(&path, &proj).map_err(crate::util::log_err)?;
  let ret = proj.clone();
  state.set(id.clone(), proj, path)?;
  Ok(ProjectWithEntries { id, project: ret, entries })
}

#[tauri::command]
pub fn open_app_dir(app: AppHandle) -> Result<(), String> {
  let config_dir = app_config_dir(&app)?;
  app.opener().open_path(&*config_dir.to_string_lossy(), None::<&str>).map_err(|e| e.to_string())
}
