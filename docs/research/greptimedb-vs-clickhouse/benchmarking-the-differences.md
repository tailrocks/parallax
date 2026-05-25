# Benchmarking The Differences — Targeted Cases For Parallax

<!-- markdownlint-disable MD013 -->

Status: pass 14. Turns every mechanism-level difference found in passes 1–13 into
a **targeted benchmark** (hypothesis → workload → metric → pass/fail →
prerequisites), and routes runnable cases into the harness
(`storage-benchmark-prototype.md`, which holds veto). Each case isolates one
difference; this is not a general scan. Measurement protocol + generator knobs are
the prototype's; this note says *what to run and why it matters*.

Pins: GreptimeDB `v1.0.2` (`0ef5451`), ClickHouse `v26.5.1.882-stable` (`5b96a8d8`).

Legend: **Runnable now** = expressible in the current prototype/`bench/compose.yml`.
**Needs harness ext.** = the generator/harness must be extended first.

## B1 — Selective log/trace scan, cold cache, GB–TB (the flip-trigger)

- **Hypothesis / mechanism:** ClickHouse is faster on selective log/trace scans
  because of the 8,192-row granule (vs GreptimeDB's 102,400-row Parquet row group)
  + PREWHERE late materialization + decade-tuned vectorized scan
  (`read-path-indexing-and-execution.md`). Smoke showed direction (Run 1) but was
  cache-resident — *inconclusive for throughput*.
- **Workload:** `logs` table, `WHERE service=X AND level=error` over a 1-day window,
  at `small` (25–50 GB) then `medium`, **cold cache** (restart + drop page cache).
- **Record:** per-class latency p50/p95/p99 (cold + warm), bytes read.
- **Pass/fail:** if GreptimeDB cold p95 is within ~1.5× of ClickHouse → "close
  enough" for Parallax; if >3× → ClickHouse's read-path edge is *material* and,
  combined with a log-search-dominated query mix, **flips the verdict** (Q5).
- **Prereq:** `small`+ tier with realistic log text; cold-cache harness step.
- **Status:** Needs harness ext. (cold-cache control + larger tier). **Top priority.**

## B2 — `trace_id` point lookup with the GreptimeDB inverted-index fix

- **Hypothesis / mechanism:** Adding `trace_id INVERTED INDEX` to GreptimeDB spans
  (`greptimedb-implementation.md`) closes the Run-1 gap (16 ms un-indexed vs 2 ms
  ClickHouse sort-prefix) to near-parity, *without* exploding series cardinality.
- **Workload:** `spans WHERE trace_id = ?`, 1M+ rows, warm + cold.
- **Record:** lookup latency p50/p95; GreptimeDB rows/granules scanned.
- **Pass/fail:** GreptimeDB indexed lookup within ~2× of ClickHouse → confirms the
  schema fix; if still ≫ → inverted index insufficient, escalate.
- **Prereq:** rebuild GreptimeDB `spans` with `trace_id INVERTED INDEX`.
- **Status:** **Runnable now** (single-node, existing dataset) — cheapest high-value
  next run.

## B3 — Evidence-bundle Q1/Q4 anchored join at scale

- **Hypothesis / mechanism:** For *anchored* bundle queries, the join algorithm is
  **not** a differentiator — both engines propagate the anchor and prune before
  joining (Run-2 EXPLAIN). Expectation: latency tracks key placement + scan, not
  join strategy.
- **Workload:** Q1 (UNION) + Q4 (LEFT JOIN) anchored on one `trace_id`, at `small`+,
  warm + cold, under concurrent ingest.
- **Record:** Q1/Q4 p50/p95/p99; confirm both still prune to anchor (EXPLAIN).
- **Pass/fail:** both sub-300 ms warm at `small` (the prototype Q6 gate); neither
  degrades super-linearly with table size given the anchor.
- **Prereq:** the multi-signal dataset (have it) at larger tier.
- **Status:** Runnable now at smoke; Needs harness ext. for larger tier.

## B4 — Un-anchored large↔large `trace_id` join

- **Hypothesis / mechanism:** Where there is *no* selective anchor, GreptimeDB's
  partitioned hash join (repartition both sides) handles large↔large better than
  ClickHouse's broadcast/grace-spill (`read-path-indexing-and-execution.md`).
- **Workload:** join a day of `spans` to a day of `logs` on `trace_id` with **no**
  `trace_id` constant (e.g. all error-trace spans ↔ their logs).
- **Record:** latency, peak RSS (spill behavior), whether ClickHouse grace-spills.
- **Pass/fail:** informational — Parallax rarely runs this (bundles are anchored);
  flag only if one engine OOMs/spills catastrophically.
- **Prereq:** query templates without the anchor constant.
- **Status:** Needs harness ext. **Low priority** (not a Parallax pattern).

## B5 — Metrics: PromQL nativeness + aggregation latency at scale

- **Hypothesis / mechanism:** GreptimeDB serves PromQL natively (Run 3 capability
  win); ClickHouse needs a PromQL→SQL layer. Aggregation latency was within ~1.3×
  at smoke — does it hold at high series cardinality?
- **Workload:** 40k+ series (the prototype's `metrics_series`), PromQL-style
  `avg by (service) (rate(...))` over a window; ClickHouse via
  `AggregatingMergeTree` + the translated SQL.
- **Record:** range-agg p50/p95; ingest-to-queryable for metrics; series-cardinality
  ceiling.
- **Pass/fail:** GreptimeDB within ~1.5× on agg latency *and* PromQL-native →
  confirms the metrics advantage is real and not just capability.
- **Prereq:** high-cardinality metric generator (prototype has `metrics_series`).
- **Status:** Runnable now (smoke done); Needs harness ext. for 40k-series tier.

## B6 — Float-metric compression with realistic shapes

- **Hypothesis / mechanism:** ClickHouse `Gorilla`/`DoubleDelta` beat GreptimeDB
  Parquet on *real* metric shapes (flat gauges, monotonic counters) — Run 4 showed
  gauge 78× / counter 7.3× for ClickHouse, but Run 3's random-walk data was
  incompressible and GreptimeDB won there. Result is shape-dependent
  (`compression-and-cost.md`).
- **Workload:** generate flat gauges + monotonic counters + a few noisy signals;
  measure retained bytes per column on both.
- **Record:** retained size + compression ratio by metric shape.
- **Pass/fail:** quantify the gap per shape; "close enough" if total metric
  footprint within ~1.3×.
- **Prereq:** generator emits realistic metric shapes (not just random walk).
- **Status:** Needs harness ext. (generator shapes).

## B7 — Log/error text compression with realistic cardinality

- **Hypothesis / mechanism:** Run 4's GreptimeDB log win (5.5 vs 10.24 MiB) is a
  **synthetic artifact** (10 distinct messages → extreme dictionary friendliness).
  Real log text (high-entropy, many unique strings, stack traces) likely narrows or
  reverses it toward ClickHouse ZSTD.
- **Workload:** realistic log corpus (varied messages, real stack traces) into both
  `logs`/`error_events`; measure retained bytes.
- **Record:** retained size + ratio by signal; `message` column size specifically.
- **Pass/fail:** establishes the *real* log/trace storage cost ranking (the dominant
  Parallax volume).
- **Prereq:** realistic text generator (or a real log sample).
- **Status:** Needs harness ext. **High priority** (logs dominate volume + cost).

## B8 — Ingest-to-queryable freshness under concurrent load

- **Hypothesis / mechanism:** Both are visible-on-write (Run 5 tie). Under
  concurrent ingest+query, does either degrade? ClickHouse `async_insert` (default
  on, 50–200 ms) adds a small visibility delay; GreptimeDB memtable is immediate.
- **Workload:** the prototype's freshness protocol — stamp `t_emit`, poll every
  50 ms until visible — run while `load` drives target ingest and `query` runs Q1–Q6.
- **Record:** freshness p50/p95/p99 under write-only vs mixed; query p95 delta vs
  query-only.
- **Pass/fail:** prototype gate — freshness p95 ≤ 5 s mixed; concurrent penalty ≤ 2×.
- **Prereq:** concurrent load+query driver (prototype `load --mode mixed --with-query`).
- **Status:** Needs harness ext. (concurrent driver). Axis-1 priority.

## B9 — Small-write part-explosion vs memtable absorption

- **Hypothesis / mechanism:** ClickHouse one-part-per-INSERT risks "too many parts"
  on high-frequency small writes → needs `async_insert`/batching; GreptimeDB's LSM
  memtable absorbs them (`write-path-and-ingestion.md`).
- **Workload:** sustained single-row (or tiny-batch) inserts at rising rate, no
  client batching, `async_insert=0` on ClickHouse to expose the failure mode; then
  with default `async_insert=1`.
- **Record:** insert error rate / `parts_to_throw_insert` hits; part count over time;
  GreptimeDB SST/memtable behavior at the same rate.
- **Pass/fail:** confirms GreptimeDB ingests Parallax's streaming small-batch shape
  without a batching layer; quantifies the ClickHouse batching requirement.
- **Prereq:** small-write load mode.
- **Status:** Needs harness ext. (small-write driver).

## B10 — Object-storage economics (MinIO), both engines

- **Hypothesis / mechanism:** GreptimeDB is object-store-native (OpenDAL + default
  read cache); ClickHouse uses an S3 disk under a storage policy with TTL-move
  tiering. Cost tracks retained bytes × price + egost on cold re-read
  (`compression-and-cost.md`, `retention-cost-model.md`).
- **Workload:** bring up MinIO (add to `bench/compose.yml`); GreptimeDB `[storage]
  type=S3` + ClickHouse `s3` disk policy; ingest `small` tier; run cold-cache Q1–Q6.
- **Record:** retained object bytes per signal; S3 GET/PUT/LIST counts during ingest
  and during cold queries; cold-read egress; warm vs cold latency.
- **Pass/fail:** quantify the object-store cost + the cold-read request amplification
  per engine; is GreptimeDB's native path cheaper or just simpler?
- **Prereq:** MinIO in compose; S3 config for both; request-counting instrumentation.
- **Status:** Needs harness ext. (MinIO + S3 configs + request counters). Axis-2 key.

## B11 — Multi-node scale-out hold + rebalance

- **Hypothesis / mechanism:** GreptimeDB scales out by adding datanodes (Metasrv
  rebalances regions); ClickHouse OSS needs manual sharding + resharding
  (`distributed-and-scaling.md`). Architecture-reasoned only so far.
- **Workload:** 3+ node clusters of each; run Q1–Q6 at `medium`; then add a node and
  observe rebalance effort/latency.
- **Record:** query p95 as nodes added; GreptimeDB region-migration time vs the
  manual ClickHouse resharding steps; operator-action count.
- **Pass/fail:** does p95 hold as nodes are added; is GreptimeDB's rebalance
  hands-off vs ClickHouse manual?
- **Prereq:** multi-node compose for both; orchestration. **Heaviest** case.
- **Status:** Needs harness ext. Axis-3, lower priority than single-node settle.

## Priority order (what to run next)

1. **B2** (GreptimeDB inverted-index trace lookup) — runnable now, cheap, validates
   the implementation fix.
2. **B1** (cold GB–TB log/trace scan) — the verdict's flip-trigger; needs larger
   tier + cold cache.
3. **B7** (realistic log-text compression) + **B10** (object-store $) — the cost
   axis truth.
4. **B8** (concurrent freshness) + **B5** (metrics at 40k series) — axis-1 confirms.
5. **B9, B3, B11, B4, B6** — remaining, in roughly that order.

## Routing into the harness

All cases use the prototype's `StorageAdapter` + measurement protocol; new ones
are **folded back into `storage-benchmark-prototype.md`** (extend the generator and
`QueryClass` set), not forked here. Harness gaps to add: cold-cache control,
larger-tier streaming generator, realistic log-text + metric-shape generators,
concurrent load+query driver, small-write driver, MinIO + S3 request counters,
multi-node compose. This note proposes; the prototype runs and holds veto.
