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
use repo_stats::{DashboardSummary, DueItem};
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

// ---- Dev seed ----

#[tauri::command]
async fn dev_seed(state: tauri::State<'_, StoreState>) -> Result<(), String> {
    data::write(&state, |d| {
        let ramesh = repo_borrowers::create(
            d,
            BorrowerInput {
                name: "Ramesh Kumar".into(),
                phone: Some("9876543210".into()),
                address: Some("Bengaluru".into()),
                photo_path: None,
                notes: None,
            },
        );
        let asha = repo_borrowers::create(
            d,
            BorrowerInput {
                name: "Asha Verma".into(),
                phone: Some("9123456780".into()),
                address: None,
                photo_path: None,
                notes: None,
            },
        );
        if let Ok(loan1) = repo_loans::create(
            d,
            LoanInput {
                borrower_id: ramesh.id,
                principal: 10_000.0,
                monthly_rate: 0.02,
                interest_type: "simple".into(),
                repayment_mode: "installments".into(),
                term_months: Some(5),
                start_date: "2026-01-15".into(),
                note: Some("Sample installment loan".into()),
            },
        ) {
            let _ = repo_payments::create(
                d,
                PaymentInput {
                    loan_id: loan1.id,
                    amount: 2_200.0,
                    interest_component: None,
                    principal_component: None,
                    paid_date: "2026-02-15".into(),
                    note: None,
                },
            );
        }
        let _ = repo_loans::create(
            d,
            LoanInput {
                borrower_id: asha.id,
                principal: 5_000.0,
                monthly_rate: 0.0,
                interest_type: "simple".into(),
                repayment_mode: "one_time".into(),
                term_months: Some(6),
                start_date: "2026-02-01".into(),
                note: Some("Zero-interest, friend".into()),
            },
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
            report_monthly,
            report_by_person,
            export_csv,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
