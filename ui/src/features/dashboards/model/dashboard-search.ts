import { rangeLinkSearch, resolveRangeSearch } from "@/lib/range"

export type DashboardSearch = {
  range?: string | undefined
  from?: string | undefined
  to?: string | undefined
  widget_metric?: string | undefined
  widget_agg?: string | undefined
  widget_group_by?: string | undefined
}

function searchString(value: unknown): string | undefined {
  if (typeof value === "string") return value
  if (typeof value === "number" && Number.isFinite(value)) return String(value)
  return undefined
}

export function validateDashboardSearch(
  search: Record<string, unknown>
): DashboardSearch {
  return {
    range: searchString(search["range"]),
    from: searchString(search["from"]),
    to: searchString(search["to"]),
    widget_metric: searchString(search["widget_metric"]),
    widget_agg: searchString(search["widget_agg"]),
    widget_group_by: searchString(search["widget_group_by"]),
  }
}

export function dashboardRangeSearch(search: DashboardSearch) {
  return rangeLinkSearch(resolveRangeSearch(search))
}
