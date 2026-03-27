use std::path::{Path, PathBuf};
use std::sync::Mutex;

use serde::Serialize;

use crate::dictionary::{DictDb, KanjiEntry, LookupResult};
use crate::core::FlatEntry;
use tauri::{AppHandle, Manager, State};

use crate::project::{self, Project, ProjectFiles, ProjectSettings, RecentProject};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectWithEntries {
  #[serde(flatten)]
  pub project: Project,
  pub entries: Vec<FlatEntry>,
}

pub struct DictState(pub Mutex<Option<DictDb>>);
pub struct ProjectState(pub Mutex<Option<(Project, PathBuf)>>);

impl DictState {
  fn with_db<T>(
    &self,
    f: impl FnOnce(&DictDb) -> Result<T, String>,
  ) -> Result<T, String> {
    let lock = self.0.lock().map_err(|e| e.to_string())?;
    let db = lock.as_ref().ok_or("Dictionary not initialized")?;
    f(db)
  }
}

impl ProjectState {
  fn with_ref<T>(
    &self,
    f: impl FnOnce(&Project, &Path) -> Result<T, String>,
  ) -> Result<T, String> {
    let lock = self.0.lock().map_err(|e| e.to_string())?;
    let (proj, path) = lock.as_ref().ok_or("No project open")?;
    f(proj, path)
  }

  fn with_mut<T>(
    &self,
    f: impl FnOnce(&mut Project, &Path) -> Result<T, String>,
  ) -> Result<T, String> {
    let mut lock = self.0.lock().map_err(|e| e.to_string())?;
    let (proj, path) = lock.as_mut().ok_or("No project open")?;
    f(proj, path)
  }

  fn set(&self, proj: Project, path: PathBuf) -> Result<(), String> {
    let mut lock = self.0.lock().map_err(|e| e.to_string())?;
    *lock = Some((proj, path));
    Ok(())
  }
}

fn pair_files(files: &ProjectFiles) -> Result<Vec<FlatEntry>, String> {
  let jp_path = Path::new(&files.jp);
  let en_path = Path::new(&files.en);
  if !jp_path.exists() {
    return Err(format!("Japanese file not found: {}", files.jp));
  }
  if !en_path.exists() {
    return Err(format!("English file not found: {}", files.en));
  }
  let jp = crate::strings::parse_strings(jp_path)
    .map_err(|e| format!("JP file: {}", e))?;
  let en = crate::strings::parse_strings(en_path)
    .map_err(|e| format!("EN file: {}", e))?;
  Ok(crate::core::pair_files(&jp, &en))
}

#[tauri::command]
pub fn save_en_file(
  entries: Vec<FlatEntry>,
  state: State<'_, ProjectState>,
) -> Result<(), String> {
  let path = state.with_ref(|proj, _| Ok(proj.files.en.clone()))?;
  let reconstructed = crate::core::reconstruct_en_entries(&entries);
  crate::strings::write_strings(&reconstructed, Path::new(&path))
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn lookup_word(
  query: String,
  state: State<'_, DictState>,
) -> Result<LookupResult, String> {
  state.with_db(|db| db.lookup_word(&query).map_err(|e| e.to_string()))
}

#[tauri::command]
pub fn lookup_kanji(
  ch: String,
  state: State<'_, DictState>,
) -> Result<Option<KanjiEntry>, String> {
  let c = ch.chars().next().ok_or("Empty character")?;
  state.with_db(|db| db.lookup_kanji(c).map_err(|e| e.to_string()))
}

fn app_data_dir(app: &AppHandle) -> Result<PathBuf, String> {
  app.path().app_data_dir().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_project(
  app: AppHandle,
  name: String,
  files: ProjectFiles,
  state: State<'_, ProjectState>,
) -> Result<ProjectWithEntries, String> {
  let entries = pair_files(&files)?;
  let data_dir = app_data_dir(&app)?;
  let (proj, path) = project::create_project(&data_dir, &name, files)?;
  let ret = proj.clone();
  state.set(proj, path)?;
  Ok(ProjectWithEntries { project: ret, entries })
}

#[tauri::command]
pub fn open_project(
  app: AppHandle,
  id: String,
  state: State<'_, ProjectState>,
) -> Result<ProjectWithEntries, String> {
  let data_dir = app_data_dir(&app)?;
  let (proj, path) = project::open_project(&data_dir, &id)?;
  let entries = pair_files(&proj.files)?;
  let ret = proj.clone();
  state.set(proj, path)?;
  Ok(ProjectWithEntries { project: ret, entries })
}

#[tauri::command]
pub fn save_project(state: State<'_, ProjectState>) -> Result<(), String> {
  state.with_ref(|proj, path| project::save_project(path, proj))
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
  let data_dir = app_data_dir(&app)?;
  Ok(project::list_recent(&data_dir))
}

#[tauri::command]
pub fn list_all_projects(app: AppHandle) -> Result<Vec<RecentProject>, String> {
  let data_dir = app_data_dir(&app)?;
  Ok(project::list_all(&data_dir))
}

#[tauri::command]
pub fn remove_recent_project(app: AppHandle, id: String) -> Result<(), String> {
  let data_dir = app_data_dir(&app)?;
  project::remove_from_recent(&data_dir, &id);
  Ok(())
}

#[tauri::command]
pub fn delete_project(app: AppHandle, id: String) -> Result<(), String> {
  let data_dir = app_data_dir(&app)?;
  project::delete_project(&data_dir, &id)
}

#[tauri::command]
pub fn export_project(
  dest_path: String,
  state: State<'_, ProjectState>,
) -> Result<(), String> {
  state.with_ref(|proj, _| {
    let mut exported = proj.clone();
    exported.files = ProjectFiles {
      jp: String::new(),
      en: String::new(),
    };
    let json =
      serde_json::to_string(&exported).map_err(|e| format!("serialize error: {}", e))?;
    std::fs::write(&dest_path, &json)
      .map_err(|e| crate::project::friendly_io_msg(&e, Path::new(&dest_path)))
  })
}

#[tauri::command]
pub fn rename_project(
  name: String,
  state: State<'_, ProjectState>,
) -> Result<(), String> {
  state.with_mut(|proj, path| {
    proj.name = name;
    project::save_project(path, proj)
  })
}

#[tauri::command]
pub fn update_project_settings(
  settings: ProjectSettings,
  state: State<'_, ProjectState>,
) -> Result<(), String> {
  state.with_mut(|proj, path| {
    proj.settings = settings;
    project::save_project(path, proj)
  })
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
    .map_err(|e| project::friendly_io_msg(&e, path))?;
  serde_json::from_str::<Project>(&content)
    .map_err(|e| format!("Invalid project file: {}", e))
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
  let data_dir = app_data_dir(&app)?;
  let (mut proj, path) = project::create_project(&data_dir, &name, files)?;
  proj.confirmed_lines = imported.confirmed_lines;
  proj.settings = imported.settings;
  project::save_project(&path, &proj)?;
  let ret = proj.clone();
  state.set(proj, path)?;
  Ok(ProjectWithEntries { project: ret, entries })
}
