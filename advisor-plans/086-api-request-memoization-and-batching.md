# Plan 086: Stop paying per-field store round-trips — request-scoped memoization, batched run stats, one histogram scan for RED, concurrent independent fetches, per-batch SSE serialization

> **Executor instructions**: Follow this plan step by step. Run every
> verification command and confirm the expected result before moving to the
> next step. If anything in the "STOP conditions" section occurs, stop and
> report — do not improvise. When done, update the status row for this plan
> in `advisor-plans/README.md` — unless a reviewer dispatched you and told you
> they maintain the index.
>
> **Drift check (run first)**: `git diff --stat df81d86..HEAD -- crates/parallax-api/src/lib.rs crates/parallax-server/src/serve.rs crates/parallax-server/src/live.rs crates/parallax-storage/src/adapter.rs`
> Note: `crates/parallax-api/src/lib.rs` carried uncommitted edits at planning
> time. Excerpts below are from that working tree. Mismatch = STOP.

## Status

- **Priority**: P1
- **Effort**: L
- **Risk**: MED (per-request context construction changes the GraphQL wiring; batch methods change the adapter trait)
- **Depends on**: 070 (its lib.rs fixes land first). Coordinate with 074/075/085 only through the adapter trait — this plan adds methods, it does not edit greptime.rs query bodies except to implement new trait methods. MUST land BEFORE 078 (the lib.rs split) — tell the index maintainer if 078 already ran.
- **Category**: perf
- **Planned at**: commit `df81d86`, 2026-07-10

## Why this matters

Every store call in `parallax-api` is one HTTP round-trip into GreptimeDB, and
Juniper 0.17 resolves sibling fields SEQUENTIALLY. Audit-traced costs per page
load (working tree, 2026-07-10): trace detail = 9 serial round-trips of which
6 re-fetch data already fetched in the same request (`spans_by_trace` × 5,
`logs_by_trace` × 3); runs list = 2 round-trips PER RUN row (default 50 rows =
100 round-trips, serial); service overview = up to 15 round-trips with the
same request-duration histogram scanned 4 separate times; command center ≈ 12.
`ApiContext` is built once at startup and shared (`serve.rs:306-314`), so
nothing is request-scoped and nothing memoizes. The live SSE path additionally
re-serializes every row once per subscriber. This plan makes the store-call
count per page proportional to the DATA needed, not the field count.

## Current state

- `crates/parallax-api/src/lib.rs` (~5,290 lines) — schema + all resolvers.
  - `ApiContext` (`lib.rs:29-33`): `{ store: Arc<dyn TelemetryStore>, metadata: Arc<MetadataStore>, otlp_grpc_port: u16 }`. No cache.
  - `spans_by_trace` call sites: `lib.rs:1904` (trace), `:1923` (traceEvents),
    `:1941` (linkedTraces), `:1956` (traceCriticalPath), `:1973/:1981`
    (traceCompare A/B), `:2091` (story), `:2136` (evidenceGaps), `:2638`,
    `:2721` (bundle). `logs_by_trace`: `:1994`, `:2096`, `:2141`, `:2271`,
    `:2333`, `:2643`, `:2747`.
  - The trace-detail UI batches `trace`, `linkedTraces`, `story`/`rpcTraceEvents`,
    `logsByTrace` into ONE GraphQL document (`ui/src/routes/traces.$traceId.tsx:192-210`)
    — so the same `trace_id`'s spans are fetched up to 5× per request.
  - `Run::stats` (`lib.rs:992-1037`): per-Run `OnceCell` dedupes `errorCount`+
    `traceCount` within one row but each row still runs `spans_by_run` then
    `error_events_by_traces` — 2 serial round-trips × N rows for the runs list.
  - `ServiceOverview` (`lib.rs:1386-1509`, excerpt verified):

```rust
    async fn duration_quantile(&self, context: &ApiContext, q: f64) -> FieldResult<Vec<SeriesPoint>> {
        for name in semconv::REQUEST_DURATION_METRICS {
            let series = context.store
                .histogram_quantile(name, Some(&self.service), self.from..=self.to, self.step, q)
                .await ...
```

    `latency_p50/p95/p99` each call `duration_quantile` (3 histogram scans),
    `request_rate` calls `histogram_count_series` (4th scan of the same
    table+window), `cpu`/`memory` loop candidate metric names serially.
  - `graphql_handler` (`crates/parallax-server/src/serve.rs:167-195`): executes
    against `state.context: Arc<ApiContext>` — one shared context.
