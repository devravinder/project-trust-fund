//! Interest & repayment computations.
//!
//! v1 implements **simple interest** (monthly rate, whole-month accrual,
//! pro-rated on early payoff, frozen at term). Zero interest (rate = 0) is
//! supported and simply yields zero interest. Compound is deferred to v2.
//!
//! See docs/interest-logic.md for the authoritative rules.

use chrono::{Datelike, NaiveDate};

/// Round to 2 decimal places (money).
pub fn round2(v: f64) -> f64 {
    (v * 100.0).round() / 100.0
}

/// Number of whole months elapsed from `start` to `as_of` (anniversary based).
/// A month counts only once the day-of-month anniversary is reached.
pub fn whole_months_between(start: NaiveDate, as_of: NaiveDate) -> i64 {
    if as_of <= start {
        return 0;
    }
    let mut months =
        (as_of.year() - start.year()) as i64 * 12 + (as_of.month() as i64 - start.month() as i64);
    // If the day hasn't reached the anniversary day, subtract one.
    if as_of.day() < start.day() {
        months -= 1;
    }
    months.max(0)
}

/// Effective months for interest: elapsed whole months, capped at the term
/// (interest freezes at term end). If no term, uses elapsed months.
pub fn effective_months(elapsed: i64, term_months: Option<i64>) -> i64 {
    match term_months {
        Some(t) => elapsed.min(t),
        None => elapsed,
    }
}

/// Simple interest accrued on the original principal over `months`.
/// interest = principal * monthly_rate * months
pub fn simple_interest(principal: f64, monthly_rate: f64, months: i64) -> f64 {
    round2(principal * monthly_rate * months as f64)
}

/// One installment row (Option B: fixed principal + interest on balance).
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduleRow {
    pub seq: i64,
    pub principal_due: f64,
    pub interest_due: f64,
    pub total_due: f64,
}

/// Generate an installment schedule using Option B for a **simple** loan:
/// - principal split equally across `n` months
/// - each month's interest = remaining principal * monthly_rate
///
/// Any rounding remainder in principal is added to the final installment so
/// the principal sums exactly to the original.
pub fn generate_schedule_simple(
    principal: f64,
    monthly_rate: f64,
    n: i64,
) -> Vec<ScheduleRow> {
    let mut rows = Vec::new();
    if n <= 0 {
        return rows;
    }
    let per_principal = round2(principal / n as f64);
    let mut remaining = principal;
    let mut allocated = 0.0;

    for seq in 1..=n {
        // Interest on the opening balance for this period.
        let interest = round2(remaining * monthly_rate);
        // Last installment absorbs the rounding remainder.
        let principal_due = if seq == n {
            round2(principal - allocated)
        } else {
            per_principal
        };
        allocated = round2(allocated + principal_due);
        remaining = round2(remaining - principal_due);
        rows.push(ScheduleRow {
            seq,
            principal_due,
            interest_due: interest,
            total_due: round2(principal_due + interest),
        });
    }
    rows
}

/// Result of allocating a payment (interest-first).
#[derive(Debug, Clone, PartialEq)]
pub struct Allocation {
    pub interest_component: f64,
    pub principal_component: f64,
}

/// Allocate a payment interest-first: cover `interest_due` first, remainder to
/// principal. Neither component exceeds what's owed where applicable.
pub fn allocate_interest_first(
    amount: f64,
    interest_due: f64,
    outstanding_principal: f64,
) -> Allocation {
    let interest = amount.min(interest_due).max(0.0);
    let remainder = round2(amount - interest);
    let principal = remainder.min(outstanding_principal).max(0.0);
    Allocation {
        interest_component: round2(interest),
        principal_component: round2(principal),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn months_anniversary_based() {
        let start = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();
        assert_eq!(
            whole_months_between(start, NaiveDate::from_ymd_opt(2026, 2, 14).unwrap()),
            0
        );
        assert_eq!(
            whole_months_between(start, NaiveDate::from_ymd_opt(2026, 2, 15).unwrap()),
            1
        );
        assert_eq!(
            whole_months_between(start, NaiveDate::from_ymd_opt(2026, 6, 20).unwrap()),
            5
        );
    }

    #[test]
    fn simple_interest_and_zero_rate() {
        // 10,000 at 2%/month for 5 months = 1,000
        assert_eq!(simple_interest(10_000.0, 0.02, 5), 1_000.0);
        // zero interest
        assert_eq!(simple_interest(10_000.0, 0.0, 5), 0.0);
    }

    #[test]
    fn effective_months_freezes_at_term() {
        assert_eq!(effective_months(8, Some(5)), 5);
        assert_eq!(effective_months(3, Some(5)), 3);
        assert_eq!(effective_months(8, None), 8);
    }

    #[test]
    fn schedule_matches_docs_example() {
        // 10,000, 2%/month, 5 months (docs example).
        let rows = generate_schedule_simple(10_000.0, 0.02, 5);
        assert_eq!(rows.len(), 5);
        assert_eq!(rows[0].interest_due, 200.0);
        assert_eq!(rows[0].principal_due, 2_000.0);
        assert_eq!(rows[4].interest_due, 40.0);
        let total_principal: f64 = rows.iter().map(|r| r.principal_due).sum();
        assert_eq!(round2(total_principal), 10_000.0);
        let total_interest: f64 = rows.iter().map(|r| r.interest_due).sum();
        assert_eq!(round2(total_interest), 600.0);
    }

    #[test]
    fn allocation_interest_first() {
        // Pay 500 when 200 interest due, 10,000 principal outstanding.
        let a = allocate_interest_first(500.0, 200.0, 10_000.0);
        assert_eq!(a.interest_component, 200.0);
        assert_eq!(a.principal_component, 300.0);

        // Pay less than interest due.
        let a = allocate_interest_first(150.0, 200.0, 10_000.0);
        assert_eq!(a.interest_component, 150.0);
        assert_eq!(a.principal_component, 0.0);

        // Overpay beyond principal.
        let a = allocate_interest_first(11_000.0, 200.0, 10_000.0);
        assert_eq!(a.interest_component, 200.0);
        assert_eq!(a.principal_component, 10_000.0);
    }
}
