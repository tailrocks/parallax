import type { ResolvedRange } from "@/lib/range"

export function stepSecondsForRange(range: ResolvedRange): number {
  const spanNs = BigInt(range.toNanos) - BigInt(range.fromNanos)
  return Math.max(30, Math.round(Number(spanNs / 1_000_000_000n) / 60))
}

export function contextWindow(
  anchorNanos: string,
  windowSeconds = 30
): ResolvedRange {
  const anchor = BigInt(anchorNanos)
  const width = BigInt(windowSeconds) * 1_000_000_000n
  const from = anchor > width ? anchor - width : 0n
  return {
    key: "custom",
    fromNanos: from.toString(),
    toNanos: (anchor + width).toString(),
  }
}
