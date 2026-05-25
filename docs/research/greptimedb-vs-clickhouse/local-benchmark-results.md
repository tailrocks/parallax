# Local Benchmark Results — Docker Smoke Runs

<!-- markdownlint-disable MD013 -->

Empirical log of local Docker runs. **Every number here is an indicative
laptop/dev-box smoke result, not a production verdict** (per the brief's honesty
rule). Numbers exist to confirm/refute the mechanism predictions in the internals
notes and the public performance claims — not to settle the choice. The runnable
`parallax-bench` harness in `storage-benchmark-prototype.md` holds the real veto.

## Run log

### Run 1 — 2026-05-25 — spans smoke, local disk, warm cache

**Environment**

| Item | Value |
| --- | --- |
| Host | Linux container dev box (orbstack); Docker 29.5.0, compose v5.1.3 |
| Compose | `bench/compose.yml` (local disk, no MinIO yet) |
| GreptimeDB | `greptime/greptimedb:v1.0.2` — standalone, default config (no codec tuning) |
| ClickHouse | `clickhouse/clickhouse-server:26.5.1.882` |
| Dataset | 1,000,000 synthetic spans, 14 spans/trace (~71k traces), 12 services, 3% error, seed 42. Identical CSV loaded into both. ~129 MB raw TSV. |
| Schema | The seed DDL from `storage-benchmark-prototype.md` spans table, **minus** the JSON/Map `attributes` column (added in a later run). ClickHouse `ENGINE=MergeTree ORDER BY (trace_id, ts)`; GreptimeDB `PRIMARY KEY (service, name)`, `ts TIME INDEX`. |
| Measurement | ClickHouse: `clickhouse-client --time` on `FORMAT Null`, min of 3 (warm). GreptimeDB: HTTP `/v1/sql` server-reported `execution_time_ms`, min of 3 (warm). |
| Caveat | Queries run **inside** the containers via `docker exec` (sandbox blocks host→container published ports). |

**Correctness parity (gate — must pass before trusting latency): PASS**

| Check | ClickHouse | GreptimeDB |
| --- | --- | --- |
| `count()` | 1,000,000 | 1,000,000 |
| `count WHERE status='error'` | 29,731 | 29,731 |
| spans for one `trace_id` | 14 | 14 |
| `avg(duration_ms)` | 24.96 | 24.96 |

**Retained on-disk size (after `OPTIMIZE FINAL` / `flush_table`)**

| Engine | Retained data | Note |
| --- | --- | --- |
| ClickHouse | **28.9 MiB** (1 part; 27.9 MiB compressed vs 101 MiB uncompressed ≈ 3.6×) | Schema uses tuned codecs: `ts CODEC(DoubleDelta,ZSTD)`, `LowCardinality` on service/name/status. |
| GreptimeDB | **38 MiB** SST (Parquet) | + 46 MiB WAL (transient, raft-engine; truncates in steady state — **not** counted as retained) + 2.1 MiB metadata. Default Parquet codecs, **no codec tuning** in the seed DDL. |

→ ClickHouse ~24% smaller on this dataset, **but** the comparison is codec-tuned
(ClickHouse) vs defaults (GreptimeDB) — a *schema-tuning* gap, not purely engine.
Re-run with matched codec effort before drawing a cost conclusion.

**Query latency (warm, min of 3)**

| Query | ClickHouse | GreptimeDB | Read |
| --- | --- | --- | --- |
| `count(), avg(duration_ms)` (full scan+agg) | 4 ms | 11 ms | both scan 1M rows |
| `count WHERE status='error'` (selective, off-key) | 3 ms | 9 ms | neither has status in key |
| `count WHERE trace_id=…` (point lookup) | **2 ms** | **16 ms** | **schema asymmetry** (below) |
| `GROUP BY service` | 4 ms | 12 ms | low-card group-by |

## What these numbers do and do not show

**Honest reading — ClickHouse won every query here, but interpret with care:**

1. **Scale is trivial (1M rows, ~30 MB).** The whole dataset is cache-resident.
   These are **fixed-overhead / minimum-latency floors**, NOT scan throughput.
   They cannot confirm or refute the at-scale scan claims (the interesting regime
   is GB–TB, cold cache). *Inconclusive at this scale* for the throughput claims.
2. **The `trace_id` lookup gap (2 ms vs 16 ms) is the predicted schema
   asymmetry, not raw engine speed.** ClickHouse's seed schema puts `trace_id`
   first in `ORDER BY`, so the sparse primary index seeks ~1 granule. GreptimeDB's
   seed schema keys on `(service, name)` with `trace_id` un-indexed, so it scans.
   This **confirms the pass-3 prediction** that trace-context retrieval is decided
   by key placement — and flags that the GreptimeDB Parallax schema must put
   `trace_id` in the primary key / add an index (feeds `greptimedb-implementation.md`).
3. **Measurement is only roughly comparable.** ClickHouse `--time` is native-client
   query time; GreptimeDB `execution_time_ms` is its own server-side figure over
   HTTP. Close enough to read direction at this scale, not for a precise ratio.

**Claims checked**

| Claim | Result | Status |
| --- | --- | --- |
| ClickHouse faster on selective/scan queries (pass 3 mechanism) | Faster here, but at cache-resident scale only | *inconclusive at this scale* (direction consistent) |
| Trace lookup decided by sort-key placement (pass 3) | 2 ms (keyed) vs 16 ms (un-keyed) | **confirmed** |
| ClickHouse smaller on-disk (codec breadth, pass 2) | 28.9 vs 38 MiB | *workload-specific* — tuned vs default codecs; re-run matched |
| GreptimeDB metric/PromQL advantage | not tested (no metrics signal in this run) | pending |
| Evidence-bundle large↔large join advantage | not tested | pending |

### Run 2 — 2026-05-25 — evidence-bundle join (Q1 + Q4), EXPLAIN plans

Same containers/dataset as Run 1, plus `logs` (214,287 rows) and `error_events`
(2,226 rows) generated to share the spans' `trace_id`/`span_id` (one real pair per
trace). ClickHouse `logs ORDER BY (service, ts)`, `error_events ORDER BY (project,
fingerprint, ts)`; GreptimeDB per seed DDL `PRIMARY KEY` equivalents.

**Correctness parity (anchor `trace_id` with an error): PASS**

| Query | ClickHouse | GreptimeDB |
| --- | --- | --- |
| Q1 `trace_context` (UNION spans+logs+error) | 18 rows (14+3+1) | 18 rows |
| Q4 `cross_tier` (spans LEFT JOIN error_events ON trace_id, span_id) | 14 rows, 1 matched error | 14 rows, 1 matched error |

**Query latency (warm, min of 3)** — same smoke-scale caveat as Run 1.

| Query | ClickHouse | GreptimeDB |
| --- | --- | --- |
| Q1 trace_context | 4 ms | 24 ms |
| Q4 cross_tier join | 3 ms | 54 ms |

**EXPLAIN plans — the real mechanism evidence (scale-independent):**

ClickHouse Q4 (`EXPLAIN actions=1`):

```text
Join (JOIN FillRightFirst)
Algorithm: SpillingHashJoin(ConcurrentHashJoin)
Clauses: [(trace_id, span_id) = (trace_id, span_id)]
  ReadFromMergeTree (default.spans)        Granules: 1   Prewhere: trace_id = '3fb2…'
  ReadFromMergeTree (default.error_events) Granules: 1   Prewhere: trace_id = '3fb2…'
```

GreptimeDB Q4 (`EXPLAIN`):

```text
SortPreservingMergeExec
  HashJoinExec: mode=Partitioned, join_type=Left, on=[(trace_id,trace_id),(span_id,span_id)]
    RepartitionExec: Hash([trace_id, span_id], 10)
      FilterExec: trace_id = '3fb2…'   <- MergeScanExec (spans region)
    RepartitionExec: Hash([trace_id, span_id], 10)
      FilterExec: trace_id = '3fb2…'   <- MergeScanExec (error_events region)
```

**What the plans confirm (and one correction to pass 3):**

1. **ClickHouse:** `FillRightFirst` + `SpillingHashJoin(ConcurrentHashJoin)`
   confirms the broadcast/concurrent-hash + grace-spill family from
   `clickhouse-internals.md`. The anchor `trace_id` became a **PREWHERE** and the
   **sparse index pruned to `Granules: 1`** on the spans side — empirical proof of
   the pass-3 PREWHERE + 8192-row-granule-skip mechanism.
2. **GreptimeDB:** `HashJoinExec: mode=Partitioned` + `RepartitionExec Hash(…,10)`
   confirms the **partitioned hash join** (repartition both sides) from pass 3 —
   the structure that scales to large↔large joins.
3. **Both engines propagate the anchor constant to BOTH join inputs.** ClickHouse
   pushed `trace_id='…'` to the `error_events` scan as a PREWHERE (`Granules: 1`);
   GreptimeDB pushed `FilterExec: trace_id='…'` to *both* region scans. **This
   corrects pass 3**, which implied ClickHouse's broadcast join must build the
   whole right table — for a *constant-anchored* join it does not; the optimizer
   propagates the equi-join constant and prunes both sides first.
4. **Consequence for Parallax (important):** the evidence-bundle queries (Q1–Q6)
   are **always anchored** on a specific `trace_id`/`fingerprint`. Both engines
   reduce each side to a tiny set *before* the join, so the join-algorithm
   difference (broadcast vs partitioned) is **largely irrelevant for Parallax's
   actual query pattern**. The "join strategy decides the winner" framing applies
   only to *un-anchored large↔large* joins, which Parallax does not run for bundle
   assembly. This downgrades the join from "the deciding factor" to "not a
   differentiator for the anchored pattern" — the **key placement** (so the anchor
   prunes cheaply) matters far more, which Run 1 already showed.

**Claims checked (Run 2)**

| Claim | Result | Status |
| --- | --- | --- |
| ClickHouse PREWHERE + sparse-index granule skip on key-anchored read | spans pruned to `Granules: 1` | **confirmed (plan)** |
| GreptimeDB uses partitioned hash join (pass 3) | `mode=Partitioned` in plan | **confirmed (plan)** |
| GreptimeDB pushes the anchor filter into region scans (pass 3) | `FilterExec` on both `MergeScanExec` inputs | **confirmed (plan)** |
| ClickHouse broadcast join must build whole right side (pass 3) | constant propagated → right side pruned to 1 granule | **contradicted (plan)** for anchored joins |
| Evidence-bundle join algorithm decides the winner | both prune before join on anchored queries | **refined**: not a differentiator for anchored Parallax queries |
| Cross-engine bundle correctness (Q1/Q4 identical) | 18 / 14 rows both | **confirmed** |

### Run 3 — 2026-05-25 — metrics signal + PromQL nativeness

Tests the operator hypothesis's strongest GreptimeDB claim: metrics + PromQL.
Dataset: 864,000 points, 1,200 series (12 services × 100 instances), one value
every 30 s over 6 h, a smooth random walk. Same containers.

- ClickHouse: `http_req_latency (ts DateTime64(3) CODEC(DoubleDelta,ZSTD),
  service LowCardinality, instance LowCardinality, value Float64 CODEC(Gorilla,
  ZSTD)) ENGINE=MergeTree ORDER BY (service, instance, ts)`.
- GreptimeDB: `http_req_latency (ts TIME INDEX, service, instance, value DOUBLE,
  PRIMARY KEY (service, instance))` — a plain time-series table.

**Correctness parity: PASS** — SQL 5-min range-aggregation grouped by service:
both return 864 groups (12 × 72 buckets); svc-0 first-bucket `avg(value)` =
**106.2274** on both.

**PromQL nativeness — the capability gap (most important result):**

| | GreptimeDB | ClickHouse |
| --- | --- | --- |
| Native PromQL | **Yes** — `GET /v1/prometheus/api/v1/query_range?query=avg by (service)(http_req_latency)` returned 12 series × 73 points directly over the plain table. | **No** — no PromQL engine. Must translate PromQL→SQL in an external layer. |
| Range query model | Native `query_range` (start/end/step) + PromQL functions (`rate`, `avg_over_time`, `… by (label)`). | Hand-written SQL with `toStartOfInterval` + `groupArray`/window funcs. |

This is a **capability difference, not just a speed delta**. If Parallax exposes
PromQL or ingests Prometheus remote-write, GreptimeDB does it natively; ClickHouse
requires building and maintaining a PromQL compatibility layer. **Confirmed** —
the clearest GreptimeDB advantage found so far, and it is on the metrics signal
exactly as the hypothesis predicted.

**Latency (warm, min of 3, smoke scale — indicative)**

