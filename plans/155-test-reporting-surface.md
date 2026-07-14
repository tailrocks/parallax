# Plan 155: Test reporting and test observability surface

> **Executor instructions**: This plan adds a new Parallax product capability:
> tests as a first-class, filterable surface fused with traces, logs, issues,
> and runs. It was collected as an information-only packet on 2026-07-14
> (operator-directed); no code changed at planning time. Evidence base:
> [`docs/research/market/test-reporting-ecosystem.md`](../docs/research/market/test-reporting-ecosystem.md).
> The playground emits the matching payload under plan 154 W4. Plan 124 keeps
> CI-provider (GitHub Actions API) collection; this plan consumes only
> telemetry that arrives through OTLP ingest. Do not create branches or PRs.

## Status

- **Priority**: P1
- **Effort**: XL
- **Risk**: MEDIUM
- **Depends on**: 149, 152, 153 (UI foundations; cannot land during plan
  140's move-only migration — extend Runs/Tests surfaces only after 140
  closes); soft 104 (test-mode bundle anchor), 119 (semconv constants), 121
  (deploy/change joins), 124 (CI-provider enrichment), 140 (Runs feature)
- **Category**: product capability / ingest derivation / UI surface
- **Planned at**: `8f24808`, 2026-07-14
- **Status**: TODO

## Why

Operator direction (2026-07-14): Parallax is not only a telemetry viewer — it
must also work as a **test reporting system**. Every test visible; every
failed test opens the error, the attempt chain, and the distributed trace of
the system under test; tests get their own page with filters, history,
flakiness, and version-under-test attribution.

Research (see evidence doc) shows the field splits into report generators
with no store or trace linkage (Allure), closed test-as-telemetry platforms
(Datadog Test Optimization), flaky-detection SaaS bolted onto JUnit XML
(Trunk, Codecov, Buildkite), and an abandoned trace-based-testing niche
(Tracetest). Nobody ships: SUT telemetry inside the test window, or one
fingerprint space shared between test failures and production issues. Sentry
owned both halves and sold the test half (Codecov → Harness, 2026-06) without
fusing them. Parallax's existing engines (native OTLP ingest, fingerprinting,
issues, runs, evidence bundles) cover most of the hard parts; what is missing
is the test domain model and the surface.

## Design decisions (bound by this plan)

### D1 — Identity model (three separated layers + explicit override)

The operator explicitly flagged: do not assume `parallax.run.id` is the test
identity. It is not. A run scopes ONE session invocation; test history must
survive across runs, branches, and CI jobs (ReportPortal's deprecated
launch-name-in-identity is the canonical mistake). Model:

