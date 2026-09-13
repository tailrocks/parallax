// Previous-period compare for metric detail (X2 timeshift).
// The backend serves the shifted window (`metricQuery(shiftSeconds:)`); this
// module aligns that series onto the current axis for overlay rendering.

import type { ResolvedRange } from "@/domain/time-range/range"

export const COMPARE_PREVIOUS = "previous"

// GraphQL Int ceiling: windows longer than this still shift by at most i32 seconds.
const MAX_SHIFT_SECONDS = 2_147_483_647

export interface TimeshiftPoint {
  tsNanos: string
  value: number
}

export interface TimeshiftSeries {
  groupValue: string | null
  points: TimeshiftPoint[]
}

export type ChartRow = Record<string, string | number>

export function compareGroupKey(group: string): string {
  return `${group} (previous)`
}

/**
 * Whole-second window length (minimum 1): the `shiftSeconds` that fetches the
 * immediately previous window. Unparseable ranges compare against one second.
 */
export function previousWindowShiftSeconds(range: ResolvedRange): number {
  try {
    const length = (BigInt(range.toNanos) - BigInt(range.fromNanos)) / 1_000_000_000n
    if (length < 1n) return 1
    if (length > BigInt(MAX_SHIFT_SECONDS)) return MAX_SHIFT_SECONDS
    return Number(length)
  } catch {
    return 1
  }
}

/** Slide a previous-window series forward so its buckets overlay the current axis. */
export function shiftSeriesForward(
  series: ReadonlyArray<TimeshiftSeries>,
  shiftNanos: bigint
): TimeshiftSeries[] {
  return series.map((entry, index) => ({
    groupValue: entry.groupValue ?? `series-${index + 1}`,
    points: entry.points.flatMap((point) => {
      try {
        return [{ tsNanos: (BigInt(point.tsNanos) + shiftNanos).toString(), value: point.value }]
      } catch {
        return []
      }
    }),
  }))
}

function groupName(entry: TimeshiftSeries, index: number): string {
  return entry.groupValue ?? `series-${index + 1}`
}

function rowTime(tsNanos: string): string {
  return new Date(Number(BigInt(tsNanos) / 1_000_000n)).toLocaleTimeString()
}

/**
 * Chart rows for the current series plus an optional previous-window overlay.
 * Current-series semantics are the metric-detail contract: the newest bucket
 * is usually incomplete, so its segment renders as a dashed `__tail`
 * continuation; previous-window buckets are fully in the past and need none.
 */
export function buildCompareRows(
  series: ReadonlyArray<TimeshiftSeries>,
  previous: ReadonlyArray<TimeshiftSeries>,
  shiftNanos: bigint
): ChartRow[] {
  const byTime = new Map<string, ChartRow>()
  series.forEach((entry, index) => {
    const key = groupName(entry, index)
    const tailStart = Math.max(entry.points.length - 2, 0)
    entry.points.forEach((point, pointIndex) => {
      const row = byTime.get(point.tsNanos) ?? {
        time: rowTime(point.tsNanos),
        tsNanos: point.tsNanos,
      }
      if (pointIndex < entry.points.length - 1) {
        row[key] = point.value
      }
      if (pointIndex >= tailStart && entry.points.length > 1) {
        row[`${key}__tail`] = point.value
      }
      byTime.set(point.tsNanos, row)
    })
  })
  for (const entry of shiftSeriesForward(previous, shiftNanos)) {
    const key = compareGroupKey(entry.groupValue ?? "")
    for (const point of entry.points) {
      const row = byTime.get(point.tsNanos) ?? {
        time: rowTime(point.tsNanos),
        tsNanos: point.tsNanos,
      }
      row[key] = point.value
      byTime.set(point.tsNanos, row)
    }
  }
  return Array.from(byTime.entries())
    .sort(([a], [b]) => (BigInt(a) < BigInt(b) ? -1 : 1))
    .map(([, row]) => row)
}
