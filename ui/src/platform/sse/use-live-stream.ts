import { useEffect, useRef, useState } from "react"

import type { RuntimeDecoder } from "@/platform/external-values/runtime-decoder"
import { browserEventSourceFactory, type EventSourceFactory } from "@/platform/sse/event-source"
import {
  createLiveStreamController,
  type LiveStreamStatus,
} from "@/platform/sse/live-stream-controller"
import { usePageVisible } from "@/platform/visibility/use-page-visible"

export type { LiveStreamStatus }

export interface UseLiveStreamOptions<T> {
  /** Full stream URL incl. query params; null disables the stream. */
  url: string | null
  /** Unknown-first frame decoder producing a batch array. Required. */
  decoder: RuntimeDecoder<T[]>
  /** Called with each flushed batch (caller owns ordering). */
  onBatch: (items: T[]) => void
  flushMs?: number
  signal?: AbortSignal
  eventSourceFactory?: EventSourceFactory
  /** Arrival buffer cap between flushes (plan 147). */
  maxBufferedItems?: number
}

/**
 * Shared SSE consumer. Closes the EventSource and pauses the flush timer while
 * the tab is hidden; reconnects when the page becomes visible again.
 * Every frame is decoded from `unknown` via the feature-owned RuntimeDecoder.
 */
export function useLiveStream<T>({
  url,
  decoder,
  onBatch,
  flushMs = 250,
  signal,
  eventSourceFactory = browserEventSourceFactory,
  maxBufferedItems,
}: UseLiveStreamOptions<T>): LiveStreamStatus {
  const [status, setStatus] = useState<LiveStreamStatus>("idle")
  const onBatchRef = useRef(onBatch)
  onBatchRef.current = onBatch
  const decoderRef = useRef(decoder)
  decoderRef.current = decoder
  const visible = usePageVisible()

  useEffect(() => {
    const controller = createLiveStreamController<T>({
      url,
      decoder: {
        safeParse(input: unknown) {
          const current = decoderRef.current
          return current.safeParse(input)
        },
      },
      onBatch: (items) => onBatchRef.current(items),
      onStatus: setStatus,
      flushMs,
      visible,
      eventSourceFactory,
      ...(maxBufferedItems !== undefined ? { maxBufferedItems } : {}),
      ...(signal ? { signal } : {}),
    })
    return () => controller.dispose()
  }, [url, flushMs, visible, signal, eventSourceFactory, maxBufferedItems])

  return status
}
