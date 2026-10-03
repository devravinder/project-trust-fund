import { useEffect, useRef, useState } from 'react'
import QRCode from 'qrcode'
import { toast } from 'sonner'
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { db } from '@/lib/api'
import { encodeConnection } from '@/lib/connection-code'

export function ShareConnectionDialog({
  open,
  onOpenChange,
}: {
  open: boolean
  onOpenChange: (open: boolean) => void
}) {
  const canvasRef = useRef<HTMLCanvasElement>(null)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    if (!open) return
    setError(null)
    db.getCredentials()
      .then((creds) => {
        if (!creds) {
          setError('No connection configured on this device yet.')
          return
        }
        const payload = encodeConnection(creds.sync_url, creds.auth_token)
        if (canvasRef.current) {
          QRCode.toCanvas(canvasRef.current, payload, { width: 240 }, (err) => {
            if (err) setError(String(err))
          })
        }
      })
      .catch((e) => toast.error(String(e)))
  }, [open])

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Share connection</DialogTitle>
        </DialogHeader>
        {error ? (
          <p className="text-sm text-muted-foreground">{error}</p>
        ) : (
          <div className="flex flex-col items-center gap-3">
            <canvas ref={canvasRef} className="rounded-md bg-white p-2" />
            <p className="text-center text-xs text-muted-foreground">
              Scan this on another device to connect to the same database. Treat
              it like a password — it contains your auth token.
            </p>
          </div>
        )}
      </DialogContent>
    </Dialog>
  )
}
