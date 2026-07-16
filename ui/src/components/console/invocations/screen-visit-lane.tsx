import { Badge } from "@/components/ui/badge"
import type { ScreenVisit } from "@/lib/api"
import { formatDurationNs } from "@/lib/format"
import { cn } from "@/lib/utils"

/** Gantt-style dwell lane: one row per screen visit, navigation order. */
export function ScreenVisitLane({ visits }: { visits: ScreenVisit[] }) {
  if (visits.length === 0) return null
  const start = BigInt(visits[0]!.enteredNanos)
  const end = visits.reduce((max, visit) => {
    const candidate = BigInt(visit.exitedNanos ?? visit.enteredNanos)
    return candidate > max ? candidate : max
  }, start)
  const total = end > start ? end - start : 1n
  return (
    <div className="space-y-1">
      {visits.map((visit) => {
        const from = BigInt(visit.enteredNanos)
        const to = visit.exitedNanos ? BigInt(visit.exitedNanos) : end
        const leftPct = Number(((from - start) * 1000n) / total) / 10
        const widthPct = Math.max(
          Number(((to - from) * 1000n) / total) / 10,
          1.5
        )
        return (
          <div
            key={visit.visitId}
            className="grid grid-cols-[8rem_minmax(0,1fr)_6rem] items-center gap-2 text-xs"
          >
            <span className="truncate font-medium" title={visit.screenId}>
              {visit.navigationSequence != null ? (
                <span className="mr-1 text-muted-foreground">
                  {visit.navigationSequence}.
                </span>
              ) : null}
              {visit.screenId}
            </span>
            <div className="relative h-4 rounded bg-muted/40">
              <div
                className={cn(
                  "absolute inset-y-0 rounded bg-violet-500/70",
                  visit.exitedNanos == null && "animate-pulse"
                )}
                style={{ left: `${leftPct}%`, width: `${widthPct}%` }}
              />
            </div>
            <span className="text-right text-muted-foreground">
              {visit.exitedNanos != null ? (
                formatDurationNs(
                  (
                    BigInt(visit.exitedNanos) - BigInt(visit.enteredNanos)
                  ).toString()
                )
              ) : (
                <Badge variant="blue">active</Badge>
              )}
            </span>
          </div>
        )
      })}
    </div>
  )
}
