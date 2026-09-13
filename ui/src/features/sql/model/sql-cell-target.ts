import { CLI_INVOCATION_ID } from "@/shared/semconv"
import { normalizeColumn } from "@/features/sql/model/sql-row"

export type SqlCellTarget =
  | { to: "/traces/$traceId"; params: { traceId: string } }
  | { to: "/invocations/$invocationId"; params: { invocationId: string } }
  | { to: "/issues/$service/$fingerprint"; params: { service: string; fingerprint: string } }
  | { to: "/services/$service"; params: { service: string } }

function cellValue(row: Record<string, string>, keys: readonly string[]): string | null {
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

  const directTargets: Record<string, SqlCellTarget> = {
    trace_id: { to: "/traces/$traceId", params: { traceId: value } },
    run_id: { to: "/invocations/$invocationId", params: { invocationId: value } },
    invocation_id: { to: "/invocations/$invocationId", params: { invocationId: value } },
    [CLI_INVOCATION_ID]: { to: "/invocations/$invocationId", params: { invocationId: value } },
    service: { to: "/services/$service", params: { service: value } },
    service_name: { to: "/services/$service", params: { service: value } },
  }
  if (directTargets[normalized]) return directTargets[normalized]

  if (normalized === "span_id") {
    const traceId = cellValue(row, ["trace_id"])
    return traceId ? { to: "/traces/$traceId", params: { traceId } } : null
  }
  if (normalized !== "fingerprint") return null
  const service = cellValue(row, ["service", "service_name"])
  return service
    ? { to: "/issues/$service/$fingerprint", params: { service, fingerprint: value } }
    : null
}
