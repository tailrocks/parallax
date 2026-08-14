import { resolveRangeSearch, type ResolvedRange } from "@/domain/time-range/range"
import {
  coerceAggregation,
  inferMetricKind,
  legalAggregations,
  type MetricAggregation,
  type MetricKind,
} from "@/features/runtime-metrics"
import { gqlString, graphqlCached } from "@/platform/graphql/transport"
import { whereClauseFromSearch } from "@/shared/where-clause"

export interface MetricDetailSearch {
  range?: string | undefined
  from?: string | undefined
  to?: string | undefined
  agg?: string | undefined
  where?: string | undefined
  groupBy?: string | undefined
  step?: string | undefined
  kind?: string | undefined
}

export interface SeriesOut {
  groupValue: string | null
  points: Array<{ tsNanos: string; value: number }>
}

export interface MetricExemplarLink {
  tsNanos: string
  traceId: string
  spanId: string
  value: number
}

export interface DetailData {
  labels: string[]
  series: SeriesOut[]
  range: ResolvedRange
  exemplars: MetricExemplarLink[]
}

export function backendKind(kind: MetricKind): "gauge" | "sum" | "histogram" {
  switch (kind) {
    case "sum":
      return "sum"
    case "histogram":
    case "summary":
      return "histogram"
    case "gauge":
    case "unknown":
      return "gauge"
  }
}

export function supportedAggregations(kind: MetricKind): MetricAggregation[] {
  return [...legalAggregations(backendKind(kind))]
}

export function resolveAggregation(kind: MetricKind, raw: string | undefined): MetricAggregation {
  return coerceAggregation(backendKind(kind), raw) ?? "avg"
}

async function loadExemplars(
  metricName: string,
  range: ResolvedRange
): Promise<MetricExemplarLink[]> {
  try {
    const data = await graphqlCached<{ metricExemplars: MetricExemplarLink[] }>(`{
      metricExemplars(name: "${gqlString(metricName)}", fromNanos: "${range.fromNanos}", toNanos: "${range.toNanos}", limit: 20) {
        tsNanos traceId spanId value
      }
    }`)
    return data.metricExemplars.filter((row) => row.traceId.length > 0)
  } catch {
    return []
  }
}

function queryArguments(metricName: string, search: MetricDetailSearch) {
  const range = resolveRangeSearch(search)
  const kind = (search.kind as MetricKind) || inferMetricKind(metricName)
  const agg = resolveAggregation(kind, search.agg)
  const stepSeconds = Number(search.step ?? "60") || 60
  const name = `"${gqlString(metricName)}"`
  const window = `fromNanos: "${range.fromNanos}", toNanos: "${range.toNanos}"`
  const groupBy = search.groupBy ? `, groupBy: "${gqlString(search.groupBy)}"` : ""
  const whereFilters = whereClauseFromSearch(search.where)
  const where = whereFilters.length
    ? `, attributeFilters: [${whereFilters
        .map(
          (filter) =>
            `{key: "${gqlString(filter.key)}", op: "${gqlString(filter.op)}", value: "${gqlString(filter.value)}"}`
        )
        .join(", ")}]`
    : ""
  return { range, kind, agg, stepSeconds, name, window, groupBy, where }
}

async function loadCanonicalDetail(
  metricName: string,
  args: ReturnType<typeof queryArguments>
): Promise<DetailData> {
  const data = await graphqlCached<{
    metricLabels: string[]
    metricQuery: { series: SeriesOut[] }
  }>(`{
    metricLabels(name: ${args.name})
    metricQuery(name: ${args.name}, kind: "${gqlString(backendKind(args.kind))}", agg: "${gqlString(args.agg)}", ${args.window}, stepSeconds: ${args.stepSeconds}${args.groupBy}${args.where}) {
      kind effectiveStepSeconds series { groupValue points { tsNanos value } }
    }
  }`)
  return {
    labels: data.metricLabels,
    series: data.metricQuery.series,
    range: args.range,
    exemplars: await loadExemplars(metricName, args.range),
  }
}

async function loadLegacyDetail(
  metricName: string,
  args: ReturnType<typeof queryArguments>
): Promise<DetailData> {
  if ((args.kind === "histogram" || args.kind === "summary") && args.agg.startsWith("p")) {
    const q = Number(args.agg.slice(1)) / 100
    const data = await graphqlCached<{
      metricLabels: string[]
      histogramQuantile: Array<{ tsNanos: string; value: number }>
    }>(`{
      metricLabels(name: ${args.name})
      histogramQuantile(name: ${args.name}, ${args.window}, q: ${q}, stepSeconds: ${args.stepSeconds}) { tsNanos value }
    }`)
    return {
      labels: data.metricLabels,
      series: [{ groupValue: null, points: data.histogramQuantile }],
      range: args.range,
      exemplars: await loadExemplars(metricName, args.range),
    }
  }
  const data = await graphqlCached<{ metricLabels: string[]; metricSeries: SeriesOut[] }>(`{
    metricLabels(name: ${args.name})
    metricSeries(name: ${args.name}, ${args.window}, agg: "${gqlString(args.agg)}", stepSeconds: ${args.stepSeconds}${args.groupBy}) {
      groupValue points { tsNanos value }
    }
  }`)
  return {
    labels: data.metricLabels,
    series: data.metricSeries,
    range: args.range,
    exemplars: await loadExemplars(metricName, args.range),
  }
}

export async function loadMetricDetail(
  metricName: string,
  search: MetricDetailSearch
): Promise<DetailData> {
  const args = queryArguments(metricName, search)
  try {
    return await loadCanonicalDetail(metricName, args)
  } catch {
    return loadLegacyDetail(metricName, args)
  }
}
