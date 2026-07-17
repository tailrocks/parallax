// Plan 147 — capacity-bounded flush buffer for live SSE frames.

export interface BoundedBufferOptions {
  readonly maxBufferedItems: number
  /** When full, drop oldest (true) or newest (false). Default: drop oldest. */
  readonly dropOldest?: boolean
}

export interface BoundedBufferDiagnostics {
  readonly dropped: number
  readonly highWater: number
}

export interface BoundedFrameBuffer<T> {
  push(items: readonly T[]): void
  flush(): T[]
  clear(): void
  readonly size: number
  readonly diagnostics: BoundedBufferDiagnostics
}

export function createBoundedFrameBuffer<T>(options: BoundedBufferOptions): BoundedFrameBuffer<T> {
  const max = Math.max(1, options.maxBufferedItems)
  const dropOldest = options.dropOldest ?? true
  let buffer: T[] = []
  let dropped = 0
  let highWater = 0

  return {
    push(items) {
      if (items.length === 0) return
      buffer.push(...items)
      if (buffer.length > max) {
        const overflow = buffer.length - max
        dropped += overflow
        buffer = dropOldest ? buffer.slice(overflow) : buffer.slice(0, max)
      }
      if (buffer.length > highWater) highWater = buffer.length
    },
    flush() {
      if (buffer.length === 0) return []
      const out = buffer
      buffer = []
      return out
    },
    clear() {
      buffer = []
    },
    get size() {
      return buffer.length
    },
    get diagnostics() {
      return { dropped, highWater }
    },
  }
}
