/** Client grouping contract. Authoritative ranking is GraphQL
 * `Trace.dominantDbQueries` (see `traceDetailQuery`). */

export interface DominantDbSpan {
  readonly attributes: string
  readonly durationNs: string
  readonly spanId: string
  readonly service: string
}

export interface DominantDbQuery {
  readonly normalized: string
  readonly example: string
  readonly count: number
  readonly totalNs: bigint
  readonly maxNs: bigint
  readonly exampleSpanId: string
  readonly service: string
}

function parseAttributes(raw: string): Record<string, unknown> {
  try {
    const parsed: unknown = JSON.parse(raw)
    return parsed && typeof parsed === "object" && !Array.isArray(parsed)
      ? (parsed as Record<string, unknown>)
      : {}
  } catch {
    return {}
  }
}

function dbQueryText(attributes: Record<string, unknown>): string | null {
  const value = attributes["db.query.text"] ?? attributes["db.statement"]
  if (typeof value !== "string") return null
  const trimmed = value.trim()
  return trimmed.length > 0 ? trimmed : null
}

/** Match Rust `normalize_message` digits collapse for SQL grouping. */
export function normalizeDbQuery(query: string): string {
  return query.replace(/\d+/g, "<n>").replace(/\s+/g, " ").trim()
}

type MutableDbQuery = { -readonly [K in keyof DominantDbQuery]: DominantDbQuery[K] }

export function dominantDbQueries(spans: readonly DominantDbSpan[], limit = 8): DominantDbQuery[] {
  if (limit <= 0) return []
  const groups = new Map<string, MutableDbQuery>()
  for (const span of spans) {
    const example = dbQueryText(parseAttributes(span.attributes))
    if (!example) continue
    const durationNs = BigInt(span.durationNs || "0")
    const normalized = normalizeDbQuery(example)
    const existing = groups.get(normalized)
    if (!existing) {
      groups.set(normalized, {
        normalized,
        example,
        count: 1,
        totalNs: durationNs,
        maxNs: durationNs,
        exampleSpanId: span.spanId,
        service: span.service,
      })
      continue
    }
    existing.count += 1
    existing.totalNs += durationNs
    if (durationNs > existing.maxNs) {
      existing.maxNs = durationNs
      existing.exampleSpanId = span.spanId
      existing.example = example
      existing.service = span.service
    }
  }
  return [...groups.values()]
    .sort((left, right) => {
      if (right.totalNs !== left.totalNs) return right.totalNs > left.totalNs ? 1 : -1
      if (right.count !== left.count) return right.count - left.count
      return left.normalized.localeCompare(right.normalized)
    })
    .slice(0, limit)
}
