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
- **Status**: BLOCKED
- **Blocker**: Plan 121 residual (GitHub deploy/change durable path) incomplete.
  Provider = GitHub Actions on tailrocks repos (unblock 2026-07-17) but
  implementation must not invent permissions/retention before fixtures.

## Residual only (after 121 durable path patterns)

1. Record exact APIs/events/report formats, repos, permissions, retention,
   output budgets, claim wording.
2. Normalized run/job/check/test-attempt IDs; flaky requires multi-attempt
   evidence (not any retry).
3. Signature-verified webhook + idempotent backfill; Turso state; doctor CLI.
4. Bundle correlation without root-cause overclaim; dated coverage rows.

## Done Criteria

- [ ] Read-only, bounded, durable, idempotent, rate-aware collection.
- [ ] Stable attempt identities across retry/rerun/redelivery.
- [ ] Flaky labels require multi-attempt evidence; raw logs short-lived/redacted.
- [ ] Dated coverage/precision rows for every enabled claim.

## STOP / Remove When

STOP if least-privilege impossible or output cannot be bounded/redacted.
Delete when collector ships or product CI collection rejected.
