import { useEffect, useState } from 'react'
import { toast } from 'sonner'
import { PageHeader } from '@/components/layout/PageHeader'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { db } from '@/lib/api'

export function SettingsPage() {
  const [hasCreds, setHasCreds] = useState(false)
  const [syncUrl, setSyncUrl] = useState('')
  const [authToken, setAuthToken] = useState('')
  const [connecting, setConnecting] = useState(false)
  const [syncing, setSyncing] = useState(false)

  useEffect(() => {
    db.hasCredentials()
      .then(setHasCreds)
      .catch((e) => toast.error(String(e)))
  }, [])

  const handleConnect = async () => {
    if (!syncUrl.trim() || !authToken.trim()) {
      toast.error('Enter both the database URL and auth token')
      return
    }
    setConnecting(true)
    try {
      await db.connectTurso(syncUrl.trim(), authToken.trim())
      toast.success('Connected and synced. Restart not required.')
      setHasCreds(true)
      setAuthToken('')
    } catch (e) {
      toast.error(`Connection failed: ${e}`)
    } finally {
      setConnecting(false)
    }
  }

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
                database. Use the same credentials on another device to sync.
              </p>
              <div className="flex gap-2">
                <Button onClick={handleSync} disabled={syncing}>
                  {syncing ? 'Syncing…' : 'Sync now'}
                </Button>
                <Button variant="outline" onClick={handleDisconnect}>
                  Disconnect
                </Button>
              </div>
            </>
          ) : (
            <>
              <div className="grid gap-2">
                <Label htmlFor="url">Database URL</Label>
                <Input
                  id="url"
                  placeholder="libsql://your-db.turso.io"
                  value={syncUrl}
                  onChange={(e) => setSyncUrl(e.target.value)}
                />
              </div>
              <div className="grid gap-2">
                <Label htmlFor="token">Auth token</Label>
                <Input
                  id="token"
                  type="password"
                  placeholder="Paste your Turso auth token"
                  value={authToken}
                  onChange={(e) => setAuthToken(e.target.value)}
                />
              </div>
              <Button onClick={handleConnect} disabled={connecting}>
                {connecting ? 'Connecting…' : 'Connect & sync'}
              </Button>
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
            <li>Paste both above and select Connect &amp; sync.</li>
          </ol>
          <p className="mt-3 text-xs text-muted-foreground">
            Your token is stored on this device only and grants access to your
            own database.
          </p>
        </CardContent>
      </Card>
    </div>
  )
}
