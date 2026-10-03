import { useCallback, useEffect, useState } from 'react'
import { useParams, useNavigate } from 'react-router-dom'
import { ArrowLeft, Plus } from 'lucide-react'
import { toast } from 'sonner'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { formatCurrency, formatDate, formatRate } from '@/lib/format'
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
      toast.success(`Loan marked ${status.replace('_', ' ')}`)
      void load()
    } catch (e) {
      toast.error(String(e))
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
            <p className="text-sm text-muted-foreground">Status</p>
            <p className="text-xl font-bold capitalize">
              {summary.status.replace('_', ' ')}
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
            Write off
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
                <div
                  key={p.id}
                  className="flex items-center justify-between border-b pb-2 text-sm last:border-0"
                >
                  <span>{formatDate(p.paid_date)}</span>
                  <span className="text-muted-foreground">
                    int {formatCurrency(p.interest_component)} · prin{' '}
                    {formatCurrency(p.principal_component)}
                  </span>
                  <span className="font-medium">
                    {formatCurrency(p.amount)}
                  </span>
                </div>
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
    </div>
  )
}
