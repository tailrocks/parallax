import { useEffect, useRef, useState } from "react"

import { boundaryError } from "@/platform/external-values/boundary-error"
import { reportBoundaryError } from "@/platform/external-values/boundary-diagnostic"
import type { RuntimeDecoder } from "@/platform/external-values/runtime-decoder"
import {
  browserEventSourceFactory,
  type EventSourceFactory,
} from "@/platform/sse/event-source.client"
import {
  createLiveStreamController,
  type LiveStreamStatus,
} from "@/platform/sse/live-stream-controller"
import { usePageVisible } from "@/platform/visibility/use-page-visible"

export type { LiveStreamStatus }

export interface UseLiveStreamOptions<T> {
  /** Full stream URL incl. query params; null disables the stream. */
  url: string | null
  /**
   * Parse one SSE frame payload into items; return [] to skip.
   * Kept for Plans 140-142 until they supply a RuntimeDecoder.
   */
  parse?: (data: string) => T[]
  /** Unknown-first frame decoder producing a batch array. */
  decoder?: RuntimeDecoder<T[]>
  /** Called with each flushed batch (caller owns ordering). */
  onBatch: (items: T[]) => void
  flushMs?: number
  signal?: AbortSignal
  eventSourceFactory?: EventSourceFactory
}

/**
 * Shared SSE consumer. Closes the EventSource and pauses the flush timer while
 * the tab is hidden; reconnects when the page becomes visible again.
 */
export function useLiveStream<T>({
  url,
  parse,
  decoder,
  onBatch,
  flushMs = 250,
  signal,
  eventSourceFactory = browserEventSourceFactory,
}: UseLiveStreamOptions<T>): LiveStreamStatus {
  const [status, setStatus] = useState<LiveStreamStatus>("idle")
  const onBatchRef = useRef(onBatch)
  onBatchRef.current = onBatch
  const parseRef = useRef(parse)
  parseRef.current = parse
  const decoderRef = useRef(decoder)
  decoderRef.current = decoder
  const visible = usePageVisible()

  useEffect(() => {
    // Legacy parse path: EventSource delivers strings; parse owns JSON.
    // Prefer decoder when provided (unknown-first).
    if (!decoderRef.current) {
      const legacyOptions: {
        url: string | null
        visible: boolean
        flushMs: number
        signal?: AbortSignal
        factory: EventSourceFactory
        parse: (data: string) => T[]
        onBatch: (items: T[]) => void
        onStatus: (status: LiveStreamStatus) => void
      } = {
        url,
        visible,
        flushMs,
        factory: eventSourceFactory,
        parse: (data) => {
          const current = parseRef.current
          if (!current) return []
          return current(data)
        },
        onBatch: (items) => onBatchRef.current(items),
        onStatus: setStatus,
      }
      if (signal) legacyOptions.signal = signal
      return mountLegacyParseController(legacyOptions)
    }

    const controller = createLiveStreamController<T>({
      url,
      decoder: {
        safeParse(input: unknown) {
          const current = decoderRef.current
          if (!current) return { success: false, error: "missing-decoder" }
          return current.safeParse(input)
        },
      },
      onBatch: (items) => onBatchRef.current(items),
      onStatus: setStatus,
      flushMs,
      visible,
      eventSourceFactory,
      ...(signal ? { signal } : {}),
    })
    return () => controller.dispose()
  }, [url, flushMs, visible, signal, eventSourceFactory])

  return status
}

function mountLegacyParseController<T>(options: {
  url: string | null
  visible: boolean
  flushMs: number
  signal?: AbortSignal
  factory: EventSourceFactory
  parse: (data: string) => T[]
  onBatch: (items: T[]) => void
  onStatus: (status: LiveStreamStatus) => void
}): () => void {
  const activeUrl = options.visible ? options.url : null
  if (!activeUrl || options.signal?.aborted) {
    options.onStatus("idle")
    return () => undefined
  }

  let disposed = false
  let buffer: T[] = []
  options.onStatus("connecting")
  const source = options.factory(activeUrl)
  source.onopen = () => {
    if (!disposed) options.onStatus("open")
  }
  source.onerror = () => {
    if (!disposed) options.onStatus("error")
  }
  source.onmessage = (event: MessageEvent) => {
    if (disposed) return
    try {
      const data = typeof event.data === "string" ? event.data : ""
      buffer.push(...options.parse(data))
    } catch {
      reportBoundaryError(boundaryError("sse.live-stream", "schema-rejected", "frame"))
    }
  }
  const flush = setInterval(() => {
    if (disposed || buffer.length === 0) return
    const incoming = buffer
    buffer = []
    options.onBatch(incoming)
  }, options.flushMs)

  const onAbort = () => cleanup()
  options.signal?.addEventListener("abort", onAbort)

  function cleanup() {
    if (disposed) return
    disposed = true
    options.signal?.removeEventListener("abort", onAbort)
    source.onopen = null
    source.onerror = null
    source.onmessage = null
    source.close()
    clearInterval(flush)
    buffer = []
    options.onStatus("idle")
  }

  return cleanup
}
