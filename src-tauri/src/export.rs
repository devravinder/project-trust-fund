//! CSV export of core tables for data-safety / backup.

use libsql::Connection;

use crate::db::DbError;

fn esc(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

fn cell(v: libsql::Value) -> String {
    match v {
        libsql::Value::Null => String::new(),
        libsql::Value::Integer(i) => i.to_string(),
        libsql::Value::Real(r) => r.to_string(),
        libsql::Value::Text(t) => esc(&t),
        libsql::Value::Blob(_) => String::from("<blob>"),
    }
}

/// Export a query's rows as CSV with the given header.
async fn export_query(
    conn: &Connection,
    sql: &str,
    header: &str,
    columns: usize,
) -> Result<String, DbError> {
    let mut out = String::from(header);
    out.push('\n');
    let mut rows = conn.query(sql, ()).await?;
    while let Some(row) = rows.next().await? {
        let mut line = Vec::with_capacity(columns);
        for i in 0..columns as i32 {
            line.push(cell(row.get_value(i)?));
        }
        out.push_str(&line.join(","));
        out.push('\n');
    }
    Ok(out)
}

pub async fn borrowers_csv(conn: &Connection) -> Result<String, DbError> {
    export_query(
        conn,
        "SELECT id, name, phone, address, notes, created_at FROM borrowers \
         WHERE deleted_at IS NULL ORDER BY name",
        "id,name,phone,address,notes,created_at",
        6,
    )
    .await
}

pub async fn loans_csv(conn: &Connection) -> Result<String, DbError> {
    export_query(
        conn,
        "SELECT l.id, b.name, l.principal, l.monthly_rate, l.interest_type, \
            l.repayment_mode, l.term_months, l.start_date, l.status, l.created_at \
         FROM loans l JOIN borrowers b ON b.id = l.borrower_id \
         WHERE l.deleted_at IS NULL ORDER BY l.created_at",
        "id,borrower,principal,monthly_rate,interest_type,repayment_mode,term_months,start_date,status,created_at",
        10,
    )
    .await
}

pub async fn payments_csv(conn: &Connection) -> Result<String, DbError> {
    export_query(
        conn,
        "SELECT p.id, b.name, p.amount, p.interest_component, p.principal_component, \
            p.paid_date, p.note, p.created_at \
         FROM payments p JOIN loans l ON l.id = p.loan_id \
         JOIN borrowers b ON b.id = l.borrower_id \
         WHERE p.deleted_at IS NULL ORDER BY p.paid_date",
        "id,borrower,amount,interest,principal,paid_date,note,created_at",
        8,
    )
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::BorrowerInput;
    use crate::repo_borrowers;

    #[tokio::test]
    async fn borrowers_export_has_header_and_row() {
        let db = libsql::Builder::new_local(":memory:").build().await.unwrap();
        let conn = db.connect().unwrap();
        crate::migrations::run_migrations(&conn).await.unwrap();
        repo_borrowers::create(
            &conn,
            BorrowerInput {
                name: "A, B".into(), // comma to test escaping
                phone: None,
                address: None,
                photo_path: None,
                notes: None,
            },
        )
        .await
        .unwrap();
        let csv = borrowers_csv(&conn).await.unwrap();
        assert!(csv.starts_with("id,name,phone"));
        assert!(csv.contains("\"A, B\""));
    }
}
