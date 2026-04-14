use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use log::{debug, error, info};
use serde::Serialize;
use tauri::{AppHandle, State};
use tauri_plugin_opener::OpenerExt;

use crate::core::FlatEntry;
use crate::jmdict::{JmdictDb, LookupResult};
use crate::kanjidic::{KanjiDb, KanjiEntry};
use crate::project::{self, GlossaryEntry, Project, ProjectFiles, ProjectSettings, RecentProject};
use crate::recovery::{RecoveryData, RecoveryEntry, RecoveryInfo};
use crate::settings::AppSettings;
use crate::wiktionary::{WiktDb, WiktResult};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EnvironmentInfo {
  pub app_name: String,
  pub app_version: String,
  pub tauri_version: String,
  pub webview_version: String,
  pub os: String,
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
  debug!("Saving translation ({} entries)", entries.len());

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
  debug!("Looking up JMdict '{}' (partial={})", query, partial);

  state.with_db(|db| {
    db.lookup(&query, partial).map_err(|e| {
      error!("Dictionary lookup failed for '{}': {}", query, e);
      e.to_string()
    })
  })
}

#[tauri::command]
pub fn lookup_kanji(ch: String, state: State<'_, KanjidicState>) -> Result<Option<KanjiEntry>, String> {
  debug!("Looking up kanji '{}'", ch);

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
  debug!("Looking up Wiktionary '{}' (partial={})", term, partial);

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
pub fn close_project(state: State<'_, ProjectState>) -> Result<(), String> {
  let mut lock = state.0.lock().map_err(|e| e.to_string())?;
  if let Some(open) = lock.take() {
    info!("Closing project {}", open.id);
  }
  Ok(())
}

#[tauri::command]
pub fn confirm_line(index: usize, state: State<'_, ProjectState>) -> Result<(), String> {
  debug!("Confirming line {}", index);
  state.with_mut(|proj, _| {
    proj.confirmed_lines.insert(index);
    Ok(())
  })
}

#[tauri::command]
pub fn unconfirm_line(index: usize, state: State<'_, ProjectState>) -> Result<(), String> {
  debug!("Unconfirming line {}", index);
  state.with_mut(|proj, _| {
    proj.confirmed_lines.remove(&index);
    Ok(())
  })
}

#[tauri::command]
pub fn list_recent_projects(data_dir: State<'_, DataDir>) -> Result<Vec<RecentProject>, String> {
  debug!("Listing recent projects");
  Ok(project::list_recent(&data_dir.0))
}

#[tauri::command]
pub fn list_all_projects(data_dir: State<'_, DataDir>) -> Result<Vec<RecentProject>, String> {
  debug!("Listing all projects");
  Ok(project::list_all(&data_dir.0))
}

#[tauri::command]
pub fn remove_recent_project(id: String, data_dir: State<'_, DataDir>) -> Result<(), String> {
  debug!("Removing recent project {}", id);
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
  debug!("Exporting project to '{}'", dest_path);

  state.with_ref(|proj, _| {
    let dest = Path::new(&dest_path);
    let file = std::fs::File::create(dest)
      .map_err(|e| crate::util::log_err(crate::util::friendly_io_msg("Export file", dest, &e)))?;
    let mut zip = zip::ZipWriter::new(file);
    let options = zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    let mut exported = proj.clone();
    exported.files = ProjectFiles {
      jp: String::new(),
      en: String::new(),
    };
    let json = serde_json::to_string(&exported).map_err(|e| format!("serialize error: {}", e))?;
    zip
      .start_file("project.json", options)
      .map_err(|e| crate::util::log_err(format!("Could not write to ZIP archive: {e}.")))?;
    std::io::Write::write_all(&mut zip, json.as_bytes())
      .map_err(|e| crate::util::log_err(format!("Could not write to ZIP archive: {e}.")))?;

    for (prefix, top_path) in [("jp", &proj.files.jp), ("en", &proj.files.en)] {
      let top = Path::new(top_path);
      let root = top.parent().unwrap_or(Path::new("."));
      let entries =
        crate::strings::parse_strings(top).map_err(|e| crate::util::log_err(map_strings_err(&e, prefix)))?;
      let mut paths = vec![top.to_path_buf()];
      paths.extend(crate::strings::collect_file_paths(&entries, root));

      for path in &paths {
        let relative = path
          .strip_prefix(root)
          .unwrap_or(path)
          .to_string_lossy()
          .replace('\\', "/");
        let zip_path = format!("{prefix}/{relative}");
        zip
          .start_file(&zip_path, options)
          .map_err(|e| crate::util::log_err(format!("Could not write to ZIP archive: {e}.")))?;
        let content = std::fs::read(path)
          .map_err(|e| crate::util::log_err(crate::util::friendly_io_msg("Strings file", path, &e)))?;
        std::io::Write::write_all(&mut zip, &content)
          .map_err(|e| crate::util::log_err(format!("Could not write to ZIP archive: {e}.")))?;
      }
    }

    zip
      .finish()
      .map_err(|e| crate::util::log_err(format!("Could not finalize ZIP archive: {e}.")))?;
    Ok(())
  })
}

#[tauri::command]
pub fn get_project_info(id: String, data_dir: State<'_, DataDir>) -> Result<ProjectInfo, String> {
  debug!("Getting project info for {}", id);

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
  debug!("Previewing import from '{}'", source_path);

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
  debug!("Importing project '{}' from '{}'", name, source_path);

  let imported = read_import_file(&source_path)?;
  let entries = pair_files(&files)?;
  let (id, new_proj, path) = project::create_project(&data_dir.0, &name, files).map_err(crate::util::log_err)?;
  let proj = Project {
    name: new_proj.name,
    files: new_proj.files,
    ..imported
  };
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
  debug!("Opening app directory");
  app
    .opener()
    .open_path(&*data_dir.0.to_string_lossy(), None::<&str>)
    .map_err(|e| e.to_string())
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettingsResponse {
  #[serde(flatten)]
  pub settings: AppSettings,
  pub min_auto_save_interval_secs: u32,
}

#[tauri::command]
pub fn get_app_settings(state: State<'_, AppSettingsState>) -> Result<AppSettingsResponse, String> {
  debug!("Getting app settings");
  let lock = state.0.lock().map_err(|e| e.to_string())?;
  Ok(AppSettingsResponse {
    settings: lock.clone(),
    min_auto_save_interval_secs: crate::settings::MIN_AUTO_SAVE_INTERVAL,
  })
}

#[tauri::command]
pub fn update_app_settings(
  settings: AppSettings,
  data_dir: State<'_, DataDir>,
  state: State<'_, AppSettingsState>,
) -> Result<(), String> {
  debug!("Updating app settings");

  crate::settings::save(&data_dir.0, &settings)?;
  let mut lock = state.0.lock().map_err(|e| e.to_string())?;
  *lock = settings;
  Ok(())
}

#[tauri::command]
pub fn update_glossary(glossary: Vec<GlossaryEntry>, state: State<'_, ProjectState>) -> Result<(), String> {
  debug!("Updating glossary ({} entries)", glossary.len());
  state.with_mut(|proj, path| {
    proj.glossary = glossary;
    project::save_project(path, proj).map_err(crate::util::log_err)
  })
}

#[tauri::command]
pub fn log_error(message: String, stack: Option<String>) {
  match stack {
    Some(ref s) => error!("(WebView) {}\n {}", message, s),
    None => error!("(WebView) {}", message),
  }
}

#[tauri::command]
pub fn write_recovery(
  entries: BTreeMap<usize, RecoveryEntry>,
  data_dir: State<'_, DataDir>,
  state: State<'_, ProjectState>,
) -> Result<(), String> {
  let lock = state.0.lock().map_err(|e| e.to_string())?;
  let open = lock.as_ref().ok_or("No project open")?;
  crate::recovery::write(&data_dir.0, &open.id, &entries, &open.project.confirmed_lines).map_err(crate::util::log_err)
}

#[tauri::command]
pub fn check_recovery(id: String, data_dir: State<'_, DataDir>) -> Result<Option<RecoveryInfo>, String> {
  debug!("Checking recovery for project {}", id);
  Ok(crate::recovery::check(&data_dir.0, &id))
}

#[tauri::command]
pub fn load_recovery(id: String, data_dir: State<'_, DataDir>) -> Result<RecoveryData, String> {
  info!("Loading recovery data for project {}", id);
  crate::recovery::load(&data_dir.0, &id).map_err(crate::util::log_err)
}

#[tauri::command]
pub fn delete_recovery(id: String, data_dir: State<'_, DataDir>) -> Result<(), String> {
  debug!("Deleting recovery for project {}", id);
  crate::recovery::delete(&data_dir.0, &id);
  Ok(())
}

#[tauri::command]
pub fn get_environment_info(app: AppHandle) -> EnvironmentInfo {
  debug!("Getting environment info");

  let pkg = app.package_info();
  EnvironmentInfo {
    app_name: pkg.name.to_string(),
    app_version: pkg.version.to_string(),
    tauri_version: tauri::VERSION.to_string(),
    webview_version: tauri::webview_version().unwrap_or_default(),
    os: os_info::get().to_string(),
    debug: cfg!(debug_assertions),
  }
}
