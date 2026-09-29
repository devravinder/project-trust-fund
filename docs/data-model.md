# TrustFund — Data Model

SQLite / libSQL schema. Designed for **multi-device sync** (UUID keys), auditability (timestamps), and safe history (soft-delete).

## Conventions

- **Primary keys:** `TEXT` UUIDs generated **on-device** (avoids autoincrement collisions across synced devices).
- **Timestamps:** `created_at`, `updated_at` as ISO-8601 `TEXT` (UTC).
- **Soft-delete:** `deleted_at TEXT NULL` — records are hidden, not physically removed, to preserve reports/history.
- **Money:** `REAL`, rounded to 2 decimals. Currency fixed to INR.
- **Dates:** ISO-8601 `TEXT` (`YYYY-MM-DD`).

## Tables

### borrowers
```sql
CREATE TABLE borrowers (
    id          TEXT PRIMARY KEY,        -- UUID
    name        TEXT NOT NULL,
    phone       TEXT,
    address     TEXT,
    photo_path  TEXT,                    -- local file reference (optional)
    notes       TEXT,
    created_at  TEXT NOT NULL,
    updated_at  TEXT NOT NULL,
    deleted_at  TEXT
);
CREATE INDEX idx_borrowers_name ON borrowers(name);
```

### loans
```sql
CREATE TABLE loans (
    id             TEXT PRIMARY KEY,     -- UUID
    borrower_id    TEXT NOT NULL REFERENCES borrowers(id),
    principal      REAL NOT NULL,
    monthly_rate   REAL NOT NULL,        -- e.g. 0.02 for 2%/month
    interest_type  TEXT NOT NULL CHECK (interest_type IN ('simple','compound')),
    repayment_mode TEXT NOT NULL CHECK (repayment_mode IN ('one_time','installments')),
    term_months    INTEGER,             -- required for installments; term for one-time (interest freezes after)
    start_date     TEXT NOT NULL,       -- anniversary/accrual anchor
    status         TEXT NOT NULL DEFAULT 'active'
                    CHECK (status IN ('active','overdue','closed','written_off')),
    note           TEXT,
    created_at     TEXT NOT NULL,
    updated_at     TEXT NOT NULL,
    deleted_at     TEXT
);
CREATE INDEX idx_loans_borrower ON loans(borrower_id);
CREATE INDEX idx_loans_status   ON loans(status);
```

### payments (repayments)
```sql
CREATE TABLE payments (
    id                 TEXT PRIMARY KEY, -- UUID
    loan_id            TEXT NOT NULL REFERENCES loans(id),
    amount             REAL NOT NULL,
    interest_component REAL NOT NULL DEFAULT 0,  -- allocated to interest
    principal_component REAL NOT NULL DEFAULT 0, -- allocated to principal
    paid_date          TEXT NOT NULL,
    note               TEXT,
    created_at         TEXT NOT NULL,
    updated_at         TEXT NOT NULL,
    deleted_at         TEXT
);
CREATE INDEX idx_payments_loan ON payments(loan_id);
CREATE INDEX idx_payments_date ON payments(paid_date);
```

### installment_schedule
Generated for `repayment_mode = 'installments'`. This is the **plan**; actuals live in `payments`.
```sql
CREATE TABLE installment_schedule (
    id               TEXT PRIMARY KEY,  -- UUID
    loan_id          TEXT NOT NULL REFERENCES loans(id),
    seq              INTEGER NOT NULL,  -- 1..N
    due_date         TEXT NOT NULL,
    principal_due    REAL NOT NULL,
    interest_due     REAL NOT NULL,
    total_due        REAL NOT NULL,
    status           TEXT NOT NULL DEFAULT 'pending'
                      CHECK (status IN ('pending','paid','partial','overdue')),
    created_at       TEXT NOT NULL,
    updated_at       TEXT NOT NULL
);
CREATE INDEX idx_schedule_loan ON installment_schedule(loan_id);
CREATE INDEX idx_schedule_due  ON installment_schedule(due_date);
```

### app_meta (optional)
Key/value for app state (e.g., schema version, last sync marker).
```sql
CREATE TABLE app_meta (
    key        TEXT PRIMARY KEY,
    value      TEXT,
    updated_at TEXT NOT NULL
);
```

## Computed vs stored balances

Balances (outstanding principal, accrued interest, collected, recovered) are **computed** from `loans` + `payments` using the rules in `interest-logic.md`, not stored as mutable columns — this avoids drift and sync conflicts. Dashboard summaries may be cached in memory or `app_meta` if performance requires, recomputed from source.

## Key queries (illustrative)

```sql
-- Principal recovered per loan
SELECT loan_id, SUM(principal_component) AS principal_recovered
FROM payments WHERE deleted_at IS NULL GROUP BY loan_id;

-- Interest collected per loan
SELECT loan_id, SUM(interest_component) AS interest_collected
FROM payments WHERE deleted_at IS NULL GROUP BY loan_id;

-- Total lent (active loans)
SELECT SUM(principal) FROM loans
WHERE deleted_at IS NULL AND status IN ('active','overdue');

-- Interest collected this month
SELECT SUM(interest_component) FROM payments
WHERE deleted_at IS NULL AND paid_date >= :month_start AND paid_date < :month_end;

-- Dues this month (from schedule)
SELECT * FROM installment_schedule
WHERE status IN ('pending','partial','overdue')
  AND due_date >= :month_start AND due_date < :month_end
ORDER BY due_date;
```

## Status derivation

- **overdue:** loan/installment past due date with outstanding balance → derived at read time (or refreshed on app open).
- **closed / written_off:** set manually by the user.
- Outstanding principal = `principal − SUM(principal_component)`; accrued interest per `interest-logic.md`.

## Sync notes

- UUID keys + soft-delete make records merge-safe across devices.
- `updated_at` supports last-writer-wins style resolution if needed.
- Photos stored as local files (path referenced); keep sizes small to avoid bloating the synced DB.
