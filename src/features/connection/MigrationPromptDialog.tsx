import { useState } from 'react'
import { toast } from 'sonner'
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
  DialogFooter,
} from '@/components/ui/dialog'
import { Button } from '@/components/ui/button'
import { db } from '@/lib/api'

/**
 * Shown right after connecting to Turso when the local JSON store still has
 * data. Yes = push local data into Turso then delete the file. No = discard
 * the local file.
 */
export function MigrationPromptDialog({
  open,
  onOpenChange,
  onDone,
}: {
  open: boolean
  onOpenChange: (open: boolean) => void
  onDone: () => void
}) {
  const [busy, setBusy] = useState(false)

  const sync = async () => {
    setBusy(true)
    try {
      await db.migrateJsonToTurso()
      toast.success('Local data synced to Turso')
      onOpenChange(false)
      onDone()
    } catch (e) {
      toast.error(`Sync failed: ${e}`)
    } finally {
      setBusy(false)
    }
  }

  const discard = async () => {
    setBusy(true)
    try {
      await db.deleteLocalJson()
      toast.success('Local data discarded')
      onOpenChange(false)
      onDone()
    } catch (e) {
      toast.error(String(e))
    } finally {
      setBusy(false)
    }
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Sync local data to Turso?</DialogTitle>
          <DialogDescription>
            You have data saved on this device from offline use. Do you want to
            copy it into your Turso database? Choosing “No, discard” permanently
            deletes the local file.
          </DialogDescription>
        </DialogHeader>
        <DialogFooter>
          <Button variant="outline" onClick={discard} disabled={busy}>
            No, discard
          </Button>
          <Button onClick={sync} disabled={busy}>
            {busy ? 'Working…' : 'Yes, sync'}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
