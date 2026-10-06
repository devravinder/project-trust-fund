import {
  LayoutDashboard,
  Users,
  HandCoins,
  Receipt,
  BarChart3,
  Settings,
} from 'lucide-react'
import { NavLink, Outlet } from 'react-router-dom'
import { cn } from '@/lib/utils'

const navItems = [
  { to: '/', label: 'Dashboard', icon: LayoutDashboard, end: true },
  { to: '/borrowers', label: 'Borrowers', icon: Users },
  { to: '/loans', label: 'Loans', icon: HandCoins },
  { to: '/payments', label: 'Payments', icon: Receipt },
  { to: '/reports', label: 'Reports', icon: BarChart3 },
  { to: '/settings', label: 'Settings', icon: Settings },
]

function NavItems({ orientation }: { orientation: 'side' | 'bottom' }) {
  const isBottom = orientation === 'bottom'
  return (
    <>
      {navItems.map(({ to, label, icon: Icon, end }) => (
        <NavLink
          key={to}
          to={to}
          end={end}
          aria-label={label}
          title={label}
          className={({ isActive }) =>
            cn(
              'flex items-center gap-3 rounded-md px-3 py-2 text-sm font-medium transition-colors',
              isBottom && 'justify-center p-2',
              isActive
                ? 'bg-primary text-primary-foreground'
                : 'text-muted-foreground hover:bg-accent hover:text-accent-foreground',
            )
          }
        >
          <Icon className={isBottom ? 'size-6' : 'size-4'} />
          {!isBottom && <span>{label}</span>}
        </NavLink>
      ))}
    </>
  )
}

export function AppLayout() {
  return (
    <div className="flex h-screen overflow-hidden bg-background text-foreground">
      {/* Sidebar (desktop) — scrolls independently */}
      <aside className="hidden w-56 shrink-0 flex-col overflow-y-auto border-r p-4 md:flex">
        <div className="mb-6 flex items-center gap-3 px-1">
          <img src="/logo.svg?v=8" alt="TrustFund" className="size-12" />
          <div>
            <h1 className="text-lg font-bold leading-tight">TrustFund</h1>
            <p className="text-xs text-muted-foreground">Lend &amp; recover</p>
          </div>
        </div>
        <nav className="flex flex-col gap-1">
          <NavItems orientation="side" />
        </nav>
      </aside>

      {/* Main content column */}
      <div className="flex min-w-0 flex-1 flex-col overflow-hidden">
        {/* Compact logo header (mobile only) */}
        <header className="flex shrink-0 items-center gap-2 border-b p-3 md:hidden">
          <img src="/logo.svg?v=8" alt="TrustFund" className="size-8" />
          <span className="text-base font-bold">TrustFund</span>
        </header>

        {/* Scrollable content area (independent of the sidebar) */}
        <main className="flex-1 overflow-y-auto p-4 pb-20 md:pb-4">
          <Outlet />
        </main>

        {/* Bottom nav (mobile) */}
        <nav className="fixed inset-x-0 bottom-0 flex justify-around border-t bg-background p-2 md:hidden">
          <NavItems orientation="bottom" />
        </nav>
      </div>
    </div>
  )
}
