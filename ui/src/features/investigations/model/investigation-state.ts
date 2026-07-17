import { DEFAULT_RANGE_KEY } from "@/domain/time-range/range"

export type InvestigationPinKind = "trace" | "issue" | "run" | "log" | "view"

export interface InvestigationWindow {
  range: string
  from?: string | undefined
  to?: string | undefined
}

export interface InvestigationPin {
  kind: InvestigationPinKind
  ref: string
  label: string
  note?: string | undefined
}

export interface InvestigationState {
  version: 1
  window: InvestigationWindow
  pins: InvestigationPin[]
  notes: string
}

export const INVESTIGATION_PIN_LIMIT = 100

export function emptyInvestigationState(): InvestigationState {
  return {
    version: 1,
    window: { range: DEFAULT_RANGE_KEY },
    pins: [],
    notes: "",
  }
}

function isPinKind(value: unknown): value is InvestigationPinKind {
  return (
    value === "trace" ||
    value === "issue" ||
    value === "run" ||
    value === "log" ||
    value === "view"
  )
}

function parseWindow(value: unknown): InvestigationWindow {
  if (!value || typeof value !== "object") return { range: DEFAULT_RANGE_KEY }
  const record = value as Record<string, unknown>
  const range =
    typeof record["range"] === "string" ? record["range"] : DEFAULT_RANGE_KEY
  const from = typeof record["from"] === "string" ? record["from"] : undefined
  const to = typeof record["to"] === "string" ? record["to"] : undefined
  return { range, from, to }
}

function parsePin(value: unknown): InvestigationPin | null {
  if (!value || typeof value !== "object") return null
  const record = value as Record<string, unknown>
  if (
    !isPinKind(record["kind"]) ||
    typeof record["ref"] !== "string" ||
    typeof record["label"] !== "string"
  ) {
    return null
  }
  return {
    kind: record["kind"],
    ref: record["ref"],
    label: record["label"],
    note: typeof record["note"] === "string" ? record["note"] : undefined,
  }
}

export function parseInvestigationState(state: string): InvestigationState {
  try {
    const parsed: unknown = JSON.parse(state)
    if (!parsed || typeof parsed !== "object") return emptyInvestigationState()
    const record = parsed as Record<string, unknown>
    if (record["version"] !== 1) return emptyInvestigationState()
    const pins = Array.isArray(record["pins"])
      ? record["pins"]
          .map(parsePin)
          .filter((pin): pin is InvestigationPin => Boolean(pin))
          .slice(0, INVESTIGATION_PIN_LIMIT)
      : []
    return {
      version: 1,
      window: parseWindow(record["window"]),
      pins,
      notes: typeof record["notes"] === "string" ? record["notes"] : "",
    }
  } catch {
    return emptyInvestigationState()
  }
}

export function serializeInvestigationState(state: InvestigationState): string {
  return JSON.stringify({
    version: 1,
    window: state.window,
    pins: state.pins.slice(0, INVESTIGATION_PIN_LIMIT),
    notes: state.notes,
  })
}

export function windowFromHref(href: string): InvestigationWindow {
  const query = href.includes("?") ? href.split("?")[1]?.split("#")[0] : ""
  const params = new URLSearchParams(query)
  const from = params.get("from") ?? undefined
  const to = params.get("to") ?? undefined
  if (from && to) return { range: "custom", from, to }
  return { range: params.get("range") ?? DEFAULT_RANGE_KEY }
}

export function buildInvestigationPin(
  kind: InvestigationPinKind,
  label: string,
  href: string,
  note = ""
): InvestigationPin {
  return {
    kind,
    ref: href,
    label: label.trim() || href,
    note,
  }
}

export function appendInvestigationPin(
  state: InvestigationState,
  pin: InvestigationPin
): InvestigationState {
  return {
    ...state,
    pins: [...state.pins, pin].slice(0, INVESTIGATION_PIN_LIMIT),
  }
}

export function investigationWindowSearch(window: InvestigationWindow): {
  range?: string
  from?: string
  to?: string
} {
  if (window.range === "custom" && window.from && window.to) {
    return { range: "custom", from: window.from, to: window.to }
  }
  return { range: window.range || DEFAULT_RANGE_KEY }
}

export function hrefForPin(pin: InvestigationPin): string {
  if (pin.ref.startsWith("/")) return pin.ref
  if (pin.kind === "trace") return `/traces/${encodeURIComponent(pin.ref)}`
  if (pin.kind === "issue") return `/issues/${encodeURIComponent(pin.ref)}`
  if (pin.kind === "run") return `/runs/${encodeURIComponent(pin.ref)}`
  return pin.ref
}
