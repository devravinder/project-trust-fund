//! Payment operations on the in-memory dataset.

use chrono::NaiveDate;
use serde::Serialize;

use crate::interest::{allocate_interest_first, round2};
use crate::models::{Payment, PaymentInput};
use crate::repo_loans;
use crate::data::Dataset;
use crate::util::{new_id, now_iso};

#[derive(Debug, Clone, Serialize)]
pub struct PaymentView {
    #[serde(flatten)]
    pub payment: Payment,
    pub borrower_name: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct PaymentDetail {
    #[serde(flatten)]
    pub payment: Payment,
    pub borrower_name: String,
    pub loan_principal: f64,
    pub loan_start_date: String,
}

fn active(d: &Dataset) -> impl Iterator<Item = &Payment> {
    d.payments
        .iter()
        .filter(|p| !d.deleted_payments.contains(&p.id))
}

fn borrower_name_for_loan(d: &Dataset, loan_id: &str) -> String {
    let borrower_id = d
        .loans
        .iter()
        .find(|l| l.id == loan_id)
        .map(|l| l.borrower_id.clone())
        .unwrap_or_default();
    d.borrowers
        .iter()
        .find(|b| b.id == borrower_id)
        .map(|b| b.name.clone())
        .unwrap_or_default()
}

pub fn list(d: &Dataset, search: Option<&str>) -> Vec<PaymentView> {
    let term = search.map(|s| s.trim().to_lowercase()).unwrap_or_default();
    let mut out: Vec<PaymentView> = active(d)
        .map(|p| PaymentView {
            payment: p.clone(),
            borrower_name: borrower_name_for_loan(d, &p.loan_id),
        })
        .filter(|v| {
            term.is_empty() || v.borrower_name.to_lowercase().contains(&term)
        })
        .collect();
    out.sort_by(|a, b| b.payment.paid_date.cmp(&a.payment.paid_date));
    out
}

/// Single payment with borrower + loan context for the detail view.
pub fn get_detail(d: &Dataset, id: &str) -> Option<PaymentDetail> {
    let p = active(d).find(|p| p.id == id)?.clone();
    let loan = d.loans.iter().find(|l| l.id == p.loan_id);
    let (loan_principal, loan_start_date) = loan
        .map(|l| (l.principal, l.start_date.clone()))
        .unwrap_or((0.0, String::new()));
    Some(PaymentDetail {
        borrower_name: borrower_name_for_loan(d, &p.loan_id),
        loan_principal,
        loan_start_date,
        payment: p,
    })
}

pub fn list_for_loan(d: &Dataset, loan_id: &str) -> Vec<Payment> {
    let mut out: Vec<Payment> = active(d)
        .filter(|p| p.loan_id == loan_id)
        .cloned()
        .collect();
    out.sort_by(|a, b| b.paid_date.cmp(&a.paid_date));
    out
}

/// Interest due and outstanding principal on a loan as of `as_of`.
fn interest_due_and_outstanding(d: &Dataset, loan_id: &str, as_of: NaiveDate) -> (f64, f64) {
    let loan = match d.loans.iter().find(|l| l.id == loan_id) {
        Some(l) => l,
        None => return (0.0, 0.0),
    };
    let accrued = repo_loans::interest_accrued(loan, as_of);
    let collected: f64 = active(d)
        .filter(|p| p.loan_id == loan_id)
        .map(|p| p.interest_component)
        .sum();
    let principal_paid: f64 = active(d)
        .filter(|p| p.loan_id == loan_id)
        .map(|p| p.principal_component)
        .sum();
    (
        round2((accrued - collected).max(0.0)),
        round2((loan.principal - principal_paid).max(0.0)),
    )
}

pub fn create(d: &mut Dataset, input: PaymentInput) -> Result<Payment, String> {
    let paid = NaiveDate::parse_from_str(&input.paid_date, "%Y-%m-%d")
        .map_err(|e| format!("bad paid_date: {e}"))?;

    let (interest_component, principal_component) =
        match (input.interest_component, input.principal_component) {
            (Some(i), Some(p)) => (round2(i), round2(p)),
            _ => {
                let (interest_due, outstanding) =
                    interest_due_and_outstanding(d, &input.loan_id, paid);
                let alloc = allocate_interest_first(input.amount, interest_due, outstanding);
                (alloc.interest_component, alloc.principal_component)
            }
        };

    let now = now_iso();
    let payment = Payment {
        id: new_id(),
        loan_id: input.loan_id.clone(),
        amount: input.amount,
        interest_component,
        principal_component,
        paid_date: input.paid_date,
        note: input.note,
        created_at: now.clone(),
        updated_at: now,
    };
    d.payments.push(payment.clone());
    repo_loans::refresh_schedule_status(d, &input.loan_id);
    Ok(payment)
}

pub fn soft_delete(d: &mut Dataset, id: &str) {
    let loan_id = d
        .payments
        .iter()
        .find(|p| p.id == id)
        .map(|p| p.loan_id.clone());
    if !d.deleted_payments.iter().any(|x| x == id) {
        d.deleted_payments.push(id.to_string());
    }
    if let Some(lid) = loan_id {
        repo_loans::refresh_schedule_status(d, &lid);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{BorrowerInput, LoanInput};
    use crate::{repo_borrowers, repo_loans};

    #[test]
    fn payment_allocates_interest_first() {
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
                end_date: Some("2027-01-10".into()),
                start_date: "2026-01-10".into(),
                note: None,
            },
        )
        .unwrap();

        // As of 2026-04-10 -> 3 months -> interest 600. Pay 1000.
        let p = create(
            &mut d,
            PaymentInput {
                loan_id: loan.id,
                amount: 1_000.0,
                interest_component: None,
                principal_component: None,
                paid_date: "2026-04-10".into(),
                note: None,
            },
        )
        .unwrap();
        assert_eq!(p.interest_component, 600.0);
        assert_eq!(p.principal_component, 400.0);
    }
}
