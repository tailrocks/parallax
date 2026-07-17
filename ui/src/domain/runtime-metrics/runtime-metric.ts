// Framework-neutral runtime metric values (Plan 149).
// Fetch/decode/presentation live in features/runtime-metrics.

export type MetricPoint = {
  readonly tsNanos: string
  readonly value: number
}

export type RuntimeMetric = {
  readonly family: string
  readonly metric: string
  readonly unit: string | null
  readonly points: readonly MetricPoint[]
}
