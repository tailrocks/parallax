# PromQL and Metrics Query — The Capability Gap, Re-Verified

<!-- markdownlint-disable MD013 -->

Status: pass 44 (capability/maturity) + pass 72 (PromQL **speed** characterization, Run 44)
+ pass 98 (**re-verified live, no drift, Run 62**) + **Run 105 (re-verified, no drift: GT
PromQL ~675 ms vs GT SQL ~120 ms vs CH SQL ~55 ms on `avg by(service)` over a 60-min/60-s
range — ~5.6× GT-SQL, ordering CH SQL > GT SQL > GT PromQL confirmed; sharper caveat: a WIDE
PromQL range over 40k series is OVER the 300 ms gate, so drive hot panels with SQL/Flow, not
wide PromQL)**. **Run 403 (2026-07-18) — CH TimeSeries surface DRIFT CORRECTION** on pins
`v1.1.3` / `26.6.1.1193` / head `26.7.1.1097`: outer `SELECT * FROM TimeSeries` is still
Code 48, but that is **by design** (facade has no storage); **SQL `INSERT` into the outer
table works**, splits into three MergeTree-family inners, and **`prometheusQuery` /
`prometheusQueryRange` return real series** when called with the 3-arg form
`(ts_table, promql, eval_time)`. Prior run-log lines that treated "Code 48 SELECT" as
"PromQL path broken" are **wrong** — the intended query path is table-function-only and
**works**. The GA-vs-experimental maturity gap still holds; capability is no longer vapor.

The PromQL planning path (a GreptimeDB system-lead) **and** a required re-verification of
the verdict's load-bearing claim that "ClickHouse has no PromQL." Metrics/PromQL
nativeness is the verdict's #1 GreptimeDB advantage, so a version-drift here is
decision-critical. Source + live (Runs 23, 24, 44, 62, **403**).

**Re-verification (Run 62, v1.0.2 / 26.5.1.882 — historical):** GreptimeDB PromQL
still GA + zero-setup. ClickHouse: `allow_experimental_time_series_table=0` off by default;
`TimeSeries` creatable with flag. **Superseded on INSERT:** Run 24 claimed INSERT
NOT_IMPLEMENTED; **Run 403** on 26.6.1 + 26.7.1 shows SQL INSERT into the outer table
**succeeds** (docs now document it). Outer SELECT remains NOT_IMPLEMENTED.

**Headline correction:** the old "ClickHouse has **no** PromQL, needs an external
PromQL→SQL layer" is **outdated as of ClickHouse 26.x**. ClickHouse now ships
**experimental** native PromQL. The gap is no longer *present vs absent* — it is
**GA-native-ergonomic (GreptimeDB) vs experimental-off-by-default-setup-heavy
(ClickHouse)**. This narrows, but does not flip, the metrics verdict.

Pins: GreptimeDB **`v1.1.3`** / CH **`26.6.1.1193`** (Run 183 re-pin 2026-07-17); historical Run 105 numbers were on `v1.0.2` / `26.5.1.882`.

## GreptimeDB — native PromQL planner, GA and default-on

PromQL is a first-class query path (`src/promql` crate):

- A PromQL expression is parsed and lowered into **custom DataFusion logical nodes**,
  then `PromExtensionPlanner` (a DataFusion `ExtensionPlanner`,
  `extension_plan/planner.rs`) maps each to a physical `ExecutionPlan`:
  **`SeriesNormalize`** (sort/dedup a series), **`SeriesDivide`** (split by series),
  **`InstantManipulate`** / **`RangeManipulate`** (instant- and range-vector step
  alignment + lookback), **`HistogramFold`** (histogram quantiles), **`ScalarCalculate`**,
  **`Absent`**, **`EmptyMetric`**, **`UnionDistinctOn`**. PromQL functions (`prom_rate`,
  …) are DataFusion UDFs. So PromQL semantics (range vectors, `rate()` extrapolation,
  step, lookback delta) are executed *inside* the same engine as SQL — not translated.
