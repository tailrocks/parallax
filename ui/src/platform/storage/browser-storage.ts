// Plan 153 — sole handwritten owner of localStorage / sessionStorage.

import { boundaryError } from "@/platform/external-values/boundary-error"
import { reportBoundaryError } from "@/platform/external-values/boundary-diagnostic"
import type { BoundaryResult } from "@/platform/external-values/runtime-decoder"

const BOUNDARY_ID = "storage.browser"

export type BrowserStorageKind = "local" | "session"

export interface BrowserStorage {
  getItem(key: string): string | null
  setItem(key: string, value: string): void
  removeItem(key: string): void
}

function resolveStorage(
  kind: BrowserStorageKind,
  injected?: BrowserStorage | null
): BoundaryResult<BrowserStorage> {
  if (injected) return { ok: true, value: injected }
  try {
    if (typeof globalThis === "undefined") {
      const error = boundaryError(BOUNDARY_ID, "unavailable", undefined)
      reportBoundaryError(error)
      return { ok: false, error }
    }
    const storage =
      kind === "local"
        ? (globalThis as { localStorage?: BrowserStorage }).localStorage
        : (globalThis as { sessionStorage?: BrowserStorage }).sessionStorage
    if (!storage) {
      const error = boundaryError(BOUNDARY_ID, "unavailable", undefined)
      reportBoundaryError(error)
      return { ok: false, error }
    }
    return { ok: true, value: storage }
  } catch {
    const error = boundaryError(BOUNDARY_ID, "unavailable", undefined)
    reportBoundaryError(error)
    return { ok: false, error }
  }
}

export function readBrowserStorage(
  kind: BrowserStorageKind,
  key: string,
  injected?: BrowserStorage | null
): BoundaryResult<string | null> {
  const storage = resolveStorage(kind, injected)
  if (!storage.ok) return storage
  try {
    return { ok: true, value: storage.value.getItem(key) }
  } catch {
    const error = boundaryError(BOUNDARY_ID, "read-failed", key)
    reportBoundaryError(error)
    return { ok: false, error }
  }
}

export function writeBrowserStorage(
  kind: BrowserStorageKind,
  key: string,
  value: string,
  injected?: BrowserStorage | null
): BoundaryResult<true> {
  const storage = resolveStorage(kind, injected)
  if (!storage.ok) return storage
  try {
    storage.value.setItem(key, value)
    return { ok: true, value: true }
  } catch {
    const error = boundaryError(BOUNDARY_ID, "write-failed", key)
    reportBoundaryError(error)
    return { ok: false, error }
  }
}

export function removeBrowserStorage(
  kind: BrowserStorageKind,
  key: string,
  injected?: BrowserStorage | null
): BoundaryResult<true> {
  const storage = resolveStorage(kind, injected)
  if (!storage.ok) return storage
  try {
    storage.value.removeItem(key)
    return { ok: true, value: true }
  } catch {
    const error = boundaryError(BOUNDARY_ID, "write-failed", key)
    reportBoundaryError(error)
    return { ok: false, error }
  }
}
