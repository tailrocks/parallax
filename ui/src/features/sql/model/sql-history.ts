export const SQL_HISTORY_KEY = "parallax.sql.history"
export const SQL_HISTORY_CAP = 20

/**
 * Parse history wire value. Absent/malformed/non-array → [].
 * Mixed arrays keep the baseline unchecked-array cast (no silent filter).
 */
export function parseHistoryWire(raw: string | null): string[] {
  if (raw == null) return []
  try {
    const parsed: unknown = JSON.parse(raw)
    return Array.isArray(parsed) ? (parsed as string[]) : []
  } catch {
    return []
  }
}

/** Most-recent-first, de-duplicated, capped at SQL_HISTORY_CAP. */
export function pushHistoryEntry(
  current: readonly string[],
  sql: string
): string[] {
  return [sql, ...current.filter((q) => q !== sql)].slice(0, SQL_HISTORY_CAP)
}

export function serializeHistoryWire(entries: readonly string[]): string {
  return JSON.stringify(entries)
}
