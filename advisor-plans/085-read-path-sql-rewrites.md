# Plan 085: Push read-path work into the engine — window the unbounded scans, aggregate histograms/edges in SQL, collapse redundant round-trips, fix the uncast timestamp read

> **Executor instructions**: Follow this plan step by step. Run every
> verification command and confirm the expected result before moving to the
> next step. If anything in the "STOP conditions" section occurs, stop and
> report — do not improvise. When done, update the status row for this plan
> in `advisor-plans/README.md` — unless a reviewer dispatched you and told you
> they maintain the index.
>
> **Drift check (run first)**: `git diff --stat df81d86..HEAD -- crates/parallax-storage/src/greptime.rs crates/parallax-storage/src/memory.rs crates/parallax-storage/src/adapter.rs`
> Plans 070/074/075/084 legitimately edit `greptime.rs` first. The excerpts
> below were taken from the working tree at `df81d86` (which already carried
> uncommitted greptime.rs edits). Verify each function still matches in shape;
> if 074's golden-SQL builders exist, edit the builders and their goldens.

## Status

- **Priority**: P1
- **Effort**: L
- **Risk**: MED (several queries change result semantics at window boundaries — each is called out and must be decided, not slipped in)
- **Depends on**: 074 (golden-SQL + conformance net), 075 (traces_search/attribute-compare/runtime-snapshot/table-cache land first — same file, overlapping regions), 084 (service_name log column exists → this plan's log queries build on it)
- **Category**: perf
- **Planned at**: commit `df81d86`, 2026-07-10

## Why this matters

After Plan 075 fixes the four known hot-query problems, a second ring of read
queries still scans whole tables regardless of what the user asked for, ships
raw rows to Rust for aggregation the engine can do, or issues 3-6 serial HTTP
round-trips for one logical answer. These costs grow with retention (7-day TTL
default), not with the view. One is a correctness landmine: `select_spans`
reads the raw `TIMESTAMP(9)` column without a cast, and the row decoder
silently turns any non-u64 JSON into `0`.

## Current state

All in `crates/parallax-storage/src/greptime.rs` (working-tree line numbers at
planning time). Every `self.sql(...)`/`sql_lenient(...)` call is one HTTP
round-trip to the embedded GreptimeDB; there is no concurrency inside any
single method (`futures-util` is not a dependency of parallax-storage; check
`crates/parallax-storage/Cargo.toml` — if Plan 075 added it, reuse it,
otherwise prefer `tokio::join!`/`try_join!`).

Engine facts (verified against docs + live engine, 2026-07-10 audit):
- `information_schema` queries are in-memory catalog lookups (~9 ms over 360
  tables live) — cheap, but each is still a full HTTP round-trip.
- `approx_percentile_cont(col, q)` (positional), `date_bin`, `ROW_NUMBER()
  OVER`, CTEs, and GreptimeDB RANGE queries (`agg(x) RANGE '5m' ... ALIGN`)
  all work on the shipped engine (live-verified on 1.1.0).

1. **`service_names` (`:1252-1271`)** — 3-way `UNION` over WHOLE tables, no
   time bound, per-row `json_get_string` on every log row:

```rust
        let rows = self
            .sql_lenient(&format!(
                r#"SELECT DISTINCT "service_name" AS "svc" FROM opentelemetry_traces
                   UNION SELECT DISTINCT
                          {} AS "svc"
                          FROM opentelemetry_logs
                   UNION SELECT DISTINCT "service" AS "svc" FROM run_metric_points
                   ORDER BY "svc""#,
                resource_json_get(semconv::SERVICE_NAME)
            ))
```

2. **`observed_runs` (`:1830-1917`)** — three serial GROUP-BY scans with no
   time bound, including `opentelemetry_logs l JOIN opentelemetry_traces s ON
   s."trace_id" = l."trace_id"` (`:1866-1875`) — an unbounded join of the two
   biggest tables — and a third full logs scan (`:1884-1891`). The join and
   log-scan fallbacks run even when the native run-id column produced rows.

3. **`spans_by_run` (`:1093-1150`)** — second fallback `select_spans` with
   `"trace_id" IN (SELECT DISTINCT "trace_id" FROM opentelemetry_logs WHERE
   <run_col> = '…')` (`:1118-1124`): inner scan unbounded, runs whenever the
   first query returned fewer than `limit` spans — including the common case
   where the run legitimately has few spans.

4. **`histogram_quantile` (`:1701-1753`)** — no GROUP BY, no LIMIT: pulls every
   `(ts, le, cumulative)` bucket row in the range and merges client-side:

```rust
                r#"SELECT CAST("greptime_timestamp" AS BIGINT) AS "ts_ms",
                          CAST("le" AS DOUBLE) AS "le", "greptime_value" AS "cumulative"
                   FROM "{}"
                   WHERE "greptime_timestamp" >= {} AND "greptime_timestamp" <= {}{service_clause}
                   ORDER BY "greptime_timestamp" ASC"#,
```

   then `:1740-1744` does `+= cumulative` per (window, le). NOTE the math:
   `greptime_value` on a `_bucket` row is a CUMULATIVE counter sample; summing
   samples across all scrape points inside a window over-weights buckets —
   it is only proportional if the distribution is stationary. The in-SQL
   rewrite must pick an explicit semantic (see Step 4).

5. **`overview_totals` (`:1273-1343`)** — 3 serial queries; the third re-scans
   the traces table again (`UNION ALL` of windowed traces + logs) although
   query 1 already computed `COUNT(DISTINCT "service_name")` on the same
   window.

6. **`service_map` (`:2255-2341`)** — fetches one row PER edge instance
   (`:2289-2307`), then groups and computes p50/p95 in Rust via
   `duration_quantile_ms` (sorts a Vec per edge). `service_summaries`
   (`:1410-1443`) already shows the server-side pattern:
   `approx_percentile_cont("duration_nano", 0.95)`.

7. **`span_field_stats` (`:2151-2253`)** — 4 serial round-trips per call:
   `span_field_columns()` (info_schema), totals, `COUNT(DISTINCT)` over the
   sample subquery, top-values over THE SAME sample subquery (`sample_sql`
   built at `:2195-2201`, embedded twice).

8. **`metric_label_values` (`:1210-1250`)** / **`metric_series_grouped`
   (`:2387-2452`)** — chain `metric_table_for_name` + `metric_labels` (which
   internally re-resolves the table) + the data query: 4-6 serial round-trips,
   the table resolved 2-3 times per logical call.

9. **`select_spans` (`:540-585`)** — reads `timestamp`/`duration_nano` WITHOUT
   a cast (`cols.u128("timestamp", row)` at `:564`), while every other time
   read in the file casts (`select_logs:598`: `CAST("timestamp" AS BIGINT)`).
   `u128_at` (`:805-810`) returns **0** for any JSON value that is not a u64
   (string, float, negative). If the engine's JSON encoding of a raw
   `TIMESTAMP(9)` ever differs from a plain integer, every span timestamp
   silently becomes 0.

10. **`select_logs` (`:590-614`)** — projects
    `json_to_string("log_attributes")` / `json_to_string("resource_attributes")`,
    then `json_at` (`:878-886`) re-parses each string per row. Serialize→parse
    round-trip per row with no functional gain IF the bare JSON column comes
    back as structured JSON over HTTP (verify — Step 8).

11. **`discover_metric_names` (`:2691-2757`)** — appends an unbounded
    `SELECT DISTINCT "name" FROM run_metric_points` to every call; called by
    `metric_names()` which `runtime_snapshot` hits on every render.

Conventions: strict clippy, cargo-nextest, `rtk` prefix, Conventional Commits
+ DCO + Claude trailer, direct on `main`. The memory adapter
(`crates/parallax-storage/src/memory.rs`) defines contract semantics: any
deliberate result-semantics change must be mirrored there or STOPped on.

## Commands you will need

| Purpose | Command | Expected on success |
|---------|---------|---------------------|
| Storage tests | `rtk cargo nextest run -p parallax-storage` | all pass |
| Full suite | `rtk cargo nextest run --workspace` | all pass |
| Lint | `rtk cargo clippy --workspace --all-targets` | zero warnings |
| Conformance (gated, real engine) | `rtk cargo nextest run -p parallax-server m6_conformance --run-ignored only` | all pass |
| Live EXPLAIN (serve running) | `curl -s -XPOST 'http://127.0.0.1:24000/v1/sql?db=public' -d "sql=EXPLAIN ANALYZE <q>"` | plan JSON |

## Scope

**In scope** (the only files you should modify):
- `crates/parallax-storage/src/greptime.rs`
- `crates/parallax-storage/src/memory.rs` (ONLY where a semantic decision says "align both")
- `crates/parallax-storage/src/adapter.rs` (ONLY if a method signature gains a range parameter)
- `crates/parallax-api/src/lib.rs` (ONLY the call sites of methods whose signature gains a range parameter — mechanical threading)
- Golden-SQL tests from Plan 074 (update deliberately)
- `advisor-plans/README.md` (status row)

**Out of scope**:
- `traces_search` internals (Plan 075 owns them; its executor was separately
  told to window the participation subquery — see the amendment note in 075).
- Resolver-level memoization/batching — Plan 086.
- Any new dependency beyond `futures-util` (and only if 075 didn't add it).

## Git workflow

Direct on `main`; Conventional Commits + `git commit -s` +
`Co-authored-by: Claude <noreply@anthropic.com>`. One commit per step.

## Steps

### Step 1: Window `service_names`, `discover_metric_names`, and the `observed_runs`/`spans_by_run` fallbacks

Semantic decision (record in code comments): pickers and run lists show the
RETAINED-AND-RECENT world, not all-time. Add a `range: RangeInclusive<u128>`
parameter to `service_names` and `observed_runs` in `adapter.rs`, thread it
from the resolvers in `lib.rs` (they already carry from/to on the calling
queries — where a caller genuinely has no window, pass
`now - 24h ..= u128::MAX` and note it). Then:

- `service_names`: add `"timestamp" >= … AND <= …` to the traces and logs
  branches and `"ts" >= … AND <= …` to `run_metric_points`; switch `UNION` to
  `UNION ALL` + one outer `SELECT DISTINCT` (single dedup instead of three).
- `discover_metric_names`: bound the `run_metric_points` DISTINCT scan to the
  same range param (thread through `metric_names`).
- `observed_runs`: apply the window to all three queries, AND skip the two
  fallback queries entirely when the native-column query returned ≥ limit rows.
- `spans_by_run`: bound the inner logs subquery with the same window the
  caller provides (add the range param), and only run the fallback when the
  native-column query errored with missing-column OR returned zero rows (not
  merely fewer than `limit`).

Memory adapter: mirror the same windowing so conformance stays green (its
scans are in-memory filters — add the same range checks).

**Verify**: `rtk cargo nextest run --workspace` → all pass (update goldens +
any memory-adapter tests deliberately; the diff is the review artifact).

### Step 2: Single-pass `overview_totals` + concurrent queries

Merge the third query into the second: compute the log-side distinct services
in the logs query (`COUNT(DISTINCT <service expr>)`), and compute the union
count in Rust — NOTE the union of two distinct sets is not the sum; keep SQL
correctness by having query 3 remain the only cross-table distinct **OR**
return the per-source distinct lists bounded (`SELECT DISTINCT service …`)
and union in Rust (services are low-cardinality; ≤ a few hundred strings).
Choose the Rust-union form — it removes the traces re-scan. Then run the two
remaining queries concurrently with `tokio::try_join!`.

**Verify**: storage tests pass; conformance overview scenario returns
identical numbers on both adapters (memory is the oracle).

### Step 3: `service_map` percentiles in SQL

Replace the per-edge-instance fetch with a grouped aggregate, matching the
`service_summaries` precedent (`:1418-1428`):

```sql
SELECT "parent"."service_name", "child"."service_name",
       COUNT(*), SUM(CASE WHEN "child"."span_status_code" = 'STATUS_CODE_ERROR' THEN 1 ELSE 0 END),
       approx_percentile_cont("child"."duration_nano", 0.50),
       approx_percentile_cont("child"."duration_nano", 0.95)
FROM … JOIN … WHERE …(same predicates)…
GROUP BY "parent"."service_name", "child"."service_name"
```

Keep the trace-id cap subquery as-is. Document: p50/p95 become approximate
(engine t-digest) instead of exact client-side quantiles — same tradeoff the
service summaries already made.

**Verify**: storage tests pass; gated conformance service-map scenario ranks
edges identically (counts exact; quantiles approximate — if a conformance
assertion pins exact quantile values for greptime, STOP and report).

### Step 4: Aggregate `histogram_quantile` server-side with explicit math

Two changes in one:

- SQL: `SELECT CAST(date_bin(INTERVAL '<step>s', "greptime_timestamp") AS BIGINT) AS b,
  CAST("le" AS DOUBLE) AS le, MAX("greptime_value") AS cum FROM "<t>_bucket"
  WHERE … GROUP BY b, le ORDER BY b` — `MAX` per (window, le) takes the
  latest cumulative sample in the window per bucket (cumulative counters are
  monotonic per series), which fixes the current `+=` over-counting AND cuts
  returned rows to `windows × buckets`.
- CAVEAT to verify (STOP if false): if MULTIPLE series (label sets) write the
  same `le` in one window, `MAX` picks one series instead of summing across
  series. Check live whether the bucket table carries extra tag columns beyond
  `le`/`service_name` for a representative histogram
  (`SHOW CREATE TABLE <t>_bucket`). If multi-series is real, use
  `SUM(max_per_series)` via a two-level aggregate:
  `SELECT b, le, SUM(cum) FROM (SELECT date_bin(...) b, le, <tags>, MAX(greptime_value) cum GROUP BY b, le, <tags>) GROUP BY b, le`.
- Share one fetch across the three quantiles: add
  `histogram_quantiles(name, service, range, step, qs: &[f64]) -> Vec<(f64, Vec<SeriesPoint>)>`
  to the trait with a default impl looping `histogram_quantile`; implement it
  natively in greptime (one fetch, N interpolations) and memory. Plan 086
  switches the API to it.
- Mirror the merge-math fix in `memory.rs` if its histogram path sums samples
  the same way (check `histogram_quantile` there; align both or STOP).

**Verify**: storage + conformance pass; add one unit test on
`quantile_from_cumulative` inputs proving the windowed merge uses
latest-cumulative (construct two scrape points in one window; expected
quantile from the second sample only).

### Step 5: One-pass `span_field_stats`

Fold the totals + distinct + top-values into two queries max: keep the totals
query; compute distinct-count and top-values from ONE sample scan using a CTE:

```sql
WITH "field_sample" AS (<sample_sql>)
SELECT * FROM (
  SELECT "value", COUNT(*) AS "count" FROM "field_sample" GROUP BY "value"
  ORDER BY "count" DESC, "value" ASC LIMIT <FIELD_TOP_VALUES_CAP>
)
```

plus `SELECT COUNT(DISTINCT "value") FROM "field_sample"` — if the engine
allows two statements referencing one CTE only within one statement, instead
return `APPROX_DISTINCT("value")` as an extra column of the totals query over
the same windowed rows (verify `approx_distinct` exists live; DataFusion has
it). Choose whichever verifies; the requirement is: ≤ 2 data queries + reuse
of the already-fetched `span_field_columns` (thread the column in from the
caller if it already holds the list — `span_field_keys` does).

**Verify**: storage tests pass; Discover field panel values unchanged on the
memory adapter conformance scenario.

### Step 6: Thread table resolution through the metric call chains

Add a private `resolved_metric_table(&self, name) -> Option<(table, labels)>`
that resolves the table once (via the Plan-075 cache) and lists label columns
once; rewrite `metric_labels`, `metric_label_values`, and
`metric_series_grouped` to share it so one logical call = table resolution +
1 data query (2-3 round-trips → 1-2). Do not change public signatures.

**Verify**: storage tests pass.

### Step 7: Cast the span time columns

In `select_spans` change the projection to explicit columns? NO — `SELECT *`
stays (auto-widened attribute columns are the point; recorded decision). The
uncast read fix therefore goes in the decoder: make `u128_at`/`ColumnIndex::u128`
fall back to parsing `Value::String` as u128 and `Value::Number(f64)` via
`as_f64() as u128` before defaulting to 0, AND add a debug_assert/tracing warn
on the fallback path so a format change is visible. Add a unit test feeding
string/float/int JSON into `u128_at`.

**Verify**: `rtk cargo nextest run -p parallax-storage` → new decoder tests pass.

### Step 8: Drop the `json_to_string` round-trip in `select_logs` (verify-first)

Live-check with the running engine what
`SELECT "log_attributes" FROM opentelemetry_logs LIMIT 1` returns over HTTP
(structured JSON object vs string). If structured (or a JSON string that
`json_at` already handles via its `Some(other)` branch), project the bare
columns and delete the `json_to_string` wrappers. If the raw column comes back
in a shape `json_at` cannot decode, leave as-is and record the finding with
the observed payload shape in the commit message.

**Verify**: gated conformance logs scenario passes against the real engine;
log attributes still render (non-empty `attributes` in a `logs_search` result).

### Step 9: Full gates

**Verify**: `rtk cargo fmt --all`; clippy zero warnings;
`rtk cargo nextest run --workspace` all pass; gated conformance all pass.

## Test plan

- Golden-SQL updates for every rewritten query (074's net) — each diff is the
  review artifact.
- New unit tests: windowed-merge histogram math (Step 4), `u128_at` decoder
  fallbacks (Step 7), body of `service_names` UNION ALL shape if goldens don't
  already pin it.
- Conformance suite on both adapters after Steps 1-5 (memory adapter is the
  semantic oracle; every deliberate divergence must be listed in the commit
  message).

## Done criteria

Machine-checkable. ALL must hold:

- [ ] `grep -n "FROM opentelemetry_logs" crates/parallax-storage/src/greptime.rs` → every occurrence inside `service_names`/`observed_runs`/`spans_by_run` has a `"timestamp"` bound in the same statement
- [ ] `grep -n "ORDER BY \"greptime_timestamp\" ASC" crates/parallax-storage/src/greptime.rs` → 0 matches inside `histogram_quantile` (replaced by GROUP BY form)
- [ ] `grep -n "duration_quantile_ms" crates/parallax-storage/src/greptime.rs` → not called from `service_map` (function may remain for other callers or be deleted)
- [ ] `grep -c "metric_table_for_name" crates/parallax-storage/src/greptime.rs` → reduced vs baseline (single resolution helper in the metric chains)
- [ ] `rtk cargo nextest run --workspace` exits 0; clippy zero warnings
- [ ] `git status` clean outside in-scope list
- [ ] `advisor-plans/README.md` status row updated

## STOP conditions

Stop and report back (do not improvise) if:

- The memory adapter computes UNwindowed results for any query Step 1 windows
  and a test pins that — contract question for the operator (same class as
  Plan 075's STOP).
- Step 4's bucket-table schema shows per-series tags and the two-level
  aggregate form is rejected by the engine.
- Any conformance assertion pins exact quantiles that Step 3/4 makes
  approximate.
- Threading the new range parameters through `lib.rs` touches more than ~15
  call sites (signature blast radius bigger than planned — report the list).
- Plan 075 has not landed and its regions conflict with yours (traces_search /
  attribute_compare / runtime_snapshot / metric-table cache) — coordinate via
  the index, do not merge both by hand.

## Maintenance notes

- Windowed `service_names`/`observed_runs` changes what an idle-service picker
  shows — release note it. The 24h default window for windowless callers is a
  product knob to revisit.
- Step 4's histogram math is now "latest cumulative per window" — if rate-style
  histogram deltas land later (PromQL-like `increase()`), revisit both adapters
  together.
- GreptimeDB RANGE queries (`agg RANGE '…' ALIGN '…'`) are the engine-native
  form of every `date_bin + GROUP BY` in this file — a future sweep could
  migrate wholesale; deferred because `date_bin` is verified-working and the
  migration is cosmetic until measured otherwise (see Plan 090's measurements).
- After this plan + 075, the remaining unbounded reads are `spans_by_trace`/
  `logs_by_trace` (single-trace anchored, capped by trace size) — Plan 086
  bounds them at the API layer.
