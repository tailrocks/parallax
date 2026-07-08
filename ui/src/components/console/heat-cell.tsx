import { cn } from "@/lib/utils"

const bucketClass = [
  "text-green-600 dark:text-green-400",
  "text-emerald-600 dark:text-emerald-400",
  "text-amber-600 dark:text-amber-400",
  "text-orange-600 dark:text-orange-400",
  "text-red-600 dark:text-red-400",
]

export type HeatScale = readonly number[]

export function buildHeatScale(values: readonly number[]): HeatScale {
  return values.filter(Number.isFinite).sort((a, b) => a - b)
}

export function percentileBucket(value: number, scale: HeatScale) {
  if (!scale.length) return 0
  let low = 0
  let high = scale.length - 1
  let rank = scale.length - 1
  while (low <= high) {
    const mid = Math.floor((low + high) / 2)
    if (value <= scale[mid]!) {
      rank = mid
      high = mid - 1
    } else {
      low = mid + 1
    }
  }
  const pct = rank / Math.max(1, scale.length - 1)
  return Math.min(4, Math.floor(pct * 5))
}

export function HeatCell({
  value,
  scale,
  children = value == null ? "-" : value,
}: {
  value: number | null | undefined
  scale: HeatScale
  children?: React.ReactNode
}) {
  if (value == null || !Number.isFinite(value)) {
    return <span className="text-muted-foreground">-</span>
  }
  const bucket = percentileBucket(value, scale)
  return (
    <span
      title={`Quintile ${bucket + 1}`}
      className={cn("font-mono tabular-nums", bucketClass[bucket])}
    >
      {children}
    </span>
  )
}