1. **`test_case_key`** — stable identity of a test case. Fallback chain,
   highest wins: explicit `parallax.test.id` attribute (ALLURE_ID pattern,
   survives renames) → code reference (fully qualified name; never
   line/column — Allure's Playwright adapter breaks history on line shifts)
   → name path (suite chain + test name). Hash stored, components stored
   queryable.
2. **`test_variant_key`** — `test_case_key` + ordered non-excluded parameter
   values (Allure `historyId` analog). History, retries, and flaky state key
   on the variant.
3. **Configuration axis, NOT in any hash** — environment/os/browser/service
   attributes (`test.configuration.*`-shaped) stored as queryable dimensions
   so "flaky only on macOS" is a filter, not a fork (Datadog/Trunk/Buildkite
   lesson).
4. Results are keyed `(test_variant_key, parallax.run.id, attempt)`; the run
   id links a result to its session, never to identity.

### D2 — Signal mapping (native tables only)

Test telemetry arrives as ordinary OTLP: test = root span (steps = child
spans, assertion failure = span status ERROR + exception event), session =
`parallax.run.id`, semconv `test.*` + `cicd.pipeline.*` + `vcs.*` +
`service.version`. Test spans/logs live in the native GreptimeDB
`opentelemetry_traces`/`opentelemetry_logs` tables — **no hand-rolled
raw-signal table** (AGENTS.md native-table rule). Mutable state
(case/variant registry, flaky state, mute/known flags) lives in **Turso**
(same rationale as issue identity). Any derived acceleration table follows
the documented extension process in
`docs/research/decisions/native-otel-tables.md` first.

### D3 — Status taxonomy

`passed / failed / broken / skipped / unknown`, where failed = assertion
(product defect) and broken = any other harness/infra error — derived from
`error.type`/exception class family at analysis time. This split is Allure's
most-praised triage feature and no OTel attribute carries it today; derive
it, store the rule, and surface it (failed → SUT owner; broken → harness
owner).

### D4 — Attempt chains, never latest-wins

Every attempt is kept and displayed as a chain (CTRF `retryAttempts` shape).
Aggregations state which policy they use; the default rollup marks a
pass-after-fail result as **flaky-pass**, not pass (Allure's latest-wins
masks flakiness — do not copy it).

### D5 — Failure clustering shares the production fingerprint space

Test failures run through the existing
`fingerprint_with_operation(error_type, message, stacktrace, operation)`
pipeline. Two independent axes: failure fingerprint (message/stack
normalized; NO test identity inside → one root cause clusters across many
tests, and a test failure can equal a production issue) × test variant
(history). The Tests surface shows both: "this failure = issue <fingerprint>"
and "this test's history".

### D6 — Flaky state machine (per variant, in Turso)

Detection signals: same-commit divergence (pass+fail on one
`vcs.ref.head.revision`), intra-run attempt mix, windowed transition count.
States: `healthy → flaky → fixed(expiry)` plus `broken` (consistently
failing) — broken is never classified flaky (Trunk lesson). Expiry: N
consecutive passes (default 30) or operator action. Mute/known flags follow
plan 124's guardrail: flaky ≠ "any retry"; multi-attempt evidence required.
Quarantine *enforcement* (runner fetches state / exit-code rewriting) is a
recorded trigger, not V1.

### D7 — Separate Tests surface

Operator direction: tests get their own page, not a Runs tab. `/tests` +
`/tests/$caseKey` routes (post-migration: `features/tests` behind plan-149
facades, plan-152 generated GraphQL documents), nav entry in `workspaceNav`
next to Runs. Tests link to Runs (session), Traces (stitched SUT trace),
Logs (test window), Issues (shared fingerprint) via declared feature edges.

## Scope

In scope:

- Turso entities + migrations: `test_cases`, `test_variants`, `test_results`
  (result index rows referencing native span ids), flaky state fields.
- Ingest-side derivation: recognize test root spans (semconv `test.*` or
  `parallax.test.*`), populate registry, attempt chains, status taxonomy,
  fingerprint linkage.
- GraphQL `tests` namespace (resolver module pattern of `runs.rs`/`issues.rs`):
  `test_runs` (sessions), `test_cases` (explorer with filters:
  suite, service, status, flaky state, owner, environment, release, time
  range), `test_case(caseKey)` (history, variants, attempts), links to
  traces/logs/issues/runs.
- UI: Tests list page (filter toolbar + virtualized table — `issues.index`
  is the template) and test detail (history trend per variant, attempt
  chain, failure message/stack, linked trace waterfall, logs-in-window,
  related issue, release/version attribution, flaky badge with evidence).
- Overview/Runs cross-links: a run that is a test session shows its test
  rollup; a failed test deep-links its evidence.
- Semconv constants for `test.*`, `cicd.pipeline.*`, `parallax.test.id`
  (hand-written until plan 119 codegen, then generated).
- Evidence: verified end-to-end against plan 154 W4 playground payload
  (nextest, JUnit 5, Playwright).

Out of scope (recorded triggers):

- JUnit XML / CTRF file ingestion (reopen when a real consumer cannot emit
  OTLP; normalize JUnit → CTRF-shaped internal model when opened).
- Runner-enforced quarantine protocol, PR/MR feedback comments, test impact
  analysis/selection, manual test cases / TMS features (authoring, plans,
  milestones — never rebuild), declarative category-rule files (evaluate
  after fingerprint override rules exist), AI failure triage.
- Any GreptimeDB custom raw-signal table (STOP + escalation process instead).
- Evidence-bundle test-anchor kind — additive `bundle-v1` change goes through
  plan 104's contract decision, not here.

## Steps

1. Write the domain contract first: identity derivation (D1), status
   taxonomy mapping (D3), attempt semantics (D4), flaky signals/states (D6)
   as a spec section in this plan's implementation commit; add semconv
   constants.
2. Turso migrations + `parallax-model` types (`TestCaseRecord`,
   `TestVariantRecord`, `TestResultRecord`) + `parallax-metadata` modules
   following `turso/runs.rs` pattern.
3. Analysis/derivation: test-span recognition, registry upsert, attempt
   chaining, failed/broken derivation, fingerprint linkage
   (`parallax-analysis`), unit + property tests over normalization.
4. GraphQL namespace + resolvers + clamped queries; wire into `lib.rs` Query
   root; fixtures.
5. UI Tests feature (after plans 149/152/153 land and 140 closes): list +
   detail routes per D7, generated GraphQL documents, Playwright contract
   rows (plans 144-146 gates).
6. Flaky state machine job over ingested results (same-commit divergence
   needs `vcs.ref.head.revision` present — playground provides it; document
   degraded mode when absent).
7. Verify end-to-end with plan 154 W4 payload: one failed Playwright test and
   one failed Rust integration test produce list/detail/history/trace/issue
   linkage; record evidence in `docs/research/validation/`.

## Test Plan

- Unit/property: identity fallback chain, parameter exclusion, hash
  stability across attribute reordering; failed-vs-broken mapping table;
  flaky state transitions incl. broken-never-flaky and expiry.
- Integration: OTLP fixture batches (Rust/Java/Playwright shapes from the
  playground) → registry rows, attempt chains, fingerprints asserted via
  GraphQL.
- UI: Bun Vitest for feature logic; Playwright contract tests for list
  filters, detail tabs, cross-links (fixture-backed per plan 144).
- Cross-repo: plan 154 W4 §19 acceptance demo doubles as this plan's live
  gate.

## Done Criteria

- [ ] Identity/status/attempt/flaky contracts implemented exactly as D1-D6
      with tests; no run/launch identity inside test identity.
- [ ] Test spans/logs remain solely in native GreptimeDB tables; mutable
      state solely in Turso; no new raw-signal table.
- [ ] GraphQL `tests` namespace + `/tests` UI surface shipped behind the
      plan-149/152 architecture with Playwright gates green.
- [ ] Failed test detail shows: error, attempt chain, stitched SUT trace,
      logs window, shared-fingerprint issue link, release/version, history.
- [ ] Flaky states computed from at least two signals with expiry; visibly
      badged with evidence counts.
- [ ] Live verification against the plan 154 playground recorded under
      `docs/research/validation/`.

## STOP Conditions

- The derivation would require a custom GreptimeDB raw-signal table →
  native-otel-tables escalation process instead.
- Plans 149/152/153 not landed or plan 140 still migrating → do not start
  the UI step (backend steps 1-4 may proceed).
- `bundle-v1` change needed → route through plan 104.
- Identity contract conflicts with what plan 154 W4 emitters can produce →
  reconcile the cross-repo contract before writing code.

## Remove When

All done criteria checked with live evidence, triggers recorded in the
`plans/README.md` ledger (JUnit/CTRF ingestion, quarantine protocol, PR
feedback, impact analysis), and the operator confirms the surface; delete
this file and its index row in the same commit.
