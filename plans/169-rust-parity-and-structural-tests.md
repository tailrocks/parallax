# Plan 169: Fake/engine parity, resolver depth, and metadata versioning (wave 2)

> **Executor instructions**: Follow this plan step by step. Run every
> verification command and confirm the expected result before moving on. If
> anything in "STOP conditions" occurs, stop and report — do not improvise.
> When done, update this plan's row in `plans/README.md`.
>
> **Drift check (run first)**: `git diff --stat f6208070..HEAD -- crates/parallax-test-support/src/conformance.rs crates/parallax-server/tests/ crates/parallax-api/src/resolvers/issues* crates/parallax-metadata/src/turso.rs crates/parallax-metadata/src/turso/connection.rs crates/parallax-metadata/src/turso/tests.rs crates/parallax-server/src/serve.rs crates/parallax-ingest/src/tests.rs crates/parallax-evidence/src/bundle/`
> — on mismatch with the excerpts below, STOP.
>
> **Ratchet gate (applies to EVERY step)**: `ratchet.toml` pins per-file
> `rust.assertions` / `rust.file-lines` / `rust.inline-test-modules` with
> EXACT-match enforcement (above OR below the pin errors). Every step that
> adds tests must update the touched files' rows to the new actuals in the
> same commit and pass `cargo xtask policy --only structural`. New test
> files need new rows.

## Status

- **Priority**: P2
- **Effort**: L
- **Risk**: MED (parity work will likely surface real fake-vs-engine
  divergences; migration versioning touches live user data)
- **Depends on**: plans/168-rust-correctness-test-wave.md (Step 1's real
  `cargo xtask integration` gate is the runway here)
- **Category**: tests
- **Planned at**: parallax `f6208070`, 2026-08-13

## Why this matters

