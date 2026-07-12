# Plan 121: Collect deploy, release, code-change, and work-item context

> **Executor instructions**: Provider text is untrusted evidence, never policy or
> root-cause proof. Start read-only with one approved provider and stable IDs.
> Do not add writeback, broad repository tokens, raw issue text in bundles, or a
> provider claim that has not passed real webhook/backfill fixtures.

## Status

- **Priority**: P3
- **Effort**: XL
- **Risk**: HIGH
- **Depends on**: 099, 104, 109, 111, 116
- **Category**: future provider integration / causal evidence / security
- **Planned at**: `eefa4617`, 2026-07-12
- **Status**: BLOCKED
- **Blocker**: Provider ingestion is not in the open product scope; the operator
  has not selected the first provider, auth/deployment profile, or evidence claim.

## Why

The deploy/change research defines releases, deployments, commits, PRs, CI
checks, workflow jobs, and work items as useful context for "what changed?" but
not causality by themselves. It also contains a provider implementation order
while its evidence ledger remains `not_measured`. This plan is the sole future
implementation owner and keeps provider work gated until auth, redaction,
retention, and exact claim scope are approved.

## Scope

In scope after the blocker clears:

- One selected provider's read-only webhook/API ingestion with least-privilege
  credentials, signature verification, delivery IDs, replay protection, and
  deterministic backfill/reconciliation.
- Versioned release, deploy, commit, compare, PR/file, check/job, and work-item
  records with strong/medium/weak edge rules and explicit missing evidence.
- Stable joins to telemetry through commit SHA, release, environment, service,
  run, trace, and provider IDs before time-window inference.
- Turso ownership for mutable integration/config/state and bounded approved
  derived evidence; raw provider payloads remain short-lived scoped references.
- Redacted summaries and source-field/projection policy in canonical bundles.
- `doctor` coverage diagnostics and the claim-level measurement ledger.

Out of scope:

- Root-cause claims from adjacency alone, provider writeback, issue mutation,
  deployment control, repository administration, generic GitHub/Linear/Jira MCP,
  or direct production access.
- Adding all providers at once or accepting broad organization tokens.
- Storing raw comments, customer requests, release notes, or deploy logs in
  default agent-visible projections.

## Steps

1. Reproduce the trigger and record the first provider, exact API/webhook
   versions, supported entities, least-privilege permissions, and claim wording.
2. Specify typed normalized records, stable IDs, edge strengths, retry/replay,
   retention, redaction, missing-evidence, and conflict/backfill semantics.
3. Generate sanitized real fixtures for delivery, redelivery, missing/out-of-order
   state, pagination, compare bases, PR file truncation, protected deployments,
   renamed/deleted entities, rate limits, and schema drift.
4. Implement signature-verified ingestion and bounded API backfill through typed
   provider ports. Persist mutable state in Turso and acknowledge webhooks only
   after durable idempotent acceptance.
5. Build release/deploy/code/work-item edges and canonical bundle projections.
   Strong linkage still requires runtime evidence before causal ranking.
6. Add `parallax doctor deploy-context` and measure release/commit/deploy/status/
   compare/file-list/work-item coverage against predeclared thresholds.
7. Admit another provider only through a separate adapter/fixture slice. Keep
   writeback blocked until plan 123's outcome loop is independently proven.

## Test Plan

- Signature/auth/permission, replay, redelivery, pagination, rate-limit, and
  backfill reconciliation tests from sanitized provider fixtures.
- Stable-ID/edge-strength tests including contradictory and time-only evidence.
- Seeded secret/PII/prompt-injection provider text across storage and projections.
- Missing/truncated provider data and truthful doctor/claim-level tests.
- Turso migration/concurrency/retention and bundle hash/projection conformance.

## Done Criteria

- [ ] Operator-approved provider/scope/permissions and claim level are explicit.
- [ ] Webhooks and backfill are signature/auth checked, bounded, durable, and idempotent.
- [ ] Stable identifiers dominate; inferred adjacency is visibly weak and never root-cause proof.
- [ ] Raw provider content is scoped/short-lived and excluded from default agent output.
- [ ] Missing evidence and provider drift fail closed and appear in `doctor` output.
- [ ] Coverage thresholds have dated real rows before any product claim advances.
- [ ] Full Turso, bundle, redaction, API, and strict Rust gates pass.

## STOP Conditions

- Provider/product/auth scope is not explicitly opened.
- Required access exceeds least privilege or cannot separate read from write/admin.
- Provider delivery/backfill cannot be made idempotent and auditable.
- Product wording would imply causality from release/deploy adjacency alone.
- Implementation requires a fallback store or raw unredacted agent projection.

## Remove When

Delete this plan and row when every approved provider adapter and claim gate is
shipped and measured, or when provider context is explicitly rejected and no
actionable integration remains.
