// Plan 147 — pure, linear, capacity-bounded live log merge.
// Does not mutate incoming arrays; preserves references for unchanged items.

export interface LiveLogIdentity {
  readonly tsNanos: string
  readonly body: string
  readonly service?: string | null | undefined
  readonly severity?: string | null | undefined
}

export interface MergeLiveLogsResult<T extends LiveLogIdentity> {
  readonly items: readonly T[]
  readonly duplicates: number
  readonly dropped: number
}

function logIdentityKey(row: LiveLogIdentity): string {
  // Collision-resistant domain key: timestamp + body + service + severity.
  return `${row.tsNanos}\0${row.body}\0${row.service ?? ""}\0${row.severity ?? ""}`
}

/**
 * Prepend newest-first live batch onto existing newest-first list.
 * `incoming` is treated as an unordered batch from the stream; it is ordered
 * newest-first without mutating the caller's array.
 */
export function mergeLiveLogs<T extends LiveLogIdentity>(
  current: readonly T[],
  incoming: readonly T[],
  maxVisible: number
): MergeLiveLogsResult<T> {
  if (incoming.length === 0) {
    return { items: current, duplicates: 0, dropped: 0 }
  }

  const seen = new Set<string>()
  for (const row of current) {
    seen.add(logIdentityKey(row))
  }

  const orderedIncoming = [...incoming].sort((a, b) => {
    const av = BigInt(a.tsNanos)
    const bv = BigInt(b.tsNanos)
    if (av === bv) return 0
    return av < bv ? 1 : -1
  })

  let duplicates = 0
  const fresh: T[] = []
  for (const row of orderedIncoming) {
    const key = logIdentityKey(row)
    if (seen.has(key)) {
      duplicates += 1
      continue
    }
    seen.add(key)
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
