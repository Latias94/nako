import { NakoRouter } from "@/components/nako/nako-router"
import { QueryProvider } from "@/lib/query-provider"

export function AppRoot() {
  return (
    <QueryProvider>
      <NakoRouter />
    </QueryProvider>
  )
}
