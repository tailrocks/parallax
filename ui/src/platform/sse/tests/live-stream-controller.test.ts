import { afterEach, beforeEach, describe, expect, it, vi } from "vitest"

import type { RuntimeDecoder } from "@/platform/external-values/runtime-decoder"
import type { EventSourceLike } from "@/platform/sse/event-source.client"
import { createLiveStreamController } from "@/platform/sse/live-stream-controller"

class FakeEventSource implements EventSourceLike {
  static instances: FakeEventSource[] = []
  onopen: ((this: EventSourceLike, ev: Event) => unknown) | null = null
  onerror: ((this: EventSourceLike, ev: Event) => unknown) | null = null
  onmessage: ((this: EventSourceLike, ev: MessageEvent) => unknown) | null = null
  closed = false
  readonly url: string
  constructor(url: string) {
    this.url = url
    FakeEventSource.instances.push(this)
  }
  close() {
    this.closed = true
  }
  emitMessage(data: string) {
    this.onmessage?.call(this, { data } as MessageEvent)
  }
}

const arrayDecoder: RuntimeDecoder<string[]> = {
  safeParse(input) {
    return Array.isArray(input) && input.every((item) => typeof item === "string")
      ? { success: true, data: input as string[] }
      : { success: false, error: "bad" }
  },
}

beforeEach(() => {
  FakeEventSource.instances = []
  vi.useFakeTimers()
})

afterEach(() => {
  vi.useRealTimers()
})

describe("createLiveStreamController", () => {
  it("buffers decoded frames and flushes on interval", () => {
    const onBatch = vi.fn()
    const controller = createLiveStreamController({
      url: "/stream",
      decoder: arrayDecoder,
      onBatch,
      flushMs: 250,
      eventSourceFactory: (url) => new FakeEventSource(url),
    })
    const source = FakeEventSource.instances[0]!
    source.emitMessage(JSON.stringify(["a"]))
    source.emitMessage(JSON.stringify(["b"]))
    expect(onBatch).not.toHaveBeenCalled()
    vi.advanceTimersByTime(250)
    expect(onBatch).toHaveBeenCalledWith(["a", "b"])
    controller.dispose()
  })

  it("skips malformed frames without closing", () => {
    const onBatch = vi.fn()
    createLiveStreamController({
      url: "/stream",
      decoder: arrayDecoder,
      onBatch,
      flushMs: 250,
      eventSourceFactory: (url) => new FakeEventSource(url),
    })
    const source = FakeEventSource.instances[0]!
    source.emitMessage("not-json")
    source.emitMessage(JSON.stringify(["ok"]))
    vi.advanceTimersByTime(250)
    expect(onBatch).toHaveBeenCalledWith(["ok"])
    expect(source.closed).toBe(false)
  })

  it("disposes source and timer; ignores late events", () => {
    const onBatch = vi.fn()
    const controller = createLiveStreamController({
      url: "/stream",
      decoder: arrayDecoder,
      onBatch,
      flushMs: 250,
      eventSourceFactory: (url) => new FakeEventSource(url),
    })
    const source = FakeEventSource.instances[0]!
    source.emitMessage(JSON.stringify(["x"]))
    controller.dispose()
    expect(source.closed).toBe(true)
    source.emitMessage(JSON.stringify(["late"]))
    vi.advanceTimersByTime(250)
    expect(onBatch).not.toHaveBeenCalled()
  })

  it("aborts via signal", () => {
    const onBatch = vi.fn()
    const abort = new AbortController()
    createLiveStreamController({
      url: "/stream",
      decoder: arrayDecoder,
      onBatch,
      signal: abort.signal,
      eventSourceFactory: (url) => new FakeEventSource(url),
    })
    expect(FakeEventSource.instances).toHaveLength(1)
    abort.abort()
    expect(FakeEventSource.instances[0]!.closed).toBe(true)
  })
})
