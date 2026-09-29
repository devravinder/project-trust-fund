//! Loan repository: CRUD, installment schedule generation, balances, status.

use chrono::{Datelike, NaiveDate};
use libsql::{params, Connection, Row};

use crate::db::DbError;
use crate::interest::{generate_schedule_simple, round2};
use crate::models::{Loan, LoanInput, ScheduleItem};
use crate::util::{new_id, now_iso};

fn map_row(row: &Row) -> Result<Loan, DbError> {
    Ok(Loan {
        id: row.get(0)?,
        borrower_id: row.get(1)?,
        principal: row.get(2)?,
        monthly_rate: row.get(3)?,
        interest_type: row.get(4)?,
        repayment_mode: row.get(5)?,
        term_months: row.get(6)?,
        start_date: row.get(7)?,
        status: row.get(8)?,
        note: row.get(9)?,
        created_at: row.get(10)?,
        updated_at: row.get(11)?,
    })
}

const SELECT: &str = "SELECT id, borrower_id, principal, monthly_rate, interest_type, \
     repayment_mode, term_months, start_date, status, note, created_at, updated_at \
     FROM loans WHERE deleted_at IS NULL";

/// Loan summary with computed balances, for list/detail views.
#[derive(Debug, Clone, serde::Serialize)]
pub struct LoanSummary {
    #[serde(flatten)]
    pub loan: Loan,
    pub borrower_name: String,
    pub principal_recovered: f64,
    pub interest_collected: f64,
    pub outstanding_principal: f64,
}

pub async fn list(
    conn: &Connection,
    status_filter: Option<String>,
    search: Option<String>,
) -> Result<Vec<LoanSummary>, DbError> {
    // Base query joins borrower and aggregates payments.
    let mut sql = String::from(
        "SELECT l.id, l.borrower_id, l.principal, l.monthly_rate, l.interest_type, \
            l.repayment_mode, l.term_months, l.start_date, l.status, l.note, \
            l.created_at, l.updated_at, b.name, \
            COALESCE((SELECT SUM(p.principal_component) FROM payments p \
              WHERE p.loan_id = l.id AND p.deleted_at IS NULL), 0), \
            COALESCE((SELECT SUM(p.interest_component) FROM payments p \
              WHERE p.loan_id = l.id AND p.deleted_at IS NULL), 0) \
         FROM loans l JOIN borrowers b ON b.id = l.borrower_id \
         WHERE l.deleted_at IS NULL",
    );
    if let Some(s) = &status_filter {
        if s != "all" {
            sql.push_str(&format!(" AND l.status = '{}'", s.replace('\'', "")));
        }
    }
    if let Some(term) = &search {
        if !term.trim().is_empty() {
            let safe = term.trim().replace('\'', "");
            sql.push_str(&format!(" AND b.name LIKE '%{safe}%'"));
        }
    }
    sql.push_str(" ORDER BY l.created_at DESC");

    let mut rows = conn.query(&sql, ()).await?;
    let mut out = Vec::new();
    while let Some(row) = rows.next().await? {
        let loan = map_row(&row)?;
        let borrower_name: String = row.get(12)?;
        let principal_recovered: f64 = row.get(13)?;
        let interest_collected: f64 = row.get(14)?;
        out.push(LoanSummary {
            outstanding_principal: round2(loan.principal - principal_recovered),
            loan,
            borrower_name,
            principal_recovered: round2(principal_recovered),
            interest_collected: round2(interest_collected),
        });
    }
    Ok(out)
}

pub async fn get(conn: &Connection, id: &str) -> Result<Option<Loan>, DbError> {
    let mut rows = conn
        .query(&format!("{SELECT} AND id = ?1"), params![id])
        .await?;
    match rows.next().await? {
        Some(row) => Ok(Some(map_row(&row)?)),
        None => Ok(None),
    }
}

/// Add `months` to a date, clamping the day to the month's length.
fn add_months(date: NaiveDate, months: i64) -> NaiveDate {
    let mut y = date.year();
    let mut m0 = date.month0() as i64 + months;
    y += (m0 / 12) as i32;
    m0 %= 12;
    if m0 < 0 {
        m0 += 12;
        y -= 1;
    }
    let month = (m0 + 1) as u32;
    // Clamp day to last day of target month.
    let last_day = last_day_of_month(y, month);
    let day = date.day().min(last_day);
    NaiveDate::from_ymd_opt(y, month, day).unwrap()
}

fn last_day_of_month(year: i32, month: u32) -> u32 {
    let (ny, nm) = if month == 12 {
        (year + 1, 1)
    } else {
        (year, month + 1)
    };
    let first_next = NaiveDate::from_ymd_opt(ny, nm, 1).unwrap();
    first_next.pred_opt().unwrap().day()
}

