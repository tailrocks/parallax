// Public facade for dashboards (Plan 137). Named exports only.

export {
  DashboardsPage,
  WidgetPicker,
  DashboardCreateDialog,
  DashboardCards,
} from "@/features/dashboards/components/dashboards-page"
export { DashboardDetailPage } from "@/features/dashboards/components/dashboard-detail-page"
export {
  loadDashboardsList,
  loadDashboardDetail,
  saveDashboard,
  deleteDashboard,
} from "@/features/dashboards/api/dashboard-api"
export { loadWidgetSeries } from "@/features/dashboards/api/widget-series-api"
export { toWidgetData } from "@/features/dashboards/model/widget-data"
export type { WidgetData } from "@/features/dashboards/model/widget-data"
export {
  validateDashboardSearch,
  dashboardRangeSearch,
} from "@/features/dashboards/model/dashboard-search"
export type { DashboardSearch } from "@/features/dashboards/model/dashboard-search"
export type { Dashboard } from "@/features/dashboards/model/dashboard"
export {
  emptyWidget,
  parseLayout,
  serializeWidgets,
  AGGS,
  CHARTS,
} from "@/features/dashboards/model/widget"
export type { Widget } from "@/features/dashboards/model/widget"
