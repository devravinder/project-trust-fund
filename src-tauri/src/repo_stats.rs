//! Dashboard statistics and "dues this month" queries.

use chrono::{Datelike, Local, NaiveDate};
use libsql::{params, Connection};

use crate::db::DbError;
use crate::interest::{effective_months, round2, simple_interest, whole_months_between};
use crate::util::get_f64;

#[derive(Debug, Clone, serde::Serialize)]
pub struct DashboardSummary {
    pub total_lent: f64,             // principal of active/overdue loans
    pub outstanding_principal: f64,  // lent - principal recovered
    pub active_loans: i64,
    pub overdue_loans: i64,
    pub overdue_amount: f64,
    pub interest_collected: f64,       // all time
    pub interest_due_this_month: f64,  // accrued-but-unpaid, current month
    pub principal_recovered: f64,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct DueItem {
    pub loan_id: String,
    pub borrower_name: String,
    pub due_date: String,
    pub total_due: f64,
    pub status: String,
}

fn month_bounds(today: NaiveDate) -> (String, String) {
    let start = NaiveDate::from_ymd_opt(today.year(), today.month(), 1).unwrap();
    let (ny, nm) = if today.month() == 12 {
        (today.year() + 1, 1)
    } else {
        (today.year(), today.month() + 1)
    };
    let next = NaiveDate::from_ymd_opt(ny, nm, 1).unwrap();
    (
        start.format("%Y-%m-%d").to_string(),
        next.format("%Y-%m-%d").to_string(),
    )
}

pub async fn summary(conn: &Connection) -> Result<DashboardSummary, DbError> {
    let today = Local::now().date_naive();
    let today_str = today.format("%Y-%m-%d").to_string();

    // Total lent (active + overdue).
    let total_lent = scalar(
        conn,
        "SELECT COALESCE(SUM(principal), 0) FROM loans \
         WHERE deleted_at IS NULL AND status IN ('active','overdue')",
    )
    .await?;

    // Principal recovered (all non-deleted loans).
    let principal_recovered = scalar(
        conn,
        "SELECT COALESCE(SUM(p.principal_component), 0) FROM payments p \
         JOIN loans l ON l.id = p.loan_id \
         WHERE p.deleted_at IS NULL AND l.deleted_at IS NULL",
    )
    .await?;

    let interest_collected = scalar(
        conn,
        "SELECT COALESCE(SUM(p.interest_component), 0) FROM payments p \
         JOIN loans l ON l.id = p.loan_id \
         WHERE p.deleted_at IS NULL AND l.deleted_at IS NULL",
    )
    .await?;

    let active_loans = scalar_i64(
        conn,
        "SELECT COUNT(*) FROM loans WHERE deleted_at IS NULL AND status = 'active'",
    )
    .await?;

    // Overdue: loans past their term end (start + term months < today) with
    // outstanding balance, among active/overdue. Computed in Rust below.
    let (overdue_loans, overdue_amount, interest_due_this_month) =
        overdue_and_due(conn, today).await?;

    let _ = today_str;
    Ok(DashboardSummary {
        total_lent: round2(total_lent),
        outstanding_principal: round2(total_lent - principal_recovered),
        active_loans,
        overdue_loans,
        overdue_amount: round2(overdue_amount),
        interest_collected: round2(interest_collected),
        interest_due_this_month: round2(interest_due_this_month),
        principal_recovered: round2(principal_recovered),
    })
}

/// Walk active/overdue loans to compute overdue count/amount and total
/// interest currently due (accrued minus collected). v1: simple interest.
async fn overdue_and_due(
    conn: &Connection,
    today: NaiveDate,
) -> Result<(i64, f64, f64), DbError> {
    let mut rows = conn
        .query(
            "SELECT l.id, l.principal, l.monthly_rate, l.term_months, l.start_date, \
                COALESCE((SELECT SUM(p.principal_component) FROM payments p \
                  WHERE p.loan_id = l.id AND p.deleted_at IS NULL), 0), \
                COALESCE((SELECT SUM(p.interest_component) FROM payments p \
                  WHERE p.loan_id = l.id AND p.deleted_at IS NULL), 0) \
             FROM loans l \
             WHERE l.deleted_at IS NULL AND l.status IN ('active','overdue')",
            (),
        )
        .await?;

    let mut overdue_count = 0i64;
    let mut overdue_amount = 0.0;
    let mut interest_due_total = 0.0;

    while let Some(row) = rows.next().await? {
        let principal: f64 = get_f64(&row, 1)?;
        let rate: f64 = get_f64(&row, 2)?;
        let term: Option<i64> = row.get(3)?;
        let start_str: String = row.get(4)?;
        let principal_paid: f64 = get_f64(&row, 5)?;
        let interest_paid: f64 = get_f64(&row, 6)?;

        let start = match NaiveDate::parse_from_str(&start_str, "%Y-%m-%d") {
            Ok(d) => d,
            Err(_) => continue,
        };

        let elapsed = whole_months_between(start, today);
        let months = effective_months(elapsed, term);
        let accrued = simple_interest(principal, rate, months);
        let interest_due = (accrued - interest_paid).max(0.0);
        interest_due_total += interest_due;

        let outstanding = (principal - principal_paid).max(0.0);

        // Overdue if past term end with outstanding balance.
        if let Some(t) = term {
            let term_end_reached = elapsed >= t;
            if term_end_reached && outstanding > 0.0 {
                overdue_count += 1;
                overdue_amount += outstanding + interest_due;
            }
        }
    }

    Ok((overdue_count, overdue_amount, interest_due_total))
}

/// Dues this month, drawn from installment schedules that fall in the current
/// month and are not fully paid.
pub async fn dues_this_month(
    conn: &Connection,
    limit: i64,
) -> Result<Vec<DueItem>, DbError> {
    let today = Local::now().date_naive();
    let (start, next) = month_bounds(today);

    let mut rows = conn
        .query(
            "SELECT s.loan_id, b.name, s.due_date, s.total_due, s.status \
             FROM installment_schedule s \
             JOIN loans l ON l.id = s.loan_id \
             JOIN borrowers b ON b.id = l.borrower_id \
             WHERE l.deleted_at IS NULL AND s.status IN ('pending','partial','overdue') \
               AND s.due_date >= ?1 AND s.due_date < ?2 \
             ORDER BY s.due_date LIMIT ?3",
            params![start, next, limit],
        )
        .await?;

    let mut out = Vec::new();
    while let Some(row) = rows.next().await? {
        out.push(DueItem {
            loan_id: row.get(0)?,
            borrower_name: row.get(1)?,
            due_date: row.get(2)?,
            total_due: get_f64(&row, 3)?,
            status: row.get(4)?,
        });
    }
    Ok(out)
}

async fn scalar(conn: &Connection, sql: &str) -> Result<f64, DbError> {
    let mut rows = conn.query(sql, ()).await?;
    match rows.next().await? {
        Some(row) => Ok(get_f64(&row, 0)?),
        None => Ok(0.0),
    }
}

async fn scalar_i64(conn: &Connection, sql: &str) -> Result<i64, DbError> {
    let mut rows = conn.query(sql, ()).await?;
    match rows.next().await? {
        Some(row) => Ok(row.get::<i64>(0)?),
        None => Ok(0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{BorrowerInput, LoanInput, PaymentInput};
    use crate::{repo_borrowers, repo_loans, repo_payments};

    async fn test_conn() -> Connection {
        let db = libsql::Builder::new_local(":memory:").build().await.unwrap();
        let conn = db.connect().unwrap();
        crate::migrations::run_migrations(&conn).await.unwrap();
        conn
    }

    #[tokio::test]
    async fn summary_reflects_loans_and_payments() {
        let conn = test_conn().await;
        let b = repo_borrowers::create(
            &conn,
            BorrowerInput {
                name: "T".into(),
                phone: None,
                address: None,
                photo_path: None,
                notes: None,
            },
        )
        .await
        .unwrap();
        let loan = repo_loans::create(
            &conn,
            LoanInput {
                borrower_id: b.id,
                principal: 10_000.0,
                monthly_rate: 0.02,
                interest_type: "simple".into(),
                repayment_mode: "one_time".into(),
                term_months: Some(12),
                start_date: "2020-01-01".into(),
                note: None,
            },
        )
        .await
        .unwrap();
        // Pay 2000 principal-ish; allocation depends on accrued interest.
        repo_payments::create(
            &conn,
            PaymentInput {
                loan_id: loan.id,
                amount: 2_000.0,
                interest_component: Some(0.0),
                principal_component: Some(2_000.0),
                paid_date: "2020-02-01".into(),
                note: None,
            },
        )
        .await
        .unwrap();

        let s = summary(&conn).await.unwrap();
        assert_eq!(s.total_lent, 10_000.0);
        assert_eq!(s.principal_recovered, 2_000.0);
        assert_eq!(s.outstanding_principal, 8_000.0);
        assert_eq!(s.active_loans, 1);
    }
}
