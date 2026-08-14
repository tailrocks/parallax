# Plan 173: Alerts carry evidence — every incident notification delivers a bundle, not a question

> **Executor instructions**: Follow this plan step by step. Spec-first: the
> contract lands in the implementation spec before code. Run every
> verification command. On any "STOP conditions" item, stop and report.
>
> **Drift check (run first)**: `git diff --stat 7418bc9..HEAD -- crates/parallax-server/src/alerting/ crates/parallax-evidence/src/bundle/ crates/parallax-metadata/src/turso/alerts.rs crates/parallax-api/src/resolvers/alerts.rs docs/research/architecture/v1-implementation-spec.md ui/graphql/schema.graphql`
> — on mismatch with the excerpts below, STOP.
>
> **Ratchet gate**: `ratchet.toml` pins per-file structural metrics with
> EXACT-match enforcement; every touched Rust file's row must be updated to
> new actuals in the same commit; `cargo xtask policy --only structural`
> must pass. New UI tests need `ui/test-matrix.json` entries
> (`cargo xtask policy --only ui.tests`).

## Status

- **Priority**: P1
- **Effort**: M
- **Risk**: MED (touches the alerting fire path; mitigated: bundle assembly
  is read-only and failure must never block delivery)
- **Depends on**: none (playground c4 scenario — plan 164 — is the live
  verification once both exist)
- **Category**: direction
- **Planned at**: parallax `7418bc9`, 2026-08-14
- **Evidence base**: `docs/research/market/competitor-pain-points.md`
  (alert fatigue = #1 obstacle in two consecutive Grafana surveys; 67% of
  engineers admit dismissing alerts uninvestigated; Sentry alert-noise
  complaints span 2016→2026)

## Why this matters

An alert that carries only rule/value/state is a *question* ("go look") —
the user opens five tabs to answer it, and that loop is what makes noise
expensive and fatigue the industry's #1 incident-response obstacle.
Parallax's entire thesis is the bounded, redacted evidence bundle; that its
own alerting delivers question-shaped payloads while the bundle machinery
sits unused one crate away is an internal inconsistency, not a
nice-to-have. Correctness framing: the alert payload is *incomplete
evidence output* from a product whose contract is evidence. After this
plan, every incident notification carries (a reference to) a bundle
assembled at fire time — the anomalous measurement, correlated
traces/logs, deploy adjacency, and hypotheses — so a human or agent can
triage from the notification alone.

## Current state (verified)

- Delivery payloads: `crates/parallax-server/src/alerting/delivery.rs:72`
  `webhook_payload_json(ctx: &NotificationContext)` and `:114`
  `slack_webhook_payload_json` — rule/state/value fields only, no evidence
  reference (read `NotificationContext` at the top of the file for the
  exact field set before extending).
- Bundle machinery: `crates/parallax-evidence/src/bundle/` — anchors today
  are fingerprint | invocationId | traceId (exactly one); assembly already
  computes `deploy_adjacency: Vec<String>` (linkage-only statements,
  `bundle/assembly.rs:32`), hypotheses ranking, `missing_evidence`,
  redaction, canonical hash, token bounding.
- Incidents: Turso `alert_incidents` rows
  (`crates/parallax-metadata/src/turso/alerts.rs`), opened/resolved by the
  evaluator (`alerting/evaluator.rs`); GraphQL `alertIncident(s)` resolvers
  in `crates/parallax-api/src/resolvers/alerts.rs`.
