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

function spanStart(span: TraceTreeSpan): bigint {
  return BigInt(span.tsNanos)
}

function spanDuration(span: TraceTreeSpan): bigint {
  const duration = BigInt(span.durationNs)
  return duration > 0n ? duration : 0n
}

function spanEnd(span: TraceTreeSpan): bigint {
  return spanStart(span) + spanDuration(span)
}

function compareByStart<T extends TraceTreeSpan>(a: T, b: T): number {
  const byStart = spanStart(a) - spanStart(b)
  if (byStart < 0n) return -1
  if (byStart > 0n) return 1
  return a.spanId.localeCompare(b.spanId)
}

/** Order spans depth-first by parent so the waterfall reads top-to-bottom. */
export function orderSpans<T extends TraceTreeSpan>(
  spans: readonly T[]
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
    for (const child of (children.get(span.spanId) ?? []).sort(
      compareByStart
    )) {
      walk(child, depth + 1)
    }
  }

  for (const root of roots.sort(compareByStart)) {
    walk(root, 0)
  }

  return ordered
}

/** Trace-relative window: absolute start (ns) and total duration (ns, min 1). */
export function computeWindow(spans: readonly TraceTreeSpan[]): TraceWindow {
  if (spans.length === 0) return { startNs: 0n, durationNs: 1n }
  let start = spanStart(spans[0]!)
  let end = spanEnd(spans[0]!)
  for (const span of spans.slice(1)) {
    const spanStartNs = spanStart(span)
    const spanEndNs = spanEnd(span)
    if (spanStartNs < start) start = spanStartNs
    if (spanEndNs > end) end = spanEndNs
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
  const window = computeWindow(spans)
  return orderSpans(spans).map(({ span, depth }) => ({
    span,
    depth,
    ...positionPct(span.tsNanos, span.durationNs, window),
  }))
}
