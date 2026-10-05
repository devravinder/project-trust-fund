import { useEffect, useMemo, useState } from 'react'
import {
  Bar,
  BarChart,
  CartesianGrid,
  Cell,
  Legend,
  Pie,
  PieChart,
  ResponsiveContainer,
  Tooltip,
  XAxis,
  YAxis,
} from 'recharts'
import { ChevronLeft, ChevronRight } from 'lucide-react'
import { toast } from 'sonner'
import { PageHeader } from '@/components/layout/PageHeader'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { formatCurrency } from '@/lib/format'
import { reports, type MonthlyPoint, type PersonReport } from '@/lib/api'

type Tab = 'time' | 'person'

const DONUT_COLORS = ['#16a34a', '#f59e0b']
const WINDOW = 6 // months shown at once

/** Format a "YYYY-MM" string as "Mon YYYY". */
function monthLabel(ym: string): string {
  const [y, m] = ym.split('-').map(Number)
  if (!y || !m) return ym
  const d = new Date(y, m - 1, 1)
  return d.toLocaleDateString('en-IN', { month: 'short', year: 'numeric' })
}

export function ReportsPage() {
  const [tab, setTab] = useState<Tab>('time')
  const [monthly, setMonthly] = useState<MonthlyPoint[]>([])
  const [people, setPeople] = useState<PersonReport[]>([])
  const [loading, setLoading] = useState(true)
  // Offset from the latest window: 0 = most recent WINDOW months, 1 = older, etc.
  const [monthOffset, setMonthOffset] = useState(0)

  useEffect(() => {
    async function load() {
      setLoading(true)
      try {
        const [m, p] = await Promise.all([
          reports.monthly(),
          reports.byPerson(),
        ])
        setMonthly(m)
        setPeople(p)
        setMonthOffset(0)
      } catch (e) {
        toast.error(String(e))
      } finally {
        setLoading(false)
      }
    }
    void load()
  }, [])

  // Window the monthly data (data is ascending by month). offset pages backward.
  const windowed = useMemo(() => {
    const end = monthly.length - monthOffset * WINDOW
    const start = Math.max(0, end - WINDOW)
    return monthly.slice(start, Math.max(start, end))
  }, [monthly, monthOffset])

  const canOlder = monthly.length - (monthOffset + 1) * WINDOW > 0
  const canNewer = monthOffset > 0
  const first = windowed[0]
  const last = windowed[windowed.length - 1]
  const rangeLabel = !first
    ? ''
    : first === last
      ? monthLabel(first.month)
      : `${monthLabel(first.month)} – ${monthLabel(last!.month)}`

  const totalRecovered = people.reduce((s, p) => s + p.principal_recovered, 0)
  const totalOutstanding = people.reduce(
    (s, p) => s + p.outstanding_principal,
    0,
  )
  const donutData = [
    { name: 'Recovered', value: Math.max(totalRecovered, 0) },
    { name: 'Outstanding', value: Math.max(totalOutstanding, 0) },
  ]

  return (
    <div>
      <PageHeader title="Reports" description="By time and by person" />

      <div className="mb-4 flex gap-2">
        <Button
          variant={tab === 'time' ? 'default' : 'outline'}
          size="sm"
          onClick={() => setTab('time')}
        >
          By time
        </Button>
        <Button
          variant={tab === 'person' ? 'default' : 'outline'}
          size="sm"
          onClick={() => setTab('person')}
        >
          By person
        </Button>
      </div>

      {loading ? (
        <p className="text-muted-foreground">Loading…</p>
      ) : tab === 'time' ? (
        <div className="grid gap-4">
          <Card>
            <CardHeader>
              <div className="flex items-center justify-between gap-2">
                <CardTitle>Collected per month</CardTitle>
                <div className="flex items-center gap-1">
                  <Button
                    variant="outline"
                    size="icon"
                    aria-label="Older months"
                    disabled={!canOlder}
                    onClick={() => setMonthOffset((o) => o + 1)}
                  >
                    <ChevronLeft />
                  </Button>
                  <Button
                    variant="outline"
                    size="icon"
                    aria-label="Newer months"
                    disabled={!canNewer}
                    onClick={() => setMonthOffset((o) => Math.max(0, o - 1))}
                  >
                    <ChevronRight />
                  </Button>
                </div>
              </div>
              {rangeLabel && (
                <p className="text-xs text-muted-foreground">{rangeLabel}</p>
              )}
            </CardHeader>
            <CardContent>
              {windowed.length === 0 ? (
                <p className="text-muted-foreground">No payment data yet.</p>
              ) : (
                <ResponsiveContainer width="100%" height={280}>
                  <BarChart data={windowed}>
                    <CartesianGrid strokeDasharray="3 3" />
                    <XAxis dataKey="month" tickFormatter={monthLabel} />
                    <YAxis />
                    <Tooltip
                      labelFormatter={(l) => monthLabel(String(l))}
                      formatter={(v) => formatCurrency(Number(v))}
                    />
                    <Legend />
                    <Bar
                      dataKey="interest_collected"
                      name="Interest"
                      fill="#6366f1"
                    />
                    <Bar
                      dataKey="principal_recovered"
                      name="Principal"
                      fill="#16a34a"
                    />
                  </BarChart>
                </ResponsiveContainer>
              )}
            </CardContent>
          </Card>

          <Card>
            <CardHeader>
              <CardTitle>Recovered vs outstanding</CardTitle>
            </CardHeader>
            <CardContent>
              {totalRecovered + totalOutstanding === 0 ? (
                <p className="text-muted-foreground">No loan data yet.</p>
              ) : (
                <ResponsiveContainer width="100%" height={260}>
                  <PieChart>
                    <Pie
                      data={donutData}
                      dataKey="value"
                      nameKey="name"
                      innerRadius={60}
                      outerRadius={90}
                      label
                    >
                      {donutData.map((_, i) => (
                        <Cell key={i} fill={DONUT_COLORS[i]} />
                      ))}
                    </Pie>
                    <Tooltip formatter={(v) => formatCurrency(Number(v))} />
                    <Legend />
                  </PieChart>
                </ResponsiveContainer>
              )}
            </CardContent>
          </Card>
        </div>
      ) : (
        <Card>
          <CardHeader>
            <CardTitle>By person</CardTitle>
          </CardHeader>
          <CardContent>
            {people.length === 0 ? (
              <p className="text-muted-foreground">No borrowers yet.</p>
            ) : (
              <div className="overflow-x-auto">
                <table className="w-full text-sm">
                  <thead>
                    <tr className="border-b text-left text-muted-foreground">
                      <th className="py-2 pr-4">Borrower</th>
                      <th className="py-2 pr-4 text-right">Lent</th>
                      <th className="py-2 pr-4 text-right">Recovered</th>
                      <th className="py-2 pr-4 text-right">Interest</th>
                      <th className="py-2 text-right">Outstanding</th>
                    </tr>
                  </thead>
                  <tbody>
                    {people.map((p) => (
                      <tr key={p.borrower_id} className="border-b">
                        <td className="py-2 pr-4">{p.borrower_name}</td>
                        <td className="py-2 pr-4 text-right">
                          {formatCurrency(p.total_lent)}
                        </td>
                        <td className="py-2 pr-4 text-right">
                          {formatCurrency(p.principal_recovered)}
                        </td>
                        <td className="py-2 pr-4 text-right">
                          {formatCurrency(p.interest_collected)}
                        </td>
                        <td className="py-2 text-right">
                          {formatCurrency(p.outstanding_principal)}
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            )}
          </CardContent>
        </Card>
      )}
    </div>
  )
}