- Contract home: `docs/research/architecture/v1-implementation-spec.md`
  (§ bundle contract: "exactly one anchor: fingerprint | invocationId |
  traceId"; §8 GraphQL surface). SDL `ui/graphql/schema.graphql` is
  generated and drift-gated (`cargo xtask ui graphql check`).
- Constraint: alerting evaluator/delivery live in parallax-server, which
  may depend on parallax-evidence (check `cargo xtask arch` tier graph —
  the API layer already assembles bundles, so the capability exists; if
  the server tier may NOT call evidence assembly directly, route through
  the same internal service the API resolvers use).

## Commands you will need

| Purpose | Command | Expected |
|---|---|---|
| Gates | `cargo xtask ci --fast && cargo xtask lint && cargo xtask test && cargo xtask arch && cargo xtask policy --only structural` | green |
| SDL | `cargo xtask ui graphql export && cargo xtask ui graphql check` | one new field family, no drift |
| One suite | `cargo nextest run -p parallax-server -E 'test(/alert/)'` | pass |
| Docs links | `cargo xtask docs links` | pass |

## Scope

**In scope**: v1-implementation-spec (bundle anchor addition + §8 fields),
`crates/parallax-evidence/src/bundle/` (incident anchor),
`crates/parallax-server/src/alerting/` (assemble-on-open, payload fields),
`crates/parallax-metadata/src/turso/alerts.rs` (+ migration for the bundle
reference column — numbered step if plan 169's versioning landed),
`crates/parallax-api/src/resolvers/alerts.rs` (expose bundle on incident),
`ui/graphql/schema.graphql` (generated), alerts UI incident view (bundle
panel), tests throughout.

**Out of scope**: alert rule model changes; delivery transport changes
(email stays deferred); alert *routing* logic; the playground c4 scenario
script (plan 164 owns it — extend it only if both plans are live).

## Git workflow

PR-only `main`; one branch, one PR; spec commit first; `git commit -s`;
Conventional Commits; agent trailer per `COMMITS.md`.

## Steps

### Step 1: Spec the incident anchor

Amend the bundle contract in `v1-implementation-spec.md`: a fourth anchor
`alertIncidentId` — bundle window = the rule's evaluation window around
the breach (± the rule's `windowMinutes`, capped by the existing anchor
window rules), scope = the rule's service scoping + group key; sections =
measured series snapshot (bounded, metric-summary semantics), correlated
traces/logs in-window for the scoped services, `deploy_adjacency`,
hypotheses, `missing_evidence` (absent deploy data or absent traces are
gaps, not omissions). §8: `alertIncident` gains `bundle` (nullable — old
incidents have none) and the notification payloads gain
`bundle_hash`, `bundle_url` (UI deep link), `top_hypothesis`,
`deploy_adjacency` (bounded list). Assembly failure NEVER blocks or delays
delivery: payload then carries `bundle_error` instead — deliver-first is
part of the contract.

**Verify**: spec diff reviewed against the existing anchor section's
wording; `cargo xtask docs links` pass.

### Step 2: Implement the anchor in parallax-evidence

Add the incident anchor variant to bundle assembly, reusing the existing
windowed-anchor machinery (trace/invocation anchors show the pattern).
Input = incident row fields (rule snapshot, group key, breach window).
Unit tests: bundle for a seeded incident contains the measured-series
section + gaps when traces absent; token bounding applies; hash stable.

**Verify**: `cargo nextest run -p parallax-evidence -E 'test(/incident/)'`
→ new tests pass.

### Step 3: Assemble on incident open, persist the reference

In the evaluator's incident-open path: after the incident row commits,
assemble the bundle (async, non-blocking — a failed/slow assembly must not
delay the state machine or delivery; enforce with a timeout) and persist
`bundle_hash` + assembly timestamp on the incident row (schema migration).
Renotify reuses the existing bundle unless the incident window moved.

**Verify**: evaluator tests: incident opens even when assembly fails
(inject a failing store); happy path persists the hash;
`cargo nextest run -p parallax-server -E 'test(/alert/)'` green.

### Step 4: Payloads + API + UI

- Extend `webhook_payload_json` / `slack_webhook_payload_json` with the
  Step-1 fields; existing payload tests
  (`delivery.rs:227,244`) extended — payload without a bundle (assembly
  failed) still validates with `bundle_error`.
- Resolver: `alertIncident.bundle` returns the stored bundle (same
  redaction/bounding path as other bundle reads). SDL export + drift check.
- UI: incident detail (alerts route) renders the bundle panel — reuse the
  issue-detail bundle presentation components; deep link target for
  `bundle_url`.

**Verify**: `cargo xtask ui graphql export && cargo xtask ui graphql check`;
UI unit test for the panel (matrix entry); Slack/webhook payload snapshot
tests updated.

### Step 5: Redaction egress proof

The bundle now leaves the host via webhooks — extend the redaction
egress assertion set (plan 164 c10 pattern if present; else a unit test in
delivery tests): seeded canary secrets never appear in the webhook payload
bytes.

**Verify**: `cargo nextest run -p parallax-server -E 'test(/payload|redact/)'` green.

## Test plan

Unit: anchor assembly, evaluator non-blocking behavior, payload shapes,
redaction canary. Resolver: MemoryStore incident-with-bundle. UI: panel
render test. Live: extend playground `c4-alerting.sh` (only if plan 164
landed) to assert the webhook sink received `bundle_hash` + the hash
matches `alertIncident.bundle`.

## Done criteria

- [x] Spec carries the fourth anchor + payload fields (committed before
      code).
- [x] Incident open assembles + persists a bundle without ever blocking
      delivery (failure-injection test proves it).
- [x] Webhook + Slack payloads carry bundle_hash/url/top_hypothesis/
      deploy_adjacency or `bundle_error`.
- [x] `alertIncident.bundle` in SDL; UI incident view shows the bundle.
- [x] Canary secrets provably absent from payload bytes.
- [x] Targeted gates green (lint, arch, structural policy, graphql
      check, ui.tests, incident/alert/bundle nextest). Full `ci --fast`
      + `cargo xtask test` wait for the 162–176 close-out dual gate.
- [x] `plans/README.md` row updated.

## STOP conditions

1. Drift check fails.
2. The tier graph forbids server→evidence assembly and no existing internal
   service provides it — report the architectural seam options, don't punch
   the layering.
3. Bundle assembly at fire time needs a query the storage traits don't
   expose — report the missing capability.
4. Any design choice would make delivery wait on assembly — that inverts
   the contract; deliver-first is non-negotiable.

## Maintenance notes

- Rule-model changes (plan 171's preview, future signal types) must keep
  the incident anchor's window/scope derivation in sync — one derivation
  function, used by preview AND incident assembly, prevents drift.
- The payload is now an egress surface: any new bundle section must pass
  the redaction canary test before shipping in webhooks.
- Root-cause note: this plan removes the enabling condition of
  question-shaped alerts (no evidence primitive at fire time). The
  remaining symptom-layer item — digest/quiet-hours notification policy —
  is deliberately not here; it is routing, not evidence, and belongs to a
  future alerting-routing decision.
