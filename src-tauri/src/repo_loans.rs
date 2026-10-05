//! Loan operations on the in-memory dataset.

use chrono::{Datelike, Local, NaiveDate};
use serde::Serialize;

use crate::interest::{
    effective_months, generate_schedule_simple, round2, simple_interest, whole_months_between,
};
use crate::models::{Loan, LoanInput, ScheduleItem};
use crate::data::Dataset;
use crate::util::{new_id, now_iso};

#[derive(Debug, Clone, Serialize)]
pub struct LoanSummary {
    #[serde(flatten)]
    pub loan: Loan,
    pub borrower_name: String,
    pub principal_recovered: f64,
    pub interest_collected: f64,
    pub outstanding_principal: f64,
    /// Interest accrued from start date to today (whole-month, frozen at term).
    pub interest_accrued_to_date: f64,
    /// Accrued interest not yet collected.
    pub interest_due: f64,
}

fn is_active_loan(d: &Dataset, id: &str) -> bool {
    !d.deleted_loans.contains(&id.to_string())
}

fn borrower_name(d: &Dataset, borrower_id: &str) -> String {
    d.borrowers
        .iter()
        .find(|b| b.id == borrower_id)
        .map(|b| b.name.clone())
        .unwrap_or_default()
}

fn principal_recovered(d: &Dataset, loan_id: &str) -> f64 {
    d.payments
        .iter()
        .filter(|p| p.loan_id == loan_id && !d.deleted_payments.contains(&p.id))
        .map(|p| p.principal_component)
        .sum()
}

fn interest_collected(d: &Dataset, loan_id: &str) -> f64 {
    d.payments
        .iter()
        .filter(|p| p.loan_id == loan_id && !d.deleted_payments.contains(&p.id))
        .map(|p| p.interest_component)
        .sum()
}

/// Interest accrued on a loan from its start date to `as_of` (whole months,
/// capped at term). v1: simple interest only.
pub fn interest_accrued(loan: &Loan, as_of: NaiveDate) -> f64 {
    let start = match NaiveDate::parse_from_str(&loan.start_date, "%Y-%m-%d") {
        Ok(d) => d,
        Err(_) => return 0.0,
    };
    let elapsed = whole_months_between(start, as_of);
    let months = effective_months(elapsed, loan.term_months);
    simple_interest(loan.principal, loan.monthly_rate, months)
}

fn summarize(d: &Dataset, loan: &Loan) -> LoanSummary {
    let recovered = round2(principal_recovered(d, &loan.id));
    let collected = round2(interest_collected(d, &loan.id));
    let accrued = interest_accrued(loan, Local::now().date_naive());
    LoanSummary {
        borrower_name: borrower_name(d, &loan.borrower_id),
        principal_recovered: recovered,
        interest_collected: collected,
        outstanding_principal: round2(loan.principal - recovered),
        interest_accrued_to_date: accrued,
        interest_due: round2((accrued - collected).max(0.0)),
        loan: loan.clone(),
    }
}

pub fn list(d: &Dataset, status: Option<&str>, search: Option<&str>) -> Vec<LoanSummary> {
    let term = search.map(|s| s.trim().to_lowercase()).unwrap_or_default();
    let mut out: Vec<LoanSummary> = d
        .loans
        .iter()
        .filter(|l| is_active_loan(d, &l.id))
        .filter(|l| match status {
            Some(s) if s != "all" => l.status == s,
            _ => true,
        })
        .filter(|l| {
            if term.is_empty() {
                true
            } else {
                borrower_name(d, &l.borrower_id)
                    .to_lowercase()
                    .contains(&term)
            }
        })
        .map(|l| summarize(d, l))
        .collect();
    out.sort_by(|a, b| b.loan.created_at.cmp(&a.loan.created_at));
    out
}

pub fn get(d: &Dataset, id: &str) -> Option<Loan> {
    d.loans
        .iter()
        .find(|l| l.id == id && is_active_loan(d, &l.id))
        .cloned()
}

pub fn get_summary(d: &Dataset, id: &str) -> Option<LoanSummary> {
    d.loans
        .iter()
        .find(|l| l.id == id && is_active_loan(d, &l.id))
        .map(|l| summarize(d, l))
}

fn last_day_of_month(year: i32, month: u32) -> u32 {
    let (ny, nm) = if month == 12 {
        (year + 1, 1)
    } else {
        (year, month + 1)
    };
    NaiveDate::from_ymd_opt(ny, nm, 1)
        .unwrap()
        .pred_opt()
        .unwrap()
        .day()
}

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
    let day = date.day().min(last_day_of_month(y, month));
    NaiveDate::from_ymd_opt(y, month, day).unwrap()
}

