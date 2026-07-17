import type { WidgetSeries } from "@/features/dashboards/api/widget-series-schema"
import type { Widget } from "@/features/dashboards/model/widget"
import { formatTimeInRange } from "@/shared/format"
import type { ResolvedRange } from "@/domain/time-range/range"

export type WidgetData = {
  widget: Widget
  groups: string[]
  rows: Record<string, number | string>[]
}

const MAX_GROUPS = 5
const MAX_FILLED_ROWS = 2_000

export function toWidgetData(
  widget: Widget,
  series: WidgetSeries[],
  range: ResolvedRange
): WidgetData {
  const kept = series.slice(0, MAX_GROUPS)
  const groups = kept.map((s, i) => s.groupValue ?? (i === 0 ? "value" : `#${i}`))
  const byTime = new Map<string, Record<string, number | string>>()
  kept.forEach((s, index) => {
    const group = groups[index]
    if (!group) return
    for (const point of s.points) {
      const row = byTime.get(point.tsNanos) ?? {
        time: formatTimeInRange(point.tsNanos, range),
      }
      row[group] = point.value
      byTime.set(point.tsNanos, row)
    }
  })
  const entries = [...byTime.entries()].sort(([a], [b]) => (BigInt(a) < BigInt(b) ? -1 : 1))
  const rows = fillBucketGaps(entries, range)
  return { widget, groups, rows }
}

/** Insert empty rows for skipped buckets so a gauge that stopped reporting
 * renders a line BREAK instead of silently bridging the gap. */
function fillBucketGaps(
  entries: Array<[string, Record<string, number | string>]>,
  range: ResolvedRange
): Array<Record<string, number | string>> {
  if (entries.length < 2) return entries.map(([, row]) => row)
  let step = 0n
  for (let i = 1; i < entries.length; i += 1) {
    const delta = BigInt(entries[i]![0]) - BigInt(entries[i - 1]![0])
    if (delta > 0n && (step === 0n || delta < step)) step = delta
  }
  if (step === 0n) return entries.map(([, row]) => row)
  const rows: Array<Record<string, number | string>> = []
  for (let i = 0; i < entries.length; i += 1) {
    const [ts, row] = entries[i]!
    if (i > 0) {
      let cursor = BigInt(entries[i - 1]![0]) + step
      const end = BigInt(ts)
      while (cursor < end && rows.length < MAX_FILLED_ROWS) {
        rows.push({ time: formatTimeInRange(cursor.toString(), range) })
        cursor += step
      }
    }
    rows.push(row)
  }
  return rows
}
