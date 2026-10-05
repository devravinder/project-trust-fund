import { useCallback, useEffect, useState } from 'react'
import { Plus, Search, Trash2 } from 'lucide-react'
import { toast } from 'sonner'
import { PageHeader } from '@/components/layout/PageHeader'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Card, CardContent } from '@/components/ui/card'
import { formatCurrency, formatDate } from '@/lib/format'
import { payments, type PaymentView } from '@/lib/api'
import { PaymentFormDialog } from '@/features/payments/PaymentFormDialog'
import { ConfirmDialog } from '@/components/ui/confirm-dialog'

export function PaymentsPage() {
  const [items, setItems] = useState<PaymentView[]>([])
  const [search, setSearch] = useState('')
  const [loading, setLoading] = useState(true)
  const [dialogOpen, setDialogOpen] = useState(false)
  const [deleteTarget, setDeleteTarget] = useState<PaymentView | null>(null)

  const load = useCallback(async (term: string) => {
    setLoading(true)
    try {
      setItems(await payments.list(term || undefined))
    } catch (e) {
      toast.error(String(e))
    } finally {
      setLoading(false)
    }
  }, [])

  useEffect(() => {
    const t = setTimeout(() => void load(search), 250)
    return () => clearTimeout(t)
  }, [search, load])

  const confirmDelete = async () => {
    if (!deleteTarget) return
    try {
      await payments.remove(deleteTarget.id)
      toast.success('Payment deleted')
      void load(search)
    } catch (e) {
      toast.error(String(e))
    }
  }

  return (
    <div>
      <PageHeader
        title="Payments"
        description="Repayments received"
        actions={
          <Button onClick={() => setDialogOpen(true)}>
            <Plus /> Record
          </Button>
        }
      />

      <div className="relative mb-4">
        <Search className="absolute left-3 top-1/2 size-4 -translate-y-1/2 text-muted-foreground" />
        <Input
          placeholder="Search by borrower name"
          className="pl-9"
          value={search}
          onChange={(e) => setSearch(e.target.value)}
        />
      </div>

      {loading ? (
        <p className="text-muted-foreground">Loading…</p>
      ) : items.length === 0 ? (
        <p className="text-muted-foreground">No payments recorded yet.</p>
      ) : (
        <div className="grid gap-2">
          {items.map((p) => (
            <Card key={p.id}>
              <CardContent className="flex items-center justify-between p-4">
                <div>
                  <p className="font-medium">
                    {p.borrower_name} · {formatCurrency(p.amount)}
                  </p>
                  <p className="text-sm text-muted-foreground">
                    {formatDate(p.paid_date)} · interest{' '}
                    {formatCurrency(p.interest_component)} · principal{' '}
                    {formatCurrency(p.principal_component)}
                  </p>
                </div>
                <Button
                  variant="ghost"
                  size="icon"
                  onClick={() => setDeleteTarget(p)}
                  aria-label="Delete payment"
                >
                  <Trash2 />
                </Button>
              </CardContent>
            </Card>
          ))}
        </div>
      )}

      <PaymentFormDialog
        open={dialogOpen}
        onOpenChange={setDialogOpen}
        onSaved={() => void load(search)}
      />

      <ConfirmDialog
        open={deleteTarget !== null}
        onOpenChange={(o) => !o && setDeleteTarget(null)}
        title="Delete payment?"
        description="This removes the payment and recalculates the loan's balances."
        confirmText="Delete"
        destructive
        onConfirm={confirmDelete}
      />
    </div>
  )
}
