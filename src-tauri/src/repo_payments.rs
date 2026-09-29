//! Payment repository: CRUD, interest-first allocation, search.

use chrono::NaiveDate;
use libsql::{params, Connection, Row};

use crate::db::DbError;
use crate::interest::{
    allocate_interest_first, effective_months, round2, simple_interest, whole_months_between,
};
use crate::models::{Payment, PaymentInput};
use crate::util::{new_id, now_iso};

fn map_row(row: &Row) -> Result<Payment, DbError> {
    Ok(Payment {
        id: row.get(0)?,
        loan_id: row.get(1)?,
        amount: row.get(2)?,
        interest_component: row.get(3)?,
        principal_component: row.get(4)?,
        paid_date: row.get(5)?,
        note: row.get(6)?,
        created_at: row.get(7)?,
        updated_at: row.get(8)?,
    })
}

const SELECT: &str = "SELECT id, loan_id, amount, interest_component, principal_component, \
     paid_date, note, created_at, updated_at FROM payments WHERE deleted_at IS NULL";

/// Payment enriched with borrower name for list views.
#[derive(Debug, Clone, serde::Serialize)]
pub struct PaymentView {
    #[serde(flatten)]
    pub payment: Payment,
    pub borrower_name: String,
}

/// List payments, optionally filtered by borrower name search.
pub async fn list(conn: &Connection, search: Option<String>) -> Result<Vec<PaymentView>, DbError> {
    let mut sql = String::from(
        "SELECT p.id, p.loan_id, p.amount, p.interest_component, p.principal_component, \
            p.paid_date, p.note, p.created_at, p.updated_at, b.name \
         FROM payments p \
         JOIN loans l ON l.id = p.loan_id \
         JOIN borrowers b ON b.id = l.borrower_id \
         WHERE p.deleted_at IS NULL",
    );
    if let Some(term) = &search {
        if !term.trim().is_empty() {
            let safe = term.trim().replace('\'', "");
            sql.push_str(&format!(" AND b.name LIKE '%{safe}%'"));
        }
    }
    sql.push_str(" ORDER BY p.paid_date DESC, p.created_at DESC");

    let mut rows = conn.query(&sql, ()).await?;
    let mut out = Vec::new();
    while let Some(row) = rows.next().await? {
        let payment = map_row(&row)?;
        let borrower_name: String = row.get(9)?;
        out.push(PaymentView { payment, borrower_name });
    }
    Ok(out)
}

pub async fn list_for_loan(conn: &Connection, loan_id: &str) -> Result<Vec<Payment>, DbError> {
    let mut rows = conn
        .query(
            &format!("{SELECT} AND loan_id = ?1 ORDER BY paid_date DESC"),
            params![loan_id],
        )
        .await?;
    let mut out = Vec::new();
    while let Some(row) = rows.next().await? {
        out.push(map_row(&row)?);
    }
    Ok(out)
}

async fn get(conn: &Connection, id: &str) -> Result<Option<Payment>, DbError> {
    let mut rows = conn
        .query(&format!("{SELECT} AND id = ?1"), params![id])
        .await?;
    match rows.next().await? {
        Some(row) => Ok(Some(map_row(&row)?)),
        None => Ok(None),
    }
}

