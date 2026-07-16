// Pure CLI-invocation model helpers shared by the list and hub routes.

import type { Invocation, ObservedInvocation } from "@/lib/api"

export type InvocationStatus = "running" | "finished" | "failed" | "stale"

export type AppMode = "one_shot" | "interactive" | "daemon" | "capsule"

/** An unfinished invocation with no signal newer than this is stale. */
export const STALE_AFTER_NS = 5n * 60n * 1_000_000_000n

/** One row of the merged invocation list (registered and/or observed). */
export interface InvocationRow {
  invocationId: string
  source: "cli" | "external"
  command: string | null
  appMode: string | null
  outcome: string | null
  service: string | null
  registeredStatus: string | null
  exitCode: number | null
  startedAtNanos: string
  endedAtNanos: string | null
  lastNanos: string
  errorCount: number | null
  traceCount: number | null
  sessionCount: number | null
  spanCount: number
  logCount: number
}

/**
 * Derived lifecycle status: `failed` when the exit code or outcome says so,
 * `finished` when ended, `running` while signals are fresh, else `stale`.
 */
export function invocationStatus(
  row: Pick<
    InvocationRow,
    "endedAtNanos" | "exitCode" | "outcome" | "lastNanos" | "startedAtNanos"
  >,
  nowMs = Date.now()
): InvocationStatus {
  const failedOutcome =
    row.outcome === "failure" ||
    row.outcome === "error" ||
    row.outcome === "timeout"
  if (row.endedAtNanos != null) {
    return (row.exitCode ?? 0) !== 0 || failedOutcome ? "failed" : "finished"
  }
  if (failedOutcome) return "failed"
  const nowNs = BigInt(nowMs) * 1_000_000n
  const lastSeen = BigInt(
    row.lastNanos !== "0" && row.lastNanos !== ""
      ? row.lastNanos
      : row.startedAtNanos
  )
  return nowNs - lastSeen < STALE_AFTER_NS ? "running" : "stale"
}

export function appModeLabel(mode: string | null): string | null {
  if (mode === "one_shot") return "one-shot"
  return mode
}

/** Wall duration in ns as a string, or null when unknown / not started. */
export function invocationDurationNs(
  row: Pick<InvocationRow, "startedAtNanos" | "endedAtNanos" | "lastNanos">,
  status: InvocationStatus,
  nowMs = Date.now()
): string | null {
  const start = BigInt(row.startedAtNanos)
  const end =
    status === "running"
      ? BigInt(nowMs) * 1_000_000n
      : row.endedAtNanos != null
        ? BigInt(row.endedAtNanos)
        : BigInt(row.lastNanos)
  if (end <= start) return null
  return (end - start).toString()
}

/** Union registered invocations with observed telemetry, CLI rows winning. */
export function mergeInvocations(
  invocations: Invocation[],
  observed: ObservedInvocation[]
): InvocationRow[] {
  const rows = new Map<string, InvocationRow>()
  for (const row of observed) {
    rows.set(row.invocationId, {
      invocationId: row.invocationId,
      source: "external",
      command: row.lastCommand,
      appMode: row.appMode,
      outcome: null,
      service: row.service,
      registeredStatus: null,
      exitCode: null,
      startedAtNanos: row.firstNanos,
      endedAtNanos: null,
      lastNanos: row.lastNanos,
      errorCount: null,
      traceCount: null,
      sessionCount: null,
      spanCount: row.spanCount,
      logCount: row.logCount,
    })
  }
  for (const row of invocations) {
    const seen = rows.get(row.invocationId)
    rows.set(row.invocationId, {
      invocationId: row.invocationId,
      source: "cli",
      command: row.command ?? seen?.command ?? null,
      appMode: row.appMode ?? seen?.appMode ?? null,
      outcome: row.outcome,
      service: seen?.service ?? null,
      registeredStatus: row.status,
      exitCode: row.exitCode,
      startedAtNanos: row.startedAtNanos,
      endedAtNanos: row.endedAtNanos,
      lastNanos: maxNanos(
        seen?.lastNanos,
        row.endedAtNanos ?? row.startedAtNanos
      ),
      errorCount: row.errorCount,
      traceCount: row.traceCount,
      sessionCount: row.sessionCount,
      spanCount: seen?.spanCount ?? 0,
      logCount: seen?.logCount ?? 0,
    })
  }
  return [...rows.values()].sort((a, b) =>
    BigInt(a.lastNanos) < BigInt(b.lastNanos) ? 1 : -1
  )
}

function maxNanos(a: string | undefined, b: string): string {
  if (a == null || a === "") return b
  return BigInt(a) >= BigInt(b) ? a : b
}
