// Formatting helpers. Currency fixed to INR with Indian digit grouping.

const inr = new Intl.NumberFormat('en-IN', {
  style: 'currency',
  currency: 'INR',
  maximumFractionDigits: 2,
})

export function formatCurrency(value: number): string {
  return inr.format(value ?? 0)
}

export function formatDate(iso: string): string {
  if (!iso) return ''
  const d = new Date(iso)
  if (Number.isNaN(d.getTime())) return iso
  return d.toLocaleDateString('en-IN', {
    day: '2-digit',
    month: 'short',
    year: 'numeric',
  })
}

/** Format a monthly rate stored as a fraction (0.02) into a percentage string. */
export function formatRate(rate: number): string {
  return `${(rate * 100).toFixed(2)}%/mo`
}
