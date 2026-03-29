#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod core;
mod jmdict;
mod kanjidic;
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
const RES_WIKTIONARY: &str = "resources/wiktionary.sqlite";

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
    .manage(commands::JmdictState(Mutex::new(None)))
    .manage(commands::KanjidicState(Mutex::new(None)))
    .manage(commands::ProjectState(Mutex::new(None)))
    .manage(commands::WiktState(Mutex::new(None)))
    .invoke_handler(tauri::generate_handler![
      commands::save_translation,
      commands::lookup_jmdict,
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
      commands::get_environment_info,
    ])
    .setup(|app| {
      let config_dir = app.path().app_config_dir()?;
      logging::init(&config_dir);
      info!("Starting...");

      let resource_dir = app.path().resource_dir()?;
      let jmdict_path = resource_dir.join(RES_JMDICT);
      let ipadic_path = resource_dir.join(RES_IPADIC);

      if jmdict_path.exists() && ipadic_path.exists() {
        match jmdict::JmdictDb::open(&jmdict_path, &ipadic_path) {
          Ok(db) => {
            let state = app.state::<commands::JmdictState>();
            *state.0.lock().expect("JMdict state lock poisoned") = Some(db);
            info!("JMdict loaded from bundled resources");
          }
          Err(e) => {
            error!("Failed to load JMdict: {}", e);
          }
        }
      } else {
        warn!("JMdict files not found at {:?}", jmdict_path);
      }

      let kanjidic_path = resource_dir.join(RES_KANJIDIC);
      if kanjidic_path.exists() {
        match kanjidic::KanjiDb::open(&kanjidic_path) {
          Ok(db) => {
            let state = app.state::<commands::KanjidicState>();
            *state.0.lock().expect("KANJIDIC2 state lock poisoned") = Some(db);
            info!("KANJIDIC2 loaded from bundled resources");
          }
          Err(e) => {
            error!("Failed to load KANJIDIC2: {}", e);
          }
        }
      } else {
        warn!("KANJIDIC2 database not found at {:?}", kanjidic_path);
      }

      let wikt_path = resource_dir.join(RES_WIKTIONARY);
      if wikt_path.exists() {
        match wiktionary::WiktDb::open(&wikt_path) {
          Ok(db) => {
            let state = app.state::<commands::WiktState>();
            *state.0.lock().expect("Wiktionary state lock poisoned") = Some(db);
            info!("Wiktionary dictionary loaded from bundled resources");
          }
          Err(e) => {
            error!("Failed to load Wiktionary dictionary: {}", e);
          }
        }
      } else {
        warn!("Wiktionary database not found at {:?}", wikt_path);
      }

      Ok(())
    })
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
