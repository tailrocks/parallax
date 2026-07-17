// Plan 153 — payload-free boundary error contract.

export type BoundaryErrorCode =
  | "invalid-type"
  | "invalid-json"
  | "schema-rejected"
  | "unavailable"
  | "read-failed"
  | "write-failed"
  | "origin-rejected"
  | "cancelled"

export type ObservedKind =
  | "undefined"
  | "null"
  | "boolean"
  | "number"
  | "bigint"
  | "string"
  | "symbol"
  | "function"
  | "array"
  | "object"

export interface BoundaryError {
  readonly boundaryId: string
  readonly code: BoundaryErrorCode
  readonly observedKind: ObservedKind
  readonly meta?: Readonly<Record<string, number>>
}

export function observedKindOf(input: unknown): ObservedKind {
  if (input === undefined) return "undefined"
  if (input === null) return "null"
  if (Array.isArray(input)) return "array"
  const type = typeof input
  if (
    type === "boolean" ||
    type === "number" ||
    type === "bigint" ||
    type === "string" ||
    type === "symbol" ||
    type === "function" ||
    type === "object"
  ) {
    return type
  }
  return "object"
}

export function boundaryError(
  boundaryId: string,
  code: BoundaryErrorCode,
  input: unknown,
  meta?: Readonly<Record<string, number>>
): BoundaryError {
  return {
    boundaryId,
    code,
    observedKind: observedKindOf(input),
    ...(meta ? { meta } : {}),
  }
}
