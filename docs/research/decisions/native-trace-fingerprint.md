# Native trace fingerprint deviation

- **Status:** Approved
- **Decision date:** 2026-07-17
- **Depends on:** closed plan 104 Option C; live probes stable+nightly
- **Implementation status (2026-07-17):** Shipped, including legacy-column
  cleanup and product-query prohibition.

## Decision

1. **Canonical correlation** for fingerprint ↔ trace/span remains the derived
   **`error_events`** table (`fingerprint`, `trace_id`, `span_id`).
2. **Do not populate** a native `opentelemetry_traces.fingerprint` column.
3. **Fresh installs:** never ADD the column (landed `f21bc65`).
4. **Existing installs:** **DROP COLUMN `fingerprint`** when
   `information_schema` shows it present (live-proven safe on GreptimeDB
   stable 1.1.3 and nightly 1.2.0-20260713; see probe packet).
5. Product queries must not read the native column.

## Evidence

[2026-07-17-plan-125-fingerprint-probe.md](../validation/2026-07-17-plan-125-fingerprint-probe.md)

## Implementation

`GreptimeStore::drop_legacy_trace_fingerprint_column` runs from
`ensure_traces_deviations` after TTL reconcile, once per process.
