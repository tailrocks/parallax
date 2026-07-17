import { Kind, print } from "graphql"
import { describe, expect, it, vi } from "vitest"

import { loadWidgetSeries } from "@/features/dashboards/api/widget-series-api"
import {
  WIDGET_SERIES_CHUNK,
  assertWidgetSeriesDocumentInvariants,
  buildWidgetSeriesChunks,
} from "@/features/dashboards/api/widget-series-operation"
import { GraphqlBoundaryError } from "@/platform/graphql/error"

const range = { fromNanos: "100", toNanos: "200" }

function widget(metric: string, index = 0) {
  return { metric, agg: "avg", groupBy: index % 2 === 0 ? "host" : null }
}

describe("buildWidgetSeriesChunks", () => {
  it("returns no chunks for an empty widget list", () => {
    expect(buildWidgetSeriesChunks([], range)).toEqual([])
  })

  it("builds one document for up to 24 widgets with series_<ordinal> aliases", () => {
    const widgets = Array.from({ length: 3 }, (_, i) => widget(`m${i}`, i))
    const chunks = buildWidgetSeriesChunks(widgets, range)
    expect(chunks).toHaveLength(1)
    const chunk = chunks[0]!
    expect(chunk.aliases).toEqual(["series_0", "series_1", "series_2"])
    assertWidgetSeriesDocumentInvariants(chunk.document)
    const printed = print(chunk.document)
    expect(printed).toContain("query DashboardWidgetSeries")
    expect(printed).toContain("series_0: metricSeries")
    expect(printed).not.toContain('"m0"')
    expect(chunk.variables["name_0"]).toBe("m0")
    expect(chunk.variables["from_0"]).toBe("100")
    expect(chunk.variables["groupBy_1"]).toBeNull()
  })

  it("splits at 24 and preserves order across chunks", () => {
    const widgets = Array.from({ length: 25 }, (_, i) => widget(`m${i}`, i))
    const chunks = buildWidgetSeriesChunks(widgets, range)
    expect(chunks).toHaveLength(2)
    expect(chunks[0]!.aliases).toHaveLength(WIDGET_SERIES_CHUNK)
    expect(chunks[1]!.aliases).toEqual(["series_24"])
    expect(chunks[0]!.ordinals[0]).toBe(0)
    expect(chunks[1]!.ordinals[0]).toBe(24)
  })

  it("uses only Variable argument nodes", () => {
    const [chunk] = buildWidgetSeriesChunks([widget("cpu")], range)
    const op = chunk!.document.definitions[0]
    expect(op?.kind).toBe(Kind.OPERATION_DEFINITION)
    if (op?.kind !== Kind.OPERATION_DEFINITION) return
    for (const selection of op.selectionSet.selections) {
      if (selection.kind !== Kind.FIELD) continue
      for (const arg of selection.arguments ?? []) {
        expect(arg.value.kind).toBe(Kind.VARIABLE)
      }
    }
  })
})

describe("loadWidgetSeries", () => {
  it("preserves result order for multi-chunk loads", async () => {
    const widgets = Array.from({ length: 25 }, (_, i) => widget(`m${i}`, i))
    const fetch = vi.fn(async (query: string) => {
      const aliases = [...query.matchAll(/series_(\d+)\s*:/g)].map((match) => `series_${match[1]}`)
      const data: Record<string, unknown> = {}
      for (const alias of aliases) {
        data[alias] = [
          {
            groupValue: alias,
            points: [{ tsNanos: "1", value: Number(alias.split("_")[1]) }],
          },
        ]
      }
      return data
    })

    const series = await loadWidgetSeries(widgets, range, fetch)
    expect(fetch).toHaveBeenCalledTimes(2)
    expect(series).toHaveLength(25)
    expect(series[0]![0]!.groupValue).toBe("series_0")
    expect(series[24]![0]!.groupValue).toBe("series_24")
  })

  it("rejects alias set mismatch", async () => {
    const fetch = vi.fn(async () => ({
      series_0: [],
      extra: [],
    }))
    await expect(loadWidgetSeries([widget("cpu")], range, fetch)).rejects.toBeInstanceOf(
      GraphqlBoundaryError
    )
  })

  it("rejects malformed series values without leaking payload", async () => {
    const fetch = vi.fn(async () => ({
      series_0: [{ groupValue: "x", points: [{ tsNanos: 1, value: "bad" }] }],
    }))
    const error = await loadWidgetSeries([widget("cpu")], range, fetch).then(
      () => {
        throw new Error("must throw")
      },
      (caught: unknown) => caught
    )
    expect(error).toBeInstanceOf(GraphqlBoundaryError)
    const message = (error as Error).message
    expect(message).not.toContain("bad")
    expect(message).not.toContain("secret-token")
  })

  it("makes no request for an empty widget list", async () => {
    const fetch = vi.fn(async () => ({}))
    await expect(loadWidgetSeries([], range, fetch)).resolves.toEqual([])
    expect(fetch).not.toHaveBeenCalled()
  })
})
