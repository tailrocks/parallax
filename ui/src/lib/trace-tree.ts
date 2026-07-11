export interface TraceTreeSpan {
  spanId: string
  parentSpanId: string | null
  tsNanos: string
  durationNs: string
}

export interface OrderedTraceSpan<T extends TraceTreeSpan> {
  span: T
  depth: number
  offsetPct: number
  widthPct: number
}

export interface TraceWindow {
  startNs: bigint
  durationNs: bigint
}

export interface ServiceTraceSpan extends TraceTreeSpan {
  service: string
}

export interface ErrorTraceSpan extends TraceTreeSpan {
  statusCode: string
}

export interface SkewPair {
  parentId: string
  childId: string
  driftMs: number
}

export interface SkewReport {
  suspectPairs: SkewPair[]
  maxDriftMs: number
}

/** Cross-service parent/child clocks must disagree by this much before warning. */
export const TRACE_SKEW_THRESHOLD_MS = 50
const TRACE_SKEW_THRESHOLD_NS = BigInt(TRACE_SKEW_THRESHOLD_MS) * 1_000_000n
const TRACE_ROOT_SKEW_THRESHOLD_NS = 5n * 60n * 1_000_000_000n

interface SpanTimes {
  start: bigint
  end: bigint
  duration: bigint
}

/** Parse each span's timestamps once and index by spanId. */
function buildTimesMap(
  spans: readonly TraceTreeSpan[]
): Map<string, SpanTimes> {
  const times = new Map<string, SpanTimes>()
  for (const span of spans) {
    const start = BigInt(span.tsNanos)
    const rawDuration = BigInt(span.durationNs)
    const duration = rawDuration > 0n ? rawDuration : 0n
    times.set(span.spanId, {
      start,
      duration,
      end: start + duration,
    })
  }
  return times
}

function compareByStartWithTimes<T extends TraceTreeSpan>(
  times: Map<string, SpanTimes>,
  a: T,
  b: T
): number {
  const aStart = times.get(a.spanId)?.start ?? 0n
  const bStart = times.get(b.spanId)?.start ?? 0n
  const byStart = aStart - bStart
  if (byStart < 0n) return -1
  if (byStart > 0n) return 1
  return a.spanId.localeCompare(b.spanId)
}

/** Order spans depth-first by parent so the waterfall reads top-to-bottom. */
export function orderSpans<T extends TraceTreeSpan>(
  spans: readonly T[],
  times = buildTimesMap(spans)
): Array<{ span: T; depth: number }> {
  const byId = new Map(spans.map((span) => [span.spanId, span]))
  const children = new Map<string, T[]>()
  const roots: T[] = []

  for (const span of spans) {
    if (span.parentSpanId && byId.has(span.parentSpanId)) {
      const list = children.get(span.parentSpanId) ?? []
      list.push(span)
      children.set(span.parentSpanId, list)
    } else {
      roots.push(span)
    }
  }

  const ordered: Array<{ span: T; depth: number }> = []
  const walk = (span: T, depth: number) => {
    ordered.push({ span, depth })
    for (const child of (children.get(span.spanId) ?? []).sort((a, b) =>
      compareByStartWithTimes(times, a, b)
    )) {
      walk(child, depth + 1)
    }
  }

  for (const root of roots.sort((a, b) =>
    compareByStartWithTimes(times, a, b)
  )) {
    walk(root, 0)
  }

  return ordered
}

/** Trace-relative window: absolute start (ns) and total duration (ns, min 1). */
export function computeWindow(
  spans: readonly TraceTreeSpan[],
  times = buildTimesMap(spans)
): TraceWindow {
  if (spans.length === 0) return { startNs: 0n, durationNs: 1n }
  let start = times.get(spans[0]!.spanId)?.start ?? 0n
  let end = times.get(spans[0]!.spanId)?.end ?? 0n
  for (let i = 1; i < spans.length; i += 1) {
    const t = times.get(spans[i]!.spanId)
    if (!t) continue
    if (t.start < start) start = t.start
    if (t.end > end) end = t.end
  }
  const duration = end - start
  return { startNs: start, durationNs: duration > 0n ? duration : 1n }
}

