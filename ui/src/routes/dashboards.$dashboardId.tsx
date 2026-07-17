import { createFileRoute, notFound } from "@tanstack/react-router"

import {
  DashboardDetailPage,
  loadDashboardDetail,
  loadWidgetSeries,
  parseLayout,
  toWidgetData,
} from "@/features/dashboards"
import { resolveRangeSearch, rangeSearchSchema } from "@/domain/time-range/range"

export const Route = createFileRoute("/dashboards/$dashboardId")({
  validateSearch: (search: Record<string, unknown>) => rangeSearchSchema.parse(search),
  loaderDeps: ({ search }) => search,
  loader: async ({ params, deps }) => {
    const range = resolveRangeSearch(deps)
    const { dashboard, metricNames } = await loadDashboardDetail(params.dashboardId)
    if (!dashboard) throw notFound()
    const widgets = parseLayout(dashboard.layout)
    const seriesList = await loadWidgetSeries(widgets, range)
    const data = widgets.map((widget, index) =>
      toWidgetData(widget, seriesList[index] ?? [], range)
    )
    return {
      id: dashboard.id,
      name: dashboard.name,
      widgets,
      data,
      metricNames,
      range,
    }
  },
  component: DashboardDetailRoute,
})

function DashboardDetailRoute() {
  const loaded = Route.useLoaderData()
  return <DashboardDetailPage {...loaded} />
}
