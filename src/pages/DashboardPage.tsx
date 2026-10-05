import { useEffect, useState } from 'react'
import { useNavigate } from 'react-router-dom'
import { toast } from 'sonner'
import { PageHeader } from '@/components/layout/PageHeader'
import { Card, CardContent } from '@/components/ui/card'
import { formatCurrency, formatDate } from '@/lib/format'
import {
  dashboard,
  type DashboardSummary,
  type DueItem,
  type OverdueLoan,
} from '@/lib/api'

function StatCard({
  label,
  value,
  hint,
  emphasis,
}: {
  label: string
  value: string
  hint?: string
  emphasis?: boolean
}) {
  return (
    <Card>
      <CardContent className="p-4">
        <p className="text-sm text-muted-foreground">{label}</p>
        <p
          className={
            emphasis
              ? 'text-2xl font-bold text-destructive'
              : 'text-2xl font-bold'
          }
        >
          {value}
        </p>
        {hint && <p className="text-xs text-muted-foreground">{hint}</p>}
      </CardContent>
    </Card>
  )
}

export function DashboardPage() {
  const [summary, setSummary] = useState<DashboardSummary | null>(null)
  const [dues, setDues] = useState<DueItem[]>([])
  const [overdue, setOverdue] = useState<OverdueLoan[]>([])
  const [loading, setLoading] = useState(true)
  const navigate = useNavigate()

  useEffect(() => {
    async function load() {
      setLoading(true)
      try {
        const [s, d] = await Promise.all([
          dashboard.summary(),
          dashboard.dues(5),
        ])
        setSummary(s)
        setDues(d)
        setOverdue(await dashboard.overdue(10))
      } catch (e) {
        toast.error(String(e))
      } finally {
        setLoading(false)
      }
    }
    void load()
  }, [])

  return (
    <div>
      <PageHeader
        title="Dashboard"
        description="Overview of your lending activity"
      />

      {loading || !summary ? (
        <p className="text-muted-foreground">Loading…</p>
      ) : (
        <>
          <div className="grid grid-cols-2 gap-3 lg:grid-cols-4">
            <StatCard
              label="Total lent"
              value={formatCurrency(summary.total_lent)}
              hint="Active + overdue"
            />
            <StatCard
              label="Outstanding"
              value={formatCurrency(summary.outstanding_principal)}
              hint="Principal to recover"
            />
            <StatCard
              label="Interest collected"
              value={formatCurrency(summary.interest_collected)}
              hint="All time"
            />
            <StatCard
              label="Interest due (month)"
              value={formatCurrency(summary.interest_due_this_month)}
            />
            <StatCard
              label="Active loans"
              value={String(summary.active_loans)}
            />
            <StatCard
              label="Overdue loans"
              value={String(summary.overdue_loans)}
              emphasis={summary.overdue_loans > 0}
            />
            <StatCard
              label="Overdue amount"
              value={formatCurrency(summary.overdue_amount)}
              emphasis={summary.overdue_amount > 0}
            />
            <StatCard
              label="Principal recovered"
              value={formatCurrency(summary.principal_recovered)}
            />
          </div>

          <h2 className="mb-2 mt-6 text-lg font-semibold">Dues this month</h2>
          {dues.length === 0 ? (
            <p className="text-muted-foreground">No dues this month.</p>
          ) : (
            <div className="grid gap-2">
              {dues.map((d, i) => (
                <Card
                  key={`${d.loan_id}:${d.due_date}:${i}`}
                  className="cursor-pointer transition-colors hover:bg-accent"
                  onClick={() => navigate(`/loans/${d.loan_id}`)}
                >
                  <CardContent className="flex items-center justify-between gap-2 p-4">
                    <div>
                      <p className="font-medium">{d.borrower_name}</p>
                      <p className="text-sm text-muted-foreground">
                        Due {formatDate(d.due_date)}
                      </p>
                    </div>
                    <p className="font-medium">{formatCurrency(d.total_due)}</p>
                  </CardContent>
                </Card>
              ))}
            </div>
          )}

          <h2 className="mb-2 mt-6 text-lg font-semibold">Overdue loans</h2>
          {overdue.length === 0 ? (
            <p className="text-muted-foreground">No overdue loans.</p>
          ) : (
            <div className="grid gap-2">
              {overdue.map((o) => (
                <Card
                  key={o.loan_id}
                  className="cursor-pointer transition-colors hover:bg-accent"
                  onClick={() => navigate(`/loans/${o.loan_id}`)}
                >
                  <CardContent className="flex items-center justify-between gap-2 p-4">
                    <div>
                      <p className="font-medium">{o.borrower_name}</p>
                      <p className="text-sm text-destructive">
                        {o.days_overdue} day{o.days_overdue === 1 ? '' : 's'}{' '}
                        overdue · ended {formatDate(o.end_date)}
                      </p>
                    </div>
                    <div className="text-right">
                      <p className="font-medium">
                        {formatCurrency(o.total_due)}
                      </p>
                      <p className="text-xs text-muted-foreground">
                        {formatCurrency(o.outstanding)} + int{' '}
                        {formatCurrency(o.interest_due)}
                      </p>
                    </div>
                  </CardContent>
                </Card>
              ))}
            </div>
          )}
        </>
      )}
    </div>
  )
}
