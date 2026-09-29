import { useEffect } from 'react'
import { useForm } from 'react-hook-form'
import { zodResolver } from '@hookform/resolvers/zod'
import { z } from 'zod'
import { toast } from 'sonner'
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogFooter,
} from '@/components/ui/dialog'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Textarea } from '@/components/ui/textarea'
import { borrowers } from '@/lib/api'
import type { Borrower } from '@/types'

const schema = z.object({
  name: z.string().min(1, 'Name is required'),
  phone: z.string().optional(),
  address: z.string().optional(),
  notes: z.string().optional(),
})

type FormValues = z.infer<typeof schema>

export function BorrowerFormDialog({
  open,
  onOpenChange,
  borrower,
  onSaved,
}: {
  open: boolean
  onOpenChange: (open: boolean) => void
  borrower?: Borrower | null
  onSaved: () => void
}) {
  const {
    register,
    handleSubmit,
    reset,
    formState: { errors, isSubmitting },
  } = useForm<FormValues>({
    resolver: zodResolver(schema),
    defaultValues: { name: '', phone: '', address: '', notes: '' },
  })

  useEffect(() => {
    if (open) {
      reset({
        name: borrower?.name ?? '',
        phone: borrower?.phone ?? '',
        address: borrower?.address ?? '',
        notes: borrower?.notes ?? '',
      })
    }
  }, [open, borrower, reset])

  const onSubmit = async (values: FormValues) => {
    try {
      const input = {
        name: values.name,
        phone: values.phone || null,
        address: values.address || null,
        notes: values.notes || null,
      }
      if (borrower) {
        await borrowers.update(borrower.id, input)
        toast.success('Borrower updated')
      } else {
        await borrowers.create(input)
        toast.success('Borrower added')
      }
      onOpenChange(false)
      onSaved()
    } catch (e) {
      toast.error(String(e))
    }
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>
            {borrower ? 'Edit borrower' : 'Add borrower'}
          </DialogTitle>
        </DialogHeader>
        <form onSubmit={handleSubmit(onSubmit)} className="grid gap-4">
          <div className="grid gap-2">
            <Label htmlFor="name">Name</Label>
            <Input id="name" {...register('name')} />
            {errors.name && (
              <p className="text-xs text-destructive">{errors.name.message}</p>
            )}
          </div>
          <div className="grid gap-2">
            <Label htmlFor="phone">Phone</Label>
            <Input id="phone" {...register('phone')} />
          </div>
          <div className="grid gap-2">
            <Label htmlFor="address">Address</Label>
            <Input id="address" {...register('address')} />
          </div>
          <div className="grid gap-2">
            <Label htmlFor="notes">Notes</Label>
            <Textarea id="notes" {...register('notes')} />
          </div>
          <DialogFooter>
            <Button
              type="button"
              variant="outline"
              onClick={() => onOpenChange(false)}
            >
              Cancel
            </Button>
            <Button type="submit" disabled={isSubmitting}>
              {borrower ? 'Save' : 'Add'}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}
