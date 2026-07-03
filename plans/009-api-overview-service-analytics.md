# Plan 009: API — overview aggregates, per-signal count series, service summaries, span-derived RED

> **Executor instructions**: Step by step; verify each; STOP conditions
> binding; update `plans/README.md` when done. Pure Rust plan — no UI changes.
>
> **Drift check (run first)**: `git diff --stat ad9115d..HEAD -- crates/parallax-api crates/parallax-storage`
> If the cited symbols moved, re-locate them by name before proceeding; if
> their shape changed, STOP.

## Status

- **Priority**: P1
- **Effort**: L
- **Risk**: MED
- **Depends on**: none (parallel to UI plans; UI consumers are plans 013/015)
- **Category**: direction (API extension for the redesigned screens)
- **Planned at**: commit `ad9115d`, 2026-07-03

## Why this matters

The redesigned Overview (plan 013) and Services (plan 015) screens need data the GraphQL API
cannot serve today: whole-system totals and rates, per-signal volume series for trend charts,
a one-query service summary list, and request/error/latency (RED) analytics that work for
services that only send **traces** (no OTel metrics). Today `serviceOverview` depends
entirely on app-sent histogram metrics — a service emitting only spans shows blank panels
(`ui/src/routes/services.tsx:272` renders its empty state precisely because of this).
Extending the API is explicitly authorized by the operator (2026-07-03).

## Current state

- GraphQL is Juniper, single file `crates/parallax-api/src/lib.rs`:
  - `pub struct Query` at `:747`; resolvers `#[graphql_object(context = ApiContext)] impl
    Query` from `:749`. `Int` is i32 (`saturate_i32` at `:35`); nanosecond timestamps cross
    as **strings**; `MAX_ROWS = 500` (`:41`).
  - Existing series types: `Point { tsNanos, value }` (`:487`), `Series { groupValue,
    points }` (`:501`), `ServiceOverview` (`:518`, resolvers `:573-677` — CPU/memory from
    well-known metric-name lists, requestRate from histogram `_count`, latency from
    `histogramQuantile`, errorRate from `error_count_series`).
  - `services` returns bare `Vec<String>` (`:1344`; storage `service_names`
    `greptime.rs:699`).
- Storage boundary: `crates/parallax-storage/src/adapter.rs` — trait `TelemetryStore`
  (`:61+`); implementations `crates/parallax-storage/src/greptime.rs` (product) and
  `crates/parallax-storage/src/memory.rs` (tests/dev only — must stay in sync; repo rule:
  GreptimeDB+Turso are the only product engines).
- Existing bucketing pattern to mirror: `log_count_series` (`greptime.rs:1222-1248`) — SQL
  `date_bin(INTERVAL '{step_secs} seconds', "timestamp")` grouped count over
  `opentelemetry_logs`; `error_count_series` (`greptime.rs:1194-1220`) — same over
  `error_events`, **currently requires a service** and is only reachable through
  `ServiceOverview.errorRate`.
- Tables (created by GreptimeDB OTLP ingest / bootstrap, see `greptime.rs`):
  `opentelemetry_traces` (cols incl. `timestamp` ns, `service_name`, `trace_id`,
  `parent_span_id`, `duration_nano`, `span_status_code`), `opentelemetry_logs`,
  `error_events` (`ts`, `service`, `fingerprint`, …), per-metric tables. Helper fns in
  greptime.rs: `sql`, `sql_lenient`, `escape`, `sql_ts`, `u128_at`.
- Engine version policy: latest stable GreptimeDB (`AGENTS.md`); quantile support must be
  **verified against the running engine** (Step 4) — candidates:
  `approx_percentile_cont(col, q)` / `uddsketch`-based functions.

## Commands you will need

From the repo root `/Users/donbeave/Projects/tailrocks/parallax-project/parallax`:

