/* @vitest-environment jsdom */
import { afterEach, describe, expect, it, vi } from "vitest"

import { loadAuthStatus, loginWithPassword } from "@/platform/auth/status"

afterEach(() => {
  vi.unstubAllGlobals()
})

describe("loadAuthStatus", () => {
  it("maps login_enabled off as the product default", async () => {
    vi.stubGlobal(
      "fetch",
      vi.fn().mockResolvedValue({
        ok: true,
        json: async () => ({ login_enabled: false, username: null }),
      }),
    )
    await expect(loadAuthStatus()).resolves.toEqual({
      loginEnabled: false,
      username: null,
    })
  })
})

describe("loginWithPassword", () => {
  it("returns the session token on 200", async () => {
    vi.stubGlobal(
      "fetch",
      vi.fn().mockResolvedValue({
        ok: true,
        status: 200,
        json: async () => ({ token: "tok", username: "op@example.com" }),
      }),
    )
    await expect(loginWithPassword("op@example.com", "secret")).resolves.toEqual({
      token: "tok",
      username: "op@example.com",
    })
  })
})
