// Runtime-metrics boundary helpers (Plan 149).

import { GraphqlBoundaryError } from "@/platform/graphql/error"

export function isRuntimeMetricsAbort(error: unknown): boolean {
  if (error instanceof GraphqlBoundaryError && error.code === "abort") {
    return true
  }
  return (
    typeof error === "object" &&
    error !== null &&
    "name" in error &&
    (error as { name?: unknown }).name === "AbortError"
  )
}