- **Two entry points, both default-on:** the Prometheus HTTP API
  (`/v1/prometheus/api/v1/query[_range]`) and in-SQL **`TQL EVAL/EXPLAIN`**.
- **Live (Run 23):** `/v1/prometheus/api/v1/query?query=up` returned proper Prometheus
  JSON (`{"status":"success","data":{"resultType":"vector",…}}`) with **zero setup**;
  `TQL EXPLAIN rate(spans[5m])` invoked the native `prom_rate` planner (it only errored
  on a column *type* — spans isn't a float metric — proving the PromQL path is live and
  default, not absent).

## ClickHouse 26.x — experimental PromQL via the TimeSeries engine

ClickHouse **has gained** PromQL, but it is experimental and off by default:

- **`TimeSeries` table engine** (`allow_experimental_time_series_table`, default **0**)
  — a **facade** over three target tables (not a single MergeTree). Live `SHOW CREATE`
  on 26.6.1 / 26.7.1 (Run 403) matches [upstream docs](https://clickhouse.com/docs/engines/table-engines/special/time_series):
  - **SAMPLES** — `MergeTree ORDER BY (id, timestamp)` (`id UUID`, `timestamp`, `value`)
  - **TAGS** — `AggregatingMergeTree PRIMARY KEY metric_name ORDER BY (metric_name, id)`
    with `id DEFAULT reinterpretAsUUID(sipHash128(metric_name, all_tags))`, optional
    `min_time`/`max_time` `SimpleAggregateFunction`, `allow_dimensions_outside_sorting_key=1`
  - **METRICS** — `ReplacingMergeTree ORDER BY metric_family_name` (type/unit/help)
  - **Outer columns** (no storage): `metric_name`, `tags Map`, `time_series Array(Tuple(ts,value))`,
    `metric_family`, `type`, `unit`, `help` — interface for INSERT/SELECT; data lives only in targets.
- **Ingest paths:** Prometheus **remote-write** (configured port) **and** SQL
  `INSERT INTO ts_table (metric_name, tags, time_series[, metric_family, type, unit, help])`
  (Run 403: 2 series / 3 samples landed; inners readable via
  `` `.inner_id.samples.<uuid>` `` or `timeSeriesSamples(ts)`).
- **Query paths (intended):** `prometheusQuery(ts_table, promql, eval_time)` and
  `prometheusQueryRange(ts_table, promql, start, end, step)` plus
  `timeSeriesSamples` / `timeSeriesTags` / `timeSeriesMetrics` / `timeSeriesData`.
  **Not** ordinary `SELECT * FROM ts_table` (Code 48 by design).
- **Live Run 403 (26.6.1.1193 + head 26.7.1.1097, identical behavior):**
  - `CREATE` + flag → OK
  - outer `SELECT *` → Code 48 `SELECT is not supported by storage TimeSeries yet`
  - SQL `INSERT` → OK; samples=3, tags=2, metrics=1
  - `prometheusQuery(ts, 'up', toDateTime64('2026-07-18 00:01:00',3))` →
    `tags=[(__name__,up),(instance,h1),(job,node)], value=1`
  - `prometheusQueryRange(ts, 'up', start, end, 60)` → 3-point series
  - 2-arg `prometheusQuery(ts, 'up')` → Code 42 (needs eval_time) — easy footgun
  - default `allow_experimental_time_series_table=0` still
- **Not supported in ClickHouse Cloud** (docs as of 2026-07-17).
- Settings: `promql_database` / `promql_table` / `promql_evaluation_time`;
  `allow_experimental_time_series_aggregate_functions` (default 0).

## Side by side

| | GreptimeDB | ClickHouse 26.x |
| --- | --- | --- |
| PromQL execution | **native, GA, default-on** (custom DataFusion plan nodes) | **experimental** (`prometheusQuery[Range]` over `TimeSeries` engine) |
| Default availability | on, zero setup | **off** (`allow_experimental_time_series_table=0`) |
| Entry point | Prom HTTP API + `TQL` | table functions in SQL (`prometheusQuery…`) |
| Storage model | any metric table (metric engine) | dedicated `TimeSeries` engine table |
| Metrics ingest | OTLP + Prom remote-write native | Prom remote-write **into TimeSeries** (also experimental) |
| Maturity | GA, production | experimental, young |

## Honest re-rating (and what it changes)

- **The "no PromQL / needs an external translation layer" claim is now WRONG** and is
  corrected across the notes. ClickHouse can execute PromQL today.
- **But the metrics verdict still favors GreptimeDB**, on *maturity + ergonomics*
  rather than *capability*: GreptimeDB's PromQL is GA, default-on, zero-setup, works on
  any metric table, and pairs with the metric engine; ClickHouse's is experimental,
  off-by-default, and needs a dedicated `TimeSeries` table + remote-write pipeline. For
  a product shipping *now* on metrics, "GA + ergonomic" beats "experimental + setup,"
  but the gap is **narrowing as ClickHouse invests** — a real trend to watch, exactly
  the version-drift the method guards against.
- **Speed is still separate:** GreptimeDB's PromQL *capability* win never implied a
  *speed* win — SQL aggregation at volume still favors ClickHouse, **~2× warm** (Run 37;
  corrected from the ~10× of Run 11, which was a cold/first-run artifact — larger cold).
  PromQL is about expressing the query, not running it fastest. **Run 44 makes this
  concrete and stronger: GreptimeDB's own PromQL path is ~5× slower than its own SQL path
  at high cardinality** (40k series: PromQL `avg by(service)` ≈590 ms vs SQL ≈120 ms vs CH
  SQL ≈65 ms). Mechanism: the PromQL planner must `SeriesDivide`/`SeriesNormalize` (sort +
  partition the full scan by series) before instant/range manipulation, a **near-fixed
  ~530 ms setup** — a single-step instant eval (~535 ms) costs almost as much as a 20-step
  range (~590 ms), proving the cost is series-normalization, not per-step. SQL's streaming
  hash-agg skips it. So for raw metric-agg *latency*: **CH SQL > GT SQL > GT PromQL**; even
  GreptimeDB's fastest metric path is SQL, and PromQL is the *expressiveness* tool
  (range vectors, `rate`/`irate`, lookback), "fast enough" but never the speed leader.

## Proxy-lens nuance — native PromQL is LESS neutralized than native OTLP (Run 164)

Re-verified live (exec, no drift): GreptimeDB `/v1/prometheus/api/v1/query` still answers GA + zero-setup
(returns the Prometheus vector envelope); ClickHouse's `TimeSeries` engine is still **creatable only with
`allow_experimental_time_series_table=1`** ("created ok" with the flag) and its PromQL path runs through
that engine + `prometheusQuery([db,] ts_table, promql[, eval])` / `timeSeriesData/Metrics/Tags`
(catalog-listed; a bare `prometheusQuery('up')` errors `UNKNOWN_FUNCTION` — it's an arg/overload
mismatch needing a `TimeSeries` table, not a missing function). So pass-44/Run-23 holds.

**The proxy reframe (`platform-fit-and-alternatives.md`) does NOT neutralize this the way it neutralizes
OTLP.** Native *ingest* protocols (OTLP/Jaeger) stopped counting because Parallax-the-proxy speaks them
and **translation is cheap** (re-shape bytes into the backend's insert). But **PromQL is a query
engine**, and translating a Parallax-exposed PromQL/Grafana API into the backend's SQL is **expensive**
(PromQL's instant/range-vector semantics, `rate`/`increase` extrapolation, staleness handling, lookback
deltas — a real engine to reimplement). So a backend that **executes PromQL natively saves Parallax
from building a PromQL engine**, whereas a backend that only speaks SQL forces Parallax to either embed a
PromQL engine over it or drop PromQL/Grafana compatibility. Therefore **GreptimeDB's GA-native PromQL is
a *more durable* surviving edge than its (neutralized) native OTLP** — it's worth real weight *if*
Parallax wants first-class PromQL/Grafana compatibility. (ClickHouse's experimental TimeSeries+
`prometheusQuery` narrows it, and "experimental counts as stable / judge trajectory" says the gap is
closing — but today GreptimeDB is the GA-native PromQL option, and the proxy can't trivially erase the
difference.)

## Axis consequence

- **Capability (axis #1 enabler):** metrics/PromQL is no longer binary. GreptimeDB
  leads on GA + ergonomics; ClickHouse has closed the *can-it-at-all* gap
  experimentally. Net: still a GreptimeDB advantage for Parallax shipping today, but
  **downgraded from "decisive binary" to "maturity/ergonomics lead"** — though under the
  proxy lens (above) it is **less neutralized than native OTLP**, so it keeps more weight
  than the other ingest-nativeness edges.
- **Replaceability (Q3):** "ClickHouse can't do PromQL" is no longer a hard blocker; it
  becomes "ClickHouse's PromQL is experimental, so relying on it for a metrics product
  today is a maturity risk + setup cost," which is softer.

## Maturity, measured end-to-end (pass 45, Run 24 — **partially superseded by Run 403**)

Pass 44 established ClickHouse PromQL *exists*; pass 45 characterized usability. **Run 403
re-measures the same surface on 26.6.1 / 26.7.1:**

| Surface | Run 24 (historical) | Run 403 (26.6.1 + 26.7.1) |
| --- | --- | --- |
| Outer `INSERT` | claimed NOT_IMPLEMENTED | **works** — rows split to samples/tags/metrics |
| Outer `SELECT *` | NOT_IMPLEMENTED | still NOT_IMPLEMENTED (facade) |
| Hand-load via SQL INSERT | "no practical path" | **practical** — INSERT outer → inners populated |
| `prometheusQuery(table, promql, eval_time)` | ran but empty (bad hand-load) | **returns real vector** (`up=1`) |
| `prometheusQueryRange` | ran | **returns real range series** |
| `timeSeriesSamples/Tags/Metrics` | listed | **return real rows** |
| Experimental flag default | 0 | still **0** |
| Works without dedicated `TimeSeries` table | no | still no |
| ClickHouse Cloud | — | **not supported** (docs) |

**Still true after Run 403:**

- **No ordinary SQL SELECT** from the facade — table-function / PromQL only.
- **Experimental + off-by-default** + dedicated engine required.
- **GreptimeDB:** `/v1/prometheus/api/v1/query?query=up` still GA zero-setup
  (`{"status":"success",…}` on stable+nightly, Run 403); works on **any** metric table
  (mito or metric-engine), multi-protocol ingest, SQL + Prom HTTP + TQL.

**Narrowed:** CH is no longer "INSERT broken / can't load data without remote-write client."
SQL INSERT + `prometheusQuery` is a viable experimental lab path. Maturity gap is now
**GA multi-surface any-table (GT)** vs **experimental facade + table-functions + dedicated
engine, Cloud-unsupported (CH)** — not "query path vaporware."

## Run 404 (2026-07-18) — PromQL function completeness smoke (tiny series)

**Pins:** GT `v1.1.3`, CH `26.6.1.1193`, head `26.7.1.1097`. Same 5-point counter per
instance (`i1`: 100→340, +60/min; `i2`: 50→170, +30/min), eval at `00:04:00`.

| Expression | CH 26.6.1 | CH head 26.7.1 | GT TQL / Prom HTTP |
| --- | --- | --- | --- |
| `up` / raw selector | OK | OK | OK (any mito table) |
| `{instance="i1"}` matcher | OK (340) | OK | OK |
| `rate(…[2m])` | **1.0 / 0.5** | same | **1.0 / 0.5** (match) |
| `sum(rate(…[2m]))` | **1.5** | same | **1.5** (match) |
| `avg by (job) (rate(…))` | **0.75** | same | **0.75** (match) |
| `irate(…[2m])` | 1.0 / 0.5 | same | (not retested) |
| `delta(…[2m])` | 120 / 60 | same | — |
| `increase(…[2m])` | **Code 48 NOT_IMPLEMENTED** | **same** | **120 / 60** (`prom_increase`) |
| `prometheusQueryRange` `rate` | OK (with ≥2m lookback window) | same | Prom `query_range` OK |

**Mechanism reading:**

- Core **rate + aggregation + label matchers** on CH experimental PromQL are **real and
  numerically aligned** with GreptimeDB on this toy counter (not empty shells).
- **`increase()` is still missing** on both stable 26.6 and head 26.7 — hard Code 48.
  Many dashboards use `increase` as the friendly form of `rate * interval`; CH forces
  `rate` (or SQL). GT implements `prom_increase` on the GA path.
- Head did **not** close the `increase` gap vs 26.6 in this cycle.
- Range queries need enough lookback relative to the range vector window (empty result
  with exit 0 is a footgun, not an error).

**Verdict impact:** maturity/ergonomics edge for GT **sharpened on completeness**, not
only on GA flag. Capability is no longer vapor for `rate`/`sum`/`avg by`, but a product
that expects full Prometheus function coverage cannot treat CH TimeSeries as drop-in yet.
Still comparator-only for Parallax stack policy.

### Run 411 — `increase` not unblocked by aggregate-fn flag

On head **26.7.1.1097**:

- `allow_experimental_time_series_aggregate_functions=1` **does not** enable
  `increase(…)` inside `prometheusQuery` — still Code 48
  `Function increase is not implemented`.
- Catalog shows a rich set of **grid helpers**
  (`timeSeriesRateToGrid`, `timeSeriesInstantRateToGrid`, `timeSeriesDeltaToGrid`,
  `timeSeriesResetsToGrid`, …) but **no** `timeSeriesIncreaseToGrid` and no PromQL
  `increase` path.
- Defaults remain: `allow_experimental_time_series_table=0`,
  `allow_experimental_time_series_aggregate_functions=0`.

**Workaround for CH lab/dashboards:** use `rate(x[w]) * window_seconds` (or SQL
over samples/tags). GT keeps first-class `increase` / `prom_increase`.

### Run 423 (2026-07-18) — expanded PromQL function matrix (CH head 26.7.1 + GT)

**Setup:** same multi-series counter/gauge load as Run 404 into `ts_r423` (CH) and
`r423_http` (GT mito). Eval at `2026-07-18 00:04:00`. CH via
`prometheusQuery(ts, promql, eval_time)`; GT via `TQL EVAL`.

| Expression class | Examples | CH head 26.7.1 | GT v1.1.3 TQL |
| --- | --- | --- | --- |
| Instant selector | `up`, label matchers | **OK** | **OK** |
| Rate family | `rate`, `irate` | **OK** (1.0 / 0.5) | **OK** |
| Increase | `increase` | **Code 48 NOT_IMPLEMENTED** | **OK** (120 / 60) |
| Delta family | `delta`, `idelta` | **OK** | (not retested) |
| Agg + grouping | `sum(rate) by (job)`, `max by (job)` | **OK** (1.5 / 340) | **OK** (`sum by` 1.5) |
| Ranking | `topk`, `bottomk` | **OK** | **OK** (`topk` → i1) |
| Last | `last_over_time` | **OK** (170 / 340) | **OK** |
| Range rollups | `min/max/avg/count_over_time` | **all Code 48** | **all OK** |
| Extrapolation | `deriv`, `predict_linear` | **Code 48** | (not retested) |
| Resets/changes | `resets`, `changes` | **Code 48** | (not retested) |
| Absent / clamp | `absent`, `clamp_min` | **Code 48** | (not retested) |
| Comparison | `rate(…) > 0.5` | **OK** (filters to i1) | (not retested) |
| Offset | `http_requests_total offset 1m` | **OK** | (not retested) |
| `histogram_quantile` | on non-histogram rate | **OK** (returns 0 — wrong input shape, no error) | (not retested) |

**26.6.1 stable:** spot-check same — `last_over_time` + `topk` OK; `min_over_time` Code 48
(identical gap surface to head for missing fns).

**Mechanism reading:**

- CH experimental PromQL is a **partial** Prometheus surface: strong on **rate +
  aggregation + ranking + last + binary compare + offset**; weak on **increase**,
  **`*_over_time` except last**, **deriv/predict_linear**, **resets/changes**,
  **absent/clamp**.
- Interesting asymmetry: SQL helper catalog has `timeSeriesResetsToGrid` /
  `timeSeriesDeltaToGrid` / `timeSeriesDerivToGrid`, but PromQL `resets`/`deriv` still
  Code 48 — grid helpers ≠ full PromQL function coverage.
- GT GA path implements the rollups CH lacks (`prom_min/max/avg/count/last_over_time`,
  `prom_increase`) on plain mito tables.

**Verdict impact:** sharpens maturity gap beyond “experimental flag.” A Grafana board
that uses `increase`, `avg_over_time`, or `resets` **cannot** drop onto CH TimeSeries
today without rewrite; GT can. Still **not** a stack flip (product = GT); comparator
watch for when CH fills `*_over_time` + `increase`.

### Run 560 (2026-07-18) — matrix re-smoke + offset nuance

**Setup:** SQL INSERT into existing `ts_r423` (26.6.1) + fresh `ts_r560h` (head 26.7.1);
counter `r560_counter` with jobs j0/j1; table-function form
`SELECT * FROM prometheusQuery(ts, promql, now())` (scalar form is UNKNOWN_FUNCTION).

| Class | Expression | CH 26.6.1 | CH head 26.7.1 |
| --- | --- | --- | --- |
| Rate | `sum(rate(r560_counter[2m]))` | **OK** `0.5` | **OK** `~0.167` (single series) |
| Increase | `sum(increase(…[2m]))` | **Code 48** | **Code 48** |
| Ranking | `topk(1, sum by (job) (rate(…)))` | **OK** (job=j1) | — |
| Last | `last_over_time(…[2m])` | **OK** | — |
| Range rollups | `avg/max/min/sum/count_over_time` | **all Code 48** | — |
| Instant delta | `delta`, `idelta`, `irate` | **OK** | — |
| Compare | `sum(rate(…)) > bool 0` | **OK** `1` | — |
| Extrapolation / resets / absent / clamp | `deriv`, `predict_linear`, `resets`, `changes`, `absent`, `clamp_min` | **Code 48** | — |
| **Simple offset** | `r560_counter offset 1m` | **OK** (j0=20, j1=110) | — |
| **Simple offset + sum** | `sum(r560_counter offset 1m)` | **OK** `130` | — |
| **Range offset** | `rate(…[2m] offset 1m)`, `last_over_time(…[2m] offset 1m)` | **Code 43** `ILLEGAL_TYPE_OF_ARGUMENT` (`toIntervalNanosecond` / Decimal) | — |
| Outer offset parse | `sum(rate(…[2m])) offset 1m` | **Code 756** `CANNOT_PARSE_PROMQL_QUERY` | — |
| Range query | `prometheusQueryRange` rate 5m/1m | **OK** (2 points) | — |

**Mechanism reading (new):**

1. **No drift** on the Run 423 gap surface: still partial PromQL; `increase` + most
   `*_over_time` + deriv/resets/absent/clamp missing on **both** feature line and head.
2. **Offset is not fully OK.** Run 423’s “offset OK” holds only for **selector-level**
   `metric offset 1m`. **Range-vector offsets** (`rate(x[w] offset …)`) hit a
   **type bug** (Decimal vs interval), not “not implemented” — different failure mode.
3. Call-site footgun: use **table function** `FROM prometheusQuery(...)`, not bare
   scalar `SELECT prometheusQuery(...)` (Code 46).

**Verdict impact:** comparator watch only; GT remains the product metrics/Prom path.
No pin bump. **Not done.**

### Run 572 (2026-07-18) — TimeSeries **not in ClickHouse Cloud**

Primary docs (fetched 2026-07-18):
[TimeSeries table engine](https://clickhouse.com/docs/engines/table-engines/special/time_series):

- Marked **Experimental feature** (may change in backwards-incompatible ways).
- Explicit line: **“Not supported in ClickHouse Cloud.”**
- Enablement still gated (`allow_experimental_time_series_table`).

**Implication for the managed comparison (Run 221/558 list rates):** even if
CH Cloud closes OSS N×/cold-S3 economics via SharedMergeTree, **native PromQL
via TimeSeries is not a Cloud product path today**. ClickStack/blog notes continue
to describe experimental PromQL growth on the engine, but Cloud customers do not
get this facade. Greptime managed + GA PromQL/OTLP/Jaeger remains the only
**managed** path in this study with first-class Prom/OTEL without a parallel
collector stack.

**No stack flip** (product already self-host GT). Comparator honesty: CH
experimental PromQL is **OSS-lab only** for now. **Not done.**

## Run 403 mechanism takeaway (why Code 48 is not a death sentence)

`TimeSeries` is closer to a **materialized-view-style multi-target router** than a
queryable storage engine:

1. INSERT on the outer table → transform → write samples + tags + metrics parts.
2. Read path is **not** MergeTree select-from-part on the facade; it is PromQL table
   functions that join tags→id→samples (and optional min/max time filters).
3. Therefore `SELECT * FROM ts` is unimplemented **while** the product query path
   (`prometheusQuery*`) is the real one — measuring only Code 48 understates CH progress.

For Parallax: still **do not** plan product metrics storage on CH TimeSeries (stack is
GT). As a **comparator maturity watch**, CH PromQL is a real experimental engine with a
working SQL INSERT lab path as of 26.6/26.7 — re-check when/if it leaves experimental
or gains Cloud support.

## Honest caveats

- ClickHouse PromQL is experimental — **feature completeness partially measured
  (Run 404):** `rate`/`irate`/`delta`/`sum`/`avg by`/matchers work; **`increase` missing**
  on 26.6+26.7. Volume/cardinality completeness still owed.
- GreptimeDB PromQL completeness vs upstream Prometheus is also not exhaustively
  tested here, but it is GA and the planner covers the core operators (above).
- This pass corrected the *query* side; the **ingest** side ("ClickHouse needs a
  collector, no native Prom remote-write") is **also softened** — the `TimeSeries`
  engine accepts Prometheus remote-write — and is flagged for a write-path re-verify.

## Source / evidence

- GreptimeDB: `src/promql/src/extension_plan/{planner.rs,instant_manipulate.rs,
  range_manipulate.rs,normalize.rs,series_divide.rs,histogram_fold.rs,absent.rs,
  scalar_calculate.rs,union_distinct_on.rs,empty_metric.rs}`; `src/promql/src/functions`
  (`prom_rate` etc.). Live: Prom HTTP API + `TQL EXPLAIN`.
- ClickHouse: `TimeSeries` table engine + `prometheusQuery`/`prometheusQueryRange` /
  `timeSeries*` table functions; settings `allow_experimental_time_series_table`
  (default 0), `allow_experimental_time_series_aggregate_functions`,
  `promql_database`/`promql_table`/`promql_evaluation_time`. Live (Run 23).
- Empirical: `local-benchmark-results.md` Run 23 (capability), Run 24 (maturity), Run 3/11/37
  (SQL metric-agg speed), **Run 44 (native PromQL path ~5× slower than GT SQL at 40k series;
  `SeriesNormalize` fixed-setup mechanism)**.
- Cross-refs: `per-signal-verdict.md`, `verdict-which-to-choose.md`,
  `write-path-and-ingestion.md` (ingest side), `query-execution-engine.md` (speed).

## Run 183 (2026-07-17) — PromQL vs SQL re-verify on v1.1.3 (scale-shape correction)

**Pass target.** Re-check Run 105 claim: GT PromQL ~5.6× slower than GT SQL on
`avg by(service)` (and both behind CH SQL). Method: server-side timings only for
fair GT SQL vs GT PromQL — use **`TQL EVAL`** (returns `execution_time_ms`, same HTTP SQL
channel as SQL). Prometheus HTTP `/query` wall includes `docker exec`+curl (~60–75 ms)
and is **not** comparable to `execution_time_ms`.

**Dataset.** N=100k, flushed mito tables with `greptime_value` double (PromQL value column).

| Case | GT SQL | GT TQL/PromQL | CH SQL | Notes |
| --- | ---: | ---: | ---: | --- |
| `prom_m` 40 series, `avg by (service)` | **6 ms** | TQL **7–8 ms** (~1.2×) | **3–4 ms** | fixed-overhead |
| `prom_hc` 400 series, flat SQL avg | **8 ms** | — | — | |
| `prom_hc` TQL narrow 100s / step 10s | — | **15 ms** (~1.9× SQL) | — | |
| `prom_hc` TQL wide 100m / step 60s | — | **13 ms** (~1.6× SQL) | — | |
| `prom_hc` TQL `rate()[5m]` wide | — | **14 ms** (~1.8× SQL) | — | |
| SQL `date_bin` 1-min panel | **9 ms** | — | — | closest SQL panel to range |

Warm median of 8; GT `execution_time_ms`; CH `clickhouse-client --time`.

**Instant PromQL gotcha:** `/v1/prometheus/api/v1/query` with default `time=now` against
historical fixture timestamps returns **empty vector**. Pin `time=` inside the data window
(or use `query_range`) — then 40 series return correctly.

**CH PromQL:** `allow_experimental_time_series_table=0` still. Enabling flag + naive
`ENGINE=TimeSeries` DDL rejected (needs INNER COLUMNS form) — experimental setup still
heavier than GT GA PromQL. Not a GA-ergonomic path.

**Verdict / correction**

1. Ordering **holds**: CH SQL ≥ GT SQL ≥ GT TQL/PromQL at equal effort.
2. The **~5.6× PromQL tax is not universal** — at N=100k laptop tier, TQL is only
   **~1.5–2×** SQL for the shapes above. Run 105’s ~5.6× was a **wide PromQL range over
   denser/higher-card data**; treat it as scale-shaped, not a constant multiplier.
3. **Blueprint unchanged:** drive hot Parallax panels with **SQL or Flow**; use PromQL when
   clients require Prometheus API compatibility, not for the absolute hottest path.
4. Mechanism (unchanged): PromQL lowers through `PromExtensionPlanner` + series
   normalize/divide plans (`src/promql`, registered in `query_engine/state.rs`) — extra plan
   stages vs plain DataFusion SQL agg.

**Reproduce**
```bash
# after gen or create prom_hc as above
docker exec parallax-bench-greptimedb-1 curl -s 'http://localhost:4000/v1/sql?db=public' \
  --data-urlencode "sql=SELECT service, avg(greptime_value) FROM prom_hc GROUP BY service"
docker exec parallax-bench-greptimedb-1 curl -s 'http://localhost:4000/v1/sql?db=public' \
  --data-urlencode "sql=TQL EVAL (1716000000, 1716060000, '60s') avg by (service) (prom_hc)"
```

## Run 196 (2026-07-17) — ClickHouse TimeSeries still experimental / incomplete (26.6)

With `allow_experimental_time_series_table=1`:

| Step | Result |
| --- | --- |
| `CREATE TABLE prometheus ENGINE=TimeSeries` | **OK** — expands to SAMPLES/TAGS/METRICS inner engines |
| `SELECT … FROM prometheus` | **Code 48 NOT_IMPLEMENTED** — “SELECT is not supported by storage TimeSeries yet” |
| Column-list DDL (`id, timestamp, value`) | Rejected — must use engine-native form / INNER COLUMNS |

**No drift from Run 24/164:** CH PromQL remains experimental and setup-heavy; table create works
but ordinary SQL SELECT is still unimplemented. GT PromQL stays the GA-native path.

### Run 576 (2026-07-18) — rate still works; increase still missing

Fresh `r576_c` counter on `ts_r423`: `sum(rate(r576_c[2m]))` → **1.0**;
`increase` → **Code 48**. Four-way health **200**. No drift vs Run 560.
**Not done.**

