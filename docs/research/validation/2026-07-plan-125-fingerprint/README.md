# Plan 125 — Native trace fingerprint deviation

**Status:** DONE (2026-07-17)

## Decision

Canonical correlation = `error_events`. Drop legacy native
`opentelemetry_traces.fingerprint` when present.
See [native-trace-fingerprint.md](../../decisions/native-trace-fingerprint.md).

## Probes

- [../2026-07-17-plan-125-fingerprint-probe.md](../2026-07-17-plan-125-fingerprint-probe.md)
- Stable + nightly DROP safe, no product readers of native column

## Code

`drop_legacy_trace_fingerprint_column` in greptime lifecycle deviations.
