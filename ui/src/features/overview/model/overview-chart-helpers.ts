import { formatTimeInRange } from "@/shared/format"
import type { ResolvedRange } from "@/domain/time-range/range"

export type SeriesPoint = {
  tsNanos: string
  value: number
}

export function mergeSignalSeries(
  range: ResolvedRange,
  spans: SeriesPoint[],
  errors: SeriesPoint[]
) {
  const rows = new Map<string, { tsNanos: string; spans: number; errors: number }>()
  for (const point of spans) {
    rows.set(point.tsNanos, {
      tsNanos: point.tsNanos,
      spans: point.value,
      errors: 0,
    })
  }
  for (const point of errors) {
    const row = rows.get(point.tsNanos) ?? {
      tsNanos: point.tsNanos,
      spans: 0,
      errors: 0,
    }
    row.errors = point.value
    rows.set(point.tsNanos, row)
  }
  return Array.from(rows.values())
    .sort((a, b) => (BigInt(a.tsNanos) < BigInt(b.tsNanos) ? -1 : 1))
    .map((point) => ({
      ...point,
      time: formatTimeInRange(point.tsNanos, range),
    }))
}

export function sampleSignalData(range: ResolvedRange) {
  return sampleTimes(range).map((tsNanos, index) => ({
    tsNanos,
    time: formatTimeInRange(tsNanos, range),
    spans: [12, 18, 14, 28, 22, 32][index] ?? 0,
    errors: [0, 1, 0, 2, 1, 0][index] ?? 0,
  }))
}

export function sampleLatencyData(range: ResolvedRange) {
  return sampleTimes(range).map((tsNanos, index) => {
    const p50 = [18, 21, 19, 24, 22, 20][index] ?? 20
    const p95 = p50 + ([38, 44, 40, 52, 48, 42][index] ?? 40)
    const p99 = p95 + ([40, 52, 44, 70, 58, 46][index] ?? 50)
    return {
      tsNanos,
      p50,
      p95,
      p99,
      p50Band: p50,
      p95Band: Math.max(p95 - p50, 0),
      p99Band: Math.max(p99 - p95, 0),
    }
  })
}

function sampleTimes(range: ResolvedRange) {
  const from = BigInt(range.fromNanos)
  const span = BigInt(range.toNanos) - from
  return Array.from({ length: 6 }, (_, index) => (from + (span * BigInt(index)) / 5n).toString())
}
