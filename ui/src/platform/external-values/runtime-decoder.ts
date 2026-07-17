// Plan 153 — structural runtime-decoder protocol (unknown-first).

import type { BoundaryError } from "./boundary-error"

export interface RuntimeDecoder<T> {
  safeParse(
    input: unknown
  ):
    | { readonly success: true; readonly data: T }
    | { readonly success: false; readonly error: unknown }
}

export type BoundaryResult<T> =
  | { readonly ok: true; readonly value: T }
  | { readonly ok: false; readonly error: BoundaryError }
