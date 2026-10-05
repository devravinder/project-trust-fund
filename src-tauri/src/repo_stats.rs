//! Dashboard statistics and dues-this-month on the in-memory dataset.

use chrono::{Datelike, Local, NaiveDate};
use serde::Serialize;

use crate::interest::round2;
use crate::repo_loans::interest_accrued;
use crate::data::Dataset;

#[derive(Debug, Clone, Serialize)]
pub struct DashboardSummary {
    pub total_lent: f64,
    pub outstanding_principal: f64,
    pub active_loans: i64,
    pub overdue_loans: i64,
    pub overdue_amount: f64,
    pub interest_collected: f64,
    pub interest_due_this_month: f64,
    pub principal_recovered: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct DueItem {
    pub loan_id: String,
    pub borrower_name: String,
    pub due_date: String,
    pub total_due: f64,
    pub status: String,
}

fn not_deleted_loan(d: &Dataset, id: &str) -> bool {
    !d.deleted_loans.contains(&id.to_string())
}
fn not_deleted_payment(d: &Dataset, id: &str) -> bool {
    !d.deleted_payments.contains(&id.to_string())
}

pub fn summary(d: &Dataset) -> DashboardSummary {
    let today = Local::now().date_naive();

    let active_or_overdue: Vec<_> = d
        .loans
        .iter()
        .filter(|l| not_deleted_loan(d, &l.id) && (l.status == "active" || l.status == "overdue"))
        .collect();

    let total_lent: f64 = active_or_overdue.iter().map(|l| l.principal).sum();

    let principal_recovered: f64 = d
        .payments
        .iter()
        .filter(|p| not_deleted_payment(d, &p.id) && not_deleted_loan(d, &p.loan_id))
        .map(|p| p.principal_component)
        .sum();

    let interest_collected: f64 = d
        .payments
        .iter()
        .filter(|p| not_deleted_payment(d, &p.id) && not_deleted_loan(d, &p.loan_id))
        .map(|p| p.interest_component)
        .sum();

    let active_loans = d
        .loans
        .iter()
        .filter(|l| not_deleted_loan(d, &l.id) && l.status == "active")
        .count() as i64;

    let mut overdue_loans = 0i64;
    let mut overdue_amount = 0.0f64;
    let mut interest_due_total = 0.0f64;

    for l in &active_or_overdue {
        let accrued = interest_accrued(l, today);
        let collected: f64 = d
            .payments
            .iter()
            .filter(|p| p.loan_id == l.id && not_deleted_payment(d, &p.id))
            .map(|p| p.interest_component)
            .sum();
        let interest_due = (accrued - collected).max(0.0);
        interest_due_total += interest_due;

        let principal_paid: f64 = d
            .payments
            .iter()
            .filter(|p| p.loan_id == l.id && not_deleted_payment(d, &p.id))
            .map(|p| p.principal_component)
            .sum();
        let outstanding = (l.principal - principal_paid).max(0.0);

        if let Some(term) = l.term_months {
            if let Ok(start) = NaiveDate::parse_from_str(&l.start_date, "%Y-%m-%d") {
                let elapsed = crate::interest::whole_months_between(start, today);
                if elapsed >= term && outstanding > 0.0 {
                    overdue_loans += 1;
                    overdue_amount += outstanding + interest_due;
                }
            }
        }
    }

    DashboardSummary {
        total_lent: round2(total_lent),
        outstanding_principal: round2(total_lent - principal_recovered),
        active_loans,
        overdue_loans,
        overdue_amount: round2(overdue_amount),
        interest_collected: round2(interest_collected),
        interest_due_this_month: round2(interest_due_total),
        principal_recovered: round2(principal_recovered),
    }
}

pub fn dues_this_month(d: &Dataset, limit: usize) -> Vec<DueItem> {
    let today = Local::now().date_naive();
    let month_start = NaiveDate::from_ymd_opt(today.year(), today.month(), 1).unwrap();
    let (ny, nm) = if today.month() == 12 {
        (today.year() + 1, 1)
    } else {
        (today.year(), today.month() + 1)
    };
    let next_month = NaiveDate::from_ymd_opt(ny, nm, 1).unwrap();

    let mut items: Vec<DueItem> = d
        .schedule
        .iter()
        .filter(|s| {
            not_deleted_loan(d, &s.loan_id)
                && matches!(s.status.as_str(), "pending" | "partial" | "overdue")
        })
        .filter_map(|s| {
            let due = NaiveDate::parse_from_str(&s.due_date, "%Y-%m-%d").ok()?;
            if due >= month_start && due < next_month {
                let loan = d.loans.iter().find(|l| l.id == s.loan_id)?;
                let borrower_name = d
                    .borrowers
                    .iter()
                    .find(|b| b.id == loan.borrower_id)
                    .map(|b| b.name.clone())
                    .unwrap_or_default();
                Some(DueItem {
                    loan_id: s.loan_id.clone(),
                    borrower_name,
                    due_date: s.due_date.clone(),
                    total_due: s.total_due,
                    status: s.status.clone(),
                })
            } else {
                None
            }
        })
        .collect();
    items.sort_by(|a, b| a.due_date.cmp(&b.due_date));
    items.truncate(limit);
    items
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{BorrowerInput, LoanInput, PaymentInput};
    use crate::{repo_borrowers, repo_loans, repo_payments};

    #[test]
    fn summary_reflects_loans_and_payments() {
        let mut d = Dataset::default();
        let b = repo_borrowers::create(
            &mut d,
            BorrowerInput {
                name: "T".into(),
                phone: None,
                address: None,
                photo_path: None,
                notes: None,
            },
        );
        let loan = repo_loans::create(
            &mut d,
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
        .unwrap();
        repo_payments::create(
            &mut d,
            PaymentInput {
                loan_id: loan.id,
                amount: 2_000.0,
                interest_component: Some(0.0),
                principal_component: Some(2_000.0),
                paid_date: "2020-02-01".into(),
                note: None,
            },
        )
        .unwrap();

        let s = summary(&d);
        assert_eq!(s.total_lent, 10_000.0);
        assert_eq!(s.principal_recovered, 2_000.0);
        assert_eq!(s.outstanding_principal, 8_000.0);
        assert_eq!(s.active_loans, 1);
    }
}
