import { lazy, Suspense, useEffect, useState, type ReactNode } from 'react'
import { createHashRouter, RouterProvider } from 'react-router-dom'
import { Toaster } from 'sonner'
import { AppLayout } from '@/components/layout/AppLayout'
import { db } from '@/lib/api'

const DashboardPage = lazy(() =>
  import('@/pages/DashboardPage').then((m) => ({ default: m.DashboardPage })),
)
const BorrowersPage = lazy(() =>
  import('@/pages/BorrowersPage').then((m) => ({ default: m.BorrowersPage })),
)
const BorrowerDetailPage = lazy(() =>
  import('@/pages/BorrowerDetailPage').then((m) => ({
    default: m.BorrowerDetailPage,
  })),
)
const LoansPage = lazy(() =>
  import('@/pages/LoansPage').then((m) => ({ default: m.LoansPage })),
)
const LoanDetailPage = lazy(() =>
  import('@/pages/LoanDetailPage').then((m) => ({
    default: m.LoanDetailPage,
  })),
)
const PaymentsPage = lazy(() =>
  import('@/pages/PaymentsPage').then((m) => ({ default: m.PaymentsPage })),
)
const PaymentDetailPage = lazy(() =>
  import('@/pages/PaymentDetailPage').then((m) => ({
    default: m.PaymentDetailPage,
  })),
)
const ReportsPage = lazy(() =>
  import('@/pages/ReportsPage').then((m) => ({ default: m.ReportsPage })),
)
const SettingsPage = lazy(() =>
  import('@/pages/SettingsPage').then((m) => ({ default: m.SettingsPage })),
)

const pageFallback = <p className="text-muted-foreground">Loading…</p>

function withSuspense(node: ReactNode) {
  return <Suspense fallback={pageFallback}>{node}</Suspense>
}

const router = createHashRouter([
  {
    path: '/',
    element: <AppLayout />,
    children: [
      { index: true, element: withSuspense(<DashboardPage />) },
      { path: 'borrowers', element: withSuspense(<BorrowersPage />) },
      { path: 'borrowers/:id', element: withSuspense(<BorrowerDetailPage />) },
      { path: 'loans', element: withSuspense(<LoansPage />) },
      { path: 'loans/:id', element: withSuspense(<LoanDetailPage />) },
      { path: 'payments', element: withSuspense(<PaymentsPage />) },
      { path: 'payments/:id', element: withSuspense(<PaymentDetailPage />) },
      { path: 'reports', element: withSuspense(<ReportsPage />) },
      { path: 'settings', element: withSuspense(<SettingsPage />) },
    ],
  },
])

function App() {
  const [ready, setReady] = useState(false)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    async function bootstrap() {
      try {
        // Connects to saved Turso creds if present, else the local JSON store.
        await db.connectSaved()
        setReady(true)
      } catch (e) {
        setError(String(e))
      }
    }
    void bootstrap()
  }, [])

  if (error) {
    return (
      <div className="flex min-h-screen items-center justify-center p-6 text-center">
        <div>
          <p className="font-medium text-destructive">
            Failed to open database
          </p>
          <p className="mt-1 text-sm text-muted-foreground">{error}</p>
        </div>
      </div>
    )
  }

  if (!ready) {
    return (
      <div className="flex min-h-screen items-center justify-center">
        <p className="text-muted-foreground">Starting TrustFund…</p>
      </div>
    )
  }

  return (
    <>
      <RouterProvider router={router} />
      <Toaster richColors closeButton position="top-center" duration={5000} />
    </>
  )
}

export default App
