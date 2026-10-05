mod credentials;
mod data;
mod export;
mod interest;
mod models;
mod repo_borrowers;
mod repo_loans;
mod repo_payments;
mod repo_reports;
mod repo_stats;
mod store;
mod turso;
mod util;

use credentials::TursoCredentials;
use models::{Borrower, BorrowerInput, Loan, LoanInput, Payment, PaymentInput, ScheduleItem};
use repo_loans::LoanSummary;
use repo_payments::PaymentView;
use repo_reports::{MonthlyPoint, PersonReport};
use repo_stats::{DashboardSummary, DueItem, OverdueLoan};
use store::StoreState;
use tauri::Manager;

fn json_path(app: &tauri::AppHandle) -> Result<std::path::PathBuf, String> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("app_data_dir: {e}"))?;
    std::fs::create_dir_all(&dir).map_err(|e| format!("create dir: {e}"))?;
    Ok(dir.join("trustfund.json"))
}

// ---- Connection ----

/// Open the local JSON store (offline, no account).
#[tauri::command]
async fn db_connect_local(
    app: tauri::AppHandle,
    state: tauri::State<'_, StoreState>,
) -> Result<(), String> {
    let path = json_path(&app)?;
    state.open_json(path).await.map_err(|e| e.to_string())
}

/// Connect to Turso. Persists credentials on success. Returns whether the local
/// JSON file still has data (so the UI can offer to migrate it).
#[tauri::command]
async fn db_connect_turso(
    app: tauri::AppHandle,
    state: tauri::State<'_, StoreState>,
    sync_url: String,
    auth_token: String,
) -> Result<bool, String> {
    state
        .open_turso(sync_url.clone(), auth_token.clone())
        .await
        .map_err(|e| e.to_string())?;
    credentials::save(&app, &TursoCredentials { sync_url, auth_token })?;

    // Does the local JSON file still hold data to migrate?
    let path = json_path(&app)?;
    let json = data::json_load(&path).map_err(|e| e.to_string())?;
    Ok(!json.is_empty())
}

/// On startup: connect to saved Turso creds if present, else local JSON.
/// Returns "turso" or "local".
#[tauri::command]
async fn db_connect_saved(
    app: tauri::AppHandle,
    state: tauri::State<'_, StoreState>,
) -> Result<String, String> {
    match credentials::load(&app)? {
        Some(creds) => {
            state
                .open_turso(creds.sync_url, creds.auth_token)
                .await
                .map_err(|e| e.to_string())?;
            Ok("turso".into())
        }
        None => {
            let path = json_path(&app)?;
            state.open_json(path).await.map_err(|e| e.to_string())?;
            Ok("local".into())
        }
    }
}

#[tauri::command]
fn db_has_credentials(app: tauri::AppHandle) -> Result<bool, String> {
    Ok(credentials::load(&app)?.is_some())
}

#[tauri::command]
fn db_get_credentials(app: tauri::AppHandle) -> Result<Option<TursoCredentials>, String> {
    credentials::load(&app)
}

