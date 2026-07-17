# Plan 118 — Sentry envelope residual (closed 2026-07-17)

## Scope closed

| Residual | Landed surface |
| --- | --- |
| Python SDK 2.48 fixture | `crates/parallax-ingest/tests/fixtures/sentry/python-sdk-2.48-event.envelope` |
| Event-id ledger / collision | Turso `sentry_event_acks` |
| Cross-source OTLP+Sentry identity | worker oracle + shared fingerprint |
| Fail-closed redaction + bundle | ranking/redaction canaries |
| Live accept path | `m118_sentry_envelope` integration: HTTP → spool → worker → Turso issue + MemoryStore events; redelivery idempotent |

## Verification

```text
cargo test -p parallax-server --test m118_sentry_envelope
cargo test -p parallax-ingest --lib sentry_envelope
cargo test -p parallax-server --lib otlp_and_sentry_echo
```

Sentry writes issues + `error_events` only (no custom raw-signal table). Full managed-GreptimeDB dogfood is covered by the same worker path used for OTLP exceptions; the integration harness uses MemoryStore for telemetry rows and Turso for mutable issue/ack state.
