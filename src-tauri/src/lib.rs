mod db;
mod migrations;

use db::DbState;
use tauri::Manager;

/// Connect to a local-only database in the app data dir (dev/offline fallback).
#[tauri::command]
async fn db_connect_local(
    app: tauri::AppHandle,
    state: tauri::State<'_, DbState>,
) -> Result<(), String> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("app_data_dir: {e}"))?;
    std::fs::create_dir_all(&dir).map_err(|e| format!("create dir: {e}"))?;
    let path = dir.join("trustfund.db");
    state.open_local(&path).await.map_err(|e| e.to_string())
}

/// Returns whether a database connection is currently open.
#[tauri::command]
async fn db_is_connected(state: tauri::State<'_, DbState>) -> Result<bool, String> {
    Ok(state.is_connected().await)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(DbState::default())
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![db_connect_local, db_is_connected])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
