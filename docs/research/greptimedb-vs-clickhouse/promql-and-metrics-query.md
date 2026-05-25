# PromQL and Metrics Query — The Capability Gap, Re-Verified

<!-- markdownlint-disable MD013 -->

Status: pass 44 (capability/maturity) + pass 72 (PromQL **speed** characterization, Run 44)
+ pass 98 (**re-verified live, no drift, Run 62**). The PromQL planning path (a GreptimeDB
system-lead) **and** a required re-verification of the verdict's load-bearing claim that
"ClickHouse has no PromQL." Metrics/PromQL nativeness is the verdict's #1 GreptimeDB
advantage, so a version-drift here is decision-critical. Source + live (Runs 23, 24, 44, 62).

**Re-verification (Run 62, v1.0.2 / 26.5.1.882 — pillar STABLE):** GreptimeDB PromQL
still GA + zero-setup — `/v1/prometheus/api/v1/query?query=avg(metrics_hc)` returned a
real vector (`50.77`) and `TQL EVAL` a real value (`49.98`) against a **plain `mito`
table** (no metric-engine table needed). ClickHouse unchanged: `allow_experimental_time_series_table=0`
(off by default); `prometheusQuery`/`prometheusQueryRange` exist; the `TimeSeries` engine
is creatable with the flag but **`INSERT`/`SELECT` still "not supported by storage
TimeSeries yet"** (NOT_IMPLEMENTED, reproduces Run 24) — ingest remote-write-only, query
table-function-only. The "GA-ergonomic vs experimental-setup-gated" gap holds exactly.

**Headline correction:** the old "ClickHouse has **no** PromQL, needs an external
PromQL→SQL layer" is **outdated as of ClickHouse 26.x**. ClickHouse now ships
**experimental** native PromQL. The gap is no longer *present vs absent* — it is
**GA-native-ergonomic (GreptimeDB) vs experimental-off-by-default-setup-heavy
(ClickHouse)**. This narrows, but does not flip, the metrics verdict.

Pins: GreptimeDB `v1.0.2` (`0ef5451`), ClickHouse `v26.5.1.882-stable` (`5b96a8d8`),
re-confirmed latest stable 2026-05-25.

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
  — a Prometheus-shaped store (tags/metrics/data sub-tables), fed by the **Prometheus
  remote-write** protocol.
- **PromQL table functions** (live, Run 23): `prometheusQuery([db,] ts_table, promql
  [, eval_time])` and `prometheusQueryRange(…)` — they **execute PromQL** against a
  `TimeSeries` table. Plus `timeSeriesSelector/Metrics/Data/Tags`. Settings
  `promql_database`/`promql_table`/`promql_evaluation_time` configure the target;
  `allow_experimental_time_series_aggregate_functions` (default 0) gates the agg fns.
- **Live (Run 23):** `CREATE TABLE … ENGINE=TimeSeries` succeeded (with the
  experimental flag); `prometheusQuery('up')` exists with a real 3–4 arg signature
  (errored only on arg count / empty table, not "unknown function"). So the capability
  is present — but **requires** the experimental flag, a dedicated `TimeSeries` table,
  and Prometheus remote-write ingest. **Not the default, not ergonomic, young.**

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

## Axis consequence

- **Capability (axis #1 enabler):** metrics/PromQL is no longer binary. GreptimeDB
  leads on GA + ergonomics; ClickHouse has closed the *can-it-at-all* gap
  experimentally. Net: still a GreptimeDB advantage for Parallax shipping today, but
  **downgraded from "decisive binary" to "maturity/ergonomics lead."**
- **Replaceability (Q3):** "ClickHouse can't do PromQL" is no longer a hard blocker; it
  becomes "ClickHouse's PromQL is experimental, so relying on it for a metrics product
  today is a maturity risk + setup cost," which is softer.

## Maturity, measured end-to-end (pass 45, Run 24)

Pass 44 established ClickHouse PromQL *exists*; this pass ran it to characterize *how
usable* it is. Concrete findings on the `TimeSeries` engine + `prometheusQuery[Range]`:

- **No direct `INSERT`** — `INSERT INTO <ts_table>` → `"INSERT is not supported by
  storage TimeSeries yet"` (NOT_IMPLEMENTED). Data can land **only via the Prometheus
  remote-write protocol**.
- **No direct `SELECT`** — `SELECT … FROM <ts_table>` → `"SELECT is not supported by
  storage TimeSeries yet"`. The table is queryable **only** through the
  `prometheusQuery`/`prometheusQueryRange`/`timeSeries*` functions.
- **The PromQL functions do execute.** `prometheusQuery(<table>, '<promql>',
  <eval_time>)` and `prometheusQueryRange(<table>, '<promql>', start, end, step)`
  parsed and ran `rate(http_requests_total[2m])` and an instant selector with **no
  error** (returned empty only because hand-loading the engine's inner `.data` table
  without populating the id-coupled `.tags`/`.metrics` inner tables yields no
  resolvable series — i.e. there is **no practical hand-load path**; you need a real
  remote-write client).
- **GreptimeDB, same workload, real result:** an InfluxDB-line write auto-created the
  metric table, and `TQL EVAL (start,end,'30s') rate(http_requests_total[2m])`
  returned **actual values** (`0.72`, `1.17` for `job=api`) via the native `prom_rate`
  planner — zero ceremony, multi-protocol ingest, PromQL **and** SQL **and** TQL query.

So the maturity gap is concrete: **ClickHouse PromQL = experimental, remote-write-only
ingest, table-function-only query, no INSERT/SELECT** ("yet"); **GreptimeDB PromQL =
GA, multi-protocol ingest, multi-surface query, works on any metric table.** The
*capability* exists on both (pass 44); the *workflow maturity/ergonomics* gap is large
and real (pass 45). This is the precise shape of the metrics-pillar advantage now.

## Honest caveats

- ClickHouse PromQL is experimental — **feature completeness vs Prometheus is
  unverified** (which PromQL functions/selectors work, edge cases). A fair "can it run
  Parallax's actual PromQL set" test needs data loaded into a `TimeSeries` table — owed
  to a follow-up (a strong `benchmarking-the-differences.md` case).
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
