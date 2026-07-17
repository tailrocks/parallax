/** Three-axis semantic color helpers (plan 162).
 *
 * Axis rules (never repurpose):
 * - Severity ramp: log severity / error state ONLY, always paired with the
 *   literal severity word (color is never the sole signal).
 * - Service palette: service IDENTITY only — never sentiment, state, or a
 *   metric. Deterministic: the same service name yields the same color on
 *   every page, every session.
 * - Percentile tokens (`--chart-p50/p95/p99/error/throughput`): latency/RED
 *   charts only; generic `--chart-1..5` serve arbitrary series.
 */

export type SeverityToken = "trace" | "debug" | "info" | "warn" | "error" | "fatal"

const SEVERITY_ALIASES: Record<string, SeverityToken> = {
  trace: "trace",
  debug: "debug",
  info: "info",
  information: "info",
  warn: "warn",
  warning: "warn",
  error: "error",
  err: "error",
  fatal: "fatal",
  critical: "fatal",
  panic: "fatal",
}

/** Normalize a severity-ish label onto the six-token ramp, or null. */
export function severityToken(label: string): SeverityToken | null {
  return SEVERITY_ALIASES[label.trim().toLowerCase()] ?? null
}

/** CSS color for a severity token (var reference; themes resolve it). */
export function severityColor(token: SeverityToken): string {
  return `var(--severity-${token})`
}

/** FNV-1a over the service name: cheap, stable across sessions/pages. */
function fnv1a(value: string): number {
  let hash = 0x811c9dc5
  for (let index = 0; index < value.length; index += 1) {
    hash ^= value.charCodeAt(index)
    hash = Math.imul(hash, 0x01000193)
  }
  return hash >>> 0
}

export interface ServiceColor {
  /** Hue in degrees [0, 360). */
  hue: number
  /** Solid identity color (dot fill, node stroke). */
  color: string
  /** Soft tint for backgrounds behind the identity color. */
  background: string
}

/** Deterministic service identity color: the name hashes onto one of 48
 * slots — 24 hues spaced 15° apart × 2 lightness tiers — so near-hash
 * neighbors stay visually distinguishable (raw hash-mod-360 produced
 * checkout=121° vs pricing=117°, unreadably close). Same service = same
 * color, everywhere. */
export function serviceColor(name: string): ServiceColor {
  // 120 slots: 24 hues spaced 15° × 5 lightness/chroma tiers.
  const slot = fnv1a(name.trim().toLowerCase()) % 120
  const hue = (slot % 24) * 15
  const tier = Math.floor(slot / 24)
  const lightness = [0.65, 0.55, 0.72, 0.48, 0.6][tier] ?? 0.65
  const chroma = [0.14, 0.14, 0.12, 0.13, 0.17][tier] ?? 0.14
  return {
    hue,
    color: `oklch(${lightness} ${chroma} ${hue})`,
    background: `oklch(${lightness} ${chroma} ${hue} / 12%)`,
  }
}

/** Golden-angle fallback for arbitrary series: index → hue. Any series
 * count stays pairwise distinguishable (137.508° between neighbors). */
export function goldenAngleColor(index: number): string {
  const hue = (index * 137.508) % 360
  return `oklch(0.65 0.13 ${hue.toFixed(2)})`
}

const HTTP_METHODS = new Set(["get", "post", "put", "patch", "delete", "head", "options"])

/** Semantic series-name detection: series named like severities, HTTP
 * methods, status classes, or span status get a stable semantic color;
 * everything else falls back to golden-angle rotation by index. */
export function seriesColor(name: string, index: number): string {
  const normalized = name.trim().toLowerCase()
  const severity = severityToken(normalized)
  if (severity) return severityColor(severity)
  if (/^[1-5]xx$/.test(normalized)) {
    const cls = Number(normalized[0])
    if (cls === 5) return "var(--chart-error)"
    if (cls === 4) return "var(--chart-p95)"
    return "var(--chart-p50)"
  }
  if (normalized === "ok" || normalized === "success") {
    return "var(--severity-info)"
  }
  if (normalized === "p50") return "var(--chart-p50)"
  if (normalized === "p95") return "var(--chart-p95)"
  if (normalized === "p99") return "var(--chart-p99)"
  if (normalized === "throughput" || normalized === "requests") {
    return "var(--chart-throughput)"
  }
  if (normalized === "errors" || normalized === "failed") {
    return "var(--chart-error)"
  }
  if (HTTP_METHODS.has(normalized)) {
    return goldenAngleColor([...HTTP_METHODS].indexOf(normalized))
  }
  return goldenAngleColor(index)
}
