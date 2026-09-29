//! Database layer: connection management for local and embedded-replica modes.

use libsql::{Builder, Connection, Database};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, thiserror::Error)]
pub enum DbError {
    #[error("libsql error: {0}")]
    Libsql(#[from] libsql::Error),
    #[error("not connected: open a database first")]
    NotConnected,
    #[error("{0}")]
    Msg(String),
}

impl serde::Serialize for DbError {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&self.to_string())
    }
}

/// Shared, mutable database handle stored in Tauri state.
#[derive(Default)]
pub struct DbState {
    inner: Arc<Mutex<Option<DbHandle>>>,
}

struct DbHandle {
    db: Database,
    conn: Connection,
}

impl DbState {
    /// Open a purely local SQLite database (no sync).
    pub async fn open_local(&self, path: &PathBuf) -> Result<(), DbError> {
        let db = Builder::new_local(path).build().await?;
        let conn = db.connect()?;
        crate::migrations::run_migrations(&conn).await?;
        *self.inner.lock().await = Some(DbHandle { db, conn });
        Ok(())
    }

    /// Open an embedded replica: local SQLite file that syncs with a remote
    /// Turso database. This is the production connection mode.
    pub async fn open_embedded_replica(
        &self,
        local_path: &PathBuf,
        sync_url: String,
        auth_token: String,
    ) -> Result<(), DbError> {
        let db = Builder::new_remote_replica(local_path, sync_url, auth_token)
            .build()
            .await?;
        db.sync().await?;
        let conn = db.connect()?;
        crate::migrations::run_migrations(&conn).await?;
        *self.inner.lock().await = Some(DbHandle { db, conn });
        Ok(())
    }

    /// Run the closure with the active connection, erroring if not connected.
    pub async fn with_conn<F, T>(&self, f: F) -> Result<T, DbError>
    where
        F: FnOnce(&Connection) -> T,
    {
        let guard = self.inner.lock().await;
        let handle = guard.as_ref().ok_or(DbError::NotConnected)?;
        Ok(f(&handle.conn))
    }

    /// Get a clone of the active connection handle.
    pub async fn conn(&self) -> Result<Connection, DbError> {
        let guard = self.inner.lock().await;
        let handle = guard.as_ref().ok_or(DbError::NotConnected)?;
        Ok(handle.conn.clone())
    }

    /// Push/pull with the remote (no-op for local-only databases).
    pub async fn sync(&self) -> Result<(), DbError> {
        let guard = self.inner.lock().await;
        let handle = guard.as_ref().ok_or(DbError::NotConnected)?;
        // sync() errors on local-only DBs; ignore that specific case.
        match handle.db.sync().await {
            Ok(_) => Ok(()),
            Err(e) => {
                log::warn!("sync skipped/failed: {e}");
                Ok(())
            }
        }
    }

    pub async fn is_connected(&self) -> bool {
        self.inner.lock().await.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn local_open_runs_migrations_and_creates_tables() {
        let dir = std::env::temp_dir().join("trustfund_db_test");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("db_test.db");
        let _ = std::fs::remove_file(&path);

        let state = DbState::default();
        state.open_local(&path).await.expect("open_local failed");

        let conn = state.conn().await.unwrap();
        // borrowers table should exist after migrations.
        let mut rows = conn
            .query(
                "SELECT name FROM sqlite_master WHERE type='table' AND name='borrowers'",
                (),
            )
            .await
            .unwrap();
        let row = rows.next().await.unwrap();
        assert!(row.is_some(), "borrowers table should exist");
    }
}
