/* @vitest-environment jsdom */
import { afterEach, describe, expect, it } from "vitest"

import {
  apiAuthHeaders,
  getApiToken,
  setApiToken,
  withAccessTokenQuery,
} from "@/platform/auth/api-token"

const KEY = "parallax.api-token"

afterEach(() => {
  window.localStorage.removeItem(KEY)
})

describe("api-token storage", () => {
  it("returns null when nothing stored and trims stored values", () => {
    expect(getApiToken()).toBeNull()
    window.localStorage.setItem(KEY, "  tok  ")
    expect(getApiToken()).toBe("tok")
  })

  it("setApiToken stores, and empty string clears", () => {
    setApiToken(" secret-value ")
    expect(window.localStorage.getItem(KEY)).toBe("secret-value")
    setApiToken("   ")
    expect(window.localStorage.getItem(KEY)).toBeNull()
  })
})

describe("apiAuthHeaders", () => {
  it("emits bearer headers only when a token is stored", () => {
    expect(apiAuthHeaders()).toEqual({})
    setApiToken("t0k")
    expect(apiAuthHeaders()).toEqual({ authorization: "Bearer t0k" })
  })
})

describe("withAccessTokenQuery", () => {
  it("passes URLs through untouched without a token", () => {
    expect(withAccessTokenQuery("/v1/logs/stream?x=1")).toBe("/v1/logs/stream?x=1")
  })

  it("appends with the right separator and percent-encodes", () => {
    setApiToken("a&b/c")
    expect(withAccessTokenQuery("/v1/logs/stream")).toBe("/v1/logs/stream?access_token=a%26b%2Fc")
    expect(withAccessTokenQuery("/v1/traces/stream?svc=x")).toBe(
      "/v1/traces/stream?svc=x&access_token=a%26b%2Fc"
    )
  })
})
