use std::path::{Path, PathBuf};
use std::sync::Mutex;

use log::{error, info};
use serde::Serialize;
use tauri::{AppHandle, State};
use tauri_plugin_opener::OpenerExt;

use crate::core::FlatEntry;
use crate::jmdict::{JmdictDb, LookupResult};
use crate::kanjidic::{KanjiDb, KanjiEntry};
use crate::project::{self, Project, ProjectFiles, ProjectSettings, RecentProject};
use crate::settings::AppSettings;
use crate::wiktionary::{WiktDb, WiktResult};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EnvironmentInfo {
  pub app_name: String,
  pub app_version: String,
  pub tauri_version: String,
  pub os: String,
  pub arch: String,
  pub debug: bool,
}

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

pub struct DataDir(pub PathBuf);

pub struct JmdictState(pub Mutex<Option<JmdictDb>>);
pub struct KanjidicState(pub Mutex<Option<KanjiDb>>);
pub struct OpenProject {
  pub id: String,
  pub project: Project,
  pub dir: PathBuf,
}

pub struct ProjectState(pub Mutex<Option<OpenProject>>);
pub struct WiktState(pub Mutex<Option<WiktDb>>);
pub struct AppSettingsState(pub Mutex<AppSettings>);

impl JmdictState {
  fn with_db<T>(&self, f: impl FnOnce(&JmdictDb) -> Result<T, String>) -> Result<T, String> {
    let lock = self.0.lock().map_err(|e| e.to_string())?;
    let db = lock.as_ref().ok_or("JMdict not initialized")?;
    f(db)
  }
}

impl KanjidicState {
  fn with_db<T>(&self, f: impl FnOnce(&KanjiDb) -> Result<T, String>) -> Result<T, String> {
    let lock = self.0.lock().map_err(|e| e.to_string())?;
    let db = lock.as_ref().ok_or("KANJIDIC2 not initialized")?;
    f(db)
  }
}

impl WiktState {
  fn with_db<T>(&self, f: impl FnOnce(&WiktDb) -> Result<T, String>) -> Result<T, String> {
    let lock = self.0.lock().map_err(|e| e.to_string())?;
    let db = lock.as_ref().ok_or("Wiktionary dictionary not loaded")?;
    f(db)
  }
}

impl ProjectState {
  fn with_ref<T>(&self, f: impl FnOnce(&Project, &Path) -> Result<T, String>) -> Result<T, String> {
    let lock = self.0.lock().map_err(|e| e.to_string())?;
    let open = lock.as_ref().ok_or("No project open")?;
    f(&open.project, &open.dir)
  }

  fn with_mut<T>(&self, f: impl FnOnce(&mut Project, &Path) -> Result<T, String>) -> Result<T, String> {
    let mut lock = self.0.lock().map_err(|e| e.to_string())?;
    let open = lock.as_mut().ok_or("No project open")?;
    f(&mut open.project, &open.dir)
  }

  fn set(&self, id: String, project: Project, dir: PathBuf) -> Result<(), String> {
    let mut lock = self.0.lock().map_err(|e| e.to_string())?;
    *lock = Some(OpenProject { id, project, dir });
    Ok(())
  }

