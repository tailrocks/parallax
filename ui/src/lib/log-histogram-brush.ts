/** Log histogram brush-zoom pure helpers (plan 165).
 *
 * Maps pointer ranges onto histogram bucket edges so the page time window
 * snaps cleanly. Pure — no React/Recharts coupling.
 *
 * Preliminary — peer must mount the selection overlay on the logs histogram,
 * wire URL range updates, Esc cancel, and browser evidence.
 */

export interface HistogramBucket {
  /** Bucket start, unix nanos (or any monotonic time unit). */
  start: number
  /** Bucket end (exclusive), same unit as start. */
  end: number
  count: number
}

export interface BrushRange {
  start: number
  end: number
}

/**
 * Snap a raw brush `[t0, t1]` onto bucket edges.
 * Selection extends to the leftmost bucket that overlaps t0 and the rightmost
 * that overlaps t1. Degenerate / inverted inputs are normalized.
 * Returns null when buckets are empty or the brush hits no bucket.
 */
export function snapBrushToBuckets(
  brush: BrushRange,
  buckets: readonly HistogramBucket[]
): BrushRange | null {
  if (buckets.length === 0) return null
  const lo = Math.min(brush.start, brush.end)
  const hi = Math.max(brush.start, brush.end)
  if (hi === lo) {
    // Point click: select the bucket containing the point.
    const hit = buckets.find((b) => lo >= b.start && lo < b.end)
    return hit ? { start: hit.start, end: hit.end } : null
  }

  let first: HistogramBucket | undefined
  let last: HistogramBucket | undefined
  for (const b of buckets) {
    // Overlap test: [b.start, b.end) ∩ [lo, hi) non-empty
    if (b.end > lo && b.start < hi) {
      if (!first) first = b
      last = b
    }
  }
  if (!first || !last) return null
  return { start: first.start, end: last.end }
}

/**
 * Map a pixel X within a chart plot width to a time using linear scale
 * over `[domainStart, domainEnd]`.
 */
export function pxToTime(
  px: number,
  plotWidthPx: number,
  domainStart: number,
  domainEnd: number
): number {
  if (plotWidthPx <= 0) return domainStart
  const t = Math.min(1, Math.max(0, px / plotWidthPx))
  return domainStart + t * (domainEnd - domainStart)
}

/** Inverse of pxToTime. */
export function timeToPx(
  time: number,
  plotWidthPx: number,
  domainStart: number,
  domainEnd: number
): number {
  const span = domainEnd - domainStart
  if (span <= 0 || plotWidthPx <= 0) return 0
  const t = (time - domainStart) / span
  return Math.min(plotWidthPx, Math.max(0, t * plotWidthPx))
}

/**
 * Build ~target bucket edges over [from, to). Last bucket may be shorter.
 * Returns empty when the window is non-positive.
 */
export function buildUniformBuckets(
  from: number,
  to: number,
  targetCount: number
): HistogramBucket[] {
  if (!(to > from) || targetCount <= 0) return []
  const n = Math.max(1, Math.floor(targetCount))
  const width = (to - from) / n
  const buckets: HistogramBucket[] = []
  for (let i = 0; i < n; i++) {
    const start = from + i * width
    const end = i === n - 1 ? to : from + (i + 1) * width
    buckets.push({ start, end, count: 0 })
  }
  return buckets
}

/** Default histogram target bucket count (~150 per plan 165). */
export const DEFAULT_HISTOGRAM_BUCKETS = 150