53 of parallax-api's 60 resolver tests run against the in-memory fake.
Where the fake and GreptimeDB disagree — percentile semantics, window edge
inclusivity, service-map edge dedup — the suite is green and production is
wrong. The dual-run conformance harness exists and is 60% unused. Separately:
the issues resolver (the product's core surface) has 1 test vs metrics' 754
test LOC; the Turso schema has no version marker so upgrade paths are
untestable by construction; and `serve.rs` (698 LOC, 45 commits/6mo) has
zero tests. Wave 2 makes the fake trustworthy, the core resolver covered,
and upgrades provable.

## Current state (verified)

- `crates/parallax-test-support/src/conformance.rs` — five scenarios:
  `assert_empty`, `assert_seeded` run against the live engine in
  `crates/parallax-server/tests/m6_conformance_greptime.rs:72,99,123`.
  `overview_totals_scenario` (`:231`), `attribute_compare_scenario` (`:237`),
  `service_map_scenario` (`:244`) take `&dyn TelemetryStore` but are called
  only from `crates/parallax-test-support/src/memory/tests/traces_services.rs:357-363`
  (fake-only). `trace_search_scenario` (`:215`) and
  `log_count_series_scenario` (`:220`) are hard-typed `&MemoryStore` — they
  CANNOT run against Greptime today.
- `crates/parallax-api/src/resolvers/issues.rs` (531 LOC) +
  `issues/nested.rs` (209) — `issues/tests.rs` = 70 LOC, 1 test. Exemplar
  to mirror: `crates/parallax-api/src/resolvers/metrics/tests.rs` (754 LOC,
  MemoryStore-backed, table-style).
- `crates/parallax-metadata/src/turso.rs:45` — single `SCHEMA` const,
  `CREATE TABLE IF NOT EXISTS` × 29 tables; forward-only
  `DROP TABLE IF EXISTS runs` at `:60`;
  `crates/parallax-metadata/src/turso/connection.rs:12-20` sniffs
  `PRAGMA table_info(issues)` to add `resolved_at`. No
  `PRAGMA user_version`. Upgrade tests: exactly two bespoke ones
  (`turso/tests.rs:444`, `:1258`).
- `crates/parallax-server/src/serve.rs` — 698 LOC, no `mod tests`; owns
  router assembly (`:232`), `start_assembled` (`:335`), graceful shutdown
  (`:65`), five background-loop spawners. Siblings `otlp_http.rs`,
  `otlp_grpc.rs`, `engine_io.rs` also test-free.
- Property-test upgrades deferred from wave 1: two determinism proptests in
  `crates/parallax-ingest/src/tests.rs:399-456` call f(x)==f(x) — replace
  with real invariants.
- Hypothesis ranking: `crates/parallax-evidence/src/bundle/ranking.rs:3`
  (`rank_hypotheses`, 100 LOC) asserted only as non-empty
  (`bundle/tests.rs:358`) — ordering never tested.

Constraints (binding): GreptimeDB+Turso mandatory; `StorageAdapter` is a
capability/test boundary, not an engine-substitution promise — parity work
must not add a fallback-engine smell; migrations must adopt existing user
DBs safely (fail-closed).

## Commands you will need

| Purpose | Command | Expected |
|---|---|---|
| Unit | `cargo xtask test` | pass |
| Real engine | `cargo xtask integration` (plan 168 Step 1) | runs real-engine profile, pass |
| One suite | `cargo nextest run -p parallax-metadata -E 'test(/migration/)'` | pass |
| Gates | `cargo xtask ci --fast && cargo xtask lint && cargo xtask arch` | green |
| Structural policy | `cargo xtask policy --only structural` | green after ratchet rows updated |

## Scope

**In scope**: `crates/parallax-test-support/src/conformance.rs` (widen two
scenarios to `&dyn TelemetryStore`), `crates/parallax-server/tests/m6_conformance_greptime.rs`
(call all five), `crates/parallax-api/src/resolvers/issues/tests.rs`,
`crates/parallax-metadata/src/turso.rs` + `turso/connection.rs` +
`turso/tests.rs` (+ a `migrations.rs` if introduced),
`crates/parallax-server/src/serve.rs` (tests + minimal seams only),
`crates/parallax-ingest/src/tests.rs` (property upgrades),
`crates/parallax-evidence/src/bundle/` (ranking tests).

**Out of scope**: resolver behavior changes (test-first only — divergences
found become plan-166 DISCREPANCY rows, or one-line fake fixes where the
ENGINE is authoritative); any new storage capability; UI.

## Git workflow

PR-only `main`; 3 PRs suggested (parity, issues+ranking+properties,
migrations+serve); `git commit -s`; Conventional Commits; agent trailer per
`COMMITS.md`.

## Steps

### Step 1: Make the conformance scenarios assert, then run them dual

Two defects to fix, in order:

(a) **Seeding is fake-coupled**: `trace_search_scenario` calls
`seed_memory(store)` whose signature is `pub fn seed_memory(&MemoryStore)`
(`conformance.rs:21`, call at `:216`) — the scenario cannot take a trait
object while it seeds. Restructure: scenarios take a PRE-SEEDED
`&dyn TelemetryStore` plus an expectations struct; move seeding to the
caller (memory tests keep `seed_memory`; the Greptime test seeds the same
logical dataset through its existing fixture path in
`m6_conformance_greptime.rs`). Apply the same split to
`log_count_series_scenario` (`:220`).

(b) **Three scenarios assert almost nothing**: `overview_totals_scenario`
asserts `trace_count < u64::MAX`, `attribute_compare_scenario` and
`service_map_scenario` only `.await?` (`conformance.rs:231-247`). Give
every scenario CONCRETE expected values derived from the shared seeded
dataset (span counts, service names, edge list, compare deltas) so a
fake-vs-engine divergence FAILS the test rather than needing a manual
diff.

Then call all five scenarios from `m6_conformance_greptime.rs` after its
seeded fixture, mirroring the `assert_seeded` invocation style.

**Verify**: `cargo xtask test` (fake path green with the strengthened
asserts) and `cargo xtask integration` — five scenarios run against live
Greptime. Each engine-side failure = real divergence: record as
`DISCREPANCY: <query> | conformance | parallax | fake=<x> engine=<y> | … | parallax bug`
in the inventory doc W5 list; where the FAKE is wrong and the fix is a
small fake-side change, fix the fake in the same PR.

Additionally: add an Arrow-vs-JSON transport parity case to the Greptime
suite — same SQL through both read transports, equal decoded rows (this
replaces the tautological `row_count_parity_json_shape` that plan 168
Step 3 deletes).

### Step 2: Issues resolver suite

Build `crates/parallax-api/src/resolvers/issues/tests.rs` to the
`metrics/tests.rs` standard against MemoryStore: list filters
(service/status/query/time/tag), sort orders, paging boundaries
(`clamp_limit` interplay), single-issue lookup miss, `issueTrend` bucket
edges, status-transition via mutation + persistence, nested occurrence
paging, and the markdown bundle projection path reached from
`issues.rs:447` (assert stable section headers, not full text).

**Verify**: `cargo nextest run -p parallax-api -E 'test(/issues/)'` →
≥ 12 test cases listed (was 1), all pass.

### Step 3: Hypothesis-ranking order tests

`crates/parallax-evidence/src/bundle/`: construct bundles with controlled
evidence mixes and assert the ORDER `rank_hypotheses` returns (top
hypothesis matches the dominant evidence class; ties stable). Also a
regression: empty-evidence bundle → deterministic fallback order.

**Verify**: `cargo nextest run -p parallax-evidence -E 'test(/rank/)'` → pass.

### Step 4: Versioned metadata migrations

Introduce `PRAGMA user_version`-based numbered migrations in
parallax-metadata: version 0 = "pre-versioning DB" (adopt: run the existing
sniffing upgrades, then stamp current N); each future change = numbered
step. Move the `runs` drop and `resolved_at` add into steps 1 and 2.
Fixture tests: check in tiny pre-built DB files (or build them in-test with
historical SCHEMA strings) for v0-with-runs-table and v0-without-resolved_at;
open → assert final schema + data preserved + `user_version` == N. A
downgrade-protection test: opening a DB with `user_version > N` fails
closed with a clear error.

**Verify**: `cargo nextest run -p parallax-metadata -E 'test(/migration|user_version/)'`
→ ≥4 tests pass; existing turso tests still green.

### Step 5: serve.rs characterization

Add tests: router assembly exposes the expected route set (snapshot of
paths from `build_api_router`/`:232` — assert presence of `/graphql`,
`/health`, `/version`, `/v1/logs/stream`, `/v1/traces/stream`, sentry +
github routes per config flags); shutdown characterization pinning
CURRENT semantics — `shutdown_graceful` (`serve.rs:65-70`) ABORTS listener
tasks then drains workers (it does not join listeners): assert the abort +
drain sequence completes within a timeout and the drained workers finish
cleanly (introduce the smallest seam needed — e.g. return handles from the
spawn helper — no behavior change).

**Verify**: `cargo nextest run -p parallax-server -E 'test(/serve/)'` → pass;
`cargo xtask arch` green.

### Step 6: Property-test upgrades

Replace the two f(x)==f(x) determinism proptests in
`crates/parallax-ingest/src/tests.rs:399-456` with: span-count
conservation (normalize_traces), log-record conservation, attribute-key
conservation. (Metrics point-count conservation is OWNED BY plan 168
Step 6 — if it already landed there, do not duplicate; if 168 has not
landed, still leave metrics to it.)

**Verify**: `cargo nextest run -p parallax-ingest` → pass.

## Test plan

All steps are tests plus two minimal seams (scenario signatures, join
handles) and the migration mechanism (which is production code with its own
fixture tests — the riskiest piece; review it hardest).

## Done criteria

- [x] All five conformance scenarios execute in
      `m6_conformance_greptime.rs` under `cargo xtask integration`.
- [x] Issues resolver suite ≥ 10 cases, green.
- [x] `rank_hypotheses` ordering pinned by tests.
- [x] `PRAGMA user_version` migrations with v0-adoption + fixture tests +
      fail-closed future-version guard.
- [x] serve.rs route-table + shutdown-join tests green.
- [x] Determinism proptests replaced with conservation properties.
- [x] `cargo xtask ci --fast`, `lint`, `test`, `integration`, `arch`, `policy --only structural` all green.
- [x] `plans/README.md` row updated.

## STOP conditions

1. Drift check fails.
2. A `&MemoryStore` scenario uses fake-only internals — report what it
   needs from the trait instead of extending the trait ad hoc.
3. Parity reveals a divergence whose "correct" side is unclear from the
   spec — record DISCREPANCY, don't pick a winner silently.
4. Migration adoption cannot distinguish a v0 DB from a corrupt file —
   report; fail-closed design question for the operator.
5. serve.rs seam requires changing shutdown semantics — report first.

## Maintenance notes

- Every future resolver gets the dual proof: MemoryStore suite + a
  conformance scenario when its semantics are aggregation-sensitive.
- Migration rule going forward: schema change ⇒ numbered step + fixture
  test in the same PR; the `SCHEMA` const becomes "current shape" only.
- Reviewer: watch for parity "fixes" that quietly change engine-side SQL —
  engine is authoritative unless the spec says otherwise.
