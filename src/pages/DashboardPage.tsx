import { useEffect, useState } from 'react'
import { useNavigate } from 'react-router-dom'
import { X } from 'lucide-react'
import { toast } from 'sonner'
import { PageHeader } from '@/components/layout/PageHeader'
import { Button } from '@/components/ui/button'
import { Card, CardContent } from '@/components/ui/card'
import { formatCurrency, formatDate } from '@/lib/format'
import { dashboard, type DashboardSummary, type DueItem } from '@/lib/api'

const DISMISSED_KEY = 'trustfund-dismissed-dues'

function loadDismissed(): Set<string> {
  try {
    const raw = localStorage.getItem(DISMISSED_KEY)
    return new Set(raw ? (JSON.parse(raw) as string[]) : [])
  } catch {
    return new Set()
  }
}

function saveDismissed(set: Set<string>) {
  localStorage.setItem(DISMISSED_KEY, JSON.stringify([...set]))
}

/** Stable key for a due notification (loan + due date). */
function dueKey(d: DueItem): string {
  return `${d.loan_id}:${d.due_date}`
}

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
  const [dismissed, setDismissed] = useState<Set<string>>(() => loadDismissed())
  const [loading, setLoading] = useState(true)
  const navigate = useNavigate()

  const dismiss = (d: DueItem) => {
    setDismissed((prev) => {
      const next = new Set(prev)
      next.add(dueKey(d))
      saveDismissed(next)
      return next
    })
  }

  const visibleDues = dues.filter((d) => !dismissed.has(dueKey(d)))

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
          {visibleDues.length === 0 ? (
            <p className="text-muted-foreground">No dues this month.</p>
          ) : (
            <div className="grid gap-2">
              {visibleDues.map((d) => (
                <Card key={dueKey(d)}>
                  <CardContent className="flex items-center justify-between gap-2 p-4">
                    <button
                      type="button"
                      className="flex flex-1 items-center justify-between text-left"
                      onClick={() => navigate(`/loans/${d.loan_id}`)}
                    >
                      <div>
                        <p className="font-medium">{d.borrower_name}</p>
                        <p className="text-sm text-muted-foreground">
                          Due {formatDate(d.due_date)}
                        </p>
                      </div>
                      <p className="font-medium">
                        {formatCurrency(d.total_due)}
                      </p>
                    </button>
                    <Button
                      variant="ghost"
                      size="icon"
                      aria-label="Clear notification"
                      onClick={() => dismiss(d)}
                    >
                      <X />
                    </Button>
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
