/** Waterfall color-by strategies (plan 163).
 *
 * The trace timeline colors bars by one strategy at a time: service identity
 * (default, plan-162 deterministic palette), span kind, span status, or any
 * span attribute. The choice is URL-encoded in the trace route's search
 * params so a colored view survives reload and sharing.
 */

import { serviceColor } from "@/shared/colors"

export type ColorByStrategy =
  | { kind: "service" }
  | { kind: "spanKind" }
  | { kind: "status" }
  | { kind: "attribute"; key: string }

export const DEFAULT_COLOR_BY: ColorByStrategy = { kind: "service" }

const ATTRIBUTE_PREFIX = "attr:"

/** Strategy → URL search-param value. */
export function encodeColorBy(strategy: ColorByStrategy): string {
  switch (strategy.kind) {
    case "service":
      return "service"
    case "spanKind":
      return "kind"
    case "status":
      return "status"
    case "attribute":
      return `${ATTRIBUTE_PREFIX}${strategy.key}`
  }
}

/** URL search-param value → strategy; anything unrecognized falls back to
 * the service default so stale permalinks keep working. */
export function decodeColorBy(value: string | undefined): ColorByStrategy {
  if (!value || value === "service") return DEFAULT_COLOR_BY
  if (value === "kind") return { kind: "spanKind" }
  if (value === "status") return { kind: "status" }
  if (value.startsWith(ATTRIBUTE_PREFIX)) {
    const key = value.slice(ATTRIBUTE_PREFIX.length)
    if (key) return { kind: "attribute", key }
  }
  return DEFAULT_COLOR_BY
}

export interface ColorableSpan {
  service: string
  /** OTLP span kind, e.g. "SPAN_KIND_SERVER". */
  kind: string
  /** OTLP status code, e.g. "STATUS_CODE_ERROR". */
  statusCode: string
  attributes: Record<string, string>
}

/** Neutral color for spans the strategy cannot classify. */
export const COLOR_BY_UNKNOWN = "var(--muted-foreground)"

// Matches the waterfall's kind chip hues (span-kind.tsx): the generic
// --chart-1..5 tokens are grayscale in this theme and would erase the axis.
const SPAN_KIND_COLORS: Record<string, string> = {
  SPAN_KIND_SERVER: "oklch(0.65 0.13 235)",
  SPAN_KIND_CLIENT: "oklch(0.6 0.16 260)",
  SPAN_KIND_INTERNAL: "oklch(0.6 0.17 295)",
  SPAN_KIND_PRODUCER: "oklch(0.75 0.15 80)",
  SPAN_KIND_CONSUMER: "oklch(0.68 0.14 160)",
}

const STATUS_COLORS: Record<string, string> = {
  STATUS_CODE_ERROR: "var(--chart-error)",
  STATUS_CODE_OK: "var(--severity-info)",
  STATUS_CODE_UNSET: COLOR_BY_UNKNOWN,
}

/** Color for one span under the active strategy. Attribute values color by
 * the same deterministic hash as service identity, so the same value reads
 * as the same color across traces. */
export function colorForSpan(strategy: ColorByStrategy, span: ColorableSpan): string {
  switch (strategy.kind) {
    case "service":
      return span.service ? serviceColor(span.service).color : COLOR_BY_UNKNOWN
    case "spanKind":
      return SPAN_KIND_COLORS[span.kind] ?? COLOR_BY_UNKNOWN
    case "status":
      return STATUS_COLORS[span.statusCode] ?? COLOR_BY_UNKNOWN
    case "attribute": {
      const value = span.attributes[strategy.key]
      return value ? serviceColor(value).color : COLOR_BY_UNKNOWN
    }
  }
}

/** Sorted unique attribute keys present in the loaded trace — the option
 * list for the attribute color-by mode. */
export function attributeKeysForColorBy(spans: readonly ColorableSpan[]): string[] {
  const keys = new Set<string>()
  for (const span of spans) {
    for (const key of Object.keys(span.attributes)) keys.add(key)
  }
  return [...keys].sort((a, b) => a.localeCompare(b))
}

export interface ColorByLegendEntry {
  label: string
  color: string
}

export const COLOR_BY_LEGEND_MAX = 12

/** Legend entries for the active strategy: distinct labels in first-seen
 * order, capped at COLOR_BY_LEGEND_MAX (callers show a "+N more" hint). */
export function colorByLegend(
  strategy: ColorByStrategy,
  spans: readonly ColorableSpan[]
): ColorByLegendEntry[] {
  const entries: ColorByLegendEntry[] = []
  const seen = new Set<string>()
  for (const span of spans) {
    let label: string
    switch (strategy.kind) {
      case "service":
        label = span.service || "unknown"
        break
      case "spanKind":
        label = span.kind
        break
      case "status":
        label = span.statusCode
        break
      case "attribute":
        label = span.attributes[strategy.key] ?? "(missing)"
        break
    }
    if (seen.has(label)) continue
    seen.add(label)
    entries.push({ label, color: colorForSpan(strategy, span) })
    if (entries.length >= COLOR_BY_LEGEND_MAX) break
  }
  return entries
}
