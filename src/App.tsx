import { useEffect, useState } from 'react'
import { createHashRouter, RouterProvider } from 'react-router-dom'
import { Toaster } from 'sonner'
import { AppLayout } from '@/components/layout/AppLayout'
import { DashboardPage } from '@/pages/DashboardPage'
import { BorrowersPage } from '@/pages/BorrowersPage'
import { LoansPage } from '@/pages/LoansPage'
import { PaymentsPage } from '@/pages/PaymentsPage'
import { ReportsPage } from '@/pages/ReportsPage'
import { SettingsPage } from '@/pages/SettingsPage'
import { db } from '@/lib/api'

const router = createHashRouter([
  {
    path: '/',
    element: <AppLayout />,
    children: [
      { index: true, element: <DashboardPage /> },
      { path: 'borrowers', element: <BorrowersPage /> },
      { path: 'loans', element: <LoansPage /> },
      { path: 'payments', element: <PaymentsPage /> },
      { path: 'reports', element: <ReportsPage /> },
      { path: 'settings', element: <SettingsPage /> },
    ],
  },
])

function App() {
  const [ready, setReady] = useState(false)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    async function bootstrap() {
      try {
        // Try saved Turso credentials; fall back to a local-only DB.
        const connected = await db.connectSaved()
        if (!connected) {
          await db.connectLocal()
        }
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
      <Toaster richColors position="top-center" />
    </>
  )
}

export default App
