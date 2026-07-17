// Plan 153 — unknown-first JSON text decode with exactly one supplied decoder.

import { boundaryError } from "./boundary-error"
import { reportBoundaryError } from "./boundary-diagnostic"
import type { BoundaryResult, RuntimeDecoder } from "./runtime-decoder"

const BOUNDARY_ID = "external-values.decode-json-text"

/**
 * Prove `input` is a string, parse JSON to unknown, apply one decoder.
 * Never throws for expected malformed external values.
 */
export function decodeJsonText<T>(input: unknown, decoder: RuntimeDecoder<T>): BoundaryResult<T> {
  if (typeof input !== "string") {
    const error = boundaryError(BOUNDARY_ID, "invalid-type", input)
    reportBoundaryError(error)
    return { ok: false, error }
  }
  let parsed: unknown
  try {
    parsed = JSON.parse(input) as unknown
  } catch {
    const error = boundaryError(BOUNDARY_ID, "invalid-json", input, {
      length: input.length,
    })
    reportBoundaryError(error)
    return { ok: false, error }
  }
  let decoded: ReturnType<RuntimeDecoder<T>["safeParse"]>
  try {
    decoded = decoder.safeParse(parsed)
  } catch {
    const error = boundaryError(BOUNDARY_ID, "schema-rejected", parsed)
    reportBoundaryError(error)
    return { ok: false, error }
  }
  if (!decoded.success) {
    const error = boundaryError(BOUNDARY_ID, "schema-rejected", parsed)
    reportBoundaryError(error)
    return { ok: false, error }
  }
  return { ok: true, value: decoded.data }
}
