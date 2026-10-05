import { useEffect, useState } from 'react'
import { useForm, Controller } from 'react-hook-form'
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
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { Combobox } from '@/components/ui/combobox'
import { borrowers as borrowersApi, loans } from '@/lib/api'
import type { Borrower } from '@/types'

const schema = z.object({
  borrower_id: z.string().min(1, 'Select a borrower'),
  principal: z.coerce.number().positive('Amount must be greater than 0'),
  monthly_rate_pct: z.coerce
    .number()
    .min(0, 'Rate cannot be negative')
    .max(100, 'Rate too high'),
  repayment_mode: z.enum(['one_time', 'installments']),
  end_date: z.string().optional(),
  start_date: z.string().min(1, 'Start date is required'),
  note: z.string().optional(),
})

type FormValues = z.input<typeof schema>

const today = () => new Date().toISOString().slice(0, 10)

export function LoanFormDialog({
  open,
  onOpenChange,
  onSaved,
}: {
  open: boolean
  onOpenChange: (open: boolean) => void
  onSaved: () => void
}) {
  const [borrowerList, setBorrowerList] = useState<Borrower[]>([])
  const {
    register,
    handleSubmit,
    control,
    reset,
    watch,
    formState: { errors, isSubmitting },
  } = useForm<FormValues>({
    resolver: zodResolver(schema),
    defaultValues: {
      borrower_id: '',
      principal: 0,
      monthly_rate_pct: 0,
      repayment_mode: 'one_time',
      end_date: '',
      start_date: today(),
      note: '',
    },
  })

  const mode = watch('repayment_mode')

  useEffect(() => {
    if (open) {
      reset({
        borrower_id: '',
        principal: 0,
        monthly_rate_pct: 0,
        repayment_mode: 'one_time',
        end_date: '',
        start_date: today(),
        note: '',
      })
      borrowersApi
        .list()
        .then(setBorrowerList)
        .catch((e) => toast.error(String(e)))
    }
  }, [open, reset])

  const onSubmit = async (values: FormValues) => {
    const endDate = values.end_date?.trim() || ''
    if (endDate && endDate <= values.start_date) {
      toast.error('End date must be after the start date')
      return
    }
    if (values.repayment_mode === 'installments' && !endDate) {
      toast.error('Installment loans need an end date')
      return
    }
    try {
      await loans.create({
        borrower_id: values.borrower_id,
        principal: Number(values.principal),
        monthly_rate: Number(values.monthly_rate_pct) / 100,
        interest_type: 'simple', // v1: simple only
        repayment_mode: values.repayment_mode,
        end_date: endDate || null,
        start_date: values.start_date,
        note: values.note || null,
      })
      toast.success('Loan created')
      onOpenChange(false)
      onSaved()
    } catch (e) {
      toast.error(String(e))
    }
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-h-[90vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle>New loan</DialogTitle>
        </DialogHeader>
        <form onSubmit={handleSubmit(onSubmit)} className="grid gap-4">
          <div className="grid gap-2">
            <Label>Borrower</Label>
            <Controller
              control={control}
              name="borrower_id"
              render={({ field }) => (
                <Combobox
                  options={borrowerList.map((b) => ({
                    value: b.id,
                    label: b.name,
                  }))}
                  value={field.value}
                  onChange={field.onChange}
                  placeholder="Select borrower"
                  searchPlaceholder="Search borrowers…"
                  emptyText="No borrowers found"
                />
              )}
            />
            {errors.borrower_id && (
              <p className="text-xs text-destructive">
                {errors.borrower_id.message}
              </p>
            )}
          </div>

          <div className="grid grid-cols-2 gap-3">
            <div className="grid gap-2">
              <Label htmlFor="principal">Amount (₹)</Label>
              <Input
                id="principal"
                type="number"
                step="0.01"
                {...register('principal')}
              />
              {errors.principal && (
                <p className="text-xs text-destructive">
                  {errors.principal.message}
                </p>
              )}
            </div>
            <div className="grid gap-2">
              <Label htmlFor="rate">Rate (%/month)</Label>
              <Input
                id="rate"
                type="number"
                step="0.01"
                {...register('monthly_rate_pct')}
              />
            </div>
          </div>

          <div className="grid grid-cols-2 gap-3">
            <div className="grid gap-2">
              <Label>Repayment</Label>
              <Controller
                control={control}
                name="repayment_mode"
                render={({ field }) => (
                  <Select value={field.value} onValueChange={field.onChange}>
                    <SelectTrigger>
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="one_time">One-time</SelectItem>
                      <SelectItem value="installments">Installments</SelectItem>
                    </SelectContent>
                  </Select>
                )}
              />
            </div>
            <div className="grid gap-2">
              <Label htmlFor="start_date">Start date</Label>
              <Input id="start_date" type="date" {...register('start_date')} />
            </div>
          </div>

          <div className="grid gap-2">
            <Label htmlFor="end_date">
              End date{mode === 'one_time' ? ', optional' : ''}
            </Label>
            <Input id="end_date" type="date" {...register('end_date')} />
          </div>

          <div className="grid gap-2">
            <Label htmlFor="note">Note</Label>
            <Textarea id="note" {...register('note')} />
          </div>

          <p className="text-xs text-muted-foreground">
            Interest is calculated per whole month, added on the loan's monthly
            anniversary date. v1 supports simple interest only.
          </p>

          <DialogFooter>
            <Button
              type="button"
              variant="outline"
              onClick={() => onOpenChange(false)}
            >
              Cancel
            </Button>
            <Button type="submit" disabled={isSubmitting}>
              Create
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}
