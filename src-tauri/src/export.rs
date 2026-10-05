//! CSV export of core data for data-safety / backup, from the in-memory dataset.

use crate::data::Dataset;

fn esc(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

fn opt(s: &Option<String>) -> String {
    esc(s.as_deref().unwrap_or(""))
}

pub fn borrowers_csv(d: &Dataset) -> String {
    let mut out = String::from("id,name,phone,address,notes,created_at\n");
    for b in d
        .borrowers
        .iter()
        .filter(|b| !d.deleted_borrowers.contains(&b.id))
    {
        out.push_str(&format!(
            "{},{},{},{},{},{}\n",
            esc(&b.id),
            esc(&b.name),
            opt(&b.phone),
            opt(&b.address),
            opt(&b.notes),
            esc(&b.created_at),
        ));
    }
    out
}

pub fn loans_csv(d: &Dataset) -> String {
    let mut out = String::from(
        "id,borrower,principal,monthly_rate,interest_type,repayment_mode,end_date,start_date,status,created_at\n",
    );
    for l in d.loans.iter().filter(|l| !d.deleted_loans.contains(&l.id)) {
        let borrower = d
            .borrowers
            .iter()
            .find(|b| b.id == l.borrower_id)
            .map(|b| b.name.clone())
            .unwrap_or_default();
        out.push_str(&format!(
            "{},{},{},{},{},{},{},{},{},{}\n",
            esc(&l.id),
            esc(&borrower),
            l.principal,
            l.monthly_rate,
            esc(&l.interest_type),
            esc(&l.repayment_mode),
            esc(l.end_date.as_deref().unwrap_or("")),
            esc(&l.start_date),
            esc(&l.status),
            esc(&l.created_at),
        ));
    }
    out
}

pub fn payments_csv(d: &Dataset) -> String {
    let mut out = String::from("id,borrower,amount,interest,principal,paid_date,note,created_at\n");
    for p in d
        .payments
        .iter()
        .filter(|p| !d.deleted_payments.contains(&p.id))
    {
        let borrower = d
            .loans
            .iter()
            .find(|l| l.id == p.loan_id)
            .and_then(|l| d.borrowers.iter().find(|b| b.id == l.borrower_id))
            .map(|b| b.name.clone())
            .unwrap_or_default();
        out.push_str(&format!(
            "{},{},{},{},{},{},{},{}\n",
            esc(&p.id),
            esc(&borrower),
            p.amount,
            p.interest_component,
            p.principal_component,
            esc(&p.paid_date),
            opt(&p.note),
            esc(&p.created_at),
        ));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::BorrowerInput;
    use crate::repo_borrowers;

    #[test]
    fn borrowers_export_has_header_and_escaping() {
        let mut d = Dataset::default();
        repo_borrowers::create(
            &mut d,
            BorrowerInput {
                name: "A, B".into(),
                phone: None,
                address: None,
                photo_path: None,
                notes: None,
            },
        );
        let csv = borrowers_csv(&d);
        assert!(csv.starts_with("id,name,phone"));
        assert!(csv.contains("\"A, B\""));
    }
}
