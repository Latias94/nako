'use client'

import * as React from 'react'

type Theme = 'dark' | 'light' | 'system'

interface ThemeProviderProps {
  children: React.ReactNode
  defaultTheme?: Theme
  storageKey?: string
  attribute?: 'class'
}

interface ThemeContextValue {
  theme: Theme
  setTheme: (theme: Theme) => void
}

const ThemeContext = React.createContext<ThemeContextValue>({
  theme: 'dark',
  setTheme: () => {},
})

function resolveTheme(theme: Theme) {
  if (theme !== 'system') {
    return theme
  }

  return window.matchMedia('(prefers-color-scheme: light)').matches ? 'light' : 'dark'
}

function applyTheme(theme: Theme) {
  const resolvedTheme = resolveTheme(theme)
  document.documentElement.classList.toggle('light', resolvedTheme === 'light')
  document.documentElement.classList.toggle('dark', resolvedTheme === 'dark')
}

export function ThemeProvider({
  children,
  defaultTheme = 'dark',
  storageKey = 'nako.theme',
}: ThemeProviderProps) {
  const [theme, setThemeState] = React.useState<Theme>(() => {
    if (typeof window === 'undefined') {
      return defaultTheme
    }

    return (window.localStorage.getItem(storageKey) as Theme | null) ?? defaultTheme
  })

  React.useEffect(() => {
    applyTheme(theme)
    window.localStorage.setItem(storageKey, theme)
  }, [storageKey, theme])

  const value = React.useMemo<ThemeContextValue>(
    () => ({
      theme,
      setTheme: setThemeState,
    }),
    [theme],
  )

  return <ThemeContext.Provider value={value}>{children}</ThemeContext.Provider>
}

export function useTheme() {
  return React.useContext(ThemeContext)
}
