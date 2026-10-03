import {
  createContext,
  useContext,
  useEffect,
  useState,
  type ReactNode,
} from 'react'

type Theme = 'light' | 'dark' | 'system'

interface ThemeContextValue {
  theme: Theme
  setTheme: (t: Theme) => void
  resolved: 'light' | 'dark'
}

const ThemeContext = createContext<ThemeContextValue | null>(null)

const STORAGE_KEY = 'trustfund-theme'

function systemPrefersDark() {
  return window.matchMedia('(prefers-color-scheme: dark)').matches
}

function applyTheme(theme: Theme): 'light' | 'dark' {
  const resolved: 'light' | 'dark' =
    theme === 'system' ? (systemPrefersDark() ? 'dark' : 'light') : theme
  const root = document.documentElement
  root.classList.toggle('dark', resolved === 'dark')
  return resolved
}

export function ThemeProvider({ children }: { children: ReactNode }) {
  const [theme, setThemeState] = useState<Theme>(
    () => (localStorage.getItem(STORAGE_KEY) as Theme) || 'system',
  )
  const [resolved, setResolved] = useState<'light' | 'dark'>(() =>
    applyTheme((localStorage.getItem(STORAGE_KEY) as Theme) || 'system'),
  )

  const setTheme = (t: Theme) => {
    localStorage.setItem(STORAGE_KEY, t)
    setThemeState(t)
    setResolved(applyTheme(t))
  }

  // React to OS theme changes while in "system" mode.
  useEffect(() => {
    if (theme !== 'system') return
    const mq = window.matchMedia('(prefers-color-scheme: dark)')
    const onChange = () => setResolved(applyTheme('system'))
    mq.addEventListener('change', onChange)
    return () => mq.removeEventListener('change', onChange)
  }, [theme])

  return (
    <ThemeContext.Provider value={{ theme, setTheme, resolved }}>
      {children}
    </ThemeContext.Provider>
  )
}

export function useTheme() {
  const ctx = useContext(ThemeContext)
  if (!ctx) throw new Error('useTheme must be used within ThemeProvider')
  return ctx
}
