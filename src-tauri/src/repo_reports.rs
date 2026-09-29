//! Reports: by-time (monthly) and by-person aggregates.

use libsql::Connection;

use crate::db::DbError;
use crate::interest::round2;
use crate::util::get_f64;

#[derive(Debug, Clone, serde::Serialize)]
pub struct MonthlyPoint {
    pub month: String, // YYYY-MM
    pub interest_collected: f64,
    pub principal_recovered: f64,
    pub total_collected: f64,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct PersonReport {
    pub borrower_id: String,
    pub borrower_name: String,
    pub total_lent: f64,
    pub principal_recovered: f64,
    pub interest_collected: f64,
    pub outstanding_principal: f64,
}

/// Interest + principal collected grouped by calendar month (payment date).
pub async fn monthly(conn: &Connection) -> Result<Vec<MonthlyPoint>, DbError> {
    let mut rows = conn
        .query(
            "SELECT substr(p.paid_date, 1, 7) AS ym, \
                COALESCE(SUM(p.interest_component), 0), \
                COALESCE(SUM(p.principal_component), 0) \
             FROM payments p JOIN loans l ON l.id = p.loan_id \
             WHERE p.deleted_at IS NULL AND l.deleted_at IS NULL \
             GROUP BY ym ORDER BY ym",
            (),
        )
        .await?;
    let mut out = Vec::new();
    while let Some(row) = rows.next().await? {
        let month: String = row.get(0)?;
        let interest = get_f64(&row, 1)?;
        let principal = get_f64(&row, 2)?;
        out.push(MonthlyPoint {
            month,
            interest_collected: round2(interest),
            principal_recovered: round2(principal),
            total_collected: round2(interest + principal),
        });
    }
    Ok(out)
}

/// Per-borrower totals: lent, recovered, interest, outstanding.
pub async fn by_person(conn: &Connection) -> Result<Vec<PersonReport>, DbError> {
    let mut rows = conn
        .query(
            "SELECT b.id, b.name, \
                COALESCE(SUM(DISTINCT_L.principal), 0) AS lent, \
                COALESCE(( \
                  SELECT SUM(p.principal_component) FROM payments p \
                  JOIN loans l2 ON l2.id = p.loan_id \
                  WHERE l2.borrower_id = b.id AND p.deleted_at IS NULL AND l2.deleted_at IS NULL \
                ), 0) AS principal_recovered, \
                COALESCE(( \
                  SELECT SUM(p.interest_component) FROM payments p \
                  JOIN loans l2 ON l2.id = p.loan_id \
                  WHERE l2.borrower_id = b.id AND p.deleted_at IS NULL AND l2.deleted_at IS NULL \
                ), 0) AS interest_collected \
             FROM borrowers b \
             LEFT JOIN ( \
               SELECT borrower_id, id, principal FROM loans WHERE deleted_at IS NULL \
             ) AS DISTINCT_L ON DISTINCT_L.borrower_id = b.id \
             WHERE b.deleted_at IS NULL \
             GROUP BY b.id, b.name \
             ORDER BY lent DESC",
            (),
        )
        .await?;
    let mut out = Vec::new();
    while let Some(row) = rows.next().await? {
        let borrower_id: String = row.get(0)?;
        let borrower_name: String = row.get(1)?;
        let total_lent = get_f64(&row, 2)?;
        let principal_recovered = get_f64(&row, 3)?;
        let interest_collected = get_f64(&row, 4)?;
        out.push(PersonReport {
            borrower_id,
            borrower_name,
            total_lent: round2(total_lent),
            principal_recovered: round2(principal_recovered),
            interest_collected: round2(interest_collected),
            outstanding_principal: round2(total_lent - principal_recovered),
        });
    }
    Ok(out)
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
    async fn monthly_and_person_reports() {
        let conn = test_conn().await;
        let b = repo_borrowers::create(
            &conn,
            BorrowerInput {
                name: "Asha".into(),
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
                borrower_id: b.id.clone(),
                principal: 5_000.0,
                monthly_rate: 0.0,
                interest_type: "simple".into(),
                repayment_mode: "one_time".into(),
                term_months: Some(6),
                start_date: "2026-01-01".into(),
                note: None,
            },
        )
        .await
        .unwrap();
        repo_payments::create(
            &conn,
            PaymentInput {
                loan_id: loan.id,
                amount: 1_000.0,
                interest_component: Some(0.0),
                principal_component: Some(1_000.0),
                paid_date: "2026-03-05".into(),
                note: None,
            },
        )
        .await
        .unwrap();

        let m = monthly(&conn).await.unwrap();
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].month, "2026-03");
        assert_eq!(m[0].principal_recovered, 1_000.0);

        let people = by_person(&conn).await.unwrap();
        assert_eq!(people.len(), 1);
        assert_eq!(people[0].total_lent, 5_000.0);
        assert_eq!(people[0].principal_recovered, 1_000.0);
        assert_eq!(people[0].outstanding_principal, 4_000.0);
    }
}
