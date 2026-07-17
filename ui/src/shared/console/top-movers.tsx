import { Link } from "@tanstack/react-router"
import { IconArrowDownRight, IconArrowUpRight, IconSparkles } from "@tabler/icons-react"
import { useMemo } from "react"

import { Badge } from "@/components/ui/badge"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import { formatPercent } from "@/shared/format"
import { cn } from "@/lib/utils"

type RangeLink = {
  key: string
  fromNanos: string
  toNanos: string
}

function rangeLinkSearch(range: RangeLink): {
  range?: string
  from?: string
  to?: string
} {
  if (range.key !== "custom") return { range: range.key }
  return { range: "custom", from: range.fromNanos, to: range.toNanos }
}

/** Error-rate deltas at or above 2 percentage points become movers. */
export const ERROR_RATE_DELTA_THRESHOLD = 0.02
/** p95 latency at or above 1.5x the previous window becomes a mover. */
export const LATENCY_RATIO_THRESHOLD = 1.5
/** Span volume at or above 2x/0.5x the previous window becomes a mover. */
export const VOLUME_RATIO_THRESHOLD = 2

export interface ServiceMoverInput {
  name: string
  spanCount: string
  errorCount: string
  p95Ms?: number | null
}

export type MoverKind = "error" | "latency" | "volume" | "new"
export type MoverDirection = "up" | "down" | "new"

export interface Mover {
  service: string
  kind: MoverKind
  direction: MoverDirection
  score: number
  current: ServiceStats
  previous: ServiceStats | null
}

interface ServiceStats {
  spanCount: number
  errorCount: number
  errorRate: number
  p95Ms: number | null
}

function numberFromCount(value: string): number {
  const parsed = Number(value)
  return Number.isFinite(parsed) ? parsed : 0
}

function serviceStats(row: ServiceMoverInput): ServiceStats {
  const spanCount = numberFromCount(row.spanCount)
  const errorCount = numberFromCount(row.errorCount)
  const p95Ms = row.p95Ms ?? null
  return {
    spanCount,
    errorCount,
    errorRate: spanCount > 0 ? errorCount / spanCount : 0,
    p95Ms: p95Ms != null && Number.isFinite(p95Ms) ? p95Ms : null,
  }
}

function volumeRatio(current: number, previous: number) {
  if (current <= 0 || previous <= 0) return null
  return current >= previous
    ? { direction: "up" as const, ratio: current / previous }
    : { direction: "down" as const, ratio: previous / current }
}

function rankFor(kind: MoverKind) {
  switch (kind) {
    case "error":
      return 0
    case "latency":
      return 1
    case "new":
    case "volume":
      return 2
  }
}

export function computeMovers(
  now: readonly ServiceMoverInput[],
  previous: readonly ServiceMoverInput[]
): Mover[] {
  const previousByName = new Map(previous.map((row) => [row.name, row]))
  const movers: Mover[] = []

  for (const row of now) {
    const current = serviceStats(row)
    const prevRow = previousByName.get(row.name)
    if (!prevRow) {
      if (current.spanCount > 0) {
        movers.push({
          service: row.name,
          kind: "new",
          direction: "new",
          score: Number.MAX_SAFE_INTEGER,
          current,
          previous: null,
        })
      }
      continue
    }

    const prev = serviceStats(prevRow)
    const errorDelta = current.errorRate - prev.errorRate
    if (Math.abs(errorDelta) >= ERROR_RATE_DELTA_THRESHOLD) {
      movers.push({
        service: row.name,
        kind: "error",
        direction: errorDelta >= 0 ? "up" : "down",
        score: Math.abs(errorDelta),
        current,
        previous: prev,
      })
      continue
    }

    if (
      current.p95Ms != null &&
      prev.p95Ms != null &&
      prev.p95Ms > 0 &&
      current.p95Ms / prev.p95Ms >= LATENCY_RATIO_THRESHOLD
    ) {
      movers.push({
        service: row.name,
        kind: "latency",
        direction: "up",
        score: current.p95Ms / prev.p95Ms,
        current,
        previous: prev,
      })
      continue
    }

    const volume = volumeRatio(current.spanCount, prev.spanCount)
    if (volume && volume.ratio >= VOLUME_RATIO_THRESHOLD) {
      movers.push({
        service: row.name,
        kind: "volume",
        direction: volume.direction,
        score: volume.ratio,
        current,
        previous: prev,
      })
    }
  }

  return movers
    .sort((a, b) => {
      const rank = rankFor(a.kind) - rankFor(b.kind)
      if (rank !== 0) return rank
      if (b.score !== a.score) return b.score - a.score
      return a.service.localeCompare(b.service)
    })
    .slice(0, 6)
}

function formatRatio(value: number) {
  return `${value >= 10 ? value.toFixed(0) : value.toFixed(1)}x`
}

export function moverSentence(mover: Mover): string {
  if (mover.kind === "new") return `new service: ${mover.service}`
  if (mover.kind === "error") {
    return `${mover.service} error rate ${formatPercent(
      mover.previous?.errorRate ?? 0
    )} -> ${formatPercent(mover.current.errorRate)}`
  }
  if (mover.kind === "latency") {
    return `${mover.service} p95 up ${formatRatio(mover.score)}`
  }
  return `${mover.service} volume ${mover.direction} ${formatRatio(mover.score)}`
}

function MoverIcon({ mover }: { mover: Mover }) {
  const Icon =
    mover.kind === "new"
      ? IconSparkles
      : mover.direction === "down"
        ? IconArrowDownRight
        : IconArrowUpRight
  return (
    <span
      className={cn(
        "flex size-8 shrink-0 items-center justify-center rounded-full bg-muted text-muted-foreground",
        mover.kind === "new" && "bg-sky-500/10 text-sky-600",
        mover.direction === "up" &&
          mover.kind !== "new" &&
          "bg-amber-500/10 text-amber-700 dark:text-amber-400",
        mover.direction === "down" && "bg-emerald-500/10 text-emerald-700"
      )}
    >
      <Icon className="size-4" />
    </span>
  )
}

export function TopMovers({
  now,
  previous,
  range,
}: {
  now: ServiceMoverInput[]
  previous: ServiceMoverInput[]
  range: RangeLink
}) {
  const movers = useMemo(() => computeMovers(now, previous), [now, previous])

  return (
    <Card size="sm">
      <CardHeader className="flex-row items-center justify-between gap-2">
        <CardTitle>What changed</CardTitle>
        <Badge variant="secondary">top movers</Badge>
      </CardHeader>
      <CardContent>
        {movers.length === 0 ? (
          <p className="text-sm text-muted-foreground">
            Nothing moved more than the thresholds in this window.
          </p>
        ) : (
          <div className="grid gap-2 md:grid-cols-2 xl:grid-cols-3">
            {movers.map((mover) => (
              <Link
                key={`${mover.kind}-${mover.service}`}
                to="/services/$service"
                params={{ service: mover.service }}
                search={rangeLinkSearch(range)}
                className="flex min-w-0 items-center gap-3 rounded-lg border border-border/70 bg-background/50 p-3 text-sm transition outline-none hover:bg-muted/40 focus-visible:ring-[1.5px] focus-visible:ring-ring/50"
              >
                <MoverIcon mover={mover} />
                <span className="min-w-0">
                  <span className="block truncate font-medium">{mover.service}</span>
                  <span className="block truncate text-muted-foreground">
                    {moverSentence(mover)}
                  </span>
                </span>
              </Link>
            ))}
          </div>
        )}
      </CardContent>
    </Card>
  )
}
