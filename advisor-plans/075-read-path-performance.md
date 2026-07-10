# Plan 075: Bound and parallelize the hot read queries — traces_search window, attribute-compare fan-out, runtime-snapshot N+1, metric-table cache

> **Executor instructions**: Follow this plan step by step. Run every
> verification command and confirm the expected result before moving to the
> next step. If anything in the "STOP conditions" section occurs, stop and
> report — do not improvise. When done, update the status row for this plan
> in `advisor-plans/README.md` — unless a reviewer dispatched you and told you
> they maintain the index.
>
> **Drift check (run first)**: `git diff --stat dbaba3c..HEAD -- crates/parallax-storage/src/greptime.rs`
> Plans 070 and 074 legitimately edit this file first — verify the specific
> functions below still match in shape (074's golden-SQL extraction may have
> moved SQL into `*_sql` builder fns; if so, edit the builders instead and
> update their golden tests as part of this plan).

## Status

- **Priority**: P2
- **Effort**: M
- **Risk**: MED (windowing the aggregate changes result semantics for traces
  straddling the window edge — deliberate, documented below)
- **Depends on**: 074 (golden-SQL tests exist so these changes are visible
  diffs, and the conformance suite guards semantics). 070 first for rebase
  hygiene.
- **Category**: perf
- **Planned at**: commit `dbaba3c`, 2026-07-10

## Why this matters

Four read-path costs grow with retained data volume, not with what the user
asked for:

1. `traces_search` — the default traces page — joins against a subquery that
   aggregates the **entire** `opentelemetry_traces` table (`GROUP BY trace_id`
   with no time filter), and then executes the whole composed query **twice**
   (once wrapped in `COUNT(*)` for the total, once for the page). The one
   query the product exists for gets slower every day the store grows.
2. `attribute_compare` issues 2 sequential aggregate queries per candidate
   key (up to 2×N serial round-trips) inside the same traces-page request.
3. `runtime_snapshot` loops every metric name and awaits `metric_series`
   sequentially — and each `metric_series` first resolves its table via an
   `information_schema` query, so the panel costs ~2×(number of metrics)
   serial HTTP round-trips.
4. `metric_table_for_name` hits `information_schema.tables` on every metric
   read with no caching, multiplying cost across service-overview panels and
   the 5-second metric-strip poll.

## Current state

All in `crates/parallax-storage/src/greptime.rs` (line numbers at `dbaba3c`;
if Plan 074 extracted `*_sql` builders, find the same SQL there):

- `traces_search` (~`:1900-2050`): builds `scan_where` from
  `query.from_nanos`/`to_nanos` (`:1937-1948`), applies it ONLY to the `root`
  subquery (`:1984-1985 WHERE {scan_where}{participation}`); the joined `agg`
  subquery is unwindowed:

  ```rust
  // greptime.rs:1995-2000
  JOIN (
    SELECT "trace_id", COUNT(*) AS "span_count",
           MAX(CASE WHEN "span_status_code" = 'STATUS_CODE_ERROR' THEN 1 ELSE 0 END)
           AS "has_error"
    FROM opentelemetry_traces GROUP BY "trace_id"
  ) AS "agg" ON "root"."trace_id" = "agg"."trace_id"
  ```

  and the double execution:

  ```rust
  // greptime.rs:2004-2012
  let total_rows = self
      .sql_lenient(&format!(r#"SELECT COUNT(*) AS "total" FROM ({listed})"#))
      .await?;
  let roots = self
      .sql_lenient(&format!(
          r#"SELECT * FROM ({listed}) ORDER BY {order} LIMIT {} OFFSET {}"#,
          query.limit, query.offset,
      ))
      .await?;
  ```

- `attribute_compare` (`:2060-2098`): `for key in candidate_keys { ...
  span_attribute_counts(&key, &selected, ...).await?; ...
  span_attribute_counts(&key, &baseline, ...).await?; }` — strictly serial.

- `runtime_snapshot` (`:2455-2488`): `for metric in self.metric_names().await?
  { ... self.metric_series(&metric, ...).await? ... }` — strictly serial, and
  filters to `runtime_metric_family(&metric)` AFTER iterating (the filter is
  cheap; the awaits are not).

- `metric_table_for_name` (`:495-522`): per-call
  `SELECT "table_name" FROM information_schema.tables WHERE ... IN (...)`.
  Callers: `metric_series`, `metric_labels`, `histogram_quantile`,
  `metric_series_grouped`, `histogram_count_series` (after Plan 070), plus
  per-candidate loops in `parallax-api` (`first_nonempty_points`,
  `request_rate` at `lib.rs:1417/1457`).

