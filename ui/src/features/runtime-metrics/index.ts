// Public facade for route-less runtime metric capabilities (Plan 149).
// Named exports only — no wildcard barrel.

export { MetricStrip } from "@/features/runtime-metrics/components/metric-strip"
export { RuntimeSnapshotCard } from "@/features/runtime-metrics/components/runtime-snapshot-card"
export type { MetricPoint, RuntimeMetric } from "@/domain/runtime-metrics/runtime-metric"

export {
  clampCounterDelta,
  coerceAggregation,
  defaultAggregation,
  decodeMetricQuerySpec,
  encodeGraduationParams,
  encodeMetricQuerySpec,
  inferMetricKind,
  isLegalAggregation,
  legalAggregations,
  type GraduationTarget,
  type MetricAggregation,
  type MetricKind,
  type MetricQuerySpec,
} from "@/features/runtime-metrics/model/metric-aggregation"
