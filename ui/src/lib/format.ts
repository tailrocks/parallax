export type Delta = { dir: "up" | "down" | "flat"; pct: number }

function toNumber(value: string | number): number {
  return typeof value === "string" ? Number(value) : value
}

function nanosToMs(value: string | number): number {
  if (typeof value === "string") {
    return Number(BigInt(value) / 1_000_000n)
  }
  return value / 1_000_000
}

export function formatDurationNs(value: string | number): string {
  const ns = Math.max(0, toNumber(value))
  if (!Number.isFinite(ns)) return "-"
  if (ns < 1_000_000) return `${Math.round(ns / 1_000)}µs`
  if (ns < 1_000_000_000) return `${(ns / 1_000_000).toFixed(ns < 10_000_000 ? 1 : 0)}ms`
  if (ns < 60_000_000_000) return `${(ns / 1_000_000_000).toFixed(ns < 10_000_000_000 ? 2 : 1)}s`
  const minutes = Math.floor(ns / 60_000_000_000)
  const seconds = Math.round((ns % 60_000_000_000) / 1_000_000_000)
  return `${minutes}m ${seconds}s`
}

export function formatCount(value: number): string {
  if (!Number.isFinite(value)) return "-"
  const abs = Math.abs(value)
  if (abs >= 1_000_000) return `${(value / 1_000_000).toFixed(abs >= 10_000_000 ? 0 : 1)}M`
  if (abs >= 1_000) return `${(value / 1_000).toFixed(abs >= 10_000 ? 0 : 1)}k`
  return String(value)
}

export function formatRelative(value: string | number, now = Date.now()): string {
  const ms = nanosToMs(value)
  if (!Number.isFinite(ms) || ms <= 0) return "-"
  const seconds = Math.max(0, Math.floor((now - ms) / 1000))
  if (seconds < 60) return `${seconds}s ago`
  if (seconds < 3600) return `${Math.floor(seconds / 60)}m ago`
  if (seconds < 86400) return `${Math.floor(seconds / 3600)}h ago`
  return `${Math.floor(seconds / 86400)}d ago`
}

export function formatDateTime(value: string | number): string {
  const date = new Date(nanosToMs(value))
  return new Intl.DateTimeFormat(undefined, {
    month: "short",
    day: "numeric",
    hour: "2-digit",
    minute: "2-digit",
    second: "2-digit",
    hour12: false,
  }).format(date)
}

export function formatTimeInRange(
  value: string | number,
  range: { fromNanos: string; toNanos: string },
): string {
  const windowNs = BigInt(range.toNanos) - BigInt(range.fromNanos)
  const date = new Date(nanosToMs(value))
  if (windowNs <= 86_400_000_000_000n) {
    return new Intl.DateTimeFormat(undefined, {
      hour: "2-digit",
      minute: "2-digit",
      second: "2-digit",
      hour12: false,
    }).format(date)
  }
  return formatDateTime(value)
}

export function formatPercent(value: number): string {
  if (!Number.isFinite(value)) return "-"
  return `${(value * 100).toFixed(Math.abs(value) < 0.1 ? 1 : 0)}%`
}

export function formatDelta(current: number, previous: number): Delta {
  if (!Number.isFinite(current) || !Number.isFinite(previous) || previous === 0) {
    return { dir: "flat", pct: 0 }
  }
  const pct = ((current - previous) / Math.abs(previous)) * 100
  if (Math.abs(pct) < 0.05) return { dir: "flat", pct: 0 }
  return { dir: pct > 0 ? "up" : "down", pct: Math.abs(pct) }
}
