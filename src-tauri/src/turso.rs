//! Turso (libSQL) HTTP client using the `/v2/pipeline` API — pure Rust, no
//! native SQLite. Uses real per-row tables so queries fetch only what's needed
//! (not the whole dataset), which keeps memory and bandwidth bounded.

use serde_json::{json, Value};

#[derive(Clone)]
pub struct TursoClient {
    http: reqwest::Client,
    pipeline_url: String,
    token: String,
}

/// A decoded SQL row: ordered column values as owned JSON values.
pub type Row = Vec<Value>;

impl TursoClient {
    pub fn new(url: String, token: String) -> Self {
        let base = normalize_url(&url);
        let pipeline_url = format!("{}/v2/pipeline", base.trim_end_matches('/'));
        Self {
            http: reqwest::Client::new(),
            pipeline_url,
            token,
        }
    }

    /// Run one SQL statement with positional args; return decoded rows.
    pub async fn query(&self, sql: &str, args: Vec<Value>) -> Result<Vec<Row>, String> {
        let results = self.pipeline(vec![stmt(sql, args)]).await?;
        decode_rows(results.first())
    }

    /// Run a batch of (sql,args) statements in a single request (atomic-ish).
    pub async fn batch(&self, stmts: Vec<(String, Vec<Value>)>) -> Result<(), String> {
        let reqs: Vec<Value> = stmts.into_iter().map(|(s, a)| stmt(&s, a)).collect();
        self.pipeline(reqs).await.map(|_| ())
    }

    async fn pipeline(&self, stmts: Vec<Value>) -> Result<Vec<Value>, String> {
        let mut requests: Vec<Value> = stmts
            .into_iter()
            .map(|stmt| json!({ "type": "execute", "stmt": stmt }))
            .collect();
        requests.push(json!({ "type": "close" }));

        let resp = self
            .http
            .post(&self.pipeline_url)
            .bearer_auth(&self.token)
            .json(&json!({ "requests": requests }))
            .send()
            .await
            .map_err(|e| format!("request failed: {e}"))?;

        let status = resp.status();
        let text = resp.text().await.map_err(|e| format!("read body: {e}"))?;
        if !status.is_success() {
            return Err(format!("HTTP {status}: {text}"));
        }
        let parsed: Value = serde_json::from_str(&text).map_err(|e| format!("bad json: {e}"))?;
        let results = parsed
            .get("results")
            .and_then(|r| r.as_array())
            .ok_or_else(|| format!("unexpected response: {text}"))?;
        for r in results {
            if r.get("type").and_then(|t| t.as_str()) == Some("error") {
                let msg = r
                    .get("error")
                    .and_then(|e| e.get("message"))
                    .and_then(|m| m.as_str())
                    .unwrap_or("unknown");
                return Err(format!("turso error: {msg}"));
            }
        }
        Ok(results.clone())
    }

    /// Create the schema (idempotent) and validate connectivity/auth.
    pub async fn ensure_schema(&self) -> Result<(), String> {
        let ddl = [
            "CREATE TABLE IF NOT EXISTS borrowers (id TEXT PRIMARY KEY, name TEXT NOT NULL, phone TEXT, address TEXT, photo_path TEXT, notes TEXT, created_at TEXT, updated_at TEXT, deleted INTEGER DEFAULT 0)",
            "CREATE TABLE IF NOT EXISTS loans (id TEXT PRIMARY KEY, borrower_id TEXT, principal REAL, monthly_rate REAL, interest_type TEXT, repayment_mode TEXT, end_date TEXT, start_date TEXT, status TEXT, note TEXT, created_at TEXT, updated_at TEXT, deleted INTEGER DEFAULT 0)",
            "CREATE TABLE IF NOT EXISTS payments (id TEXT PRIMARY KEY, loan_id TEXT, amount REAL, interest_component REAL, principal_component REAL, paid_date TEXT, note TEXT, created_at TEXT, updated_at TEXT, deleted INTEGER DEFAULT 0)",
            "CREATE TABLE IF NOT EXISTS schedule (id TEXT PRIMARY KEY, loan_id TEXT, seq INTEGER, due_date TEXT, principal_due REAL, interest_due REAL, total_due REAL, status TEXT)",
        ];
        let batch = ddl.iter().map(|s| (s.to_string(), vec![])).collect();
        self.batch(batch).await
    }
}

/// Build a statement value with positional args.
fn stmt(sql: &str, args: Vec<Value>) -> Value {
    json!({ "sql": sql, "args": args })
}

/// Encode a nullable text arg.
pub fn arg_text(s: &str) -> Value {
    json!({ "type": "text", "value": s })
}
pub fn arg_opt_text(s: &Option<String>) -> Value {
    match s {
        Some(v) => json!({ "type": "text", "value": v }),
        None => json!({ "type": "null" }),
    }
}
pub fn arg_real(n: f64) -> Value {
    json!({ "type": "float", "value": n })
}
pub fn arg_int(n: i64) -> Value {
    json!({ "type": "integer", "value": n.to_string() })
}

/// Decode the Hrana rows for the first statement result into owned JSON values.
fn decode_rows(result: Option<&Value>) -> Result<Vec<Row>, String> {
    let rows = match result
        .and_then(|r| r.get("response"))
        .and_then(|r| r.get("result"))
        .and_then(|r| r.get("rows"))
        .and_then(|r| r.as_array())
    {
        Some(rows) => rows,
        None => return Ok(vec![]),
    };
    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        let cells = row.as_array().ok_or("row not array")?;
        let decoded: Row = cells
            .iter()
            .map(|c| {
                let t = c.get("type").and_then(|t| t.as_str()).unwrap_or("null");
                match t {
                    "null" => Value::Null,
                    "integer" => c
                        .get("value")
                        .and_then(|v| v.as_str())
                        .and_then(|s| s.parse::<i64>().ok())
                        .map(|n| json!(n))
                        .unwrap_or(Value::Null),
                    "float" => c
                        .get("value")
                        .and_then(|v| {
                            v.as_f64()
                                .or_else(|| v.as_str().and_then(|s| s.parse::<f64>().ok()))
                        })
                        .map(|n| json!(n))
                        .unwrap_or(Value::Null),
                    _ => c
                        .get("value")
                        .and_then(|v| v.as_str())
                        .map(|s| json!(s))
                        .unwrap_or(Value::Null),
                }
            })
            .collect();
        out.push(decoded);
    }
    Ok(out)
}

fn normalize_url(url: &str) -> String {
    let u = url.trim();
    if let Some(rest) = u.strip_prefix("libsql://") {
        format!("https://{rest}")
    } else if let Some(rest) = u.strip_prefix("wss://") {
        format!("https://{rest}")
    } else if let Some(rest) = u.strip_prefix("ws://") {
        format!("https://{rest}")
    } else if u.starts_with("http://") || u.starts_with("https://") {
        u.to_string()
    } else {
        format!("https://{u}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_handles_libsql_scheme() {
        assert_eq!(normalize_url("libsql://db.turso.io"), "https://db.turso.io");
        assert_eq!(normalize_url("db.turso.io"), "https://db.turso.io");
    }
}