pub async fn create(conn: &Connection, input: LoanInput) -> Result<Loan, DbError> {
    let id = new_id();
    let now = now_iso();
    conn.execute(
        "INSERT INTO loans (id, borrower_id, principal, monthly_rate, interest_type, \
           repayment_mode, term_months, start_date, status, note, created_at, updated_at) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 'active', ?9, ?10, ?10)",
        params![
            id.clone(),
            input.borrower_id,
            input.principal,
            input.monthly_rate,
            input.interest_type.clone(),
            input.repayment_mode.clone(),
            input.term_months,
            input.start_date.clone(),
            input.note,
            now,
        ],
    )
    .await?;

    // Generate installment schedule if applicable (v1: simple interest only).
    if input.repayment_mode == "installments" {
        if let Some(n) = input.term_months {
            let start = NaiveDate::parse_from_str(&input.start_date, "%Y-%m-%d")
                .map_err(|e| DbError::Msg(format!("bad start_date: {e}")))?;
            let rows = generate_schedule_simple(input.principal, input.monthly_rate, n);
            let ts = now_iso();
            for r in rows {
                let due = add_months(start, r.seq);
                conn.execute(
                    "INSERT INTO installment_schedule (id, loan_id, seq, due_date, \
                       principal_due, interest_due, total_due, status, created_at, updated_at) \
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 'pending', ?8, ?8)",
                    params![
                        new_id(),
                        id.clone(),
                        r.seq,
                        due.format("%Y-%m-%d").to_string(),
                        r.principal_due,
                        r.interest_due,
                        r.total_due,
                        ts.clone(),
                    ],
                )
                .await?;
            }
        }
    }

    get(conn, &id).await?.ok_or(DbError::Msg("insert failed".into()))
}

pub async fn set_status(conn: &Connection, id: &str, status: &str) -> Result<(), DbError> {
    let now = now_iso();
    conn.execute(
        "UPDATE loans SET status = ?2, updated_at = ?3 WHERE id = ?1 AND deleted_at IS NULL",
        params![id, status, now],
    )
    .await?;
    Ok(())
}

pub async fn soft_delete(conn: &Connection, id: &str) -> Result<(), DbError> {
    let now = now_iso();
    conn.execute(
        "UPDATE loans SET deleted_at = ?2, updated_at = ?2 WHERE id = ?1",
        params![id, now],
    )
    .await?;
    Ok(())
}

pub async fn schedule(conn: &Connection, loan_id: &str) -> Result<Vec<ScheduleItem>, DbError> {
    let mut rows = conn
        .query(
            "SELECT id, loan_id, seq, due_date, principal_due, interest_due, total_due, status \
             FROM installment_schedule WHERE loan_id = ?1 ORDER BY seq",
            params![loan_id],
        )
        .await?;
    let mut out = Vec::new();
    while let Some(row) = rows.next().await? {
        out.push(ScheduleItem {
            id: row.get(0)?,
            loan_id: row.get(1)?,
            seq: row.get(2)?,
            due_date: row.get(3)?,
            principal_due: row.get(4)?,
            interest_due: row.get(5)?,
            total_due: row.get(6)?,
            status: row.get(7)?,
        });
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::BorrowerInput;
    use crate::repo_borrowers;

    async fn test_conn() -> Connection {
        let db = libsql::Builder::new_local(":memory:").build().await.unwrap();
        let conn = db.connect().unwrap();
        crate::migrations::run_migrations(&conn).await.unwrap();
        conn
    }

    #[test]
    fn add_months_clamps_day() {
        let jan31 = NaiveDate::from_ymd_opt(2026, 1, 31).unwrap();
        // +1 month -> Feb 28 (2026 not leap)
        assert_eq!(add_months(jan31, 1), NaiveDate::from_ymd_opt(2026, 2, 28).unwrap());
    }

    #[tokio::test]
    async fn create_installment_loan_generates_schedule() {
        let conn = test_conn().await;
        let b = repo_borrowers::create(
            &conn,
            BorrowerInput {
                name: "Test".into(),
                phone: None,
                address: None,
                photo_path: None,
                notes: None,
            },
        )
        .await
        .unwrap();

        let loan = create(
            &conn,
            LoanInput {
                borrower_id: b.id,
                principal: 10_000.0,
                monthly_rate: 0.02,
                interest_type: "simple".into(),
                repayment_mode: "installments".into(),
                term_months: Some(5),
                start_date: "2026-01-15".into(),
                note: None,
            },
        )
        .await
        .unwrap();

        let sched = schedule(&conn, &loan.id).await.unwrap();
        assert_eq!(sched.len(), 5);
        assert_eq!(sched[0].interest_due, 200.0);
        assert_eq!(sched[0].due_date, "2026-02-15");
        let total_principal: f64 = sched.iter().map(|s| s.principal_due).sum();
        assert_eq!(round2(total_principal), 10_000.0);
    }
}
