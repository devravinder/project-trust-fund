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
import { loans as loansApi, payments } from '@/lib/api'
import type { LoanSummary } from '@/lib/api'
import { formatCurrency } from '@/lib/format'

const schema = z.object({
  loan_id: z.string().min(1, 'Select a loan'),
  amount: z.coerce.number().positive('Amount must be greater than 0'),
  paid_date: z.string().min(1, 'Date is required'),
  note: z.string().optional(),
})

type FormValues = z.input<typeof schema>

const today = () => new Date().toISOString().slice(0, 10)

export function PaymentFormDialog({
  open,
  onOpenChange,
  onSaved,
  fixedLoanId,
}: {
  open: boolean
  onOpenChange: (open: boolean) => void
  onSaved: () => void
  fixedLoanId?: string
}) {
  const [loanList, setLoanList] = useState<LoanSummary[]>([])
  const {
    register,
    handleSubmit,
    control,
    reset,
    formState: { errors, isSubmitting },
  } = useForm<FormValues>({
    resolver: zodResolver(schema),
    defaultValues: {
      loan_id: fixedLoanId ?? '',
      amount: 0,
      paid_date: today(),
      note: '',
    },
  })

  useEffect(() => {
    if (open) {
      reset({
        loan_id: fixedLoanId ?? '',
        amount: 0,
        paid_date: today(),
        note: '',
      })
      loansApi
        .list('all')
        .then(setLoanList)
        .catch((e) => toast.error(String(e)))
    }
  }, [open, reset, fixedLoanId])

  const onSubmit = async (values: FormValues) => {
    try {
      await payments.create({
        loan_id: values.loan_id,
        amount: Number(values.amount),
        paid_date: values.paid_date,
        note: values.note || null,
      })
      toast.success('Payment recorded')
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
          <DialogTitle>Record payment</DialogTitle>
        </DialogHeader>
        <form onSubmit={handleSubmit(onSubmit)} className="grid gap-4">
          {!fixedLoanId && (
            <div className="grid gap-2">
              <Label>Loan</Label>
              <Controller
                control={control}
                name="loan_id"
                render={({ field }) => (
                  <Select value={field.value} onValueChange={field.onChange}>
                    <SelectTrigger>
                      <SelectValue placeholder="Select loan" />
                    </SelectTrigger>
                    <SelectContent>
                      {loanList.map((l) => (
                        <SelectItem key={l.id} value={l.id}>
                          {l.borrower_name} · {formatCurrency(l.principal)} (
                          {formatCurrency(l.outstanding_principal)} left)
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                )}
              />
              {errors.loan_id && (
                <p className="text-xs text-destructive">
                  {errors.loan_id.message}
                </p>
              )}
            </div>
          )}

          <div className="grid gap-2">
            <Label htmlFor="amount">Amount (₹)</Label>
            <Input
              id="amount"
              type="number"
              step="0.01"
              {...register('amount')}
            />
            {errors.amount && (
              <p className="text-xs text-destructive">
                {errors.amount.message}
              </p>
            )}
          </div>

          <div className="grid gap-2">
            <Label htmlFor="paid_date">Date</Label>
            <Input id="paid_date" type="date" {...register('paid_date')} />
          </div>

          <div className="grid gap-2">
            <Label htmlFor="note">Note</Label>
            <Textarea id="note" {...register('note')} />
          </div>

          <p className="text-xs text-muted-foreground">
            Payment covers due interest first, then reduces principal.
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
              Save
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}