- `crates/parallax-server/src/live.rs` — SSE tail. Per SUBSCRIBER, per batch:
  `filter_map` runs `log_event(log)` building a fresh `serde_json::Value` with
  `log.attributes.to_string()` + `log.resource.to_string()` per row
  (`live.rs:63-79, 86-102`; spans equivalent `:165-201`). N subscribers ⇒ N×
  full re-serialization of every batch.
- `crates/parallax-storage/src/adapter.rs` — the `TelemetryStore` trait; both
  adapters implement it (`greptime.rs`, `memory.rs`).
- CPU work on the read path: `story::project_story` and
  `trace_analysis::compare` run multi-pass regex normalization per span name
  (`parallax-core/src/fingerprint.rs:47-71` chain) inline on the runtime.
  Regexes are `OnceLock`-compiled (no per-call compile).

Conventions: strict clippy, cargo-nextest, `rtk` prefix, Conventional Commits
+ DCO + `Co-authored-by: Claude <noreply@anthropic.com>`, direct on `main`.

## Commands you will need

| Purpose | Command | Expected on success |
|---------|---------|---------------------|
| Build | `rtk cargo build --workspace` | exit 0 |
| API tests | `rtk cargo nextest run -p parallax-api` | all pass |
| Full suite | `rtk cargo nextest run --workspace` | all pass |
| Lint | `rtk cargo clippy --workspace --all-targets` | zero warnings |

## Scope

**In scope**:
- `crates/parallax-api/src/lib.rs`
- `crates/parallax-server/src/serve.rs` (per-request context construction)
- `crates/parallax-server/src/live.rs` (Step 6)
- `crates/parallax-storage/src/adapter.rs` + `memory.rs` + `greptime.rs`
  (ONLY adding the new batch trait methods + their impls)
- `advisor-plans/README.md` (status row)

**Out of scope**:
- Rewriting existing greptime.rs query bodies (Plans 075/085 own those).
- GraphQL schema shape changes (field names/types stay identical — clients
  depend on them).
- The UI (Plan 088).
- Dataloader-style generic batching frameworks — rejected earlier; this plan
  uses explicit batch methods only.

## Git workflow

Direct on `main`; Conventional Commits + `git commit -s` + Claude trailer.
Commit per step.

## Steps

### Step 1: Request-scoped `ApiContext` with a memo layer

In `serve.rs::graphql_handler`, build a fresh `ApiContext` per request instead
of cloning the shared Arc: keep `store`/`metadata`/`otlp_grpc_port` shared, add

```rust
pub struct RequestMemo {
    spans_by_trace: tokio::sync::Mutex<HashMap<String, Arc<Vec<SpanRow>>>>,
    logs_by_trace:  tokio::sync::Mutex<HashMap<String, Arc<Vec<LogRow>>>>,
}
```

on `ApiContext` (`Default`-initialized per request; `GraphQlState` keeps only
schema + shared parts). Add two context helpers
`spans_for(&self, trace_id) -> FieldResult<Arc<Vec<SpanRow>>>` /
`logs_for(...)` that check the memo, fetch once on miss, and store. Replace
the `spans_by_trace`/`logs_by_trace` call sites listed in Current state with
the helpers (mechanical; keep behavior identical — resolvers that mutate the
Vec must clone out of the Arc).

Memoize ONLY these two (highest fan-in, immutable within a request). Do not
build a generic cache.

**Verify**: `rtk cargo nextest run --workspace` → pass. Manual (serve +
playground running): open a trace detail page; server log/self-telemetry shows
ONE `spans_by_trace`-shaped query per trace id per request, not 4-5. If
self-telemetry is off, assert via a temporary `tracing::debug!` in the helper
(remove before commit) or the storage-side query log.

### Step 2: Batched run stats

Add to `adapter.rs`:

```rust
async fn spans_by_runs(&self, run_ids: &[String], limit_per_run: usize) -> anyhow::Result<HashMap<String, Vec<SpanRow>>>;
```