| Query | ClickHouse | GreptimeDB |
| --- | --- | --- |
| SQL 5-min range-agg by service | 12 ms | 16 ms |
| Native PromQL `avg by (service)` (wall-clock incl. HTTP) | n/a | 48 ms |

GreptimeDB is **within ~1.3× of ClickHouse on the metric aggregation** — far
closer than the 2–3× gap it showed on log/trace scans (Run 1). Consistent with
metrics being GreptimeDB's design center. Still cache-resident scale; directional.

**Compression NOT meaningfully tested this run.** The ClickHouse `value` column
(Gorilla+ZSTD) compressed to 6.03 MiB for 864k float64 (~6.6 MB raw) — i.e. almost
no compression, because the synthetic random-walk values are high-entropy. Real
metrics (flat gauges, monotonic counters, repeated values) are exactly what
Gorilla/DoubleDelta target and compress 5–10×. **Re-run float compression with
realistic metric shapes** before any cost conclusion.

**Claims checked (Run 3)**

| Claim | Result | Status |
| --- | --- | --- |
| GreptimeDB PromQL-native; ClickHouse not | PromQL works on GreptimeDB, absent in ClickHouse | **confirmed** |
| GreptimeDB competitive/faster on metric aggregation | 16 ms vs 12 ms (within 1.3×) | *plausible* — directional, cache-resident scale |
| Gorilla codec shrinks float metrics | not exercised (incompressible synthetic data) | *inconclusive* — redo with realistic shapes |
| Cross-engine metric-aggregation correctness | 864 groups + 106.2274 both | **confirmed** |

### Run 4 — 2026-05-25 — per-signal compression (cost axis)

Measured retained size for all loaded tables (flushed / `OPTIMIZE FINAL`) plus a
realistic counter+gauge metric table. **Full analysis in
[`compression-and-cost.md`](compression-and-cost.md).** Headline: **no blanket
compression winner** — ClickHouse wins tuned counters (`DoubleDelta` 7.3×), flat
gauges (`Gorilla` 78×) and high-cardinality random strings (`spans` 28.9 vs 38
MiB); GreptimeDB wins dictionary-friendly low-card columns (`logs` 5.5 vs 10.24
MiB ⚠ synthetic) and high-entropy floats where Gorilla backfires
(`http_req_latency` 5.1 vs 6.31 MiB). Cost is closer to a tie than pass 2 implied;
object-store $ (MinIO) still unmeasured.

### Run 5 — 2026-05-25 — freshness + ingest throughput (axis #1)

Full analysis in [`write-path-and-ingestion.md`](write-path-and-ingestion.md).

- **Freshness = tie.** A single synchronous insert was **immediately queryable on
  both** engines (count=1 on the first query after ack); neither needs a
  flush/merge. Per-call ms (CH 288, GT 124) are client/HTTP overhead, not the
  mechanism — they do not rank freshness.
- **ClickHouse 26.x reports `async_insert=1` by default** (busy timeout 50–200 ms):
  small inserts auto-batch → visible after the buffer window, not instantly.
- **Bulk ingest:** ClickHouse 1M spans in 0.575 s (~1.74M rows/s, client wall) vs
  GreptimeDB 0.895 s (~1.12M rows/s, server time). Both >1M rows/s; inconclusive at
  smoke (different measurement bases, non-concurrent).
- **Mechanism difference that matters:** ClickHouse writes one part per INSERT →
  small high-frequency inserts risk "too many parts" → needs batching/async-insert;
  GreptimeDB's LSM memtable absorbs small writes natively. Favors GreptimeDB for
  streaming small-batch telemetry.

### Run 6 — 2026-05-25 — B2: GreptimeDB `trace_id INVERTED INDEX` validation

Tests `benchmarking-the-differences.md` B2: does adding `trace_id INVERTED INDEX`
to GreptimeDB spans close the Run-1 trace-lookup gap? Built `spans_idx` (same 1M
spans, `trace_id STRING INVERTED INDEX`, `append_mode`), flushed (index → Puffin),
re-measured `WHERE trace_id = ?` (warm, min of 3). Parity: 14 rows on all.

| Table | trace lookup | vs |
| --- | --- | --- |
| GreptimeDB `spans_idx` (INVERTED INDEX) | **8 ms** | the fix |
| GreptimeDB `spans` (no index, Run-1 baseline) | 14 ms | un-indexed |
| ClickHouse `spans` (`ORDER BY (trace_id, ts)`) | **2 ms** | sort-prefix seek |

**Reading (honest):** the inverted index **~halved** GreptimeDB's trace lookup
(14→8 ms) — the fix **helps and is confirmed directionally**. But it did **not**
reach ClickHouse parity (still ~4×). Since GreptimeDB's `execution_time_ms` is its
own *server-side* figure (excludes HTTP transport), the residual gap is **real
fixed query-setup overhead** (DataFusion planning + `MergeScanExec` region-scan
setup), not a measurement artifact — at 1M cache-resident rows that fixed floor
(~8 ms) dominates, below which an index cannot push. ClickHouse's leaner native
path floors lower (~2 ms).

**B2 status: partially confirmed.** Index helps; parity not reached *at smoke
scale*. The index's value (pruning) should matter more at larger scale where
actual scanning — not the fixed planning floor — dominates; **re-test at `small`+
and via the GreptimeDB MySQL native protocol** (lower per-query overhead than HTTP)
before concluding. Does not change the verdict (trace lookup is fast enough in
absolute terms — 8 ms — for anchored bundle assembly).

### Run 7 — 2026-05-25 — B9: small-write part behaviour (self-correction)

Tested `benchmarking-the-differences.md` B9: does ClickHouse's one-part-per-INSERT
cause part-explosion on small writes vs GreptimeDB's memtable? Drove 300 single-row
INSERTs (async_insert=0) into ClickHouse; 100 into GreptimeDB.

| Observation | Result |
| --- | --- |
| ClickHouse `NewPart` events (part_log) | **300** — confirms **one part per INSERT** |
| ClickHouse merge events | 61 — background merges ran concurrently |
| ClickHouse **active** parts after | **1** (300 → merged down) |
| `parts_to_throw_insert` default | **3000** |
| GreptimeDB 100 inserts | absorbed in memtable → 1 SST on flush (no per-insert files) |

**Self-correction to passes 9/14.** The mechanism is real (ClickHouse *does* create
one part per insert), **but background merges collapse bounded bursts aggressively**
(300 parts → 1 active), and the throw guard is far away (3000). So "too many parts"
is a **sustained-rate** failure — insert rate persistently exceeding merge
throughput — **not** a per-insert problem, and `async_insert` (default on in 26.x)
mitigates it further. My pass-9 framing overstated it.

**Refined claim:** GreptimeDB's memtable-absorption write-path advantage is **real
but narrower** — it matters for *sustained* high-frequency tiny writes that outpace
ClickHouse's merge rate (where ClickHouse needs async-insert/batching tuning and
GreptimeDB does not). For bounded/moderate small-write bursts, ClickHouse copes via
merges + async insert. Confirming the *sustained* failure needs a rate-ramp test
(insert faster than merges keep up until 3000) — proposed for the harness.

**B9 status: done, refined** (mechanism confirmed; severity downgraded to a
sustained-rate concern).

### Run 8 — 2026-05-25 — B10 (partial): GreptimeDB object storage on MinIO

