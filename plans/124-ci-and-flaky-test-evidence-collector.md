# Plan 124: CI and flaky-test evidence residual

> **Executor instructions**: Product evidence adapter, not Parallax CI
> maintenance. Read-only GitHub Actions; least privilege; bounded logs; never
> rerun/cancel workflows; no raw CI default to agents.

## Status

- **Priority**: P3
- **Effort**: L
- **Risk**: HIGH
- **Depends on**: 099, 104, 111 (done); 121 residual
- **Category**: future CI evidence / flaky tests
- **Status**: IN PROGRESS — webhook + Turso + doctor inventory landed
- **Blocker**: none for residual REST backfill / bundle claim rows.
  Provider = GitHub Actions on tailrocks repos (unblock 2026-07-17).

## Residual only (after 121 durable path patterns)

1. Record exact APIs/events/report formats, repos, permissions, retention,
   output budgets, claim wording.
2. ~~Normalized job/attempt IDs + flaky multi-attempt gate~~ landed
   (`parallax-evidence::github_actions`), including idempotent redelivery,
   canonical ordering, and fail-closed conflicting outcomes per attempt ID.
   Turso attempt-identity + delivery ledgers are landed as the shared durable
   boundary for future webhook and REST backfill ingestion.
   Independently configured `workflow_job` HTTP ingest verifies GitHub HMAC
   before parsing and persists through that boundary; redelivery/collision
   behavior is covered end-to-end.
3. ~~Signature-verified workflow-job webhook + Turso wiring~~ landed
   (`POST /webhooks/github` + `ci_attempts` / `ci_attempt_deliveries`).
4. ~~`doctor` CI evidence inventory~~ landed (secret presence + delivery/attempt
   counts). Still open: bounded REST backfill cursor.
5. Bundle correlation without root-cause overclaim; dated coverage rows.

## Done Criteria

- [ ] Read-only, bounded, durable, idempotent, rate-aware collection.
- [ ] Stable attempt identities across retry/rerun/redelivery.
- [ ] Flaky labels require multi-attempt evidence; raw logs short-lived/redacted.
- [ ] Dated coverage/precision rows for every enabled claim.

## STOP / Remove When

STOP if least-privilege impossible or output cannot be bounded/redacted.
Delete when collector ships or product CI collection rejected.
