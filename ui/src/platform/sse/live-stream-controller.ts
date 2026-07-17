// Plan 153 — SSE lifecycle controller (one source, timer, generation, buffer).

import { decodeJsonText } from "@/platform/external-values/decode-json-text"
import type { RuntimeDecoder } from "@/platform/external-values/runtime-decoder"
import { reportBoundaryError } from "@/platform/external-values/boundary-diagnostic"
import { boundaryError } from "@/platform/external-values/boundary-error"
import {
  browserEventSourceFactory,
  type EventSourceFactory,
  type EventSourceLike,
} from "@/platform/sse/event-source.client"

export type LiveStreamStatus = "idle" | "connecting" | "open" | "error"

export interface LiveStreamControllerOptions<T> {
  readonly url: string | null
  readonly decoder: RuntimeDecoder<T[]>
  readonly onBatch: (items: T[]) => void
  readonly onStatus?: (status: LiveStreamStatus) => void
  readonly flushMs?: number
  readonly visible?: boolean
  readonly signal?: AbortSignal
  readonly eventSourceFactory?: EventSourceFactory
}

export interface LiveStreamController {
  readonly dispose: () => void
  readonly setUrl: (url: string | null) => void
  readonly setVisible: (visible: boolean) => void
}

/**
 * Owns one EventSource, one flush timer, one generation, and one buffer.
 * Null URL, hidden document, abort, URL change, or disposal closes everything
 * and prevents late delivery. Native EventSource reconnect is preserved.
 */
export function createLiveStreamController<T>(
  options: LiveStreamControllerOptions<T>
): LiveStreamController {
  const flushMs = options.flushMs ?? 250
  const factory = options.eventSourceFactory ?? browserEventSourceFactory
  let url = options.url
  let visible = options.visible ?? true
  let generation = 0
  let source: EventSourceLike | null = null
  let timer: ReturnType<typeof setInterval> | null = null
  let buffer: T[] = []
  let disposed = false

  const setStatus = (status: LiveStreamStatus) => {
    options.onStatus?.(status)
  }

  const clearTimer = () => {
    if (timer !== null) {
      clearInterval(timer)
      timer = null
    }
  }

  const closeSource = () => {
    if (source) {
      source.onopen = null
      source.onerror = null
      source.onmessage = null
      source.close()
      source = null
    }
  }

  const invalidate = () => {
    generation += 1
    buffer = []
    clearTimer()
    closeSource()
  }

  const reconnect = () => {
    if (disposed) return
    invalidate()
    const activeUrl = visible ? url : null
    if (!activeUrl || options.signal?.aborted) {
      setStatus("idle")
      return
    }
    const gen = generation
    setStatus("connecting")
    const next = factory(activeUrl)
    source = next
    next.onopen = () => {
      if (gen !== generation || disposed) return
      setStatus("open")
    }
    next.onerror = () => {
      if (gen !== generation || disposed) return
      setStatus("error")
    }
    next.onmessage = (event: MessageEvent) => {
      if (gen !== generation || disposed) return
      const data: unknown = event.data
      const decoded = decodeJsonText(data, options.decoder)
      if (!decoded.ok) {
        // Malformed feature frame: skip + diagnose; keep stream open.
        return
      }
      if (decoded.value.length > 0) {
        buffer.push(...decoded.value)
      }
    }
    timer = setInterval(() => {
      if (gen !== generation || disposed) return
      if (buffer.length === 0) return
      const incoming = buffer
      buffer = []
      options.onBatch(incoming)
    }, flushMs)
  }

  const onAbort = () => {
    if (disposed) return
    invalidate()
    setStatus("idle")
    reportBoundaryError(boundaryError("sse.live-stream", "cancelled", null))
  }

  options.signal?.addEventListener("abort", onAbort, { once: true })
  reconnect()

  return {
    dispose() {
      if (disposed) return
      disposed = true
      options.signal?.removeEventListener("abort", onAbort)
      invalidate()
      setStatus("idle")
    },
    setUrl(next) {
      if (disposed || next === url) return
      url = next
      reconnect()
    },
    setVisible(next) {
      if (disposed || next === visible) return
      visible = next
      reconnect()
    },
  }
}
