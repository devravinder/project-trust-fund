import { useCallback, useEffect, useState } from 'react'
import { useParams, useNavigate } from 'react-router-dom'
import { ArrowLeft } from 'lucide-react'
import { toast } from 'sonner'
import { Button } from '@/components/ui/button'
import { Card, CardContent } from '@/components/ui/card'
import {
  formatCurrency,
  formatDate,
  formatRate,
  formatLoanStatus,
} from '@/lib/format'
import { borrowers, loans, type LoanSummary } from '@/lib/api'
import type { Borrower } from '@/types'

export function BorrowerDetailPage() {
  const { id = '' } = useParams()
  const navigate = useNavigate()
  const [borrower, setBorrower] = useState<Borrower | null>(null)
  const [outstanding, setOutstanding] = useState(0)
  const [borrowerLoans, setBorrowerLoans] = useState<LoanSummary[]>([])
  const [loading, setLoading] = useState(true)

  const load = useCallback(async () => {
    setLoading(true)
    try {
      const [b, total, allLoans] = await Promise.all([
        borrowers.get(id),
        borrowers.totalOutstanding(id),
        loans.list('all'),
      ])
      setBorrower(b)
      setOutstanding(total)
      setBorrowerLoans(allLoans.filter((l) => l.borrower_id === id))
    } catch (e) {
      toast.error(String(e))
    } finally {
      setLoading(false)
    }
  }, [id])

  useEffect(() => {
    void load()
  }, [load])

  if (loading) return <p className="text-muted-foreground">Loading…</p>
  if (!borrower)
    return <p className="text-muted-foreground">Borrower not found.</p>

  return (
    <div>
      <Button
        variant="ghost"
        size="sm"
        className="mb-3"
        onClick={() => navigate('/borrowers')}
      >
        <ArrowLeft /> Back to borrowers
      </Button>

      <div className="mb-4">
        <h1 className="text-2xl font-bold">{borrower.name}</h1>
      </div>

      <Card className="mb-4">
        <CardContent className="grid gap-2 p-4 text-sm">
          <div className="flex justify-between">
            <span className="text-muted-foreground">Phone</span>
            <span className="font-medium">{borrower.phone || '—'}</span>
          </div>
          <div className="flex justify-between gap-4">
            <span className="text-muted-foreground">Address</span>
            <span className="text-right font-medium">
              {borrower.address || '—'}
            </span>
          </div>
          {borrower.notes && (
            <div className="border-t pt-2">
              <span className="text-muted-foreground">Notes</span>
              <p className="mt-0.5 font-bold">{borrower.notes}</p>
            </div>
          )}
        </CardContent>
      </Card>

      <Card className="mb-6 max-w-sm">
        <CardContent className="p-4">
          <p className="text-sm text-muted-foreground">Total outstanding</p>
          <p className="text-2xl font-bold">{formatCurrency(outstanding)}</p>
        </CardContent>
      </Card>

      <h2 className="mb-2 text-lg font-semibold">Loans</h2>
      {borrowerLoans.length === 0 ? (
        <p className="text-muted-foreground">No loans for this borrower.</p>
      ) : (
        <div className="grid gap-2">
          {borrowerLoans.map((l) => (
            <Card
              key={l.id}
              className="cursor-pointer transition-colors hover:bg-accent"
              onClick={() =>
                navigate(`/loans/${l.id}`, {
                  state: { from: `/borrowers/${id}` },
                })
              }
            >
              <CardContent className="flex items-center justify-between p-4">
                <div>
                  <p className="font-medium">
                    {formatCurrency(l.principal)} · {formatRate(l.monthly_rate)}
                  </p>
                  <p className="text-sm text-muted-foreground">
                    {formatDate(l.start_date)} · {formatLoanStatus(l.status)}
                  </p>
                </div>
                <p className="text-sm">
                  {formatCurrency(l.outstanding_principal)} left
                </p>
              </CardContent>
            </Card>
          ))}
        </div>
      )}
    </div>
  )
}
