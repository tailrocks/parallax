/* @vitest-environment jsdom */

import { act, cleanup, render, screen, waitFor } from "@testing-library/react"
import { afterEach, describe, expect, it, vi } from "vitest"

import { MetricStrip } from "@/features/runtime-metrics"
import { executeGraphqlOperation } from "@/platform/graphql/client"

vi.mock("@/platform/graphql/client", () => ({
  executeGraphqlOperation: vi.fn(),
}))

type MetricPayload = {
  cpu: Array<{ points: Array<{ tsNanos: string; value: number }> }>
  memory: Array<{ points: Array<{ tsNanos: string; value: number }> }>
  tasks: Array<{ points: Array<{ tsNanos: string; value: number }> }>
}

interface GraphqlMock {
  mockImplementationOnce: (
    implementation: (
      document: unknown,
      schema: unknown,
      variables: unknown,
      options?: { signal?: AbortSignal }
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
    const mocked = executeGraphqlOperation as unknown as GraphqlMock
    mocked
      .mockImplementationOnce((_doc, _schema, _vars, options) => {
        if (options?.signal) signals.push(options.signal)
        return first.promise
      })
      .mockImplementationOnce((_doc, _schema, _vars, options) => {
        if (options?.signal) signals.push(options.signal)
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
