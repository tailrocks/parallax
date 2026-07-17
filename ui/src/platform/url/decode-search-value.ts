// Plan 153 — generic search-value decoder (feature owns schema/defaults).

import { boundaryError } from "@/platform/external-values/boundary-error"
import { reportBoundaryError } from "@/platform/external-values/boundary-diagnostic"
import type { BoundaryResult, RuntimeDecoder } from "@/platform/external-values/runtime-decoder"

const BOUNDARY_ID = "url.decode-search-value"

/**
 * Accept unknown search input and apply exactly one caller-owned decoder.
 * Routes must not cast or manually inspect unknown properties.
 */
export function decodeSearchValue<T>(
  input: unknown,
  decoder: RuntimeDecoder<T>
): BoundaryResult<T> {
  let decoded: ReturnType<RuntimeDecoder<T>["safeParse"]>
  try {
    decoded = decoder.safeParse(input)
  } catch {
    const error = boundaryError(BOUNDARY_ID, "schema-rejected", input)
    reportBoundaryError(error)
    return { ok: false, error }
  }
  if (!decoded.success) {
    const error = boundaryError(BOUNDARY_ID, "schema-rejected", input)
    reportBoundaryError(error)
    return { ok: false, error }
  }
  return { ok: true, value: decoded.data }
}
