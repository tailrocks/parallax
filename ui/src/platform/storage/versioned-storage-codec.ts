// Plan 153 — versioned storage codec (feature owns key/version/wire/policy).

import { decodeJsonText } from "@/platform/external-values/decode-json-text"
import { boundaryError } from "@/platform/external-values/boundary-error"
import { reportBoundaryError } from "@/platform/external-values/boundary-diagnostic"
import type { BoundaryResult, RuntimeDecoder } from "@/platform/external-values/runtime-decoder"
import {
  readBrowserStorage,
  writeBrowserStorage,
  type BrowserStorage,
  type BrowserStorageKind,
} from "@/platform/storage/browser-storage"

const BOUNDARY_ID = "storage.versioned-codec"

export interface VersionedEnvelope<T> {
  readonly v: number
  readonly data: T
}

export interface VersionedStorageCodec<T> {
  readonly kind: BrowserStorageKind
  readonly key: string
  readonly version: number
  readonly decoder: RuntimeDecoder<T>
  readonly encode: (value: T) => unknown
}

export function readVersionedStorage<T>(
  codec: VersionedStorageCodec<T>,
  injected?: BrowserStorage | null
): BoundaryResult<T | null> {
  const raw = readBrowserStorage(codec.kind, codec.key, injected)
  if (!raw.ok) return raw
  if (raw.value === null) return { ok: true, value: null }

  const envelopeDecoder: RuntimeDecoder<VersionedEnvelope<unknown>> = {
    safeParse(input) {
      if (input === null || typeof input !== "object" || Array.isArray(input)) {
        return { success: false, error: "envelope" }
      }
      const record = input as { v?: unknown; data?: unknown }
      if (typeof record.v !== "number" || !("data" in record)) {
        return { success: false, error: "envelope" }
      }
      return {
        success: true,
        data: { v: record.v, data: record.data },
      }
    },
  }

  const envelope = decodeJsonText(raw.value, envelopeDecoder)
  if (!envelope.ok) return envelope
  if (envelope.value.v !== codec.version) {
    // Unsupported version: do not delete or rewrite; surface schema-rejected.
    const error = boundaryError(BOUNDARY_ID, "schema-rejected", envelope.value.v, {
      version: envelope.value.v,
      expected: codec.version,
    })
    reportBoundaryError(error)
    return { ok: false, error }
  }
  let decoded: ReturnType<RuntimeDecoder<T>["safeParse"]>
  try {
    decoded = codec.decoder.safeParse(envelope.value.data)
  } catch {
    const error = boundaryError(BOUNDARY_ID, "schema-rejected", envelope.value.data)
    reportBoundaryError(error)
    return { ok: false, error }
  }
  if (!decoded.success) {
    const error = boundaryError(BOUNDARY_ID, "schema-rejected", envelope.value.data)
    reportBoundaryError(error)
    return { ok: false, error }
  }
  return { ok: true, value: decoded.data }
}

export function writeVersionedStorage<T>(
  codec: VersionedStorageCodec<T>,
  value: T,
  injected?: BrowserStorage | null
): BoundaryResult<true> {
  let payload: string
  try {
    payload = JSON.stringify({
      v: codec.version,
      data: codec.encode(value),
    })
  } catch {
    const error = boundaryError(BOUNDARY_ID, "write-failed", value)
    reportBoundaryError(error)
    return { ok: false, error }
  }
  return writeBrowserStorage(codec.kind, codec.key, payload, injected)
}
