# Plan 155: Test reporting surface residual

> **Executor instructions**: Tests as first-class surface fused with traces,
> logs, issues, runs. Native Greptime tables only; Turso for mutable registry.
> Identity never uses invocation/run as test identity (plan 156 contract:
> `(test_variant_key, cli.invocation.id, attempt)`). Do not half-land UI.

## Status

- **Priority**: P1
- **Effort**: XL residual
- **Risk**: MEDIUM
- **Depends on**: 149, 152, 153, 140 (DONE hard deps); soft 121/124 open
- **Category**: product capability / ingest derivation / UI surface
- **Status**: IN PROGRESS — domain model + identity derivation landed
- **Evidence base**:
  [`docs/research/market/test-reporting-ecosystem.md`](../docs/research/market/test-reporting-ecosystem.md)

## Landed (do not replay)

- `parallax-model` test reporting domain: versioned `TestCaseKey` /
  `TestVariantKey`, result identity, attempt chains, rollups (`FlakyPass`),
  flaky state machine boundary, identity fallback
  (explicit → code reference → name path), and typed case/variant/flaky
  persistence records without raw telemetry payloads.
- Eight tests across the suites in
  `crates/parallax-model/src/test_reporting/`; strict crate Clippy passes.
  Persistence adapter/migration behavior still requires independent review.
- Preliminary Turso schema and typed idempotent upserts now cover cases,
  variants, result references, and flaky state. Result rows store native
  trace/span identifiers, never copied raw telemetry. Schema inventory and
  full four-record persistence fixtures pass; query/read APIs and independent
  migration review remain.
- Typed reads now cover case/variant lookup, bounded invocation attempt lists,
  and flaky state. Decode rejects malformed versioned keys, attempts, enums,
  trace IDs, and JSON rather than manufacturing defaults. Explorer/filter
  queries and independent migration review remain.
- The query-neutral `MetadataStore` port now exposes all four test-reporting
  upserts and reads. Dynamic-dispatch fixtures prove server/API composition can
  persist and query the Turso registry without concrete-store downcasts.
- Pure `parallax-analysis` derivation now recognizes parented test spans by
  `test.case.name`, derives strict ordered identities/attempts/configuration,
  separates assertion failures from harness breakage, and copies only the
  matching normalized production error fingerprint. Worker persistence and
  explorer queries remain.
- The trace ingest worker now normalizes each batch once, projects valid test
  results, and idempotently persists case/variant/result records after native
  trace and issue recording. A worker integration fixture proves a parented
  failed test references the same stored production issue fingerprint.
- A variant-scoped metadata explorer now selects the latest eligible invocation,
  rolls up every attempt without latest-wins masking (`flaky_pass`), joins
  strict case/variant/result/flaky records, and exposes bound-parameter filters
  with hard page/offset clamps through the query-neutral port. Owner,
  mute/known, resolution, and session-lifecycle filters remain unavailable
  until their schema-owning residuals land.
- The generated GraphQL contract now exposes typed `testCases` explorer
  filters, rollups, parameters, configuration dimensions, flaky evidence, and
  native trace/span/fingerprint/invocation references with resolver and port
  clamps. Honest `testCase` detail/history still requires the bounded
  case-detail metadata read; no explorer-substring substitute was used.
- The query-neutral metadata port now exposes bounded case-to-variant (100)
  and variant-to-result-history (500) reads with strict versioned-key
  validation and deterministic newest-first history. A Turso fixture proves
  case isolation, attempt-chain preservation, ordering, and caller limits;
  the preliminary GraphQL `testCase` composition now consumes these reads;
  broader independent verification remains for the owning agent.
- A preliminary typed `testCase` GraphQL detail now exposes case identity,
  at most 20 variants, and at most 50 newest-first result references plus
  flaky evidence per variant. Malformed keys fail as invalid input and valid
  missing cases return null. Targeted API tests and strict Clippy pass; the
  owning agent should replace the bounded per-variant reads with one batched
  metadata query before large multi-variant UI loads and regenerate all UI
  operation artifacts with the currently owned schema work.
- Pure flaky-window evaluation now groups attempt rows into completed
  invocation chains before deriving evidence, so fail-then-pass retries set
  only the intra-invocation signal and cannot fabricate cross-invocation
  transitions. It handles exact time bounds, same-revision divergence,
  recovery streaks, consistent-failure precedence, missing revisions, and
  malformed duplicate attempts deterministically. Scheduling, bounded Turso
  scans, and state upserts remain for the owning job integration.
- The state machine now lets both `Flaky` and consistently-failing `Broken`
  variants recover to `Fixed` only after the configured clean-pass threshold;
  the prior permanent-`Broken` behavior would have made scheduled evaluation
  preserve stale failure state forever.
- The nextest adapter foundation now strictly normalizes the documented
  per-test process variables (`NEXTEST_*`, available since 0.9.116), retry
  bounds, unique attempt ID, optional `TRACEPARENT`, and the wrapper-provided
  `CLI_INVOCATION_ID`. It deliberately never substitutes `NEXTEST_RUN_ID` for
  product invocation identity. The support crate/export lifecycle and JUnit
  reconciliation remain for the owning adapter implementation.

Design decisions D1–D9 (identity, native tables, status taxonomy, attempt
chains, shared fingerprints, flaky SM, `/tests` surface, session semantics,
runner adapters) remain binding — see Git history of this plan for full text
if needed; do not reopen.

## Residual only

1. ~~Explorer/filter + Turso schema~~ — landed (metadata explorer + fixtures).
2. ~~Ingest derivation~~ — landed (`parallax-analysis` + worker persist).
3. ~~GraphQL explorer~~ — `testCases` + SDL export landed; residual:
   batch the preliminary `testCase` detail/history reads, regenerate UI
   operations, attempt-chain history, mute/known/owner fields when schema exists.
4. UI `features/tests`: list + detail + live session tree (after architecture
   owners; React Flow not required here).
5. Flaky job over ingested results — pure invocation-chain evaluator landed;
   bounded Turso candidate scan + result windows landed (port + fixtures);
   residual scheduler loop that applies evaluate_flaky_evidence + state
   upserts, and mute/known flags (no runner quarantine enforcement in V1).
6. Runner adapters: nextest env normalization landed; residual nextest support
   crate/export lifecycle, JUnit listener jar, and JUnit XML reconciliation
   gap-fill.
7. Live e2e vs plan 154 W4 playground payload; validation evidence under
   `docs/research/validation/`.

## Done Criteria

- [ ] Identity/status/attempt/flaky contracts + tests; no run id in identity.
- [ ] Native Greptime only for raw test spans/logs; Turso for mutable state.
- [ ] GraphQL + `/tests` UI with Playwright gates; failed detail shows error,
      attempts, SUT trace, logs window, issue link, version, history.
- [ ] Flaky from ≥2 signals with expiry; live session SSE + unresolved headline.
- [ ] Both D9 adapters + killed-test reconciliation proven.

## STOP / Remove When

STOP on custom raw-signal table or identity that embeds invocation/run.
Delete when surface ships with live playground proof.
