//! Data store: two backends, both **stateless in memory** — each operation
//! loads only what it needs and writes back, so we never hold the whole dataset
//! resident. This avoids memory growth and (for Turso) cross-device staleness.
//!
//! - JSON backend (offline): the dataset lives in a local file. For this app's
//!   scale (personal lending) a file read/write per operation is cheap. The
//!   file is the source of truth; we do not cache it between calls.
//! - Turso backend (remote): real per-row tables accessed over HTTP; queries
//!   fetch only the rows needed (e.g. one loan's payments), not everything.
//!
//! No native SQLite / libSQL — pure Rust, identical across all OSes.

use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::turso::TursoClient;

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("not connected: open a data store first")]
    NotConnected,
    #[error("io error: {0}")]
    Io(String),
    #[error("serialization error: {0}")]
    Serde(String),
    #[error("turso error: {0}")]
    Turso(String),
}

impl serde::Serialize for StoreError {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&self.to_string())
    }
}

/// Which backend is active. Holds only the handle/location — not the data.
pub enum Backend {
    Json { path: PathBuf },
    Turso { client: TursoClient },
}

/// Shared store configuration in Tauri state. No dataset is cached here.
#[derive(Default)]
pub struct StoreState {
    inner: Arc<Mutex<Option<Backend>>>,
}

impl StoreState {
    pub async fn open_json(&self, path: PathBuf) -> Result<(), StoreError> {
        // Touch/validate the file lazily; just record the backend.
        *self.inner.lock().await = Some(Backend::Json { path });
        Ok(())
    }

    pub async fn open_turso(&self, url: String, token: String) -> Result<(), StoreError> {
        let client = TursoClient::new(url, token);
        client.ensure_schema().await.map_err(StoreError::Turso)?;
        *self.inner.lock().await = Some(Backend::Turso { client });
        Ok(())
    }

    pub async fn is_connected(&self) -> bool {
        self.inner.lock().await.is_some()
    }

    pub async fn is_remote(&self) -> bool {
        matches!(
            self.inner.lock().await.as_ref(),
            Some(Backend::Turso { .. })
        )
    }

    /// Run an async operation with access to the active backend.
    pub async fn with_backend<F, Fut, T>(&self, f: F) -> Result<T, StoreError>
    where
        F: FnOnce(BackendRef) -> Fut,
        Fut: std::future::Future<Output = Result<T, StoreError>>,
    {
        let guard = self.inner.lock().await;
        match guard.as_ref() {
            Some(Backend::Json { path }) => f(BackendRef::Json(path.clone())).await,
            Some(Backend::Turso { client }) => f(BackendRef::Turso(client.clone())).await,
            None => Err(StoreError::NotConnected),
        }
    }
}

/// A lightweight reference to the active backend passed into operations.
pub enum BackendRef {
    Json(PathBuf),
    Turso(TursoClient),
}
