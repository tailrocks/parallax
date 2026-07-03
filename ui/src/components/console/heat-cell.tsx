import { cn } from "@/lib/utils"

const bucketClass = [
  "text-green-600 dark:text-green-400",
  "text-emerald-600 dark:text-emerald-400",
  "text-amber-600 dark:text-amber-400",
  "text-orange-600 dark:text-orange-400",
  "text-red-600 dark:text-red-400",
]

export function percentileBucket(value: number, values: number[]) {
  const sorted = values.filter(Number.isFinite).sort((a, b) => a - b)
  if (!sorted.length) return 0
  const rank = sorted.findIndex((candidate) => value <= candidate)
  const pct = (rank < 0 ? sorted.length - 1 : rank) / Math.max(1, sorted.length - 1)
  return Math.min(4, Math.floor(pct * 5))
}

export function HeatCell({
  value,
  values,
  children = value == null ? "-" : value,
}: {
  value: number | null | undefined
  values: number[]
  children?: React.ReactNode
}) {
  if (value == null || !Number.isFinite(value)) {
    return <span className="text-muted-foreground">-</span>
  }
  const bucket = percentileBucket(value, values)
  return (
    <span
      title={`Quintile ${bucket + 1}`}
      className={cn("font-mono tabular-nums", bucketClass[bucket])}
    >
      {children}
    </span>
  )
}
