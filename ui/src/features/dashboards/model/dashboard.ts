export type Dashboard = {
  readonly id: string
  readonly name: string
  readonly layout: string
  readonly updatedAtNanos: string
}

export type DashboardSummary = {
  readonly id: string
  readonly name: string
  readonly layout: string
}

export function mapDashboard(raw: {
  readonly id: string
  readonly name: string
  readonly layout: string
  readonly updatedAtNanos: string
}): Dashboard {
  return {
    id: raw.id,
    name: raw.name,
    layout: raw.layout,
    updatedAtNanos: raw.updatedAtNanos,
  }
}
