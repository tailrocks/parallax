# Plan 124: Collect CI runs and flaky-test evidence as product context

> **Executor instructions**: This is a Parallax product evidence adapter, not
> repository CI maintenance. Plans 094/101 own Parallax's own workflows. Start
> read-only with one approved provider, least privilege, bounded logs, and stable
> IDs; never rerun/cancel workflows or expose raw CI output to agents by default.

## Status

- **Priority**: P3
- **Effort**: L
- **Risk**: HIGH
- **Depends on**: 099, 104, 111, 121
- **Category**: future CI evidence / flaky tests / provider integration
- **Planned at**: `eefa4617`, 2026-07-12
- **Status**: BLOCKED
- **Blocker**: The operator has not opened product CI-provider collection or
  selected the first provider/repositories, permissions, retention, and claim scope.

## Why

The CI/flaky research specifies a prototype ending in `parallax gha collect`,
but no active plan owns that product surface. Build/test failures, retries, job
logs, and flaky histories can improve bundles only when collected with exact
workflow/job/test identities, bounded sensitive output, and explicit provider
coverage. This must not be confused with making Parallax's own CI green.

## Scope

In scope after the blocker clears:

- One selected provider's read-only workflow/run/job/check/artifact metadata
  ingestion, starting with GitHub Actions only if approved.
- Stable repository/commit/workflow/job/test/attempt identities and normalized
  pass/fail/cancel/skip/retry/flaky evidence.
- Bounded log/test-report excerpts, short-lived raw refs, typed redaction, and
  missing/truncated evidence markers.
- Webhook plus idempotent API backfill/reconciliation under rate limits.
- Correlation to release/deploy/change records from plan 121, runs/issues, and
  canonical bundles without treating CI status alone as root cause or fix success.
- A read-only collector/doctor CLI and dated claim-level coverage measurements.

Out of scope:

- Rerun, cancel, approve, dispatch, mutate secrets/variables, administer runners,
  merge PRs, or create generic provider-control tools.
- Storing complete unredacted logs indefinitely or calling any retry a flake.
- Vendor-neutral claims from one provider or one report format.
- Replacing nextest/unit/integration evidence owned by plan 101.

## Steps

1. Clear the trigger and record the provider, exact APIs/events/report formats,
   repositories, permissions, retention, output budgets, and allowed product claims.
2. Define normalized run/job/check/test-attempt records, stable IDs, ordering,
   duplicate/redelivery semantics, flaky classification, missing evidence, and
   raw-reference access policy.
3. Generate sanitized real fixtures for matrices, retries, cancellations,
   reruns, skipped jobs, partial logs, artifacts, JUnit/nextest, pagination,
   rate limits, deleted runs, and provider schema drift.
4. Implement signature-verified webhook intake plus bounded idempotent backfill.
   Persist mutable provider state in Turso; acknowledge only durable acceptance.
5. Implement the approved collector and `doctor` surface through typed APIs with
   progress/rate reporting and a final coverage summary. No hidden provider writes.
6. Correlate evidence into bundles with redacted excerpts/refs and explicit
   confidence. Measure flaky precision and workflow/job/test coverage before claims.

## Test Plan

- Auth/signature/permissions, webhook replay, pagination, rate-limit, and
  backfill-reconciliation integration tests.
- Matrix/retry/rerun/cancel/skip and test-attempt identity/property tests.
- Flaky classification fixtures proving retry success alone is insufficient.
- Seeded secrets/PII/prompt injection in logs, annotations, artifact names, and tests.
- Bounded output/raw-ref retention and canonical bundle projection/hash tests.
- Provider sandbox and full Turso/strict Rust/nextest/API gates.

## Done Criteria

- [ ] Operator-approved provider/repos/permissions/retention/claims are explicit.
- [ ] Collection is read-only, bounded, durable, idempotent, and rate-aware.
- [ ] Stable run/job/test-attempt identities survive retry, rerun, and redelivery.
- [ ] Flaky labels require the approved multi-attempt evidence and expose uncertainty.
- [ ] Raw/log evidence is redacted, short-lived, and absent from default agent output.
- [ ] Bundle correlations do not overclaim root cause or successful fixes.
- [ ] Dated real coverage/precision rows support every enabled product claim.

## STOP Conditions

- Product provider scope/permissions/retention is not explicitly opened.
- Read-only least privilege is impossible or provider output cannot be bounded/redacted.
- Stable attempt identity or truthful missing-evidence behavior cannot be established.
- Implementation would duplicate plan 094/101 or add provider-control operations.

## Remove When

Delete this plan and row when the approved provider collector and claim gates are
shipped with real evidence, or when product CI collection is rejected and no
actionable adapter remains.
