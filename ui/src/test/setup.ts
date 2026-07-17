import { cleanup } from "@testing-library/react"
import { afterEach, beforeEach } from "vitest"

import { assertDiagnostics, recordDiagnostic, resetDiagnostics } from "@/test/diagnostics"
import { networkEscapeReason } from "@/test/network"
import { resetRegisteredTestState } from "@/test/resets"

const originalConsoleError = console.error
const originalConsoleWarn = console.warn
const originalFetch = globalThis.fetch

function recordPageError(event: ErrorEvent) {
  event.preventDefault()
  recordDiagnostic("error", ["page error", event.error ?? event.message])
}

function recordUnhandledRejection(event: PromiseRejectionEvent) {
  event.preventDefault()
  recordDiagnostic("error", ["browser unhandled rejection", event.reason])
}

beforeEach(async () => {
  await resetRegisteredTestState()
  resetDiagnostics()
  console.error = (...values: unknown[]) => recordDiagnostic("error", values)
  console.warn = (...values: unknown[]) => recordDiagnostic("warn", values)
  globalThis.fetch = (input: RequestInfo | URL) =>
    Promise.reject(new Error(networkEscapeReason(input)))
  if (typeof window !== "undefined") {
    window.addEventListener("error", recordPageError)
    window.addEventListener("unhandledrejection", recordUnhandledRejection)
  }
})

afterEach(() => {
  cleanup()
  if (typeof window !== "undefined") {
    window.removeEventListener("error", recordPageError)
    window.removeEventListener("unhandledrejection", recordUnhandledRejection)
  }
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
