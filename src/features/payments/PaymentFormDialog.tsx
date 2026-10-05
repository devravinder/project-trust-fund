import { useEffect, useMemo, useState } from 'react'
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
import { Combobox } from '@/components/ui/combobox'
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
type PayMode = 'full' | 'partial'

const today = () => new Date().toISOString().slice(0, 10)

function totalDue(l: LoanSummary): number {
  return Math.round((l.outstanding_principal + l.interest_due) * 100) / 100
}

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
  const [payMode, setPayMode] = useState<PayMode>('full') // Full is the default
  const {
    register,
    handleSubmit,
    control,
    reset,
    watch,
    setValue,
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

  const selectedId = watch('loan_id')
  const selectedLoan = useMemo(
    () => loanList.find((l) => l.id === selectedId) ?? null,
    [loanList, selectedId],
  )

  useEffect(() => {
    if (!open) return
    setPayMode('full')
    reset({
      loan_id: fixedLoanId ?? '',
      amount: 0,
      paid_date: today(),
      note: '',
    })
    // Load the loan(s) to show balance details. For a fixed loan, fetch just it.
    if (fixedLoanId) {
      loansApi
        .summary(fixedLoanId)
        .then((l) => setLoanList(l ? [l] : []))
        .catch((e) => toast.error(String(e)))
    } else {
      loansApi
        .list('all')
        .then(setLoanList)
        .catch((e) => toast.error(String(e)))
    }
  }, [open, reset, fixedLoanId])

  // In Full mode, keep the amount synced to the loan's total due.
  useEffect(() => {
    if (payMode === 'full' && selectedLoan) {
      setValue('amount', totalDue(selectedLoan))
    }
  }, [payMode, selectedLoan, setValue])

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
                  <Combobox
                    options={loanList.map((l) => ({
                      value: l.id,
                      label: `${l.borrower_name} · ${formatCurrency(
                        l.principal,
                      )} (${formatCurrency(l.outstanding_principal)} left)`,
                    }))}
                    value={field.value}
                    onChange={field.onChange}
                    placeholder="Select loan"
                    searchPlaceholder="Search by borrower…"
                    emptyText="No loans found"
                  />
                )}
              />
              {errors.loan_id && (
                <p className="text-xs text-destructive">
                  {errors.loan_id.message}
                </p>
              )}
            </div>
          )}

          {/* Loan balance details */}
          {selectedLoan && (
            <div className="rounded-md border bg-muted/40 p-3 text-sm">
              <div className="flex justify-between">
                <span className="text-muted-foreground">
                  Outstanding principal
                </span>
                <span>{formatCurrency(selectedLoan.outstanding_principal)}</span>
              </div>
              <div className="flex justify-between">
                <span className="text-muted-foreground">Interest due</span>
                <span>{formatCurrency(selectedLoan.interest_due)}</span>
              </div>
              <div className="mt-1 flex justify-between border-t pt-1 font-medium">
                <span>Total due</span>
                <span>{formatCurrency(totalDue(selectedLoan))}</span>
              </div>
            </div>
          )}

          {/* Full / Partial */}
          <div className="grid gap-2">
            <Label>Payment</Label>
            <div className="flex gap-2">
              <Button
                type="button"
                size="sm"
                variant={payMode === 'full' ? 'default' : 'outline'}
                onClick={() => setPayMode('full')}
              >
                Full
              </Button>
              <Button
                type="button"
                size="sm"
                variant={payMode === 'partial' ? 'default' : 'outline'}
                onClick={() => setPayMode('partial')}
              >
                Partial
              </Button>
            </div>
          </div>

          <div className="grid gap-2">
            <Label htmlFor="amount">Amount (₹)</Label>
            <Input
              id="amount"
              type="number"
              step="0.01"
              readOnly={payMode === 'full'}
              className={payMode === 'full' ? 'bg-muted' : undefined}
              {...register('amount')}
            />
            {payMode === 'full' && (
              <p className="text-xs text-muted-foreground">
                Full settlement — switch to Partial to enter a custom amount.
              </p>
            )}
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
