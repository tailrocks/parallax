export type Widget = {
  metric: string
  agg: string
  chart: string
  title: string
  groupBy?: string | undefined
  filterValue?: string | undefined
  w?: number
  [key: string]: unknown
}

export const AGGS = ["avg", "sum", "min", "max", "rate"] as const
export const CHARTS = ["line", "area", "bar"] as const

export function emptyWidget(): Widget {
  return { metric: "", agg: "avg", chart: "line", title: "", w: 1 }
}

export function parseLayout(layout: string): Widget[] {
  try {
    const parsed: unknown = JSON.parse(layout)
    return Array.isArray(parsed)
      ? parsed.filter(
          (item): item is Widget =>
            typeof item === "object" && item !== null && typeof (item as Widget).metric === "string"
        )
      : []
  } catch {
    return []
  }
}

export function serializeWidgets(widgets: Widget[]): string {
  return JSON.stringify(widgets)
}
