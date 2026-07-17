// Plan 153 — browser document visibility source. SSR-safe: no document
// global is touched until methods run.

export interface PageVisibilitySource {
  readonly isVisible: () => boolean
  readonly subscribe: (listener: () => void) => () => void
}

/** Production visibility source (SSR-safe: visible when document absent). */
export const browserPageVisibility: PageVisibilitySource = {
  isVisible() {
    return typeof document === "undefined" || !document.hidden
  },
  subscribe(listener) {
    if (typeof document === "undefined") {
      return () => undefined
    }
    document.addEventListener("visibilitychange", listener)
    return () => document.removeEventListener("visibilitychange", listener)
  },
}
