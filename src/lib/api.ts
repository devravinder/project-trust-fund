import { invoke } from '@tauri-apps/api/core'
import type {
  Borrower,
  BorrowerInput,
  Loan,
  LoanInput,
  LoanStatus,
  Payment,
  PaymentInput,
  ScheduleItem,
} from '@/types'

export interface LoanSummary extends Loan {
  borrower_name: string
  principal_recovered: number
  interest_collected: number
  outstanding_principal: number
}

export interface PaymentView extends Payment {
  borrower_name: string
}

export interface DashboardSummary {
  total_lent: number
  outstanding_principal: number
  active_loans: number
  overdue_loans: number
  overdue_amount: number
  interest_collected: number
  interest_due_this_month: number
  principal_recovered: number
}

export interface DueItem {
  loan_id: string
  borrower_name: string
  due_date: string
  total_due: number
  status: string
}

export interface MonthlyPoint {
  month: string
  interest_collected: number
  principal_recovered: number
  total_collected: number
}

export interface PersonReport {
  borrower_id: string
  borrower_name: string
  total_lent: number
  principal_recovered: number
  interest_collected: number
  outstanding_principal: number
}

// ---- Database / connection ----

export const db = {
  connectLocal: () => invoke<void>('db_connect_local'),
  connectTurso: (syncUrl: string, authToken: string) =>
    invoke<void>('db_connect_turso', { syncUrl, authToken }),
  connectSaved: () => invoke<boolean>('db_connect_saved'),
  hasCredentials: () => invoke<boolean>('db_has_credentials'),
  clearCredentials: () => invoke<void>('db_clear_credentials'),
  isConnected: () => invoke<boolean>('db_is_connected'),
  sync: () => invoke<void>('db_sync'),
}

// ---- Borrowers ----

export const borrowers = {
  list: (search?: string) => invoke<Borrower[]>('borrowers_list', { search }),
  get: (id: string) => invoke<Borrower | null>('borrower_get', { id }),
  create: (input: BorrowerInput) =>
    invoke<Borrower>('borrower_create', { input }),
  update: (id: string, input: BorrowerInput) =>
    invoke<Borrower>('borrower_update', { id, input }),
  remove: (id: string) => invoke<void>('borrower_delete', { id }),
  totalOutstanding: (id: string) =>
    invoke<number>('borrower_total_outstanding', { id }),
}

// ---- Loans ----

export const loans = {
  list: (status?: LoanStatus | 'all', search?: string) =>
    invoke<LoanSummary[]>('loans_list', { status, search }),
  get: (id: string) => invoke<Loan | null>('loan_get', { id }),
  create: (input: LoanInput) => invoke<Loan>('loan_create', { input }),
  setStatus: (id: string, status: LoanStatus) =>
    invoke<void>('loan_set_status', { id, status }),
  remove: (id: string) => invoke<void>('loan_delete', { id }),
  schedule: (id: string) => invoke<ScheduleItem[]>('loan_schedule', { id }),
}

// ---- Payments ----

export const payments = {
  list: (search?: string) => invoke<PaymentView[]>('payments_list', { search }),
  forLoan: (loanId: string) =>
    invoke<Payment[]>('payments_for_loan', { loanId }),
  create: (input: PaymentInput) => invoke<Payment>('payment_create', { input }),
  remove: (id: string) => invoke<void>('payment_delete', { id }),
}

// ---- Dashboard ----

export const dashboard = {
  summary: () => invoke<DashboardSummary>('dashboard_summary'),
  dues: (limit?: number) => invoke<DueItem[]>('dashboard_dues', { limit }),
}

// ---- Reports ----

export const reports = {
  monthly: () => invoke<MonthlyPoint[]>('report_monthly'),
  byPerson: () => invoke<PersonReport[]>('report_by_person'),
}
