/* @vitest-environment jsdom */

import { act, cleanup, render, screen } from "@testing-library/react"
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest"

import { useLiveStream } from "@/hooks/use-live-stream"
import type { LiveStreamStatus } from "@/hooks/use-live-stream"

type Handler = ((event: MessageEvent) => void) | null
type VoidHandler = (() => void) | null

class MockEventSource {
  static instances: MockEventSource[] = []
  static constructedUrls: string[] = []

  readonly url: string
  onopen: VoidHandler = null
  onerror: VoidHandler = null
  onmessage: Handler = null
  closed = false

  constructor(url: string) {
    this.url = url
    MockEventSource.instances.push(this)
    MockEventSource.constructedUrls.push(url)
  }

  close() {
    this.closed = true
  }

  emitOpen() {
    this.onopen?.()
  }

  emitError() {
    this.onerror?.()
  }

  emitMessage(data: string) {
    this.onmessage?.({ data } as MessageEvent)
  }
}

function StatusHarness({
  url,
  parse,
  onBatch,
  flushMs,
}: {
  url: string | null
  parse: (data: string) => string[]
  onBatch: (items: string[]) => void
  flushMs?: number
}) {
  const options =
    flushMs === undefined
      ? { url, parse, onBatch }
      : { url, parse, onBatch, flushMs }
  const status: LiveStreamStatus = useLiveStream(options)
  return <output data-testid="status">{status}</output>
}

beforeEach(() => {
  MockEventSource.instances = []
  MockEventSource.constructedUrls = []
  vi.stubGlobal("EventSource", MockEventSource)
  vi.useFakeTimers()
})

afterEach(() => {
  cleanup()
  vi.useRealTimers()
  vi.unstubAllGlobals()
})

describe("useLiveStream", () => {
  it("buffers frames and flushes after flushMs via onBatch", () => {
    const onBatch = vi.fn()
    const parse = (data: string) => JSON.parse(data) as string[]

    render(
      <StatusHarness
        url="/v1/logs/stream"
        parse={parse}
        onBatch={onBatch}
        flushMs={250}
      />
    )

    const source = MockEventSource.instances[0]
    expect(source).toBeDefined()
    act(() => {
      source!.emitMessage(JSON.stringify(["a", "b"]))
      source!.emitMessage(JSON.stringify(["c"]))
    })

    expect(onBatch).not.toHaveBeenCalled()
    act(() => {
      vi.advanceTimersByTime(250)
    })
    expect(onBatch).toHaveBeenCalledTimes(1)
    expect(onBatch).toHaveBeenCalledWith(["a", "b", "c"])
  })

  it("skips malformed frames when parse throws without killing the stream", () => {
    const onBatch = vi.fn()
    const parse = (data: string) => {
      if (data === "bad") throw new Error("malformed")
      return JSON.parse(data) as string[]
    }

    render(
      <StatusHarness
        url="/v1/logs/stream"
        parse={parse}
        onBatch={onBatch}
        flushMs={250}
      />
    )

    const source = MockEventSource.instances[0]!
    act(() => {
      source.emitMessage("bad")
      source.emitMessage(JSON.stringify(["ok"]))
    })

    act(() => {
      vi.advanceTimersByTime(250)
    })
    expect(onBatch).toHaveBeenCalledTimes(1)
    expect(onBatch).toHaveBeenCalledWith(["ok"])
    expect(source.closed).toBe(false)
  })

  it("maps onerror to error and subsequent onopen to open", () => {
    const onBatch = vi.fn()
    render(
      <StatusHarness
        url="/v1/logs/stream"
        parse={() => []}
        onBatch={onBatch}
      />
    )

    expect(screen.getByTestId("status").textContent).toBe("connecting")

    const source = MockEventSource.instances[0]!
    act(() => {
      source.emitError()
    })
    expect(screen.getByTestId("status").textContent).toBe("error")

    act(() => {
      source.emitOpen()
    })
    expect(screen.getByTestId("status").textContent).toBe("open")
  })

  it("closes the source and clears the interval on unmount", () => {
    const clearIntervalSpy = vi.spyOn(globalThis, "clearInterval")
    const onBatch = vi.fn()

    const { unmount } = render(
      <StatusHarness
        url="/v1/logs/stream"
        parse={() => ["x"]}
        onBatch={onBatch}
        flushMs={250}
      />
    )

    const source = MockEventSource.instances[0]!
    act(() => {
      source.emitMessage(JSON.stringify(["x"]))
    })
    unmount()

    expect(source.closed).toBe(true)
    expect(clearIntervalSpy).toHaveBeenCalled()

    // Flushes after unmount must not deliver batches.
    act(() => {
      vi.advanceTimersByTime(250)
    })
    expect(onBatch).not.toHaveBeenCalled()
  })

  it("stays idle and constructs no EventSource when url is null", () => {
    const onBatch = vi.fn()
    render(
      <StatusHarness url={null} parse={() => []} onBatch={onBatch} />
    )

    expect(screen.getByTestId("status").textContent).toBe("idle")
    expect(MockEventSource.constructedUrls).toEqual([])
    expect(MockEventSource.instances).toHaveLength(0)
  })

  it("closes the stream while the tab is hidden and reconnects on visible", () => {
    let hidden = false
    Object.defineProperty(document, "hidden", {
      configurable: true,
      get: () => hidden,
    })

    const onBatch = vi.fn()
    render(
      <StatusHarness
        url="/v1/logs/stream"
        parse={() => []}
        onBatch={onBatch}
      />
    )
    expect(MockEventSource.instances).toHaveLength(1)
    const first = MockEventSource.instances[0]!

    act(() => {
      hidden = true
      document.dispatchEvent(new Event("visibilitychange"))
    })
    expect(first.closed).toBe(true)
    expect(screen.getByTestId("status").textContent).toBe("idle")

    act(() => {
      hidden = false
      document.dispatchEvent(new Event("visibilitychange"))
    })
    expect(MockEventSource.instances).toHaveLength(2)
    expect(MockEventSource.instances[1]!.closed).toBe(false)
  })
})
