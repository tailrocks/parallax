import type { RuntimeMetric } from "@/domain/runtime-metrics/runtime-metric"
import { formatTimeShort } from "@/lib/format"
import type { ResolvedRange } from "@/lib/range"

import type { ServiceCatalogRow } from "@/features/services/model/service-summary"

export type SeriesPoint = {
  readonly tsNanos: string
  readonly value: number
}

export interface SpanRed {
  readonly rate: readonly SeriesPoint[]
  readonly errorRate: readonly SeriesPoint[]
  readonly p50: readonly SeriesPoint[]
  readonly p95: readonly SeriesPoint[]
  readonly p99: readonly SeriesPoint[]
}

export interface ServiceOverview {
  readonly cpu: readonly SeriesPoint[]
  readonly memory: readonly SeriesPoint[]
  readonly requestRate: readonly SeriesPoint[]
  readonly errorRate: readonly SeriesPoint[]
  readonly latencyP50: readonly SeriesPoint[]
  readonly latencyP95: readonly SeriesPoint[]
  readonly latencyP99: readonly SeriesPoint[]
}

export interface MetricExemplar {
  readonly tsNanos: string
  readonly service: string
  readonly name: string
  readonly value: number
  readonly traceId: string
  readonly spanId: string
  readonly invocationId: string | null
  readonly attributes: string
}

export interface ReleaseWindow {
  readonly version: string
  readonly firstSeenNanos: string
  readonly lastSeenNanos: string
  readonly spanCount: string
}

export interface TraceSummary {
  readonly traceId: string
  readonly rootName: string
  readonly service: string
  readonly startNanos: string
  readonly durationNs: string
  readonly spanCount: number
  readonly hasError: boolean
}

export interface ServiceDetailData {
  readonly red: SpanRed
  readonly overview: ServiceOverview
  readonly releases: readonly ReleaseWindow[]
  readonly serviceCatalog: readonly ServiceCatalogRow[]
  readonly httpDurationExemplars: readonly MetricExemplar[]
  readonly rpcDurationExemplars: readonly MetricExemplar[]
  readonly runtimeSnapshot: readonly RuntimeMetric[]
  readonly tracesPage: { readonly items: readonly TraceSummary[] }
}

export type MetricChartPoint = {
  label: string
  tsNanos: string
  [key: string]: string | number
}

export type LatencyBandPoint = {
  readonly tsNanos: string
  readonly label: string
  readonly p50: number
  readonly p95: number
  readonly p99: number
  readonly p50Band: number
  readonly p95Band: number
  readonly p99Band: number
}

export type ExemplarMarker = {
  readonly exemplar: MetricExemplar
  readonly x: number
  readonly y: number
}

export function stepSecondsForRange(range: ResolvedRange): number {
  const spanNs = BigInt(range.toNanos) - BigInt(range.fromNanos)
  const seconds = Number(spanNs / 1_000_000_000n)
  return Math.max(30, Math.round(seconds / 60))
}

export function totalSeries(points: readonly SeriesPoint[]): number {
  return points.reduce((sum, point) => sum + point.value, 0)
}

export function latestValue(points: readonly SeriesPoint[]): number | null {
  return points.at(-1)?.value ?? null
}

export function latestErrorRate(red: SpanRed): number {
  return latestValue(red.errorRate) ?? 0
}

export function latencyBands(red: SpanRed): LatencyBandPoint[] {
  const points = new Map<
    string,
    { tsNanos: string; p50: number; p95: number; p99: number }
  >()
  for (const point of red.p50) {
    points.set(point.tsNanos, {
      tsNanos: point.tsNanos,
      p50: point.value,
      p95: point.value,
      p99: point.value,
    })
  }
  for (const point of red.p95) {
    const row = points.get(point.tsNanos) ?? {
      tsNanos: point.tsNanos,
      p50: 0,
      p95: 0,
      p99: 0,
    }
    row.p95 = point.value
    points.set(point.tsNanos, row)
  }
  for (const point of red.p99) {
    const row = points.get(point.tsNanos) ?? {
      tsNanos: point.tsNanos,
      p50: 0,
      p95: 0,
      p99: 0,
    }
    row.p99 = point.value
    points.set(point.tsNanos, row)
  }
  return Array.from(points.values())
    .sort((a, b) => (BigInt(a.tsNanos) < BigInt(b.tsNanos) ? -1 : 1))
    .map((point) => ({
      ...point,
      label: formatTimeShort(point.tsNanos),
      p50Band: Math.max(point.p50, 0),
      p95Band: Math.max(point.p95 - point.p50, 0),
      p99Band: Math.max(point.p99 - point.p95, 0),
    }))
}

export function formatChartTime(tsNanos: string): string {
  return formatTimeShort(tsNanos)
}

export function toLineData(
  series: Record<string, readonly SeriesPoint[]>,
  mapValue: (key: string, value: number, tsNanos: string) => number = (
    _key,
    value
  ) => value
): MetricChartPoint[] {
  const rows = new Map<string, MetricChartPoint>()
  for (const [key, points] of Object.entries(series)) {
    for (const point of points) {
      const row = rows.get(point.tsNanos) ?? {
        tsNanos: point.tsNanos,
        label: formatChartTime(point.tsNanos),
      }
      row[key] = mapValue(key, point.value, point.tsNanos)
      rows.set(point.tsNanos, row)
    }
  }
  return Array.from(rows.values()).sort((a, b) =>
    BigInt(a.tsNanos) < BigInt(b.tsNanos) ? -1 : 1
  )
}

export function exemplarMarkers(
  exemplars: readonly MetricExemplar[],
  data: ReadonlyArray<{
    tsNanos: string
    p50?: number
    p95?: number
    p99?: number
  }>,
  range: ResolvedRange
): ExemplarMarker[] {
  const from = BigInt(range.fromNanos)
  const to = BigInt(range.toNanos)
  const span = to - from
  if (span <= 0n) return []
  const chartMax = data.reduce(
    (max, row) => Math.max(max, row.p50 ?? 0, row.p95 ?? 0, row.p99 ?? 0),
    0
  )
  const exemplarMax = exemplars.reduce(
    (max, exemplar) =>
      Number.isFinite(exemplar.value) ? Math.max(max, exemplar.value) : max,
    0
  )
  const maxValue = Math.max(chartMax, exemplarMax, 1)
  return exemplars
    .filter((exemplar) => exemplar.traceId && exemplar.spanId)
    .map((exemplar) => {
      const ts = BigInt(exemplar.tsNanos)
      const clampedTs = ts < from ? from : ts > to ? to : ts
      const x = Number(((clampedTs - from) * 10_000n) / span) / 100
      const ratio = Number.isFinite(exemplar.value)
        ? Math.max(0, Math.min(1, exemplar.value / maxValue))
        : 0
      return {
        exemplar,
        x: Math.max(5, Math.min(95, x)),
        y: Math.max(12, Math.min(86, 86 - ratio * 70)),
      }
    })
}