export function positionPct(
  startNs: string | bigint,
  durationNs: string | bigint,
  window: TraceWindow
): { offsetPct: number; widthPct: number } {
  const start = typeof startNs === "bigint" ? startNs : BigInt(startNs)
  const duration =
    typeof durationNs === "bigint" ? durationNs : BigInt(durationNs)
  const windowDuration = window.durationNs > 0n ? window.durationNs : 1n
  const offsetRatio = Number(start - window.startNs) / Number(windowDuration)
  const durationRatio =
    Number(duration > 0n ? duration : 0n) / Number(windowDuration)
  const offsetPct = Math.min(100, Math.max(0, offsetRatio * 100))
  const rawWidth = Math.max(durationRatio * 100, 1.5)
  return {
    offsetPct,
    widthPct: Math.min(rawWidth, Math.max(100 - offsetPct, 0)),
  }
}

export function buildTraceTree<T extends TraceTreeSpan>(
  spans: readonly T[]
): Array<OrderedTraceSpan<T>> {
  const times = buildTimesMap(spans)
  const window = computeWindow(spans, times)
  return orderSpans(spans, times).map(({ span, depth }) => {
    const t = times.get(span.spanId)
    return {
      span,
      depth,
      ...positionPct(t?.start ?? 0n, t?.duration ?? 0n, window),
    }
  })
}

export function errorPathSpanIds<T extends ErrorTraceSpan>(
  spans: readonly T[]
): Set<string> {
  const byId = new Map(spans.map((span) => [span.spanId, span]))
  const ids = new Set<string>()

  for (const span of spans) {
    if (span.statusCode !== "STATUS_CODE_ERROR") continue

    let current: T | undefined = span
    // Shared `ids` also terminates parent-chain cycles (self-parent / A↔B).
    while (current && !ids.has(current.spanId)) {
      ids.add(current.spanId)
      current = current.parentSpanId
        ? byId.get(current.parentSpanId)
        : undefined
    }
  }

  return ids
}

export function groupByService<T extends ServiceTraceSpan>(
  ordered: readonly OrderedTraceSpan<T>[]
): Array<{ service: string; spans: Array<OrderedTraceSpan<T>> }> {
  const groups: Array<{ service: string; spans: Array<OrderedTraceSpan<T>> }> =
    []

  for (const row of ordered) {
    const service = row.span.service || "unknown"
    const current = groups[groups.length - 1]
    if (current?.service === service) {
      current.spans.push(row)
    } else {
      groups.push({ service, spans: [row] })
    }
  }

  return groups
}

function nsToMs(value: bigint): number {
  return Number(value) / 1_000_000
}

export function detectSkew<T extends ServiceTraceSpan>(
  spans: readonly T[]
): SkewReport {
  const times = buildTimesMap(spans)
  const byId = new Map(spans.map((span) => [span.spanId, span]))
  const suspectPairs: SkewPair[] = []
  let maxDriftMs = 0

  for (const child of spans) {
    if (!child.parentSpanId) continue
    const parent = byId.get(child.parentSpanId)
    if (!parent) continue

    const parentTimes = times.get(parent.spanId)
    const childTimes = times.get(child.spanId)
    if (!parentTimes || !childTimes) continue

    const startsBeforeParent = parentTimes.start - childTimes.start
    const endsAfterParent = childTimes.end - parentTimes.end
    const driftNs =
      startsBeforeParent > endsAfterParent
        ? startsBeforeParent
        : endsAfterParent
    const thresholdNs =
      parent.service === child.service
        ? TRACE_ROOT_SKEW_THRESHOLD_NS
        : TRACE_SKEW_THRESHOLD_NS
    if (driftNs <= thresholdNs) continue

    const driftMs = nsToMs(driftNs)
    maxDriftMs = Math.max(maxDriftMs, driftMs)
    suspectPairs.push({
      parentId: parent.spanId,
      childId: child.spanId,
      driftMs,
    })
  }

  const roots = spans
    .filter(
      (span) => !span.parentSpanId || !byId.has(span.parentSpanId)
    )
    .sort((a, b) => compareByStartWithTimes(times, a, b))
  for (let index = 0; index < roots.length - 1; index += 1) {
    const earlier = roots[index]!
    const later = roots[index + 1]!
    const earlierTimes = times.get(earlier.spanId)
    const laterTimes = times.get(later.spanId)
    if (!earlierTimes || !laterTimes) continue
    const gapNs = laterTimes.start - earlierTimes.end
    if (gapNs <= TRACE_ROOT_SKEW_THRESHOLD_NS) continue

    const driftMs = nsToMs(gapNs)
    maxDriftMs = Math.max(maxDriftMs, driftMs)
    suspectPairs.push({
      parentId: later.spanId,
      childId: earlier.spanId,
      driftMs,
    })
  }

  return { suspectPairs, maxDriftMs }
}