- Concurrency helpers: the crate does not currently import `futures`; check
  `crates/parallax-storage/Cargo.toml` — if `futures-core`/`futures-util` is
  absent, prefer `tokio::try_join!`-style manual joins or add `futures-util`
  (workspace already ships `futures-core`; adding `futures-util` to the
  workspace deps is acceptable and must go through `[workspace.dependencies]`
  per the existing pattern in the root `Cargo.toml`).

- `GreptimeStore` fields include `AtomicBool`s for one-time deviations
  (`greptime.rs:~25-33`) — a precedent for interior-mutable state on the
  store; the cache in Step 4 follows that precedent (use
  `tokio::sync::RwLock<HashMap<...>>` or a `Mutex` — no new crate for this).

## Commands you will need

| Purpose | Command | Expected on success |
|---------|---------|---------------------|
| Storage tests | `rtk cargo nextest run -p parallax-storage` | all pass |
| Full suite | `rtk cargo nextest run --workspace` | all pass |
| Real-engine conformance (gated) | `rtk cargo nextest run -p parallax-server m6_conformance --run-ignored only` | all pass |
| Lint | `rtk cargo clippy --workspace --all-targets` | zero warnings |

## Scope

**In scope** (the only files you should modify):
- `crates/parallax-storage/src/greptime.rs`
- `crates/parallax-storage/Cargo.toml` + root `Cargo.toml` (only if adding
  `futures-util` to workspace deps)
- Golden-SQL tests from Plan 074 (update expected strings deliberately)
- `advisor-plans/README.md` (status row)

**Out of scope** (do NOT touch, even though they look related):
- `crates/parallax-storage/src/memory.rs` — its `traces_search` semantics
  define the contract; if windowing the aggregate makes greptime diverge from
  memory, that divergence must be resolved by ALIGNING BOTH deliberately —
  see Step 1's semantic decision and STOP conditions.
- `parallax-api` resolvers — caller-side candidate loops are a separate,
  smaller win; not here.
- UI polling cadence (metric-strip 5s) — client-side, separate.
- Worker/ingest path — Plan 076.

## Git workflow

- Work directly on `main` (repo rule — `BRANCHING.md`).
- Conventional Commits, DCO signoff (`git commit -s`), trailer
  `Co-authored-by: Claude <noreply@anthropic.com>`. E.g.
  `perf(storage): window traces_search aggregate and fold count into page query`.

## Steps

### Step 1: Window the `traces_search` aggregate

Semantic decision (document it in the code comment): the `agg` subquery gets
the SAME `{scan_where}` time bounds as `root`. Consequence: `span_count` and
`has_error` are computed over the queried window, not the trace's lifetime —
a trace straddling the window boundary reports its in-window span count. This
matches what the UI presents (a windowed view) and what `memory.rs` computes
IF memory also windows its aggregate — CHECK `memory.rs`'s `traces_search`
first: if memory aggregates unwindowed, STOP and report the contract
question instead of choosing silently.

Implementation: inject `WHERE {scan_where}` into the `agg` subquery
(`greptime.rs:1999`), reusing the same string.

Also window the `participation` subquery in the same function (2026-07-10
audit addendum): the `service` filter builds
`"trace_id" IN (SELECT "trace_id" FROM opentelemetry_traces WHERE "service_name" = '…')`
with NO time bound (`greptime.rs:~1929-1935`), so a windowed search still
scans all-time for the service's trace ids. Apply the same `{scan_where}`
inside that subquery. Same semantic caveat as the agg windowing; same golden
test covers it.

Update the Plan-074 golden test for `traces_search_sql` to the new expected
string in the same commit.

**Verify**: `rtk cargo nextest run -p parallax-storage` → golden + unit tests
pass.

### Step 2: Single-pass total

Replace the two executions with one: either
`SELECT *, COUNT(*) OVER () AS "total" FROM ({listed}) ORDER BY {order} LIMIT ... OFFSET ...`
(window function — verify GreptimeDB supports `COUNT(*) OVER ()`; it is
DataFusion-based, which does), or keep two statements but share a CTE. Prefer
the window-function form; parse `total` from any returned row; when zero rows
return (offset beyond end), fall back to one count query (rare path).

**Verify**: golden test updated; storage tests pass; run the gated
conformance trace-search scenario against the real engine:
`rtk cargo nextest run -p parallax-server m6_conformance --run-ignored only`
→ passes (this proves the window function works on the shipped engine
version).

### Step 3: Parallelize the per-key and per-metric fan-outs

