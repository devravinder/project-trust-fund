//! Data-access layer bridging the two backends behind one API.
//!
//! Memory policy: nothing is cached between operations. The JSON backend loads
//! the file, performs the operation, and writes back — the in-memory `Dataset`
//! is transient (dropped when the call returns). The Turso backend issues
//! targeted SQL so only the needed rows transfer.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::models::{Borrower, Loan, Payment, ScheduleItem};
use crate::store::{BackendRef, StoreError};
use crate::turso::{arg_int, arg_opt_int, arg_opt_text, arg_real, arg_text, Row, TursoClient};

/// Transient dataset used only by the JSON backend during a single operation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Dataset {
    #[serde(default)]
    pub borrowers: Vec<Borrower>,
    #[serde(default)]
    pub loans: Vec<Loan>,
    #[serde(default)]
    pub payments: Vec<Payment>,
    #[serde(default)]
    pub schedule: Vec<ScheduleItem>,
    #[serde(default)]
    pub deleted_borrowers: Vec<String>,
    #[serde(default)]
    pub deleted_loans: Vec<String>,
    #[serde(default)]
    pub deleted_payments: Vec<String>,
}

impl Dataset {
    pub fn is_empty(&self) -> bool {
        self.borrowers.is_empty() && self.loans.is_empty() && self.payments.is_empty()
    }
}

// ---- JSON file load/save (transient) ----

pub fn json_load(path: &PathBuf) -> Result<Dataset, StoreError> {
    if !path.exists() {
        return Ok(Dataset::default());
    }
    let text = std::fs::read_to_string(path).map_err(|e| StoreError::Io(e.to_string()))?;
    if text.trim().is_empty() {
        return Ok(Dataset::default());
    }
    serde_json::from_str(&text).map_err(|e| StoreError::Serde(e.to_string()))
}

pub fn json_save(path: &PathBuf, d: &Dataset) -> Result<(), StoreError> {
    let text = serde_json::to_string_pretty(d).map_err(|e| StoreError::Serde(e.to_string()))?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| StoreError::Io(e.to_string()))?;
    }
    std::fs::write(path, text).map_err(|e| StoreError::Io(e.to_string()))
}

// ---- Turso row mapping ----

fn s(v: &Value) -> String {
    v.as_str().unwrap_or("").to_string()
}
fn os(v: &Value) -> Option<String> {
    v.as_str().map(|x| x.to_string())
}
fn f(v: &Value) -> f64 {
    v.as_f64().unwrap_or(0.0)
}
fn oi(v: &Value) -> Option<i64> {
    v.as_i64()
}

pub fn row_to_borrower(r: &Row) -> Borrower {
    Borrower {
        id: s(&r[0]),
        name: s(&r[1]),
        phone: os(&r[2]),
        address: os(&r[3]),
        photo_path: os(&r[4]),
        notes: os(&r[5]),
        created_at: s(&r[6]),
        updated_at: s(&r[7]),
    }
}

pub fn row_to_loan(r: &Row) -> Loan {
    Loan {
        id: s(&r[0]),
        borrower_id: s(&r[1]),
        principal: f(&r[2]),
        monthly_rate: f(&r[3]),
        interest_type: s(&r[4]),
        repayment_mode: s(&r[5]),
        term_months: oi(&r[6]),
        start_date: s(&r[7]),
        status: s(&r[8]),
        note: os(&r[9]),
        created_at: s(&r[10]),
        updated_at: s(&r[11]),
    }
}

pub fn row_to_payment(r: &Row) -> Payment {
    Payment {
        id: s(&r[0]),
        loan_id: s(&r[1]),
        amount: f(&r[2]),
        interest_component: f(&r[3]),
        principal_component: f(&r[4]),
        paid_date: s(&r[5]),
        note: os(&r[6]),
        created_at: s(&r[7]),
        updated_at: s(&r[8]),
    }
}

pub fn row_to_schedule(r: &Row) -> ScheduleItem {
    ScheduleItem {
        id: s(&r[0]),
        loan_id: s(&r[1]),
        seq: oi(&r[2]).unwrap_or(0),
        due_date: s(&r[3]),
        principal_due: f(&r[4]),
        interest_due: f(&r[5]),
        total_due: f(&r[6]),
        status: s(&r[7]),
    }
}

