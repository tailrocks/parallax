import { afterEach, describe, expect, it, vi } from "vitest"

import { loadMetricDetail } from "@/features/runtime-metrics/api/load-metric-detail"
import { graphqlCached } from "@/platform/graphql/transport"
import type * as GraphqlTransport from "@/platform/graphql/transport"

vi.mock("@/platform/graphql/transport", async (importOriginal) => {
  const actual = await importOriginal<typeof GraphqlTransport>()
  return {
    ...actual,
    graphqlCached: vi.fn(),
  }
})

afterEach(() => {
  vi.mocked(graphqlCached).mockReset()
})

describe("loadMetricDetail", () => {
  it("selects chartAnnotations without requiring search.service", async () => {
    vi.mocked(graphqlCached)
      .mockResolvedValueOnce({
        metricLabels: ["service.name"],
        metricQuery: {
          series: [{ groupValue: null, points: [{ tsNanos: "50", value: 3 }] }],
        },
      })
      .mockResolvedValueOnce({ metricExemplars: [] })
      .mockResolvedValueOnce({
        chartAnnotations: [{ tsNanos: "40", kind: "release", title: "v1", service: "checkout" }],
      })

    const detail = await loadMetricDetail("http.server.duration", {
      kind: "gauge",
      range: "custom",
      from: "0",
      to: "100",
    })

    const queries = vi.mocked(graphqlCached).mock.calls.map((call) => String(call[0]))
    const annotationQuery = queries.find((query) => query.includes("chartAnnotations"))
    expect(annotationQuery).toBeTruthy()
    expect(annotationQuery).toContain("chartAnnotations(fromNanos:")
    expect(annotationQuery).toContain("tsNanos kind title service")
    expect(annotationQuery).not.toMatch(/service:\s*"/)
    expect(queries.some((query) => query.includes("releases("))).toBe(false)
    expect(detail.annotations).toEqual([
      { tsNanos: "40", kind: "release", title: "v1", service: "checkout" },
    ])
    expect(detail.releases).toEqual([
      {
        version: "v1",
        firstSeenNanos: "40",
        lastSeenNanos: "40",
        spanCount: "0",
      },
    ])
  })

  it("passes service into chartAnnotations when the catalog carried one", async () => {
    vi.mocked(graphqlCached)
      .mockResolvedValueOnce({
        metricLabels: [],
        metricQuery: { series: [] },
      })
      .mockResolvedValueOnce({ metricExemplars: [] })
      .mockResolvedValueOnce({ chartAnnotations: [] })

    await loadMetricDetail("http.server.duration", {
      kind: "gauge",
      service: "checkout",
      range: "custom",
      from: "0",
      to: "100",
    })

    const annotationQuery = vi
      .mocked(graphqlCached)
      .mock.calls.map((call) => String(call[0]))
      .find((query) => query.includes("chartAnnotations"))
    expect(annotationQuery).toContain('service: "checkout"')
  })

  it("fetches the previous window with shiftSeconds when compare=previous", async () => {
    vi.mocked(graphqlCached)
      .mockResolvedValueOnce({
        metricLabels: [],
        metricQuery: {
          series: [{ groupValue: null, points: [{ tsNanos: "50", value: 3 }] }],
        },
      })
      .mockResolvedValueOnce({ metricExemplars: [] })
      .mockResolvedValueOnce({ chartAnnotations: [] })
      .mockResolvedValueOnce({
        metricQuery: {
          series: [{ groupValue: null, points: [{ tsNanos: "40", value: 1 }] }],
        },
      })

    const detail = await loadMetricDetail("http.server.duration", {
      kind: "gauge",
      range: "custom",
      from: "0",
      to: "3600000000000",
      compare: "previous",
    })

    const queries = vi.mocked(graphqlCached).mock.calls.map((call) => String(call[0]))
    const shifted = queries.filter((query) => query.includes("shiftSeconds:"))
    expect(shifted).toHaveLength(1)
    expect(shifted[0]).toContain("shiftSeconds: 3600")
    expect(shifted[0]).toContain('name: "http.server.duration"')
    expect(detail.previous).toEqual([{ groupValue: null, points: [{ tsNanos: "40", value: 1 }] }])
  })

  it("skips the shifted query and leaves previous empty by default", async () => {
    vi.mocked(graphqlCached)
      .mockResolvedValueOnce({
        metricLabels: [],
        metricQuery: { series: [] },
      })
      .mockResolvedValueOnce({ metricExemplars: [] })
      .mockResolvedValueOnce({ chartAnnotations: [] })

    const detail = await loadMetricDetail("http.server.duration", {
      kind: "gauge",
      range: "custom",
      from: "0",
      to: "3600000000000",
    })

    const queries = vi.mocked(graphqlCached).mock.calls.map((call) => String(call[0]))
    expect(queries.some((query) => query.includes("shiftSeconds:"))).toBe(false)
    expect(detail.previous).toEqual([])
  })
})
