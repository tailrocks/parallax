// Plan 147 — pure, linear, capacity-bounded live log merge.
// Does not mutate incoming arrays; preserves references for unchanged items.

export interface LiveLogIdentity {
  readonly tsNanos: string
  readonly body: string
  readonly service?: string | null | undefined
  /** Preferred severity label (LogDoc.severityText). */
  readonly severityText?: string | null | undefined
  /** Legacy alias accepted for fixtures; prefer severityText. */
  readonly severity?: string | null | undefined
  readonly spanId?: string | null | undefined
  readonly traceId?: string | null | undefined
}

export interface MergeLiveLogsResult<T extends LiveLogIdentity> {
  readonly items: readonly T[]
  readonly duplicates: number
  readonly dropped: number
}

function logIdentityKey(row: LiveLogIdentity): string {
  // Collision-resistant domain key: timestamp + body + service + severity +
  // optional span/trace when the wire contract provides them.
  const severity = row.severityText ?? row.severity ?? ""
  return (
    row.tsNanos +
    "\0" +
    row.body +
    "\0" +
    (row.service ?? "") +
    "\0" +
    severity +
    "\0" +
    (row.spanId ?? "") +
    "\0" +
    (row.traceId ?? "")
  )
}

function compareNanosNewestFirst(a: string, b: string): number {
  if (a.length === b.length) {
    if (a === b) return 0
    return a < b ? 1 : -1
  }
  // Rare unequal digit length — numeric compare.
  const av = BigInt(a)
  const bv = BigInt(b)
  if (av === bv) return 0
  return av < bv ? 1 : -1
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
  for (let i = 0; i < current.length; i += 1) {
    seen.add(logIdentityKey(current[i]!))
  }

  // Sort a shallow copy only; never mutate caller's incoming.
  const orderedIncoming = incoming.slice()
  orderedIncoming.sort((a, b) => compareNanosNewestFirst(a.tsNanos, b.tsNanos))

  let duplicates = 0
  const fresh: T[] = []
  for (let i = 0; i < orderedIncoming.length; i += 1) {
    const row = orderedIncoming[i]!
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

  const total = fresh.length + current.length
  if (total <= maxVisible) {
    return { items: fresh.concat(current as T[]), duplicates, dropped: 0 }
  }

  const dropped = total - maxVisible
  // Prefer fresh (newest) then fill remaining capacity from current head.
  if (fresh.length >= maxVisible) {
    return { items: fresh.slice(0, maxVisible), duplicates, dropped }
  }
  const keepFromCurrent = maxVisible - fresh.length
  return {
    items: fresh.concat((current as T[]).slice(0, keepFromCurrent)),
    duplicates,
    dropped,
  }
}
