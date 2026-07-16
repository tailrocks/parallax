import { Link } from "@tanstack/react-router"
import {
  IconAlertTriangleFilled,
  IconArrowUpRight,
  IconDoorEnter,
  IconDoorExit,
  IconHandClick,
  IconLogin2,
  IconLogout2,
} from "@tabler/icons-react"

import { Badge } from "@/components/ui/badge"
import type { ScreenVisit, Session, UiAction } from "@/lib/api"
import { formatDurationNs, formatTimeShort } from "@/lib/format"
import { cn } from "@/lib/utils"

export interface JourneyError {
  tsNanos: string
  title: string
  fingerprint: string | null
  traceId: string | null
}

export type JourneyEntry =
  | { kind: "session-start"; tsNanos: string; session: Session }
  | { kind: "session-end"; tsNanos: string; session: Session }
  | {
      kind: "screen-entered"
      tsNanos: string
      visit: ScreenVisit
      dwellNs: string | null
    }
  | { kind: "screen-exited"; tsNanos: string; visit: ScreenVisit }
  | { kind: "action"; tsNanos: string; action: UiAction }
  | {
      kind: "error"
      tsNanos: string
      error: JourneyError
      screenId: string | null
    }

/**
 * Pure journey builder: one chronological narrative per session, with each
 * error attributed to the screen whose visit interval contains it.
 * Unattributable errors are kept (screenId null), never dropped.
 */
export function buildJourney(
  session: Session,
  visits: ScreenVisit[],
  actions: UiAction[],
  errors: JourneyError[]
): JourneyEntry[] {
  const sessionVisits = visits.filter(
    (visit) => visit.sessionId == null || visit.sessionId === session.sessionId
  )
  const sessionEnd = session.endNanos ? BigInt(session.endNanos) : null
  const inSession = (ts: bigint) =>
    ts >= BigInt(session.startNanos) && (sessionEnd == null || ts <= sessionEnd)

  const entries: JourneyEntry[] = [
    { kind: "session-start", tsNanos: session.startNanos, session },
  ]
  for (const visit of sessionVisits) {
    const dwellNs =
      visit.exitedNanos != null
        ? (BigInt(visit.exitedNanos) - BigInt(visit.enteredNanos)).toString()
        : null
    entries.push({
      kind: "screen-entered",
      tsNanos: visit.enteredNanos,
      visit,
      dwellNs,
    })
    if (visit.exitedNanos != null) {
      entries.push({ kind: "screen-exited", tsNanos: visit.exitedNanos, visit })
    }
  }
  for (const action of actions) {
    const ts = BigInt(action.startNanos)
    if (
      (action.sessionId == null || action.sessionId === session.sessionId) &&
      inSession(ts)
    ) {
      entries.push({ kind: "action", tsNanos: action.startNanos, action })
    }
  }
  for (const error of errors) {
    const ts = BigInt(error.tsNanos)
    if (!inSession(ts)) continue
    const screen = sessionVisits.find((visit) => {
      const entered = BigInt(visit.enteredNanos)
      const exited = visit.exitedNanos ? BigInt(visit.exitedNanos) : null
      return ts >= entered && (exited == null || ts <= exited)
    })
    entries.push({
      kind: "error",
      tsNanos: error.tsNanos,
      error,
      screenId: screen?.screenId ?? null,
    })
  }
  if (session.endNanos != null) {
    entries.push({ kind: "session-end", tsNanos: session.endNanos, session })
  }
  entries.sort((a, b) => {
    const at = BigInt(a.tsNanos)
    const bt = BigInt(b.tsNanos)
    if (at !== bt) return at < bt ? -1 : 1
    return journeyPhase(a) - journeyPhase(b)
  })
  return entries
}

function journeyPhase(entry: JourneyEntry): number {
  switch (entry.kind) {
    case "session-start":
      return 0
    case "screen-entered":
      return 1
    case "action":
      return 2
    case "error":
      return 3
    case "screen-exited":
      return 4
    case "session-end":
      return 5
  }
}

