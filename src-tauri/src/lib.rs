mod credentials;
mod db;
mod migrations;
mod models;
mod repo_borrowers;
mod util;

use credentials::TursoCredentials;
use db::DbState;
use models::{Borrower, BorrowerInput};
use tauri::Manager;

fn app_local_db_path(app: &tauri::AppHandle) -> Result<std::path::PathBuf, String> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("app_data_dir: {e}"))?;
    std::fs::create_dir_all(&dir).map_err(|e| format!("create dir: {e}"))?;
    Ok(dir.join("trustfund.db"))
}

/// Connect to a local-only database in the app data dir (dev/offline fallback).
#[tauri::command]
async fn db_connect_local(
    app: tauri::AppHandle,
    state: tauri::State<'_, DbState>,
) -> Result<(), String> {
    let path = app_local_db_path(&app)?;
    state.open_local(&path).await.map_err(|e| e.to_string())
}

/// Validate + connect using Turso credentials, then persist them on success.
#[tauri::command]
async fn db_connect_turso(
    app: tauri::AppHandle,
    state: tauri::State<'_, DbState>,
    sync_url: String,
    auth_token: String,
) -> Result<(), String> {
    let path = app_local_db_path(&app)?;
    state
        .open_embedded_replica(&path, sync_url.clone(), auth_token.clone())
        .await
        .map_err(|e| e.to_string())?;
    // Persist only after a successful connect.
    credentials::save(&app, &TursoCredentials { sync_url, auth_token })?;
    Ok(())
}

/// On startup: if credentials exist, connect with them; else report not-configured.
#[tauri::command]
async fn db_connect_saved(
    app: tauri::AppHandle,
    state: tauri::State<'_, DbState>,
) -> Result<bool, String> {
    match credentials::load(&app)? {
        Some(creds) => {
            let path = app_local_db_path(&app)?;
            state
                .open_embedded_replica(&path, creds.sync_url, creds.auth_token)
                .await
                .map_err(|e| e.to_string())?;
            Ok(true)
        }
        None => Ok(false),
    }
}

/// Whether credentials are stored.
#[tauri::command]
fn db_has_credentials(app: tauri::AppHandle) -> Result<bool, String> {
    Ok(credentials::load(&app)?.is_some())
}

/// Forget credentials (disconnect on next launch).
#[tauri::command]
fn db_clear_credentials(app: tauri::AppHandle) -> Result<(), String> {
    credentials::clear(&app)
}

/// Returns whether a database connection is currently open.
#[tauri::command]
async fn db_is_connected(state: tauri::State<'_, DbState>) -> Result<bool, String> {
    Ok(state.is_connected().await)
}

/// Trigger a manual sync with the remote (no-op for local-only).
#[tauri::command]
async fn db_sync(state: tauri::State<'_, DbState>) -> Result<(), String> {
    state.sync().await.map_err(|e| e.to_string())
}

// ---- Borrowers ----

#[tauri::command]
async fn borrowers_list(
    state: tauri::State<'_, DbState>,
    search: Option<String>,
) -> Result<Vec<Borrower>, String> {
    let conn = state.conn().await.map_err(|e| e.to_string())?;
    repo_borrowers::list(&conn, search)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn borrower_get(
    state: tauri::State<'_, DbState>,
    id: String,
) -> Result<Option<Borrower>, String> {
    let conn = state.conn().await.map_err(|e| e.to_string())?;
    repo_borrowers::get(&conn, &id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn borrower_create(
    state: tauri::State<'_, DbState>,
    input: BorrowerInput,
) -> Result<Borrower, String> {
    let conn = state.conn().await.map_err(|e| e.to_string())?;
    let result = repo_borrowers::create(&conn, input)
        .await
        .map_err(|e| e.to_string())?;
    let _ = state.sync().await;
    Ok(result)
}

#[tauri::command]
async fn borrower_update(
    state: tauri::State<'_, DbState>,
    id: String,
    input: BorrowerInput,
) -> Result<Borrower, String> {
    let conn = state.conn().await.map_err(|e| e.to_string())?;
    let result = repo_borrowers::update(&conn, &id, input)
        .await
        .map_err(|e| e.to_string())?;
    let _ = state.sync().await;
    Ok(result)
}

#[tauri::command]
async fn borrower_delete(state: tauri::State<'_, DbState>, id: String) -> Result<(), String> {
    let conn = state.conn().await.map_err(|e| e.to_string())?;
    repo_borrowers::soft_delete(&conn, &id)
        .await
        .map_err(|e| e.to_string())?;
    let _ = state.sync().await;
    Ok(())
}

#[tauri::command]
async fn borrower_total_outstanding(
    state: tauri::State<'_, DbState>,
    id: String,
) -> Result<f64, String> {
    let conn = state.conn().await.map_err(|e| e.to_string())?;
    repo_borrowers::total_outstanding(&conn, &id)
        .await
        .map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::default().build())
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
        .invoke_handler(tauri::generate_handler![
            db_connect_local,
            db_connect_turso,
            db_connect_saved,
            db_has_credentials,
            db_clear_credentials,
            db_is_connected,
            db_sync,
            borrowers_list,
            borrower_get,
            borrower_create,
            borrower_update,
            borrower_delete,
            borrower_total_outstanding,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
