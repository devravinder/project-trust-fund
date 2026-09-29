//! Small shared helpers for repositories.

use chrono::Utc;
use libsql::{Row, Value};

/// Generate a new UUID v4 string (sync-safe primary key).
pub fn new_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

/// Current UTC timestamp as ISO-8601 string.
pub fn now_iso() -> String {
    Utc::now().to_rfc3339()
}

/// Read a numeric column as f64, tolerating Integer, Real, or Null
/// (SQLite `SUM`/`COALESCE` can return any of these). Null -> 0.0.
pub fn get_f64(row: &Row, idx: i32) -> Result<f64, libsql::Error> {
    match row.get_value(idx)? {
        Value::Real(r) => Ok(r),
        Value::Integer(i) => Ok(i as f64),
        Value::Null => Ok(0.0),
        _ => Ok(0.0),
    }
}
