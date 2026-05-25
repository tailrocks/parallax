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
- **Status:** **DONE at medium-warm (Run 12).** 5M realistic logs, both with text
  indexes: **full-text search ClickHouse 7 ms vs GreptimeDB 130 ms (~18×)**, full
  scan ~4×, but **selective keyed filter a tie** (4 vs 5 ms). Flip-trigger
  confirmed: log-search-at-volume strongly favors ClickHouse; anchored/keyed access
  does not. True cold GB–TB (drop OS cache, 25–50 GB) owed to the full harness —
  expected to widen the gap.

## B2 — `trace_id` point lookup with the GreptimeDB inverted-index fix

- **Hypothesis / mechanism:** Adding `trace_id INVERTED INDEX` to GreptimeDB spans
  (`greptimedb-implementation.md`) closes the Run-1 gap (16 ms un-indexed vs 2 ms
  ClickHouse sort-prefix) to near-parity, *without* exploding series cardinality.
- **Workload:** `spans WHERE trace_id = ?`, 1M+ rows, warm + cold.
- **Record:** lookup latency p50/p95; GreptimeDB rows/granules scanned.
- **Pass/fail:** GreptimeDB indexed lookup within ~2× of ClickHouse → confirms the
  schema fix; if still ≫ → inverted index insufficient, escalate.
