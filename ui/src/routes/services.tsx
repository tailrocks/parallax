import { createFileRoute } from "@tanstack/react-router"

import { ServicesRouteShell, loadServices, validateServicesSearch } from "@/features/services"
import { resolveRangeSearch } from "@/domain/time-range/range"

export const Route = createFileRoute("/services")({
  validateSearch: validateServicesSearch,
  loaderDeps: ({ search }) => search,
  loader: ({ deps }) => loadServices(resolveRangeSearch(deps)),
  component: ServicesRoute,
})

function ServicesRoute() {
  const data = Route.useLoaderData()
  const search = Route.useSearch()
  return <ServicesRouteShell data={data} search={search} />
}