| Purpose | Command | Expected |
|---------|---------|----------|
| Format  | `rtk cargo fmt --all` | no diff after |
| Lint    | `rtk cargo clippy --workspace --all-targets` | zero warnings (repo rule) |
| Tests   | `rtk cargo nextest run` | all pass |
| Build   | `rtk cargo build --workspace` | exit 0 |
| Live engine (for Step 4 verification) | `rtk cargo run -p parallax-cli -- serve` (or the repo's documented serve command; see README) | ready banner with GraphQL/OTLP ports |

## Scope

**In scope**:
- `crates/parallax-storage/src/adapter.rs` (trait additions + new structs)
- `crates/parallax-storage/src/greptime.rs` (SQL implementations)
- `crates/parallax-storage/src/memory.rs` (in-memory implementations for tests)
- `crates/parallax-api/src/lib.rs` (GraphQL types + resolvers)
- Tests in those crates (unit + the crates' existing test layout)

**Out of scope**:
- Any UI file. Any CLI surface. `ServiceOverview` stays as-is (existing consumers);
  the new `serviceRed` complements it.
- Ingest/write paths (`worker.rs`, derive) — read-only additions here.

## Git workflow

`main`; Conventional Commits (`feat(api): overview + service analytics queries`);
`git commit -s`; trailer `Co-authored-by: Claude <noreply@anthropic.com>`.

## Steps

### Step 1: Storage trait + structs (`adapter.rs`)

Add (names final — UI plans bind to them):

```rust
pub struct OverviewTotals {
    pub span_count: u64, pub trace_count: u64, pub log_count: u64,
    pub metric_point_count: u64, pub error_count: u64,
    pub error_rate: f64,            // errored spans / spans in window
    pub active_services: u64,
}
pub struct ServiceSummary {
    pub name: String, pub last_seen_nanos: u128,
    pub span_count: u64, pub error_count: u64,
    pub p95_ms: Option<f64>,
}
pub enum SignalKind { Spans, Traces, Logs, Errors, MetricPoints }
```

Trait methods:
```rust
async fn overview_totals(&self, range: RangeInclusive<u128>) -> anyhow::Result<OverviewTotals>;
async fn signal_count_series(&self, kind: SignalKind, service: Option<&str>,
    range: RangeInclusive<u128>, step_nanos: u128) -> anyhow::Result<Vec<SeriesPoint>>;
async fn service_summaries(&self, range: RangeInclusive<u128>)
    -> anyhow::Result<Vec<ServiceSummary>>;
async fn span_red_series(&self, service: Option<&str>, range: RangeInclusive<u128>,
    step_nanos: u128) -> anyhow::Result<SpanRed>;   // SpanRed { rate, error_rate, p50, p95, p99: Vec<SeriesPoint> }
```

**Verify**: `rtk cargo build --workspace` fails ONLY with "not implemented" for the two
stores — then implement Steps 2-3 before the next build gate.

### Step 2: GreptimeDB implementations (`greptime.rs`)

Mirror the `log_count_series` pattern (date_bin + COUNT; `sql_lenient` so missing tables →
empty, matching existing behavior):
- `overview_totals`: one query per signal within the window — `COUNT(*)`,
  `COUNT(DISTINCT trace_id)`, errored spans via `span_status_code = 'STATUS_CODE_ERROR'`
  (confirm the stored literal by inspecting existing span queries in this file — `SpanRow`
  mapping shows the status column values), `COUNT(DISTINCT service_name)`; metric points =
  sum over discovered per-metric tables **bounded to the window** — if that fan-out is
  expensive, return 0 for `metric_point_count` in v1 and document it (STOP is not needed;
  note it in the report).
- `signal_count_series`: `Spans`/`Traces` over `opentelemetry_traces` (traces =
  `COUNT(DISTINCT trace_id)` per bucket), `Logs` over `opentelemetry_logs`, `Errors` over
  `error_events` (service optional — generalize `error_count_series`'s WHERE), 
  `MetricPoints` may return empty in v1 (same note).
- `service_summaries`: one grouped query over `opentelemetry_traces`
  (`GROUP BY service_name`: `MAX(timestamp)`, `COUNT(*)`, error count) + optional p95 via
  the Step 4 quantile (skip p95 with `None` if quantile unsupported).
- `span_red_series`: per-bucket over `opentelemetry_traces` — rate = spans/bucket (or
  root-spans/bucket; pick spans/bucket and document), error rate = errored/total per bucket,
  p50/p95/p99 = quantile over `duration_nano` per bucket (Step 4 function).

**Verify**: `rtk cargo build --workspace` → exit 0.

### Step 3: Memory implementations (`memory.rs`)

Straightforward in-memory math over stored rows (exact percentiles fine). Keep semantics
identical (window inclusive, ns strings).

**Verify**: `rtk cargo nextest run -p parallax-storage` → pass (with Step 6 tests).

### Step 4: Verify engine quantile support (REQUIRED before shipping p95/p99)

With `parallax serve` running against real GreptimeDB, run through the existing raw-SQL path
(GraphQL `sql` query or the storage `raw_sql` helper in a test):
`SELECT approx_percentile_cont(duration_nano, 0.95) FROM opentelemetry_traces` (and the
`WITHIN GROUP (ORDER BY …)` form if the plain form errors — engine versions differ).
Whichever form the engine accepts becomes the implementation; record the working SQL in a
code comment. If NO quantile function works, set all percentile fields to `None`/empty,
leave rate/error-rate fully functional, and report it as a limitation — do not fake
percentiles.

### Step 5: GraphQL surface (`parallax-api/src/lib.rs`)

New types + resolvers on `Query` (mirror neighboring resolvers' arg parsing — the ns-string
`parse` closure in `traces` at `:1138-1145`):
- `overview(fromNanos: String!, toNanos: String!): Overview!` — counts as **String** fields
  (i64/u64-safe; do NOT saturate through i32) + `errorRate: Float!` +
  `activeServices: Int!`.
- `signalCountSeries(kind: SignalKind!, service: String, fromNanos: String!,
  toNanos: String!, stepSeconds: Int): [Point!]!` (enum `SPANS TRACES LOGS ERRORS
  METRIC_POINTS`).
- `serviceList(fromNanos: String!, toNanos: String!): [ServiceSummary!]!` (`lastSeenNanos`
  as String, counts as String, `p95Ms: Float`).
- `serviceRed(service: String, fromNanos: String!, toNanos: String!, stepSeconds: Int):
  SpanRed!` with `rate/errorRate/p50/p95/p99: [Point!]!`.
Document each with a doc comment (Juniper surfaces it in the schema).

**Verify**: `rtk cargo build --workspace` → 0; `rtk cargo clippy --workspace --all-targets`
→ zero warnings.

### Step 6: Tests

- `parallax-storage`: unit tests against the in-memory store — totals over a seeded window,
  bucket math for each `SignalKind`, `service_summaries` ordering + error counts,
  `span_red_series` percentile math on a known distribution (exact in memory).
- `parallax-api`: resolver test executing the new queries against the memory store (follow
  the crate's existing resolver-test pattern — look for existing `#[tokio::test]` GraphQL
  executions in `parallax-api` and model on them). Assert ns-string round-trip and that
  counts survive > i32::MAX (String path).

**Verify**: `rtk cargo nextest run` → all pass; `rtk cargo fmt --all` → clean;
clippy zero warnings.

## Test plan

Covered in Step 6. Minimum new cases: empty window (all zeros, no error), single-bucket
window, service filter on/off, error-rate division-by-zero guard, percentile on 1-element
set.

## Done criteria

- [ ] `rtk cargo build --workspace` exit 0; clippy zero warnings; fmt clean
- [ ] `rtk cargo nextest run` → all pass incl. new tests
- [ ] GraphQL schema exposes `overview`, `signalCountSeries`, `serviceList`, `serviceRed`
      (verify: run serve, POST an introspection or the queries themselves; document output)
- [ ] Counts returned as String (no i32 saturation) — test asserts
- [ ] Quantile support verified against a live engine, or percentiles explicitly None +
      limitation reported
- [ ] `plans/README.md` row updated

## STOP conditions

- The `TelemetryStore` trait has diverged (methods renamed/moved) since `ad9115d`.
- GreptimeDB rejects `date_bin` on `opentelemetry_traces.timestamp` (type mismatch) — report
  the actual column type instead of casting blindly.
- Any change would touch ingest/write paths.
- Adding the trait methods breaks an implementor you didn't know about (search
  `impl TelemetryStore` across the workspace first; expected: greptime, memory).

## Maintenance notes

- Plans 013 (Overview UI) and 015 (Services UI) consume exactly these query names/shapes;
  coordinate any rename with those plans before execution.
- `metric_point_count`/`METRIC_POINTS` may be a documented v1 gap; when per-metric table
  scanning gets an index/rollup later, fill it in.
- If sort-by-duration lands in `traces_search` (plan 010), consider reusing its quantile SQL
  helper here.
