//! Borrower operations on the in-memory dataset.

use crate::interest::round2;
use crate::models::{Borrower, BorrowerInput};
use crate::data::Dataset;
use crate::util::{new_id, now_iso};

fn active(d: &Dataset) -> impl Iterator<Item = &Borrower> {
    d.borrowers
        .iter()
        .filter(|b| !d.deleted_borrowers.contains(&b.id))
}

pub fn list(d: &Dataset, search: Option<&str>) -> Vec<Borrower> {
    let term = search.map(|s| s.trim().to_lowercase()).unwrap_or_default();
    let mut out: Vec<Borrower> = active(d)
        .filter(|b| {
            if term.is_empty() {
                true
            } else {
                b.name.to_lowercase().contains(&term)
                    || b
                        .phone
                        .as_deref()
                        .map(|p| p.to_lowercase().contains(&term))
                        .unwrap_or(false)
            }
        })
        .cloned()
        .collect();
    out.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    out
}

pub fn get(d: &Dataset, id: &str) -> Option<Borrower> {
    active(d).find(|b| b.id == id).cloned()
}

pub fn create(d: &mut Dataset, input: BorrowerInput) -> Borrower {
    let now = now_iso();
    let b = Borrower {
        id: new_id(),
        name: input.name,
        phone: input.phone,
        address: input.address,
        photo_path: input.photo_path,
        notes: input.notes,
        created_at: now.clone(),
        updated_at: now,
    };
    d.borrowers.push(b.clone());
    b
}

pub fn update(d: &mut Dataset, id: &str, input: BorrowerInput) -> Option<Borrower> {
    let now = now_iso();
    if let Some(b) = d.borrowers.iter_mut().find(|b| b.id == id) {
        b.name = input.name;
        b.phone = input.phone;
        b.address = input.address;
        b.photo_path = input.photo_path;
        b.notes = input.notes;
        b.updated_at = now;
        Some(b.clone())
    } else {
        None
    }
}

pub fn soft_delete(d: &mut Dataset, id: &str) {
    if !d.deleted_borrowers.iter().any(|x| x == id) {
        d.deleted_borrowers.push(id.to_string());
    }
}

/// Total outstanding principal for a borrower across their active loans.
pub fn total_outstanding(d: &Dataset, borrower_id: &str) -> f64 {
    let active_loans: Vec<&crate::models::Loan> = d
        .loans
        .iter()
        .filter(|l| {
            l.borrower_id == borrower_id
                && !d.deleted_loans.contains(&l.id)
                && (l.status == "active" || l.status == "overdue")
        })
        .collect();
    let principal: f64 = active_loans.iter().map(|l| l.principal).sum();
    let paid: f64 = d
        .payments
        .iter()
        .filter(|p| {
            !d.deleted_payments.contains(&p.id)
                && active_loans.iter().any(|l| l.id == p.loan_id)
        })
        .map(|p| p.principal_component)
        .sum();
    round2(principal - paid)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn borrower_crud_and_search() {
        let mut d = Dataset::default();
        let created = create(
            &mut d,
            BorrowerInput {
                name: "Ramesh Kumar".into(),
                phone: Some("9876543210".into()),
                address: None,
                photo_path: None,
                notes: None,
            },
        );
        assert_eq!(created.name, "Ramesh Kumar");
        assert_eq!(list(&d, Some("rame")).len(), 1);

        let updated = update(
            &mut d,
            &created.id,
            BorrowerInput {
                name: "Ramesh K".into(),
                phone: Some("9876543210".into()),
                address: Some("Bengaluru".into()),
                photo_path: None,
                notes: None,
            },
        )
        .unwrap();
        assert_eq!(updated.address.as_deref(), Some("Bengaluru"));

        soft_delete(&mut d, &created.id);
        assert_eq!(list(&d, None).len(), 0);
    }
}
