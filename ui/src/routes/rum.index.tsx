import { createFileRoute } from "@tanstack/react-router"

import { loadRum, loadRumTrace, loadRumVital, RumPage, validateRumSearch } from "@/features/rum"
import { resolveRangeSearch } from "@/domain/time-range/range"
import { whereClauseFromSearch } from "@/shared/where-clause"

export const Route = createFileRoute("/rum/")({
  validateSearch: validateRumSearch,
  loaderDeps: ({ search }) => search,
  loader: async ({ deps }) => {
    const range = resolveRangeSearch(deps)
    const data = await loadRum(deps, range, whereClauseFromSearch(deps.where))
    const service = deps.service ?? data.service ?? undefined
    const [vital, trace] = await Promise.all([
      loadRumVital(service ? { ...deps, service } : deps, range),
      deps.traceId ? loadRumTrace(deps.traceId) : Promise.resolve(null),
    ])
    return { data, vital, trace }
  },
  component: RumRoute,
})

function RumRoute() {
  const { data, vital, trace } = Route.useLoaderData()
  const search = Route.useSearch()
  return <RumPage data={data} vital={vital} trace={trace} search={search} />
}
