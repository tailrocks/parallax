import { CLI_INVOCATION_ID } from "@/shared/semconv"
import { normalizeColumn } from "@/features/sql/model/sql-row"

export type SqlCellTarget =
  | { to: "/traces/$traceId"; params: { traceId: string } }
  | { to: "/invocations/$invocationId"; params: { invocationId: string } }
  | { to: "/issues/$fingerprint"; params: { fingerprint: string } }
  | { to: "/services/$service"; params: { service: string } }

function cellValue(
  row: Record<string, string>,
  keys: readonly string[]
): string | null {
  for (const key of keys) {
    const value = row[key]
    if (value && value !== "null") return value
  }
  return null
}

export function targetForCell(
  column: string,
  value: string,
  row: Record<string, string>
): SqlCellTarget | null {
  if (!value || value === "null") return null
  const normalized = normalizeColumn(column)
  if (normalized === "trace_id") {
    return { to: "/traces/$traceId", params: { traceId: value } }
  }
  if (normalized === "span_id") {
    const traceId = cellValue(row, ["trace_id"])
    return traceId ? { to: "/traces/$traceId", params: { traceId } } : null
  }
  if (
    normalized === "run_id" ||
    normalized === "invocation_id" ||
    normalized === CLI_INVOCATION_ID
  ) {
    // Plan 157 owns the /invocations route rename; keep link target until then.
    return { to: "/invocations/$invocationId", params: { invocationId: value } }
  }
  if (normalized === "fingerprint") {
    return { to: "/issues/$fingerprint", params: { fingerprint: value } }
  }
  if (normalized === "service" || normalized === "service_name") {
    return { to: "/services/$service", params: { service: value } }
  }
  return null
}
