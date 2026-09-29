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
  return (
    <>
      {navItems.map(({ to, label, icon: Icon, end }) => (
        <NavLink
          key={to}
          to={to}
          end={end}
          className={({ isActive }) =>
            cn(
              'flex items-center gap-3 rounded-md px-3 py-2 text-sm font-medium transition-colors',
              orientation === 'bottom' && 'flex-col gap-1 px-2 py-1 text-xs',
              isActive
                ? 'bg-primary text-primary-foreground'
                : 'text-muted-foreground hover:bg-accent hover:text-accent-foreground',
            )
          }
        >
          <Icon className={orientation === 'bottom' ? 'size-5' : 'size-4'} />
          <span>{label}</span>
        </NavLink>
      ))}
    </>
  )
}

export function AppLayout() {
  return (
    <div className="flex min-h-screen bg-background text-foreground">
      {/* Sidebar (desktop) */}
      <aside className="hidden w-56 shrink-0 flex-col border-r p-4 md:flex">
        <div className="mb-6 px-3">
          <h1 className="text-lg font-bold">TrustFund</h1>
          <p className="text-xs text-muted-foreground">Lend &amp; recover</p>
        </div>
        <nav className="flex flex-col gap-1">
          <NavItems orientation="side" />
        </nav>
      </aside>

      {/* Main content */}
      <div className="flex flex-1 flex-col">
        <main className="flex-1 p-4 pb-20 md:pb-4">
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
