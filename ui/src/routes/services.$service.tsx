import { createFileRoute } from "@tanstack/react-router"

import {
  ServiceDetailRoutePage,
  loadServiceDetail,
  validateServicesSearch,
} from "@/features/services"
import { resolveRangeSearch } from "@/domain/time-range/range"

export const Route = createFileRoute("/services/$service")({
  validateSearch: validateServicesSearch,
  loaderDeps: ({ search }) => search,
  loader: ({ params, deps }) => loadServiceDetail(params.service, resolveRangeSearch(deps)),
  component: ServiceDetailRoute,
})

function ServiceDetailRoute() {
  const data = Route.useLoaderData()
  const params = Route.useParams()
  const search = Route.useSearch()
  return <ServiceDetailRoutePage service={params.service} data={data} search={search} />
}
