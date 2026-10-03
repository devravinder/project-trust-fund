# TrustFund — Tasks

Track all work here. Complete tasks **one by one**; after each completed task, do a **git commit** with a precise message (**under 15 words**).

## Scope

- **v1:** Simple interest only. **Zero interest allowed** (lend to friends, just track).
- **v2:** Compound interest.
- **Platform order:** Windows desktop first → then Android → then Linux.
- Interest freezes at term end. Whole-month accrual. Interest-first payment allocation.

## Legend
- [ ] pending  ·  [x] done

---

## Phase 0 — Foundation & setup
- [x] Initialize git repo (if not already) and add `.gitignore`
- [x] Validate spike: libSQL embedded replica via Tauri command (desktop confirmed; Android deferred)
- [x] Scaffold Tauri v2 + React + TypeScript + Vite project
- [x] Add Tailwind CSS + shadcn/ui
- [x] Add base tooling: Prettier, TypeScript strict config (oxlint from scaffold)
- [x] Set up app shell: routing (React Router), layout, navigation

## Phase 1 — Data layer (Rust core)
- [x] Add `libsql` Rust crate to Tauri core
- [x] Implement DB init: local SQLite file + embedded replica connect (BYODB creds)
- [x] Create schema migrations (borrowers, loans, payments, installment_schedule, app_meta)
- [x] Implement secure credential storage (Tauri Store) for Turso URL + token
- [x] Expose DB operations to React via Tauri commands (IPC)
- [x] Seed/dev helper for local test data

## Phase 2 — Onboarding
- [x] First-launch flow: local fallback + "Connect" via Settings
- [x] In-app guide: steps to create free Turso account + DB
- [x] Credential entry + validate connection + persist securely
- [x] Initialize embedded replica after successful connect

## Phase 3 — Borrowers
- [x] Borrower model + Tauri commands (CRUD, soft-delete)
- [x] Borrowers list + search UI
- [x] Borrower create/edit form (name, phone, address, notes) — RHF + Zod
- [x] Borrower detail (with total outstanding for that borrower)

## Phase 4 — Loans (Simple interest, v1)
- [x] Loan model + Tauri commands (CRUD, soft-delete)
- [x] Simple-interest computation (monthly, whole-month, pro-rated early payoff, freeze at term)
- [x] Zero-interest support (rate = 0) in simple loans
- [x] Installment schedule generation (Option B: fixed principal + interest on balance)
- [x] Loan create form (borrower dropdown, amount, monthly rate, term, start date, note, repayment mode)
- [x] Loans list with filters: active / overdue / closed / all + search
- [x] Loan detail: balances, schedule (if installments), payments list
- [x] Status: commands for manual close / write-off (auto-overdue in dashboard calc)

## Phase 5 — Payments (Repayments)
- [x] Payment model + Tauri commands (CRUD, soft-delete)
- [x] Interest-first allocation logic (store interest/principal components; manual override)
- [x] Payment create form (loan dropdown, amount, date, note)
- [x] Payments list + search by borrower
- [x] Update installment schedule paid/partial/overdue status against actuals

## Phase 6 — Dashboard
- [x] Summary computations (total lent, outstanding, active loans, overdue count + amount, interest collected, interest due this month)
- [x] Dashboard cards UI
- [x] "Dues this month" short list (tap-through to loans)

## Phase 7 — Reports
- [x] Report computations: by time (monthly) and by person
- [x] Charts: interest collected/month (bar), recovered vs outstanding (donut), by-person table
- [x] Reports UI (monthly default; custom date range deferred)

## Phase 8 — Polish & cross-cutting
- [x] INR formatting (Indian digit grouping) + consistent rounding (2 decimals)
- [x] Empty/first-run states and guidance
- [x] Export to CSV/JSON (data safety) — CSV export for borrowers/loans/payments
- [x] Error handling + loading states (toasts, loading text)
- [x] Basic tests for interest/schedule/allocation logic

## Phase 9 — Windows desktop release
- [ ] Windows build config + icon/branding
- [ ] Build + smoke test Windows desktop installer
- [ ] Tag v1 (Windows)

## Phase 10 — UX features
- [x] Light/dark theme: provider (light/dark/system), toggle in nav, persisted
- [x] Share connection details as QR code (from Settings)
- [x] Connect by scanning QR (camera, default option) with manual entry fallback

## Later platforms (after Windows feature-complete)
- [ ] Android: SDK/NDK setup, embedded-replica spike on device, build APK
- [ ] Linux: build config + package
- [ ] v2: Compound interest type

---

## Commit log (running)
- Add tasks.md and scope docs (v1 simple interest, Windows first)
- Scaffold Tauri v2 + React + TS project; add gitignore and cargo net config
- Add Tailwind CSS v4 and shadcn/ui setup with path alias
- Add Prettier, enable strict TypeScript, add tooling scripts
- Add app shell: routing, responsive layout, nav, placeholder pages
- Validate libsql spike on Windows (libsql 0.9, MSVC required)
- Add data layer: DbState, schema migrations, connect commands
- Add secure credential storage and Turso connect commands
- Add borrower model, repository, and CRUD commands with tests
- Add Borrowers UI: list, search, form dialog, IPC bindings, UI kit
- Add loans backend: simple interest, schedule, balances, commands, tests
- Add Loans UI: filters, search, list, create form, select component
- Add payments backend: interest-first allocation, robust numeric reads
- Add Payments UI: list, search, record form, delete
- Add dashboard stats backend: summary metrics and dues this month
- Add Dashboard UI: summary cards and dues-this-month list
- Add reports backend: monthly and by-person aggregates with tests
- Add Reports UI: monthly bar, recovered/outstanding donut, by-person table
- Add Settings/onboarding: Turso connect flow, sync, guide
- Add Windows dev scripts and per-OS variants (MSVC init)
