import { useEffect, useState } from 'react'
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
import { toast } from 'sonner'
import { PageHeader } from '@/components/layout/PageHeader'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { formatCurrency } from '@/lib/format'
import { reports, type MonthlyPoint, type PersonReport } from '@/lib/api'

type Tab = 'time' | 'person'

const DONUT_COLORS = ['#16a34a', '#f59e0b']

export function ReportsPage() {
  const [tab, setTab] = useState<Tab>('time')
  const [monthly, setMonthly] = useState<MonthlyPoint[]>([])
  const [people, setPeople] = useState<PersonReport[]>([])
  const [loading, setLoading] = useState(true)

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
      } catch (e) {
        toast.error(String(e))
      } finally {
        setLoading(false)
      }
    }
    void load()
  }, [])

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
              <CardTitle>Collected per month</CardTitle>
            </CardHeader>
            <CardContent>
              {monthly.length === 0 ? (
                <p className="text-muted-foreground">No payment data yet.</p>
              ) : (
                <ResponsiveContainer width="100%" height={280}>
                  <BarChart data={monthly}>
                    <CartesianGrid strokeDasharray="3 3" />
                    <XAxis dataKey="month" />
                    <YAxis />
                    <Tooltip formatter={(v) => formatCurrency(Number(v))} />
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
