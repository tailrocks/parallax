import { cn } from "@/lib/utils"

export function ChartLegend({
  items,
  selected,
  onSelect,
}: {
  items: Array<{ key: string; label: string; color: string }>
  selected?: string
  onSelect?: (key: string) => void
}) {
  return (
    <div className="flex flex-wrap items-center gap-3">
      {items.map((item) => (
        <button
          key={item.key}
          type="button"
          className={cn("flex items-center gap-1.5 text-xs", selected && selected !== item.key && "opacity-30")}
          onClick={() => onSelect?.(item.key)}
        >
          <span className="h-2 w-2 rounded-2xl corner-squircle" style={{ backgroundColor: item.color }} />
          {item.label}
        </button>
      ))}
    </div>
  )
}

export function thinTicks<T>(ticks: T[], target = 8): T[] {
  if (ticks.length <= target) return ticks
  const step = Math.ceil((ticks.length - 2) / Math.max(1, target - 2))
  return [ticks[0]!, ...ticks.slice(1, -1).filter((_, index) => index % step === 0), ticks.at(-1)!]
}

export function makeEdgeTick(value: string, index: number, ticks: string[]) {
  return index === 0 || index === ticks.length - 1 ? value : ""
}

export function makeBucketLabel(value: string) {
  return value
}

export function dedupeTicks<T>(ticks: T[]): T[] {
  return Array.from(new Set(ticks))
}
