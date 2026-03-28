#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod core;
mod dictionary;
mod logging;
mod project;
mod strings;
mod util;
mod wiktionary;

use std::sync::Mutex;

use log::{error, info, warn};
use tauri::Manager;

const RES_JMDICT: &str = "resources/jmdict.sqlite";
const RES_KANJIDIC: &str = "resources/kanjidic2.sqlite";
const RES_IPADIC: &str = "resources/ipadic-mecab-v270.dict";

fn main() {
  tauri::Builder::default()
    .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
      if let Some(w) = app.get_webview_window("main") {
        let _ = w.unminimize();
        let _ = w.set_focus();
      }
    }))
    .plugin(tauri_plugin_dialog::init())
    .plugin(tauri_plugin_opener::init())
    .plugin(tauri_plugin_window_state::Builder::new().build())
    .manage(commands::DictState(Mutex::new(None)))
    .manage(commands::ProjectState(Mutex::new(None)))
    .manage(commands::WiktState(Mutex::new(None)))
    .invoke_handler(tauri::generate_handler![
      commands::save_translation,
      commands::lookup_word,
      commands::lookup_kanji,
      commands::create_project,
      commands::open_project,
      commands::save_project,
      commands::confirm_line,
      commands::unconfirm_line,
      commands::list_recent_projects,
      commands::list_all_projects,
      commands::remove_recent_project,
      commands::delete_project,
      commands::export_project,
      commands::get_project_info,
      commands::update_project,
      commands::preview_import,
      commands::import_project,
      commands::open_app_dir,
      commands::lookup_wiktionary,
      commands::clear_wiktionary_cache,
    ])
    .setup(|app| {
      let config_dir = app.path().app_config_dir()?;
      logging::init(&config_dir);
      info!("Starting...");

      let resource_dir = app.path().resource_dir()?;
      let jmdict_path = resource_dir.join(RES_JMDICT);
      let kanjidic_path = resource_dir.join(RES_KANJIDIC);
      let ipadic_path = resource_dir.join(RES_IPADIC);

      if jmdict_path.exists() && kanjidic_path.exists() && ipadic_path.exists() {
        match dictionary::DictDb::open(&jmdict_path, &kanjidic_path, &ipadic_path) {
          Ok(db) => {
            let state = app.state::<commands::DictState>();
            *state.0.lock().unwrap() = Some(db);
            info!("Dictionary loaded from bundled resources");
          }
          Err(e) => {
            error!("Failed to load dictionary: {}", e);
          }
        }
      } else {
        warn!("Dictionary files not found at {:?}, {:?}", jmdict_path, kanjidic_path);
      }

      let cache_dir = app.path().app_cache_dir()?;
      match wiktionary::WiktCache::open(&cache_dir) {
        Ok(cache) => {
          let state = app.state::<commands::WiktState>();
          *state.0.lock().unwrap() = Some(cache);
          info!("Wiktionary cache initialized");
        }
        Err(e) => {
          error!("Failed to initialize Wiktionary cache: {}", e);
        }
      }

      Ok(())
    })
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