// Column lists (order must match row_to_* above).
pub const BORROWER_COLS: &str = "id,name,phone,address,photo_path,notes,created_at,updated_at";
pub const LOAN_COLS: &str = "id,borrower_id,principal,monthly_rate,interest_type,repayment_mode,term_months,start_date,status,note,created_at,updated_at";
pub const PAYMENT_COLS: &str =
    "id,loan_id,amount,interest_component,principal_component,paid_date,note,created_at,updated_at";
pub const SCHEDULE_COLS: &str =
    "id,loan_id,seq,due_date,principal_due,interest_due,total_due,status";

// ---- Turso insert arg builders ----

pub fn borrower_args(b: &Borrower) -> Vec<Value> {
    vec![
        arg_text(&b.id),
        arg_text(&b.name),
        arg_opt_text(&b.phone),
        arg_opt_text(&b.address),
        arg_opt_text(&b.photo_path),
        arg_opt_text(&b.notes),
        arg_text(&b.created_at),
        arg_text(&b.updated_at),
    ]
}

pub fn loan_args(l: &Loan) -> Vec<Value> {
    vec![
        arg_text(&l.id),
        arg_text(&l.borrower_id),
        arg_real(l.principal),
        arg_real(l.monthly_rate),
        arg_text(&l.interest_type),
        arg_text(&l.repayment_mode),
        arg_opt_int(l.term_months),
        arg_text(&l.start_date),
        arg_text(&l.status),
        arg_opt_text(&l.note),
        arg_text(&l.created_at),
        arg_text(&l.updated_at),
    ]
}

pub fn payment_args(p: &Payment) -> Vec<Value> {
    vec![
        arg_text(&p.id),
        arg_text(&p.loan_id),
        arg_real(p.amount),
        arg_real(p.interest_component),
        arg_real(p.principal_component),
        arg_text(&p.paid_date),
        arg_opt_text(&p.note),
        arg_text(&p.created_at),
        arg_text(&p.updated_at),
    ]
}

pub fn schedule_args(it: &ScheduleItem) -> Vec<Value> {
    vec![
        arg_text(&it.id),
        arg_text(&it.loan_id),
        arg_int(it.seq),
        arg_text(&it.due_date),
        arg_real(it.principal_due),
        arg_real(it.interest_due),
        arg_real(it.total_due),
        arg_text(&it.status),
    ]
}

/// Load the full dataset from Turso (used for migration/export only — normal
/// reads use targeted queries). Kept separate so it isn't used casually.
pub async fn turso_load_all(c: &TursoClient) -> Result<Dataset, StoreError> {
    let bq = c
        .query(&format!("SELECT {BORROWER_COLS} FROM borrowers WHERE deleted = 0"), vec![])
        .await
        .map_err(StoreError::Turso)?;
    let lq = c
        .query(&format!("SELECT {LOAN_COLS} FROM loans WHERE deleted = 0"), vec![])
        .await
        .map_err(StoreError::Turso)?;
    let pq = c
        .query(&format!("SELECT {PAYMENT_COLS} FROM payments WHERE deleted = 0"), vec![])
        .await
        .map_err(StoreError::Turso)?;
    let sq = c
        .query(&format!("SELECT {SCHEDULE_COLS} FROM schedule"), vec![])
        .await
        .map_err(StoreError::Turso)?;
    Ok(Dataset {
        borrowers: bq.iter().map(row_to_borrower).collect(),
        loans: lq.iter().map(row_to_loan).collect(),
        payments: pq.iter().map(row_to_payment).collect(),
        schedule: sq.iter().map(row_to_schedule).collect(),
        deleted_borrowers: vec![],
        deleted_loans: vec![],
        deleted_payments: vec![],
    })
}

