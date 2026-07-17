import { describe, expect, it } from "vitest"

import {
  COLOR_BY_LEGEND_MAX,
  COLOR_BY_UNKNOWN,
  attributeKeysForColorBy,
  colorByLegend,
  colorForSpan,
  decodeColorBy,
  encodeColorBy,
  type ColorableSpan,
} from "@/lib/color-by"
import { serviceColor } from "@/lib/colors"

function span(overrides: Partial<ColorableSpan> = {}): ColorableSpan {
  return {
    service: "checkout",
    kind: "SPAN_KIND_SERVER",
    statusCode: "STATUS_CODE_OK",
    attributes: {},
    ...overrides,
  }
}

describe("color-by URL round-trip (plan 163)", () => {
  it("round-trips every strategy through encode/decode", () => {
    for (const strategy of [
      { kind: "service" } as const,
      { kind: "spanKind" } as const,
      { kind: "status" } as const,
      { kind: "attribute", key: "http.method" } as const,
    ]) {
      expect(decodeColorBy(encodeColorBy(strategy))).toEqual(strategy)
    }
  })

  it("falls back to service for missing or unrecognized values", () => {
    expect(decodeColorBy(undefined)).toEqual({ kind: "service" })
    expect(decodeColorBy("bogus")).toEqual({ kind: "service" })
    expect(decodeColorBy("attr:")).toEqual({ kind: "service" })
  })
})

describe("colorForSpan", () => {
  it("service mode uses the plan-162 deterministic service color", () => {
    expect(colorForSpan({ kind: "service" }, span())).toBe(
      serviceColor("checkout").color
    )
    expect(colorForSpan({ kind: "service" }, span({ service: "" }))).toBe(
      COLOR_BY_UNKNOWN
    )
  })

  it("span-kind mode maps the five OTLP kinds and defaults unknown", () => {
    const kinds = [
      "SPAN_KIND_SERVER",
      "SPAN_KIND_CLIENT",
      "SPAN_KIND_INTERNAL",
      "SPAN_KIND_PRODUCER",
      "SPAN_KIND_CONSUMER",
    ]
    const colors = kinds.map((kind) =>
      colorForSpan({ kind: "spanKind" }, span({ kind }))
    )
    expect(new Set(colors).size).toBe(kinds.length)
    expect(
      colorForSpan(
        { kind: "spanKind" },
        span({ kind: "SPAN_KIND_UNSPECIFIED" })
      )
    ).toBe(COLOR_BY_UNKNOWN)
  })

  it("status mode: error red, ok info, unset neutral", () => {
    expect(
      colorForSpan(
        { kind: "status" },
        span({ statusCode: "STATUS_CODE_ERROR" })
      )
    ).toBe("var(--chart-error)")
    expect(
      colorForSpan({ kind: "status" }, span({ statusCode: "STATUS_CODE_OK" }))
    ).toBe("var(--severity-info)")
    expect(
      colorForSpan(
        { kind: "status" },
        span({ statusCode: "STATUS_CODE_UNSET" })
      )
    ).toBe(COLOR_BY_UNKNOWN)
  })

  it("attribute mode colors values deterministically, missing neutral", () => {
    const strategy = { kind: "attribute", key: "http.method" } as const
    const get = span({ attributes: { "http.method": "GET" } })
    expect(colorForSpan(strategy, get)).toBe(serviceColor("GET").color)
    expect(colorForSpan(strategy, span())).toBe(COLOR_BY_UNKNOWN)
  })
})

describe("attributeKeysForColorBy", () => {
  it("returns sorted unique keys across all spans", () => {
    const spans = [
      span({ attributes: { "http.method": "GET", "db.system": "pg" } }),
      span({ attributes: { "http.method": "POST", "rpc.service": "x" } }),
    ]
    expect(attributeKeysForColorBy(spans)).toEqual([
      "db.system",
      "http.method",
      "rpc.service",
    ])
  })
})

describe("colorByLegend", () => {
  it("lists distinct labels in first-seen order", () => {
    const spans = [
      span({ service: "checkout" }),
      span({ service: "cart" }),
      span({ service: "checkout" }),
    ]
    expect(
      colorByLegend({ kind: "service" }, spans).map((e) => e.label)
    ).toEqual(["checkout", "cart"])
  })

  it("labels missing attribute values and caps the list", () => {
    const spans = [
      span(),
      ...Array.from({ length: 20 }, (_, index) =>
        span({ attributes: { "http.route": `/route-${index}` } })
      ),
    ]
    const legend = colorByLegend(
      { kind: "attribute", key: "http.route" },
      spans
    )
    expect(legend[0]?.label).toBe("(missing)")
    expect(legend).toHaveLength(COLOR_BY_LEGEND_MAX)
  })
})
