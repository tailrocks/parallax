import {
  IconAlertTriangle,
  IconArrowDown,
  IconArrowUp,
  IconClock,
} from "@tabler/icons-react"

import { Badge } from "@/components/ui/badge"
import { formatDateTime, formatDurationNs } from "@/lib/format"
import type { RpcMessage, RpcStreamInfo } from "@/lib/rpc-trace"
import { cn } from "@/lib/utils"

function outcomeBadge(outcome: RpcStreamInfo["outcome"]) {
  if (outcome === "deadline_exceeded") {
    return { label: "deadline", variant: "rose" as const }
  }
  if (outcome === "cancelled") {
    return { label: "cancelled", variant: "rose" as const }
  }
  if (outcome === "error") return { label: "error", variant: "rose" as const }
  if (outcome === "ok") return { label: "ok", variant: "secondary" as const }
  return null
}

function messageTone(message: RpcMessage) {
  if (message.type === "SENT") {
    return {
      label: "SENT",
      dot: "rotate-45 rounded-[2px] bg-amber-500",
      icon: IconArrowUp,
      badge: "amber" as const,
    }
  }
  if (message.type === "RECEIVED") {
    return {
      label: "RECEIVED",
      dot: "rounded-full bg-emerald-500",
      icon: IconArrowDown,
      badge: "emerald" as const,
    }
  }
  return {
    label: "unknown",
    dot: "rounded-full bg-muted-foreground",
    icon: IconClock,
    badge: "secondary" as const,
  }
}

function positionPct(message: RpcMessage, stream: RpcStreamInfo): number {
  const start = BigInt(stream.startNanos || "0")
  const duration = BigInt(stream.durationNs || "0")
  const end = start + (duration > 0n ? duration : 1n)
  const time = BigInt(message.timeUnixNano || "0")
  const clamped = time < start ? start : time > end ? end : time
  return Number(((clamped - start) * 10_000n) / (end - start)) / 100
}

function messageTitle(message: RpcMessage): string {
  const parts = [
    message.type,
    message.id == null ? null : `id ${message.id}`,
    message.size == null ? null : `${message.size} bytes`,
    formatDateTime(message.timeUnixNano),
  ].filter(Boolean)
  return parts.join(" - ")
}

function MessageRows({ messages }: { messages: RpcMessage[] }) {
  return (
    <ul className="space-y-1">
      {messages.map((message, index) => {
        const tone = messageTone(message)
        const Icon = tone.icon
        return (
          <li
            key={`${message.timeUnixNano}-${index}`}
            className="grid grid-cols-[5rem_5rem_minmax(0,1fr)] items-center gap-2 rounded-md bg-muted/25 px-2 py-1 text-xs"
          >
            <Badge variant={tone.badge}>
              <Icon />
              {tone.label}
            </Badge>
            <span className="font-mono text-muted-foreground">
              {message.id == null ? "-" : `#${message.id}`}
            </span>
            <span className="text-right text-muted-foreground tabular-nums">
              {message.size == null
                ? "-"
                : `${message.size.toLocaleString()} B`}
            </span>
          </li>
        )
      })}
    </ul>
  )
}

export function RpcStreamCard({ streams }: { streams: RpcStreamInfo[] }) {
  if (streams.length === 0) return null

  return (
    <div className="space-y-4">
      {streams.map((stream) => {
        const outcome = outcomeBadge(stream.outcome)
        return (
          <section key={stream.spanId} className="space-y-3">
            <div className="flex flex-wrap items-center justify-between gap-2">
              <div className="flex min-w-0 flex-wrap items-center gap-2">
                <Badge variant="blue">{stream.system}</Badge>
                <span className="min-w-0 truncate text-sm font-medium">
                  {stream.method}
                </span>
                {outcome ? (
                  <Badge variant={outcome.variant}>{outcome.label}</Badge>
                ) : null}
              </div>
              <span className="text-xs text-muted-foreground tabular-nums">
                {stream.messages.length.toLocaleString()} messages
              </span>
            </div>

            <div className="relative h-9 rounded-md border border-border/70 bg-background/70 px-2">
              <div className="absolute inset-x-2 top-1/2 h-px -translate-y-1/2 bg-border" />
              {stream.messages.map((message, index) => {
                const tone = messageTone(message)
                return (
                  <span
                    key={`${message.timeUnixNano}-${index}`}
                    data-testid="rpc-message-dot"
                    title={messageTitle(message)}
                    className={cn(
                      "absolute top-1/2 size-2.5 -translate-y-1/2",
                      tone.dot
                    )}
                    style={{
                      left: `calc(${positionPct(message, stream)}% - 5px)`,
                    }}
                  />
                )
              })}
              {stream.outcome && stream.outcome !== "ok" ? (
                <span
                  className="absolute top-1/2 right-1 size-3 -translate-y-1/2 rounded-full border-2 border-background bg-rose-500"
                  title={stream.outcome}
                >
                  <IconAlertTriangle className="size-3 text-rose-700" />
                </span>
              ) : null}
            </div>

            {stream.messages.length < 8 ? (
              <MessageRows messages={stream.messages} />
            ) : null}
            {stream.truncated ? (
              <p className="text-xs text-muted-foreground">
                showing first {stream.messages.length.toLocaleString()} messages
              </p>
            ) : null}
            <p className="text-xs text-muted-foreground tabular-nums">
              {formatDurationNs(stream.durationNs)}
            </p>
          </section>
        )
      })}
    </div>
  )
}
