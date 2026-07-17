import { createFileRoute } from "@tanstack/react-router"

import {
  EcosystemPage,
  loadServiceMap,
  validateEcosystemSearch,
} from "@/features/ecosystem"
import { resolveRangeSearch } from "@/lib/range"

export const Route = createFileRoute("/ecosystem")({
  validateSearch: validateEcosystemSearch,
  loaderDeps: ({ search }) => search,
  loader: ({ deps }) => {
    const range = resolveRangeSearch(deps)
    return loadServiceMap({
      fromNanos: range.fromNanos,
      toNanos: range.toNanos,
      maxTraces: 100,
    }).then((serviceMap) => ({ serviceMap }))
  },
  component: EcosystemRoute,
})

function EcosystemRoute() {
  const { serviceMap } = Route.useLoaderData()
  const search = Route.useSearch()
  return <EcosystemPage serviceMap={serviceMap} search={search} />
}
