//! Secure-ish credential storage for the user's Turso DB URL + auth token.
//!
//! Uses the Tauri Store plugin (persisted to the app data dir). The token only
//! scopes to the user's own database, so on-device storage is acceptable
//! (blast radius = their own data), per docs/architecture.md.

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Wry};
use tauri_plugin_store::StoreExt;

const STORE_FILE: &str = "credentials.json";
const KEY: &str = "turso";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TursoCredentials {
    pub sync_url: String,
    pub auth_token: String,
}

/// Persist credentials to the store.
pub fn save(app: &AppHandle<Wry>, creds: &TursoCredentials) -> Result<(), String> {
    let store = app.store(STORE_FILE).map_err(|e| e.to_string())?;
    let value = serde_json::to_value(creds).map_err(|e| e.to_string())?;
    store.set(KEY, value);
    store.save().map_err(|e| e.to_string())?;
    Ok(())
}

/// Load credentials, if present.
pub fn load(app: &AppHandle<Wry>) -> Result<Option<TursoCredentials>, String> {
    let store = app.store(STORE_FILE).map_err(|e| e.to_string())?;
    match store.get(KEY) {
        Some(value) => {
            let creds: TursoCredentials =
                serde_json::from_value(value).map_err(|e| e.to_string())?;
            Ok(Some(creds))
        }
        None => Ok(None),
    }
}

/// Remove stored credentials (disconnect / reset).
pub fn clear(app: &AppHandle<Wry>) -> Result<(), String> {
    let store = app.store(STORE_FILE).map_err(|e| e.to_string())?;
    store.delete(KEY);
    store.save().map_err(|e| e.to_string())?;
    Ok(())
}
