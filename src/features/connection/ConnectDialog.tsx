import { useState } from 'react'
import { toast } from 'sonner'
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { db } from '@/lib/api'
import { decodeConnection } from '@/lib/connection-code'
import { QrScanner } from '@/features/connection/QrScanner'

type Mode = 'scan' | 'manual'

export function ConnectDialog({
  open,
  onOpenChange,
  onConnected,
}: {
  open: boolean
  onOpenChange: (open: boolean) => void
  onConnected: () => void
}) {
  const [mode, setMode] = useState<Mode>('scan') // scan is the default
  const [url, setUrl] = useState('')
  const [token, setToken] = useState('')
  const [connecting, setConnecting] = useState(false)

  const connect = async (syncUrl: string, authToken: string) => {
    setConnecting(true)
    try {
      await db.connectTurso(syncUrl.trim(), authToken.trim())
      toast.success('Connected and synced')
      onOpenChange(false)
      onConnected()
    } catch (e) {
      toast.error(`Connection failed: ${e}`)
    } finally {
      setConnecting(false)
    }
  }

  const handleScan = (text: string) => {
    try {
      const { url: u, token: t } = decodeConnection(text)
      void connect(u, t)
    } catch (e) {
      toast.error(String(e))
    }
  }

  return (
    <Dialog
      open={open}
      onOpenChange={(o) => {
        if (!o) {
          setMode('scan')
          setUrl('')
          setToken('')
        }
        onOpenChange(o)
      }}
    >
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Connect a database</DialogTitle>
        </DialogHeader>

        <div className="mb-2 flex gap-2">
          <Button
            variant={mode === 'scan' ? 'default' : 'outline'}
            size="sm"
            onClick={() => setMode('scan')}
          >
            Scan QR
          </Button>
          <Button
            variant={mode === 'manual' ? 'default' : 'outline'}
            size="sm"
            onClick={() => setMode('manual')}
          >
            Enter manually
          </Button>
        </div>

        {mode === 'scan' ? (
          <div className="grid gap-3">
            <QrScanner
              onResult={handleScan}
              onError={(m) => {
                toast.error(m)
                setMode('manual')
              }}
            />
            <p className="text-center text-xs text-muted-foreground">
              Point the camera at a TrustFund connection QR from another device.
            </p>
          </div>
        ) : (
          <div className="grid gap-4">
            <div className="grid gap-2">
              <Label htmlFor="curl">Database URL</Label>
              <Input
                id="curl"
                placeholder="libsql://your-db.turso.io"
                value={url}
                onChange={(e) => setUrl(e.target.value)}
              />
            </div>
            <div className="grid gap-2">
              <Label htmlFor="ctoken">Auth token</Label>
              <Input
                id="ctoken"
                type="password"
                value={token}
                onChange={(e) => setToken(e.target.value)}
              />
            </div>
            <Button
              onClick={() => connect(url, token)}
              disabled={connecting || !url || !token}
            >
              {connecting ? 'Connecting…' : 'Connect & sync'}
            </Button>
          </div>
        )}
      </DialogContent>
    </Dialog>
  )
}