with a default implementation that loops `spans_by_run` (so both adapters work
immediately); override in `greptime.rs` with ONE query using
`<run_col> IN (...)` (same shape as `spans_by_run`'s native branch, plus
`ROW_NUMBER() OVER (PARTITION BY <run_col> ORDER BY "timestamp" DESC) <= limit`
via a subquery — the window-function pattern already exists in `traces_search`).
Then in the `runs` list resolver, prefetch stats for the whole page: collect
run ids, call `spans_by_runs` + ONE `error_events_by_traces` over the union of
trace ids, build a `HashMap<run_id, RunStats>`, and seed each `Run`'s
`OnceCell` (the plumbing exists — `Run::stats` already caches; give `Run` an
optional pre-seeded stats field).

The single-run `run()` path stays on the existing per-run code.

**Verify**: `rtk cargo nextest run --workspace` → pass, including a new test:
memory adapter, 3 runs with spans/errors, `runs` list returns identical
errorCount/traceCount to per-run `run()` calls.

### Step 3: One histogram scan for the RED panel

Consume Plan 085's `histogram_quantiles(name, service, range, step, &[0.5,0.95,0.99])`
(if 085 has not landed, add the trait method here with the default
loop-implementation and let 085 optimize it — coordinate via the index).
Restructure `ServiceOverview`: add a private
`async fn red_source(&self, context) -> FieldResult<Arc<RedSource>>` cached in
a per-object `OnceCell` (same pattern as `Run::stats`, `lib.rs:992-1037`),
resolving the metric name ONCE across `REQUEST_DURATION_METRICS`, fetching
quantiles (one scan) + `histogram_count_series` (second scan). `latency_p50/95/99`
and `request_rate` project from it. `cpu`/`memory` keep their candidate loops
(different metrics) but run concurrently via `tokio::join!` inside a shared
`OnceCell` initializer if trivially expressible; otherwise leave.

Service page cost: 15 → ≤ 6 round-trips.

**Verify**: `rtk cargo nextest run --workspace` → pass; conformance service
scenario unchanged on memory adapter.

### Step 4: `tokio::join!` the independent pairs

Convert these serial awaits on INDEPENDENT data to concurrent joins
(post-Step-1 the trace ones operate through the memo helpers — join the helper
futures):

- `story` resolver (`:2089-2098`) and `evidenceGaps` (`:2134-2143`): spans + logs.
- `traceCompare` (`:1971-1983`): spans(A) + spans(B).
- service_map page resolver (`:1750-1759`): `service_summaries` + `service_map`.
- `bundle` (`:2604-2769`): the run-branch fetches that don't feed each other
  (spans + logs; issues after events stays sequential where it consumes the
  result) and the `bundle_metric_windows` loop (`:2994-3083`) → `join_all`
  over the 3 fixed metrics.

**Verify**: `rtk cargo nextest run --workspace` → pass (behavioral no-op;
error propagation must keep failing the field the same way — use `try_join!`).

### Step 5: Bound the anchored reads

`spans_by_trace`/`logs_by_trace` return unbounded rows. In the memo helpers
from Step 1, request at most `MAX_ROWS` (the API-side constant, `lib.rs:51`)
via the storage methods' existing limit parameters where present; where the
storage method has no limit parameter, truncate in the helper and set a
`truncated` flag on the memo entry. Surface truncation the way `agentSession`
already does (`lib.rs:2076` checks `spans.len() == MAX_ROWS`) — do NOT add new
GraphQL fields in this plan; a `tracing::warn!` plus the existing behavior is
enough.

**Verify**: unit test on the helper truncation with the memory adapter.

### Step 6: Serialize SSE batches once

In `live.rs`: pre-serialize per BATCH, not per subscriber. Change the
broadcast payload from `Arc<[LogRow]>` to a small struct
`Arc<LiveLogBatch { rows: Arc<[LogRow]>, rendered: Vec<serde_json::Value> }>`
built ONCE by the worker before `send` (only when `receiver_count() > 0`,
which the worker already checks). Subscribers filter on `rows[i]` and push the
pre-rendered `rendered[i]` — `filter.matches` keeps using the typed row.
Duplicate `resource`/`attributes` strings across rows of one batch render once
per row still, but only once per BATCH total instead of once per subscriber.
Mirror for spans. Update `worker.rs` send sites accordingly (types only — the
gating logic stays; Plan 087 restructures the worker itself, so keep this diff
minimal and mechanical).

