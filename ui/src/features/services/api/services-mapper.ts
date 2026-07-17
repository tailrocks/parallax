import type { ServiceDetailQuery } from "@/features/services/api/service-detail.generated"
import type { ServicesListQuery } from "@/features/services/api/services-list.generated"
import type {
  MetricExemplar,
  ReleaseWindow,
  SeriesPoint,
  ServiceDetailData,
  ServiceOverview,
  SpanRed,
  TraceSummary,
} from "@/features/services/model/service-detail"
import type {
  ServiceCatalogRow,
  ServiceSummary,
  ServicesData,
} from "@/features/services/model/service-summary"
import type { RuntimeMetric } from "@/domain/runtime-metrics/runtime-metric"

function mapPoint(point: { readonly tsNanos: string; readonly value: number }): SeriesPoint {
  return { tsNanos: point.tsNanos, value: point.value }
}

function mapSeries(
  points: ReadonlyArray<{ readonly tsNanos: string; readonly value: number }>
): SeriesPoint[] {
  return points.map(mapPoint)
}

function mapCatalogRow(row: {
  readonly name: string
  readonly serviceVersion: string | null
  readonly serviceNamespace: string | null
  readonly deploymentEnvironment: string | null
  readonly telemetrySdkLanguage: string | null
  readonly telemetrySdkName: string | null
  readonly telemetrySdkVersion: string | null
  readonly lastSeenNanos: string
  readonly instanceCount: string
}): ServiceCatalogRow {
  return {
    name: row.name,
    serviceVersion: row.serviceVersion,
    serviceNamespace: row.serviceNamespace,
    deploymentEnvironment: row.deploymentEnvironment,
    telemetrySdkLanguage: row.telemetrySdkLanguage,
    telemetrySdkName: row.telemetrySdkName,
    telemetrySdkVersion: row.telemetrySdkVersion,
    lastSeenNanos: row.lastSeenNanos,
    instanceCount: row.instanceCount,
  }
}

function mapSummary(row: {
  readonly name: string
  readonly lastSeenNanos: string
  readonly spanCount: string
  readonly errorCount: string
  readonly p95Ms: number | null
}): ServiceSummary {
  return {
    name: row.name,
    lastSeenNanos: row.lastSeenNanos,
    spanCount: row.spanCount,
    errorCount: row.errorCount,
    p95Ms: row.p95Ms,
  }
}

function mapRed(red: ServiceDetailQuery["red"]): SpanRed {
  return {
    rate: mapSeries(red.rate),
    errorRate: mapSeries(red.errorRate),
    p50: mapSeries(red.p50),
    p95: mapSeries(red.p95),
    p99: mapSeries(red.p99),
  }
}

function mapOverview(overview: ServiceDetailQuery["overview"]): ServiceOverview {
  return {
    cpu: mapSeries(overview.cpu),
    memory: mapSeries(overview.memory),
    requestRate: mapSeries(overview.requestRate),
    errorRate: mapSeries(overview.errorRate),
    latencyP50: mapSeries(overview.latencyP50),
    latencyP95: mapSeries(overview.latencyP95),
    latencyP99: mapSeries(overview.latencyP99),
  }
}

function mapRelease(row: {
  readonly version: string
  readonly firstSeenNanos: string
  readonly lastSeenNanos: string
  readonly spanCount: string
}): ReleaseWindow {
  return {
    version: row.version,
    firstSeenNanos: row.firstSeenNanos,
    lastSeenNanos: row.lastSeenNanos,
    spanCount: row.spanCount,
  }
}

function mapExemplar(row: {
  readonly tsNanos: string
  readonly service: string
  readonly name: string
  readonly value: number
  readonly traceId: string
  readonly spanId: string
  readonly invocationId: string | null
  readonly attributes: string
}): MetricExemplar {
  return {
    tsNanos: row.tsNanos,
    service: row.service,
    name: row.name,
    value: row.value,
    traceId: row.traceId,
    spanId: row.spanId,
    invocationId: row.invocationId,
    attributes: row.attributes,
  }
}

function mapTrace(row: {
  readonly traceId: string
  readonly rootName: string
  readonly service: string
  readonly startNanos: string
  readonly durationNs: string
  readonly spanCount: number
  readonly hasError: boolean
}): TraceSummary {
  return {
    traceId: row.traceId,
    rootName: row.rootName,
    service: row.service,
    startNanos: row.startNanos,
    durationNs: row.durationNs,
    spanCount: row.spanCount,
    hasError: row.hasError,
  }
}

function mapRuntime(row: ServiceDetailQuery["runtimeSnapshot"][number]): RuntimeMetric {
  return {
    family: row.family,
    metric: row.metric,
    unit: row.unit,
    points: mapSeries(row.points),
  }
}

export function mapServicesList(data: ServicesListQuery): ServicesData {
  return {
    serviceList: data.serviceList.map(mapSummary),
    serviceCatalog: data.serviceCatalog.map(mapCatalogRow),
  }
}

export function mapServiceDetail(data: ServiceDetailQuery): ServiceDetailData {
  return {
    red: mapRed(data.red),
    overview: mapOverview(data.overview),
    releases: data.releases.map(mapRelease),
    serviceCatalog: data.serviceCatalog.map(mapCatalogRow),
    httpDurationExemplars: data.httpDurationExemplars.map(mapExemplar),
    rpcDurationExemplars: data.rpcDurationExemplars.map(mapExemplar),
    runtimeSnapshot: data.runtimeSnapshot.map(mapRuntime),
    tracesPage: { items: data.tracesPage.items.map(mapTrace) },
  }
}
