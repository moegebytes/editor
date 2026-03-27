#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
pub mod core;
pub mod dictionary;
mod project;
pub mod strings;

use std::sync::Mutex;

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
    .plugin(tauri_plugin_window_state::Builder::new().build())
    .manage(commands::DictState(Mutex::new(None)))
    .manage(commands::ProjectState(Mutex::new(None)))
    .invoke_handler(tauri::generate_handler![
      commands::save_en_file,
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
      commands::rename_project,
      commands::preview_import,
      commands::import_project,
      commands::update_project_settings,
    ])
    .setup(|app| {
      let resource_dir = app.path().resource_dir()?;
      let jmdict_path = resource_dir.join(RES_JMDICT);
      let kanjidic_path = resource_dir.join(RES_KANJIDIC);
      let ipadic_path = resource_dir.join(RES_IPADIC);

      if jmdict_path.exists() && kanjidic_path.exists() && ipadic_path.exists() {
        match dictionary::DictDb::open(&jmdict_path, &kanjidic_path, &ipadic_path) {
          Ok(db) => {
            let state = app.state::<commands::DictState>();
            *state.0.lock().unwrap() = Some(db);
            eprintln!("Dictionary loaded from bundled resources");
          }
          Err(e) => {
            eprintln!("Failed to load dictionary: {}", e);
          }
        }
      } else {
        eprintln!("Dictionary files not found at {:?}, {:?}", jmdict_path, kanjidic_path);
      }

      Ok(())
    })
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
