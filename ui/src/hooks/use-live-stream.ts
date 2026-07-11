import { useEffect, useRef, useState } from "react"

export type LiveStreamStatus = "idle" | "connecting" | "open" | "error"

export interface UseLiveStreamOptions<T> {
  /** Full stream URL incl. query params; null/undefined disables the stream. */
  url: string | null
  /** Parse one SSE frame's payload into items; return [] to skip. */
  parse: (data: string) => T[]
  /** Called with each flushed batch (newest-first NOT applied — caller owns ordering). */
  onBatch: (items: T[]) => void
  flushMs?: number
}

export function useLiveStream<T>({
  url,
  parse,
  onBatch,
  flushMs = 250,
}: UseLiveStreamOptions<T>): LiveStreamStatus {
  const [status, setStatus] = useState<LiveStreamStatus>("idle")
  const onBatchRef = useRef(onBatch)
  onBatchRef.current = onBatch
  const parseRef = useRef(parse)
  parseRef.current = parse

  useEffect(() => {
    if (!url) {
      setStatus("idle")
      return
    }
    setStatus("connecting")
    const source = new EventSource(url)
    let buffer: T[] = []
    source.onopen = () => setStatus("open")
    // EventSource retries natively; state flips back to open on reopen.
    source.onerror = () => setStatus("error")
    source.onmessage = (event) => {
      try {
        buffer.push(...parseRef.current(event.data as string))
      } catch {
        // skip malformed frames
      }
    }
    const flush = setInterval(() => {
      if (buffer.length === 0) return
      const incoming = buffer
      buffer = []
      onBatchRef.current(incoming)
    }, flushMs)
    return () => {
      source.close()
      clearInterval(flush)
      setStatus("idle")
    }
  }, [url, flushMs])

  return status
}
