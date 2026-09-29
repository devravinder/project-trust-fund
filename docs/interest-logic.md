# TrustFund — Interest & Repayment Logic

The financial core of the app. All balances, dashboard totals, and reports derive from these rules.

## Versioning

- **v1:** Simple interest only. **Zero interest allowed** (rate = 0) for lending to friends where you only want to track principal.
- **v2:** Compound interest.

## Global rules

- **Rate is always monthly.**
- **Whole-month accrual:** interest is calculated per whole month. A new month's interest posts on the loan's **monthly anniversary date**, regardless of the day within the month. (Show this note in the UI.)
- **Payment allocation:** each payment covers **accrued interest first**, remainder reduces **principal** (default; manually overridable per payment).
- **Interest freezes at end of Term** if the loan is unpaid — it does **not** keep accruing past the term.
- Store `interest_component` and `principal_component` on every payment.

## Loan types

### Simple interest
- Interest accrues on the **original principal only**.
- Monthly interest = `principal × monthly_rate`
- Interest after *n* whole months = `principal × monthly_rate × n`
- **Early payoff:** pro-rated to actual elapsed whole months.
- Freezes at Term end.
- **Zero interest (rate = 0):** allowed — interest is always 0, loan tracks principal only.

### Compound interest
- Interest capitalizes **monthly** on the anniversary; it accrues on **principal + previously accumulated interest**.
- Balance after *n* whole months (no payments) = `principal × (1 + monthly_rate)^n`
- Monthly interest for a period = `current_balance × monthly_rate`
- Freezes at Term end (stops compounding past term).

## Payment allocation (both types)

On each payment:
1. Compute interest due as of payment date (whole months).
2. Payment covers accrued interest first.
3. Remainder reduces principal.
4. Persist `interest_component` + `principal_component`.

## Repayment mode

Chosen at loan creation. Default = **one-time**.

- **One-time:** single expected settlement (principal + accrued interest). Partial payments still allowed; no schedule.
- **Installments:** app auto-generates a schedule (below).

## Installment schedule — Option B (fixed principal + interest on balance)

For an installment loan with `Term = N` months, frequency = monthly:

- **Principal per installment** = `principal / N` (equal each month).
- **Interest per installment** = interest on the **outstanding balance** that month
  - Simple: on remaining principal.
  - Compound: on remaining balance (incl. capitalized interest).
- **Installment total** = principal portion + interest portion → **decreases** over time.

### Example — Simple, ₹10,000, 2%/month, 5 months

| # | Opening principal | Principal paid | Interest (2%) | Installment total |
|---|---|---|---|---|
| 1 | 10,000 | 2,000 | 200 | 2,200 |
| 2 | 8,000 | 2,000 | 160 | 2,160 |
| 3 | 6,000 | 2,000 | 120 | 2,120 |
| 4 | 4,000 | 2,000 | 80 | 2,080 |
| 5 | 2,000 | 2,000 | 40 | 2,040 |
|   |        | **10,000** | **600** | **10,600** |

### Schedule deviation policy
- The generated schedule is a **plan** (per-installment due date, principal, interest, total, paid status).
- Actual payments are tracked **separately** and matched against the plan.
- The schedule is **not regenerated** on every payment; the app shows **planned vs actual variance**.

## Per-loan derived state

- Outstanding **principal**
- Accrued unpaid **interest** (whole-month, frozen at term)
- Interest **collected** to date
- Principal **recovered** to date
- **Status:** `active` / `overdue` / `closed` / `written_off`
  - `overdue` auto-derived (past due date/term with outstanding balance)
  - `closed`, `written_off` set manually

## Rounding

- Round monetary values to **2 decimals** at storage; display in INR (₹). Apply rounding consistently at each computation step.
