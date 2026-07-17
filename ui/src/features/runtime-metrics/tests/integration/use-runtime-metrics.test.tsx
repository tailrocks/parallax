/* @vitest-environment jsdom */

import { act, cleanup, renderHook, waitFor } from "@testing-library/react"
import { afterEach, describe, expect, it, vi } from "vitest"

import { useRuntimeMetrics } from "@/features/runtime-metrics/hooks/use-runtime-metrics"
import { executeGraphqlOperation } from "@/platform/graphql/client"

vi.mock("@/platform/graphql/client", () => ({
  executeGraphqlOperation: vi.fn(),
}))

vi.mock("@/platform/visibility/use-page-visible", () => ({
  usePageVisible: () => true,
}))

afterEach(() => {
  cleanup()
  vi.clearAllMocks()
  vi.useRealTimers()
})

describe("useRuntimeMetrics", () => {
  it("loads once when visible and clears panels on failure", async () => {
    vi.mocked(executeGraphqlOperation)
      .mockResolvedValueOnce({
        cpu: [{ points: [{ tsNanos: "1", value: 0.1 }] }],
        memory: [{ points: [] }],
        tasks: [{ points: [] }],
      })
      .mockRejectedValueOnce(new Error("boom"))

    const { result, rerender } = renderHook(
      (props: { fromNanos: string }) =>
        useRuntimeMetrics({
          service: "api",
          fromNanos: props.fromNanos,
          toNanos: "2",
          stepSeconds: 30,
        }),
      { initialProps: { fromNanos: "1" } }
    )

    await waitFor(() => expect(result.current?.[0]?.points).toHaveLength(1))
    expect(executeGraphqlOperation).toHaveBeenCalledTimes(1)

    rerender({ fromNanos: "3" })
    await waitFor(() => expect(result.current).toEqual([]))
    expect(executeGraphqlOperation).toHaveBeenCalledTimes(2)
  })

  it("polls every five seconds while live", async () => {
    vi.useFakeTimers()
    vi.mocked(executeGraphqlOperation).mockResolvedValue({
      cpu: [],
      memory: [],
      tasks: [{ points: [{ tsNanos: "1", value: 1 }] }],
    })

    renderHook(() =>
      useRuntimeMetrics({
        service: "api",
        fromNanos: "1",
        toNanos: "2",
        stepSeconds: 30,
        live: true,
      })
    )

    await act(async () => {
      await Promise.resolve()
    })
    expect(executeGraphqlOperation).toHaveBeenCalledTimes(1)

    await act(async () => {
      await vi.advanceTimersByTimeAsync(5000)
    })
    expect(executeGraphqlOperation).toHaveBeenCalledTimes(2)
  })
})
