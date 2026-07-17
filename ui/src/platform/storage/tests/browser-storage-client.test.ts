import { describe, expect, it } from "vitest"

import {
  readBrowserStorage,
  removeBrowserStorage,
  writeBrowserStorage,
  type BrowserStorage,
} from "@/platform/storage/browser-storage"

function memoryStorage(): BrowserStorage & { store: Map<string, string> } {
  const store = new Map<string, string>()
  return {
    store,
    getItem(key) {
      return store.has(key) ? store.get(key)! : null
    },
    setItem(key, value) {
      store.set(key, value)
    },
    removeItem(key) {
      store.delete(key)
    },
  }
}

describe("browser-storage", () => {
  it("reads and writes through injected storage", () => {
    const storage = memoryStorage()
    expect(writeBrowserStorage("local", "k", "v", storage)).toEqual({
      ok: true,
      value: true,
    })
    expect(readBrowserStorage("local", "k", storage)).toEqual({
      ok: true,
      value: "v",
    })
    expect(removeBrowserStorage("local", "k", storage)).toEqual({
      ok: true,
      value: true,
    })
    expect(readBrowserStorage("local", "k", storage)).toEqual({
      ok: true,
      value: null,
    })
  })

  it("maps security/quota exceptions to typed failures", () => {
    const storage: BrowserStorage = {
      getItem() {
        throw new Error("denied")
      },
      setItem() {
        throw new Error("quota")
      },
      removeItem() {
        throw new Error("denied")
      },
    }
    expect(readBrowserStorage("session", "k", storage).ok).toBe(false)
    expect(writeBrowserStorage("session", "k", "v", storage).ok).toBe(false)
  })
})
