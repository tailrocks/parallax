/** Ecosystem map URL codec (plan 166).
 *
 * Encodes/decodes focus, hops, focusMode, minTraffic search params.
 * Pure — no React Router coupling. Peer wires into ecosystem.tsx.
 *
 * Preliminary — peer verify/extend + browser URL-reload evidence.
 */

import {
  TRAFFIC_PRESETS,
  type FocusMode,
  type TrafficPreset,
} from "@/lib/ecosystem-topology"

export interface EcosystemUrlState {
  focus: string | null
  hops: number
  focusMode: FocusMode
  /** Fraction of max edge callCount; 0 = show all. */
  minTraffic: number
}

export const DEFAULT_ECOSYSTEM_URL: EcosystemUrlState = {
  focus: null,
  hops: 1,
  focusMode: "dim",
  minTraffic: 0,
}

export const ECOSYSTEM_PARAM_KEYS = [
  "focus",
  "hops",
  "focusMode",
  "minTraffic",
] as const

/** Clamp hops to a sensible product range (0–3). */
export function clampHops(raw: number): number {
  if (!Number.isFinite(raw)) return DEFAULT_ECOSYSTEM_URL.hops
  return Math.min(3, Math.max(0, Math.floor(raw)))
}

export function parseFocusMode(raw: string | null | undefined): FocusMode {
  if (raw === "hide" || raw === "dim") return raw
  return DEFAULT_ECOSYSTEM_URL.focusMode
}

/**
 * Parse minTraffic from URL: accepts preset labels (`all`, `0.1%`, `1%`, `5%`)
 * or a bare fraction / percent number (`0.01`, `1` meaning 1%).
 */
export function parseMinTraffic(raw: string | null | undefined): number {
  if (raw == null || raw === "") return 0
  const trimmed = raw.trim()
  if (trimmed in TRAFFIC_PRESETS) {
    return TRAFFIC_PRESETS[trimmed as TrafficPreset]
  }
  // "1%" style
  if (trimmed.endsWith("%")) {
    const n = Number(trimmed.slice(0, -1))
    if (!Number.isFinite(n)) return 0
    return Math.min(1, Math.max(0, n / 100))
  }
  const n = Number(trimmed)
  if (!Number.isFinite(n)) return 0
  // bare 1..100 treated as percent; values in (0,1] as fraction
  if (n > 1) return Math.min(1, Math.max(0, n / 100))
  return Math.min(1, Math.max(0, n))
}

/** Encode minTraffic back to a stable preset label when possible. */
export function encodeMinTraffic(fraction: number): string {
  if (fraction <= 0) return "all"
  for (const [label, value] of Object.entries(TRAFFIC_PRESETS) as [
    TrafficPreset,
    number,
  ][]) {
    if (label === "all") continue
    if (Math.abs(value - fraction) < 1e-12) return label
  }
  // custom fraction as percent with trim
  const pct = fraction * 100
  return `${Number(pct.toFixed(4))}%`
}

export function decodeEcosystemUrl(
  params: URLSearchParams | Record<string, string | undefined>
): EcosystemUrlState {
  const get = (key: string): string | undefined => {
    if (params instanceof URLSearchParams) {
      return params.get(key) ?? undefined
    }
    return params[key]
  }
  const focusRaw = get("focus")?.trim()
  const hopsRaw = get("hops")
  return {
    focus: focusRaw ? focusRaw : null,
    hops: clampHops(
      hopsRaw != null ? Number(hopsRaw) : DEFAULT_ECOSYSTEM_URL.hops
    ),
    focusMode: parseFocusMode(get("focusMode")),
    minTraffic: parseMinTraffic(get("minTraffic")),
  }
}

export function encodeEcosystemUrl(state: EcosystemUrlState): URLSearchParams {
  const params = new URLSearchParams()
  if (state.focus) params.set("focus", state.focus)
  if (state.hops !== DEFAULT_ECOSYSTEM_URL.hops) {
    params.set("hops", String(clampHops(state.hops)))
  }
  if (state.focusMode !== DEFAULT_ECOSYSTEM_URL.focusMode) {
    params.set("focusMode", state.focusMode)
  }
  if (state.minTraffic > 0) {
    params.set("minTraffic", encodeMinTraffic(state.minTraffic))
  }
  return params
}
