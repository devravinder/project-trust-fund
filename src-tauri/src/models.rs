//! Domain models shared across repositories and Tauri commands.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Borrower {
    pub id: String,
    pub name: String,
    pub phone: Option<String>,
    pub address: Option<String>,
    pub photo_path: Option<String>,
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Input for creating/updating a borrower (no server-managed fields).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BorrowerInput {
    pub name: String,
    pub phone: Option<String>,
    pub address: Option<String>,
    pub photo_path: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Loan {
    pub id: String,
    pub borrower_id: String,
    pub principal: f64,
    pub monthly_rate: f64,
    pub interest_type: String,   // 'simple' | 'compound'
    pub repayment_mode: String,  // 'one_time' | 'installments'
    pub term_months: Option<i64>,
    pub start_date: String,
    pub status: String,          // active | overdue | closed | written_off
    pub note: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoanInput {
    pub borrower_id: String,
    pub principal: f64,
    pub monthly_rate: f64,
    pub interest_type: String,
    pub repayment_mode: String,
    pub term_months: Option<i64>,
    pub start_date: String,
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Payment {
    pub id: String,
    pub loan_id: String,
    pub amount: f64,
    pub interest_component: f64,
    pub principal_component: f64,
    pub paid_date: String,
    pub note: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentInput {
    pub loan_id: String,
    pub amount: f64,
    /// Optional manual allocation override. If None, allocate interest-first.
    pub interest_component: Option<f64>,
    pub principal_component: Option<f64>,
    pub paid_date: String,
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleItem {
    pub id: String,
    pub loan_id: String,
    pub seq: i64,
    pub due_date: String,
    pub principal_due: f64,
    pub interest_due: f64,
    pub total_due: f64,
    pub status: String,
}