pub fn create(d: &mut Dataset, input: LoanInput) -> Result<Loan, String> {
    let now = now_iso();
    let loan = Loan {
        id: new_id(),
        borrower_id: input.borrower_id,
        principal: input.principal,
        monthly_rate: input.monthly_rate,
        interest_type: input.interest_type,
        repayment_mode: input.repayment_mode.clone(),
        term_months: input.term_months,
        start_date: input.start_date.clone(),
        status: "active".into(),
        note: input.note,
        created_at: now.clone(),
        updated_at: now.clone(),
    };

    // Generate installment schedule if applicable (v1: simple interest).
    if input.repayment_mode == "installments" {
        if let Some(n) = input.term_months {
            let start = NaiveDate::parse_from_str(&input.start_date, "%Y-%m-%d")
                .map_err(|e| format!("bad start_date: {e}"))?;
            for r in generate_schedule_simple(input.principal, input.monthly_rate, n) {
                let due = add_months(start, r.seq);
                d.schedule.push(ScheduleItem {
                    id: new_id(),
                    loan_id: loan.id.clone(),
                    seq: r.seq,
                    due_date: due.format("%Y-%m-%d").to_string(),
                    principal_due: r.principal_due,
                    interest_due: r.interest_due,
                    total_due: r.total_due,
                    status: "pending".into(),
                });
            }
        }
    }

    d.loans.push(loan.clone());
    Ok(loan)
}

pub fn set_status(d: &mut Dataset, id: &str, status: &str) {
    if let Some(l) = d.loans.iter_mut().find(|l| l.id == id) {
        l.status = status.to_string();
        l.updated_at = now_iso();
    }
}

pub fn soft_delete(d: &mut Dataset, id: &str) {
    if !d.deleted_loans.iter().any(|x| x == id) {
        d.deleted_loans.push(id.to_string());
    }
}

pub fn schedule(d: &Dataset, loan_id: &str) -> Vec<ScheduleItem> {
    let mut rows: Vec<ScheduleItem> = d
        .schedule
        .iter()
        .filter(|s| s.loan_id == loan_id)
        .cloned()
        .collect();
    rows.sort_by_key(|s| s.seq);
    rows
}

/// Recompute installment schedule statuses against actual payments.
pub fn refresh_schedule_status(d: &mut Dataset, loan_id: &str) {
    let mut remaining: f64 = d
        .payments
        .iter()
        .filter(|p| p.loan_id == loan_id && !d.deleted_payments.contains(&p.id))
        .map(|p| p.amount)
        .sum();

    let today = Local::now().date_naive();
    let mut seqs: Vec<i64> = d
        .schedule
        .iter()
        .filter(|s| s.loan_id == loan_id)
        .map(|s| s.seq)
        .collect();
    seqs.sort_unstable();

    for seq in seqs {
        if let Some(item) = d
            .schedule
            .iter_mut()
            .find(|s| s.loan_id == loan_id && s.seq == seq)
        {
            let total = item.total_due;
            item.status = if remaining >= total - 0.005 {
                remaining = round2(remaining - total);
                "paid".into()
            } else if remaining > 0.005 {
                remaining = 0.0;
                "partial".into()
            } else {
                let due = NaiveDate::parse_from_str(&item.due_date, "%Y-%m-%d").unwrap_or(today);
                if due < today {
                    "overdue".into()
                } else {
                    "pending".into()
                }
            };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::BorrowerInput;
    use crate::repo_borrowers;

    #[test]
    fn create_installment_loan_generates_schedule() {
        let mut d = Dataset::default();
        let b = repo_borrowers::create(
            &mut d,
            BorrowerInput {
                name: "Test".into(),
                phone: None,
                address: None,
                photo_path: None,
                notes: None,
            },
        );
        let loan = create(
            &mut d,
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
        .unwrap();

        let sched = schedule(&d, &loan.id);
        assert_eq!(sched.len(), 5);
        assert_eq!(sched[0].interest_due, 200.0);
        assert_eq!(sched[0].due_date, "2026-02-15");
        let total_principal: f64 = sched.iter().map(|s| s.principal_due).sum();
        assert_eq!(round2(total_principal), 10_000.0);
    }

    #[test]
    fn interest_accrued_to_date_simple() {
        let loan = Loan {
            id: "l1".into(),
            borrower_id: "b1".into(),
            principal: 10_000.0,
            monthly_rate: 0.02,
            interest_type: "simple".into(),
            repayment_mode: "one_time".into(),
            term_months: Some(12),
            start_date: "2026-01-10".into(),
            status: "active".into(),
            note: None,
            created_at: "x".into(),
            updated_at: "x".into(),
        };
        // 3 whole months -> 10000 * 0.02 * 3 = 600
        let as_of = NaiveDate::from_ymd_opt(2026, 4, 10).unwrap();
        assert_eq!(interest_accrued(&loan, as_of), 600.0);
    }
}
