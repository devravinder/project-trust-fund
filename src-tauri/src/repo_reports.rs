//! Reports (by-time monthly, by-person) on the in-memory dataset.

use std::collections::BTreeMap;

use serde::Serialize;

use crate::interest::round2;
use crate::data::Dataset;

#[derive(Debug, Clone, Serialize)]
pub struct MonthlyPoint {
    pub month: String,
    pub interest_collected: f64,
    pub principal_recovered: f64,
    pub total_collected: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct PersonReport {
    pub borrower_id: String,
    pub borrower_name: String,
    pub total_lent: f64,
    pub principal_recovered: f64,
    pub interest_collected: f64,
    pub outstanding_principal: f64,
}

fn not_deleted_loan(d: &Dataset, id: &str) -> bool {
    !d.deleted_loans.contains(&id.to_string())
}
fn not_deleted_payment(d: &Dataset, id: &str) -> bool {
    !d.deleted_payments.contains(&id.to_string())
}

pub fn monthly(d: &Dataset) -> Vec<MonthlyPoint> {
    // month (YYYY-MM) -> (interest, principal)
    let mut map: BTreeMap<String, (f64, f64)> = BTreeMap::new();
    for p in d
        .payments
        .iter()
        .filter(|p| not_deleted_payment(d, &p.id) && not_deleted_loan(d, &p.loan_id))
    {
        let month = p.paid_date.get(0..7).unwrap_or("").to_string();
        let entry = map.entry(month).or_insert((0.0, 0.0));
        entry.0 += p.interest_component;
        entry.1 += p.principal_component;
    }
    map.into_iter()
        .map(|(month, (interest, principal))| MonthlyPoint {
            month,
            interest_collected: round2(interest),
            principal_recovered: round2(principal),
            total_collected: round2(interest + principal),
        })
        .collect()
}

pub fn by_person(d: &Dataset) -> Vec<PersonReport> {
    let mut out: Vec<PersonReport> = d
        .borrowers
        .iter()
        .filter(|b| !d.deleted_borrowers.contains(&b.id))
        .map(|b| {
            let loans: Vec<_> = d
                .loans
                .iter()
                .filter(|l| l.borrower_id == b.id && not_deleted_loan(d, &l.id))
                .collect();
            let total_lent: f64 = loans.iter().map(|l| l.principal).sum();
            let principal_recovered: f64 = d
                .payments
                .iter()
                .filter(|p| {
                    not_deleted_payment(d, &p.id) && loans.iter().any(|l| l.id == p.loan_id)
                })
                .map(|p| p.principal_component)
                .sum();
            let interest_collected: f64 = d
                .payments
                .iter()
                .filter(|p| {
                    not_deleted_payment(d, &p.id) && loans.iter().any(|l| l.id == p.loan_id)
                })
                .map(|p| p.interest_component)
                .sum();
            PersonReport {
                borrower_id: b.id.clone(),
                borrower_name: b.name.clone(),
                total_lent: round2(total_lent),
                principal_recovered: round2(principal_recovered),
                interest_collected: round2(interest_collected),
                outstanding_principal: round2(total_lent - principal_recovered),
            }
        })
        .collect();
    out.sort_by(|a, b| {
        b.total_lent
            .partial_cmp(&a.total_lent)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{BorrowerInput, LoanInput, PaymentInput};
    use crate::{repo_borrowers, repo_loans, repo_payments};

    #[test]
    fn monthly_and_person_reports() {
        let mut d = Dataset::default();
        let b = repo_borrowers::create(
            &mut d,
            BorrowerInput {
                name: "Asha".into(),
                phone: None,
                address: None,
                photo_path: None,
                notes: None,
            },
        );
        let loan = repo_loans::create(
            &mut d,
            LoanInput {
                borrower_id: b.id.clone(),
                principal: 5_000.0,
                monthly_rate: 0.0,
                interest_type: "simple".into(),
                repayment_mode: "one_time".into(),
                end_date: Some("2026-07-01".into()),
                start_date: "2026-01-01".into(),
                note: None,
            },
        )
        .unwrap();
        repo_payments::create(
            &mut d,
            PaymentInput {
                loan_id: loan.id,
                amount: 1_000.0,
                interest_component: Some(0.0),
                principal_component: Some(1_000.0),
                paid_date: "2026-03-05".into(),
                note: None,
            },
        )
        .unwrap();

        let m = monthly(&d);
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].month, "2026-03");
        assert_eq!(m[0].principal_recovered, 1_000.0);

        let people = by_person(&d);
        assert_eq!(people.len(), 1);
        assert_eq!(people[0].total_lent, 5_000.0);
        assert_eq!(people[0].outstanding_principal, 4_000.0);
    }
}
