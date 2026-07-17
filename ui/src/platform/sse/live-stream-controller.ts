// Plan 153/147 — SSE lifecycle controller (one source, timer, generation, buffer).

import { decodeJsonText } from "@/platform/external-values/decode-json-text"
import type { RuntimeDecoder } from "@/platform/external-values/runtime-decoder"
import { reportBoundaryError } from "@/platform/external-values/boundary-diagnostic"
import { boundaryError } from "@/platform/external-values/boundary-error"
import {
  browserEventSourceFactory,
  type EventSourceFactory,
  type EventSourceLike,
} from "@/platform/sse/event-source.client"
import { createBoundedFrameBuffer } from "@/platform/sse/bounded-frame-buffer"
import {
  initialStreamStatus,
  reduceStreamStatus,
  type LiveStreamStatus,
} from "@/platform/sse/stream-state"

export type { LiveStreamStatus }

/** Default arrival buffer cap (plan 147) — not the feature visible cap. */
export const DEFAULT_MAX_BUFFERED_ITEMS = 2_000

export interface LiveStreamControllerOptions<T> {
  readonly url: string | null
  readonly decoder: RuntimeDecoder<T[]>
  readonly onBatch: (items: T[]) => void
  readonly onStatus?: (status: LiveStreamStatus) => void
  readonly flushMs?: number
  readonly visible?: boolean
  readonly signal?: AbortSignal
  readonly eventSourceFactory?: EventSourceFactory
  /** Max items held between flushes. Overflow drops oldest. */
  readonly maxBufferedItems?: number
}

export interface LiveStreamController {
  readonly dispose: () => void
  readonly setUrl: (url: string | null) => void
  readonly setVisible: (visible: boolean) => void
  readonly status: () => LiveStreamStatus
}

/**
 * Owns one EventSource, one flush timer, one generation, and one buffer.
 * Null URL, hidden document, abort, URL change, or disposal closes everything
 * and prevents late delivery. Native EventSource reconnect is preserved;
 * transport errors after open surface as `reconnecting`.
 */
export function createLiveStreamController<T>(
  options: LiveStreamControllerOptions<T>
): LiveStreamController {
  const flushMs = options.flushMs ?? 250
  const factory = options.eventSourceFactory ?? browserEventSourceFactory
  const frameBuffer = createBoundedFrameBuffer<T>({
    maxBufferedItems: options.maxBufferedItems ?? DEFAULT_MAX_BUFFERED_ITEMS,
    dropOldest: true,
  })
  let url = options.url
  let visible = options.visible ?? true
  let generation = 0
  let source: EventSourceLike | null = null
  let timer: ReturnType<typeof setInterval> | null = null
  let disposed = false
  let status: LiveStreamStatus = initialStreamStatus()

  const setStatus = (next: LiveStreamStatus) => {
    status = next
    options.onStatus?.(next)
  }

  const applyEvent = (event: Parameters<typeof reduceStreamStatus>[1]) => {
    setStatus(reduceStreamStatus(status, event))
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
    frameBuffer.clear()
    clearTimer()
    closeSource()
  }

  const reconnect = () => {
    if (disposed) return
    invalidate()
    const activeUrl = visible ? url : null
    if (!activeUrl || options.signal?.aborted) {
      applyEvent({ type: "stop" })
      return
    }
    const gen = generation
    applyEvent({ type: "start" })
    const next = factory(activeUrl)
    source = next
    next.onopen = () => {
      if (gen !== generation || disposed) return
      applyEvent({ type: "opened" })
    }
    next.onerror = () => {
      if (gen !== generation || disposed) return
      applyEvent({ type: "transport-error" })
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
        frameBuffer.push(decoded.value)
      }
    }
    timer = setInterval(() => {
      if (gen !== generation || disposed) return
      if (frameBuffer.size === 0) return
      const incoming = frameBuffer.flush()
      options.onBatch(incoming)
    }, flushMs)
  }

  const onAbort = () => {
    if (disposed) return
    invalidate()
    applyEvent({ type: "stop" })
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
      applyEvent({ type: "stop" })
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
    status: () => status,
  }
}
