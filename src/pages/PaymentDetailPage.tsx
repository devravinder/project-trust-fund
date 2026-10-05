import { useCallback, useEffect, useState } from 'react'
import { useParams, useNavigate } from 'react-router-dom'
import { ArrowLeft, Trash2 } from 'lucide-react'
import { toast } from 'sonner'
import { Button } from '@/components/ui/button'
import { Card, CardContent } from '@/components/ui/card'
import { ConfirmDialog } from '@/components/ui/confirm-dialog'
import { formatCurrency, formatDate } from '@/lib/format'
import { payments, type PaymentDetail } from '@/lib/api'

function Row({ label, value }: { label: string; value: string }) {
  return (
    <div className="flex justify-between py-1 text-sm">
      <span className="text-muted-foreground">{label}</span>
      <span className="font-medium">{value}</span>
    </div>
  )
}

export function PaymentDetailPage() {
  const { id = '' } = useParams()
  const navigate = useNavigate()
  const [p, setP] = useState<PaymentDetail | null>(null)
  const [loading, setLoading] = useState(true)
  const [deleteOpen, setDeleteOpen] = useState(false)

  const load = useCallback(async () => {
    setLoading(true)
    try {
      setP(await payments.get(id))
    } catch (e) {
      toast.error(String(e))
    } finally {
      setLoading(false)
    }
  }, [id])

  useEffect(() => {
    void load()
  }, [load])

  const confirmDelete = async () => {
    try {
      await payments.remove(id)
      toast.success('Payment deleted')
      navigate(-1)
    } catch (e) {
      toast.error(String(e))
    }
  }

  if (loading) return <p className="text-muted-foreground">Loading…</p>
  if (!p) return <p className="text-muted-foreground">Payment not found.</p>

  return (
    <div>
      <Button
        variant="ghost"
        size="sm"
        className="mb-3"
        onClick={() => navigate(-1)}
      >
        <ArrowLeft /> Back
      </Button>

      <div className="mb-4 flex items-start justify-between">
        <div>
          <h1 className="text-2xl font-bold">{formatCurrency(p.amount)}</h1>
          <p className="text-sm text-muted-foreground">
            {p.borrower_name} · {formatDate(p.paid_date)}
          </p>
        </div>
        <Button
          variant="destructive"
          size="sm"
          onClick={() => setDeleteOpen(true)}
        >
          <Trash2 /> Delete
        </Button>
      </div>

      <Card>
        <CardContent className="p-4">
          <Row label="Amount" value={formatCurrency(p.amount)} />
          <Row
            label="Interest paid"
            value={formatCurrency(p.interest_component)}
          />
          <Row
            label="Principal paid"
            value={formatCurrency(p.principal_component)}
          />
          <Row label="Paid date" value={formatDate(p.paid_date)} />
          <Row label="Borrower" value={p.borrower_name} />
          <Row label="Loan amount" value={formatCurrency(p.loan_principal)} />
          <Row label="Loan start" value={formatDate(p.loan_start_date)} />
          {p.note && (
            <div className="mt-2 border-t pt-2 text-sm">
              <p className="text-muted-foreground">Note</p>
              <p className="italic">“{p.note}”</p>
            </div>
          )}
        </CardContent>
      </Card>

      <div className="mt-4">
        <Button variant="outline" onClick={() => navigate(`/loans/${p.loan_id}`)}>
          View loan
        </Button>
      </div>

      <ConfirmDialog
        open={deleteOpen}
        onOpenChange={setDeleteOpen}
        title="Delete payment?"
        description="This removes the payment and recalculates the loan's balances."
        confirmText="Delete"
        destructive
        onConfirm={confirmDelete}
      />
    </div>
  )
}
