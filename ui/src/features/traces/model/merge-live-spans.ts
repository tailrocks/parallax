// Plan 147 — pure, linear, capacity-bounded live span merge.

export interface LiveSpanIdentity {
  readonly spanId: string
  readonly startNanos?: string | null | undefined
  readonly tsNanos?: string | null | undefined
}

export interface MergeLiveSpansResult<T extends LiveSpanIdentity> {
  readonly items: readonly T[]
  readonly duplicates: number
  readonly dropped: number
}

function spanOrderNanos(row: LiveSpanIdentity): bigint {
  const raw = row.startNanos ?? row.tsNanos ?? "0"
  try {
    return BigInt(raw)
  } catch {
    return 0n
  }
}

/**
 * Prepend newest-first live spans onto existing list. Identity = spanId.
 * Does not mutate `incoming`.
 */
export function mergeLiveSpans<T extends LiveSpanIdentity>(
  current: readonly T[],
  incoming: readonly T[],
  maxVisible: number
): MergeLiveSpansResult<T> {
  if (incoming.length === 0) {
    return { items: current, duplicates: 0, dropped: 0 }
  }

  const seen = new Set(current.map((row) => row.spanId))
  const orderedIncoming = [...incoming].sort((a, b) => {
    const av = spanOrderNanos(a)
    const bv = spanOrderNanos(b)
    if (av === bv) return 0
    return av < bv ? 1 : -1
  })

  let duplicates = 0
  const fresh: T[] = []
  for (const row of orderedIncoming) {
    if (seen.has(row.spanId)) {
      duplicates += 1
      continue
    }
    seen.add(row.spanId)
    fresh.push(row)
  }

  if (fresh.length === 0) {
    return { items: current, duplicates, dropped: 0 }
  }

  const merged = [...fresh, ...current]
  if (merged.length <= maxVisible) {
    return { items: merged, duplicates, dropped: 0 }
  }
  const dropped = merged.length - maxVisible
  return { items: merged.slice(0, maxVisible), duplicates, dropped }
}
