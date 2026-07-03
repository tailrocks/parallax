import { z } from "zod"

export const RANGE_PRESETS = [
  { key: "1h", label: "Last hour", ms: 60 * 60 * 1000 },
  { key: "24h", label: "Last 24h", ms: 24 * 60 * 60 * 1000 },
  { key: "7d", label: "Last 7d", ms: 7 * 24 * 60 * 60 * 1000 },
  { key: "30d", label: "Last 30d", ms: 30 * 24 * 60 * 60 * 1000 },
  { key: "90d", label: "Last 90d", ms: 90 * 24 * 60 * 60 * 1000 },
] as const

export const DEFAULT_RANGE_KEY = "24h"
export const rangeSearchSchema = z.object({
  range: z.string().optional(),
  from: z.string().optional(),
  to: z.string().optional(),
})

export type ResolvedRange = {
  key: string
  fromNanos: string
  toNanos: string
}

function msToNanos(ms: number) {
  return (BigInt(ms) * 1_000_000n).toString()
}

export function resolvePreset(key = DEFAULT_RANGE_KEY, now = Date.now()): ResolvedRange {
  const preset =
    RANGE_PRESETS.find((candidate) => candidate.key === key) ??
    RANGE_PRESETS.find((candidate) => candidate.key === DEFAULT_RANGE_KEY)!
  return {
    key: preset.key,
    fromNanos: msToNanos(now - preset.ms),
    toNanos: msToNanos(now),
  }
}

export function customRange(fromNanos: string, toNanos: string): ResolvedRange {
  return { key: "custom", fromNanos, toNanos }
}

export function resolveRangeSearch(
  search: z.input<typeof rangeSearchSchema>,
  now = Date.now(),
): ResolvedRange {
  const parsed = rangeSearchSchema.safeParse(search)
  if (!parsed.success) return resolvePreset(DEFAULT_RANGE_KEY, now)
  if (parsed.data.from && parsed.data.to) {
    try {
      if (BigInt(parsed.data.from) < BigInt(parsed.data.to)) {
        return customRange(parsed.data.from, parsed.data.to)
      }
    } catch {
      return resolvePreset(parsed.data.range, now)
    }
  }
  return resolvePreset(parsed.data.range, now)
}

export function formatRangeLabel(range: ResolvedRange): string {
  const preset = RANGE_PRESETS.find((candidate) => candidate.key === range.key)
  if (preset) return preset.label
  const from = new Date(Number(BigInt(range.fromNanos) / 1_000_000n))
  const to = new Date(Number(BigInt(range.toNanos) / 1_000_000n))
  return `${from.toLocaleDateString()} - ${to.toLocaleDateString()}`
}
