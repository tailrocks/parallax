import { useEffect, useRef, useState } from "react"
import { Area, AreaChart } from "recharts"
import { IconCircleArrowDownFilled, IconCircleArrowUpFilled } from "@tabler/icons-react"

import type { Delta } from "@/shared/format"
import { cn } from "@/lib/utils"
import { Badge } from "@/components/ui/badge"
import { Card, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { ChartContainer, ChartTooltip, ChartTooltipContent } from "@/components/ui/chart"
import type { ChartConfig } from "@/components/ui/chart"

function prefersReducedMotion(): boolean {
  return (
    typeof window !== "undefined" && window.matchMedia("(prefers-reduced-motion: reduce)").matches
  )
}

function parseDisplayNumber(value: string): { n: number; prefix: string; suffix: string } | null {
  const match = /^(\D*?)(-?\d[\d,]*(?:\.\d+)?)(.*)$/.exec(value)
  if (!match) return null
  const n = Number(match[2]!.replace(/,/g, ""))
  if (!Number.isFinite(n)) return null
  return { n, prefix: match[1] ?? "", suffix: match[3] ?? "" }
}

function formatTicker(from: { prefix: string; suffix: string }, n: number): string {
  const rounded = Number.isInteger(n) ? String(Math.round(n)) : n.toFixed(1)
  return `${from.prefix}${rounded}${from.suffix}`
}

function Ticker({ value }: { value: React.ReactNode }) {
  const text = typeof value === "string" || typeof value === "number" ? String(value) : null
  const [display, setDisplay] = useState(text ?? "")
  const previous = useRef(text)

  useEffect(() => {
    if (text == null) return
    const prior = previous.current
    previous.current = text
    if (prior === text || prior == null || prefersReducedMotion()) {
      setDisplay(text)
      return
    }
    const start = parseDisplayNumber(prior)
    const end = parseDisplayNumber(text)
    if (!start || !end) {
      setDisplay(text)
      return
    }
    const duration = 150
    const origin = performance.now()
    let frame = 0
    const tick = (now: number) => {
      const t = Math.min(1, (now - origin) / duration)
      const eased = 1 - (1 - t) * (1 - t)
      setDisplay(formatTicker(end, start.n + (end.n - start.n) * eased))
      if (t < 1) frame = requestAnimationFrame(tick)
    }
    frame = requestAnimationFrame(tick)
    return () => cancelAnimationFrame(frame)
  }, [text])

  if (text == null) return <>{value}</>
  return <span className="tabular-nums">{display}</span>
}

export function DeltaBadge({
  delta,
  inverted = false,
}: {
  delta?: Delta | null
  inverted?: boolean
}) {
  if (!delta || delta.dir === "flat") {
    return <Badge variant="secondary">~0%</Badge>
  }
  const good = inverted ? delta.dir === "down" : delta.dir === "up"
  const Icon = delta.dir === "up" ? IconCircleArrowUpFilled : IconCircleArrowDownFilled
  return (
    <Badge variant={good ? "emerald" : "rose"}>
      <Icon />
      {delta.pct.toFixed(delta.pct < 10 ? 1 : 0)}%
    </Badge>
  )
}

/** Order stat cards by the canonical story: Volume -> Health -> Performance -> Cost. */
export function StatCard({
  label,
  value,
  hint,
  delta,
  deltaInverted = false,
  icon: Icon,
  iconClassName,
  chart,
}: {
  label: string
  value: React.ReactNode
  hint?: React.ReactNode
  delta?: Delta | null
  deltaInverted?: boolean
  icon?: React.ComponentType<{ className?: string }>
  iconClassName?: string
  chart?: React.ReactNode
}) {
  return (
    <Card size="sm" className="min-h-32">
      <CardHeader className="gap-1.5">
        <div className="flex items-center justify-between gap-1.5">
          <div className="flex items-center gap-1.5">
            {Icon ? (
              <Icon className={cn("size-[13px] shrink-0 text-muted-foreground", iconClassName)} />
            ) : null}
            <CardDescription>{label}</CardDescription>
          </div>
          {delta ? <DeltaBadge delta={delta} inverted={deltaInverted} /> : null}
        </div>
        <div className="flex items-baseline justify-between gap-2">
          <CardTitle className="tracking-tight tabular-nums">
            <Ticker value={value} />
          </CardTitle>
          {hint ? (
            <span className="min-w-0 truncate text-end text-xs text-muted-foreground/70">
              {hint}
            </span>
          ) : null}
        </div>
      </CardHeader>
      {chart ? <div className="mt-auto -mb-5">{chart}</div> : null}
    </Card>
  )
}

const sparkConfig = { value: { color: "currentColor" } } satisfies ChartConfig

export function CardSparkline({
  data,
  className,
}: {
  data: Array<{ value: number }>
  className?: string
}) {
  const complete = data.map((point, index) => ({
    value: index < data.length - 1 ? point.value : null,
  }))
  const tail = data.map((point, index) => ({
    value: index >= data.length - 2 ? point.value : null,
  }))
  const dashed = data.length >= 2
  return (
    <ChartContainer config={sparkConfig} className={cn("h-12 w-full", className)}>
      <AreaChart data={data} margin={{ left: 0, right: 0, top: 4, bottom: 0 }}>
        <defs>
          <linearGradient id="sparkline-fill" x1="0" y1="0" x2="0" y2="1">
            <stop offset="5%" stopColor="currentColor" stopOpacity={0.25} />
            <stop offset="95%" stopColor="currentColor" stopOpacity={0} />
          </linearGradient>
        </defs>
        <ChartTooltip content={<ChartTooltipContent />} />
        <Area
          data={dashed ? complete : data}
          dataKey="value"
          type="monotone"
          stroke="currentColor"
          fill="url(#sparkline-fill)"
          strokeWidth={1.5}
          dot={false}
          connectNulls={false}
        />
        {dashed ? (
          <Area
            data={tail}
            dataKey="value"
            type="monotone"
            stroke="currentColor"
            fill="none"
            strokeWidth={1.5}
            strokeDasharray="3 3"
            dot={false}
            connectNulls
          />
        ) : null}
      </AreaChart>
    </ChartContainer>
  )
}

export function PillMeter({ value, segments = 12 }: { value: number; segments?: number }) {
  const filled = Math.round(Math.max(0, Math.min(1, value)) * segments)
  return (
    <div className="flex h-2 gap-0.5 rounded-full bg-muted p-0.5">
      {Array.from({ length: segments }, (_, index) => (
        <span
          key={index}
          className={cn("flex-1 rounded-full", index < filled ? "bg-primary" : "bg-transparent")}
        />
      ))}
    </div>
  )
}
