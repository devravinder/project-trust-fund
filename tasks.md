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
- [ ] Implement secure credential storage (Tauri Store / OS keystore) for Turso URL + token
- [ ] Expose DB operations to React via Tauri commands (IPC)
- [ ] Seed/dev helper for local test data

## Phase 2 — Onboarding
- [ ] First-launch flow: "Connect existing" vs "Create new" (guide)
- [ ] In-app guide: steps to create free Turso account + DB
- [ ] Credential entry + validate connection + persist securely
- [ ] Initialize embedded replica after successful connect

## Phase 3 — Borrowers
- [ ] Borrower model + Tauri commands (CRUD, soft-delete)
- [ ] Borrowers list + search UI
- [ ] Borrower create/edit form (name, phone, address, photo optional) — RHF + Zod
- [ ] Borrower detail (with total outstanding for that borrower)

## Phase 4 — Loans (Simple interest, v1)
- [ ] Loan model + Tauri commands (CRUD, soft-delete)
- [ ] Simple-interest computation (monthly, whole-month, pro-rated early payoff, freeze at term)
- [ ] Zero-interest support (rate = 0) in simple loans
- [ ] Installment schedule generation (Option B: fixed principal + interest on balance)
- [ ] Loan create/edit form (borrower dropdown+search, amount, monthly rate, term, start date, note, repayment mode)
- [ ] Loans list with filters: active / overdue / closed / all + search
- [ ] Loan detail: balances, schedule (if installments), payments list
- [ ] Auto status derivation (active/overdue); manual close / write-off

## Phase 5 — Payments (Repayments)
- [ ] Payment model + Tauri commands (CRUD, soft-delete)
- [ ] Interest-first allocation logic (store interest/principal components; manual override)
- [ ] Payment create/edit form (loan dropdown, amount, date, note)
- [ ] Payments list + search by borrower + date filter
- [ ] Update installment schedule paid/partial/overdue status against actuals

## Phase 6 — Dashboard
- [ ] Summary computations (total lent, outstanding principal, active loans, overdue count + amount, interest collected, interest due this month)
- [ ] Dashboard cards UI
- [ ] "Dues this month" short list (tap-through to loan)

## Phase 7 — Reports
- [ ] Report computations: by time (default monthly) and by person
- [ ] Charts: interest collected/month (bar), outstanding over time (line), recovered vs outstanding (donut), top borrowers
- [ ] Reports UI (monthly default; custom date range deferred)

## Phase 8 — Polish & cross-cutting
- [ ] INR formatting (Indian digit grouping) + consistent rounding (2 decimals)
- [ ] Empty/first-run states and guidance
- [ ] Export to CSV/JSON (data safety)
- [ ] Error handling + loading states
- [ ] Basic tests for interest/schedule/allocation logic

## Phase 9 — Windows desktop release
- [ ] Windows build config + icon/branding
- [ ] Build + smoke test Windows desktop installer
- [ ] Tag v1 (Windows)

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
