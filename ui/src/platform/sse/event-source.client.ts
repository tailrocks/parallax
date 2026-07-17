// Plan 153 — sole browser EventSource constructor owner.

export interface EventSourceLike {
  onopen: ((this: EventSourceLike, ev: Event) => unknown) | null
  onerror: ((this: EventSourceLike, ev: Event) => unknown) | null
  onmessage: ((this: EventSourceLike, ev: MessageEvent) => unknown) | null
  close(): void
}

export type EventSourceFactory = (url: string) => EventSourceLike

/** Production factory — constructs the browser EventSource. */
export const browserEventSourceFactory: EventSourceFactory = (url) => {
  // Cast: browser EventSource is structurally compatible for our handlers.
  return new EventSource(url) as unknown as EventSourceLike
}
