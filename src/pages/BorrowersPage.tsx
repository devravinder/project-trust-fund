import { useCallback, useEffect, useState } from 'react'
import { useNavigate } from 'react-router-dom'
import { Plus, Pencil, Trash2, Search } from 'lucide-react'
import { toast } from 'sonner'
import { PageHeader } from '@/components/layout/PageHeader'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Card, CardContent } from '@/components/ui/card'
import { borrowers } from '@/lib/api'
import type { Borrower } from '@/types'
import { BorrowerFormDialog } from '@/features/borrowers/BorrowerFormDialog'

export function BorrowersPage() {
  const [items, setItems] = useState<Borrower[]>([])
  const [search, setSearch] = useState('')
  const [loading, setLoading] = useState(true)
  const [dialogOpen, setDialogOpen] = useState(false)
  const [editing, setEditing] = useState<Borrower | null>(null)
  const navigate = useNavigate()

  const load = useCallback(async (term: string) => {
    setLoading(true)
    try {
      setItems(await borrowers.list(term || undefined))
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

  const handleAdd = () => {
    setEditing(null)
    setDialogOpen(true)
  }

  const handleEdit = (b: Borrower) => {
    setEditing(b)
    setDialogOpen(true)
  }

  const handleDelete = async (b: Borrower) => {
    if (!confirm(`Delete ${b.name}? This hides them but keeps history.`)) return
    try {
      await borrowers.remove(b.id)
      toast.success('Borrower deleted')
      void load(search)
    } catch (e) {
      toast.error(String(e))
    }
  }

  return (
    <div>
      <PageHeader
        title="Borrowers"
        description="People you lend to"
        actions={
          <Button onClick={handleAdd}>
            <Plus /> Add
          </Button>
        }
      />

      <div className="relative mb-4">
        <Search className="absolute left-3 top-1/2 size-4 -translate-y-1/2 text-muted-foreground" />
        <Input
          placeholder="Search by name or phone"
          className="pl-9"
          value={search}
          onChange={(e) => setSearch(e.target.value)}
        />
      </div>

      {loading ? (
        <p className="text-muted-foreground">Loading…</p>
      ) : items.length === 0 ? (
        <p className="text-muted-foreground">
          No borrowers yet. Add your first one.
        </p>
      ) : (
        <div className="grid gap-2">
          {items.map((b) => (
            <Card key={b.id}>
              <CardContent className="flex items-center justify-between p-4">
                <button
                  type="button"
                  className="flex-1 text-left"
                  onClick={() => navigate(`/borrowers/${b.id}`)}
                >
                  <p className="font-medium">{b.name}</p>
                  {b.phone && (
                    <p className="text-sm text-muted-foreground">{b.phone}</p>
                  )}
                </button>
                <div className="flex gap-1">
                  <Button
                    variant="ghost"
                    size="icon"
                    onClick={() => handleEdit(b)}
                    aria-label={`Edit ${b.name}`}
                  >
                    <Pencil />
                  </Button>
                  <Button
                    variant="ghost"
                    size="icon"
                    onClick={() => handleDelete(b)}
                    aria-label={`Delete ${b.name}`}
                  >
                    <Trash2 />
                  </Button>
                </div>
              </CardContent>
            </Card>
          ))}
        </div>
      )}

      <BorrowerFormDialog
        open={dialogOpen}
        onOpenChange={setDialogOpen}
        borrower={editing}
        onSaved={() => void load(search)}
      />
    </div>
  )
}
