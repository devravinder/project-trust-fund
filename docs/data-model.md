# TrustFund — Data Model

The same logical model is persisted two ways: as one **JSON document** (offline
backend) or as **per-row tables in Turso** (remote backend). Entities and fields
are identical; only the storage shape differs.

## Conventions

- **IDs:** `TEXT` UUIDs generated on-device (sync-safe, no autoincrement
  collisions across devices).
- **Timestamps:** `created_at`, `updated_at` as ISO-8601 strings (UTC).
- **Soft-delete:** tracked via `deleted_*` id lists (JSON) / a `deleted` flag
  (Turso) so history is preserved for reports/export. (Loan "delete with related
  data" is a hard delete — removes the loan + its payments + schedule.)
- **Money:** numbers rounded to 2 decimals; negative-zero normalized to 0.
  Currency fixed to INR (₹), Indian digit grouping in the UI.
- **Dates:** ISO-8601 (`YYYY-MM-DD`).

## Entities

### Borrower
`id, name, phone?, address?, photo_path?, notes?, created_at, updated_at`

### Loan
`id, borrower_id, principal, monthly_rate (fraction, e.g. 0.02 = 2%/month),
interest_type ('simple' | 'compound'; v1 = simple), repayment_mode ('one_time' |
'installments'), end_date? (YYYY-MM-DD; drives term, interest freeze, overdue),
start_date, status ('active' | 'overdue' | 'closed' | 'written_off'), note?,
created_at, updated_at`

### Payment
`id, loan_id, amount, interest_component, principal_component, paid_date, note?,
created_at, updated_at`

### ScheduleItem (installment plan)
Generated when `repayment_mode = 'installments'`. The plan; actuals live in
payments.
`id, loan_id, seq, due_date, principal_due, interest_due, total_due, status
('pending' | 'paid' | 'partial' | 'overdue')`

## JSON backend shape

The whole dataset is one file (`<app-data>/trustfund.json`):

```json
{
  "borrowers": [ { "id": "…", "name": "…", … } ],
  "loans": [ { "id": "…", "borrower_id": "…", "principal": 10000, … } ],
  "payments": [ { "id": "…", "loan_id": "…", "amount": 2200, … } ],
  "schedule": [ { "id": "…", "loan_id": "…", "seq": 1, … } ],
  "deleted_borrowers": ["…"],
  "deleted_loans": ["…"],
  "deleted_payments": ["…"]
}
```

Memory policy: the file is loaded per operation, mutated, written back, and
dropped — never held resident between calls.

## Turso backend tables

Created on connect (idempotent) via the HTTP pipeline API. A `deleted INTEGER`
flag implements soft-delete; queries filter `WHERE deleted = 0`.

```
borrowers(id, name, phone, address, photo_path, notes, created_at, updated_at, deleted)
loans(id, borrower_id, principal, monthly_rate, interest_type, repayment_mode,
      end_date, start_date, status, note, created_at, updated_at, deleted)
payments(id, loan_id, amount, interest_component, principal_component,
         paid_date, note, created_at, updated_at, deleted)
schedule(id, loan_id, seq, due_date, principal_due, interest_due, total_due, status)
```

## Computed, not stored

Balances and summaries are computed in Rust from the loaded data (see
`interest-logic.md`), never persisted as mutable columns — avoids drift:

- **Outstanding principal** = `principal − Σ principal_component`
- **Interest collected** = `Σ interest_component`
- **Interest to date** = simple interest accrued from `start_date` to today
  (whole months, frozen at term)
- **Interest due** = `interest_to_date − interest_collected`
- **Total lent** = `Σ principal` of active/overdue loans
- **Overdue** = past term end with outstanding balance (derived at read time)
- **Dues this month** = schedule items due in the current month, not fully paid

## Migration (JSON → Turso)

On connecting Turso, if the local JSON has data the UI prompts:
- **Yes** → all JSON records are inserted into Turso, then the JSON file is
  deleted.
- **No** → the JSON file is deleted (discarded).

## Multi-device notes

- UUID ids keep records merge-safe.
- Turso is the shared source of truth; reads are live (no stale local cache in
  connected mode).
