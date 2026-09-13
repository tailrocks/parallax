import { Link } from "@tanstack/react-router"
import type { ReactNode } from "react"

import { rangeLinkSearch, type ResolvedRange } from "@/domain/time-range/range"
import { cn } from "@/lib/utils"

/** Range-carrying cross-signal links (workstream D).
 *
 * Every meaningful object links onward with the viewer's time range
 * preserved, so issue → trace → logs → metric never resets the window.
 * Pages must use these instead of hand-building `Link` + `rangeLinkSearch`
 * pairs (which drift: some links today drop range, some drop context).
 *
 * Each component renders `Link` directly with a literal `to` so TanStack
 * infers params/search types; do not re-abstract over `Link` props. */

type LinkTone = "plain" | "mono" | "muted"

const toneClass: Record<LinkTone, string> = {
  plain: "font-medium hover:underline",
  mono: "font-mono text-xs hover:underline",
  muted: "text-sm text-muted-foreground hover:text-foreground",
}

export function TraceLink({
  traceId,
  range,
  short = false,
  className,
}: {
  traceId: string
  range: ResolvedRange
  short?: boolean
  className?: string
}) {
  return (
    <Link
      to="/traces/$traceId"
      params={{ traceId }}
      search={rangeLinkSearch(range)}
      aria-label={`trace ${traceId}`}
      className={cn(toneClass.mono, className)}
    >
      {short ? traceId.slice(0, 8) : traceId}
    </Link>
  )
}

export function IssueLink({
  service,
  fingerprint,
  range,
  children,
  className,
}: {
  service: string
  fingerprint: string
  range: ResolvedRange
  children: ReactNode
  className?: string
}) {
  return (
    <Link
      to="/issues/$service/$fingerprint"
      params={{ service, fingerprint }}
      search={rangeLinkSearch(range)}
      className={cn(toneClass.plain, className)}
    >
      {children}
    </Link>
  )
}

export function ServiceLink({
  service,
  range,
  className,
}: {
  service: string
  range: ResolvedRange
  className?: string
}) {
  return (
    <Link
      to="/services/$service"
      params={{ service }}
      search={rangeLinkSearch(range)}
      className={cn(toneClass.plain, className)}
    >
      {service}
    </Link>
  )
}

export function InvocationLink({
  invocationId,
  range,
  tab,
  className,
}: {
  invocationId: string
  range?: ResolvedRange
  tab?: "errors" | "jobs" | "logs" | "overview" | "sessions" | "traces"
  className?: string
}) {
  return (
    <Link
      to="/invocations/$invocationId"
      params={{ invocationId }}
      search={{ ...(range ? rangeLinkSearch(range) : {}), ...(tab ? { tab } : {}) }}
      aria-label={`invocation ${invocationId}`}
      className={cn(toneClass.mono, className)}
    >
      {invocationId.slice(0, 8)}
    </Link>
  )
}

export function MetricLink({
  metricName,
  range,
  className,
}: {
  metricName: string
  range: ResolvedRange
  className?: string
}) {
  return (
    <Link
      to="/metrics/$metricName"
      params={{ metricName }}
      search={rangeLinkSearch(range)}
      className={cn(toneClass.mono, className)}
    >
      {metricName}
    </Link>
  )
}

/** Link into Logs preserving range plus optional service/body/anchor context.
 * Anchor mode opens the ±30s context window around one event. */
export function LogsLink({
  range,
  service,
  q,
  anchor,
  children,
  className,
}: {
  range: ResolvedRange
  service?: string
  q?: string
  anchor?: string
  children: ReactNode
  className?: string
}) {
  return (
    <Link
      to="/logs"
      search={{
        ...rangeLinkSearch(range),
        ...(service ? { service } : {}),
        ...(q ? { q } : {}),
        ...(anchor ? { anchor } : {}),
      }}
      className={cn(toneClass.muted, className)}
    >
      {children}
    </Link>
  )
}
