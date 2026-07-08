/* @vitest-environment jsdom */

import { act, cleanup, render, screen, waitFor } from "@testing-library/react"
import { afterEach, describe, expect, it, vi } from "vitest"

import { MetricStrip } from "@/components/metric-strip"
import { graphql } from "@/lib/api"

vi.mock("@/lib/api", () => ({
  gqlString: (value: string) =>
    value.replace(/\\/g, "\\\\").replace(/"/g, '\\"').replace(/\n/g, "\\n"),
  graphql: vi.fn(),
}))

interface MetricPoint {
  tsNanos: string
  value: number
}

type MetricPayload = Record<
  "cpu" | "memory" | "tasks",
  Array<{ points: MetricPoint[] }> | undefined
>

interface GraphqlMock {
  mockImplementationOnce: (
    implementation: (
      query: string,
      init?: { signal?: AbortSignal }
    ) => Promise<MetricPayload>
  ) => GraphqlMock
}

function deferred<T>() {
  let resolve: (value: T) => void = () => {}
  let reject: (reason?: unknown) => void = () => {}
  const promise = new Promise<T>((res, rej) => {
    resolve = res
    reject = rej
  })
  return { promise, resolve, reject }
}

afterEach(() => {
  cleanup()
  vi.clearAllMocks()
})

describe("MetricStrip", () => {
  it("aborts stale requests and ignores late responses", async () => {
    const first = deferred<MetricPayload>()
    const second = deferred<MetricPayload>()
    const signals: AbortSignal[] = []
    const mockedGraphql = graphql as unknown as GraphqlMock
    mockedGraphql
      .mockImplementationOnce((_query, init) => {
        if (init?.signal) signals.push(init.signal)
        return first.promise
      })
      .mockImplementationOnce((_query, init) => {
        if (init?.signal) signals.push(init.signal)
        return second.promise
      })

    const rendered = render(
      <MetricStrip
        title="Metrics"
        service="api-a"
        fromNanos="1"
        toNanos="2"
        stepSeconds={30}
      />
    )

    rendered.rerender(
      <MetricStrip
        title="Metrics"
        service="api-b"
        fromNanos="1"
        toNanos="2"
        stepSeconds={30}
      />
    )
    expect(signals[0]?.aborted).toBe(true)

    await act(async () => {
      second.resolve({
        cpu: [{ points: [] }],
        memory: [{ points: [] }],
        tasks: [{ points: [{ tsNanos: "2", value: 3 }] }],
      })
    })
    expect(await screen.findByText("Tokio alive tasks")).toBeTruthy()

    await act(async () => {
      first.resolve({
        cpu: [{ points: [{ tsNanos: "1", value: 0.5 }] }],
        memory: [{ points: [] }],
        tasks: [{ points: [] }],
      })
    })
    await waitFor(() => expect(screen.queryByText("CPU (%)")).toBeNull())
    expect(screen.getByText("Tokio alive tasks")).toBeTruthy()

    rendered.unmount()
    expect(signals[1]?.aborted).toBe(true)
  })
})