First object-storage run. Stood up MinIO + bucket `greptimedb` on an isolated
network; ran a GreptimeDB `v1.0.2` standalone with `[storage] type = "S3"`,
`endpoint = http://…minio:9000`, path-style, against MinIO; ingested the 1M spans,
flushed. (Config via `docker create` + `docker cp` + `docker start` — bind-mounts
don't reach the orbstack daemon.)

| Observation | Result |
| --- | --- |
| GreptimeDB-S3 startup | clean — logs confirm `store: S3(bucket: greptimedb)`; healthy in ~4 s |
| Ingest 1M spans → flush | OK (COPY 950 ms server-side), 1,000,000 rows queryable |
| **MinIO footprint** | **36 MiB across 4 objects** |
| vs local-disk SST (Run 1) | 38 MiB — **no object-storage size penalty** (same Parquet SST) |

**Findings (cost axis #2):**

1. **GreptimeDB object-store-native is real and clean** — one `[storage]` block,
   data lands in S3 directly as Parquet. Empirically confirms the verdict's
   "object-store-native" claim (vs ClickHouse's S3-disk-under-a-policy).
2. **Few, large objects (4 for 1M rows)** → **request-efficient on S3**: fewer
   GET/PUT/LIST, so lower per-request cost amplification — the thing that dominates
   object-store bills for a re-read-heavy engine (`retention-and-ttl.md`).
   ClickHouse Wide parts store **one object per column per part** → many more,
   smaller objects → more requests; this is the contrast to measure next.

**B10 status: partial.** GreptimeDB side done. **Still owed:** ClickHouse `s3`
disk + storage-policy run on the same MinIO (object count + bytes), and actual
GET/PUT/LIST counts (MinIO audit log / `mc admin trace`) during ingest and during
cold-cache Q1–Q6 — the real request-cost comparison. Cold-read egress too.

### Run 9 — 2026-05-25 — B10 complete: ClickHouse vs GreptimeDB object layout on MinIO

Stood up a ClickHouse `v26.5.1.882` with an `s3` disk + `storage_policy='s3only'`
against the **same MinIO**, loaded the same 1M spans, `OPTIMIZE FINAL`. Compared
the object layout to GreptimeDB-S3 (Run 8).

| Engine | Objects in S3 | S3 bytes used | Active logical bytes |
| --- | --- | --- | --- |
| **GreptimeDB** | **4** | 37 MiB | 37 MiB |
| **ClickHouse** | **74** | 63 MiB | 31.82 MiB (1 Wide part) |

**Findings (cost axis #2 — the decisive object-store result):**

1. **Object count: GreptimeDB 4 vs ClickHouse 74 (~18×).** ClickHouse's Wide part
   stores **one S3 object per column** (+ marks + metadata), so a single table
   becomes dozens of objects; GreptimeDB writes a few large Parquet objects. **This
   is the object-store-economics advantage**: per-request pricing dominates an
   object-store bill, and a cold read in ClickHouse must issue **many more S3 GETs**
   (one per needed column file) than GreptimeDB's few-Parquet-file reads. Confirms
   the verdict's "object-store-native" claim with a hard number.
2. **Size nuance — a partial reversal.** Active logical data: ClickHouse 31.82 MiB
   < GreptimeDB 37 MiB (ClickHouse's tuned spans codecs win on the high-card hex
   columns, consistent with Run 1's local result). But ClickHouse's **raw S3 usage
   was 63 MiB** — nearly 2× its logical — because pre-`OPTIMIZE` merge parts' S3
   objects are **not yet garbage-collected** (ClickHouse S3 cleanup is async). So
   ClickHouse on object storage carries **transient space amplification** from
   merge garbage until cleanup runs — an operational cost GreptimeDB's LSM-flush
   model largely avoids.

**B10 status: done** (object layout + footprint). Remaining refinement: actual
GET/PUT/LIST **request counts** during cold-cache Q1–Q6 (MinIO audit / `mc admin
trace`) to quantify the request-cost gap the 4-vs-74 object split implies — but the
object-count proxy already shows the direction decisively.

Cleanup: the MinIO + GreptimeDB-S3 + ClickHouse-S3 containers and `pbench-s3`
network are torn down after this run (ephemeral; nothing committed).

### Run 10 — 2026-05-25 — B7: realistic-cardinality log-text compression

Re-ran log compression with **realistic high-entropy text** (500k rows, **99%
unique messages**: templated with embedded UUIDs/IDs/latencies + stack-trace
lines), fixing Run 4's synthetic-cardinality distortion (Run 4 had 10 distinct
messages).

| Schema | Total on disk | Notes |
| --- | --- | --- |
| GreptimeDB `logs_real` (default ZSTD-all) | **25 MiB** | Parquet + table-wide ZSTD |
| ClickHouse `logs_real` (only `message` ZSTD; ids default **LZ4**) | 35.53 MiB | trace_id 15.3M + span_id 7.7M dominate (LZ4 on hex) |
| ClickHouse `logs_real_z` (**ZSTD on all string cols**) | **24.24 MiB** | trace_id 15.3→7.8M, span_id 7.7→3.9M |

**Finding — corrects both earlier framings:**

- Run-4's GreptimeDB logs win was **not** purely a synthetic artifact: with
  realistic 99%-unique text GreptimeDB **still wins at defaults** (25 vs 35.5 MiB).
- **But the win is a default-codec effect, not engine superiority.** ClickHouse's
  per-column default is **LZ4**; the high-cardinality hex `trace_id`/`span_id`
  columns compress poorly under LZ4. Switching them to ZSTD drops ClickHouse to
  **24.24 MiB ≈ GreptimeDB's 25 MiB** — essentially a **tie when both tuned**.
- **Operational nuance:** GreptimeDB ZSTDs everything automatically → good log
  compression with **zero tuning**; ClickHouse needs explicit per-column `CODEC(ZSTD)`
  on high-card columns to match (its default LZ4 leaves ~30% on the table here).

**B7 status: done.** Realistic-log compression is a **tie at matched effort**,
**GreptimeDB-favored out-of-the-box**. Reinforces the pass-8 "compression is a
tuning-dependent wash" conclusion with realistic data, plus the defaults nuance.

### Run 11 — 2026-05-25 — B5: high-cardinality metrics (40k series, 8M rows)

Re-ran the metric path at the prototype's real cardinality (40 services × 1000
instances = **40,000 series**, 200 points each = 8M rows), vs Run 3's 1,200 series.
Plain time-series table on both (not the metric engine / PromQL path).

| Measure | ClickHouse | GreptimeDB |
| --- | --- | --- |
| Bulk ingest 8M rows | 0.669 s (~12M rows/s) | 2.98 s (~2.7M rows/s) |
| Retained size | 57.42 MiB | 62 MiB |
| **`avg by service`, 5-min buckets (SQL group-by)** | **65 ms** | **638 ms (~10×)** [⚠ superseded — see Run 37: warm steady-state is ~2× (CH 50 / GT 107 ms); the 638 ms was cold/first-run] |
| single-series lookup | 3 ms | 9 ms |

**Significant refinement of the metrics finding.** At 1,200 series (Run 3) the SQL
range-aggregation was a near-tie (16 vs 12 ms); at **40k series / 8M rows it is
~10× in ClickHouse's favour** (65 vs 638 ms), and ClickHouse ingested ~4.5× faster.
This is **predicted by the internals** — ClickHouse's decade-tuned vectorized C++
group-by over a columnar scan is the throughput bar (`clickhouse-internals.md`),
and Run-3's near-tie was a **small-scale / cache-resident artifact** (the
fixed-overhead floor, not throughput). At real volume the scan-aggregate engine
gap shows.

**Consequence (sharpens the verdict's metrics pillar):** GreptimeDB's metrics
advantage is **PromQL-nativeness + native ingest (capability), NOT aggregation
speed at volume.** For heavy metric *analytics* at scale, ClickHouse is materially
faster (~10×) — it just can't speak PromQL. So "metrics → GreptimeDB" holds **only
on the capability/ingest axis**, not on raw query latency at volume.

Caveat: this is the SQL group-by (ClickHouse's core strength), not GreptimeDB's
native PromQL planner or the metric engine (logical→physical) — a PromQL-path run
+ the metric engine could differ and is owed. But for SQL-shape metric aggregation,
the volume result is clear. Also a **preview of B1**: at 8M rows ClickHouse's scan
engine already shows ~10×; the cold GB–TB log/trace scan likely shows it larger.

**B5 status: done** (SQL aggregation); PromQL-path + metric-engine high-card run owed.

### Run 12 — 2026-05-25 — B1 (medium tier, warm): log full-text search + scan

The verdict's flip-trigger, at medium volume. 5M realistic logs (99%-unique
messages) loaded into **both with their text indexes** — ClickHouse native `text`
index (`tokenizer='splitByNonAlpha'`), GreptimeDB `FULLTEXT INDEX` (English
analyzer). Parity exact: `timeout` token = **698,955** both; `svc-3`+`ERROR` =
**49,679** both.

| Query | ClickHouse | GreptimeDB | Gap |
| --- | --- | --- | --- |
| **Full-text token search** (`hasToken`/`matches` 'timeout') | **7 ms** | **130 ms** | **~18× ClickHouse** |
| Selective filter (`service` + `level`) | 4 ms | 5 ms | **~tie** |
| Full count-by-`level` (scan) | 7 ms | 28 ms | ~4× ClickHouse |

**Findings (decisive for the flip-trigger):**

1. **ClickHouse wins log full-text search ~18×**, *even with both engines using
   their text indexes*. ClickHouse's mature `text` posting-list index + vectorized
   `hasToken` far outruns GreptimeDB's `FULLTEXT` (Puffin) + DataFusion `matches()`
   at 5M rows. This is the **dominant-signal flip-trigger query**, and ClickHouse's
   advantage is large and real — confirming the verdict's trigger: *if Parallax's
   query mix is dominated by ad-hoc log search at volume, ClickHouse wins decisively.*
2. **Selective keyed filter is a tie** (4 vs 5 ms): when the filter hits indexed/
   low-card columns (`service` PK prefix, `level`), GreptimeDB prunes as well as
   ClickHouse. Anchored/keyed access — Parallax's actual bundle pattern — does not
   show the gap.
3. **Full scan ~4×** (consistent with B5's ~10× at 8M metric rows): ClickHouse's
   vectorized engine widens with volume.

**Consequence:** the decision genuinely hinges on Parallax's real query mix.
*Anchored bundle assembly* (trace_id/fingerprint lookups + keyed filters) → both
fine, GreptimeDB's fit pillars win. *Heavy ad-hoc full-text log search at volume*
→ ClickHouse ~18×, the flip-trigger fires. Parallax is designed around anchored
evidence bundles, so the verdict holds — but this is the number that would flip it.

**B1 status: done at medium-warm.** True cold-cache GB–TB (drop OS page cache,
25–50 GB) would likely widen the scan/search gaps further; owed to the full
harness. Caveat: 5M rows still largely cache-resident — the 18× search gap is an
index-implementation difference, not just scan throughput.

### Run 13 — 2026-05-25 — B8: concurrent ingest + query penalty (axis #1 gate)

Tests the prototype's **concurrent-penalty gate** (query p95 under mixed load ≤ 2×
query-only). Seeded 3M rows each, ran an `avg by s` aggregation 5× as baseline,
then again while a background loop ingested ~8M more rows (3M → 11M during the
query window).

| Engine | Query-only baseline | Under concurrent ingest | Penalty | Gate (≤2×) |
| --- | --- | --- | --- | --- |
| ClickHouse | 11 ms | 17 ms | **1.55×** | **PASS** |
| GreptimeDB | 66 ms | 91 ms | **1.38×** | **PASS** |

**Findings:**

1. **Both pass the concurrent-penalty gate** — neither blocks reads on heavy
   concurrent ingest (ClickHouse atomic part visibility + background merges;
   GreptimeDB MVCC `Version` snapshot + memtable). GreptimeDB's penalty *ratio* was
   slightly lower (1.38× vs 1.55×). Both stayed queryable while ingesting 8M rows.
2. **Absolute agg latency at 11M rows: ClickHouse ~5× faster** (17 vs 91 ms) — the
   same vectorized-engine-at-volume gap as B5/B1, not a concurrency effect.
3. **Freshness held under load**: both served queries continuously while row counts
   grew 3M→11M; visible-on-write was not disrupted by concurrent reads.

**B8 status: done (within-engine penalty).** The mixed-load *freshness p95*
(stamp-emit → poll-visible under load, the other half of the gate) needs the
harness's freshness instrumentation for a precise sub-second number; the penalty
ratio + continuous visibility here already show neither engine has a concurrent
read-blocking problem. Caveat: cache-resident scale + docker-exec measurement
coarseness — directional.

### Run 14 — 2026-05-25 — B10/B12 partial: cold-read S3 GET count (anchored lookup)

Using the now-committed `bench/s3/` stack: loaded 1M spans into both S3-backed
engines, **cleared the local read caches + restarted** (forced cold S3 reads), then
counted `s3.GetObject` via `mc admin trace` during a **cold anchored `trace_id`
lookup** (14 spans). Both returned 14 (parity).

| Engine | Cold `s3.GetObject` for one anchored trace lookup |
| --- | --- |
| ClickHouse (`ORDER BY (trace_id, ts)`) | **5** |
| GreptimeDB (`trace_id INVERTED INDEX`) | **22** |

**Partial correction to B10's inference.** B10 measured the *total object count*
(GreptimeDB 4 vs ClickHouse 74) and I inferred GreptimeDB would issue fewer cold S3
requests. **For an *anchored keyed lookup* the opposite is true** (CH 5 < GT 22),
and the mechanism is clear:

- **ClickHouse** physically **clusters data by `trace_id`** (`ORDER BY` prefix), so
  the sparse index pinpoints ~1 granule and the cold read is a handful of ranged
  GETs into the relevant column files.
- **GreptimeDB** keys on `(service,name)` with `trace_id` as a **secondary inverted
  index** → it must GET the SST footer + the **Puffin index objects** + the column
  pages + manifest = ~22 ranged GETs (index indirection + more round-trips).

So **object-store request cost is query-shape-dependent**:

- **Anchored point/keyed lookups** (Parallax's evidence-bundle pattern) → **ClickHouse
  issues fewer cold GETs** (sort-key locality beats index indirection). This
  **counters** the earlier "GreptimeDB is object-store request-efficient" reading
  *for the anchored case*.
- **Full-scan / wide cold reads** (JSONBench-style) → GreptimeDB's **few large
  objects** win (fewer objects to touch for a scan) — consistent with the JSONBench
  cold-run claim (B12).

**Bounding caveat:** GreptimeDB's **read cache** (which I deliberately evicted here)
means warm re-reads are **local (0 S3 GETs)** for both engines — so the 5-vs-22 cold
gap only bites on genuinely cache-cold reads; Parallax's hot/recent bundles stay
cached. One measurement, 1M-span SST, single trace — directional, not a law.

**B10 status: extended (request counts done for the anchored case).** **B12** (full-
scan/JSONBench cold reads, where GreptimeDB is expected to win on object count) still
owed — needs the wide/JSON dataset; the stack is ready (`bench/s3/`).

### Run 15 — 2026-05-25 — B12 (local): cold full-scan S3 GET count

Companion to Run 14 (anchored). Same S3 stack + 1M spans, cold caches; counted
`s3.GetObject` during a cold **full-scan** query (`count`, `avg(duration_ms)`,
`uniq(service)` over all 1M rows). Parity: both returned 1,000,000 / 24.96 / 12.

| Query shape (cold) | ClickHouse `s3.GetObject` | GreptimeDB `s3.GetObject` | Fewer |
| --- | --- | --- | --- |
| **Anchored keyed lookup** (Run 14) | 5 | 22 | **ClickHouse** |
| **Full scan** (Run 15) | 57 | 26 | **GreptimeDB** |

**This completes the cold object-store request-cost story — it splits cleanly by
query shape:**

- **Anchored / keyed lookup** → **ClickHouse fewer GETs** (data clustered by
  `ORDER BY` key → sparse index pinpoints ~1 granule → minimal ranged reads).
- **Full scan** → **GreptimeDB fewer GETs** (few large Parquet objects → fewer S3
  round-trips than ClickHouse's many per-column-file objects). **This locally
  confirms the JSONBench cold-run mechanism** (`public-performance-claims.md` #6):
  GreptimeDB's object layout wins cold full-scan/wide reads.

**Consequence for Parallax:** its dominant retrieval is **anchored** evidence-bundle
assembly → on the cold object-store request-cost sub-axis, **ClickHouse is the
better fit for the anchored pattern** (5 vs 22), while GreptimeDB wins the
scan-heavy regime (dashboards over wide windows, JSONBench) it does less of. Bounded
by: the read cache makes warm re-reads local (0 GETs) on both, so this only bites
genuinely cold reads. One measurement each, 1M-span SST — directional. B12's local
full-scan question is answered; the 1B-doc JSONBench scale stays the prototype's.

### Run 16 — 2026-05-25 — Q6 evidence-bundle composite (the query that matters most)

Completed the end-to-end evidence-bundle measurement (Run 2 did Q1/Q4 separately;
Q2/Q3/composite were untimed). Anchor: `fingerprint=fp-000`, `release=v1.7.0`,
`trace_id=3fb2d84c…`, prior release `v1.6.0`. **Parity PASS**: Q1=18 rows, Q2
count=11 (same first/last-seen instants), Q3=38 regression fingerprints — identical
on both.

| Sub-query | ClickHouse | GreptimeDB |
| --- | --- | --- |
| Q1 trace_context (3-way UNION spans+logs+errors) | 4 ms | 24 ms |
| Q2 issue_history (`min/max/count` by project+fingerprint) | 3 ms | **3 ms (tie)** |
| Q3 release_regression (`NOT IN` anti-join) | 3 ms | 6 ms |
| **Q6 composite (sum)** | **~10 ms** | **~33 ms** |

**Findings:**

1. **Both assemble the full bundle correctly and fast** — ~10 ms (CH) / ~33 ms (GT)
   at 1M-span smoke, **both far under the prototype's Q6 ≤300 ms warm gate**.
2. **Q2 issue-history is a tie** (3 ms each): `(project, fingerprint)` is
   GreptimeDB's PRIMARY KEY prefix = ClickHouse's `ORDER BY` prefix → both do a
   fast keyed lookup. Confirms the anchored/keyed pattern is not latency-bound on
   either engine.
3. **GreptimeDB's gap is concentrated in Q1** — the 3-way UNION pays GreptimeDB's
   per-query fixed overhead (DataFusion planning + HTTP) ×3 sub-scans; it is **not**
   algorithmic (Q2 tie, Q3 close). At larger scale the keyed sub-queries stay cheap
   (anchored), so the composite should remain bounded.

**Consequence:** for Parallax's **single most important query** (assemble the
evidence bundle from an anchor), **engine choice is not latency-bound** — both are
fast and well within gate. This confirms the verdict's core point: the decision
rests on the *fit* pillars (metrics-native, ingest ergonomics, cost, scaling), not
on bundle-assembly speed. (Smoke scale; warm. The composite at `small`+ cold and
under concurrent ingest is the prototype's to settle.)

### Run 17 — 2026-05-25 — TTL eviction cost: rewrite-survivors vs whole-file drop

Confirms the `retention-and-ttl.md` mechanism (pass 36) with measured numbers. Env:
same pinned stack (GreptimeDB `v1.0.2`, ClickHouse `v26.5.1.882`), laptop smoke,
isolated throwaway tables. Loaded one mixed part/region of 1M (CH) / 20 (GT) rows,
half/all expired, forced eviction, read the engine's own accounting.

**ClickHouse — `system.part_log` (the headline, quantified).** One mixed part (1M
rows, half 5-days-old vs `TTL ts + INTERVAL 1 DAY`), default vs tuned table:

| table | TTL event (`merge_reason`) | read_rows | result_rows | read | written |
| --- | --- | --- | --- | --- | --- |
| `ret_default` (default `ttl_only_drop_parts=0`) | **`TTLDeleteMerge`** | **1,000,000** | **500,000** | 114 MiB | **50 MiB** |
| `ret_drop` (`ttl_only_drop_parts=1` + `PARTITION BY toYYYYMMDD`) | **`TTLDropMerge`** | 16,384 | **0** | 1.9 MiB | **572 B** |

→ Default TTL **read the whole 1M-row part and rewrote the 500k survivors** (50 MiB
written) just to evict the other half — measured write-amplification. Tuned dropped
the expired *partition* whole: `read_rows`=16,384 is a single granule (metadata),
`result_rows`=0, nothing rewritten. ClickHouse's own `merge_reason` enum names the
two paths (`TTLDeleteMerge` = rewrite vs `TTLDropMerge` = whole-part drop) — exactly
the pass-36 split, now numeric.

**GreptimeDB — whole-SST drop + multi-stage TTL filter.** With `ttl='5s'`: insert 20
rows → `ADMIN flush_table` → **1 SST** on disk → wait 7s (rows age out) →
`ADMIN compact_table` → **0 SSTs** (the Parquet file physically deleted; `count(*)`=0).
No rewritten/merged file appears — the expired SST is *dropped*, not re-emitted.
Separately, with `ttl='1d'` + 5-days-old rows: the old rows were **never queryable**
(`SELECT` returned only fresh rows *before* any compaction) **and never persisted to
a durable SST** (flush of already-expired rows produced no SST), and the surviving
fresh SST was **byte-identical** (same filename + 2877 B) before and after compaction
— i.e. no rewrite. So GreptimeDB applies TTL at **three** points: read-path filter
(immediate), flush (skips already-expired rows), and compaction (whole-SST physical
drop). Only the last reclaims storage; the first two are free.

**Two refinements to pass 36:**

1. **ClickHouse `merge_with_ttl_timeout`=4h is a *repeat* floor, not an initial
   delay.** The first TTL eviction fired within seconds of insert (the merge selector
   picked it up immediately); the 4h only throttles *re-checking the same data*. So
   "≥4h granularity" was too pessimistic — first eviction is prompt.
2. **GreptimeDB's TTL is cheaper than even "whole-SST drop" implies**: already-expired
   data is filtered at read and dropped at flush, so it often costs *zero* durable
   writes — the compaction drop only handles data that aged out *after* being written.

**Claim status:** pass-36 retention mechanism → **confirmed (measured)**. Default
ClickHouse TTL = rewrite-survivors write-amp; tuned = whole-part drop; GreptimeDB =
whole-SST drop with no rewrite. Cost-axis (#2) retention sub-cell: GreptimeDB cheap by
default, ClickHouse cheap **iff** `PARTITION BY` time + `ttl_only_drop_parts=1`.
(Smoke scale; the write-amp *magnitude* at production volume + sustained churn is the
prototype's to settle.)

### Run 18 — 2026-05-25 — Schema evolution: auto-add vs ALTER vs JSON

Backs `schema-evolution-and-dynamic-columns.md` (pass 38). Same pinned stack, smoke.

**ClickHouse** (`se_test`, 1M-row part):

- `ALTER TABLE … ADD COLUMN b String DEFAULT 'x'` → **0.005 s**; part `all_1_1_0`
  byte-identical (3.85 MiB) + same `modification_time` before/after → **metadata-only,
  no rewrite** (matches `AlterCommands.cpp` `isRequireMutationStage`=false).
- `INSERT … (ts,a,c)` with undeclared `c` → **server exception** (no schema-on-write).
- `JSON` column: inserted `{k1:1}`, `{k2:"v",k3:true}` → `JSONAllPathsWithTypes` =
  `('k1','Int64'),('k2','String'),('k3','Bool')` (each path a **typed subcolumn**);
  `attributes.k2` returns `v` reading only that subcolumn.

**GreptimeDB** (`weather`, InfluxDB line protocol):

- write `weather,location=us temp=82` → table `(location, temp, greptime_timestamp)`.
- write `weather,location=us,city=nyc temp=80,humidity=30,wind=5` → **auto-added
  `city`(tag→PK), `humidity`,`wind`(field→DOUBLE)**; first row reads `NULL` for them
  (schema-on-read, no rewrite). Confirms `create_or_alter_tables_on_demand`.
- `Json` column: `DESC` = `attrs Json`; queried `json_get_string(attrs,'k2')` →
  per-row blob parse (single binary column, not per-path subcolumns).

**Claim status:** both `ADD COLUMN` metadata-only → **confirmed**; GreptimeDB
schema-on-write auto-evolution → **confirmed (live)**; ClickHouse no-auto-schema →
**confirmed**; JSON storage models (CH columnar subcolumns vs GT binary blob) →
**confirmed**. Ingest-ergonomics edge GreptimeDB; dynamic-attr path-query edge
ClickHouse. Smoke; column-explosion threshold + JSON query speed at volume owed.

### Run 19 — 2026-05-25 — Dedup/update semantics: read-time vs merge-time

Backs `dedup-and-update-semantics.md` (pass 39). Same pinned stack, smoke.

**GreptimeDB — read-time dedup (always correct, no compaction forced):**

- `merge_mode=last_row` (default): `(k='A',ts=1000)` inserted v=1 then v=2 → plain
  `SELECT` = **1 row, v=2**.
- `merge_mode='last_non_null'`: partial writes `(v1=1)` then `(v2=2)` at same key/ts →
  plain `SELECT` = **1 row, v1=1 AND v2=2** (per-field merge).

**ClickHouse — `ReplacingMergeTree(ver)` merge-time dedup:**

- key=1 inserted ver=1 then ver=2 = **2 parts**.
- plain `SELECT` → **2 rows** (`old`,`new`) — duplicates visible, not yet merged.
- `SELECT … FINAL` → **1 row** (`new`, ver=2 wins) — dedup forced at read.
- `OPTIMIZE TABLE … FINAL` then plain `SELECT` → **1 row** (collapsed).
- Timing plain vs FINAL both 0.002 s at 2 rows — FINAL cost only bites at scale
  (many covering parts); not a smoke signal.

**Claim status:** GreptimeDB dedup at read (DedupReader in scan path) → **confirmed
(live)**; ClickHouse dedup eventual/merge-time, dupes visible without `FINAL` →
**confirmed (live)**. Consequence: latest-state queries (issue status, deploy marker,
metric last-value) correct-by-default on GreptimeDB; ClickHouse needs `FINAL` or
`argMax`/`AggregatingMergeTree`. Append signals: dedup moot (GT `append_mode` / CH
plain `MergeTree`). FINAL-vs-read-dedup cost crossover at volume owed to harness.

### Run 20 — 2026-05-25 — Durability defaults (live config confirmation)

Backs `wal-and-durability.md` (pass 41). Not a latency benchmark — empirical
confirmation of the durability-relevant defaults on the running pinned servers.

**ClickHouse** (`system.merge_tree_settings` / `system.settings`):

- `fsync_after_insert = 0`, `fsync_part_directory = 0` → inserted parts are **not
  fsynced** (page cache only).
- `async_insert = 1`, `wait_for_async_insert = 1` → ack waits for the buffer to flush
  to a part, but the part is not fsynced. (`wait=0` would ack before the part exists.)
- MergeTree has **no WAL** (`in_memory_parts_enable_wal` etc. obsolete in 26.x).

**GreptimeDB** (running standalone filesystem):

- `…/wal/0000000000000001.raftlog …` segments ~128–137 MiB each → **local raft-engine
  WAL is active**; segment size matches `file_size`=128 MiB default.
- Source default `sync_write = false` → not fsynced per write either, but the WAL is a
  **replayable** log (crash recovery replays it); ClickHouse has no replay log.

**Claim status:** both default to throughput-over-strict-fsync → **confirmed**;
GreptimeDB has a replayable WAL (local raft-engine; Kafka remote decouples durability
from the datanode) while ClickHouse relies on part-on-disk + replication →
**confirmed**. Durability + scaling edge GreptimeDB; strict-durability perf cost
(`sync_write=true` vs `fsync_after_insert=1`) owed to harness.

### Run 21 — 2026-05-25 — Execution-engine config (live confirmation)

Backs `query-execution-engine.md` (pass 42). Live settings, not a latency benchmark —
the engine knobs behind the Run 11/12 throughput gaps.

**ClickHouse** (`system.settings`): `max_block_size = 65409` (≈65536, ~8× DataFusion's
batch), `max_threads = auto(10)` (per-core pipeline lanes), `compile_expressions = 1`
+ `compile_aggregate_expressions = 1` (LLVM JIT on, `min_count_to_compile_expression =
3`), `max_bytes_before_external_group_by = 0` (in-memory aggregation).

**GreptimeDB**: DataFusion `=52.1` (Cargo); `SessionConfig.with_target_partitions(...)`
+ custom `ParallelizeScan` rule; default Arrow batch 8,192. EXPLAIN of `GROUP BY
service` → `CooperativeExec → MergeScanExec` (scan+aggregate pushed into the region
via DataFusion).

**Claim status:** "decade-tuned C++ vectorized engine" → **confirmed concrete**:
8× larger vectors + JIT expressions/aggregation + bespoke SIMD kernels + specialized
hash aggregation explain ClickHouse's scan/aggregate throughput lead (Runs 11–12).
GreptimeDB trades raw kernel speed for DataFusion extensibility (PromQL, metric
engine). Anchored Q6 stays not-throughput-bound (Run 16). Isolated micro-benchmark of
each knob owed to harness.

### Run 22 — 2026-05-25 — Index file formats (live confirmation)

Backs `indexing-internals.md` (pass 43). On-disk index format check, smoke.

**GreptimeDB** (table with `INVERTED`+`FULLTEXT`+`SKIPPING` index, flushed): the SST
produced a **`.puffin` sidecar with the same UUID as the `.parquet`** —
`6e4627ae….parquet` + `6e4627ae….puffin`. All indexes live as named blobs in that one
Puffin file (`greptime-inverted-index-v1` FST+roaring, `greptime-fulltext-index-v1`
tantivy / `-bloom`, `greptime-bloom-filter-v1`).

**ClickHouse** (table with `bloom_filter`+`tokenbf_v1`+`set` skip indexes): per-part
files `primary.cidx` (sparse primary) + **one `skp_idx_<name>.idx` + `.cmrk4` per skip
index** (`skp_idx_i_tid.idx` 530 B, `skp_idx_i_msg.idx` 3.79 KiB, `skp_idx_i_lvl.idx`
37 B). `GRANULARITY N` = coarse, one entry per N×8192-row granules.

**Claim status:** GreptimeDB's index *toolkit* is richer/more precise (FST+roaring
inverted = true secondary index; tantivy = Lucene-class full-text) → **confirmed**;
ClickHouse skip indexes are coarse granule-pruners → **confirmed**. **Paradox
reconciled:** richer index ≠ faster — ClickHouse still won full-text ~18× (Run 12) and
anchored lookup (Run 6) because *index↔vectorized-scan integration + sort-key locality*
dominate index-format richness (ties `query-execution-engine.md`). Not a verdict flip;
corrects the tempting "richer index → faster" inference. Index-build cost + cold-scale
search latency owed to harness.

### Run 23 — 2026-05-25 — PromQL capability re-verification (verdict-material)

Backs `promql-and-metrics-query.md` (pass 44). Re-checked the verdict's load-bearing
"ClickHouse has no PromQL" claim against the pinned 26.5.1.882 — **it is now outdated.**

**ClickHouse 26.5 (live):** has PromQL. `system.table_functions` lists
`prometheusQuery`, `prometheusQueryRange`, `timeSeriesSelector/Metrics/Data/Tags`;
`system.table_engines` lists **`TimeSeries`**. `CREATE TABLE … ENGINE=TimeSeries`
succeeded with `allow_experimental_time_series_table=1`. `prometheusQuery('up')`
exists with a real 3–4 arg signature (`[db,] ts_table, promql [, eval_time]`).
Settings present: `allow_experimental_time_series_table=0` (default),
`allow_experimental_time_series_aggregate_functions=0`, `promql_database`/
`promql_table`/`promql_evaluation_time=auto`. → **experimental, off by default,
setup-heavy (dedicated TimeSeries table + remote-write).**

**GreptimeDB (live):** PromQL GA + default-on. `/v1/prometheus/api/v1/query?query=up`
→ proper Prometheus JSON, zero setup. `TQL EXPLAIN rate(spans[5m])` invoked the native
`prom_rate` planner (errored only on a column *type*, proving the path is live).
Custom DataFusion plan nodes (`InstantManipulate`/`RangeManipulate`/`SeriesNormalize`/
`SeriesDivide`/`HistogramFold`/`Absent`/…).

**Claim status:** "ClickHouse has no PromQL" → **REFUTED at 26.x** (experimental
PromQL exists). Re-rated: GreptimeDB's metrics win is now **maturity/ergonomics
(GA, default-on) vs experimental**, not present-vs-absent. Verdict/per-signal/
write-path corrected. Does **not** flip the recommendation; narrows a pillar.
Feature-completeness of ClickHouse PromQL vs Prometheus unverified — follow-up case.

### Run 24 — 2026-05-25 — PromQL maturity, end-to-end (follow-up to Run 23)

Backs `promql-and-metrics-query.md` (pass 45). Turned "ClickHouse PromQL exists"
(Run 23) into "how usable" by running it end-to-end. Smoke.

**ClickHouse `TimeSeries` + `prometheusQuery`:** `CREATE TABLE … ENGINE=TimeSeries`
exposes a flat view (id/timestamp/value/metric_name/tags) over 3 inner tables
(data/tags/metrics, `AggregatingMergeTree`/`ReplacingMergeTree`). But:
- `INSERT INTO <ts>` → **"INSERT is not supported by storage TimeSeries yet"**.
- `SELECT … FROM <ts>` → **"SELECT is not supported by storage TimeSeries yet"**.
- Ingest is **Prometheus-remote-write only**; query is **table-function only**.
- `prometheusQuery(pm,'http_requests_total',now())` and
  `prometheusQueryRange(pm,'rate(http_requests_total[2m])',start,end,30)` **parsed +
  executed with no error** (returned empty — hand-loaded the inner `.data` table but
  the id-coupled `.tags`/`.metrics` were empty, so no series resolved; there is no
  practical hand-load path without a remote-write client).

**GreptimeDB, same counter:** InfluxDB-line write auto-created `http_requests_total`
(job tag, value, ts); `TQL EVAL (start,end,'30s') rate(http_requests_total[2m])`
returned **real values** (`0.72`, `1.17` for `job=api`) via native `prom_rate`.

**Claim status:** sharpens Run 23. PromQL *capability* present on both; **maturity/
ergonomics gap large** — ClickHouse: experimental, remote-write-only ingest,
table-function-only query, no INSERT/SELECT ("yet"). GreptimeDB: GA, multi-protocol
ingest, PromQL+SQL+TQL, any metric table, real result with zero ceremony. Verdict
metrics pillar = maturity/ergonomics lead (confirmed concretely), not present-vs-absent.

### Run 25 — 2026-05-25 — OTLP ingest re-verification (claim holds, no drift)

Backs `write-path-and-ingestion.md` (pass 46). After PromQL drifted (Run 23), re-checked
the sibling claim "ClickHouse needs an OTLP collector" against pinned 26.5.1.882.

**ClickHouse 26.5:** **no native OTLP receiver.** `system.table_functions` /
`system.functions` have **no** `otlp`/`otel`/`opentel` entry; `src/Server` source has
**no** OTLP HTTP handler. OTLP ingest still requires the OTel Collector + ClickHouse
exporter (or a bundled collector). → claim **HOLDS (no drift)**.

**GreptimeDB v1.0.2:** native OTLP, GA, default-on. `src/servers/src/http/otlp.rs`
handles **metrics + traces + logs** (`opentelemetry_proto` + OTel-Arrow). Live:
`/v1/otlp/v1/{metrics,traces}` → **HTTP 400** (endpoint exists, dummy payload rejected —
not 404).

**Claim status:** "ClickHouse needs an OTLP collector; GreptimeDB native OTLP" →
**CONFIRMED at 26.5.** Notable contrast with Run 23: ClickHouse's 26.x observability
investment went to **Prometheus** (TimeSeries + remote-write + PromQL), **not OTLP**.
For Parallax's OTLP-centric telemetry the native-ingest edge stays decisively
GreptimeDB. (Re-verification — confirms an existing claim, the honest opposite of the
PromQL drift.)

### Run 26 — 2026-05-25 — Metric high-cardinality mechanism (config confirm)

Backs `metric-cardinality.md` (pass 48). Config-level confirm of the high-cardinality
storage mechanism (not a sized storage benchmark — that's owed).

**ClickHouse:** `low_cardinality_max_dictionary_size = 8192` (live). Source doc: data
past the cap is written "in an ordinary method" → a `LowCardinality(String)` label
column with **>8192 distinct values overflows the dict and falls back to plain
storage** = the high-cardinality cliff. (A 50k-distinct demo table was created but the
quick `system.columns` size probe returned 0 — a view/timing artifact; the cliff is
source-documented, the cap is live-confirmed.)

**GreptimeDB:** metric engine series key = `__tsid` (label-set hash;
`benches/bench_tsid_generator.rs` exists → perf-critical for high card); PartitionTree
memtable dict-encodes label sets + shards series + multi-partitions by primary key — no
per-series dict cap, high cardinality is the design center.

**Claim status:** high-cardinality **storage/ingest ergonomics → GreptimeDB**
(metric engine + PartitionTree, no LowCardinality cliff); high-cardinality
**aggregation latency → ClickHouse** (Run 11 ~10×, vectorized engine). Split across
axes — "GreptimeDB handles high card better" = modeling/storage, NOT agg speed. Sized
storage comparison (1k→1M distinct series) routed to B13.

### Run 27 — 2026-05-25 — Trace span-tree: flat fetch vs in-DB recursion

Backs `trace-span-tree.md` (pass 49). Smoke, on the existing 1M-row `spans` table.

- **Recursive CTE works on BOTH** (verdict-relevant tie): `WITH RECURSIVE … sum(1..5)`
  → `15` on ClickHouse (native) and GreptimeDB (DataFusion). Real span-tree recursive
  join over `spans` executed on both — CH ~7 ms, **GreptimeDB ~8 ms server-side** (the
  synthetic data isn't a clean parent chain so depth grouping was trivial, but the
  recursive join ran with no error on both).
- **Flat anchored fetch** (all 14 spans of one `trace_id`, the dominant pattern, app
  builds the tree): **ClickHouse 4 ms** (`ORDER BY (trace_id, ts)` sort-key locality →
  one granule range) vs **GreptimeDB ~54 ms** HTTP (inverted index on `trace_id` +
  fixed HTTP/setup floor; `trace_id` not the PK prefix in the seed).

**Claim status:** span-tree retrieval is **not a new differentiator** — it = the
anchored `trace_id` fetch (ClickHouse edge via sort-key locality, Run 2/6) + app-side
tree assembly; in-DB recursive CTE is a **capability tie** (DataFusion gives GreptimeDB
recursion for free). Reinforces, doesn't move, the verdict. Clean-tree recursion-depth
latency owed to harness.

### Run 28 — 2026-05-25 — ClickHouse projections vs GreptimeDB index (access paths)

Backs `projections-and-access-paths.md` (pass 50). Smoke.

**ClickHouse:** `proj_test ORDER BY (trace_id, ts)` + `PROJECTION p_service (SELECT *
ORDER BY service)`, 500k rows. `EXPLAIN indexes=1` for `WHERE service='svc5'` →
**`ReadFromMergeTree (p_service)`** — optimizer transparently picked the projection
(not the base trace_id order). One table, two access paths. **Storage ~doubles:**
`system.parts` total 4.07 MiB vs `system.projection_parts` 2.07 MiB → the normal
projection is a near-full second copy.

**GreptimeDB:** **no projection feature** — parser rejects `PROJECTION` ("Cannot use
keyword 'PROJECTION' as column name"). Multi-access = secondary indexes
(inverted/skipping/fulltext), row-positions at index size, no second physical copy.

**Claim status:** ClickHouse projections = a real capability for **scan-by-alternate-
ordering** (no GreptimeDB equivalent), at **~2× storage per normal projection**.
GreptimeDB's inverted index is leaner for **anchored point/filter** (Parallax's shape).
For anchored reads it's a wash (both fast); projections win scan-heavy multi-ordering
at a storage cost. Reinforces the read-path/cost picture; no verdict flip. GB-scale
projection-scan vs index-lookup latency owed to harness.

### Run 29 — 2026-05-25 — Deletes + mutations (corrections / GDPR-erase / update)

Backs `deletes-and-mutations.md` (pass 51). Smoke.

**ClickHouse:**
- Lightweight `DELETE FROM del_test WHERE id<50000` (plain table) → 100k→50k rows;
  `system.mutations` = **`UPDATE _row_exists = 0 WHERE id<50000`**, part `all_1_1_0`→
  `all_1_1_0_2` (a `_row_exists` **mask**, not a surviving-row rewrite). GA-ish,
  default-on (`lightweight_deletes_sync=2`).
- Lightweight `UPDATE upd_test SET v='new'` → **rejected**: "Lightweight updates …
  supported only for tables with materialized `_block_number` column … enable
  `enable_block_number_column=1`." Settings `enable_lightweight_update=1` +
  `allow_experimental_lightweight_update=1` default-on but **experimental + per-table
  setup**; else `UPDATE` = heavy `ALTER UPDATE` part rewrite.

**GreptimeDB:** `DELETE FROM gt_del WHERE k='b'` → row **immediately** gone from
queries (`['a','c']`), no compaction forced (tombstone + read-filter, pass 39). UPDATE =
re-insert `(PK,ts)` → dedup last-wins (cheap upsert, GA).

**Claim status:** **DELETE ≈ parity** — ClickHouse lightweight delete (mask, default)
caught up to GreptimeDB tombstone; both read-immediate. **UPDATE → GreptimeDB** — GA
zero-setup upsert vs ClickHouse heavy rewrite (lightweight update experimental +
per-table block-number column). Reinforces LSM-native correction ergonomics; updates
the divergence. GB-scale rewrite-vs-mask-vs-tombstone cost owed to harness.

### Run 30 — 2026-05-25 — Q4 cross-tier frontend↔backend join (anchored)

Backs the evidence-bundle verdict (the brief's Q4). Completes the Q1–Q6 smoke set
(Q1/Q2/Q3 = Run 16; Q4 here). New `frontend_events` table (one event per trace),
joined to `spans` on `trace_id`, anchored on one trace (14 spans). Smoke.

- **ClickHouse: 5 ms.** `EXPLAIN` — both sides prune to the anchor via
  `ORDER BY (trace_id, ts)` sort-key locality: `frontend_events` **Granules 1/9**,
  `spans` **Granules 1/123**, plus a 26.x **`BuildRuntimeFilter`** on the join key.
  `Join (FillRightFirst)` over the tiny pruned inputs.
- **GreptimeDB: 59 ms** (HTTP-measured, ~50 ms fixed floor). `EXPLAIN` — anchor
  `trace_id=X` **Filter pushed to BOTH inputs** (frontend_events + spans), then
  `HashJoinExec mode=Partitioned` + `RepartitionExec Hash([trace_id], 10)`.
- Result parity: **14 rows both** (1 frontend event × 14 backend spans).

**Claim status:** confirms pass-5 framing with measurement — **anchored cross-tier
join is NOT join-algorithm-decided**; both engines propagate the anchor constant to
both inputs and join a tiny set. The gap is the familiar fixed overhead (CH sort-key
locality + runtime filter; GT HTTP floor + 10-way repartition of a toy input, a
small-scale artifact). Part of the not-latency-bound bundle (Run 16). Reinforces, does
not move, the verdict. Un-anchored large↔large join (B4) still owed.

### Run 31 — 2026-05-25 — Q5 high-cardinality filter (completes Q1–Q6 smoke set)

Backs the evidence-bundle verdict (the brief's Q5). Filter the 1M `spans` table by a
**high-cardinality, non-sort-key** column (`span_id`, ~1M distinct; neither engine keys
it — CH `ORDER BY (trace_id,ts)`, GT PK `(service,name)`). Smoke, matched dataset.

- **ClickHouse: 10 ms.** `EXPLAIN` = `Granules: 123/123` — **full scan** (no skip index
  on `span_id`), vectorized C++ filter. Found 1 row.
- **GreptimeDB: 95 ms** (HTTP-measured) — full DataFusion scan of 1M + filter. 1 row.

**Two Q5 regimes, both now covered:**
1. **Unindexed high-card filter → full scan** (this run): ClickHouse ~**10×** faster —
   the vectorized-engine throughput edge (pass 42), the honest "ClickHouse wins scans"
   result; operator hypothesis still doesn't hold for scan-shaped queries.
2. **Indexed high-card filter → anchored lookup** = the `trace_id` case (Runs 2/6):
   CH via sort-key locality, GT via inverted index — both fast/acceptable.
3. **JSON-attribute high-card filter:** CH columnar subcolumn beats GT blob-parse
   `json_get_*` (pass 38 / Run 18 mechanism).

**Parallax lesson:** index the high-card attributes you filter on (both engines can —
CH bloom/skip, GT inverted/skipping); the dominant bundle queries are *anchored* anyway
(not Q5-scan-bound). **Q1–Q6 smoke set now complete** (Q1/Q2/Q3 Run 16, Q4 Run 30,
Q5 here, Q6 composite Run 16). Larger-tier cold scan still the prototype's.

### Run 32 — 2026-05-25 — Jaeger query API (closes public claim #7)

Backs `public-performance-claims.md` claim #7. The last unverified sub-claim
("GreptimeDB native Jaeger API").

- **GreptimeDB: native GA Jaeger query API.** Live: `GET /v1/jaeger/api/services` →
  **HTTP 200** with Jaeger-format JSON (`{"data":null,"total":0,…}` — empty, no
  Jaeger-ingested traces, but the endpoint works default-on). Source
  `src/servers/src/http/jaeger.rs` (1750 lines): `handle_get_services` +
  Operations/OperationsNames/Traces handlers + **tag/span-attribute search**
  (`tags="{…}"`) + trace limits — the full Jaeger query surface. So Jaeger UI / Grafana
  Jaeger datasource can query GreptimeDB traces with **zero adapter**.
- **ClickHouse: no native Jaeger** — no `jaeger` function; integration is the external
  **`jaeger-clickhouse` storage plugin** (Jaeger's own query service reads ClickHouse
  via a gRPC backend), same external-adapter pattern as OTLP.

**Claim status:** claim #7 **fully resolved** — all three GreptimeDB protocols verified
(OTLP Run 25, PromQL Runs 23–24, Jaeger Run 32); ClickHouse has none natively (collector
/ experimental TimeSeries / external plugin). Reinforces GreptimeDB's
observability-ecosystem-native fit; the one correction stands (PromQL not "absent" on
ClickHouse, just experimental).

### Run 33 — 2026-05-25 — Async-insert buffer mechanism + freshness window

Backs `write-path-and-ingestion.md` (pass 56). Config + mechanism confirm.

**ClickHouse** (`AsynchronousInsertQueue.cpp`, live settings): `async_insert=1`,
`wait_for_async_insert=1` default; buffer flush triggers = `async_insert_max_data_size`
**10 MiB** / `async_insert_max_query_number` **450** / adaptive busy timeout
`min_ms=50`/`max_ms=200`. So small inserts buffer server-side and flush to one part on
size/count/timeout → solves part-explosion, but data is invisible + non-durable until
flush (≤200 ms window; wait=1 blocks the client to absorb it, wait=0 leaves a loss
window). Freshness window too small to catch across separate docker-exec calls
(~50–100 ms each) — a single async insert had already flushed by query time; mechanism
+ triggers are source/settings-confirmed.

**GreptimeDB**: no async buffer — the LSM memtable absorbs small writes natively and is
**queryable immediately** (re-confirmed: single insert → `count=1` instantly, no
window) **and durable** (WAL-first). Same absorption, zero freshness/durability cost.

**Claim status:** confirms + sharpens pass-9 — ClickHouse small-write absorption is a
server-side **buffer** costing a ≤200 ms freshness/durability/latency window;
GreptimeDB's LSM gives it natively, visible+durable on write. Write-path ergonomics +
freshness edge GreptimeDB (mechanism-grounded; modest absolute ms). No verdict flip.

### Run 34 — 2026-05-25 — Zero-copy replication (replication storage economics)

Backs `distributed-and-scaling.md` (pass 57). Config + source confirm.

**ClickHouse:** `allow_remote_fs_zero_copy_replication = 0` (live default). Source
(`MergeTreeSettings.cpp:1955`) marks it **EXPERIMENTAL** with the explicit warning
**"Don't use this setting in production, because it is not ready."** Surrounding
machinery confirms the fragility: ZooKeeper-coordinated part-removal split/postpone
locks (`zero_copy_concurrent_part_removal_*`), `remote_fs_zero_copy_zookeeper_path=
/clickhouse/zero_copy`, and `freeze`/`detach`/`fetch partition` **disabled** under it.
→ OSS `ReplicatedMergeTree` on S3 realistically stores **N full copies for N replicas**
(N× S3 cost); the 1× shared-copy path is not production-ready, and `SharedMergeTree` is
Cloud-only.

**GreptimeDB:** no zero-copy concept — object-store-native means storage is inherently
shared; a region's SSTs live once in S3, datanodes open them (reopen-from-S3, pass 34).
HA replication = region leadership + Metasrv metadata + remote WAL, **not data copy**.
1× S3 storage by default.

**Claim status:** for **HA on object storage**, GreptimeDB's shared-storage model is
cheaper (1× vs N× S3) and simpler (no fragile coordination); OSS ClickHouse must pick
N× cost, not-production-ready zero-copy, or Cloud. Reinforces the object-store-native
edge on the replication dimension (cost #2 + scaling #3). Arch + ClickHouse's own
source warning; multi-replica S3 cost measurement owed to harness.

### Run 35 — 2026-05-25 — Query-result cache (footnote-level caching layer)

Backs `caching-and-cold-warm.md` (pass 60). Completes the caching-layer comparison
(data/index caches done pass 24; this is the query-*result* layer). Version re-confirmed
(GreptimeDB v1.0.2, ClickHouse v26.5.1.882 — no bump).

- **ClickHouse:** has a query-result cache. `use_query_cache=0` (off by default),
  `query_cache_ttl=60` s, `enable_reads_from_query_cache=1` (live). On a hit a repeated
  identical SELECT returns the cached result and **skips execution**.
- **GreptimeDB:** **no *whole-query* result cache** [refined Run 36: it *does* have a
  partition-range scan-result cache `read/range_cache.rs`; the distinction is granularity].
  `src/mito2/src/cache/` = file/index/manifest/
  write caches + an *index-probe* `index/result_cache.rs` (caches index-match rows, not
  the final result). A repeated query re-executes on warm data (live: 66 → 4 ms = data-
  cache warmth, not result-caching).

**Claim status:** footnote. ClickHouse can skip re-execution on repeated-identical
queries (off-by-default result cache); GreptimeDB always re-executes on warm caches.
Modest CH edge for repeated dashboard refreshes; **near-zero hit on Parallax's anchored,
unique-key bundle queries** → not a hot-path differentiator, no verdict move.

### Run 36 — 2026-05-25 — Changelog review of pinned versions (method #4) + a self-correction

Maintenance pass: systematically reviewed the **release changelogs** of the pinned
versions (not just settings/source) for perf-relevant changes that could affect
load-bearing findings. Versions unchanged (GreptimeDB v1.0.2 latest; ClickHouse
v26.5.1.882, no 26.6/27.x).

**GreptimeDB v1.0.2 release notes — two relevant items:**

1. **Self-correction to Run 35 (pass 60).** PR #8105 ("range result cache could reuse a
   previous query's result under `merge_mode` + `OR` time-filter") revealed GreptimeDB
   **does** have a result-level cache — `src/mito2/src/read/range_cache.rs`, a
   **partition-range scan-result cache** (fingerprint-keyed, reused across queries
   scanning the same range). My Run-35 "no query-result cache" was imprecise: the
   accurate statement is **no *whole-query* result cache** (ClickHouse `query_cache`
   skips full execution on a hit) but GreptimeDB **has a scan-range result cache** (skips
   scan I/O+decode for matching ranges, still re-plans+re-aggregates). Corrected in
   `caching-and-cold-warm.md`. (Pinned v1.0.2 has the correctness fix.)
2. **PromQL perf #7926:** time-range pushdown now works for non-ms time precision
   (`Timestamp(ns)`/`(us)`) — previously bounded PromQL on sub-ms tables fell back to
   full SST scan. Doesn't affect Parallax (its `greptime_timestamp` is ms), but confirms
   the pinned version includes active PromQL pushdown work; no finding invalidated.

**No finding invalidated by the changelog review.** ClickHouse pin has no newer stable;
GreptimeDB pin's notes are bug-fixes + a sub-ms PromQL pushdown + the range-cache fix —
none change the verdict. Net: a real accuracy correction (range cache) caught by the
method-#4 changelog sweep, not padding.

### Run 37 — 2026-05-25 — Re-verify Run 11 metric-agg → the "~10×" was cold; warm is ~2×

Maintenance re-verification of the most load-bearing measured claim (ClickHouse ~10×
metric aggregation, the result that refutes the operator hypothesis on agg speed).
Re-ran Run 11's **exact** query (`avg by service, 5-min buckets`) on the intact
`metrics_hc` (8M rows / 40k series), both **warm** (data resident ~5 h). Versions
unchanged.

| | ClickHouse | GreptimeDB | ratio |
| --- | --- | --- | --- |
| Run 11 (pass 20) | 65 ms | **638 ms** | ~10× |
| **Run 37 (warm, min of 3)** | **50 ms** | **107 ms** (server `execution_time_ms`) | **~2×** |

ClickHouse is consistent (50–65 ms); **GreptimeDB went 638 → 107 ms (~6× faster than
Run 11)**. The result is only 800 rows, so HTTP transfer can't explain it → **Run 11's
638 ms was a cold/first-run GreptimeDB measurement** (taken right after the 2.98 s
ingest, caches cold → full SST scan + decode), not the warm steady-state. **Warm, the
SQL metric-agg gap is ~2×, not ~10×.** This also fits the mechanism better: the pass-42
exec-engine edge (8× block + JIT + SIMD) predicts a ~2–3× warm gap, not 10× — the 10×
was always suspiciously large for the mechanism, and the cold-cache explanation
resolves it.

**Correction (honest, load-bearing):** the "ClickHouse ~10× on metric aggregation"
claim is **warm-overstated** — warm steady-state is **~2×**; the ~10× reflected a
**cold/first-run** GreptimeDB scan (a valid *cold-regime* data point, but it was
labeled as the general agg gap). Updated per-signal-verdict, verdict, and
metric-cardinality. Net: ClickHouse still wins SQL metric agg (vectorized engine,
pass 42) but by **~2× warm**, materially narrower than stated — slightly strengthens
GreptimeDB's position (does not flip the verdict). Cold-regime agg gap (larger) ties to
`caching-and-cold-warm.md`; the precise cold number is owed to the cold-tier harness.

### Run 38 — 2026-05-25 — Re-verify Run 12 full-text ~18× → HOLDS warm (unlike the agg)

Companion to Run 37: applied the same warm-vs-cold scrutiny to the **other** load-bearing
ClickHouse win — the ~18× full-text gap (the verdict's flip-trigger). Re-ran on the
intact `logs_b1` (5M, both text-indexed), warm. Versions unchanged.

| | ClickHouse (`hasToken`) | GreptimeDB (`matches`) | ratio |
| --- | --- | --- | --- |
| Run 12 (pass 21) | 7 ms | 130 ms | ~18× |
| **Run 38 (warm, min of 3–4)** | **7 ms** | **129 ms** (server `execution_time_ms`) | **~18×** |

Parity preserved (n = **698,955** both). **The ~18× HOLDS warm — it was *not*
cold-inflated**, unlike the metric-agg (Run 37: 10× cold → 2× warm).

**Why the two re-verifications differ — and both are now trustworthy:**

- **Metric-agg (Run 11/37) is *scan-bound*** — a full scan+aggregate of 8M rows. Cold
  caches → full SST scan/decode (the 638 ms/10×); warm → ~2×. **Cold-sensitive.**
- **Full-text (Run 12/38) is *index-bound*** — both use a small text index (CH `text`
  posting-list vs GT Puffin/tantivy + DataFusion `matches()`); the index stays warm, so
  the gap is **execution/index-maturity**, not cold scan. CH's vectorized posting-list
  `hasToken` vs GT's tantivy-probe→DataFusion-`matches()` row eval → **~18× warm, real
  and stable.**

So the corrected, coherent picture of ClickHouse's warm wins: **full-text log search
~18× (index maturity, real, stable)**; **SQL scan-aggregation ~2× warm (larger cold)**;
selective keyed filter a tie; anchored bundle not latency-bound. **The verdict's
flip-trigger (log-search-dominated mix → ClickHouse wins decisively) STANDS** —
confirmed warm. No correction needed (unlike Run 37); confirmation strengthens it.

### Run 39 — 2026-05-25 — Re-verify Run 12 count-by-level scan ~4× → HOLDS warm

Third re-verification (after Runs 37/38), completing the warm-check of Run 12's three
numbers. Count-by-`level` scan on `logs_b1` (5M), warm, min of 3. Versions unchanged.

| | ClickHouse | GreptimeDB | ratio |
| --- | --- | --- | --- |
| Run 12 | 7 ms | 28 ms | ~4× |
| **Run 39 (warm)** | **8 ms** | **32 ms** (first run 94 ms cold) | **~4×** |

**Holds warm (~4×)** — *not* cold-inflated. So Run 12's three numbers now stress-tested:
full-text ~18× (Run 38, holds), count-by-level scan ~4× (holds), selective filter tie.
Only the **separate** metric-agg (Run 11/37) was cold-inflated (10×→2×).

**Refines the cold-inflation model:** the cold penalty is ∝ **bytes decoded cold**, not
"scan vs index" alone —
- **metric-agg** scans 8M rows reading **value(Float64)+ts+service** + per-row
  time-bucketing → heavy cold decode → 638 ms cold (10×), 107 ms warm (2×);
- **count-by-level** scans 5M rows reading **one `LowCardinality(level)` column** into ~5
  groups → light cold decode → 94 ms cold, 32 ms warm (~4× both);
- **full-text** reads a small index, no wide scan → no cold inflation (~18× warm).

So warm gaps: full-text ~18× (index), count-by-level scan ~4× (light scan), metric-agg
~2× (heavy bucketed agg). The *cold* regime widens the scan gaps (∝ bytes decoded) —
the cold-tier harness will quantify it; the read-path warm numbers are now all
verified. No verdict move; confirmation + a cleaner cold/warm mental model.

### Run 40 — 2026-05-25 — Fair trace-lookup: strip the HTTP floor + the index caveat

Re-measured the anchored `trace_id` point lookup (Parallax's dominant query) on a
**fair basis** — GreptimeDB **server `execution_time_ms`** (HTTP-stripped), since all
prior GT point-query numbers carried the ~40–50 ms HTTP-wall floor. Versions unchanged.

| | ClickHouse | GreptimeDB |
| --- | --- | --- |
| trace lookup, warm (min 3) | **2 ms** (ORDER BY `(trace_id,ts)` sort-key seek, 1 granule) | **14 ms server** (first run 65 ms cold) |

**Two fairness clarifications:**

1. **HTTP floor stripped.** GT's server-side lookup is **14 ms**, not the **54 ms**
   reported via HTTP wall (pass 49 / Run-1's 16 ms also HTTP-ish). The ~40 ms gap was
   HTTP/JSON round-trip, not engine time. So *all* GT point-query latencies in earlier
   runs are HTTP-inflated by ~40 ms; the engine numbers are far smaller.
2. **The bench `spans` has NO `trace_id` index** (PK = `service,name`) → GreptimeDB is
   **full-scanning 1M rows** for this lookup (14 ms server). **Parallax's GreptimeDB
   *design* adds `trace_id INVERTED INDEX`** (`greptimedb-implementation.md`, the "Run-1
   fix"); with it the lookup is ~8 ms (Run 6). So the designed-path gap is even smaller.

**Fair anchored-lookup gap:** CH **2 ms** (sort-key locality) vs GT **~8 ms indexed /
14 ms unindexed-scan** (server) — ClickHouse ~**4–7×** by sort-key locality, but **both
are single-/low-double-digit ms, ≪ the 300 ms gate**. So GreptimeDB's "loss" on the
anchored hot path is (a) partly an HTTP-measurement artifact and (b) shrinks with the
trace_id index Parallax's design already specifies. Reinforces **anchored bundle = not
latency-bound** (Run 16). Honest fairness correction; no verdict move (CH still faster
on the lookup, GT still chosen on fit). Caveat noted: re-running GT point-queries via
the MySQL native protocol would strip the HTTP floor in future runs.

### Run 41 — 2026-05-25 — Cross-path validation: GT engine-time is stable (~14 ms)

Closes the measurement-methodology thread from Run 40 via a **third measurement path**.
No `mysql` client in the containers, but **ClickHouse's `mysql()` table function reached
GreptimeDB:4002** (MySQL wire) and ran the trace lookup. Versions unchanged.

| Path for the GT trace lookup | wall | what it includes |
| --- | --- | --- |
| Server `execution_time_ms` (HTTP report) | **14 ms** | **engine only** |
| HTTP wall (pass 49) | 54 ms | engine + ~40 ms HTTP transport |
| ClickHouse `mysql()` federation | 39 ms | engine + ~25 ms fresh-conn/MySQL federation |

**GT engine time is ~14 ms across all three paths**; the larger walls are
transport/connection overhead, not engine. → **confirms `execution_time_ms` is the
engine-fair metric** my re-verifications (Runs 37–40) used, and the old HTTP-wall
numbers were transport-inflated (~25–40 ms). No further latency correction needed; the
recorded server-time numbers stand.

**Interop bonus:** GreptimeDB's **MySQL wire protocol is confirmed working** — ClickHouse
federated a query into it via `mysql()`. So MySQL-protocol clients / BI tools / Grafana's
MySQL datasource can query GreptimeDB directly (relevant to Parallax's tooling surface).

This completes the load-bearing-number re-verification arc (Runs 37–41): one correction
(metric-agg 10×→2× warm), confirmations (full-text ~18×, scan ~4×), a cold-inflation
model, a fairness fix (HTTP floor), and this cross-path validation. The empirical base
is now self-consistent and HTTP-fair.

### Run 42 — 2026-05-25 — Q6 anchored component server-time (not-latency-bound robust)

Maintenance: checked whether Run 16's GT Q6 composite (~33 ms) was HTTP-inflated enough
to matter. Re-ran the **Q1 trace_context shape** (anchored 3-way UNION over
spans+logs+error_events) server-time, min 3. Versions unchanged; Q6 tables intact
(spans 1M, logs 214k, error_events 2,226).

- GT Q1 3-way union: **~16 ms server** — dominated by the **un-indexed spans full-scan**
  (~14 ms, Run 40; bench `spans` has no `trace_id` index, which Parallax's design adds).
- So GT's Q6 composite is ~25–33 ms whether read as engine-time or HTTP-wall; CH ~10 ms.

**Conclusion robust:** both ≪ the 300 ms gate → **the dominant anchored bundle is not
latency-bound on either engine, regardless of the HTTP-vs-engine-time reading** (Run 16
holds). GT's anchored fetch would drop further with the `trace_id INVERTED INDEX` its
implementation specifies (Run 6/40). No correction; confirmation that the headline
"not latency-bound" survives the HTTP-floor scrutiny applied in Runs 40–41.

This effectively closes the empirical re-verification: every load-bearing number is now
warm + HTTP-fair-checked (Runs 37–42), and all conclusions hold (one correction:
metric-agg 10×→2× warm; everything else confirmed). Further empirical value needs the
larger-tier/cold/multi-node harness.

### Run 43 — Rollup / continuous aggregation, live (Flow vs MV+AggregatingMergeTree)

First **live** test of the rollup mechanism — `rollup-and-continuous-aggregation.md` was
the only major note that was pure source-reasoning (no Docker run). Env: same containers,
GreptimeDB `v1.0.2`, ClickHouse `v26.5.1.882-stable`. Source: `metrics_real` (864000 rows,
~6 h span, 12 services, 100 instances, `gauge Float64`). Rollup built on both: **1 h
`avg(gauge)` by service** → 84 rollup rows. Measured warm (GT = `execution_time_ms`; CH =
`--time`).

| Metric | GreptimeDB (Flow) | ClickHouse (MV + AggregatingMergeTree) |
| --- | --- | --- |
| Raw windowed-avg over 864k (warm) | ~16–25 ms | ~10–13 ms |
| Rollup-table read (warm) | ~3–4 ms (first 46 ms cold/plan) | ~2 ms |
| Pre-aggregation read speedup | **~5×** | **~5–6×** |
| Forward maintenance | `CREATE FLOW` + new insert → sink updates (verified) | push-MV on insert block → target updates (verified) |
| Historical backfill | **forward-only auto-pop**; sink is a plain table → one-off `INSERT…SELECT` backfills (verified, 84 rows) | target is a plain table → one-off `INSERT…SELECT …State()` backfills (verified, 84 rows) |
| Stored form | **finalized** values, read direct | partial `-State`, read needs `-Merge` |

Findings:

- **Both deliver ~5–6× rollup read speedup** (raw windowed-agg vs reading the
  pre-aggregated table). The "pre-aggregation moves compute to ingest/background; reads
  get cheap on both" claim is now **confirmed live**, not just reasoned. Raw windowed-agg
  itself is CH-faster (~10–13 ms vs ~16–25 ms), consistent with the established
  scan-aggregation edge (~1.5–2× warm).
- **GreptimeDB Flow is forward-only on auto-population.** `CREATE FLOW` over `metrics_real`
  then `ADMIN FLUSH_FLOW` produced **0 sink rows** — the 864k pre-existing rows were not
  pulled in; only data inserted *after* flow creation flowed to the sink (verified: a fresh
  `flow_probe` insert appeared post-flush). **But the sink is an ordinary writable table**,
  so a one-off `INSERT INTO sink SELECT … GROUP BY date_bin(…)` backfills history (verified,
  84 rows). Net: operationally **parallel** to ClickHouse's "MV maintains forward + manual
  `INSERT…SELECT` backfills the target."
- **Flow correctness confirmed.** The `flow_probe` sink row (avg 40.0 / n 2) matched the raw
  truth exactly — the apparent "n=2 not 5" was GreptimeDB read-time dedup: 5 inserts shared
  one `now()` ms, so PK `(ts,service,instance)` collapsed them to 2 logical rows (i1→30,
  i2→50; avg=40). Cross-confirms `dedup-and-update-semantics.md` (LastRow) and that Flow
  aggregates over the *deduplicated* source.
- **CH MV catches new inserts live**: a row inserted into `metrics_real` immediately
  surfaced in the rollup via `avgMerge` (mv_probe_svc→42).
- **Mechanism contrast confirmed live**: GT Flow sink holds **finalized** values (read
  directly, zero ceremony); CH AggregatingMergeTree holds partial **`-State`** (read via
  `avgMerge`/`FINAL`). The cleaner-model point for GreptimeDB is now empirical, not just RFC.

Verdict on the note's claim: **"wash with opposite tilts" holds, now with an empirical
backbone** — both give Parallax the rollup tooling it needs at ~5–6× read speedup;
GreptimeDB's model is cleaner (finalized rows, no `-State`/`-Merge`, forward-only auto-pop
softened by trivial manual backfill); ClickHouse's MV+AggregatingMergeTree is more mature.
Neither moves the verdict. Cleanup: dropped both rollups + flow/MV and the probe rows;
both base tables back to 864000.

### Run 44 — High-cardinality metric agg via GreptimeDB's NATIVE PromQL path (the twice-owed run)

Closes the item Runs 11 & 37 both flagged owed: every metric-agg number so far used SQL
`GROUP BY` (ClickHouse's home turf); none exercised **GreptimeDB's native PromQL planner**
— the verdict's actual #1 metrics pillar. Question: does the PromQL path deliver a *speed*
benefit at high cardinality, or is it purely capability? Env: same containers, `metrics_hc`
(8M rows, **40 svc × 1000 inst = 40k series**, ~100 min span, `value` FIELD). All warm
(resident ~5 h). GT via `TQL EVAL`; result sizes verified equal (800 points = 40 svc × 20
steps). Same-session re-measure of the SQL bars for a self-consistent comparison.

| Path | Query | Warm (min of 3) |
| --- | --- | --- |
| **ClickHouse SQL** | `avg(value) … GROUP BY service, 5-min bucket` | **~62–78 ms** |
| **GreptimeDB SQL** | same (`date_bin('5 minutes')`) | **~120 ms** (≈ Run 37's 107) |
| **GreptimeDB PromQL** | `TQL EVAL (…,'5m') avg by (service) (metrics_hc)` (20 steps) | **~580–647 ms** |
| GreptimeDB PromQL, **single instant** | `TQL EVAL (t,t,'5m') avg by (service) (…)` (1 step) | **~528–545 ms** |
| GreptimeDB PromQL, **rate()** | `… avg by (service) (rate(metrics_hc[5m]))` | **~661–693 ms** |

**Finding — GreptimeDB's own PromQL path is ~5× slower than its own SQL path** (and ~9× the
CH SQL bar) at high cardinality. The mechanism is the **kicker**: the **single-step instant
eval (~535 ms) is nearly as expensive as the full 20-step range (~590 ms)** → the cost is
**not** per-step; it is a **near-fixed series-normalization setup**. GreptimeDB's PromQL
planner must `SeriesDivide`/`SeriesNormalize` — sort + partition the entire scanned input by
series — before applying the instant/range manipulation (`promql-and-metrics-query.md`
planner nodes). Over 40k series × 8M rows that sort/partition is the dominant ~530 ms,
incurred once regardless of step count. The SQL path (120 ms) avoids it: a streaming
vectorized hash-aggregation needs no per-series sort. `rate()` is the same setup + range
extrapolation (~670 ms).

**Consequence (sharpens the verdict's #1 pillar, does not flip it):** the metrics → GreptimeDB
case is **capability/ergonomics, NOT speed — now confirmed harder**. For raw metric-aggregation
*latency* at volume the ordering is **CH SQL (≈65 ms) > GT SQL (≈120 ms) > GT PromQL (≈590 ms)**.
Even GreptimeDB's *fastest* metric path is SQL, not PromQL; PromQL's value is **expressiveness**
(range vectors, `rate`/`irate`, lookback, step alignment — things SQL can't say natively), and
it is "fast enough" (sub-second on 8M/40k-series smoke), not a speed leader. So "metrics →
GreptimeDB" rests entirely on GA PromQL ergonomics + native multi-protocol ingest + the
metric-engine *storage* model, never on query speed.

**Honest caveats:** (1) `metrics_hc` is a **plain table** queried via PromQL, not the metric
engine's logical→physical wide table — but the PromQL *planner* (and its `SeriesNormalize`
cost) is identical either way; the metric engine changes *storage/ingest* layout, not this
query path (`metric-cardinality.md`). (2) ClickHouse's experimental PromQL (`TimeSeries` engine)
can't be compared here — it needs remote-write ingest and won't query an existing `MergeTree`
table (Run 23/24), so the only practical CH metric-agg path is SQL. (3) Smoke scale; the
fixed series-normalization cost should grow with series count — a cold/larger-tier run is owed
to the harness. (4) GT first-call was 219 ms (cold/plan) vs 120 ms warm — warm used throughout.

### Run 45 — Build the GreptimeDB implementation DDL live (the "buildable design" bar)

`greptimedb-implementation.md` claimed "DDL syntax verified against the pinned source" but
the full schema had **never been executed** — only read against `create_parser.rs`. The
brief's bar is "we know *exactly* how we would build it." Ran the entire schema on live
GreptimeDB `v1.0.2` in a scratch database (`ddlcheck`, dropped after). **Two real defects
caught — the design did NOT build as written:**

1. **Reserved-keyword columns rejected.** `service`, `name`, `status`, `level`, `release`,
   `url`, **`message`** are reserved in v1.0.2's SQL parser → `Cannot use keyword '…' as
   column name`. Fix: quote them (`"col"` *or* `` `col` `` — both confirmed working; my
   first "quoting doesn't work" reading was a shell command-substitution artifact on
   backticks, retested clean via `--data-urlencode sql@file`). Not reserved:
   project/environment/fingerprint/error_type/span_id/trace_id/duration_ms/session_id/
   user_id/command/tool/app/event_type/action_type/commit_sha/host/instance.
2. **Empty `PRIMARY KEY ()` invalid** on the metric-engine physical table →
   `Expected: identifier, found: )`. Fix: omit the clause; `ENGINE = metric WITH
   ("physical_metric_table" = '')` alone is correct.

After both fixes: **all 8 signal tables + 1 logical metric table build clean.** `SHOW CREATE
TABLE` confirmed `trace_id … INVERTED INDEX` (spans) and `message … FULLTEXT INDEX` (logs)
attached (not silently dropped), `SKIPPING INDEX` on `user_id` (cli/frontend) accepted, and
the logical→physical metric link (`on_physical_table = 'greptime_physical_metrics'`) works.

**Consequence (axis: correctness of the design, not speed):** the recommended engine's
storage design is now **verified buildable**, not just syntax-reasoned — and two drift bugs
that would have bitten a real implementer on day one are fixed in the note. No verdict
impact (both defects are DDL-surface, not mechanism). Bench base data untouched (scratch db
only). **Owed next: the same live-build pass on `clickhouse-implementation.md`** (codecs like
`Gorilla`/`DoubleDelta`/`T64`, `LowCardinality`, the `text`/`tokenbf` skip indexes, MV/AggMT
— confirm each parses on `26.5.1.882`).

### Run 46 — Build the ClickHouse implementation DDL live (parallel to Run 45)

Companion to Run 45: executed the full `clickhouse-implementation.md` schema on live
ClickHouse `v26.5.1.882-stable` in a scratch database (`ddlcheck`, dropped after). The note
flagged the `text` index / `AggregatingMergeTree` MV / S3 tiering as "not yet built."

- **All 7 tables + the rollup MV build clean** after one fix. `JSON` type builds **bare**
  (stable in 26.5 — no `allow_experimental_json_type`), `CODEC(DoubleDelta, ZSTD)` /
  `CODEC(Gorilla, ZSTD)`, `LowCardinality`, `bloom_filter` skip indexes,
  `SETTINGS ttl_only_drop_parts = 1`, `AggregatingMergeTree` + `avgState/maxState`
  materialized view, and JSON-path access (`WHERE attributes.user = ?`) all accepted.
- **One real defect:** `INDEX … TYPE text(tokenizer = 'default')` → `Code: 36 … Unknown
  tokenizer: 'default'`. Probed the valid set on 26.5.1: **`splitByNonAlpha`,
  `splitByString`, `array`** are valid; `'default'`, `'standard'`, `'ngram'`, `'split'`,
  `'no_op'` are **rejected**. Fixed the note to `splitByNonAlpha` (word-token search, the
  intended semantics).

**Consequence (design correctness, not speed):** ClickHouse's buildable design is now
**verified buildable**, with far less drift than the GreptimeDB side — one tokenizer-name fix
vs Run 45's 7 reserved-keyword columns + invalid metric-table PK. Both implementation designs
are now live-built; the remaining ClickHouse gap is the **S3-disk storage policy + `TTL … TO
VOLUME` tiering**, which needs the MinIO compose (owed to `benchmarking-the-differences.md`).
No verdict impact. Bench data untouched (scratch db only).

### Run 47 — The full-text gap is the post-index SCAN, not the index apply (metric isolation)

Probed *where* GreptimeDB's ~18× warm full-text gap (Run 12/38) actually goes, using the
engine's own Prometheus metrics to isolate index-apply cost from total query time. Env: GT
`v1.0.2`, `logs_b1` (5M rows, `message` text-indexed), warm. Query:
`SELECT count(*) FROM logs_b1 WHERE matches(message, 'users')` (333,433 matches), 3× warm.

- **Total query: ~147–167 ms** warm (consistent with Run 12's ~130 ms GT full-text).
- **Fulltext index apply: ~0.15 ms/query.** `greptime_index_apply_elapsed_sum{type="fulltext_index"}`
  went 0.0013485 → 0.0018128 s over the 3 runs (count 8 → 11) = **0.46 ms for 3 applies ≈
  0.15 ms each = ~0.1 % of the query**.
- **Live cache state confirms indexes are cached:** `greptime_mito_cache_bytes{type="index_content"}`
  = 2.7 MiB, `{type="index_result"}` = 27 KiB with `greptime_mito_cache_hit{type="index_result"}`
  = 202. So index bytes + apply-results are warm-cached in memory.

**Finding:** the ~18× warm full-text gap is **dominated by the post-index scan/count over the
333k matched rows, not the index lookup** (which is sub-ms and cached). GreptimeDB resolves the
matching row-set in ~0.15 ms via the tantivy index, then DataFusion scans/counts those rows —
that scan is where ClickHouse's vectorized `hasToken`-confirm-on-65k-blocks wins. This
**refines `greptimedb-parity-roadmap.md` #1**: its primary lever is the **scan engine (#2 bigger
batches/JIT/SIMD) + index→scan fusion**, **not** an in-memory tantivy cache — pass 78 flagged the
tantivy dir-cache, but the apply is already fast, so that is second-order. #1 and #2 share the
same real lever (the scan engine). Refutes nothing in the verdict (ClickHouse still wins
full-text by its engine); sharpens *why* and *what to fix*.

**Caveats:** smoke scale; `count(*)` doesn't materialize wide columns (so gap #3 PREWHERE
matters more for `SELECT *`-shaped log search); 333k/5M = 6.7 % scattered matches → poor
row-group-skip locality (a very selective term would isolate the apply even more cleanly — a
follow-up). No verdict impact; bench data untouched (read-only).

## Next runs (to make the numbers mean something)

1. **Bigger tier** (`small` ≈ 25–50 GB, cold cache) so scans exceed cache and the
   vectorized-engine + granule-skip mechanisms actually bite. Drop OS page cache
   between cold runs.
2. **Matched-codec/object-cost gate**: run the
   [storage size and object cost gate](../storage-size-and-object-cost-gate.md)
   so retained bytes, object counts, request costs, cache needs, and egress are
   measured rather than inferred from the tiny local-disk smoke result.
3. **Full mixed-load Q6 gate**: run the
   [storage freshness and bundle latency gate](../storage-freshness-and-bundle-latency-gate.md)
   with all bundle signals, per-signal freshness probes, and concurrent ingest.
4. **Metrics float compression with realistic shapes** (flat gauges, monotonic
   counters, repeated values) to actually exercise Gorilla/DoubleDelta vs
   GreptimeDB Parquet — Run 3's random-walk data was incompressible. (PromQL
   nativeness + aggregation latency already done in Run 3.)
5. **Fairer GreptimeDB timing** via the MySQL native protocol, not HTTP.
6. **Object-storage path** (MinIO) for both — add to `bench/compose.yml`; cost
   interpretation belongs to the
   [storage size and object cost gate](../storage-size-and-object-cost-gate.md).

These route into `benchmarking-the-differences.md` (case design) and the runnable
`parallax-bench` harness (`storage-benchmark-prototype.md`), which owns the real veto.

## Reproduce

```bash
docker compose -f bench/compose.yml up -d
# generate spans.csv (seed 42, 1M rows, 14/trace) — see bench generator
# ClickHouse: CREATE TABLE spans ... ENGINE=MergeTree ORDER BY (trace_id, ts);
#   INSERT INTO spans FROM INFILE '/tmp/spans.csv' FORMAT CSV
# GreptimeDB: CREATE TABLE spans (... PRIMARY KEY ("service","name")); ts TIME INDEX
#   COPY spans FROM '/tmp/spans_h.csv' WITH (FORMAT='CSV')   # needs header row
docker compose -f bench/compose.yml down -v   # cleanup (data dirs are gitignored)
```
