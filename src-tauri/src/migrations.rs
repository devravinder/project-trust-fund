//! Schema migrations. Applied in order on startup; tracked via `user_version`.

use libsql::Connection;

use crate::db::DbError;

/// Ordered list of migration SQL scripts. Index + 1 = target user_version.
const MIGRATIONS: &[&str] = &[
    // v1: initial schema
    r#"
    CREATE TABLE IF NOT EXISTS borrowers (
        id          TEXT PRIMARY KEY,
        name        TEXT NOT NULL,
        phone       TEXT,
        address     TEXT,
        photo_path  TEXT,
        notes       TEXT,
        created_at  TEXT NOT NULL,
        updated_at  TEXT NOT NULL,
        deleted_at  TEXT
    );
    CREATE INDEX IF NOT EXISTS idx_borrowers_name ON borrowers(name);

    CREATE TABLE IF NOT EXISTS loans (
        id             TEXT PRIMARY KEY,
        borrower_id    TEXT NOT NULL REFERENCES borrowers(id),
        principal      REAL NOT NULL,
        monthly_rate   REAL NOT NULL,
        interest_type  TEXT NOT NULL CHECK (interest_type IN ('simple','compound')),
        repayment_mode TEXT NOT NULL CHECK (repayment_mode IN ('one_time','installments')),
        term_months    INTEGER,
        start_date     TEXT NOT NULL,
        status         TEXT NOT NULL DEFAULT 'active'
                        CHECK (status IN ('active','overdue','closed','written_off')),
        note           TEXT,
        created_at     TEXT NOT NULL,
        updated_at     TEXT NOT NULL,
        deleted_at     TEXT
    );
    CREATE INDEX IF NOT EXISTS idx_loans_borrower ON loans(borrower_id);
    CREATE INDEX IF NOT EXISTS idx_loans_status ON loans(status);

    CREATE TABLE IF NOT EXISTS payments (
        id                  TEXT PRIMARY KEY,
        loan_id             TEXT NOT NULL REFERENCES loans(id),
        amount              REAL NOT NULL,
        interest_component  REAL NOT NULL DEFAULT 0,
        principal_component REAL NOT NULL DEFAULT 0,
        paid_date           TEXT NOT NULL,
        note                TEXT,
        created_at          TEXT NOT NULL,
        updated_at          TEXT NOT NULL,
        deleted_at          TEXT
    );
    CREATE INDEX IF NOT EXISTS idx_payments_loan ON payments(loan_id);
    CREATE INDEX IF NOT EXISTS idx_payments_date ON payments(paid_date);

    CREATE TABLE IF NOT EXISTS installment_schedule (
        id            TEXT PRIMARY KEY,
        loan_id       TEXT NOT NULL REFERENCES loans(id),
        seq           INTEGER NOT NULL,
        due_date      TEXT NOT NULL,
        principal_due REAL NOT NULL,
        interest_due  REAL NOT NULL,
        total_due     REAL NOT NULL,
        status        TEXT NOT NULL DEFAULT 'pending'
                       CHECK (status IN ('pending','paid','partial','overdue')),
        created_at    TEXT NOT NULL,
        updated_at    TEXT NOT NULL
    );
    CREATE INDEX IF NOT EXISTS idx_schedule_loan ON installment_schedule(loan_id);
    CREATE INDEX IF NOT EXISTS idx_schedule_due ON installment_schedule(due_date);

    CREATE TABLE IF NOT EXISTS app_meta (
        key        TEXT PRIMARY KEY,
        value      TEXT,
        updated_at TEXT NOT NULL
    );
    "#,
];

/// Apply any migrations newer than the current `user_version`.
pub async fn run_migrations(conn: &Connection) -> Result<(), DbError> {
    let current = current_version(conn).await?;
    for (i, sql) in MIGRATIONS.iter().enumerate() {
        let version = (i + 1) as i64;
        if version > current {
            conn.execute_batch(sql).await?;
            // user_version can't be parameterized; value is a controlled constant.
            conn.execute_batch(&format!("PRAGMA user_version = {version};"))
                .await?;
        }
    }
    Ok(())
}

async fn current_version(conn: &Connection) -> Result<i64, DbError> {
    let mut rows = conn.query("PRAGMA user_version", ()).await?;
    if let Some(row) = rows.next().await? {
        Ok(row.get::<i64>(0)?)
    } else {
        Ok(0)
    }
}
