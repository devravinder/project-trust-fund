//! Small shared helpers for repositories.

use chrono::Utc;

/// Generate a new UUID v4 string (sync-safe primary key).
pub fn new_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

/// Current UTC timestamp as ISO-8601 string.
pub fn now_iso() -> String {
    Utc::now().to_rfc3339()
}
