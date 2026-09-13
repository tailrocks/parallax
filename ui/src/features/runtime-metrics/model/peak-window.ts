/** Peak timestamp + traces search for a metric series (spike → related traces). */

export const PEAK_WINDOW_PAD_NS = 60_000_000_000n

export interface SeriesPoint {
  tsNanos: string
  value: number
}

export interface SeriesBucket {
  groupValue: string | null
  points: SeriesPoint[]
}

export interface PeakWindow {
  fromNanos: string
  toNanos: string
  peakNanos: string
  peakValue: number
}

export interface ChartAnnotationMark {
  key: string
  time: string
  title: string
  service: string
  kind: string
  tsNanos: string
}

/** Highest-value sample across series, padded into a traces time window. */
export function peakWindowFromSeries(
  series: ReadonlyArray<SeriesBucket>,
  padNanos: bigint = PEAK_WINDOW_PAD_NS
): PeakWindow | null {
  let peak: { tsNanos: bigint; value: number } | null = null
  for (const bucket of series) {
    for (const point of bucket.points) {
      let ts: bigint
      try {
        ts = BigInt(point.tsNanos)
      } catch {
        continue
      }
      if (peak === null || point.value > peak.value) {
        peak = { tsNanos: ts, value: point.value }
      }
    }
  }
  if (peak === null) return null
  const from = peak.tsNanos > padNanos ? peak.tsNanos - padNanos : 0n
  return {
    fromNanos: from.toString(),
    toNanos: (peak.tsNanos + padNanos).toString(),
    peakNanos: peak.tsNanos.toString(),
    peakValue: peak.value,
  }
}

/** `/traces` search for the padded peak window. */
export function tracesAroundPeakSearch(
  peak: PeakWindow,
  service?: string
): { range: "custom"; from: string; to: string; service?: string } {
  return {
    range: "custom",
    from: peak.fromNanos,
    to: peak.toNanos,
    ...(service ? { service } : {}),
  }
}

function absDiff(left: bigint, right: bigint): bigint {
  return left >= right ? left - right : right - left
}

/** Snap an annotation timestamp onto the nearest chart category (`time`). */
export function nearestChartTime(
  rows: ReadonlyArray<{ time?: unknown; tsNanos?: unknown }>,
  tsNanos: string
): string | undefined {
  let target: bigint
  try {
    target = BigInt(tsNanos)
  } catch {
    return undefined
  }
  let best: { time: string; dist: bigint } | undefined
  for (const row of rows) {
    if (typeof row.time !== "string" || typeof row.tsNanos !== "string") continue
    let ts: bigint
    try {
      ts = BigInt(row.tsNanos)
    } catch {
      continue
    }
    const dist = absDiff(ts, target)
    if (!best || dist < best.dist) best = { time: row.time, dist }
  }
  return best?.time
}

export function chartAnnotationMarks(
  rows: ReadonlyArray<{ time?: unknown; tsNanos?: unknown }>,
  annotations: ReadonlyArray<{
    tsNanos: string
    kind: string
    title: string
    service: string
  }>
): ChartAnnotationMark[] {
  const marks: ChartAnnotationMark[] = []
  const used = new Set<string>()
  for (const annotation of annotations) {
    const time = nearestChartTime(rows, annotation.tsNanos)
    if (!time) continue
    const key = `${annotation.service}:${annotation.title}:${annotation.tsNanos}`
    if (used.has(key)) continue
    used.add(key)
    marks.push({
      key,
      time,
      title: annotation.title,
      service: annotation.service,
      kind: annotation.kind,
      tsNanos: annotation.tsNanos,
    })
  }
  return marks
}
