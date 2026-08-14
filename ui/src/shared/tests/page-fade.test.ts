/* @vitest-environment jsdom */

import { afterEach, describe, expect, it } from "vitest"

import { resetPageFadeForTests, shouldPlayPageFade } from "@/shared/page-fade"

function stubMatchMedia(reduce: boolean) {
  Object.defineProperty(window, "matchMedia", {
    configurable: true,
    writable: true,
    value: (query: string): MediaQueryList => ({
      addEventListener: () => {},
      addListener: () => {},
      dispatchEvent: () => true,
      matches: reduce && query.includes("prefers-reduced-motion"),
      media: query,
      onchange: null,
      removeEventListener: () => {},
      removeListener: () => {},
    }),
  })
}

afterEach(() => {
  resetPageFadeForTests()
  stubMatchMedia(false)
})

describe("page fade gate", () => {
  it("fires once per boot", () => {
    stubMatchMedia(false)
    expect(shouldPlayPageFade()).toBe(true)
  })

  it("second call is a no-op", () => {
    stubMatchMedia(false)
    expect(shouldPlayPageFade()).toBe(true)
    expect(shouldPlayPageFade()).toBe(false)
  })

  it("reduced-motion short-circuits", () => {
    stubMatchMedia(true)
    expect(shouldPlayPageFade()).toBe(false)
    expect(shouldPlayPageFade()).toBe(false)
  })
})
