import { afterEach, describe, expect, it, vi } from "vitest"

import { loadRuntimeMetricStrip } from "@/features/runtime-metrics/api/load-runtime-metrics"
import { mapRuntimeMetricStrip } from "@/features/runtime-metrics/api/runtime-metrics-mapper"
import { isRuntimeMetricsAbort } from "@/features/runtime-metrics/model/runtime-metrics-error"
import { executeGraphqlOperation } from "@/platform/graphql/client"
import { GraphqlBoundaryError } from "@/platform/graphql/error"

vi.mock("@/platform/graphql/client", () => ({
  executeGraphqlOperation: vi.fn(),
}))

afterEach(() => {
  vi.clearAllMocks()
})

describe("mapRuntimeMetricStrip", () => {
  it("maps cpu ratio to percent and preserves empty series", () => {
    const panels = mapRuntimeMetricStrip({
      cpu: [{ points: [{ tsNanos: "1", value: 0.5 }] }],
      memory: [],
      tasks: [{ points: [{ tsNanos: "2", value: 9 }] }],
    })
    expect(panels[0]).toMatchObject({
      key: "cpu",
      unit: "%",
      points: [{ tsNanos: "1", value: 50 }],
    })
    expect(panels[1]?.points).toEqual([])
    expect(panels[2]?.points).toEqual([{ tsNanos: "2", value: 9 }])
  })
})

describe("loadRuntimeMetricStrip", () => {
  it("prefers invocationId over service and uses raw transport", async () => {
    vi.mocked(executeGraphqlOperation).mockResolvedValueOnce({
      cpu: [],
      memory: [],
      tasks: [],
    })
    await loadRuntimeMetricStrip({
      service: "api",
      invocationId: "run-1",
      fromNanos: "1",
      toNanos: "2",
      stepSeconds: 30,
    })
    expect(executeGraphqlOperation).toHaveBeenCalledTimes(1)
    const variables = vi.mocked(executeGraphqlOperation).mock.calls[0]?.[2]
    expect(variables).toMatchObject({
      fromNanos: "1",
      toNanos: "2",
      stepSeconds: 30,
      service: null,
      invocationId: "run-1",
    })
  })

  it("passes service when invocationId is absent", async () => {
    vi.mocked(executeGraphqlOperation).mockResolvedValueOnce({
      cpu: [],
      memory: [],
      tasks: [],
    })
    await loadRuntimeMetricStrip({
      service: "api",
      fromNanos: "1",
      toNanos: "2",
      stepSeconds: 15,
    })
    const variables = vi.mocked(executeGraphqlOperation).mock.calls[0]?.[2]
    expect(variables).toMatchObject({
      service: "api",
      invocationId: null,
    })
  })
})

describe("isRuntimeMetricsAbort", () => {
  it("recognizes GraphQL abort and DOM AbortError", () => {
    expect(
      isRuntimeMetricsAbort(
        new GraphqlBoundaryError({
          code: "abort",
          operationName: "RuntimeMetricStrip",
          status: null,
          schemaIssueCount: null,
          schemaIssuePaths: null,
        })
      )
    ).toBe(true)
    expect(isRuntimeMetricsAbort(new DOMException("aborted", "AbortError"))).toBe(true)
    expect(isRuntimeMetricsAbort(new Error("network"))).toBe(false)
  })
})
