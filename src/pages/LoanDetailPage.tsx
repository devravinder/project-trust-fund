import { useCallback, useEffect, useState } from 'react'
import { useParams, useNavigate } from 'react-router-dom'
import { ArrowLeft, Plus, Trash2 } from 'lucide-react'
import { toast } from 'sonner'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
  DialogFooter,
} from '@/components/ui/dialog'
import {
  formatCurrency,
  formatDate,
  formatRate,
  formatLoanStatus,
} from '@/lib/format'
import { loans, payments as paymentsApi, type LoanSummary } from '@/lib/api'
import type { LoanStatus, Payment, ScheduleItem } from '@/types'
import { PaymentFormDialog } from '@/features/payments/PaymentFormDialog'

export function LoanDetailPage() {
  const { id = '' } = useParams()
  const navigate = useNavigate()
  const [summary, setSummary] = useState<LoanSummary | null>(null)
  const [schedule, setSchedule] = useState<ScheduleItem[]>([])
  const [loanPayments, setLoanPayments] = useState<Payment[]>([])
  const [loading, setLoading] = useState(true)
  const [payOpen, setPayOpen] = useState(false)
  const [deleteOpen, setDeleteOpen] = useState(false)
  const [deleting, setDeleting] = useState(false)

  const load = useCallback(async () => {
    setLoading(true)
    try {
      const [s, sch, pays] = await Promise.all([
        loans.summary(id),
        loans.schedule(id),
        paymentsApi.forLoan(id),
      ])
      setSummary(s)
      setSchedule(sch)
      setLoanPayments(pays)
    } catch (e) {
      toast.error(String(e))
    } finally {
      setLoading(false)
    }
  }, [id])

  useEffect(() => {
    void load()
  }, [load])

  const changeStatus = async (status: LoanStatus) => {
    try {
      await loans.setStatus(id, status)
      toast.success(`Loan marked ${formatLoanStatus(status).toLowerCase()}`)
      void load()
    } catch (e) {
      toast.error(String(e))
    }
  }

  const handleDelete = async () => {
    setDeleting(true)
    try {
      await loans.removeWithRelated(id)
      toast.success('Loan and related data deleted')
      setDeleteOpen(false)
      navigate('/loans')
    } catch (e) {
      toast.error(String(e))
    } finally {
      setDeleting(false)
    }
  }

  if (loading) return <p className="text-muted-foreground">Loading…</p>
  if (!summary) return <p className="text-muted-foreground">Loan not found.</p>

  return (
    <div>
      <Button
        variant="ghost"
        size="sm"
        className="mb-3"
        onClick={() => navigate('/loans')}
      >
        <ArrowLeft /> Back to loans
      </Button>

      <div className="mb-4 flex items-start justify-between">
        <div>
          <h1 className="text-2xl font-bold">{summary.borrower_name}</h1>
          <p className="text-sm text-muted-foreground">
            {formatCurrency(summary.principal)} ·{' '}
            {formatRate(summary.monthly_rate)} ·{' '}
            {summary.repayment_mode.replace('_', '-')} ·{' '}
            {formatDate(summary.start_date)}
          </p>
          {summary.note && (
            <p className="mt-1 text-sm font-bold">{summary.note}</p>
          )}
        </div>
        <Button onClick={() => setPayOpen(true)}>
          <Plus /> Payment
        </Button>
      </div>

      <div className="grid grid-cols-2 gap-3 lg:grid-cols-4">
        <Card>
          <CardContent className="p-4">
            <p className="text-sm text-muted-foreground">Outstanding</p>
            <p className="text-xl font-bold">
              {formatCurrency(summary.outstanding_principal)}
            </p>
          </CardContent>
        </Card>
        <Card>
          <CardContent className="p-4">
            <p className="text-sm text-muted-foreground">Principal recovered</p>
            <p className="text-xl font-bold">
              {formatCurrency(summary.principal_recovered)}
            </p>
          </CardContent>
        </Card>
        <Card>
          <CardContent className="p-4">
            <p className="text-sm text-muted-foreground">Interest collected</p>
            <p className="text-xl font-bold">
              {formatCurrency(summary.interest_collected)}
            </p>
          </CardContent>
        </Card>
        <Card>
          <CardContent className="p-4">
            <p className="text-sm text-muted-foreground">
              Interest to date
            </p>
            <p className="text-xl font-bold">
              {formatCurrency(summary.interest_accrued_to_date)}
            </p>
            <p className="text-xs text-muted-foreground">
              accrued from {formatDate(summary.start_date)}
            </p>
          </CardContent>
        </Card>
        <Card>
          <CardContent className="p-4">
            <p className="text-sm text-muted-foreground">Interest due</p>
            <p className="text-xl font-bold text-destructive">
              {formatCurrency(summary.interest_due)}
            </p>
            <p className="text-xs text-muted-foreground">accrued − collected</p>
          </CardContent>
        </Card>
        <Card>
          <CardContent className="p-4">
            <p className="text-sm text-muted-foreground">Status</p>
            <p className="text-xl font-bold capitalize">
              {formatLoanStatus(summary.status)}
            </p>
          </CardContent>
        </Card>
      </div>

      <div className="mt-4 flex flex-wrap gap-2">
        {summary.status !== 'closed' && (
          <Button
            variant="outline"
            size="sm"
            onClick={() => changeStatus('closed')}
          >
            Mark closed
          </Button>
        )}
        {summary.status !== 'written_off' && (
          <Button
            variant="outline"
            size="sm"
            onClick={() => changeStatus('written_off')}
          >
            Mark as lost
          </Button>
        )}
        {summary.status !== 'active' && (
          <Button
            variant="outline"
            size="sm"
            onClick={() => changeStatus('active')}
          >
            Reopen (active)
          </Button>
        )}
        <Button
          variant="destructive"
          size="sm"
          onClick={() => setDeleteOpen(true)}
        >
          <Trash2 /> Delete loan
        </Button>
      </div>

      {schedule.length > 0 && (
        <Card className="mt-6">
          <CardHeader>
            <CardTitle>Installment schedule</CardTitle>
          </CardHeader>
          <CardContent className="overflow-x-auto">
            <table className="w-full text-sm">
              <thead>
                <tr className="border-b text-left text-muted-foreground">
                  <th className="py-2 pr-4">#</th>
                  <th className="py-2 pr-4">Due</th>
                  <th className="py-2 pr-4 text-right">Principal</th>
                  <th className="py-2 pr-4 text-right">Interest</th>
                  <th className="py-2 pr-4 text-right">Total</th>
                  <th className="py-2 text-right">Status</th>
                </tr>
              </thead>
              <tbody>
                {schedule.map((s) => (
                  <tr key={s.id} className="border-b">
                    <td className="py-2 pr-4">{s.seq}</td>
                    <td className="py-2 pr-4">{formatDate(s.due_date)}</td>
                    <td className="py-2 pr-4 text-right">
                      {formatCurrency(s.principal_due)}
                    </td>
                    <td className="py-2 pr-4 text-right">
                      {formatCurrency(s.interest_due)}
                    </td>
                    <td className="py-2 pr-4 text-right">
                      {formatCurrency(s.total_due)}
                    </td>
                    <td className="py-2 text-right capitalize">{s.status}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </CardContent>
        </Card>
      )}

      <Card className="mt-6">
        <CardHeader>
          <CardTitle>Payments</CardTitle>
        </CardHeader>
        <CardContent>
          {loanPayments.length === 0 ? (
            <p className="text-muted-foreground">No payments yet.</p>
          ) : (
            <div className="grid gap-2">
              {loanPayments.map((p) => (
                <button
                  key={p.id}
                  type="button"
                  onClick={() => navigate(`/payments/${p.id}`)}
                  className="w-full rounded-md border-b pb-2 text-left text-sm last:border-0 hover:bg-accent"
                >
                  <div className="flex items-center justify-between">
                    <span>{formatDate(p.paid_date)}</span>
                    <span className="text-muted-foreground">
                      int {formatCurrency(p.interest_component)} · prin{' '}
                      {formatCurrency(p.principal_component)}
                    </span>
                    <span className="font-medium">
                      {formatCurrency(p.amount)}
                    </span>
                  </div>
                  {p.note && (
                    <p className="mt-0.5 text-xs font-bold">{p.note}</p>
                  )}
                </button>
              ))}
            </div>
          )}
        </CardContent>
      </Card>

      <PaymentFormDialog
        open={payOpen}
        onOpenChange={setPayOpen}
        onSaved={() => void load()}
        fixedLoanId={id}
      />

      <Dialog open={deleteOpen} onOpenChange={setDeleteOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Delete this loan?</DialogTitle>
            <DialogDescription>
              This permanently deletes the loan for{' '}
              <span className="font-medium text-foreground">
                {summary.borrower_name}
              </span>{' '}
              along with all its payments and installment schedule. This cannot
              be undone.
            </DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <Button
              variant="outline"
              onClick={() => setDeleteOpen(false)}
              disabled={deleting}
            >
              Cancel
            </Button>
            <Button
              variant="destructive"
              onClick={handleDelete}
              disabled={deleting}
            >
              {deleting ? 'Deleting…' : 'Delete loan'}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  )
}
