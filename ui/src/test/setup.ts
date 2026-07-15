import { cleanup } from "@testing-library/react"
import { afterEach, beforeEach } from "vitest"

import {
  assertDiagnostics,
  recordDiagnostic,
  resetDiagnostics,
} from "@/test/diagnostics"
import { networkEscapeReason } from "@/test/network"

afterEach(cleanup)

const originalConsoleError = console.error
const originalConsoleWarn = console.warn
const originalFetch = globalThis.fetch

beforeEach(() => {
  resetDiagnostics()
  console.error = (...values: unknown[]) => recordDiagnostic("error", values)
  console.warn = (...values: unknown[]) => recordDiagnostic("warn", values)
  globalThis.fetch = (input: RequestInfo | URL) =>
    Promise.reject(new Error(networkEscapeReason(input)))
})

afterEach(() => {
  console.error = originalConsoleError
  console.warn = originalConsoleWarn
  globalThis.fetch = originalFetch
  assertDiagnostics()
})

process.env["TZ"] = "UTC"

class TestResizeObserver implements ResizeObserver {
  readonly #callback: ResizeObserverCallback

  constructor(callback: ResizeObserverCallback) {
    this.#callback = callback
  }

  disconnect() {}
  observe(target: Element) {
    const contentRect = new DOMRectReadOnly(0, 0, 1024, 640)
    this.#callback(
      [
        {
          borderBoxSize: [],
          contentBoxSize: [],
          contentRect,
          devicePixelContentBoxSize: [],
          target,
        },
      ],
      this
    )
  }
  unobserve() {}
}

Object.defineProperty(globalThis, "ResizeObserver", {
  configurable: true,
  value: TestResizeObserver,
  writable: true,
})

if (typeof window !== "undefined") {
  Object.defineProperty(window, "scrollTo", {
    configurable: true,
    value: () => {},
    writable: true,
  })

  Object.defineProperty(window, "matchMedia", {
    configurable: true,
    value: (query: string): MediaQueryList => ({
      addEventListener: () => {},
      addListener: () => {},
      dispatchEvent: () => true,
      matches: false,
      media: query,
      onchange: null,
      removeEventListener: () => {},
      removeListener: () => {},
    }),
    writable: true,
  })

  Object.defineProperty(window.HTMLElement.prototype, "scrollIntoView", {
    configurable: true,
    value: () => {},
    writable: true,
  })

  Object.defineProperty(window.Element.prototype, "getAnimations", {
    configurable: true,
    value: () => [],
    writable: true,
  })
}
