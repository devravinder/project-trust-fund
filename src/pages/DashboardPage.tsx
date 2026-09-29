import { PageHeader } from '@/components/layout/PageHeader'

export function DashboardPage() {
  return (
    <div>
      <PageHeader
        title="Dashboard"
        description="Overview of your lending activity"
      />
      <p className="text-muted-foreground">Summary cards coming soon.</p>
    </div>
  )
}
