import { useEffect, useState } from 'react'
import { toast } from 'sonner'
import { PageHeader } from '@/components/layout/PageHeader'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { db } from '@/lib/api'
import { exporter } from '@/lib/api'
import { downloadText } from '@/lib/download'
import { ConnectDialog } from '@/features/connection/ConnectDialog'
import { ShareConnectionDialog } from '@/features/connection/ShareConnectionDialog'

export function SettingsPage() {
  const [hasCreds, setHasCreds] = useState(false)
  const [syncing, setSyncing] = useState(false)
  const [connectOpen, setConnectOpen] = useState(false)
  const [shareOpen, setShareOpen] = useState(false)

  useEffect(() => {
    db.hasCredentials()
      .then(setHasCreds)
      .catch((e) => toast.error(String(e)))
  }, [])

  const handleSync = async () => {
    setSyncing(true)
    try {
      await db.sync()
      toast.success('Synced')
    } catch (e) {
      toast.error(String(e))
    } finally {
      setSyncing(false)
    }
  }

  const handleDisconnect = async () => {
    if (
      !confirm(
        'Disconnect this device? Your cloud data stays; the app falls back to local on next launch.',
      )
    )
      return
    try {
      await db.clearCredentials()
      setHasCreds(false)
      toast.success('Credentials cleared. Restart the app to apply.')
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
        description="Database connection (bring your own Turso database)"
      />

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
                This device is configured to sync with your Turso cloud
                database. Share the connection to set up another device
                instantly.
              </p>
              <div className="flex flex-wrap gap-2">
                <Button onClick={handleSync} disabled={syncing}>
                  {syncing ? 'Syncing…' : 'Sync now'}
                </Button>
                <Button variant="outline" onClick={() => setShareOpen(true)}>
                  Share connection (QR)
                </Button>
                <Button variant="outline" onClick={handleDisconnect}>
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
        onConnected={() => setHasCreds(true)}
      />
      <ShareConnectionDialog open={shareOpen} onOpenChange={setShareOpen} />

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
          <Button
            variant="outline"
            onClick={async () => {
              try {
                await db.seed()
                toast.success('Sample data loaded')
              } catch (e) {
                toast.error(String(e))
              }
            }}
          >
            Load sample data
          </Button>
          <p className="mt-2 text-xs text-muted-foreground">
            Adds sample borrowers, loans, and a payment for testing.
          </p>
        </CardContent>
      </Card>
    </div>
  )
}
