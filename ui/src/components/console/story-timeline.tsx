import { Link } from "@tanstack/react-router"
import { IconAlertTriangle, IconArticle, IconCircleDot, IconClock } from "@tabler/icons-react"

import { Badge } from "@/components/ui/badge"
import type { StoryBeat } from "@/lib/api"
import { formatDurationNs, formatTimeInRange } from "@/lib/format"
import { cn } from "@/lib/utils"

function beatTone(beat: StoryBeat) {
  if (beat.kind === "error" || beat.severity === "ERROR") {
    return {
      icon: IconAlertTriangle,
      badge: "rose" as const,
      row: "border-rose-500/30 bg-rose-500/10",
    }
  }
  if (beat.kind === "log") {
    return { icon: IconArticle, badge: "secondary" as const, row: "" }
  }
  if (beat.kind === "span.end") {
    return { icon: IconClock, badge: "outline" as const, row: "" }
  }
  return { icon: IconCircleDot, badge: "outline" as const, row: "" }
}

function timeRange(beats: StoryBeat[]) {
  const timestamps = beats.map((beat) => BigInt(beat.tsNanos))
  const start = timestamps.reduce((min, ts) => (ts < min ? ts : min), timestamps[0] ?? 0n)
  const end = timestamps.reduce((max, ts) => (ts > max ? ts : max), start)
  return {
    fromNanos: start.toString(),
    toNanos: (end > start ? end : start + 1n).toString(),
  }
}

function logWindow(tsNanos: string) {
  const ts = BigInt(tsNanos)
  return {
    from: (ts - 5_000_000_000n).toString(),
    to: (ts + 5_000_000_000n).toString(),
  }
}

function BeatLink({ beat }: { beat: StoryBeat }) {
  if (beat.kind === "log" || (beat.kind === "error" && !beat.spanId)) {
    const window = logWindow(beat.tsNanos)
    return (
      <Link
        to="/logs"
        search={{ service: beat.lane, from: window.from, to: window.to }}
        className="font-medium break-words hover:underline"
      >
        {beat.title}
      </Link>
    )
  }
  return (
    <Link
      to="/traces/$traceId"
      params={{ traceId: beat.traceId }}
      search={{}}
      className="font-medium break-words hover:underline"
    >
      {beat.title}
    </Link>
  )
}

export function StoryTimeline({ beats }: { beats: StoryBeat[] }) {
  const ordered = beats
  if (ordered.length === 0) {
    return <p className="text-sm text-muted-foreground">No story beats.</p>
  }
  const range = timeRange(ordered)

  return (
    <ol className="flex flex-col gap-2">
      {ordered.map((beat, index) => {
        const tone = beatTone(beat)
        const Icon = tone.icon
        return (
          <li
            key={`${beat.tsNanos}-${beat.kind}-${beat.traceId}-${beat.spanId ?? index}`}
            data-testid="story-row"
            className={cn(
              "grid gap-3 rounded-lg border border-border/70 bg-background/70 px-3 py-2 text-sm md:grid-cols-[8rem_8rem_minmax(0,1fr)]",
              tone.row
            )}
          >
            <span className="flex items-center gap-1.5 text-xs text-muted-foreground tabular-nums">
              <Icon />
              {formatTimeInRange(beat.tsNanos, range)}
            </span>
            <span className="min-w-0 truncate font-mono text-xs text-muted-foreground">
              {beat.lane}
            </span>
            <span className="flex min-w-0 flex-col gap-1">
              <span className="flex flex-wrap items-center gap-1.5">
                <Badge variant={tone.badge}>{beat.kind}</Badge>
                {beat.severity ? (
                  <Badge variant={tone.badge}>{beat.severity}</Badge>
                ) : null}
                {beat.durationNs ? (
                  <span className="text-xs text-muted-foreground tabular-nums">
                    {formatDurationNs(beat.durationNs)}
                  </span>
                ) : null}
              </span>
              <BeatLink beat={beat} />
            </span>
          </li>
        )
      })}
    </ol>
  )
}
