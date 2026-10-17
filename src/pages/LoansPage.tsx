import { useCallback, useEffect, useState } from 'react'
import { useNavigate } from 'react-router-dom'
import { Plus, Search } from 'lucide-react'
import { toast } from 'sonner'
import { PageHeader } from '@/components/layout/PageHeader'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Card, CardContent } from '@/components/ui/card'
import { cn } from '@/lib/utils'
import {
  formatCurrency,
  formatRate,
  formatDate,
  formatLoanStatus,
} from '@/lib/format'
import { loans, type LoanSummary } from '@/lib/api'
import type { LoanStatus } from '@/types'
import { LoanFormDialog } from '@/features/loans/LoanFormDialog'
import { useInfiniteList } from '@/lib/useInfiniteList'
import { VirtualList } from '@/components/VirtualList'

const FILTERS: { key: LoanStatus | 'all'; label: string }[] = [
  { key: 'active', label: 'Active' },
  { key: 'overdue', label: 'Overdue' },
  { key: 'closed', label: 'Closed' },
  { key: 'all', label: 'All' },
]

const statusColor: Record<string, string> = {
  active: 'text-green-600',
  overdue: 'text-destructive',
  closed: 'text-muted-foreground',
  written_off: 'text-muted-foreground',
}

export function LoansPage() {
  const [filter, setFilter] = useState<LoanStatus | 'all'>('active')
  const [search, setSearch] = useState('')
  const [debounced, setDebounced] = useState('')
  const [dialogOpen, setDialogOpen] = useState(false)
  const navigate = useNavigate()

  useEffect(() => {
    const t = setTimeout(() => setDebounced(search), 250)
    return () => clearTimeout(t)
  }, [search])

  const fetchPage = useCallback(
    (limit: number, offset: number) =>
      loans.page(limit, offset, filter, debounced || undefined),
    [filter, debounced],
  )

  const { items, loading, loadingMore, hasMore, loadMore, reload } =
    useInfiniteList<LoanSummary>(fetchPage, [filter, debounced], (e) =>
      toast.error(String(e)),
    )

  return (
    <div className="flex h-full flex-col">
      <PageHeader
        title="Loans"
        description="Track active and closed loans"
        actions={
          <Button onClick={() => setDialogOpen(true)}>
            <Plus /> New loan
          </Button>
        }
      />

      <div className="mb-4 flex flex-wrap gap-2">
        {FILTERS.map((f) => (
          <Button
            key={f.key}
            variant={filter === f.key ? 'default' : 'outline'}
            size="sm"
            onClick={() => setFilter(f.key)}
          >
            {f.label}
          </Button>
        ))}
      </div>

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
        <p className="text-muted-foreground">No loans found.</p>
      ) : (
        <div className="min-h-0 flex-1">
          <VirtualList
            items={items}
            estimateSize={92}
            onReachEnd={loadMore}
            hasMore={hasMore}
            getKey={(l) => l.id}
            footer={
              loadingMore ? (
                <p className="py-2 text-center text-sm text-muted-foreground">
                  Loading more…
                </p>
              ) : null
            }
            renderItem={(l) => (
              <Card
                className="cursor-pointer transition-colors hover:bg-accent"
                onClick={() =>
                  navigate(`/loans/${l.id}`, { state: { from: '/loans' } })
                }
              >
                <CardContent className="p-4">
                  <div className="flex items-start justify-between">
                    <div>
                      <p className="font-medium">{l.borrower_name}</p>
                      <p className="text-sm text-muted-foreground">
                        {formatCurrency(l.principal)} ·{' '}
                        {formatRate(l.monthly_rate)} ·{' '}
                        {formatDate(l.start_date)}
                      </p>
                    </div>
                    <span
                      className={cn(
                        'text-xs font-medium capitalize',
                        statusColor[l.effective_status],
                      )}
                    >
                      {formatLoanStatus(l.effective_status)}
                    </span>
                  </div>
                  <div className="mt-2 flex gap-4 text-sm">
                    <span>
                      <span className="text-muted-foreground">
                        Outstanding:{' '}
                      </span>
                      {formatCurrency(l.outstanding_principal)}
                    </span>
                    <span>
                      <span className="text-muted-foreground">Interest: </span>
                      {formatCurrency(l.interest_collected)}
                    </span>
                  </div>
                </CardContent>
              </Card>
            )}
          />
        </div>
      )}

      <LoanFormDialog
        open={dialogOpen}
        onOpenChange={setDialogOpen}
        onSaved={() => void reload()}
      />
    </div>
  )
}
