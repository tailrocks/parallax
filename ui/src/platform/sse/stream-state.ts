// Plan 147 — pure SSE connection state transitions.

/**
 * Discriminated connection lifecycle.
 * - idle: no URL / hidden / disposed
 * - connecting: EventSource constructed, waiting for open
 * - open: receiving frames
 * - reconnecting: transport error after open (native EventSource may retry)
 * - error: terminal/hard error while still owning a source, or failed connect
 */
export type LiveStreamStatus = "idle" | "connecting" | "open" | "reconnecting" | "error"

export type StreamStateEvent =
  | { readonly type: "start" }
  | { readonly type: "opened" }
  | { readonly type: "transport-error" }
  | { readonly type: "stop" }

export function initialStreamStatus(): LiveStreamStatus {
  return "idle"
}

/** Pure status reducer — no side effects. */
export function reduceStreamStatus(
  current: LiveStreamStatus,
  event: StreamStateEvent
): LiveStreamStatus {
  switch (event.type) {
    case "start":
      return "connecting"
    case "opened":
      return "open"
    case "transport-error":
      if (current === "open" || current === "reconnecting") {
        return "reconnecting"
      }
      if (current === "connecting") {
        return "error"
      }
      return current === "idle" ? "idle" : "error"
    case "stop":
      return "idle"
    default: {
      const _exhaustive: never = event
      return _exhaustive
    }
  }
}