  fn update_if_open(&self, id: &str, name: &str, files: &ProjectFiles, settings: &ProjectSettings) {
    if let Ok(mut lock) = self.0.lock() {
      if let Some(open) = lock.as_mut() {
        if open.id == id {
          open.project.name = name.to_string();
          open.project.files = files.clone();
          open.project.settings = settings.clone();
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
    crate::strings::StringsError::CircularInclude { .. } | crate::strings::StringsError::PathTraversal { .. } => {
      e.to_string()
    }
  }
}

fn pair_files(files: &ProjectFiles) -> Result<Vec<FlatEntry>, String> {
  let jp_path = Path::new(&files.jp);
  let en_path = Path::new(&files.en);
  let jp =
    crate::strings::parse_strings(jp_path).map_err(|e| crate::util::log_err(map_strings_err(&e, "Japanese file")))?;
  let en =
    crate::strings::parse_strings(en_path).map_err(|e| crate::util::log_err(map_strings_err(&e, "English file")))?;
  Ok(crate::core::pair_files(&jp, &en))
}

#[tauri::command]
pub fn save_translation(entries: Vec<FlatEntry>, state: State<'_, ProjectState>) -> Result<(), String> {
  let path = state.with_ref(|proj, _| Ok(proj.files.en.clone()))?;
  info!("Saving translation to '{}'", path);
  let reconstructed = crate::core::reconstruct_entries(&entries);
  crate::strings::write_strings(&reconstructed, Path::new(&path)).map_err(|e| crate::util::log_err(e.to_string()))
}

#[tauri::command]
pub fn lookup_jmdict(
  query: String,
  state: State<'_, JmdictState>,
  app_settings: State<'_, AppSettingsState>,
) -> Result<LookupResult, String> {
  let partial = app_settings.0.lock().map_err(|e| e.to_string())?.partial_search;
  state.with_db(|db| {
    db.lookup(&query, partial).map_err(|e| {
      error!("Dictionary lookup failed for '{}': {}", query, e);
      e.to_string()
    })
  })
}

#[tauri::command]
pub fn lookup_kanji(ch: String, state: State<'_, KanjidicState>) -> Result<Option<KanjiEntry>, String> {
  let c = ch.chars().next().ok_or("Empty character")?;
  state.with_db(|db| {
    db.lookup(c).map_err(|e| {
      error!("Kanji lookup failed for '{}': {}", ch, e);
      e.to_string()
    })
  })
}

#[tauri::command]
pub fn lookup_wiktionary(
  term: String,
  state: State<'_, WiktState>,
  app_settings: State<'_, AppSettingsState>,
) -> Result<WiktResult, String> {
  let partial = app_settings.0.lock().map_err(|e| e.to_string())?.partial_search;
  state.with_db(|db| {
    db.lookup(&term, partial).map_err(|e| {
      error!("Wiktionary lookup failed for '{}': {}", term, e);
      e.to_string()
    })
  })
}

#[tauri::command]
pub fn create_project(
  name: String,
  files: ProjectFiles,
  data_dir: State<'_, DataDir>,
  state: State<'_, ProjectState>,
) -> Result<ProjectWithEntries, String> {
  info!("Creating project '{}' (jp='{}', en='{}')", name, files.jp, files.en);
  let entries = pair_files(&files)?;
  let (id, proj, path) = project::create_project(&data_dir.0, &name, files).map_err(crate::util::log_err)?;
  info!("Project created with id {}", id);
  state.set(id.clone(), proj.clone(), path)?;
  Ok(ProjectWithEntries {
    id,
    project: proj,
    entries,
  })
}

#[tauri::command]
pub fn open_project(
  id: String,
  data_dir: State<'_, DataDir>,
  state: State<'_, ProjectState>,
) -> Result<ProjectWithEntries, String> {
  info!("Opening project {}", id);
  let (proj, path) = project::open_project(&data_dir.0, &id).map_err(crate::util::log_err)?;
  let entries = pair_files(&proj.files)?;
  state.set(id.clone(), proj.clone(), path)?;
  Ok(ProjectWithEntries {
    id,
    project: proj,
    entries,
  })
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
pub fn list_recent_projects(data_dir: State<'_, DataDir>) -> Result<Vec<RecentProject>, String> {
  Ok(project::list_recent(&data_dir.0))
}

#[tauri::command]
pub fn list_all_projects(data_dir: State<'_, DataDir>) -> Result<Vec<RecentProject>, String> {
  Ok(project::list_all(&data_dir.0))
}

#[tauri::command]
pub fn remove_recent_project(id: String, data_dir: State<'_, DataDir>) -> Result<(), String> {
  project::remove_from_recent(&data_dir.0, &id);
  Ok(())
}

#[tauri::command]
pub fn delete_project(id: String, data_dir: State<'_, DataDir>) -> Result<(), String> {
  info!("Deleting project {}", id);
  project::delete_project(&data_dir.0, &id).map_err(crate::util::log_err)
}

#[tauri::command]
pub fn export_project(dest_path: String, state: State<'_, ProjectState>) -> Result<(), String> {
  state.with_ref(|proj, _| {
    let mut exported = proj.clone();
    exported.files = ProjectFiles {
      jp: String::new(),
      en: String::new(),
    };
    let json = serde_json::to_string(&exported).map_err(|e| format!("serialize error: {}", e))?;
    std::fs::write(&dest_path, &json)
      .map_err(|e| crate::util::log_err(crate::util::friendly_io_msg("", Path::new(&dest_path), &e)))
  })
}

#[tauri::command]
pub fn get_project_info(id: String, data_dir: State<'_, DataDir>) -> Result<ProjectInfo, String> {
  let proj = project::read_project(&data_dir.0, &id).map_err(crate::util::log_err)?;
  Ok(ProjectInfo {
    name: proj.name,
    files: proj.files,
    settings: proj.settings,
  })
}

#[tauri::command]
pub fn update_project(
  id: String,
  name: String,
  files: ProjectFiles,
  settings: ProjectSettings,
  data_dir: State<'_, DataDir>,
  state: State<'_, ProjectState>,
) -> Result<(), String> {
  info!("Updating project {}", id);
  project::update_project(&data_dir.0, &id, &name, files.clone(), settings.clone()).map_err(crate::util::log_err)?;
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
  let content =
    std::fs::read_to_string(path).map_err(|e| crate::util::log_err(crate::util::friendly_io_msg("", path, &e)))?;
  serde_json::from_str::<Project>(&content).map_err(|e| crate::util::log_err(format!("Invalid project file: {}", e)))
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
  source_path: String,
  name: String,
  files: ProjectFiles,
  data_dir: State<'_, DataDir>,
  state: State<'_, ProjectState>,
) -> Result<ProjectWithEntries, String> {
  let imported = read_import_file(&source_path)?;
  let entries = pair_files(&files)?;
  let (id, mut proj, path) = project::create_project(&data_dir.0, &name, files).map_err(crate::util::log_err)?;
  proj.confirmed_lines = imported.confirmed_lines;
  proj.settings = imported.settings;
  project::save_project(&path, &proj).map_err(crate::util::log_err)?;
  let ret = proj.clone();
  state.set(id.clone(), proj, path)?;
  Ok(ProjectWithEntries {
    id,
    project: ret,
    entries,
  })
}

#[tauri::command]
pub fn open_app_dir(app: AppHandle, data_dir: State<'_, DataDir>) -> Result<(), String> {
  app
    .opener()
    .open_path(&*data_dir.0.to_string_lossy(), None::<&str>)
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_app_settings(state: State<'_, AppSettingsState>) -> Result<AppSettings, String> {
  let lock = state.0.lock().map_err(|e| e.to_string())?;
  Ok(lock.clone())
}

#[tauri::command]
pub fn update_app_settings(
  settings: AppSettings,
  data_dir: State<'_, DataDir>,
  state: State<'_, AppSettingsState>,
) -> Result<(), String> {
  crate::settings::save(&data_dir.0, &settings)?;
  let mut lock = state.0.lock().map_err(|e| e.to_string())?;
  *lock = settings;
  Ok(())
}

#[tauri::command]
pub fn get_environment_info(app: AppHandle) -> EnvironmentInfo {
  let pkg = app.package_info();
  EnvironmentInfo {
    app_name: pkg.name.to_string(),
    app_version: pkg.version.to_string(),
    tauri_version: tauri::VERSION.to_string(),
    os: std::env::consts::OS.to_string(),
    arch: std::env::consts::ARCH.to_string(),
    debug: cfg!(debug_assertions),
  }
}
