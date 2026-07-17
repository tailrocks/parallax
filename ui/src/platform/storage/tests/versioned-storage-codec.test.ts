import { describe, expect, it } from "vitest"

import type { RuntimeDecoder } from "@/platform/external-values/runtime-decoder"
import type { BrowserStorage } from "@/platform/storage/browser-storage"
import {
  readVersionedStorage,
  writeVersionedStorage,
} from "@/platform/storage/versioned-storage-codec"

const listDecoder: RuntimeDecoder<string[]> = {
  safeParse(input) {
    return Array.isArray(input) &&
      input.every((item) => typeof item === "string")
      ? { success: true, data: input as string[] }
      : { success: false, error: "bad" }
  },
}

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

describe("versioned-storage-codec", () => {
  const codec = {
    kind: "local" as const,
    key: "sql.history",
    version: 1,
    decoder: listDecoder,
    encode: (value: string[]) => value,
  }

  it("round-trips versioned values", () => {
    const storage = memoryStorage()
    expect(writeVersionedStorage(codec, ["a", "b"], storage).ok).toBe(true)
    expect(readVersionedStorage(codec, storage)).toEqual({
      ok: true,
      value: ["a", "b"],
    })
  })

  it("returns null for missing keys without rewriting", () => {
    const storage = memoryStorage()
    expect(readVersionedStorage(codec, storage)).toEqual({
      ok: true,
      value: null,
    })
    expect(storage.store.size).toBe(0)
  })

  it("does not delete corrupt or unsupported-version data", () => {
    const storage = memoryStorage()
    storage.setItem(codec.key, "not-json")
    expect(readVersionedStorage(codec, storage).ok).toBe(false)
    expect(storage.getItem(codec.key)).toBe("not-json")

    storage.setItem(codec.key, JSON.stringify({ v: 99, data: [] }))
    expect(readVersionedStorage(codec, storage).ok).toBe(false)
    expect(storage.getItem(codec.key)).toContain('"v":99')
  })
})
