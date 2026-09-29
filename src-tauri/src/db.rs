//! Database spike module.
//!
//! Purpose: validate that the `libsql` Rust crate compiles and runs inside the
//! Tauri core, and that both a local SQLite connection and an embedded replica
//! (local file synced to a Turso cloud DB) work.
//!
//! This is intentionally minimal — the real data layer (migrations, models,
//! commands) is built in Phase 1.

use libsql::{Builder, Connection};
use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum DbError {
    #[error("libsql error: {0}")]
    Libsql(#[from] libsql::Error),
    #[error("{0}")]
    Msg(String),
}

/// Open a purely local SQLite database (no sync). Used to prove libsql works
/// on the host platform without requiring Turso credentials.
pub async fn open_local(path: &PathBuf) -> Result<Connection, DbError> {
    let db = Builder::new_local(path).build().await?;
    let conn = db.connect()?;
    Ok(conn)
}

/// Open an embedded replica: a local SQLite file that syncs with a remote
/// Turso database. This is the production connection mode.
pub async fn open_embedded_replica(
    local_path: &PathBuf,
    sync_url: String,
    auth_token: String,
) -> Result<Connection, DbError> {
    let db = Builder::new_remote_replica(local_path, sync_url, auth_token)
        .build()
        .await?;
    // Pull any remote changes on connect.
    db.sync().await?;
    let conn = db.connect()?;
    Ok(conn)
}

/// Run a trivial round-trip (create table, insert, read back) to prove the
/// connection actually executes SQL. Returns the read-back value.
pub async fn smoke_test(conn: &Connection) -> Result<String, DbError> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS _spike (id INTEGER PRIMARY KEY, note TEXT)",
        (),
    )
    .await?;
    conn.execute("DELETE FROM _spike", ()).await?;
    conn.execute("INSERT INTO _spike (id, note) VALUES (1, ?1)", ["libsql ok"])
        .await?;

    let mut rows = conn
        .query("SELECT note FROM _spike WHERE id = 1", ())
        .await?;
    if let Some(row) = rows.next().await? {
        let note: String = row.get(0)?;
        Ok(note)
    } else {
        Err(DbError::Msg("no row returned".into()))
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn local_smoke_test_round_trips() {
        let dir = std::env::temp_dir().join("trustfund_spike_test");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("spike_test.db");
        // Clean any previous run.
        let _ = std::fs::remove_file(&path);

        let conn = open_local(&path).await.expect("open_local failed");
        let value = smoke_test(&conn).await.expect("smoke_test failed");
        assert_eq!(value, "libsql ok");
    }
}
