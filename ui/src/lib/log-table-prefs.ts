/** Logs table column / density prefs (plan 165).
 *
 * URL-encoded pinned attribute columns + local density/wrap toggles.
 * Pure — no React/localStorage side effects so unit tests drive the real
 * encode/decode helpers.
 *
 * Preliminary — peer must wire columns control, document-sheet pin action,
 * density class on logs-table, and browser evidence.
 */

export type LogRowDensity = "compact" | "comfortable"

export const LOG_DENSITY_STORAGE_KEY = "parallax.logs.density"
export const LOG_WRAP_STORAGE_KEY = "parallax.logs.wrap"

/** Default density when nothing is stored. */
export const DEFAULT_LOG_DENSITY: LogRowDensity = "comfortable"

/**
 * Decode `?columns=` CSV of attribute keys into a stable unique list.
 * Empty / missing → []. Whitespace trimmed; empty segments dropped.
 */
export function decodePinnedColumns(
  raw: string | null | undefined
): string[] {
  if (!raw) return []
  const seen = new Set<string>()
  const out: string[] = []
  for (const part of raw.split(",")) {
    const key = part.trim()
    if (!key || seen.has(key)) continue
    seen.add(key)
    out.push(key)
  }
  return out
}

/** Encode pinned keys for the `columns` search param (CSV). */
export function encodePinnedColumns(keys: readonly string[]): string {
  return decodePinnedColumns(keys.join(",")).join(",")
}

/** Pin a key (append if absent); returns a new array. */
export function pinColumn(keys: readonly string[], key: string): string[] {
  const k = key.trim()
  if (!k) return [...keys]
  if (keys.includes(k)) return [...keys]
  return [...keys, k]
}

/** Unpin a key; returns a new array. */
export function unpinColumn(keys: readonly string[], key: string): string[] {
  return keys.filter((k) => k !== key)
}

/** Toggle pin for a key. */
export function togglePinnedColumn(
  keys: readonly string[],
  key: string
): string[] {
  return keys.includes(key) ? unpinColumn(keys, key) : pinColumn(keys, key)
}

/** Parse density from storage/string; invalid → default. */
export function parseLogDensity(
  raw: string | null | undefined
): LogRowDensity {
  if (raw === "compact" || raw === "comfortable") return raw
  return DEFAULT_LOG_DENSITY
}

/** CSS class fragment for the density mode (peer applies on table root). */
export function logDensityClass(density: LogRowDensity): string {
  return density === "compact" ? "log-rows-compact" : "log-rows-comfortable"
}

/** Parse wrap preference (`"1"` / `"true"` / `"0"` / `"false"`). */
export function parseLogWrap(raw: string | null | undefined): boolean {
  if (raw == null || raw === "") return false
  const v = raw.toLowerCase()
  return v === "1" || v === "true" || v === "yes"
}

export function encodeLogWrap(wrap: boolean): string {
  return wrap ? "1" : "0"
}
