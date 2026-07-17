import {
  parseHistoryWire,
  pushHistoryEntry,
  serializeHistoryWire,
  SQL_HISTORY_KEY,
} from "@/features/sql/model/sql-history"
import { SqlError } from "@/features/sql/model/sql-error"
import {
  readBrowserStorage,
  writeBrowserStorage,
  type BrowserStorage,
} from "@/platform/storage/browser-storage"

export function loadSqlHistory(storage?: BrowserStorage | null): string[] {
  const result = readBrowserStorage("local", SQL_HISTORY_KEY, storage)
  if (!result.ok) return []
  return parseHistoryWire(result.value)
}

/**
 * Record a successful query. Returns the next history list.
 * Write failure projects to SqlError only when throwOnWriteFailure is true;
 * baseline swallows storage exceptions by not catching setItem in try — wait:
 * baseline localStorage.setItem can throw (quota) and would abort setHistory.
 * Platform write returns ok:false; we preserve success UI by still returning
 * next list (query succeeded). Callers may inspect writeOk if needed.
 */
export function recordSqlHistory(
  current: readonly string[],
  sql: string,
  storage?: BrowserStorage | null
): { entries: string[]; writeOk: boolean } {
  const entries = pushHistoryEntry(current, sql)
  const wire = serializeHistoryWire(entries)
  const written = writeBrowserStorage("local", SQL_HISTORY_KEY, wire, storage)
  return { entries, writeOk: written.ok }
}

export function requireSqlHistoryWrite(
  current: readonly string[],
  sql: string,
  storage?: BrowserStorage | null
): string[] {
  const { entries, writeOk } = recordSqlHistory(current, sql, storage)
  if (!writeOk) {
    throw new SqlError("history-persistence", "failed to persist SQL history")
  }
  return entries
}
