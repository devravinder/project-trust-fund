// Domain types mirroring the Rust models (src-tauri/src/models.rs).

export interface Borrower {
  id: string
  name: string
  phone: string | null
  address: string | null
  photo_path: string | null
  notes: string | null
  created_at: string
  updated_at: string
}

export interface BorrowerInput {
  name: string
  phone?: string | null
  address?: string | null
  photo_path?: string | null
  notes?: string | null
}

export type InterestType = 'simple' | 'compound'
export type RepaymentMode = 'one_time' | 'installments'
export type LoanStatus = 'active' | 'overdue' | 'closed' | 'written_off'

export interface Loan {
  id: string
  borrower_id: string
  principal: number
  monthly_rate: number
  interest_type: InterestType
  repayment_mode: RepaymentMode
  end_date: string | null
  start_date: string
  status: LoanStatus
  note: string | null
  created_at: string
  updated_at: string
}

export interface LoanInput {
  borrower_id: string
  principal: number
  monthly_rate: number
  interest_type: InterestType
  repayment_mode: RepaymentMode
  end_date?: string | null
  start_date: string
  note?: string | null
}

export interface Payment {
  id: string
  loan_id: string
  amount: number
  interest_component: number
  principal_component: number
  paid_date: string
  note: string | null
  created_at: string
  updated_at: string
}

export interface PaymentInput {
  loan_id: string
  amount: number
  interest_component?: number | null
  principal_component?: number | null
  paid_date: string
  note?: string | null
}

export interface ScheduleItem {
  id: string
  loan_id: string
  seq: number
  due_date: string
  principal_due: number
  interest_due: number
  total_due: number
  status: 'pending' | 'paid' | 'partial' | 'overdue'
}
