import { sortRows } from "@/shared/console/data-table"

export interface ServiceSummary {
  readonly name: string
  readonly lastSeenNanos: string
  readonly spanCount: string
  readonly errorCount: string
  readonly p95Ms: number | null
}

export interface ServiceCatalogRow {
  readonly name: string
  readonly serviceVersion: string | null
  readonly serviceNamespace: string | null
  readonly deploymentEnvironment: string | null
  readonly telemetrySdkLanguage: string | null
  readonly telemetrySdkName: string | null
  readonly telemetrySdkVersion: string | null
  readonly lastSeenNanos: string
  readonly instanceCount: string
}

export interface ServicesData {
  readonly serviceList: readonly ServiceSummary[]
  readonly serviceCatalog: readonly ServiceCatalogRow[]
}

export type ServiceTableRow = ServiceSummary &
  Partial<Omit<ServiceCatalogRow, "name" | "lastSeenNanos">> & {
    readonly catalogLastSeenNanos: string | undefined
  }

export function serviceErrorRate(row: ServiceSummary): number {
  const spans = Number(row.spanCount)
  if (!Number.isFinite(spans) || spans <= 0) return 0
  return Number(row.errorCount) / spans
}

export function serviceHref(service: string): string {
  return `/services/${encodeURIComponent(service)}`
}

export function servicesWithCatalog(data: ServicesData): ServiceTableRow[] {
  const summaries = new Map(data.serviceList.map((row) => [row.name, row]))
  const catalog = new Map(data.serviceCatalog.map((row) => [row.name, row]))
  const names = new Set([...summaries.keys(), ...catalog.keys()])
  return Array.from(names).map((name) => {
    const summary = summaries.get(name)
    const identity = catalog.get(name)
    return {
      name,
      lastSeenNanos: summary?.lastSeenNanos ?? identity?.lastSeenNanos ?? "0",
      spanCount: summary?.spanCount ?? "0",
      errorCount: summary?.errorCount ?? "0",
      p95Ms: summary?.p95Ms ?? null,
      serviceVersion: identity?.serviceVersion ?? null,
      serviceNamespace: identity?.serviceNamespace ?? null,
      deploymentEnvironment: identity?.deploymentEnvironment ?? null,
      telemetrySdkLanguage: identity?.telemetrySdkLanguage ?? null,
      telemetrySdkName: identity?.telemetrySdkName ?? null,
      telemetrySdkVersion: identity?.telemetrySdkVersion ?? null,
      instanceCount: identity?.instanceCount ?? "0",
      catalogLastSeenNanos: identity?.lastSeenNanos,
    }
  })
}

export function sortedServices(
  rows: ServiceTableRow[],
  sort?: string
): ServiceTableRow[] {
  return sortRows(rows, sort ?? "lastSeen:desc", {
    name: (row) => row.name.toLowerCase(),
    version: (row) => row.serviceVersion?.toLowerCase(),
    runtime: (row) => row.telemetrySdkLanguage?.toLowerCase(),
    env: (row) => row.deploymentEnvironment?.toLowerCase(),
    spans: (row) => Number(row.spanCount),
    errors: (row) => Number(row.errorCount),
    errorRate: serviceErrorRate,
    p95: (row) => row.p95Ms,
    lastSeen: (row) => Number(row.lastSeenNanos),
  })
}
