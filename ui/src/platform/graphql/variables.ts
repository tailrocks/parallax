// Plan 152 — canonical GraphQL variable encoding for cache keys and requests.

import { graphqlError } from "@/platform/graphql/error"

/**
 * Encode GraphQL variables deterministically for cache/in-flight keys.
 * - Object keys sorted
 * - Array order preserved
 * - undefined / non-finite / cyclic / non-JSON values fail before fetch
 */
export function encodeGraphqlVariables(variables: unknown): string {
  const seen = new WeakSet<object>()
  const encoded = encodeValue(variables, seen)
  return JSON.stringify(encoded)
}

function encodeValue(value: unknown, seen: WeakSet<object>): unknown {
  if (value === undefined) {
    throw graphqlError("invalid-variables", {
      message: "graphql variables must not contain undefined",
    })
  }
  if (value === null) return null
  if (typeof value === "string" || typeof value === "boolean") return value
  if (typeof value === "number") {
    if (!Number.isFinite(value)) {
      throw graphqlError("invalid-variables", {
        message: "graphql variables must not contain non-finite numbers",
      })
    }
    return value
  }
  if (typeof value === "bigint") {
    throw graphqlError("invalid-variables", {
      message: "graphql variables must not contain bigint",
    })
  }
  if (typeof value === "function" || typeof value === "symbol") {
    throw graphqlError("invalid-variables", {
      message: "graphql variables must be JSON-compatible",
    })
  }
  if (Array.isArray(value)) {
    return value.map((item) => encodeValue(item, seen))
  }
  if (typeof value === "object") {
    if (seen.has(value)) {
      throw graphqlError("invalid-variables", {
        message: "graphql variables must not be cyclic",
      })
    }
    seen.add(value)
    const record = value as Record<string, unknown>
    const keys = Object.keys(record).sort()
    const out: Record<string, unknown> = {}
    for (const key of keys) {
      const entry = record[key]
      if (entry === undefined) {
        // Omit undefined object properties (GraphQL variable omission).
        continue
      }
      out[key] = encodeValue(entry, seen)
    }
    seen.delete(value)
    return out
  }
  throw graphqlError("invalid-variables", {
    message: "graphql variables must be JSON-compatible",
  })
}