**Verify**: `rtk cargo nextest run -p parallax-server` → live tests pass
(`log_event` unit test moves/updates with the code). Manual: two concurrent
`curl -N http://127.0.0.1:4000/v1/logs/stream` clients both receive events.

### Step 7: `spawn_blocking` for trace analysis (measure-gated)

Measure first: time `trace_analysis::compare` on two 500-span traces (write a
quick `#[bench]`-style test or a timed unit test with synthetic spans printed
to stderr). If > ~5 ms, wrap the pure-CPU calls in the `traceCompare`,
`traceCriticalPath`, and `story` resolvers in `tokio::task::spawn_blocking`.
If ≤ 5 ms, skip and record the measured number in the commit message.

**Verify**: measurement recorded; if wrapped, full suite passes.

### Step 8: Full gates

**Verify**: `rtk cargo fmt --all`; `rtk cargo clippy --workspace --all-targets`
zero warnings; `rtk cargo nextest run --workspace` all pass.

## Test plan

- New: batched run-stats parity test (Step 2), memo-helper truncation test
  (Step 5), SSE two-subscriber smoke via existing live test patterns (Step 6).
- Existing API/server suites unchanged otherwise; any assertion change =
  behavior change = STOP.
- Round-trip counts: assert qualitatively via storage-call counting on the
  memory adapter if a counting decorator exists; if not, add a tiny
  `#[cfg(test)]` counting wrapper around `TelemetryStore` for the trace-detail
  and runs-list tests (counts: trace detail spans fetches == 1; runs list
  spans fetches == 1).

## Done criteria

- [ ] `grep -c "spans_by_trace" crates/parallax-api/src/lib.rs` → ≤ 2 (the memo helper + possibly bundle) — all resolver sites go through `spans_for`
- [ ] `grep -n "spans_by_runs" crates/parallax-storage/src/adapter.rs crates/parallax-api/src/lib.rs` → trait method + list-resolver use
- [ ] `grep -cn "duration_quantile(context" crates/parallax-api/src/lib.rs` → 0 (replaced by the shared red_source)
- [ ] `grep -n "try_join\|join!" crates/parallax-api/src/lib.rs` → ≥ 4 matches
- [ ] `grep -n "rendered" crates/parallax-server/src/live.rs` → per-batch pre-serialization exists
- [ ] `rtk cargo nextest run --workspace` exits 0; clippy zero warnings
- [ ] `git status` clean outside in-scope list
- [ ] `advisor-plans/README.md` status row updated (and note for 078's executor that lib.rs moved under them if 078 is still TODO)

## STOP conditions

Stop and report back (do not improvise) if:

- Plan 078 (lib.rs split) already ran — the line references here are void;
  re-derive against the new module layout only if the functions are findable
  by name, otherwise report.
- Making `ApiContext` per-request breaks Juniper's `Context` trait bounds or
  the subscription/SSE wiring in a way that needs schema-type changes.
- The batched `spans_by_runs` window-function query fails on the live engine.
- Any existing test's ASSERTIONS need weakening.
- Step 6's payload-type change ripples into the UI's SSE event shape (it must
  not — bytes on the wire stay identical; if they can't, STOP).

## Maintenance notes

- The memo layer is REQUEST-scoped by construction; if anyone later makes
  `ApiContext` long-lived again, the memo becomes a stale-data bug. The
  per-request construction in `graphql_handler` is the load-bearing line.
- Plan 078's split must keep the memo helpers on the context type, not in a
  resolver module.
- Batch methods (`spans_by_runs`, `histogram_quantiles`) are the pattern for
  future N+1s (e.g. `Issue::latestEvent` if a client ever lists it — latent
  500-round-trip risk recorded in the audit).
- Deferred: per-service `resource` string caching in span/log serialization
  (`lib.rs:429-434, 474-479`) — measure after Steps 1-4 land; GraphQL
  variables/codegen for the UI client (recorded separately, Plan 079 family).
