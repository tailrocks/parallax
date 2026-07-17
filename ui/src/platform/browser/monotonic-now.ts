// Plan 100/153 — sole handwritten owner of performance.now / monotonic clock.

/** Monotonic milliseconds suitable for elapsed-time measurement. */
export function monotonicNowMs(): number {
  if (
    typeof performance !== "undefined" &&
    typeof performance.now === "function"
  ) {
    return performance.now()
  }
  return Date.now()
}
