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
- **Status**: IN PROGRESS — ingest + event-id ledger landed; residual below
- **Decision**:
  [`docs/research/decisions/sentry-envelope-adapter.md`](../docs/research/decisions/sentry-envelope-adapter.md)

## Landed (do not replay)

- Pure envelope parser + unit fixtures (`parallax-ingest::sentry_envelope`).
- `ErrorSource::SentryEnvelope`, `derive_from_sentry_event`, `Signal::Sentry`
  spool of **normalized** accept record.
- `POST /api/<project_id>/envelope[/]` with public-key map, gzip, typed
  outcomes; disabled by default; worker writes issues + `error_events` only.
- Turso `sentry_event_acks` ledger: exact duplicate `(project, event_id,
  payload_hash)` returns `200` without re-spool; collision → `409`.
- Derived OTLP and Sentry echoes share stable issue/occurrence identity by
  `(trace, span, fingerprint)`; native OTLP evidence wins stored-row dedup.

## Residual only

1. ~~Real sanitized `sentry` Python SDK 2.48 envelope fixture~~ landed
   (`tests/fixtures/sentry/python-sdk-2.48-event.envelope` + unit accept).
2. ~~Idempotent event-id collision handling~~ landed.
3. ~~Cross-source OTLP+Sentry stable issue/occurrence identity~~ landed.
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
