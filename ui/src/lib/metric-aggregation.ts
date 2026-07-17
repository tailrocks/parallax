/** Metric aggregation legality + explorer URL helpers (plan 168).
 *
 * Aggregation options are typed by metric kind so illegal combinations are
 * unrepresentable in the UI. Pure — no GraphQL, no routes.
 *
 * Preliminary — peer must wire metricCatalog/metricQuery, /metrics routes,
 * graduation URL handoff to dashboards/alerts, and plan-105 decision record.
 */

export type MetricKind = "sum" | "gauge" | "histogram" | "summary" | "unknown"

export type MetricAggregation =
  | "sum"
  | "rate"
  | "increase"
  | "avg"
  | "min"
  | "max"
  | "last"
  | "p50"
  | "p95"
  | "p99"

const SUM_AGGS: readonly MetricAggregation[] = ["rate", "increase", "sum"]
const GAUGE_AGGS: readonly MetricAggregation[] = ["avg", "min", "max", "last"]
const HISTOGRAM_AGGS: readonly MetricAggregation[] = [
  "p50",
  "p95",
  "p99",
  "avg",
]
const SUMMARY_AGGS: readonly MetricAggregation[] = ["p50", "p95", "p99", "avg"]

/** Legal aggregations for a metric kind (empty for unknown). */
export function legalAggregations(
  kind: MetricKind
): readonly MetricAggregation[] {
  switch (kind) {
    case "sum":
      return SUM_AGGS
    case "gauge":
      return GAUGE_AGGS
    case "histogram":
      return HISTOGRAM_AGGS
    case "summary":
      return SUMMARY_AGGS
    case "unknown":
      return []
  }
}

/** Default aggregation when opening a metric of this kind. */
export function defaultAggregation(kind: MetricKind): MetricAggregation | null {
  const legal = legalAggregations(kind)
  return legal[0] ?? null
}

/** True when `agg` is legal for `kind`. */
export function isLegalAggregation(
  kind: MetricKind,
  agg: MetricAggregation
): boolean {
  return legalAggregations(kind).includes(agg)
}

/**
 * Coerce a (possibly stale URL) aggregation into a legal one for the kind.
 * Returns null only when the kind has no legal aggregations.
 */
export function coerceAggregation(
  kind: MetricKind,
  agg: string | null | undefined
): MetricAggregation | null {
  const legal = legalAggregations(kind)
  if (legal.length === 0) return null
  if (agg && legal.includes(agg as MetricAggregation)) {
    return agg as MetricAggregation
  }
  return legal[0] ?? null
}

/** Infer kind from a native metric name / OTel type string (best-effort). */
export function inferMetricKind(
  typeOrName: string | null | undefined
): MetricKind {
  if (!typeOrName) return "unknown"
  const t = typeOrName.toLowerCase()
  if (
    t === "sum" ||
    t === "counter" ||
    t === "monotonic_counter" ||
    t.endsWith("_total") ||
    t.includes("counter")
  ) {
    return "sum"
  }
  if (t === "gauge" || t.includes("gauge")) return "gauge"
  if (
    t === "histogram" ||
    t === "exponentialhistogram" ||
    t === "exponential_histogram" ||
    t.endsWith("_bucket") ||
    t.includes("histogram")
  ) {
    return "histogram"
  }
  if (t === "summary" || t.includes("summary")) return "summary"
  return "unknown"
}

/** Explorer query spec carried in URL params / graduation handoff. */
export interface MetricQuerySpec {
  name: string
  kind: MetricKind
  aggregation: MetricAggregation
  where?: string
  groupBy?: string
  /** Bucket step in seconds; omit for auto. */
  stepSeconds?: number
}

const SPEC_KEYS = ["q", "type", "agg", "where", "groupBy", "step"] as const

/** Encode a query spec into URLSearchParams keys used by the explorer. */
export function encodeMetricQuerySpec(spec: MetricQuerySpec): URLSearchParams {
  const params = new URLSearchParams()
  params.set("q", spec.name)
  params.set("type", spec.kind)
  params.set("agg", spec.aggregation)
  if (spec.where) params.set("where", spec.where)
  if (spec.groupBy) params.set("groupBy", spec.groupBy)
  if (spec.stepSeconds != null && spec.stepSeconds > 0) {
    params.set("step", String(spec.stepSeconds))
  }
  return params
}

/**
 * Decode explorer URL params into a query spec.
 * Invalid agg is coerced; missing name yields null.
 */
export function decodeMetricQuerySpec(
  params: URLSearchParams | Record<string, string | undefined>
): MetricQuerySpec | null {
  const get = (key: string): string | undefined => {
    if (params instanceof URLSearchParams) {
      return params.get(key) ?? undefined
    }
    return params[key]
  }
  const name = get("q")?.trim()
  if (!name) return null
  const kind = inferMetricKind(get("type") ?? undefined)
  const aggregation = coerceAggregation(
    kind === "unknown" ? "gauge" : kind,
    get("agg")
  )
  if (!aggregation) return null
  const stepRaw = get("step")
  const stepSeconds = stepRaw ? Number(stepRaw) : undefined
  const where = get("where")
  const groupBy = get("groupBy")
  const validStep =
    stepSeconds != null && Number.isFinite(stepSeconds) && stepSeconds > 0
      ? stepSeconds
      : undefined
  return {
    name,
    kind: kind === "unknown" ? "gauge" : kind,
    aggregation,
    ...(where ? { where } : {}),
    ...(groupBy ? { groupBy } : {}),
    ...(validStep === undefined ? {} : { stepSeconds: validStep }),
  }
}

/** Graduation target for "add to dashboard" / "create alert". */
export type GraduationTarget = "dashboard" | "alert"

/**
 * Build graduation URL search string carrying the current query spec.
 * Alert graduation sets `signal_type=metric` for plan-167 form init.
 */
export function encodeGraduationParams(
  spec: MetricQuerySpec,
  target: GraduationTarget
): URLSearchParams {
  const params = encodeMetricQuerySpec(spec)
  if (target === "alert") {
    params.set("signal_type", "metric")
    params.set("metric_name", spec.name)
    params.set("metric_aggregation", spec.aggregation)
  } else {
    params.set("widget", "metric")
  }
  return params
}

/** Clamp a counter rate/increase delta after a reset (never negative). */
export function clampCounterDelta(current: number, previous: number): number {
  const delta = current - previous
  return delta < 0 ? 0 : delta
}

export { SPEC_KEYS }
