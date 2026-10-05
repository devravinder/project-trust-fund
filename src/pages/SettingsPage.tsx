import { useEffect, useState } from 'react'
import { Sun, Moon, Monitor } from 'lucide-react'
import { toast } from 'sonner'
import { PageHeader } from '@/components/layout/PageHeader'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { db } from '@/lib/api'
import { exporter } from '@/lib/api'
import { downloadText } from '@/lib/download'
import { useTheme } from '@/components/theme/ThemeProvider'
import { ConnectDialog } from '@/features/connection/ConnectDialog'
import { ShareConnectionDialog } from '@/features/connection/ShareConnectionDialog'
import { MigrationPromptDialog } from '@/features/connection/MigrationPromptDialog'
import { ConfirmDialog } from '@/components/ui/confirm-dialog'

export function SettingsPage() {
  const { theme, setTheme } = useTheme()
  const [hasCreds, setHasCreds] = useState(false)
  const [connectOpen, setConnectOpen] = useState(false)
  const [shareOpen, setShareOpen] = useState(false)
  const [migrateOpen, setMigrateOpen] = useState(false)
  const [disconnectOpen, setDisconnectOpen] = useState(false)
  const [clearAllOpen, setClearAllOpen] = useState(false)
  const [seedOpen, setSeedOpen] = useState(false)

  useEffect(() => {
    db.hasCredentials()
      .then(setHasCreds)
      .catch((e) => toast.error(String(e)))
  }, [])

  const confirmSeed = async () => {
    try {
      await db.seed()
      toast.success('Sample data loaded')
    } catch (e) {
      toast.error(String(e))
    }
  }

  const confirmClearAll = async () => {
    try {
      await db.clearAll()
      toast.success('All data cleared')
    } catch (e) {
      toast.error(String(e))
    }
  }

  const confirmDisconnect = async () => {
    try {
      await db.clearCredentials()
      setHasCreds(false)
      toast.success('Disconnected. Now using local storage.')
    } catch (e) {
      toast.error(String(e))
    }
  }

  const handleExport = async (table: 'borrowers' | 'loans' | 'payments') => {
    try {
      const csv = await exporter.csv(table)
      const date = new Date().toISOString().slice(0, 10)
      downloadText(`trustfund-${table}-${date}.csv`, csv)
      toast.success(`Exported ${table}`)
    } catch (e) {
      toast.error(String(e))
    }
  }

  return (
    <div className="max-w-2xl">
      <PageHeader
        title="Settings"
        description="Appearance and database connection"
      />

      <Card className="mb-4">
        <CardHeader>
          <CardTitle>Appearance</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="flex flex-wrap gap-2">
            <Button
              variant={theme === 'light' ? 'default' : 'outline'}
              size="sm"
              onClick={() => setTheme('light')}
            >
              <Sun /> Light
            </Button>
            <Button
              variant={theme === 'dark' ? 'default' : 'outline'}
              size="sm"
              onClick={() => setTheme('dark')}
            >
              <Moon /> Dark
            </Button>
            <Button
              variant={theme === 'system' ? 'default' : 'outline'}
              size="sm"
              onClick={() => setTheme('system')}
            >
              <Monitor /> System
            </Button>
          </div>
          <p className="mt-2 text-xs text-muted-foreground">
            “System” follows your device’s light/dark setting.
          </p>
        </CardContent>
      </Card>

      <Card className="mb-4">
        <CardHeader>
          <CardTitle>
            {hasCreds ? 'Connected to Turso' : 'Connect your database'}
          </CardTitle>
        </CardHeader>
        <CardContent className="grid gap-4">
          {hasCreds ? (
            <>
              <p className="text-sm text-muted-foreground">
                This device reads and writes directly to your Turso cloud
                database. Use the same connection on another device to share the
                same data. Share the connection below to set one up instantly.
              </p>
              <div className="flex flex-wrap gap-2">
                <Button variant="outline" onClick={() => setShareOpen(true)}>
                  Share connection (QR)
                </Button>
                <Button variant="outline" onClick={() => setDisconnectOpen(true)}>
                  Disconnect
                </Button>
              </div>
            </>
          ) : (
            <>
              <p className="text-sm text-muted-foreground">
                Connect to your Turso database by scanning a QR from another
                device, or enter the details manually.
              </p>
              <div>
                <Button onClick={() => setConnectOpen(true)}>
                  Connect a database
                </Button>
              </div>
              <p className="text-xs text-muted-foreground">
                Until you connect, TrustFund stores data locally on this device
                only.
              </p>
            </>
          )}
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>How to create a free Turso database</CardTitle>
        </CardHeader>
        <CardContent>
          <ol className="ml-4 list-decimal space-y-1 text-sm text-muted-foreground">
            <li>
              Sign up for a free account at{' '}
              <span className="font-medium text-foreground">turso.tech</span>.
            </li>
            <li>Create a new database from the dashboard.</li>
            <li>
              Copy the{' '}
              <span className="font-medium text-foreground">Database URL</span>{' '}
              (starts with <code>libsql://</code>).
            </li>
            <li>Create an auth token for the database and copy it.</li>
            <li>
              Select{' '}
              <span className="font-medium text-foreground">
                Connect a database
              </span>{' '}
              above, then scan a QR or enter the details.
            </li>
          </ol>
          <p className="mt-3 text-xs text-muted-foreground">
            Your token is stored on this device only and grants access to your
            own database.
          </p>
        </CardContent>
      </Card>

      <ConnectDialog
        open={connectOpen}
        onOpenChange={setConnectOpen}
        onConnected={(hasLocalData) => {
          setHasCreds(true)
          if (hasLocalData) setMigrateOpen(true)
        }}
      />
      <ShareConnectionDialog open={shareOpen} onOpenChange={setShareOpen} />
      <MigrationPromptDialog
        open={migrateOpen}
        onOpenChange={setMigrateOpen}
        onDone={() => {}}
      />

      <ConfirmDialog
        open={disconnectOpen}
        onOpenChange={setDisconnectOpen}
        title="Disconnect from Turso?"
        description="Your cloud data stays untouched; this device switches back to local offline storage."
        confirmText="Disconnect"
        onConfirm={confirmDisconnect}
      />

      <Card className="mt-4">
        <CardHeader>
          <CardTitle>Export data</CardTitle>
        </CardHeader>
        <CardContent className="flex flex-wrap gap-2">
          <Button variant="outline" onClick={() => handleExport('borrowers')}>
            Borrowers CSV
          </Button>
          <Button variant="outline" onClick={() => handleExport('loans')}>
            Loans CSV
          </Button>
          <Button variant="outline" onClick={() => handleExport('payments')}>
            Payments CSV
          </Button>
        </CardContent>
      </Card>

      <Card className="mt-4">
        <CardHeader>
          <CardTitle>Developer</CardTitle>
        </CardHeader>
        <CardContent>
          <Button variant="outline" onClick={() => setSeedOpen(true)}>
            Load sample data
          </Button>
          <p className="mt-2 text-xs text-muted-foreground">
            Adds sample borrowers, loans, and a payment for testing.
          </p>

          <div className="mt-4 border-t pt-4">
            <Button
              variant="destructive"
              onClick={() => setClearAllOpen(true)}
            >
              Clear all data
            </Button>
            <p className="mt-2 text-xs text-muted-foreground">
              Permanently deletes all borrowers, loans, and payments from the
              current store.
            </p>
          </div>
        </CardContent>
      </Card>

      <ConfirmDialog
        open={clearAllOpen}
        onOpenChange={setClearAllOpen}
        title="Clear all data?"
        description="This permanently deletes ALL borrowers, loans, and payments from the current store. This cannot be undone."
        confirmText="Clear everything"
        destructive
        onConfirm={confirmClearAll}
      />

      <ConfirmDialog
        open={seedOpen}
        onOpenChange={setSeedOpen}
        title="Load sample data?"
        description="This adds sample borrowers, loans, and payments to the current store (in addition to any existing data)."
        confirmText="Load sample data"
        onConfirm={confirmSeed}
      />
    </div>
  )
}
