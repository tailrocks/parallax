# Plan 118: Sentry envelope adapter residual gates

> **Executor instructions**: Compatibility adapter only. OTLP remains primary;
> GreptimeDB + Turso only; no custom raw-signal table; native TLS; canonical
> redacted bundle.

## Status

- **Priority**: P3
- **Effort**: L remaining
- **Risk**: HIGH
- **Depends on**: 093, 099, 104, 111, 116 (done)
- **Category**: future compatibility / ingest / security
- **Status**: IN PROGRESS — ingest-path slice landed; residual below
- **Decision**:
  [`docs/research/decisions/sentry-envelope-adapter.md`](../docs/research/decisions/sentry-envelope-adapter.md)

## Landed (do not replay)

- Pure envelope parser + unit fixtures (`parallax-ingest::sentry_envelope`).
- `ErrorSource::SentryEnvelope`, `derive_from_sentry_event`, `Signal::Sentry`
  spool of **normalized** accept record.
- `POST /api/<project_id>/envelope[/]` with public-key map, gzip, typed
  outcomes; disabled by default; worker writes issues + `error_events` only.

## Residual only

1. Real sanitized `sentry` 0.48.x SDK envelope fixtures (compatibility claim).
2. Idempotent event-id collision handling (`409` / exact-duplicate ledger).
3. Cross-source OTLP+Sentry stable issue/occurrence identity.
4. Full fail-closed redaction + canonical bundle projection for Sentry path.
5. Real Greptime+Turso, retention/doctor, API gates end-to-end for Sentry.
6. Transactions/attachments/other languages stay unsupported until measured.

## Done Criteria

- [ ] Fixture-backed SDK/protocol claim.
- [ ] Duplicate/cross-source identity without replaying completed effects.
- [ ] Agent-visible projections canonical, bounded, redacted.
- [ ] Live Greptime+Turso + spool/retention/strict Rust gates pass.

## STOP / Remove When

STOP if OTLP-first/native-table/redaction policy would weaken or durable ack
is impossible. Delete when adapter ships with evidence, or operator rejects.
