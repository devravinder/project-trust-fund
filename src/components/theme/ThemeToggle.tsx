import { Moon, Sun, Monitor } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { useTheme } from '@/components/theme/ThemeProvider'

export function ThemeToggle() {
  const { theme, setTheme } = useTheme()

  const next =
    theme === 'light' ? 'dark' : theme === 'dark' ? 'system' : 'light'
  const Icon = theme === 'light' ? Sun : theme === 'dark' ? Moon : Monitor
  const label =
    theme === 'light' ? 'Light' : theme === 'dark' ? 'Dark' : 'System'

  return (
    <Button
      variant="ghost"
      size="sm"
      onClick={() => setTheme(next)}
      aria-label={`Theme: ${label}. Click to switch.`}
      title={`Theme: ${label}`}
    >
      <Icon className="size-4" />
      <span>{label}</span>
    </Button>
  )
}
