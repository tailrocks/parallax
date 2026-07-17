import { useEffect, useState } from "react"
import { IconClock, IconX } from "@tabler/icons-react"

import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Popover, PopoverContent, PopoverTrigger } from "@/components/ui/popover"
import { formatDurationNs } from "@/shared/format"
import { cn } from "@/lib/utils"

/** Plan 164 duration filter: preset chips seeded from the current result
 * window's duration stats plus debounced min/max ms inputs. Controlled —
 * parent owns the range (URL search params). */
export type DurationRange = {
  minMs?: number
  maxMs?: number
}

export type DurationStats = {
  p50Ms: number
  p95Ms: number
}

const DEBOUNCE_MS = 400

export function durationSummary(range: DurationRange): string | null {
  const { minMs, maxMs } = range
  if (minMs != null && maxMs != null) {
    return `${formatDurationNs(minMs * 1e6)} – ${formatDurationNs(maxMs * 1e6)}`
  }
  if (minMs != null) return `≥ ${formatDurationNs(minMs * 1e6)}`
  if (maxMs != null) return `≤ ${formatDurationNs(maxMs * 1e6)}`
  return null
}

function parseMs(raw: string): number | undefined {
  if (raw.trim() === "") return undefined
  const value = Number(raw)
  return Number.isFinite(value) && value >= 0 ? value : undefined
}

export function DurationFilter({
  range,
  stats,
  onChange,
  className,
}: {
  range: DurationRange
  stats?: DurationStats
  onChange: (next: DurationRange) => void
  className?: string
}) {
  const [minText, setMinText] = useState(range.minMs?.toString() ?? "")
  const [maxText, setMaxText] = useState(range.maxMs?.toString() ?? "")

  useEffect(() => {
    setMinText(range.minMs?.toString() ?? "")
    setMaxText(range.maxMs?.toString() ?? "")
  }, [range.minMs, range.maxMs])

  useEffect(() => {
    const timer = setTimeout(() => {
      const minMs = parseMs(minText)
      const maxMs = parseMs(maxText)
      if (minMs === range.minMs && maxMs === range.maxMs) return
      onChange({
        ...(minMs === undefined ? {} : { minMs }),
        ...(maxMs === undefined ? {} : { maxMs }),
      })
    }, DEBOUNCE_MS)
    return () => clearTimeout(timer)
  }, [minText, maxText, range.minMs, range.maxMs, onChange])

  const summary = durationSummary(range)
  const presets: Array<{ label: string; minMs: number }> = [
    ...(stats
      ? [
          {
            label: `> p50 (${formatDurationNs(stats.p50Ms * 1e6)})`,
            minMs: stats.p50Ms,
          },
          {
            label: `> p95 (${formatDurationNs(stats.p95Ms * 1e6)})`,
            minMs: stats.p95Ms,
          },
        ]
      : []),
    { label: "> 1s", minMs: 1000 },
  ]

  return (
    <Popover>
      <PopoverTrigger
        render={
          <Button
            variant="outline"
            size="sm"
            className={cn("rounded-full", summary && "font-medium", className)}
          />
        }
      >
        <IconClock className="size-3.5" />
        {summary ?? "Duration"}
        {summary ? (
          <span
            role="button"
            tabIndex={0}
            aria-label="Clear duration filter"
            className="ml-0.5 rounded-full p-0.5 hover:bg-muted"
            onClick={(event) => {
              event.stopPropagation()
              onChange({})
            }}
            onKeyDown={(event) => {
              if (event.key === "Enter" || event.key === " ") {
                event.stopPropagation()
                onChange({})
              }
            }}
          >
            <IconX className="size-3" />
          </span>
        ) : null}
      </PopoverTrigger>
      <PopoverContent align="start" className="w-64 space-y-3 p-3">
        <div className="flex flex-wrap gap-1.5">
          {presets.map((preset) => (
            <Button
              key={preset.label}
              type="button"
              variant="outline"
              size="sm"
              className={cn(
                "h-6 rounded-full px-2 text-xs",
                range.minMs === preset.minMs && range.maxMs === undefined && "bg-muted font-medium"
              )}
              onClick={() => onChange({ minMs: preset.minMs })}
            >
              {preset.label}
            </Button>
          ))}
        </div>
        <div className="flex items-center gap-2">
          <Input
            value={minText}
            onChange={(event) => setMinText(event.target.value)}
            placeholder="Min ms"
            inputMode="numeric"
            className="h-7 text-xs"
            aria-label="Minimum duration in milliseconds"
          />
          <span className="text-xs text-muted-foreground">to</span>
          <Input
            value={maxText}
            onChange={(event) => setMaxText(event.target.value)}
            placeholder="Max ms"
            inputMode="numeric"
            className="h-7 text-xs"
            aria-label="Maximum duration in milliseconds"
          />
        </div>
      </PopoverContent>
    </Popover>
  )
}
