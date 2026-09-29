# TrustFund

A personal **lend & recover** tracker. Track loans you give out, record repayments, and see what's outstanding, recovered, and overdue — all from your phone, offline-capable.

## Key ideas

- **Mobile-first** Android app (native shell, no PWA slow-network issues).
- **Offline-first**: reads/writes hit a local SQLite file instantly; syncs in the background.
- **BYODB (Bring Your Own Database)**: each user creates their **own** free Turso account and database, then pastes their DB URL + auth token into the app. No shared account, no dependency on anyone else's quota.
- **No backend service**: the app connects directly to the user's Turso cloud DB.
- **Multi-device sync**: same credentials on another device → data syncs automatically.
- **In-app notifications only**: due/overdue loans are shown when you open the app.

## Tech stack

| Layer | Choice |
|---|---|
| Language | TypeScript (UI) + Rust (core) |
| UI framework | React + TypeScript |
| App shell | Tauri v2 (Android) |
| Database | Turso (libSQL), BYODB |
| DB access | `libsql` Rust crate on Tauri core, exposed to React via Tauri commands (IPC) |
| Offline / sync | libSQL embedded replica (local SQLite ⇄ Turso cloud) |
| Build tool | Vite |
| Styling | Tailwind CSS + shadcn/ui |
| Forms | React Hook Form + Zod |
| Charts | Recharts |
| Secure storage | Tauri Store plugin / OS keystore (for DB URL + token) |

> **Why Rust core for the DB?** The libSQL embedded replica needs native SQLite bindings and filesystem access — unavailable in a browser/webview. It runs on Tauri's native Rust side; React calls it via Tauri IPC.

## First-launch flow

1. App shows a guide to create a free Turso account + database.
2. User pastes **Database URL** + **Auth token**.
3. App validates, stores credentials securely on-device, initializes the embedded replica.
4. Same credentials on another device → automatic sync.

## Web support

Web is **not supported** in the offline design — the embedded replica cannot run in a browser. A web build would require the remote-only libSQL client (online-only), which drops offline support. Treated as optional / future.

## Cost

Fully free: Tauri, React/Vite (open source) + Turso free tier (on each user's own account).

## Status

Early stage. One technical risk to validate first: **libSQL embedded replica (Rust crate) driven via Tauri commands on Android.** See [`docs/architecture.md`](docs/architecture.md).