- **Prereq:** rebuild GreptimeDB `spans` with `trace_id INVERTED INDEX`.
- **Status:** **DONE (Run 6, partially confirmed).** Inverted index cut trace
  lookup 14→8 ms (~2×) but did not reach ClickHouse parity (2 ms) at smoke — the
  residual is GreptimeDB's fixed query-setup floor (DataFusion + MergeScan), not
  index quality. 8 ms is fine in absolute terms for anchored bundles. Re-test at
  `small`+ and via the MySQL native protocol to isolate scan vs fixed overhead.

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
- **Status:** **DONE at smoke (Runs 2 + 16).** Full composite Q6 (Q1+Q2+Q3) measured
  end-to-end, parity PASS: CH ~10 ms vs GT ~33 ms total — both far under the 300 ms
  gate. Q2 issue-history (PK lookup) a tie (3 ms each); GT's gap is the 3-way UNION's
  per-query fixed overhead, not algorithmic. Confirms the anchored bundle is **not
  latency-bound** on either engine. Larger-tier cold + concurrent still owed to the
  prototype.

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
- **Status:** **DONE (Run 11) — refined against the hypothesis.** At 40k series /
  8M rows the SQL `avg by service` aggregation was **ClickHouse 65 ms vs GreptimeDB
  638 ms (~10×)** (Run-3's near-tie at 1,200 series was a small-scale artifact).
  GreptimeDB's metrics advantage is **PromQL-capability + ingest, not aggregation
  speed at volume**. Owed: native-PromQL-path + metric-engine high-card run (could
  differ from the plain-table SQL group-by measured here).

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
- **Status:** **DONE (Run 10).** 500k logs, 99% unique messages: GreptimeDB 25 MiB
  vs ClickHouse 35.5 MiB at **defaults** (ClickHouse ids default to LZ4), but
  ClickHouse with ZSTD on all string cols = **24.24 MiB ≈ tie**. Realistic-log
  compression is a tie at matched effort, GreptimeDB-favored out-of-the-box (it
  ZSTDs everything; ClickHouse needs explicit per-column ZSTD on high-card hex).
  Run-4's GreptimeDB win was a default-codec effect, not a synthetic artifact.

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
- **Status:** **DONE (Run 13, within-engine penalty).** Both pass the ≤2× gate:
  query latency under heavy concurrent ingest (3M→11M rows mid-query) rose only
  ClickHouse 1.55× (11→17 ms), GreptimeDB 1.38× (66→91 ms). Neither blocks reads on
  ingest. Absolute agg at 11M still ~5× ClickHouse (volume gap). Precise mixed-load
  freshness p95 (stamp-emit→poll) still owed to harness instrumentation.

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
- **Prereq:** small-write load mode (a **rate-ramp**: insert faster than merges
  keep up until parts approach 3000).
- **Status:** **DONE (Run 7), refined.** Mechanism confirmed (300 inserts = 300
  `NewPart` events) but background merges collapsed 300→1 active and the guard is
  3000, so the explosion is a **sustained-rate** failure, not per-insert; default
  `async_insert` mitigates. GreptimeDB advantage real but narrower. A sustained
  rate-ramp is still owed to prove the actual throw threshold under load.

## B10 — Object-storage economics (MinIO), both engines

- **Hypothesis / mechanism:** GreptimeDB is object-store-native (OpenDAL + default
  read cache); ClickHouse uses an S3 disk under a storage policy with TTL-move
  tiering. Cost tracks retained bytes × price + egress on cold re-read
  (`compression-and-cost.md`, `retention-and-ttl.md`).
- **Workload:** bring up MinIO (add to `bench/compose.yml`); GreptimeDB `[storage]
  type=S3` + ClickHouse `s3` disk policy; ingest `small` tier; run cold-cache Q1–Q6.
- **Record:** retained object bytes per signal; S3 GET/PUT/LIST counts during ingest
  and during cold queries; cold-read egress; warm vs cold latency.
- **Pass/fail:** quantify the object-store cost + the cold-read request amplification
  per engine; is GreptimeDB's native path cheaper or just simpler?
- **Prereq:** MinIO in compose; S3 config for both; request-counting instrumentation.
- **Status:** **DONE (Runs 8–9).** Same MinIO, 1M spans: **GreptimeDB 4 objects /
  37 MiB vs ClickHouse 74 objects / 63 MiB** (~18× more objects — Wide part =
  one S3 object per column + marks; confirmed the hypothesis). ClickHouse active
  logical 31.82 MiB is slightly smaller (codec edge) but raw S3 use is inflated by
  un-GC'd merge garbage (async S3 cleanup). **Run 14 added the cold per-query GET
  count (anchored lookup): ClickHouse 5 vs GreptimeDB 22** — the *reverse* of the
  object count, because ClickHouse's sort-key locality pinpoints ~1 granule while
  GreptimeDB pays inverted-index indirection. So object-store request cost is
  **query-shape-dependent**: GreptimeDB fewer *total objects* (full-scan/management
  edge), ClickHouse fewer *GETs per anchored lookup*. Full-scan cold GET count
  (B12, where GreptimeDB's few objects should win) still owed.

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

## B12 — Reproduce the JSONBench cold-run result (object-store, 1B-doc regime)

- **Hypothesis / mechanism:** GreptimeDB reportedly ranks **#1 on cold run** in
  ClickHouse's own JSONBench (1B JSON docs) — its object-store-native Parquet layout
  (few large objects, Run 9) + cold reads beats ClickHouse's S3-disk many-small-
  objects path in the **cold / object-store / wide-semi-structured** regime
  (`public-performance-claims.md` claim #6). **This is the regime closest to
  Parallax's actual retention re-reads**, the opposite of the hot in-cache scans my
  B1/B5 measured.
- **Workload:** JSONBench-style 1B (or scaled-down, e.g. 50–100M) JSON/wide-event
  docs on object storage (MinIO), **cold cache**, the JSONBench query set; both
  engines S3-backed.
- **Record:** cold-run per-query latency p50/p95; object GET count per query; warm
  vs cold delta.
- **Pass/fail:** does GreptimeDB's cold-object-store advantage reproduce
  independently? If yes, it **materially strengthens the verdict for Parallax's
  cold re-read pattern** (flips the "ClickHouse wins at volume" reading for the
  *cold* regime).
- **Prereq:** JSONBench dataset + queries; MinIO; both in S3 mode; cold-cache control.
  The **object-store stack is now committed and reproducible**: `bench/s3/run-s3-stack.sh up`
  brings up MinIO + GreptimeDB(S3) + ClickHouse(S3) (proven Runs 8–9) — so B10
  request-counts and B12 cold reads start from a one-command base; what's still
  needed is the JSONBench dataset/queries + cold-cache eviction + `mc admin trace`
  request counting.
- **Status:** **local full-scan part DONE (Run 15).** Cold full-scan GET count:
  **GreptimeDB 26 vs ClickHouse 57** — GreptimeDB's few-large-objects layout issues
  fewer cold S3 GETs on a full scan, **locally confirming the JSONBench cold-run
  mechanism**. Combined with Run 14 (anchored: CH 5 < GT 22), the cold request-cost
  splits cleanly by query shape. The **1B-doc JSONBench scale** + cold-latency (not
  just GET count) stays the prototype's job; the mechanism is now verified locally.

## B13 — High-cardinality metric storage (the `LowCardinality` cliff)

- **Hypothesis / mechanism:** GreptimeDB's metric engine (`__tsid` label-set hash +
  shared physical wide table) + PartitionTree memtable (dict-encoded label sets,
  sharded, no per-series cap) stores high-cardinality series more cost-stably than
  ClickHouse, whose `LowCardinality(String)` dictionary **caps at 8,192 distinct
  values** then falls back to plain encoding (`metric-cardinality.md`, Run 26). Above
  the cliff, ClickHouse label columns should bloat (lost dict encoding) + the sparse
  primary index gains marks, while GreptimeDB's per-memtable dict keeps sharing label
  strings.
- **Workload:** ingest a metric at **N distinct series** for N ∈ {1k, 10k, 100k, 1M}
  (label combos that cross the 8,192 cap), same sample count per series; GreptimeDB
  metric engine vs ClickHouse `ORDER BY (metric, labels…, ts)` with
  `LowCardinality(String)` labels.
- **Record:** retained bytes per series-tier (the cliff should show as a kink in the
  ClickHouse curve at ~8,192), ingest rows/s, single-label-filter latency
  (`{job="x"}`), and memtable/flush memory.
- **Pass/fail:** does ClickHouse's per-series byte cost jump past 8,192 distinct while
  GreptimeDB's stays smooth? If yes, confirms the storage-ergonomics edge for
  high-cardinality Parallax metrics. (Agg *latency* is separate — Run 11 already gives
  ClickHouse ~10×; B13 is about **storage/ingest**, not aggregation.)
- **Prereq:** a high-cardinality metric generator knob (distinct-series count) added to
  the harness; both stacks already up. **Status: proposed (pass 48), not yet run.**

## B14 — Multi-replica object-store cost (the zero-copy gap)

- **Hypothesis / mechanism:** for HA on S3, OSS ClickHouse `ReplicatedMergeTree` is
  shared-nothing — each replica stores its **own full copy** → **N replicas ≈ N× S3
  bytes**; zero-copy replication (1× shared copy) is `allow_remote_fs_zero_copy_replication=0`
  + *"not ready"* (`wal-and-durability.md` / `distributed-and-scaling.md`, Run 34), and
  `SharedMergeTree` is Cloud-only. GreptimeDB is object-store-native → **one shared S3
  copy per region** regardless of HA replicas (replication = leadership/metadata, not
  data copy). So GreptimeDB HA should cost ~1× S3, ClickHouse ~N×.
- **Workload:** a 3-replica HA cluster of each on MinIO/S3, same dataset (`small` tier);
  measure **total S3 bytes stored** across the replica set. Optionally repeat with
  ClickHouse zero-copy *on* (to quantify the not-production-ready alternative).
- **Record:** total object-store bytes for the replica set; bytes-per-replica; object
  count; (if zero-copy on) any consistency/corruption events under churn.
- **Pass/fail:** GreptimeDB replica-set S3 bytes ≈ 1× single-copy; ClickHouse default
  ≈ 3×. Confirms the replication-storage-economics edge → real HA cost multiplier on the
  cost axis. "Close enough to not matter" only if Parallax runs single-replica (no HA).
- **Prereq:** multi-node compose for both (3 replicas), MinIO, S3 byte accounting
  (`mc du` / `system.parts` bytes_on_disk × replicas). **Needs the multi-node harness.**
  **Status: proposed (pass 59), not yet run.** (Mechanism source-confirmed, Run 34.)

## B15 — Strict-durability throughput cost (`sync_write` vs `fsync_after_insert`)

- **Hypothesis / mechanism:** both default to **throughput-over-fsync** (GreptimeDB
  `sync_write=false`; ClickHouse `fsync_after_insert=false`, `wal-and-durability.md`).
  Forcing per-write durability — GreptimeDB `sync_write=true` (fsync each WAL record);
  ClickHouse `fsync_after_insert=1` (fsync each part) — costs ingest throughput. The
  question: **how much**, and is one engine's strict-durable mode cheaper? GreptimeDB's
  WAL-append fsync (sequential) should beat ClickHouse's per-part fsync (the CH doc warns
  it "significantly decreases performance").
- **Workload:** sustained single-row + small-batch ingest at a fixed rate, four configs:
  GT default / GT `sync_write=true` / CH default / CH `fsync_after_insert=1`. Same
  hardware, same NVMe.
- **Record:** ingest rows/s and p99 insert latency per config; the **throughput ratio
  strict-vs-default** for each engine.
- **Pass/fail:** quantify each engine's strict-durability penalty; confirm/refute that
  GreptimeDB's sequential-WAL fsync is the cheaper strict-durable path. Matters only if
  Parallax needs hard per-write durability (telemetry usually tolerates the default).
- **Prereq:** a fixed-rate small-write driver (have the shape from B9) + the two strict
  settings. Runnable on the existing single-node stacks. **Status: proposed (pass 59),
  not yet run** (mechanism source+live confirmed, Runs 20/33).

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
