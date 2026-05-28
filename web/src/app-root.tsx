import { NakoRouter } from "@/components/nako/nako-router"
import { ThemeProvider } from "@/components/theme-provider"
import { QueryProvider } from "@/lib/query-provider"

export function AppRoot() {
  return (
    <ThemeProvider defaultTheme="dark">
      <QueryProvider>
        <NakoRouter />
      </QueryProvider>
    </ThemeProvider>
  )
}