/// Compute the interest due (unpaid) on a loan as of `as_of` date.
/// v1: simple interest only. accrued = simple_interest(principal, rate, months)
/// where months = whole months elapsed, capped at term. due = accrued - collected.
async fn interest_due_as_of(
    conn: &Connection,
    loan_id: &str,
    as_of: NaiveDate,
) -> Result<(f64, f64), DbError> {
    // Fetch loan basics.
    let mut rows = conn
        .query(
            "SELECT principal, monthly_rate, term_months, start_date FROM loans \
             WHERE id = ?1 AND deleted_at IS NULL",
            params![loan_id],
        )
        .await?;
    let row = rows
        .next()
        .await?
        .ok_or(DbError::Msg("loan not found".into()))?;
    let principal: f64 = row.get(0)?;
    let rate: f64 = row.get(1)?;
    let term: Option<i64> = row.get(2)?;
    let start_str: String = row.get(3)?;
    let start = NaiveDate::parse_from_str(&start_str, "%Y-%m-%d")
        .map_err(|e| DbError::Msg(format!("bad start_date: {e}")))?;

    let elapsed = whole_months_between(start, as_of);
    let months = effective_months(elapsed, term);
    let accrued = simple_interest(principal, rate, months);

    // Interest already collected.
    let mut crows = conn
        .query(
            "SELECT COALESCE(SUM(interest_component), 0) FROM payments \
             WHERE loan_id = ?1 AND deleted_at IS NULL",
            params![loan_id],
        )
        .await?;
    let collected: f64 = match crows.next().await? {
        Some(r) => crate::util::get_f64(&r, 0)?,
        None => 0.0,
    };

    // Outstanding principal.
    let mut prows = conn
        .query(
            "SELECT COALESCE(SUM(principal_component), 0) FROM payments \
             WHERE loan_id = ?1 AND deleted_at IS NULL",
            params![loan_id],
        )
        .await?;
    let principal_paid: f64 = match prows.next().await? {
        Some(r) => crate::util::get_f64(&r, 0)?,
        None => 0.0,
    };

    let interest_due = round2((accrued - collected).max(0.0));
    let outstanding_principal = round2((principal - principal_paid).max(0.0));
    Ok((interest_due, outstanding_principal))
}

pub async fn create(conn: &Connection, input: PaymentInput) -> Result<Payment, DbError> {
    let paid = NaiveDate::parse_from_str(&input.paid_date, "%Y-%m-%d")
        .map_err(|e| DbError::Msg(format!("bad paid_date: {e}")))?;

    // Determine allocation: manual override if both provided, else interest-first.
    let (interest_component, principal_component) =
        match (input.interest_component, input.principal_component) {
            (Some(i), Some(p)) => (round2(i), round2(p)),
            _ => {
                let (interest_due, outstanding) =
                    interest_due_as_of(conn, &input.loan_id, paid).await?;
                let alloc = allocate_interest_first(input.amount, interest_due, outstanding);
                (alloc.interest_component, alloc.principal_component)
            }
        };

    let id = new_id();
    let now = now_iso();
    conn.execute(
        "INSERT INTO payments (id, loan_id, amount, interest_component, principal_component, \
           paid_date, note, created_at, updated_at) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?8)",
        params![
            id.clone(),
            input.loan_id,
            input.amount,
            interest_component,
            principal_component,
            input.paid_date,
            input.note,
            now,
        ],
    )
    .await?;
    get(conn, &id).await?.ok_or(DbError::Msg("insert failed".into()))
}

pub async fn soft_delete(conn: &Connection, id: &str) -> Result<(), DbError> {
    let now = now_iso();
    conn.execute(
        "UPDATE payments SET deleted_at = ?2, updated_at = ?2 WHERE id = ?1",
        params![id, now],
    )
    .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{BorrowerInput, LoanInput};
    use crate::{repo_borrowers, repo_loans};

    async fn test_conn() -> Connection {
        let db = libsql::Builder::new_local(":memory:").build().await.unwrap();
        let conn = db.connect().unwrap();
        crate::migrations::run_migrations(&conn).await.unwrap();
        conn
    }

    #[tokio::test]
    async fn payment_allocates_interest_first() {
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
        // 10,000 @ 2%/mo, one-time, started 5 months before "now".
        let loan = repo_loans::create(
            &conn,
            LoanInput {
                borrower_id: b.id,
                principal: 10_000.0,
                monthly_rate: 0.02,
                interest_type: "simple".into(),
                repayment_mode: "one_time".into(),
                term_months: Some(12),
                start_date: "2026-01-10".into(),
                note: None,
            },
        )
        .await
        .unwrap();

        // As of 2026-04-10 => 3 whole months => interest = 10000*0.02*3 = 600.
        // Pay 1000: 600 to interest, 400 to principal.
        let p = create(
            &conn,
            PaymentInput {
                loan_id: loan.id,
                amount: 1_000.0,
                interest_component: None,
                principal_component: None,
                paid_date: "2026-04-10".into(),
                note: None,
            },
        )
        .await
        .unwrap();
        assert_eq!(p.interest_component, 600.0);
        assert_eq!(p.principal_component, 400.0);
    }
}
