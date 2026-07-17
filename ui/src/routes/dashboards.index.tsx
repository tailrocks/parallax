import { createFileRoute } from "@tanstack/react-router"

import { DashboardsPage, loadDashboardsList, validateDashboardSearch } from "@/features/dashboards"

export const Route = createFileRoute("/dashboards/")({
  validateSearch: validateDashboardSearch,
  loader: () => loadDashboardsList(),
  component: DashboardsRoute,
})

function DashboardsRoute() {
  const data = Route.useLoaderData()
  const search = Route.useSearch()
  return (
    <DashboardsPage dashboards={data.dashboards} metricNames={data.metricNames} search={search} />
  )
}
