// Client-side invocation facet counts (plan 164). The invocations list
// filters over already-loaded rows, so its facet sidebar counts the same
// in-window row set instead of a backend query — dimensions mirror the URL
// params (mode/status/outcome/command/service).

import { invocationStatus, type InvocationRow } from "@/lib/invocation"

export interface InvocationFacetValue {
  value: string
  count: number
}

export interface InvocationFacet {
  dimension: "mode" | "status" | "outcome" | "command" | "service"
  values: InvocationFacetValue[]
}

export const INVOCATION_FACET_VALUES_CAP = 24

function facetValue(
  row: InvocationRow,
  dimension: InvocationFacet["dimension"],
  nowMs: number
): string | null {
  switch (dimension) {
    case "mode":
      return row.appMode
    case "status":
      return invocationStatus(row, nowMs)
    case "outcome":
      return row.outcome
    case "command":
      return row.command
    case "service":
      return row.service
  }
}

/** Per-dimension value counts over the in-window rows, count-desc then
 * value-asc, capped. Rows without a value for a dimension are not counted. */
export function invocationFacetCounts(
  rows: readonly InvocationRow[],
  nowMs = Date.now()
): InvocationFacet[] {
  const dimensions: Array<InvocationFacet["dimension"]> = [
    "mode",
    "status",
    "outcome",
    "command",
    "service",
  ]
  return dimensions.map((dimension) => {
    const counts = new Map<string, number>()
    for (const row of rows) {
      const value = facetValue(row, dimension, nowMs)
      if (!value) continue
      counts.set(value, (counts.get(value) ?? 0) + 1)
    }
    const values = [...counts.entries()]
      .map(([value, count]) => ({ value, count }))
      .sort((a, b) => b.count - a.count || a.value.localeCompare(b.value))
      .slice(0, INVOCATION_FACET_VALUES_CAP)
    return { dimension, values }
  })
}
