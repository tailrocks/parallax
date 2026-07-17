/** Alert incident timeline pure model (plan 167 UI detail).
 *
 * Builds a chronological event list from incident + delivery/check rows for
 * the incident detail page. Pure — no GraphQL.
 *
 * Preliminary — peer wires GraphQL loads and renders the timeline.
 */

export type IncidentTimelineKind =
  | "triggered"
  | "renotify"
  | "resolved"
  | "check_breach"
  | "check_healthy"
  | "delivery_ok"
  | "delivery_fail"

export interface IncidentTimelineEvent {
  kind: IncidentTimelineKind
  /** Unix nanoseconds (or any monotonic clock the UI already uses). */
  atNanos: number
  label: string
  /** Optional observed value for chart/markers. */
  value?: number
  detail?: string
}

export interface IncidentSnapshot {
  firstTriggeredAtNanos: number
  lastTriggeredAtNanos: number
  resolvedAtNanos?: number | null
  status: "open" | "resolved" | string
  lastValue?: number | null
}

export interface DeliveryEventSnapshot {
  eventType: "triggered" | "resolved" | "renotify" | string
  /** When the delivery was completed; omit if still pending. */
  deliveredAtNanos?: number | null
  attemptCount: number
  error?: string | null
}

export interface CheckSnapshot {
  atNanos: number
  status: "breach" | "healthy" | "no_data" | "error" | string
  value?: number | null
  error?: string | null
}

/**
 * Build a sorted (ascending by `atNanos`) timeline from incident lifecycle
 * stamps, optional delivery events, and optional evaluation checks.
 * Lifecycle events always appear; delivery/check rows are supplementary.
 */
export function buildIncidentTimeline(
  incident: IncidentSnapshot,
  deliveries: readonly DeliveryEventSnapshot[] = [],
  checks: readonly CheckSnapshot[] = []
): IncidentTimelineEvent[] {
  const events: IncidentTimelineEvent[] = []

  events.push({
    kind: "triggered",
    atNanos: incident.firstTriggeredAtNanos,
    label: "Incident opened",
    value: incident.lastValue ?? undefined,
  })

  if (
    incident.lastTriggeredAtNanos > incident.firstTriggeredAtNanos &&
    incident.status === "open"
  ) {
    // Last re-fire while still open — treat as renotify marker when distinct.
    events.push({
      kind: "renotify",
      atNanos: incident.lastTriggeredAtNanos,
      label: "Last re-triggered",
      value: incident.lastValue ?? undefined,
    })
  }

  if (incident.resolvedAtNanos != null) {
    events.push({
      kind: "resolved",
      atNanos: incident.resolvedAtNanos,
      label: "Incident resolved",
    })
  }

  for (const d of deliveries) {
    if (d.deliveredAtNanos == null) continue
    const fail = Boolean(d.error) || d.attemptCount >= 5
    const kind: IncidentTimelineKind = fail ? "delivery_fail" : "delivery_ok"
    const et = d.eventType || "triggered"
    events.push({
      kind,
      atNanos: d.deliveredAtNanos,
      label: fail
        ? `Delivery failed (${et}, attempt ${d.attemptCount})`
        : `Delivered (${et})`,
      detail: d.error ?? undefined,
    })
  }

  for (const c of checks) {
    if (c.status === "breach") {
      events.push({
        kind: "check_breach",
        atNanos: c.atNanos,
        label: "Evaluation: breach",
        value: c.value ?? undefined,
        detail: c.error ?? undefined,
      })
    } else if (c.status === "healthy") {
      events.push({
        kind: "check_healthy",
        atNanos: c.atNanos,
        label: "Evaluation: healthy",
        value: c.value ?? undefined,
      })
    }
  }

  events.sort((a, b) => {
    if (a.atNanos !== b.atNanos) return a.atNanos - b.atNanos
    // Stable secondary: lifecycle before check before delivery.
    return kindRank(a.kind) - kindRank(b.kind)
  })
  return events
}

function kindRank(k: IncidentTimelineKind): number {
  switch (k) {
    case "triggered":
      return 0
    case "check_breach":
    case "check_healthy":
      return 1
    case "renotify":
      return 2
    case "delivery_ok":
    case "delivery_fail":
      return 3
    case "resolved":
      return 4
  }
}

/**
 * Severity-mix bar segments for a pattern cluster (plan 165) or alert chip.
 * Returns percentages that sum to 100 (largest-remainder) for CSS flex widths.
 */
export function severityMixSegments(
  counts: readonly { severity: string; count: number }[]
): { severity: string; count: number; pct: number }[] {
  const total = counts.reduce((s, c) => s + Math.max(0, c.count), 0)
  if (total <= 0) return []
  const raw = counts
    .filter((c) => c.count > 0)
    .map((c) => ({
      severity: c.severity,
      count: c.count,
      exact: (c.count / total) * 100,
    }))
  // Largest-remainder method so integer percents sum to 100.
  const floors = raw.map((r) => ({
    ...r,
    pct: Math.floor(r.exact),
    frac: r.exact - Math.floor(r.exact),
  }))
  let sum = floors.reduce((s, r) => s + r.pct, 0)
  const byFrac = [...floors].sort((a, b) => b.frac - a.frac)
  let i = 0
  while (sum < 100 && byFrac.length > 0) {
    byFrac[i % byFrac.length]!.pct += 1
    sum += 1
    i += 1
  }
  return floors.map((r) => ({
    severity: r.severity,
    count: r.count,
    pct: r.pct,
  }))
}
