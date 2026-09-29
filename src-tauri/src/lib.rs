mod db;

use tauri::Manager;

/// Spike command: opens a local libSQL DB in the app data dir and runs a
/// create/insert/read round-trip. Proves libsql works from a Tauri command.
#[tauri::command]
async fn db_spike(app: tauri::AppHandle) -> Result<String, String> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("app_data_dir: {e}"))?;
    std::fs::create_dir_all(&dir).map_err(|e| format!("create dir: {e}"))?;
    let path = dir.join("spike.db");

    let conn = db::open_local(&path)
        .await
        .map_err(|e| format!("open_local: {e}"))?;
    let value = db::smoke_test(&conn)
        .await
        .map_err(|e| format!("smoke_test: {e}"))?;
    Ok(format!("local libSQL round-trip returned: '{value}'"))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
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
        .invoke_handler(tauri::generate_handler![db_spike])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
