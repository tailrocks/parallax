/* @vitest-environment jsdom */

import { afterEach, beforeEach, describe, expect, it, vi } from "vitest"

import {
  clearGraphqlOperationCache,
  executeCachedGraphqlOperation,
  executeGraphqlOperation,
} from "@/platform/graphql/client"
import { GraphqlBoundaryError } from "@/platform/graphql/error"
import {
  GraphqlContractStaticProbeDocument,
  GraphqlContractStaticProbeQuerySchema,
} from "@/platform/graphql/tests/fixtures/static-probe.generated"

const probeVariables = {
  fromNanos: "0",
  toNanos: "1",
  fingerprint: "abc",
  service: null,
  limit: 60,
}

const validData = {
  health: "ok",
  version: "0.1.0",
  otlpGrpcPort: 4317,
  otlpHttpPort: 4318,
  overview: { spanCount: "1", errorRate: 0, activeServices: 1 },
  issue: null,
  metricNames: ["cpu"],
  signalCountSeries: [{ tsNanos: "0", value: 1 }],
}

describe("executeGraphqlOperation", () => {
  beforeEach(() => {
    clearGraphqlOperationCache()
    vi.stubGlobal(
      "fetch",
      vi.fn(async () => Response.json({ data: validData }))
    )
  })

  afterEach(() => {
    clearGraphqlOperationCache()
    vi.unstubAllGlobals()
    vi.useRealTimers()
  })

  it("sends operationName, query, and variables", async () => {
    await executeGraphqlOperation(
      GraphqlContractStaticProbeDocument,
      GraphqlContractStaticProbeQuerySchema,
      probeVariables
    )
    expect(fetch).toHaveBeenCalledTimes(1)
    const init = vi.mocked(fetch).mock.calls[0]?.[1] as RequestInit
    const body = JSON.parse(String(init.body)) as {
      operationName: string
      query: string
      variables: Record<string, unknown>
    }
    expect(body.operationName).toBe("GraphqlContractStaticProbe")
    expect(body.query).toContain("query GraphqlContractStaticProbe")
    expect(body.variables).toMatchObject({
      fromNanos: "0",
      toNanos: "1",
      fingerprint: "abc",
    })
  })

  it("decodes a valid operation result", async () => {
    const result = await executeGraphqlOperation(
      GraphqlContractStaticProbeDocument,
      GraphqlContractStaticProbeQuerySchema,
      probeVariables
    )
    expect(result.health).toBe("ok")
    expect(result.issue).toBeNull()
  })

  it("rejects non-empty errors even when data is present", async () => {
    vi.stubGlobal(
      "fetch",
      vi.fn(async () => Response.json({ data: validData, errors: [{ message: "x" }] }))
    )
    await expect(
      executeGraphqlOperation(
        GraphqlContractStaticProbeDocument,
        GraphqlContractStaticProbeQuerySchema,
        probeVariables
      )
    ).rejects.toMatchObject({ code: "graphql-errors" satisfies string })
  })

  it("rejects missing data", async () => {
    vi.stubGlobal(
      "fetch",
      vi.fn(async () => Response.json({ errors: [] }))
    )
    await expect(
      executeGraphqlOperation(
        GraphqlContractStaticProbeDocument,
        GraphqlContractStaticProbeQuerySchema,
        probeVariables
      )
    ).rejects.toBeInstanceOf(GraphqlBoundaryError)
  })

  it("rejects schema-invalid operation data with bounded paths", async () => {
    vi.stubGlobal(
      "fetch",
      vi.fn(async () =>
        Response.json({
          data: { ...validData, health: 123 },
        })
      )
    )
    const error = await executeGraphqlOperation(
      GraphqlContractStaticProbeDocument,
      GraphqlContractStaticProbeQuerySchema,
      probeVariables
    ).then(
      () => {
        throw new Error("must throw")
      },
      (caught: unknown) => caught
    )
    expect(error).toBeInstanceOf(GraphqlBoundaryError)
    const boundary = error as GraphqlBoundaryError
    expect(boundary.code).toBe("invalid-operation-data")
    expect(boundary.schemaIssuePaths).toContain("health")
    expect(boundary.message).not.toContain("cpu")
    expect(boundary.message).not.toContain("abc")
  })

  it("maps HTTP failures", async () => {
    vi.stubGlobal(
      "fetch",
      vi.fn(async () => new Response("nope", { status: 503 }))
    )
    await expect(
      executeGraphqlOperation(
        GraphqlContractStaticProbeDocument,
        GraphqlContractStaticProbeQuerySchema,
        probeVariables
      )
    ).rejects.toMatchObject({ code: "http", status: 503 })
  })

  it("maps abort", async () => {
    const controller = new AbortController()
    controller.abort()
    await expect(
      executeGraphqlOperation(
        GraphqlContractStaticProbeDocument,
        GraphqlContractStaticProbeQuerySchema,
        probeVariables,
        { signal: controller.signal }
      )
    ).rejects.toMatchObject({ code: "abort" })
  })

  it("cached form dedupes concurrent identical operations", async () => {
    const [a, b] = await Promise.all([
      executeCachedGraphqlOperation(
        GraphqlContractStaticProbeDocument,
        GraphqlContractStaticProbeQuerySchema,
        probeVariables
      ),
      executeCachedGraphqlOperation(
        GraphqlContractStaticProbeDocument,
        GraphqlContractStaticProbeQuerySchema,
        probeVariables
      ),
    ])
    expect(a.health).toBe("ok")
    expect(b.health).toBe("ok")
    expect(fetch).toHaveBeenCalledTimes(1)
  })

  it("cached form serves TTL hit and refetches after expiry", async () => {
    vi.useFakeTimers()
    vi.setSystemTime(new Date("2026-01-01T00:00:00Z"))
    await executeCachedGraphqlOperation(
      GraphqlContractStaticProbeDocument,
      GraphqlContractStaticProbeQuerySchema,
      probeVariables
    )
    await executeCachedGraphqlOperation(
      GraphqlContractStaticProbeDocument,
      GraphqlContractStaticProbeQuerySchema,
      probeVariables
    )
    expect(fetch).toHaveBeenCalledTimes(1)
    vi.setSystemTime(new Date("2026-01-01T00:00:16Z"))
    await executeCachedGraphqlOperation(
      GraphqlContractStaticProbeDocument,
      GraphqlContractStaticProbeQuerySchema,
      probeVariables
    )
    expect(fetch).toHaveBeenCalledTimes(2)
  })

  it("rejects non-object envelopes", async () => {
    vi.stubGlobal(
      "fetch",
      vi.fn(async () => Response.json(["not", "an", "object"]))
    )
    await expect(
      executeGraphqlOperation(
        GraphqlContractStaticProbeDocument,
        GraphqlContractStaticProbeQuerySchema,
        probeVariables
      )
    ).rejects.toMatchObject({ code: "invalid-envelope" })
  })
})
