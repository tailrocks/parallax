import { QueryClientProvider } from "@tanstack/react-query"
import type { QueryClient } from "@tanstack/react-query"
import type { ReactNode } from "react"

export function AppQueryProvider({
  client,
  children,
}: {
  client: QueryClient
  children: ReactNode
}) {
  return <QueryClientProvider client={client}>{children}</QueryClientProvider>
}