/// Disconnect from Turso: clear credentials and switch back to local JSON.
#[tauri::command]
async fn db_clear_credentials(
    app: tauri::AppHandle,
    state: tauri::State<'_, StoreState>,
) -> Result<(), String> {
    credentials::clear(&app)?;
    let path = json_path(&app)?;
    state.open_json(path).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn db_is_connected(state: tauri::State<'_, StoreState>) -> Result<bool, String> {
    Ok(state.is_connected().await)
}

#[tauri::command]
async fn db_is_remote(state: tauri::State<'_, StoreState>) -> Result<bool, String> {
    Ok(state.is_remote().await)
}

// ---- JSON -> Turso migration ----

/// Push all local JSON data into the (currently connected) Turso store, then
/// delete the local JSON file.
#[tauri::command]
async fn migrate_json_to_turso(
    app: tauri::AppHandle,
    state: tauri::State<'_, StoreState>,
) -> Result<(), String> {
    if !state.is_remote().await {
        return Err("not connected to Turso".into());
    }
    let path = json_path(&app)?;
    let json = data::json_load(&path).map_err(|e| e.to_string())?;
    if !json.is_empty() {
        // Merge the JSON dataset into Turso via the write runner.
        data::write(&state, move |d| {
            d.borrowers.extend(json.borrowers.clone());
            d.loans.extend(json.loans.clone());
            d.payments.extend(json.payments.clone());
            d.schedule.extend(json.schedule.clone());
            d.deleted_borrowers.extend(json.deleted_borrowers.clone());
            d.deleted_loans.extend(json.deleted_loans.clone());
            d.deleted_payments.extend(json.deleted_payments.clone());
        })
        .await
        .map_err(|e| e.to_string())?;
    }
    // Remove the local file after a successful migration.
    let _ = std::fs::remove_file(&path);
    Ok(())
}

/// Discard the local JSON file (user chose not to migrate).
#[tauri::command]
fn delete_local_json(app: tauri::AppHandle) -> Result<(), String> {
    let path = json_path(&app)?;
    if path.exists() {
        std::fs::remove_file(&path).map_err(|e| format!("delete failed: {e}"))?;
    }
    Ok(())
}

// ---- Dev seed / clear ----

/// Erase all data in the active store (borrowers, loans, payments, schedule).
#[tauri::command]
async fn clear_all_data(state: tauri::State<'_, StoreState>) -> Result<(), String> {
    data::write(&state, |d| {
        d.borrowers.clear();
        d.loans.clear();
        d.payments.clear();
        d.schedule.clear();
        d.deleted_borrowers.clear();
        d.deleted_loans.clear();
        d.deleted_payments.clear();
    })
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
async fn dev_seed(state: tauri::State<'_, StoreState>) -> Result<(), String> {
    use crate::data::Dataset;

    // Free helpers (not closures) so multiple can borrow the dataset in turn.
    fn seed_borrower(d: &mut Dataset, name: &str, phone: &str, addr: Option<&str>) -> String {
        repo_borrowers::create(
            d,
            BorrowerInput {
                name: name.into(),
                phone: Some(phone.into()),
                address: addr.map(|a| a.into()),
                photo_path: None,
                notes: None,
            },
        )
        .id
    }

    #[allow(clippy::too_many_arguments)]
    fn seed_loan(
        d: &mut Dataset,
        borrower_id: String,
        principal: f64,
        rate: f64,
        mode: &str,
        start: &str,
        end: &str,
        note: Option<&str>,
    ) -> Option<String> {
        repo_loans::create(
            d,
            LoanInput {
                borrower_id,
                principal,
                monthly_rate: rate,
                interest_type: "simple".into(),
                repayment_mode: mode.into(),
                end_date: Some(end.into()),
                start_date: start.into(),
                note: note.map(|n| n.into()),
            },
        )
        .ok()
        .map(|l| l.id)
    }

    fn seed_pay(d: &mut Dataset, loan_id: &str, amount: f64, date: &str) {
        let _ = repo_payments::create(
            d,
            PaymentInput {
                loan_id: loan_id.to_string(),
                amount,
                interest_component: None,
                principal_component: None,
                paid_date: date.into(),
                note: None,
            },
        );
    }

    data::write(&state, |d| {
        let ramesh = seed_borrower(d, "Ramesh Kumar", "9876543210", Some("Bengaluru"));
        let asha = seed_borrower(d, "Asha Verma", "9123456780", None);
        let suresh = seed_borrower(d, "Suresh Rao", "9988776655", Some("Hyderabad"));
        let meena = seed_borrower(d, "Meena Nair", "9001122334", Some("Kochi"));
        let imran = seed_borrower(d, "Imran Shaikh", "9765432109", Some("Pune"));

        // Sample data spans ~1 year (2025-07 .. 2026-06) with payments spread
        // across many months so the monthly "by time" report is meaningful.

        // 1) Ramesh — 12-month installment loan, paid monthly across the year.
        if let Some(id) = seed_loan(
            d,
            ramesh,
            24_000.0,
            0.02,
            "installments",
            "2025-07-01",
            "2026-06-01",
            Some("12-month installment loan"),
        ) {
            for (date, amt) in [
                ("2025-08-01", 2480.0),
                ("2025-09-01", 2440.0),
                ("2025-10-01", 2400.0),
                ("2025-11-01", 2360.0),
                ("2025-12-01", 2320.0),
                ("2026-01-01", 2280.0),
                ("2026-02-01", 2240.0),
                ("2026-03-01", 2200.0),
            ] {
                seed_pay(d, &id, amt, date);
            }
        }

        // 2) Meena — larger installment loan started mid-2025, steady payments.
        if let Some(id) = seed_loan(
            d,
            meena,
            60_000.0,
            0.015,
            "installments",
            "2025-09-01",
            "2026-09-01",
            None,
        ) {
            for (date, amt) in [
                ("2025-10-01", 5900.0),
                ("2025-11-01", 5800.0),
                ("2025-12-01", 5700.0),
                ("2026-01-01", 5600.0),
                ("2026-02-01", 5500.0),
                ("2026-03-01", 5400.0),
            ] {
                seed_pay(d, &id, amt, date);
            }
        }

        // 3) Imran — one-time loan, periodic interest-only payments.
        if let Some(id) = seed_loan(
            d,
            imran,
            15_000.0,
            0.025,
            "one_time",
            "2025-08-10",
            "2026-08-10",
            Some("Interest paid periodically"),
        ) {
            for date in ["2025-09-10", "2025-11-10", "2026-01-10", "2026-03-10"] {
                seed_pay(d, &id, 375.0, date);
            }
        }

        // 4) Asha — zero-interest one-time (track-only), partially repaid.
        if let Some(id) = seed_loan(
            d,
            asha,
            5_000.0,
            0.0,
            "one_time",
            "2025-10-01",
            "2026-10-01",
            Some("Zero-interest, friend"),
        ) {
            seed_pay(d, &id, 2_000.0, "2026-01-01");
        }

        // 5) Suresh — overdue one-time loan (ended in the past, unpaid).
        let _ = seed_loan(
            d,
            suresh,
            20_000.0,
            0.015,
            "one_time",
            "2025-01-01",
            "2025-06-01",
            Some("Overdue — follow up"),
        );
    })
    .await
    .map_err(|e| e.to_string())
}

// ---- Borrowers ----

#[tauri::command]
async fn borrowers_list(
    state: tauri::State<'_, StoreState>,
    search: Option<String>,
) -> Result<Vec<Borrower>, String> {
    data::read(&state, |d| repo_borrowers::list(d, search.as_deref()))
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn borrower_get(
    state: tauri::State<'_, StoreState>,
    id: String,
) -> Result<Option<Borrower>, String> {
    data::read(&state, |d| repo_borrowers::get(d, &id))
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn borrower_create(
    state: tauri::State<'_, StoreState>,
    input: BorrowerInput,
) -> Result<Borrower, String> {
    data::write(&state, |d| repo_borrowers::create(d, input))
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn borrower_update(
    state: tauri::State<'_, StoreState>,
    id: String,
    input: BorrowerInput,
) -> Result<Borrower, String> {
    data::write(&state, |d| repo_borrowers::update(d, &id, input))
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "borrower not found".into())
}

#[tauri::command]
async fn borrower_delete(
    state: tauri::State<'_, StoreState>,
    id: String,
) -> Result<(), String> {
    data::write(&state, |d| repo_borrowers::soft_delete(d, &id))
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn borrower_total_outstanding(
    state: tauri::State<'_, StoreState>,
    id: String,
) -> Result<f64, String> {
    data::read(&state, |d| repo_borrowers::total_outstanding(d, &id))
        .await
        .map_err(|e| e.to_string())
}

// ---- Loans ----

#[tauri::command]
async fn loans_list(
    state: tauri::State<'_, StoreState>,
    status: Option<String>,
    search: Option<String>,
) -> Result<Vec<LoanSummary>, String> {
    data::read(&state, |d| {
        repo_loans::list(d, status.as_deref(), search.as_deref())
    })
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
async fn loan_get(
    state: tauri::State<'_, StoreState>,
    id: String,
) -> Result<Option<Loan>, String> {
    data::read(&state, |d| repo_loans::get(d, &id))
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn loan_summary(
    state: tauri::State<'_, StoreState>,
    id: String,
) -> Result<Option<LoanSummary>, String> {
    data::read(&state, |d| repo_loans::get_summary(d, &id))
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn loan_create(
    state: tauri::State<'_, StoreState>,
    input: LoanInput,
) -> Result<Loan, String> {
    data::write(&state, |d| repo_loans::create(d, input))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e)
}

#[tauri::command]
async fn loan_set_status(
    state: tauri::State<'_, StoreState>,
    id: String,
    status: String,
) -> Result<(), String> {
    data::write(&state, |d| repo_loans::set_status(d, &id, &status))
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn loan_delete(state: tauri::State<'_, StoreState>, id: String) -> Result<(), String> {
    data::write(&state, |d| repo_loans::soft_delete(d, &id))
        .await
        .map_err(|e| e.to_string())
}

/// Permanently delete a loan and all its related payments and schedule.
#[tauri::command]
async fn loan_delete_with_related(
    state: tauri::State<'_, StoreState>,
    id: String,
) -> Result<(), String> {
    data::write(&state, |d| repo_loans::delete_with_related(d, &id))
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn loan_schedule(
    state: tauri::State<'_, StoreState>,
    id: String,
) -> Result<Vec<ScheduleItem>, String> {
    data::read(&state, |d| repo_loans::schedule(d, &id))
        .await
        .map_err(|e| e.to_string())
}

// ---- Payments ----

#[tauri::command]
async fn payments_list(
    state: tauri::State<'_, StoreState>,
    search: Option<String>,
) -> Result<Vec<PaymentView>, String> {
    data::read(&state, |d| repo_payments::list(d, search.as_deref()))
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn payments_for_loan(
    state: tauri::State<'_, StoreState>,
    loan_id: String,
) -> Result<Vec<Payment>, String> {
    data::read(&state, |d| repo_payments::list_for_loan(d, &loan_id))
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn payment_create(
    state: tauri::State<'_, StoreState>,
    input: PaymentInput,
) -> Result<Payment, String> {
    data::write(&state, |d| repo_payments::create(d, input))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e)
}

#[tauri::command]
async fn payment_delete(state: tauri::State<'_, StoreState>, id: String) -> Result<(), String> {
    data::write(&state, |d| repo_payments::soft_delete(d, &id))
        .await
        .map_err(|e| e.to_string())
}

// ---- Dashboard ----

#[tauri::command]
async fn dashboard_summary(
    state: tauri::State<'_, StoreState>,
) -> Result<DashboardSummary, String> {
    data::read(&state, repo_stats::summary)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn dashboard_dues(
    state: tauri::State<'_, StoreState>,
    limit: Option<i64>,
) -> Result<Vec<DueItem>, String> {
    let lim = limit.unwrap_or(5) as usize;
    data::read(&state, move |d| repo_stats::dues_this_month(d, lim))
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn dashboard_overdue(
    state: tauri::State<'_, StoreState>,
    limit: Option<i64>,
) -> Result<Vec<OverdueLoan>, String> {
    let lim = limit.unwrap_or(10) as usize;
    data::read(&state, move |d| repo_stats::overdue_loans(d, lim))
        .await
        .map_err(|e| e.to_string())
}

// ---- Reports ----

#[tauri::command]
async fn report_monthly(
    state: tauri::State<'_, StoreState>,
) -> Result<Vec<MonthlyPoint>, String> {
    data::read(&state, repo_reports::monthly)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn report_by_person(
    state: tauri::State<'_, StoreState>,
) -> Result<Vec<PersonReport>, String> {
    data::read(&state, repo_reports::by_person)
        .await
        .map_err(|e| e.to_string())
}

// ---- Export (CSV) ----

#[tauri::command]
async fn export_csv(
    state: tauri::State<'_, StoreState>,
    table: String,
) -> Result<String, String> {
    data::read(&state, move |d| match table.as_str() {
        "borrowers" => Ok(export::borrowers_csv(d)),
        "loans" => Ok(export::loans_csv(d)),
        "payments" => Ok(export::payments_csv(d)),
        other => Err(format!("unknown table: {other}")),
    })
    .await
    .map_err(|e| e.to_string())?
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::default().build())
        .manage(StoreState::default())
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            db_connect_local,
            db_connect_turso,
            db_connect_saved,
            db_has_credentials,
            db_get_credentials,
            db_clear_credentials,
            db_is_connected,
            db_is_remote,
            migrate_json_to_turso,
            delete_local_json,
            dev_seed,
            clear_all_data,
            borrowers_list,
            borrower_get,
            borrower_create,
            borrower_update,
            borrower_delete,
            borrower_total_outstanding,
            loans_list,
            loan_get,
            loan_summary,
            loan_create,
            loan_set_status,
            loan_delete,
            loan_delete_with_related,
            loan_schedule,
            payments_list,
            payments_for_loan,
            payment_create,
            payment_delete,
            dashboard_summary,
            dashboard_dues,
            dashboard_overdue,
            report_monthly,
            report_by_person,
            export_csv,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
