import { createFileRoute } from "@tanstack/react-router"

import { SqlPage } from "@/features/sql"

interface SqlSearch {
  query?: string | undefined
}

export const Route = createFileRoute("/sql")({
  validateSearch: (search: Record<string, unknown>): SqlSearch => ({
    query: typeof search["query"] === "string" ? search["query"] : undefined,
  }),
  component: SqlRoute,
})

function SqlRoute() {
  const search = Route.useSearch()
  return <SqlPage searchQuery={search.query} />
}
