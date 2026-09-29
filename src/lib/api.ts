import { invoke } from '@tauri-apps/api/core'
import type { Borrower, BorrowerInput } from '@/types'

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