/// Push an entire dataset into Turso (used for JSON->Turso migration).
pub async fn turso_import_all(c: &TursoClient, d: &Dataset) -> Result<(), StoreError> {
    let mut stmts: Vec<(String, Vec<Value>)> = Vec::new();
    let del = |id: &String, list: &[String]| list.contains(id);
    for b in &d.borrowers {
        if del(&b.id, &d.deleted_borrowers) {
            continue;
        }
        stmts.push((
            format!("INSERT OR REPLACE INTO borrowers ({BORROWER_COLS},deleted) VALUES (?,?,?,?,?,?,?,?,0)"),
            borrower_args(b),
        ));
    }
    for l in &d.loans {
        if del(&l.id, &d.deleted_loans) {
            continue;
        }
        stmts.push((
            format!("INSERT OR REPLACE INTO loans ({LOAN_COLS},deleted) VALUES (?,?,?,?,?,?,?,?,?,?,?,?,0)"),
            loan_args(l),
        ));
    }
    for p in &d.payments {
        if del(&p.id, &d.deleted_payments) {
            continue;
        }
        stmts.push((
            format!("INSERT OR REPLACE INTO payments ({PAYMENT_COLS},deleted) VALUES (?,?,?,?,?,?,?,?,?,0)"),
            payment_args(p),
        ));
    }
    for it in &d.schedule {
        stmts.push((
            format!("INSERT OR REPLACE INTO schedule ({SCHEDULE_COLS}) VALUES (?,?,?,?,?,?,?,?)"),
            schedule_args(it),
        ));
    }
    if stmts.is_empty() {
        return Ok(());
    }
    c.batch(stmts).await.map_err(StoreError::Turso)
}

// ---- Unified read/write runners (reuse the &Dataset repos for both backends) ----
//
// Memory policy: the transient Dataset is built per call and dropped at the end.
// JSON reads the file; Turso reads its rows. Neither caches between calls.

use crate::store::StoreState;

/// Load a transient dataset from the active backend.
async fn load(b: &BackendRef) -> Result<Dataset, StoreError> {
    match b {
        BackendRef::Json(path) => json_load(path),
        BackendRef::Turso(c) => turso_load_all(c).await,
    }
}

/// Persist a dataset to the active backend (full write — simple + correct for
/// this app's scale; Turso uses INSERT OR REPLACE and applies soft-deletes).
async fn save(b: &BackendRef, d: &Dataset) -> Result<(), StoreError> {
    match b {
        BackendRef::Json(path) => json_save(path, d),
        BackendRef::Turso(c) => {
            turso_import_all(c, d).await?;
            // Apply soft-deletes as flags.
            apply_turso_deletes(c, d).await
        }
    }
}

async fn apply_turso_deletes(c: &TursoClient, d: &Dataset) -> Result<(), StoreError> {
    let mut stmts: Vec<(String, Vec<Value>)> = Vec::new();
    for id in &d.deleted_borrowers {
        stmts.push(("UPDATE borrowers SET deleted = 1 WHERE id = ?".into(), vec![arg_text(id)]));
    }
    for id in &d.deleted_loans {
        stmts.push(("UPDATE loans SET deleted = 1 WHERE id = ?".into(), vec![arg_text(id)]));
    }
    for id in &d.deleted_payments {
        stmts.push(("UPDATE payments SET deleted = 1 WHERE id = ?".into(), vec![arg_text(id)]));
    }
    if stmts.is_empty() {
        return Ok(());
    }
    c.batch(stmts).await.map_err(StoreError::Turso)
}

/// Run a read-only operation against a freshly-loaded transient dataset.
pub async fn read<F, T>(state: &StoreState, f: F) -> Result<T, StoreError>
where
    F: FnOnce(&Dataset) -> T,
{
    state
        .with_backend(|b| async move {
            let d = load(&b).await?;
            Ok(f(&d))
        })
        .await
}

/// Run a mutating operation: load, mutate, save, return the closure result.
pub async fn write<F, T>(state: &StoreState, f: F) -> Result<T, StoreError>
where
    F: FnOnce(&mut Dataset) -> T,
{
    state
        .with_backend(|b| async move {
            let mut d = load(&b).await?;
            let result = f(&mut d);
            save(&b, &d).await?;
            Ok(result)
        })
        .await
}

