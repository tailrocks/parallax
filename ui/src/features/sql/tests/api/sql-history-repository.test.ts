import { describe, expect, it } from "vitest"

import {
  loadSqlHistory,
  recordSqlHistory,
} from "@/features/sql/api/sql-history-repository"
import { SQL_HISTORY_KEY } from "@/features/sql/model/sql-history"
import type { BrowserStorage } from "@/platform/storage/browser-storage"

function memoryStorage(initial: Record<string, string> = {}): BrowserStorage {
  const map = new Map(Object.entries(initial))
  return {
    getItem: (key) => map.get(key) ?? null,
    setItem: (key, value) => {
      map.set(key, value)
    },
    removeItem: (key) => {
      map.delete(key)
    },
  }
}

describe("sql history repository", () => {
  it("loads and records history through platform storage", () => {
    const storage = memoryStorage({
      [SQL_HISTORY_KEY]: JSON.stringify(["old"]),
    })
    expect(loadSqlHistory(storage)).toEqual(["old"])
    const { entries, writeOk } = recordSqlHistory(["old"], "new", storage)
    expect(writeOk).toBe(true)
    expect(entries).toEqual(["new", "old"])
    expect(loadSqlHistory(storage)).toEqual(["new", "old"])
  })

  it("returns empty on unavailable storage and flags write failure", () => {
    const throwing: BrowserStorage = {
      getItem: () => {
        throw new Error("denied")
      },
      setItem: () => {
        throw new Error("denied")
      },
      removeItem: () => {
        throw new Error("denied")
      },
    }
    expect(loadSqlHistory(throwing)).toEqual([])
    const { entries, writeOk } = recordSqlHistory([], "q", throwing)
    expect(writeOk).toBe(false)
    expect(entries).toEqual(["q"])
  })
})