export function SessionJourney({
  session,
  visits,
  actions,
  errors,
  timeZone,
}: {
  session: Session
  visits: ScreenVisit[]
  actions: UiAction[]
  errors: JourneyError[]
  timeZone?: string
}) {
  const entries = buildJourney(session, visits, actions, errors)
  return (
    <ol className="space-y-1.5">
      {entries.map((entry, index) => (
        <li
          key={`${entry.kind}-${entry.tsNanos}-${index}`}
          className={cn(
            "flex flex-wrap items-center gap-2 rounded-md border bg-muted/20 px-3 py-1.5 text-sm",
            entry.kind === "error" &&
              "shadow-[inset_3px_0_0_rgba(244,63,94,0.85)]"
          )}
        >
          <span className="w-20 shrink-0 font-mono text-xs text-muted-foreground">
            {formatTimeShort(
              entry.tsNanos,
              timeZone
                ? {
                    hour: "2-digit",
                    minute: "2-digit",
                    second: "2-digit",
                    hour12: false,
                    timeZone,
                  }
                : undefined
            )}
          </span>
          <JourneyEntryBody entry={entry} />
        </li>
      ))}
    </ol>
  )
}

function JourneyEntryBody({ entry }: { entry: JourneyEntry }) {
  switch (entry.kind) {
    case "session-start":
      return (
        <span className="inline-flex items-center gap-1.5">
          <IconLogin2 className="size-4 text-sky-500" />
          session <code className="text-xs">
            {entry.session.sessionId}
          </code>{" "}
          started
          {entry.session.previousSessionId ? (
            <span className="text-xs text-muted-foreground">
              (previous {entry.session.previousSessionId})
            </span>
          ) : null}
        </span>
      )
    case "session-end":
      return (
        <span className="inline-flex items-center gap-1.5">
          <IconLogout2 className="size-4 text-muted-foreground" />
          session ended
        </span>
      )
    case "screen-entered":
      return (
        <span className="inline-flex items-center gap-1.5">
          <IconDoorEnter className="size-4 text-violet-500" />
          entered <Badge variant="outline">{entry.visit.screenId}</Badge>
          {entry.dwellNs ? (
            <span className="text-xs text-muted-foreground">
              dwelled {formatDurationNs(entry.dwellNs)}
            </span>
          ) : (
            <Badge variant="blue">active</Badge>
          )}
          {entry.visit.transitionReason ? (
            <span className="text-xs text-muted-foreground">
              via {entry.visit.transitionReason}
            </span>
          ) : null}
        </span>
      )
    case "screen-exited":
      return (
        <span className="inline-flex items-center gap-1.5 text-muted-foreground">
          <IconDoorExit className="size-4" />
          exited {entry.visit.screenId}
        </span>
      )
    case "action":
      return (
        <span className="inline-flex min-w-0 items-center gap-1.5">
          <IconHandClick className="size-4 text-emerald-500" />
          <Link
            to="/traces/$traceId"
            params={{ traceId: entry.action.traceId }}
            className="inline-flex items-center gap-1 font-medium hover:underline"
          >
            {entry.action.name}
            <IconArrowUpRight className="size-3.5" />
          </Link>
          {entry.action.screenId ? (
            <span className="text-xs text-muted-foreground">
              on {entry.action.screenId}
            </span>
          ) : null}
          {entry.action.outcome ? (
            <Badge
              variant={entry.action.outcome === "success" ? "emerald" : "rose"}
            >
              {entry.action.outcome}
            </Badge>
          ) : null}
        </span>
      )
    case "error":
      return (
        <span className="inline-flex min-w-0 items-center gap-1.5">
          <IconAlertTriangleFilled className="size-4 text-rose-500" />
          {entry.error.fingerprint ? (
            <Link
              to="/issues/$fingerprint"
              params={{ fingerprint: entry.error.fingerprint }}
              className="min-w-0 truncate font-medium hover:underline"
            >
              {entry.error.title}
            </Link>
          ) : entry.error.traceId ? (
            <Link
              to="/traces/$traceId"
              params={{ traceId: entry.error.traceId }}
              className="min-w-0 truncate font-medium hover:underline"
            >
              {entry.error.title}
            </Link>
          ) : (
            <span className="min-w-0 truncate font-medium">
              {entry.error.title}
            </span>
          )}
          <Badge variant="rose">
            {entry.screenId ? `on ${entry.screenId}` : "outside any screen"}
          </Badge>
        </span>
      )
  }
}