- `attribute_compare`: run each key's selected+baseline pair concurrently —
  build futures and `futures_util::future::try_join_all` (or chunked joins of
  ~8 to bound concurrent engine load; note the choice). Preserve output order
  determinism: collect per-key results, then iterate keys in the original
  order when building `rows` (the final sort at `:2089` re-orders anyway, but
  tie-breaking depends on stable input order — keep it stable).
- `runtime_snapshot`: filter names by `runtime_metric_family` FIRST, then
  fetch all series concurrently with the same bounded-join helper, then sort
  as today (`:2486` already sorts, so output is deterministic).

**Verify**: `rtk cargo nextest run -p parallax-storage` → the memory-side
conformance scenarios for attribute-compare still pass (they assert ranking,
which must be unchanged); clippy zero warnings.

### Step 4: Cache metric-name → table resolution

Add to `GreptimeStore`:

```rust
metric_table_cache: tokio::sync::RwLock<std::collections::HashMap<(String, Option<&'static str>), Option<String>>>,
```

(key = (name, suffix); suffix values used today are a small static set —
if the actual suffix parameter is `Option<&str>` with dynamic strings, key by
`(String, Option<String>)`.)

Semantics: `metric_table_for_name` checks the cache first; on `Some(table)`
hit, return it. Cache POSITIVE results indefinitely (a created table never
changes name). For NEGATIVE results (table doesn't exist yet — metrics
auto-create on first ingest), do NOT cache, so a metric that appears later is
found on the next call. This avoids TTL machinery entirely.

**Verify**: add a unit test if the resolution logic is separable
(candidate-key construction is pure — `metric_table_candidates` already
tested per Plan 070); otherwise assert via the gated real-engine test that
repeated `metric_series` calls still return data. Run
`rtk cargo nextest run --workspace` → all pass.

### Step 5: Full gates

**Verify**: `rtk cargo fmt --all`;
`rtk cargo clippy --workspace --all-targets` → zero warnings;
`rtk cargo nextest run --workspace` → all pass; gated conformance suite passes
on the real engine.

## Test plan

- Golden-SQL tests (from 074) updated for windowed `agg` + single-pass total —
  the diff IS the review artifact.
- Conformance scenarios (074) re-run on both adapters — semantic guard.
- New unit test: negative-result-not-cached behavior of the table cache if
  testable without a live engine (inject via the builder seam); otherwise
  covered by the gated test.
- No benchmark gate in this plan (the repo's benchmark discipline is the
  four-build matrix under `bench/` — out of scope here), but note observed
  timings in the commit message if you run the real engine locally.

## Done criteria

Machine-checkable. ALL must hold:

- [ ] `grep -n "FROM opentelemetry_traces GROUP BY" crates/parallax-storage/src/greptime.rs` → the agg subquery now contains a `WHERE` (inspect the golden test string: it must show the window in both subqueries)
- [ ] `grep -cn "sql_lenient" crates/parallax-storage/src/greptime.rs` in `traces_search` region → the count+page double execution is gone (one execution on the happy path)
- [ ] `grep -n "try_join" crates/parallax-storage/src/greptime.rs` → ≥2 matches (attribute_compare, runtime_snapshot)
- [ ] `grep -n "metric_table_cache" crates/parallax-storage/src/greptime.rs` → ≥2 matches
- [ ] `rtk cargo nextest run --workspace` exits 0
- [ ] `rtk cargo clippy --workspace --all-targets` → zero warnings
- [ ] `git status` shows no modified files outside the in-scope list
- [ ] `advisor-plans/README.md` status row updated

## STOP conditions

Stop and report back (do not improvise) if:

- `memory.rs`'s `traces_search` aggregates UNwindowed (contract divergence —
  the operator must pick the windowed or lifetime semantics for both).
- `COUNT(*) OVER ()` is rejected by the shipped GreptimeDB (gated test
  fails) — fall back to the shared-CTE two-statement form ONLY if a CTE
  executes once server-side; if that's unverifiable, report.
- Plan 074's builders don't exist AND `traces_search` has drifted beyond the
  excerpts.
- Bounded-join concurrency causes real-engine errors (too many concurrent
  queries) — reduce the chunk size once; if still failing, report.

## Maintenance notes

- The windowed-aggregate semantics change is user-visible for boundary-
  straddling traces; release notes / commit message must say so.
- The positive-only cache assumes tables are never renamed/dropped while the
  server runs; `parallax prune`-style future work that DROPs metric tables
  must invalidate this cache.
- Deferred read-path items recorded in the index, not planned: per-row
  `.cloned()` in `sql_with_schema`, `SELECT *` on `select_spans` (entangled
  with schema auto-widening), UI metric-strip poll delta-fetching, single
  ingest worker pipelining.
