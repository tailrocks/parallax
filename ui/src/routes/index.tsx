import { createFileRoute } from "@tanstack/react-router"

import { OverviewPage, loadOverview } from "@/features/overview"
import { rangeSearchSchema, resolveRangeSearch } from "@/lib/range"

export const Route = createFileRoute("/")({
  validateSearch: (search: Record<string, unknown>) =>
    rangeSearchSchema.parse(search),
  loaderDeps: ({ search }) => search,
  loader: ({ deps }) => loadOverview(resolveRangeSearch(deps)),
  component: OverviewRoute,
})

function OverviewRoute() {
  const data = Route.useLoaderData()
  const search = Route.useSearch()
  return <OverviewPage data={data} search={search} />
}
