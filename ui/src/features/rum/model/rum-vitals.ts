// Web-vital identity + rating (RUM surface).
//
// Browser telemetry arrives as plain OTLP: vitals are metric points, errors
// and journeys are spans/logs joined by trace_id. No dedicated RUM API
// exists, so vitals are discovered by matching catalog names against known
// vital tokens (OTel `web-vitals` emitters use `lcp`/`inp`/`cls`-style names
// or `browser.*`-prefixed variants).

export type VitalId = "LCP" | "INP" | "CLS" | "FCP" | "TTFB" | "FID"

export type VitalRating = "good" | "needs-improvement" | "poor"

export interface VitalSpec {
  readonly id: VitalId
  readonly label: string
  /** [goodMax, poorMin] in display units (ms, or score for CLS). */
  readonly thresholds: readonly [number, number]
  readonly unitless: boolean
}

export const VITAL_SPECS: Record<VitalId, VitalSpec> = {
  LCP: {
    id: "LCP",
    label: "Largest Contentful Paint",
    thresholds: [2500, 4000],
    unitless: false,
  },
  INP: {
    id: "INP",
    label: "Interaction to Next Paint",
    thresholds: [200, 500],
    unitless: false,
  },
  CLS: {
    id: "CLS",
    label: "Cumulative Layout Shift",
    thresholds: [0.1, 0.25],
    unitless: true,
  },
  FCP: {
    id: "FCP",
    label: "First Contentful Paint",
    thresholds: [1800, 3000],
    unitless: false,
  },
  TTFB: {
    id: "TTFB",
    label: "Time to First Byte",
    thresholds: [800, 1800],
    unitless: false,
  },
  FID: {
    id: "FID",
    label: "First Input Delay",
    thresholds: [100, 300],
    unitless: false,
  },
}

const TOKEN_TO_VITAL: ReadonlyArray<{ token: string; id: VitalId }> = [
  { token: "largestcontentfulpaint", id: "LCP" },
  { token: "interactiontonextpaint", id: "INP" },
  { token: "cumulativelayoutshift", id: "CLS" },
  { token: "firstcontentfulpaint", id: "FCP" },
  { token: "timetofirstbyte", id: "TTFB" },
  { token: "firstinputdelay", id: "FID" },
  { token: "lcp", id: "LCP" },
  { token: "inp", id: "INP" },
  { token: "cls", id: "CLS" },
  { token: "fcp", id: "FCP" },
  { token: "ttfb", id: "TTFB" },
  { token: "fid", id: "FID" },
]

/** Match a metric name to a vital, or null when it is not a vital. Long
 * tokens win so `firstcontentfulpaint` does not match `ttfb`-style short
 * tokens first; matching is substring over the alnum-folded name. */
export function matchVital(metricName: string): VitalId | null {
  const folded = metricName.toLowerCase().replace(/[^a-z0-9]/g, "")
  for (const { token, id } of TOKEN_TO_VITAL) {
    if (folded.includes(token)) return id
  }
  return null
}

/** Normalize a raw vital value to display units. Time vitals arrive in ms;
 * second-denominated units are scaled up. CLS is unitless. */
export function normalizeVitalValue(raw: number, unit: string | null): number {
  if (unit === "s") return raw * 1000
  return raw
}

export function rateVital(id: VitalId, displayValue: number): VitalRating {
  const [goodMax, poorMin] = VITAL_SPECS[id].thresholds
  if (displayValue <= goodMax) return "good"
  if (displayValue < poorMin) return "needs-improvement"
  return "poor"
}

export function ratingLabel(rating: VitalRating): string {
  switch (rating) {
    case "good":
      return "Good"
    case "needs-improvement":
      return "Needs improvement"
    case "poor":
      return "Poor"
  }
}

export function formatVitalValue(id: VitalId, displayValue: number): string {
  if (VITAL_SPECS[id].unitless) return displayValue.toFixed(3)
  if (displayValue >= 1000) return `${(displayValue / 1000).toFixed(2)}s`
  return `${Math.round(displayValue)}ms`
}
