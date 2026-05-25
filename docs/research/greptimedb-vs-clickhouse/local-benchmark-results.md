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
   **Validated (Run 60):** GreptimeDB `execution_time_ms` *matches* its native
   MySQL-wire client-wall for heavy queries (agg ~96 ms both) and slightly
   *over*-states GreptimeDB on tiny queries (anchor: HTTP ~10 ms vs native ~5 ms) —
   so the basis is fair-to-GreptimeDB-conservative, never flattering. Reported gaps
   are real, not artifacts.

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

1. **At the time, ClickHouse appeared to win log full-text search ~18×**, *even with both
   engines using their text indexes*. ClickHouse's mature `text` posting-list index +
   vectorized `hasToken` outran GreptimeDB's bloom-backed `FULLTEXT` queried through
   DataFusion `matches()` at 5M rows. **Later correction:** Runs 48-49 showed this was a
   backend/function mismatch, so the current flip-trigger is broad-term analytics rather
   than selective incident search.
2. **Selective keyed filter is a tie** (4 vs 5 ms): when the filter hits indexed/
   low-card columns (`service` PK prefix, `level`), GreptimeDB prunes as well as
   ClickHouse. Anchored/keyed access — Parallax's actual bundle pattern — does not
   show the gap.
3. **Full scan ~4×** (consistent with B5's ~10× at 8M metric rows): ClickHouse's
   vectorized engine widens with volume.

**Consequence:** the decision genuinely hinges on Parallax's real query mix.
*Anchored bundle assembly* (trace_id/fingerprint lookups + keyed filters) → both
fine, GreptimeDB's fit pillars win. *Heavy ad-hoc full-text log search at volume*
→ appeared to fire the ClickHouse flip-trigger at the time. Parallax is designed around
anchored evidence bundles, so the verdict held — and Runs 48-49 narrowed the trigger further.

**Later correction (Runs 48-49):** this measured a bloom-backed GreptimeDB index through
the wrong function (`matches()`), so the ~18× is not a current selective-search verdict.
Correct pairings prune: tantivy+`matches()` ~6 ms, bloom+`matches_term()` ~8 ms, ClickHouse
~3 ms. The surviving gap is broad-term scan analytics, not selective incident grep.

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
reconciled, later narrowed by Runs 48-49:** richer index ≠ automatically faster — the
old full-text ~18× (Run 12) was a backend/function mismatch, while anchored lookup
(Run 6) still shows sort-key locality beating secondary-index lookup. Not a verdict flip;
corrects the tempting "richer index → faster" inference. Index-build cost + cold-scale
broad-term search latency owed to harness.

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

**Why the two re-verifications differed before the Run 48-49 correction:**

- **Metric-agg (Run 11/37) is *scan-bound*** — a full scan+aggregate of 8M rows. Cold
  caches → full SST scan/decode (the 638 ms/10×); warm → ~2×. **Cold-sensitive.**
- **Full-text (Run 12/38) looked *index-bound*** — both used a small text index and the
  gap held warm. Runs 48-49 later showed this was not an index-maturity gap: the GT table
  was bloom-backed but queried through `matches()`, which full-scanned. Correct pairings
  prune and make selective full-text ~6-8 ms.

So the corrected, coherent picture of ClickHouse's warm wins after Runs 48-49: **selective
full-text ~2×, not 18×; broad-term log scan remains ~12×; SQL scan-aggregation ~2× warm
(larger cold)**; selective keyed filter a tie; anchored bundle not latency-bound. **The verdict's
flip-trigger narrows** from "log-search-dominated mix" to **broad ad-hoc log/trace analytics
dominates over anchored retrieval**.

### Run 39 — 2026-05-25 — Re-verify Run 12 count-by-level scan ~4× → HOLDS warm

Third re-verification (after Runs 37/38), completing the warm-check of Run 12's three
numbers. Count-by-`level` scan on `logs_b1` (5M), warm, min of 3. Versions unchanged.

| | ClickHouse | GreptimeDB | ratio |
| --- | --- | --- | --- |
| Run 12 | 7 ms | 28 ms | ~4× |
| **Run 39 (warm)** | **8 ms** | **32 ms** (first run 94 ms cold) | **~4×** |

**Holds warm (~4×)** — *not* cold-inflated. So Run 12's scan numbers were stress-tested:
count-by-level scan ~4× (holds), selective filter tie. The full-text ~18× also held warm in
Run 38 but was later reinterpreted by Runs 48-49 as a backend/function mismatch.
Only the **separate** metric-agg (Run 11/37) was cold-inflated (10×→2×).

**Refines the cold-inflation model:** the cold penalty is ∝ **bytes decoded cold**, not
"scan vs index" alone —
- **metric-agg** scans 8M rows reading **value(Float64)+ts+service** + per-row
  time-bucketing → heavy cold decode → 638 ms cold (10×), 107 ms warm (2×);
- **count-by-level** scans 5M rows reading **one `LowCardinality(level)` column** into ~5
  groups → light cold decode → 94 ms cold, 32 ms warm (~4× both);
- **full-text** looked cold-insensitive in Run 38, but Runs 48-49 showed the selective
  gap was the wrong backend/function pairing, not a current index gap.

So warm gaps after Runs 48-49: selective full-text ~2×, broad-term log scan ~12×,
count-by-level scan ~4× (light scan), metric-agg ~2× (heavy bucketed agg). The *cold*
regime widens the scan gaps (∝ bytes decoded) — the cold-tier harness will quantify it.
No verdict move; the main result is a cleaner cold/warm mental model.

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

This completed the first load-bearing-number re-verification arc (Runs 37–41): one correction
(metric-agg 10×→2× warm), one later-superseded confirmation (full-text ~18×), scan ~4×, a
cold-inflation model, a fairness fix (HTTP floor), and this cross-path validation. Runs 48-49
then corrected the full-text interpretation.

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

### Run 48 — The ~18× full-text gap was a query-form artifact (`matches()` vs `matches_term()`)

Follow-up to Run 47 (selective term). Env: GT `v1.0.2`, `logs_b1` (5M), warm. **Key context
discovered via `SHOW CREATE TABLE`: `logs_b1`'s `message` fulltext index is `backend = 'bloom'`**
(granularity 10240, fpr 0.01), **not** tantivy. The bloom backend pairs with the exact-term
function `matches_term()`; `matches()` is the tantivy-style *query-syntax* function.

| Query (selective, 1 match) | GreptimeDB | EXPLAIN scan `output_rows` | ClickHouse (`hasToken`) |
| --- | --- | --- | --- |
| `matches('ae119f2b')` (tantivy syntax) | **~150 ms** | **5,000,000 (full scan — no prune)** | — |
| `matches_term('ae119f2b')` (exact term) | **~8 ms warm** (32 ms cold) | **1 (pruned via bloom)** | **~3 ms** |
| `matches_term('users')` (333k matches) | ~85 ms | (scales with matched rows) | ~7 ms |

**Finding (load-bearing correction):** the **~18× full-text gap (Run 12) and the ~150 ms
"fixed-cost" of Run 47 were a query-form/backend-pairing artifact** — `matches()` on a
`backend='bloom'` index does **not** push to the index, so it **full-scans 5M rows** (EXPLAIN
ANALYZE: `UnorderedScan output_rows: 5000000`), fixed regardless of selectivity. With the
**correct pairing** (`matches_term()` on the bloom index) GreptimeDB **prunes** (scan
`output_rows: 1`) and selective exact-term search is **~8 ms warm — ~2–3× ClickHouse's ~3 ms,
not 18×.** Broad-term (`users`, 333k) is ~85 ms (~12×, scales with matched rows = real
scan-engine territory, Improvement #2).

**Consequence:** Improvement #1's user story — *an SRE greps for a request-id during an
incident* — is an **exact-term selective** search, and GreptimeDB already serves it in **~8 ms**
with `matches_term()` + the bloom backend. **The big ~18× only hits (a) `matches()`
query-syntax/phrase search on a bloom index (use the tantivy backend for that), or (b)
broad-term analytics.** This **narrows the verdict's one big ClickHouse win** (log search): for
the actual incident-grep pattern the gap is ~2–3× (both sub-perceptible), not a chasm.
Sharpens the verdict + parity-roadmap #1; updated both. No data changed (read-only).

**Caveats:** smoke 5M; `count(*)` shape. **Resolved by Run 49:** the tantivy backend
(`backend='tantivy'`) makes `matches()` query-syntax search prune too (~6 ms selective).
The right Parallax choice is **bloom + `matches_term` for exact-term incident grep** and
**tantivy + `matches` for query-syntax/phrase search**.

### Run 49 — Tantivy backend: `matches()` query-syntax search prunes (~6 ms) — the gap is fully a pairing issue

Answers the question Run 48 left owed: does the **tantivy** fulltext backend make `matches()`
(query-syntax) prune, or is query-syntax log search a real gap? Built a tantivy-backed copy
(`logs_tantivy`, `message FULLTEXT INDEX WITH(backend='tantivy')`, 1M rows from `logs_b1`,
flushed), warm. Dropped after.

| Query on tantivy backend | Result | EXPLAIN scan `output_rows` | Latency (warm) |
| --- | --- | --- | --- |
| `matches('ae119f2b')` selective (1 match) | **pruned** | **1** | **~6 ms** |
| `matches('users')` broad (66,493 matches) | scales | (many) | ~26 ms |

**Finding — the full-text picture is now definitive and the ~18× is fully explained as a
pairing artifact:**

| backend × function | selective behavior | selective latency |
| --- | --- | --- |
| **tantivy + `matches()`** (query syntax) | **prunes** (Run 49) | **~6 ms** |
| **bloom + `matches_term()`** (exact term) | **prunes** (Run 48) | **~8 ms** |
| **bloom + `matches()`** (MISMATCH) | **full-scans 5M** (Run 48) | ~150 ms ← the Run-12 ~18× |
| ClickHouse `hasToken`/`text` | prunes | ~3 ms |

So **with the correct backend for the query type, GreptimeDB selective full-text search is
~6–8 ms (~2× ClickHouse, both sub-perceptible) — on *both* query-syntax and exact-term paths.**
The reported ~18× (Run 12) was 100 % a backend/function misconfiguration (`matches()` on a
bloom index), **not** a fundamental full-text gap. The only residual ClickHouse log advantage
is **broad-term scans matching many rows** (analytics → scan engine, Improvement #2), not
interactive incident search.

**Consequence:** the verdict's one big ClickHouse win (log search) **dissolves for the
interactive/selective case on both query types** given correct backend choice; updated the
verdict + roadmap #1 accordingly. Parallax guidance: **tantivy backend for query-syntax/phrase
log search, bloom backend for exact-term grep** — both ~6–8 ms. Caveat: smoke (1M tantivy / 5M
bloom), `count(*)` shape; cold-cache GB-scale still owed to the harness. Cleanup: `logs_tantivy`
dropped; bench data untouched.

### Run 50 — 2026-05-25 — Re-verification sweep of the headline claims + a fairness correction (experimental-as-stable)

**Why this run.** Operator clarified two durable rules: (a) count ClickHouse's
*experimental* observability as **stable** — judge on mechanism + future trajectory,
do not maturity-shame; (b) every comparison statement is a theory to re-verify on the
live stack. This run re-checks the load-bearing numbers against the **same containers
that have been up 7 h** (so it also tests warm-steady-state stability), and corrects
what no longer holds.

**Environment**

| Item | Value |
| --- | --- |
| Host | Linux dev box (orbstack); Docker 29.5.0, compose v5.1.3 |
| Stack | `bench/compose.yml`, both containers `(healthy)` for ~7 h |
| GreptimeDB | `greptime/greptimedb:v1.0.2` (`0ef5451`) standalone |
| ClickHouse | `clickhouse/clickhouse-server:26.5.1.882` (`5b96a8d8`) |
| Access | `docker exec` (sandbox blocks host→container ports — confirmed: host `curl localhost:8123/9000/4000` all refused; exec works). CH via `clickhouse-client --time`; GT via HTTP `/v1/sql` `execution_time_ms`; PromQL via `/v1/prometheus/api/v1/query[_range]`, wall-clock with an exec+curl baseline subtracted. |
| Datasets (pre-loaded, unchanged) | `metrics_hc` 8 M (40k series), `logs_b1` 5 M (bloom FULLTEXT on `message`), `spans` 1 M + `spans_idx` 1 M (trace_id `INVERTED`), `metrics_real` 864 k. |

**A. Metric aggregation `avg(value) by service` on `metrics_hc` (8 M), warm, min of 3**

- CH SQL: `SELECT service,avg(value) FROM metrics_hc GROUP BY service FORMAT Null` → **31–33 ms**
- GT SQL: same SQL via `/v1/sql` → **105–113 ms** (server)
- GT PromQL: `avg by(service)(metrics_hc)` instant → **~595 ms** server-equivalent (≈650 ms wall − ≈55 ms exec/curl baseline)

→ Ordering **CH SQL > GT SQL (~3.3×) > GT PromQL (~5.4×)** — **confirms** Run 37 (warm ~2–3×) and Run 44 (native PromQL ~5× slower than GT SQL; `SeriesNormalize`/`SeriesDivide` fixed setup). Mechanism holds. *Fairness note:* ClickHouse's PromQL-equivalent runs on the **fast SQL engine** (`timeSeries*ToGrid`, below), so counting CH's experimental metrics path as stable makes CH **stronger** on metric-agg latency, not weaker.

**B. Selective full-text log search on `logs_b1` (5 M), term `0835d162` (matches exactly 1 row), warm, min of 3**

- CH `text` index: `WHERE hasToken(message,'0835d162')` → **~4 ms**
- GT bloom + `matches_term(message,'0835d162')` (exact-term fn) → **~9–11 ms** (prunes)
- GT bloom + `matches(message,'0835d162')` (query-syntax fn on a *bloom* index) → **~152 ms** (full-scan)

→ **Confirms Runs 48–49 exactly**: CH ~4 ms vs GT ~10 ms (~2.5×, *not* 18×) for the real incident-grep shape; the old "18×" reproduces **only** with the wrong fn/backend pairing (`matches()` on bloom). The dissolution of the full-text gap stands.

**C. `trace_id` point lookup, warm, min of 3** (`f6a4d02…` = 14 spans)

- CH `spans` (`ORDER BY (trace_id,ts)` → PK seek) → **~3 ms**
- GT `spans_idx` (`trace_id INVERTED`) → **~16 ms** (prunes, scattered reads)
- GT `spans` (trace_id un-indexed) → **~35 ms** (scan)

→ **Confirms Run 1 / Run 40**: CH wins via clustered-PK locality; GT competitive only with the inverted index, still ~4–5× CH — but both ≪ the 300 ms gate. Schema-discipline point holds.

**D. ClickHouse experimental observability — verified live, counts as stable**

| Feature | Probe | Result |
| --- | --- | --- |
| `TimeSeries` engine | `CREATE TABLE … ENGINE=TimeSeries` (flag on) | **builds** — DATA(MergeTree)+TAGS(AggregatingMergeTree)+METRICS(ReplacingMergeTree) |
| `prometheusQuery` / `prometheusQueryRange` | `system.table_functions` | **present** (they are **table functions**, not in `system.functions` — earlier-pass naming is correct; a `system.functions`-only search misses them, noted so future passes don't mis-correct) |
| `timeSeries*ToGrid` family | `system.functions` | **12 fns**: rate, delta, instant rate/delta, deriv, predict_linear, changes, resets, resample-with-staleness, last, last-two — broader than "rate/delta/increase" |
| PromQL-style rate, executed | `timeSeriesRateToGrid(...)(ts, toFloat64(counter))` on `metrics_real` | **returns correct per-service rate grid**, `NULL` first bucket (no prior sample) — works |
| `JSON` typed subcolumn | `attrs.\`http.status\`.:Int64` group-by | **reads typed subcolumn** correctly |
| `async_insert` | `system.settings` | **`=1` (DEFAULT ON)**, 10 MiB / 200 ms flush, `wait_for_async_insert=1` |
| lightweight `DELETE` / `UPDATE` | `system.settings` | delete GA (`lightweight_deletes_sync=2`); experimental update flag present |
| native OTLP receiver | functions + config scan | **none** — OTLP ingest is genuinely collector-mediated (this point stands) |

GreptimeDB symmetric check: `/v1/prometheus/api/v1/query_range` returns a proper Prometheus **matrix**; `count(metrics_real)` parses as PromQL — real PromQL-**language** HTTP endpoint, drop-in for Grafana.

**Corrections this run produces (applied to the notes):**

1. **`async_insert` is default-on in 26.5.1** → the verdict's "ClickHouse needs an ingest-batching layer for streaming small writes" is **overstated**: server-side batching is built in and on by default. Re-stated as "tune/confirm async-insert," not "build a batching layer."
2. **CH PromQL is not "limited to rate/delta/increase"** → it executes arbitrary PromQL via `prometheusQuery[Range]` table functions *and* exposes 12 `timeSeries*ToGrid` aggregate primitives. Verdict wording corrected.
3. The honest distinction that **survives** (not a maturity penalty, a mechanism/integration fact): GreptimeDB = PromQL-**language** over the standard Prometheus HTTP API (drop-in Grafana datasource); ClickHouse = PromQL-**equivalent computation in SQL** + table functions (capable, runs on the fast engine, but not a PromQL-string HTTP endpoint, and OTLP ingest is collector-mediated).

**Reproduce (copy-paste).**

```bash
# A — metric agg
docker exec parallax-bench-clickhouse-1 clickhouse-client --time -q \
  "SELECT service,avg(value) FROM metrics_hc GROUP BY service FORMAT Null"
docker exec parallax-bench-greptimedb-1 curl -s "http://localhost:4000/v1/sql?db=public" \
  --data-urlencode "sql=SELECT service,avg(value) FROM metrics_hc GROUP BY service" | grep -o '"execution_time_ms":[0-9]*'
docker exec parallax-bench-greptimedb-1 curl -s "http://localhost:4000/v1/prometheus/api/v1/query" \
  --data-urlencode "query=avg by(service)(metrics_hc)" --data-urlencode "time=2024-05-18T03:00:00Z" -o /dev/null
# B — full-text (term matches 1 row)
docker exec parallax-bench-clickhouse-1 clickhouse-client --time -q \
  "SELECT count() FROM logs_b1 WHERE hasToken(message,'0835d162') FORMAT Null"
docker exec parallax-bench-greptimedb-1 curl -s "http://localhost:4000/v1/sql?db=public" \
  --data-urlencode "sql=SELECT count(*) FROM logs_b1 WHERE matches_term(message,'0835d162')" | grep -o '"execution_time_ms":[0-9]*'
docker exec parallax-bench-greptimedb-1 curl -s "http://localhost:4000/v1/sql?db=public" \
  --data-urlencode "sql=SELECT count(*) FROM logs_b1 WHERE matches(message,'0835d162')" | grep -o '"execution_time_ms":[0-9]*'
# C — trace lookup
docker exec parallax-bench-clickhouse-1 clickhouse-client --time -q \
  "SELECT span_id,service,name FROM spans WHERE trace_id='f6a4d0239985efee1cfd72928e65e92a' ORDER BY ts FORMAT Null"
docker exec parallax-bench-greptimedb-1 curl -s "http://localhost:4000/v1/sql?db=public" \
  --data-urlencode "sql=SELECT span_id,service,name FROM spans_idx WHERE trace_id='f6a4d0239985efee1cfd72928e65e92a' ORDER BY ts" | grep -o '"execution_time_ms":[0-9]*'
# D — CH experimental obs (counts as stable)
docker exec parallax-bench-clickhouse-1 clickhouse-client -q \
  "SELECT name FROM system.table_functions WHERE name ILIKE '%prometh%'"
docker exec parallax-bench-clickhouse-1 clickhouse-client --allow_experimental_time_series_aggregate_functions=1 -q \
  "SELECT service, timeSeriesRateToGrid(toDateTime64('2024-05-18 02:40:00',3), toDateTime64('2024-05-18 03:40:00',3), INTERVAL 600 SECOND, INTERVAL 600 SECOND)(ts, toFloat64(counter)) FROM metrics_real WHERE service='svc-0' GROUP BY service"
docker exec parallax-bench-clickhouse-1 clickhouse-client -q \
  "SELECT name,value FROM system.settings WHERE name='async_insert'"
```

Caveat: all warm, cache-resident smoke (1–8 M rows). Directional only; cold-cache GB–TB and concurrent ingest+query stay owed to the sized harness.

### Run 51 — 2026-05-25 — Full-text index *storage* cost, fair inverted-vs-inverted (the "9×" was a bloom-vs-text artifact)

**Pass target.** Rotate the re-verification slice off latency (Run 50 swept the
latency headlines) onto the **cost axis**: how much disk does each system's
full-text index cost for the same log corpus? The verdict's cost note
(`compression-and-cost.md`) measured *column* compression (a wash) but never the
*full-text index* — a major log-storage cost. Naive reading of the live tables
(ClickHouse `text` 170 MiB vs GreptimeDB `logs_b1` full-text 18 MiB) suggests a ~9×
GreptimeDB win — but that compares ClickHouse's **inverted** index against
GreptimeDB's **bloom**-backend full-text. That is exactly the apples-to-oranges
trick the brief forbids. This run builds the fair inverted-vs-inverted comparison.

**Environment**

| Item | Value |
| --- | --- |
| Host | Linux container dev box (orbstack); same as Runs 1–50 |
| Compose | `bench/compose.yml` (local disk) |
| GreptimeDB | `greptime/greptimedb:v1.0.2` (`0ef5451`) — standalone, default config |
| ClickHouse | `clickhouse/clickhouse-server:26.5.1.882` (`5b96a8d8`) |
| Versions re-pinned this pass | GreptimeDB latest GA = `v1.0.2` (newer tags are `v1.1.0-nightly`/`v1.0.0-nightly` only); ClickHouse latest stable feature line = `v26.5.1.882-stable` (later-dated `v26.2.19.43`/`v26.4.3.37` are lower-line backports, not higher). **No bump.** |
| Dataset | The existing `logs_b1` corpus: **5,000,000 log rows**, `message` = high-entropy text (embedded UUIDs/IDs/latencies + stack traces). Identical bytes on both sides (the GreptimeDB tantivy variant is `INSERT … SELECT`-copied from `logs_b1`). |
| Measurement | Metadata only (stable, not timing): ClickHouse `system.parts` (`bytes_on_disk`, `data_compressed_bytes`) + `system.data_skipping_indices` (`data_compressed_bytes`); GreptimeDB `information_schema.region_statistics` (`sst_size`, `index_size`, `disk_size`). All tables compacted to **1 SST/part** so the comparison is segment-matched. |

**Schema under test (full-text index on `message`, copy-paste):**

```sql
-- ClickHouse: true inverted posting-list index
INDEX idx_msg message TYPE text(tokenizer = splitByNonAlpha) GRANULARITY 100000000
-- (table: ENGINE=MergeTree ORDER BY (service, ts); message String CODEC(ZSTD(1)))

-- GreptimeDB A — bloom backend (probabilistic, fpr=0.01) — the live logs_b1
"message" STRING NULL FULLTEXT INDEX WITH(analyzer='English', backend='bloom',
  case_sensitive='false', false_positive_rate='0.01', granularity='10240')

-- GreptimeDB B — tantivy backend (true inverted, Lucene-class) — built this run
CREATE TABLE "logs_b1_tan" (... "message" STRING NULL
  FULLTEXT INDEX WITH(analyzer='English', backend='tantivy', case_sensitive='false'),
  ... TIME INDEX("ts"), PRIMARY KEY("service","level")) ENGINE=mito WITH(append_mode='true');
INSERT INTO "logs_b1_tan" SELECT * FROM "logs_b1";   -- 5M rows, 6.8 s
ADMIN flush_table('logs_b1_tan'); ADMIN compact_table('logs_b1_tan');  -- settle to 1 SST
```

**Measured (5M identical log rows, 1 SST/part each):**

| Full-text index | column/SST data | full-text index | total on disk | index overhead on data | index size vs CH |
| --- | --- | --- | --- | --- | --- |
| **ClickHouse `text`** (inverted, `splitByNonAlpha`) | 228.2 MiB | **170.4 MiB** | 399.2 MiB | ~75% | 1.0× (baseline) |
| **GreptimeDB tantivy** (inverted, Lucene-class) | 239.9 MiB | **148.3 MiB** | 388.2 MiB | ~62% | **0.87× (13% smaller)** |
| **GreptimeDB bloom** (probabilistic full-text, fpr=0.01) | 239.8 MiB | **18.1 MiB** | 258.0 MiB | ~7.5% | 0.11× (9.4× smaller) |

**Method notes / honesty.**

- **1-SST gate matters.** tantivy builds one index per SST; pre-compaction the
  variant showed 7 SSTs / idx 108 MiB, then transiently 3 SSTs / 149 MiB
  (mid-compaction double-count of old+new puffin sidecars). Only the **settled
  1-SST reading (148.3 MiB)** is reported — matching `logs_b1`'s 1-SST bloom state.
- ClickHouse `bytes_on_disk` (399.2 MiB) is authoritative for total; `system.parts_columns`
  `data_compressed_bytes` summed to 1.34 GiB because that column is part-level
  repeated per column (6 cols) — **do not sum it**; use `system.parts.data_compressed_bytes`
  (228.2 MiB, columns only, excludes skip indexes).

**The comparison logic & verdict.**

- **What this isolates:** the *storage* cost of full-text indexing for logs, held
  on an identical corpus, segment-matched (1 SST/part), index-family-matched for the
  fair cell (inverted vs inverted).
- **Column data is parity** (CH 228 MiB vs GT 240 MiB; CH ~5% smaller from tuned
  `ZSTD`+`LowCardinality` vs GreptimeDB Parquet defaults — consistent with Run 10).
- **Fair inverted-vs-inverted: GreptimeDB tantivy is ~13% smaller than ClickHouse
  `text` (148 vs 170 MiB).** Both true inverted indexes cost **60–75% on top of the
  column data** — full-text indexing is expensive on *both* engines. The headline
  "~9× smaller" is **REFUTED as an inverted-index claim** — it only appears when
  comparing CH inverted against GT *bloom*.
- **The real cost lever is GreptimeDB's bloom-backend full-text:** ~7.5% overhead
  (18 MiB) vs ~75% for an inverted index, i.e. **~9× smaller index** — and Run 49
  measured it at ~8 ms exact-term (`matches_term`). For Parallax's *anchored* log
  search (exact request-id/trace-id grep), this is a genuine cost-axis win with a
  capability tradeoff (probabilistic, 1% false positive, re-checked at scan; no
  ranking/phrase). **Status: confirmed (fair inverted compare); the "9×" headline
  refuted as inverted, recharacterized as a bloom-vs-inverted *capability/cost
  tradeoff*.**

**Owed for full symmetry (do NOT claim until measured):** build a ClickHouse
`tokenbf_v1` bloom skip-index variant and compare **bloom-vs-bloom** — only then is
the bloom-tier size claim symmetric. ClickHouse's bloom token filter is the fair
analog to GreptimeDB's bloom full-text; this run measured CH only on its `text`
(inverted) index. Routed to `benchmarking-the-differences.md`.

**Reproduce (copy-paste).**

```bash
# ClickHouse: total, column-data, and text-index bytes (authoritative)
docker exec parallax-bench-clickhouse-1 clickhouse-client -q \
  "SELECT formatReadableSize(sum(bytes_on_disk)) total, formatReadableSize(sum(data_compressed_bytes)) cols FROM system.parts WHERE active AND database='default' AND table='logs_b1'"
docker exec parallax-bench-clickhouse-1 clickhouse-client -q \
  "SELECT name, formatReadableSize(sum(data_compressed_bytes)) FROM system.data_skipping_indices WHERE database='default' AND table='logs_b1' GROUP BY name"
# GreptimeDB: build tantivy variant over the same corpus, settle to 1 SST, measure
docker exec parallax-bench-greptimedb-1 curl -s "http://localhost:4000/v1/sql?db=public" \
  --data-urlencode "sql=CREATE TABLE IF NOT EXISTS \"logs_b1_tan\" (\"ts\" TIMESTAMP(3) NOT NULL, \"service\" STRING NULL, \"level\" STRING NULL, \"message\" STRING NULL FULLTEXT INDEX WITH(analyzer='English', backend='tantivy', case_sensitive='false'), \"trace_id\" STRING NULL, \"span_id\" STRING NULL, TIME INDEX(\"ts\"), PRIMARY KEY(\"service\",\"level\")) ENGINE=mito WITH(append_mode='true')"
docker exec parallax-bench-greptimedb-1 curl -s "http://localhost:4000/v1/sql?db=public" \
  --data-urlencode "sql=INSERT INTO \"logs_b1_tan\" SELECT * FROM \"logs_b1\""
docker exec parallax-bench-greptimedb-1 curl -s "http://localhost:4000/v1/sql?db=public" --data-urlencode "sql=ADMIN flush_table('logs_b1_tan')"
docker exec parallax-bench-greptimedb-1 curl -s "http://localhost:4000/v1/sql?db=public" --data-urlencode "sql=ADMIN compact_table('logs_b1_tan')"   # repeat until sst_num=1
docker exec parallax-bench-greptimedb-1 curl -s "http://localhost:4000/v1/sql?db=public" \
  --data-urlencode "sql=SELECT t.table_name, r.region_rows, r.sst_size, r.index_size, r.disk_size, r.sst_num FROM information_schema.region_statistics r JOIN information_schema.tables t ON r.table_id=t.table_id WHERE t.table_name IN ('logs_b1','logs_b1_tan')"
docker exec parallax-bench-greptimedb-1 curl -s "http://localhost:4000/v1/sql?db=public" --data-urlencode "sql=DROP TABLE \"logs_b1_tan\""   # cleanup scratch
```

Caveat: warm, cache-resident smoke; metadata sizes are exact and stable (not
timing). 5M-row single-node laptop scale — directional for cost ratios, not a
production retention-bill verdict.

### Run 52 — 2026-05-25 — Bloom-vs-bloom full-text, fair 1% fpr (corrects Run 51's "no CH equivalent" over-claim)

**Pass target.** Close the symmetry Run 51 owed: compare GreptimeDB's bloom-backend
full-text against ClickHouse's bloom token filter (`tokenbf_v1`) — the fair
bloom-vs-bloom cell — and check whether the cheap full-text option is really a
GreptimeDB-only cost lever (Run 51's tentative claim) or exists equally on both.

**Environment.** Same as Run 51 (GreptimeDB `v1.0.2` `0ef5451`, ClickHouse
`v26.5.1.882` `5b96a8d8`, `bench/compose.yml` local disk). Versions re-pinned this
pass — both still latest GA/stable, no bump. Same identical 5M-row `logs_b1` corpus
(`message` ≈ 9.85 tokens/row, ~6.76M distinct tokens globally; **27,062 distinct
tokens per 8192-row granule** — measured, this drives bloom sizing).

**Schema under test (bloom full-text, copy-paste):**

```sql
-- GreptimeDB bloom backend (the live logs_b1), fpr=0.01, 10240-row blocks
"message" STRING NULL FULLTEXT INDEX WITH(analyzer='English', backend='bloom',
  case_sensitive='false', false_positive_rate='0.01', granularity='10240')

-- ClickHouse tokenbf_v1 — sized for ~1% fpr at n≈27k tokens/granule:
--   m = -n·ln(0.01)/(ln2)^2 ≈ 259k bits ≈ 32 KB; k = (m/n)·ln2 ≈ 7
INDEX idx_msg_tbf message TYPE tokenbf_v1(32768, 7, 0) GRANULARITY 1   -- 8192-row granules
```

**The sizing-fairness correction (why the first attempt was a trick).** First build
used `tokenbf_v1(98304, 6, 0)` on a *guessed* n≈80k tokens/granule → index **57.5
MiB**, pruning **1/611** granules. But the *measured* distinct tokens/granule is
**27,062**, not 80k (most of the 80,690 raw tokens repeat — common words, levels,
services; only UUIDs are unique). So 98 KB/granule was **~3× oversized** (fpr ≪ 1%) —
an unfair, over-provisioned filter. Resized to the math-correct **32 KB/granule** for
genuine ~1% fpr and re-measured. *This is the no-tricks rule applied to my own prior
pass: the 57 MiB number was an artifact of my sizing, not an engine property.*

**Measured (5M identical log rows, fair ~1% fpr, warm):**

| Full-text index | type | index size | exact-term latency | granules pruned (unique term) |
| --- | --- | --- | --- | --- |
| ClickHouse `text` | inverted | 170.4 MiB | **3 ms** | exact (posting list) |
| GreptimeDB tantivy | inverted | 148.3 MiB | ~6 ms (Run 49) | exact (posting list) |
| **ClickHouse `tokenbf_v1`** (1% fpr) | **bloom** | **19.2 MiB** | **8 ms** | **3/611** (1 true + 2 fp) |
| **GreptimeDB bloom** (1% fpr) | **bloom** | **18.1 MiB** | **9 ms** | block-bloom (Run 49 ~8 ms) |

Term `0835d162` matches exactly 1 row on both (correctness anchor). CH `tokenbf`
latency went 18 ms (oversized 98 KB) → **8 ms** (fair 32 KB) — fewer bytes to load
per probed granule, same pruning quality (3/611).

**The comparison logic & verdict.**

- **Bloom-vs-bloom is a TIE.** At matched ~1% fpr: ClickHouse `tokenbf_v1` **19.2
  MiB / 8 ms** vs GreptimeDB bloom **18.1 MiB / 9 ms**. Bloom-filter size at a fixed
  fpr is governed by distinct-token count (pure math: `m ≈ 9.585·n` bits for 1%),
  which is ~equal on the same corpus — so **neither engine has a bloom-tier size or
  speed advantage.** **Status: Run 51's "GreptimeDB bloom is the cost lever with no
  managed CH equivalent" is REFUTED / CORRECTED — ClickHouse's equal-cost equivalent
  is `tokenbf_v1` (or `ngrambf_v1`).**
- **The real axis is index *family*, identical on both engines:** *inverted*
  (148–170 MiB, ~60–75% overhead, 3–6 ms exact-term, supports phrase/ranking) **vs**
  *bloom* (~18–19 MiB, ~8% overhead, 8–9 ms, token-membership only, probabilistic).
  Bloom is ~9× smaller and ~2–3× slower than inverted — **on both engines.** Choosing
  bloom over inverted saves ~55–65% of total log-table size at a ~2–3× exact-term
  latency cost — a real cost/latency lever, but **engine-neutral.**
- **What survives as a GreptimeDB nuance (ergonomics, not cost/speed):** GreptimeDB
  exposes both tiers behind one `FULLTEXT INDEX WITH(backend=bloom|tantivy)` knob
  *with analyzer/case/phrase semantics*; ClickHouse splits them — `text` (inverted,
  GA, ranking/phrase) vs `tokenbf_v1`/`ngrambf_v1` (bloom *skip-index*, token-only,
  no analyzer-class features). Capability/ergonomics difference, not a storage-cost
  or latency one. Feeds `compression-and-cost.md` (corrected) and `indexing-internals.md`.

**Reproduce (copy-paste).**

```bash
# measured distinct tokens per granule (drives bloom sizing — do this first)
docker exec parallax-bench-clickhouse-1 clickhouse-client -q \
  "SELECT uniqExact(arrayJoin(splitByNonAlpha(message))) FROM (SELECT message FROM logs_b1 ORDER BY service, ts LIMIT 8192)"
# CH fair-sized bloom: build, size, prune, time
docker exec parallax-bench-clickhouse-1 clickhouse-client --multiquery -q "
DROP TABLE IF EXISTS logs_b1_tbf;
CREATE TABLE logs_b1_tbf (ts DateTime64(3) CODEC(DoubleDelta,ZSTD(1)), service LowCardinality(String),
  level LowCardinality(String), message String CODEC(ZSTD(1)), trace_id String CODEC(ZSTD(1)),
  span_id String CODEC(ZSTD(1)), INDEX idx_msg_tbf message TYPE tokenbf_v1(32768, 7, 0) GRANULARITY 1)
ENGINE=MergeTree ORDER BY (service, ts) SETTINGS index_granularity=8192;
INSERT INTO logs_b1_tbf SELECT * FROM logs_b1; OPTIMIZE TABLE logs_b1_tbf FINAL;"
docker exec parallax-bench-clickhouse-1 clickhouse-client -q "SELECT name, formatReadableSize(sum(data_compressed_bytes)) FROM system.data_skipping_indices WHERE database='default' AND table='logs_b1_tbf' GROUP BY name"
docker exec parallax-bench-clickhouse-1 clickhouse-client -q "EXPLAIN indexes=1 SELECT count() FROM logs_b1_tbf WHERE hasToken(message,'0835d162')"
docker exec parallax-bench-clickhouse-1 clickhouse-client --time -q "SELECT count() FROM logs_b1_tbf WHERE hasToken(message,'0835d162') FORMAT Null"
docker exec parallax-bench-clickhouse-1 clickhouse-client -q "DROP TABLE logs_b1_tbf"  # cleanup
# GreptimeDB bloom (live logs_b1)
docker exec parallax-bench-greptimedb-1 curl -s "http://localhost:4000/v1/sql?db=public" \
  --data-urlencode "sql=SELECT count(*) FROM logs_b1 WHERE matches_term(message,'0835d162')" | grep -o '"execution_time_ms":[0-9]*'
```

Caveat: warm, cache-resident smoke; sizes are exact metadata. Bloom sizing depends on
distinct-token count, so the size *tie* generalizes (it's fpr math); the precise MiB
scales with corpus token cardinality.

### Run 53 — 2026-05-25 — Concurrent ingest+query penalty, re-verified (the production state)

**Pass target.** Rotate off cost/full-text onto the verdict's **#1 axis: does query
latency hold under concurrent ingest?** Re-verify Run 13's load-bearing "neither
engine blocks reads on ingest" (Run 13 measured CH 1.55× / GT 1.38× at 11M rows,
~40 passes ago) against the live containers at higher write pressure.

**Environment.** GreptimeDB `v1.0.2` (`0ef5451`), ClickHouse `v26.5.1.882`
(`5b96a8d8`), `bench/compose.yml` local disk. Versions re-pinned this pass — both
still latest GA/stable, no bump. Tables: `metrics_hc` (8M, scan-agg query),
`spans`/`spans_idx` (1M, anchored point lookup). Warm.

**Method.** Each engine tested **in isolation** (one engine ingesting+querying itself
at a time — avoids cross-engine host-CPU confounding). Baseline = query ×7 median
with no ingest. Then a background loop ran `INSERT INTO ingest_load SELECT … LIMIT
200000` back-to-back for ~24 s while the same query ran ×10 (median). `ingest_load`
is a spans-shaped scratch table (CH `AS spans`; GreptimeDB 7-col, `PRIMARY KEY(trace_id)`).
Penalty = during-ingest median ÷ baseline median.

| Query | Engine | baseline | during ingest | **penalty** |
| --- | --- | --- | --- | --- |
| metric-agg (`GROUP BY service` over 8M) | ClickHouse | 32 ms | 36 ms | **1.13×** |
| metric-agg | GreptimeDB | 100 ms | 119 ms | **1.19×** |
| anchored lookup (`trace_id=…`) | ClickHouse | 2 ms | 2 ms | **1.0×** |
| anchored lookup | GreptimeDB | 13 ms | 15 ms | **1.15×** |

**Achieved write load during the window (NOT matched — see caveat):**

| Engine | batches ×200k | submitted rows | ~rows/s | write-path state after |
| --- | --- | --- | --- | --- |
| ClickHouse | 173 | 34.6M (all retained) | **~1.44M/s** | **17 active parts** (merges paced it — no part explosion) |
| GreptimeDB | 68 | 13.6M submitted | ~567k/s submitted | **1 SST + 538 MiB memtable** (LSM absorbed; deduped to ~3.7M retained, PK=trace_id) |

**The comparison logic & verdict.**

- **Confirmed: neither engine blocks reads under concurrent ingest.** All penalties
  are **1.0–1.19×**, well under the ≤2× gate — *tighter* than Run 13's 1.38–1.55×.
  The load-bearing "stays queryable under load" claim **still reproduces.** Status:
  **confirmed** (re-verified, drift = penalties even lower at this scale).
- **Mechanism, per query shape:** the **anchored point lookup is ~immune** (CH 1.0×
  index seek; GT 1.15× index seek) — Parallax's *hot path stays flat even under
  ingest*, reinforcing the "anchored bundle not latency-bound" verdict pillar. The
  **scan-agg absorbs the contention** (CH 1.13×, GT 1.19×) because it shares CPU with
  background merge (CH) / memtable+dedup work (GT).
- **ClickHouse degraded slightly *less* while under ~2.5× heavier achieved write
  load** (1.44M vs 567k rows/s) — its vectorized scan + paced merges (17 parts, no
  explosion) handled concurrency at least as well as GreptimeDB's LSM here. But the
  loads were **not matched**, so this is *not* a clean head-to-head penalty ratio —
  only each engine vs its own baseline is apples-to-apples.

**Fairness caveats (honesty).**

1. **Loads not matched.** `INSERT…SELECT` is server-side and throttle-free, so each
   engine ran as fast as it could — CH pushed more rows/s. A clean penalty *comparison*
   needs both throttled to an identical rows/s. **Routed to the harness** (add a
   rate-limited concurrent-load generator).
2. **GreptimeDB deduped on ingest** (`PRIMARY KEY(trace_id)`, same 200k rows re-read
   each batch → last-row-wins overwrite), so retained ≠ submitted; the *write work*
   (parse + memtable + dedup) still applied as load, but rows/s is "submitted", not
   "net new".
3. Single-node laptop smoke, warm. Directional; cold + sized + matched-rate concurrency
   is the harness's job.

**Reproduce (copy-paste).**

```bash
# scratch ingest targets
docker exec parallax-bench-clickhouse-1 clickhouse-client -q "CREATE TABLE ingest_load AS spans"
docker exec parallax-bench-greptimedb-1 curl -s "http://localhost:4000/v1/sql?db=public" --data-urlencode 'sql=CREATE TABLE ingest_load ("trace_id" STRING,"span_id" STRING,"service" STRING,"name" STRING,"ts" TIMESTAMP(3) TIME INDEX,"duration_ms" DOUBLE,"status" STRING, PRIMARY KEY("trace_id"))'
# CH: background ingest ~24s, foreground query x10 (repeat for GreptimeDB via HTTP /v1/sql)
( end=$((SECONDS+24)); while [ $SECONDS -lt $end ]; do docker exec parallax-bench-clickhouse-1 clickhouse-client -q "INSERT INTO ingest_load SELECT * FROM spans LIMIT 200000"; done ) &
for i in $(seq 10); do docker exec parallax-bench-clickhouse-1 clickhouse-client --time -q "SELECT service,avg(value) FROM metrics_hc GROUP BY service FORMAT Null"; done
wait
docker exec parallax-bench-clickhouse-1 clickhouse-client -q "DROP TABLE ingest_load"
docker exec parallax-bench-greptimedb-1 curl -s "http://localhost:4000/v1/sql?db=public" --data-urlencode 'sql=DROP TABLE ingest_load'
```

Caveat: warm smoke, unmatched load (see caveats). The *direction* (neither blocks,
point lookup immune, scan absorbs contention) is robust; precise penalty ratios await
matched-rate harness runs.

### Run 54 — 2026-05-25 — Object-store object-count re-verified (the cost-axis pillar), + a size-order reversal

**Pass target.** Re-verify the stalest verdict-critical claim: GreptimeDB writes
~order-of-magnitude fewer S3 objects than ClickHouse (Runs 8–9, ~pass 18–19:
GreptimeDB 4 obj / 37 MiB vs ClickHouse 74 obj / 63 MiB for 1M spans) — the
load-bearing evidence behind the "object-store-native economics" recommendation, since
per-request pricing dominates a re-read-heavy bill.

**Environment.** Brought up the isolated S3 stack `bench/s3/run-s3-stack.sh up` (MinIO
+ GreptimeDB(S3) + ClickHouse(S3) on net `pbench-s3`, separate from the main local-disk
stack). GreptimeDB `v1.0.2` (`[storage] type=S3`), ClickHouse `v26.5.1.882`
(`storage_policy='s3only'`, s3 disk). Versions re-pinned this pass — both latest, no
bump. **Dataset:** the identical 1M-span set dumped from the main ClickHouse `spans`
(`FORMAT CSVWithNames`, 8 cols, ~129 MB) and loaded into *both* S3 instances (CH
`INSERT … FROM INFILE … CSVWithNames`; GreptimeDB `COPY … WITH(FORMAT='CSV')`). Both
verified `count()=1,000,000`. GreptimeDB `PRIMARY KEY(trace_id)`, CH `ORDER BY
(trace_id, ts)`.

**Measured (MinIO `mc ls --recursive | wc -l` + `mc du`, after GreptimeDB
`flush_table` / ClickHouse `OPTIMIZE FINAL`):**

| | object count | raw S3 bytes | active logical | active parts |
| --- | --- | --- | --- | --- |
| **GreptimeDB** | **3 objects** | 21 MiB | 21.8 MiB (1 SST) | 1 region/SST |
| **ClickHouse** | **74 objects** | 57 MiB | **28.9 MiB** (1 active part) | 1 active (+1 un-GC'd → 2 total) |

**The comparison logic & verdict.**

- **Object count CONFIRMED — reproduces strongly.** ClickHouse **74 objects** is
  *identical* to Run 9; GreptimeDB **3** (Run 8 was 4 — one fewer
  metadata/manifest object now). Ratio **~25× fewer** (Run 9 was ~18× at 4 vs 74).
  Mechanism unchanged: ClickHouse **Wide parts write one S3 object per column** (8
  cols) + `.mrk`/checksums/metadata **per part** → ~18–20 objects for a single active
  part, ×N parts until merge-GC; GreptimeDB writes **one Parquet SST** (+ manifest)
  per flush → a handful of objects. **Even fully GC'd** (active part only) ClickHouse
  is ~18–20 objects vs GreptimeDB 3 → still ~6–7×; the 74 includes transient un-GC'd
  merge parts (S3 lazy cleanup — `OPTIMIZE FINAL` left 2 parts on object store).
  **Status: confirmed.** This is the concrete object-store request-efficiency edge for
  GreptimeDB (fewer GET/PUT/LIST on cold reads).
- **New nuance — size order REVERSED vs local disk.** Active logical: **GreptimeDB
  21.8 MiB < ClickHouse 28.9 MiB** (GreptimeDB ~25% smaller) — *opposite* to Run 1
  (local disk, `PK(service,name)`: CH 28.9 < GreptimeDB 38). Cause: `PRIMARY
  KEY(trace_id)` sorts the data by `trace_id`, clustering the high-cardinality hex
  `trace_id`/`span_id`/`parent_span_id` columns so Parquet dictionary/RLE + ZSTD
  compress them far better than the `service`-sorted layout did. Confirms the
  "compression is sort-order/pattern dependent, not a blanket engine win" finding
  (`compression-and-cost.md`): GreptimeDB on its anchored-retrieval schema (trace_id PK,
  which Parallax wants anyway) is also the smaller *and* the more object-efficient on S3.
- ClickHouse active size (28.9 MiB) is *byte-for-byte* the main-stack local-disk spans
  size (Run 1) — the s3 disk stores the same compressed column files, just as S3 objects.

**Caveat / owed.** Object COUNT is measured; the **request-count on a cold read**
(GET/LIST per query) is the number that actually prices a re-read-heavy engine — still
owed (B10, `mc admin trace` / MinIO audit). Fewer objects strongly implies fewer GETs
but is not the same measurement. Single-node smoke, 1M rows.

**Reproduce (copy-paste).**

```bash
bench/s3/run-s3-stack.sh up
docker exec parallax-bench-clickhouse-1 clickhouse-client -q "SELECT * FROM spans FORMAT CSVWithNames" > /tmp/spans.csv
docker cp /tmp/spans.csv pbench-ch-s3:/spans.csv; docker cp /tmp/spans.csv pbench-gt-s3:/spans.csv
docker exec pbench-ch-s3 clickhouse-client -q "CREATE TABLE spans (ts DateTime64(3) CODEC(DoubleDelta,ZSTD(1)), trace_id String, span_id String, parent_span_id String, service LowCardinality(String), name LowCardinality(String), duration_ms Float64, status LowCardinality(String)) ENGINE=MergeTree ORDER BY (trace_id, ts) SETTINGS storage_policy='s3only'"
docker exec pbench-ch-s3 clickhouse-client -q "INSERT INTO spans FROM INFILE '/spans.csv' FORMAT CSVWithNames"; docker exec pbench-ch-s3 clickhouse-client -q "OPTIMIZE TABLE spans FINAL"
docker exec pbench-gt-s3 curl -s "http://localhost:4000/v1/sql?db=public" --data-urlencode 'sql=CREATE TABLE spans ("ts" TIMESTAMP(3) TIME INDEX,"trace_id" STRING,"span_id" STRING,"parent_span_id" STRING,"service" STRING,"name" STRING,"duration_ms" DOUBLE,"status" STRING, PRIMARY KEY("trace_id"))'
docker exec pbench-gt-s3 curl -s "http://localhost:4000/v1/sql?db=public" --data-urlencode "sql=COPY spans FROM '/spans.csv' WITH (FORMAT='CSV')"
docker exec pbench-gt-s3 curl -s "http://localhost:4000/v1/sql?db=public" --data-urlencode "sql=ADMIN flush_table('spans')"
docker run --rm --network pbench-s3 --entrypoint sh minio/mc:latest -c "mc alias set m http://pbench-minio:9000 minioadmin minioadmin; echo GT:; mc ls --recursive m/greptimedb/data/|wc -l; mc du m/greptimedb/data/; echo CH:; mc ls --recursive m/greptimedb/clickhouse/|wc -l; mc du m/greptimedb/clickhouse/"
bench/s3/run-s3-stack.sh down
```

Caveat: warm smoke, 1M spans single-node. Object counts exact; cold-read request
counts (the real $ driver) still owed (B10).

### Run 55 — 2026-05-25 — B10: cold-read S3 request count + egress (corrects the predicted cold-re-read winner)

**Pass target.** Close the number Run 54 owed: the *request count and egress* on a
**cold** anchored read from object storage — the metric that actually prices a
re-read-heavy engine (per-GET + per-GB egress), and the basis for the verdict's
"object-store economics favour GreptimeDB" pillar (so far backed only by object
*count*, not per-query request/egress).

**Environment.** Isolated S3 stack (`bench/s3/run-s3-stack.sh up`): MinIO +
GreptimeDB(S3) `v1.0.2` + ClickHouse(S3) `v26.5.1.882` on net `pbench-s3`. Versions
re-pinned this pass — latest, no bump. Same identical 1M-span dataset as Run 54
loaded into both; GreptimeDB `PRIMARY KEY(trace_id)`, ClickHouse `ORDER BY (trace_id,
ts)`. Query: the anchored lookup `SELECT span_id,service,name FROM spans WHERE
trace_id='0001dd73c341d2b9a2c3fccad1f01beb' ORDER BY ts` (14 rows). S3 requests
captured with `mc admin trace --json` during the query, counted by bucket prefix
(`data/`=GreptimeDB, `clickhouse/`=ClickHouse).

**Forcing cold (per engine — asymmetric levers, both reach true-cold):**

- ClickHouse: `SYSTEM DROP FILESYSTEM CACHE` + `DROP MARK CACHE` + `DROP UNCOMPRESSED
  CACHE`, same process. Query then re-reads from S3.
- GreptimeDB: **first attempt was contaminated** — a `docker restart` *preserved* the
  on-disk read cache (`/greptimedb_data/cache`, 21 MiB = the whole SST,
  write-through-populated on flush), so the query served locally (0 SST GETs, only 544 B
  manifest). True cold required **`rm -rf /greptimedb_data/cache/*` + restart**. *(This
  contamination is itself the finding in the warm row below.)*

**Measured (cold anchored lookup):**

| | S3 GETs | egress | objects read |
| --- | --- | --- | --- |
| **ClickHouse** | **18** GetObject | **294 KiB** (301,308 B) | needed **column granules** only — sparse index → ~1 granule × 5 cols + marks + primary.idx |
| **GreptimeDB** | **9** (1 HeadObject + 4 manifest GETs + **5 SST GETs**) | **~23 MiB** (24,133,371 B on the SST GETs) | ~the **entire 21 MiB Parquet SST** (5 ranged reads of one `.parquet`) + manifest checkpoint/JSONs (region-open, one-time) |
| **GreptimeDB warm** (cache populated — the default after flush) | **0** SST GETs | ~0 | served from persistent local read cache; survived `docker restart` |

Latencies (warm-ish smoke, noise-level): CH cold 45 ms, GreptimeDB cold 44 ms.

**The comparison logic & verdict (two-sided — corrects a prediction).**

- **Request count → GreptimeDB** (9 vs 18, ~2× fewer). Far less than the ~25×
  *object-count* ratio (Run 54): an **anchored** query touches few objects on both, so
  the layout advantage shrinks to ~2× at query time.
- **Cold egress (selective query) → ClickHouse, ~80×** (294 KiB vs ~23 MiB).
  ClickHouse's granule-level reads fetch only the matching granule of each needed
  column; GreptimeDB on a cold cache pulls ~the whole SST. **On per-GB egress pricing
  this reverses the cost story for cold *selective* re-reads.** Status: the
  `caching-and-cold-warm.md` prediction "GreptimeDB wins cold object-store re-read"
  was **too coarse — REFINED to: GreptimeDB wins request count + warm-amortized
  re-reads; ClickHouse wins cold-selective egress.**
- **Warm/repeat → GreptimeDB.** Write-through populates the whole SST into a
  **persistent** local read cache on flush (survives process restart); after first
  touch, re-reads cost ~0 S3 req + 0 egress. For Parallax re-reading **recent** bundles
  this amortizes the one-time cold egress to zero — the dominant economics, favourable.

**Caveats / owed (honesty).**

1. **Small-SST inflates the 80× egress.** 21 MiB SST → GreptimeDB read ~all of it. At
   production SST sizes its Parquet reader should **row-group-prune** (matching row
   groups only), bounding egress — but its row group is **coarser than ClickHouse's
   8192-row granule**, so it will still fetch *more bytes per selective query*, just not
   80×. **The at-scale cold-egress ratio is owed to a larger-SST B10 run** (route to harness).
2. Asymmetric cold levers (CH drop-cache vs GreptimeDB rm-cache+restart) — both reach
   true-cold (verified: CH 18 GETs from S3; GreptimeDB 5 SST GETs from S3), but the
   GreptimeDB number includes one-time region-open manifest GETs (4) that don't recur
   per query.
   ⚠ **Reproduction conflict with Run 14** (which logged anchored cold CH 5 < GT 22 —
   CH *fewer* GETs): Run 55 gets the opposite direction (GT 9 < CH 18). The anchored
   GET *count* is **SST/part-state-dependent and does not reproduce stably** (GreptimeDB
   GETs scale with SST count: 1 compacted SST → 5 ranged reads here vs many SSTs → more
   in Run 14; CH GETs scale with active-part column files). **Treat the egress bytes
   (granules vs whole-SST), not GET count, as the robust cold differentiator.** A number
   that flips between runs is a flagged finding, not a settled one.
3. Single-node smoke, 1M rows, one anchored query shape. A **wide** cold scan (most
   columns) would narrow the egress gap (both read most data) — that is the JSONBench
   regime, which favours GreptimeDB; this run is the *selective* regime, which favours CH.

**Reproduce (copy-paste).**

```bash
bench/s3/run-s3-stack.sh up   # + load 1M spans into both as in Run 54
# ClickHouse cold:
docker exec pbench-ch-s3 clickhouse-client -q "SYSTEM DROP FILESYSTEM CACHE"; docker exec pbench-ch-s3 clickhouse-client -q "SYSTEM DROP MARK CACHE"
docker run --rm --network pbench-s3 --entrypoint sh minio/mc:latest -c "mc alias set m http://pbench-minio:9000 minioadmin minioadmin >/dev/null; timeout 8 mc admin trace --json m" > /tmp/ch.json &
sleep 2.5; docker exec pbench-ch-s3 clickhouse-client -q "SELECT span_id,service,name FROM spans WHERE trace_id='<id>' ORDER BY ts FORMAT Null"; wait
grep '"path":"/greptimedb/clickhouse/' /tmp/ch.json | grep -c GetObject   # = 18
# GreptimeDB TRUE cold (must clear the persistent cache, not just restart):
docker exec pbench-gt-s3 sh -c 'rm -rf /greptimedb_data/cache/*'
docker run --rm --network pbench-s3 --entrypoint sh minio/mc:latest -c "mc alias set m http://pbench-minio:9000 minioadmin minioadmin >/dev/null; timeout 30 mc admin trace --json m" > /tmp/gt.json &
sleep 1.5; docker restart pbench-gt-s3; until docker exec pbench-gt-s3 curl -sf localhost:4000/health >/dev/null 2>&1; do sleep 1; done
docker exec pbench-gt-s3 curl -s localhost:4000/v1/sql?db=public --data-urlencode "sql=SELECT span_id,service,name FROM spans WHERE trace_id='<id>' ORDER BY ts" ; sleep 1.5; kill %1
grep '"path":"/greptimedb/data/' /tmp/gt.json | grep GetObject | grep -o '"size":[0-9]*'   # 5 SST gets ~= 23 MiB
bench/s3/run-s3-stack.sh down
```

Caveat: warm-ish latency noise; the *counts and bytes* are the result. 1M-span
single-node smoke; at-scale selective-cold egress owed to the sized harness.

### Run 56 — 2026-05-25 — Q6 composite evidence-bundle re-verified (the core verdict pillar)

**Pass target.** Rotate off object-store onto the **single most load-bearing claim**:
"Parallax's anchored evidence-bundle hot path is **not latency-bound** on either
engine." Last measured Run 16 (~pass 40); re-verify the composite Q6 (= Q1 + Q2 + Q3
for one anchor) against the live containers to confirm it still reproduces.

**Environment.** Main `bench/compose.yml` stack, local disk. GreptimeDB `v1.0.2`
(`0ef5451`), ClickHouse `v26.5.1.882` (`5b96a8d8`). Versions re-pinned this pass —
latest, no bump. Warm, min-of-7. Tables: spans/spans_idx (1M), logs (214k),
error_events (2,226). **Anchor:** `trace_id=3fb2d84c0a2032fa7681cde05c2051e9`,
`project=parallax`, `fingerprint=fp-000`, `release=v1.7.0` (prev `v1.6.0`).

**Correctness parity (Q1 bundle): PASS** — both return 14 spans + 3 logs + 1 error.

**Measured (warm, min of 7):**

| Sub-query | ClickHouse | GreptimeDB | mechanism |
| --- | --- | --- | --- |
| Q1 trace_context (UNION spans+logs+errors by `trace_id`) | **5 ms** | **21 ms** | GreptimeDB dominated by the `spans_idx` inverted-index `trace_id` lookup floor; CH by `ORDER BY (trace_id,ts)` sparse-index seek |
| Q2 issue_context (`min/max/count` by project+fingerprint) | 2 ms | 3 ms | small keyed agg on error_events — fast on both |
| Q3 release_regression (`NOT IN` anti-join across releases) | 3 ms | 6 ms | sub-query anti-join on 2.2k rows — fast on both |
| **Q6 composite (Q1+Q2+Q3)** | **~10 ms** | **~30 ms** | — |

**Verdict.** **Q6 reproduces — no drift.** Run 16 was CH 10 ms / GT 33 ms; Run 56 is
CH ~10 ms / GT ~30 ms. Both are **far under the 300 ms interactive gate**
(`storage-benchmark-prototype.md`), so the **"anchored evidence-bundle not
latency-bound on either" pillar HOLDS** — re-verified at current versions. The ~3×
CH/GT ratio also holds, and the source is isolated: it is **entirely Q1's `trace_id`
retrieval floor** (CH sort-key seek 5 ms vs GreptimeDB inverted-index ~21 ms — the
same fixed inverted-lookup floor seen in Runs 1/6/50), **not** the correlation/assembly
itself — Q2+Q3 (the join/aggregate "bundle assembly" work) are ~tie and tiny on both
(2–3 ms vs 3–6 ms). So for Parallax the dominant evidence-bundle query is decided by
anchor-retrieval latency, and both deliver it instantly; the correlation join is not a
differentiator (consistent with Run 2/Run 30 EXPLAIN: both prune the anchor before
joining). Status: **confirmed, stable across ~16 runs.**

**Reproduce (copy-paste).**

```bash
T=3fb2d84c0a2032fa7681cde05c2051e9
# ClickHouse Q1/Q2/Q3 (warm, --time, FORMAT Null)
docker exec parallax-bench-clickhouse-1 clickhouse-client --time -q "SELECT 'span' k,span_id,CAST(duration_ms AS String) v,status m FROM spans WHERE trace_id='$T' UNION ALL SELECT 'log',span_id,level,message FROM logs WHERE trace_id='$T' UNION ALL SELECT 'error',span_id,error_type,message FROM error_events WHERE trace_id='$T' FORMAT Null"
docker exec parallax-bench-clickhouse-1 clickhouse-client --time -q "SELECT min(ts),max(ts),count() FROM error_events WHERE project='parallax' AND fingerprint='fp-000' FORMAT Null"
docker exec parallax-bench-clickhouse-1 clickhouse-client --time -q "SELECT fingerprint FROM error_events WHERE project='parallax' AND release='v1.7.0' AND fingerprint NOT IN (SELECT fingerprint FROM error_events WHERE project='parallax' AND release='v1.6.0') GROUP BY fingerprint FORMAT Null"
# GreptimeDB: same SQL via /v1/sql (spans_idx for Q1), read execution_time_ms
docker exec parallax-bench-greptimedb-1 curl -s localhost:4000/v1/sql?db=public --data-urlencode "sql=SELECT 'span' AS k,span_id,CAST(duration_ms AS STRING) AS v,status AS m FROM spans_idx WHERE trace_id='$T' UNION ALL SELECT 'log',span_id,level,message FROM logs WHERE trace_id='$T' UNION ALL SELECT 'error',span_id,error_type,message FROM error_events WHERE trace_id='$T'"
```

Caveat: warm cache-resident smoke (≤1M rows); these are minimum-latency floors, not
at-scale. The *not-latency-bound* conclusion is robust at this scale; cold GB–TB is the
harness's job.

### Run 57 — 2026-05-25 — Native out-of-the-box schema, live (the adopt-native-vs-custom decision)

**Pass target.** The brief's standing requirement: verify each system's *native
out-of-the-box* metrics/logs/traces structure with **zero schema work** and decide
adopt-native-vs-custom per signal. Rotate onto it (last native-structure work was
~pass 32–33/55). Trigger GreptimeDB's native ingest live and read the auto-created
DDL; confirm ClickHouse has no native-ingest equivalent.

**Environment.** Main stack, GreptimeDB `v1.0.2` / ClickHouse `v26.5.1.882` (versions
re-pinned this pass — latest, no bump).

**What GreptimeDB auto-created (live `SHOW CREATE TABLE`):**

- **Metrics** — `POST /v1/influxdb/write` (HTTP **204**), line `app_requests,service=api,env=prod count=42i,latency_ms=1.5`:

  ```sql
  CREATE TABLE "app_requests" ("service" STRING, "env" STRING, "count" BIGINT,
    "latency_ms" DOUBLE, "greptime_timestamp" TIMESTAMP(9) NOT NULL,
    TIME INDEX ("greptime_timestamp"), PRIMARY KEY ("service","env"))
    ENGINE=mito WITH(merge_mode='last_non_null');
  ```

  Tags → PK, fields **auto-typed** (`42i`→`BIGINT`, `1.5`→`DOUBLE`), auto TIME INDEX,
  `merge_mode='last_non_null'` (partial-upsert). One table per measurement.

- **Logs** — `POST /v1/ingest?table=app_logs&pipeline_name=greptime_identity` (HTTP
  **200**), JSON `[{"level","message","service","trace_id","span_id"}]`:

  ```sql
  CREATE TABLE "app_logs" ("greptime_timestamp" TIMESTAMP(9) NOT NULL, "level" STRING,
    "message" STRING, "service" STRING, "span_id" STRING, "trace_id" STRING,
    TIME INDEX ("greptime_timestamp")) ENGINE=mito WITH(append_mode='true');
  ```

  Every JSON key → `STRING` column, auto TIME INDEX, `append_mode='true'`, **no PK, no
  index on `trace_id`/`message`** (flat append).

- **Traces** — `POST /v1/otlp/v1/traces` with `Content-Type: application/json` →
  **HTTP 400**: `"OTLP endpoint only supports 'application/x-protobuf'"`. Native trace
  ingest is **protobuf-only** (re-confirms the pass-33 metrics finding for traces);
  the native `opentelemetry_traces` table needs a real OTLP exporter — **not
  hand-verifiable here**, owed to a collector-fed harness check.

**ClickHouse:** the same native writes have **no endpoint** — no InfluxDB/OTLP
receiver (re-confirmed: only GreptimeDB accepted these). Native ingest = an OTel
Collector + ClickHouse exporter (ClickStack) or a hand-defined schema; **no "zero
schema work" path.**

**Adopt-vs-custom verdict (feeds `greptimedb-implementation.md`):**

- **Metrics → ADOPT native** (tags-as-PK + auto-typed fields + last-non-null + PromQL
  on it = a correct metric table, zero DDL).
- **Logs → ADOPT-then-CUSTOMIZE** — the native append schema is right except it omits
  the **anchor index**; Parallax must add `trace_id INVERTED INDEX` (+ `message
  FULLTEXT`) because Run 56 showed `trace_id` retrieval is the bundle's dominant cost
  and the native table would **scan**. Name the shortfall precisely: *no index on
  `trace_id`/`message` on the auto-created log table.*
- **Traces → OWED + likely customize** — native model exists but couldn't be verified
  live (protobuf); Parallax's custom `spans` indexes `trace_id` regardless.

This is a real GreptimeDB **ingest/onboarding ergonomics edge** (usable tables with
zero/near-zero DDL) that ClickHouse structurally cannot match (collector-mediated).

**Reproduce (copy-paste).**

```bash
docker exec parallax-bench-greptimedb-1 curl -s -w '[%{http_code}]' -X POST "http://localhost:4000/v1/influxdb/write?db=public" --data-binary 'app_requests,service=api,env=prod count=42i,latency_ms=1.5'
docker exec parallax-bench-greptimedb-1 curl -s -X POST "http://localhost:4000/v1/ingest?db=public&table=app_logs&pipeline_name=greptime_identity" -H 'Content-Type: application/json' -d '[{"level":"error","message":"db timeout","service":"api","trace_id":"abc123","span_id":"s1"}]'
docker exec parallax-bench-greptimedb-1 curl -s -X POST "http://localhost:4000/v1/otlp/v1/traces" -H 'Content-Type: application/json' -d '{}'   # -> 400 protobuf-only
for t in app_requests app_logs; do docker exec parallax-bench-greptimedb-1 curl -s localhost:4000/v1/sql?db=public --data-urlencode "sql=SHOW CREATE TABLE $t"; done
docker exec parallax-bench-greptimedb-1 curl -s localhost:4000/v1/sql?db=public --data-urlencode 'sql=DROP TABLE app_requests'; docker exec parallax-bench-greptimedb-1 curl -s localhost:4000/v1/sql?db=public --data-urlencode 'sql=DROP TABLE app_logs'  # cleanup
```

Caveat: structure is exact (DDL); traces native schema unverified (protobuf-only ingest).

### Run 58 — 2026-05-25 — Unindexed-scan engine gap re-verified + characterized (CH vectorized wins; magnitude is row-dependent, corrects Run 31)

**Pass target.** Re-verify the strongest **honest counterexample** to the operator
hypothesis — "ClickHouse is genuinely faster on unindexed/ad-hoc scans" (Run 31:
unindexed `span_id` full scan, CH 10 ms / GT 95 ms ~10×, ~pass 53). Rotate the slice
onto it and characterize the gap across scan size.

**Environment.** Main stack, GreptimeDB `v1.0.2` / ClickHouse `v26.5.1.882` (re-pinned
this pass — latest, no bump). Warm, min-of-7. `span_id` is **unindexed on both**
(CH `spans ORDER BY (trace_id,ts)`; GreptimeDB `spans_idx` indexes only `trace_id`),
so the predicate forces a **full scan** on both (CH `EXPLAIN`: `Granules 123/123`).
Correctness parity: each filter returns 1 row on both.

**Measured (warm):**

| Scan | rows | ClickHouse | GreptimeDB | ratio | mechanism |
| --- | --- | --- | --- | --- | --- |
| filtered count `WHERE span_id=…` (spans) | 1M | **2 ms** | **15 ms** | ~7× | pure full scan + predicate |
| filtered count `WHERE span_id=…` (logs_b1) | 5M | **3 ms** | **43 ms** | ~14× | pure full scan + predicate |
| full aggregate `sum(value)` (metrics_hc) | 8M | **29 ms** | **91 ms** | ~3× | scan + aggregate |

**The comparison logic & verdict.**

- **Direction CONFIRMED:** ClickHouse's vectorized C++ engine (65,409-row blocks, SIMD
  predicate eval, `query-execution-engine.md`) wins every unindexed scan — the honest
  counterexample to "GreptimeDB fastest" **holds**. For ad-hoc/scan analytics ClickHouse
  is genuinely faster.
- **Magnitude CORRECTED — it is row-count-dependent throughput, not a fixed ~10×:** the
  pure-scan gap **widens with rows scanned** (~7× at 1M → ~14× at 5M), exactly what a
  per-row throughput difference predicts; the **aggregate** gap is **narrower (~3×)**
  because the `sum` work (done by both) dilutes the scan-speed difference. So "CH ~10×
  on scans" should be stated as "**CH faster on unindexed scans, ratio scales with scan
  width (~3× agg-bound up to ~14× scan-bound at these sizes), and grows at GB-scale**."
- **Run 31's specific "GT 95 ms / ~10×" does NOT reproduce** — the same 1M unindexed
  `span_id` scan is now **GT 15 ms** (`execution_time_ms`, warm). The 95 ms was almost
  certainly the **HTTP wall-clock floor** (~40 ms, see Run 40 correction) and/or a
  cold/uncompacted `spans` state, not engine scan time. **Status: scan-gap direction
  confirmed; magnitude re-characterized; the stale 95 ms artifact retired.**
- **Scale caveat (unchanged):** these are 1–8M warm cache-resident floors. The
  *decision-relevant* scan gap is GB–TB **cold**, where CH's throughput advantage should
  be largest — still owed to the sized harness (B1). At interactive smoke scale even
  GreptimeDB's "slow" scan is 15–91 ms (sub-perceptible); the gap matters for heavy
  ad-hoc analytics, not the anchored hot path (which is index-served, Run 56).

**Reproduce (copy-paste).**

```bash
SP=$(docker exec parallax-bench-clickhouse-1 clickhouse-client -q "SELECT span_id FROM spans LIMIT 1")
docker exec parallax-bench-clickhouse-1 clickhouse-client --time -q "SELECT count() FROM spans WHERE span_id='$SP' FORMAT Null"
docker exec parallax-bench-clickhouse-1 clickhouse-client -q "EXPLAIN indexes=1 SELECT count() FROM spans WHERE span_id='$SP'"   # Granules 123/123 = full scan
docker exec parallax-bench-greptimedb-1 curl -s localhost:4000/v1/sql?db=public --data-urlencode "sql=SELECT count(*) FROM spans_idx WHERE span_id='$SP'"   # execution_time_ms
docker exec parallax-bench-clickhouse-1 clickhouse-client --time -q "SELECT sum(value) FROM metrics_hc FORMAT Null"
docker exec parallax-bench-greptimedb-1 curl -s localhost:4000/v1/sql?db=public --data-urlencode "sql=SELECT sum(value) FROM metrics_hc"
```

Caveat: warm smoke; GreptimeDB timed by server `execution_time_ms` (excludes HTTP),
ClickHouse by `--time` — the row-dependent *direction* is the robust result, not a
precise cross-engine ratio.

### Run 59 — 2026-05-25 — Dedup/upsert semantics re-verified + partial-upsert loss proven

**Pass target.** Rotate onto a stale **correctness/ergonomics** claim (not latency):
"GreptimeDB is correct-by-default on upsert (read-time dedup); ClickHouse needs
`FINAL`" + "GreptimeDB `last_non_null` does partial upsert ClickHouse RMT can't"
(Run 19, ~pass 39). Re-verify live.

**Environment.** Main stack, GreptimeDB `v1.0.2` / ClickHouse `v26.5.1.882` (re-pinned
this pass — latest, no bump).

**(A) Read-time dedup vs merge-time — reproduces:**

| Action | GreptimeDB (`PRIMARY KEY(k)`, default `last_row`) | ClickHouse (`ReplacingMergeTree ORDER BY (k,ts)`) |
| --- | --- | --- |
| insert `(a,ts,10)` then `(a,ts,20)`, plain `SELECT` | **1 row, v=20** (read-time dedup, no keyword) | **2 rows (10, 20)** — NOT deduped |
| force correct | nothing needed | `SELECT … FINAL` → **1 row, v=20** |

**(B) Partial upsert — the capability gap, now proven concretely:** two partial writes
to key `x` — `(a=10, b=NULL)` then `(a=NULL, b='hello')`:

| Engine | result | mechanism |
| --- | --- | --- |
| **GreptimeDB** `merge_mode='last_non_null'` | **`a=10, b='hello'`** (per-field merge) | `DedupReader` `merge_last_non_null` (`read/dedup.rs:420`) |
| **ClickHouse** `ReplacingMergeTree … FINAL` | **`a=NULL, b='hello'`** — **`a=10` LOST** | RMT keeps the last *whole* row, no per-field merge |

**Verdict.** Run 19 **reproduces unchanged**: GreptimeDB dedups at read (plain query
always correct), ClickHouse RMT shows duplicates until `FINAL`/merge. **Run 59 adds the
concrete partial-upsert proof** the note previously asserted: RMT `FINAL` **discards a
field** set only in an earlier insert (`a=10`→`NULL`), while GreptimeDB `last_non_null`
merges per-field. To match GreptimeDB, ClickHouse needs `AggregatingMergeTree` +
`argMax(col, ts)`-per-column + a materialized view — real ceremony vs one table option.
**Status: confirmed; capability gap proven, not just asserted.** Reinforces the
"upsert ergonomics + correctness-by-default → GreptimeDB" pillar for Parallax's
partial-update signals (issue status/assignee/last-seen from different events; late
span attribute enrichment). Does not move the raw-scan/log verdicts.

Caveat: 2-row smoke — proves *semantics*, not the `FINAL`-cost-at-scale crossover (owed).

**Reproduce (copy-paste).**

```bash
# GreptimeDB partial upsert
docker exec parallax-bench-greptimedb-1 curl -s localhost:4000/v1/sql?db=public --data-urlencode "sql=CREATE TABLE upsert_gt (k STRING, ts TIMESTAMP(3) TIME INDEX, a BIGINT, b STRING, PRIMARY KEY(k)) WITH (merge_mode='last_non_null')"
docker exec parallax-bench-greptimedb-1 curl -s localhost:4000/v1/sql?db=public --data-urlencode "sql=INSERT INTO upsert_gt VALUES ('x',1000,10,NULL)"
docker exec parallax-bench-greptimedb-1 curl -s localhost:4000/v1/sql?db=public --data-urlencode "sql=INSERT INTO upsert_gt VALUES ('x',1000,NULL,'hello')"
docker exec parallax-bench-greptimedb-1 curl -s localhost:4000/v1/sql?db=public --data-urlencode "sql=SELECT k,a,b FROM upsert_gt WHERE k='x'"   # a=10,b=hello
# ClickHouse RMT (loses a=10)
docker exec parallax-bench-clickhouse-1 clickhouse-client --multiquery -q "CREATE TABLE upsert_ch (k String, ts DateTime64(3), a Nullable(Int64), b Nullable(String)) ENGINE=ReplacingMergeTree ORDER BY (k,ts); INSERT INTO upsert_ch VALUES ('x',toDateTime64(1,3),10,NULL); INSERT INTO upsert_ch VALUES ('x',toDateTime64(1,3),NULL,'hello');"
docker exec parallax-bench-clickhouse-1 clickhouse-client -q "SELECT k,a,b FROM upsert_ch FINAL WHERE k='x'"   # a=NULL,b=hello
```

### Run 60 — 2026-05-25 — Measurement-basis fairness: GreptimeDB MySQL-native vs HTTP `execution_time_ms` (validates the whole record)

**Pass target.** Resolve the long-owed cross-cutting fairness item ("Next runs #5:
fairer GreptimeDB timing via MySQL native protocol, not HTTP"). Every GreptimeDB
latency in this log is `execution_time_ms` (server-side, over HTTP); every ClickHouse
one is `--time` (native-client wall). Are these comparable, or has the basis been
flattering/penalizing GreptimeDB? Measure GreptimeDB via the **MySQL native wire
(port 4002)** — a client-wall basis comparable to ClickHouse's native client.

**Environment.** Main stack, GreptimeDB `v1.0.2` / ClickHouse `v26.5.1.882` (re-pinned
— latest, no bump). MySQL-wall measured by timing a 20-query batch in one `mysql:8`
client session on `parallax-bench_default` and subtracting a 1-query baseline (isolates
per-query from container-startup+connection ≈ 0.42 s). GreptimeDB's MySQL status line
omits per-query timing, hence the batch method.

**Measured (3 bases, two query shapes):**

| Query | GreptimeDB MySQL-wall (native, amortized) | GreptimeDB HTTP `execution_time_ms` (server) | GreptimeDB in-container HTTP curl-wall | ClickHouse `--time` |
| --- | --- | --- | --- | --- |
| anchor `trace_id` lookup (spans_idx) | **~5 ms** ((0.523−0.424)/20) | **9–10 ms** | ~10–12 ms | 2–9 ms |
| metric agg `GROUP BY service` (8M) | **~96 ms** ((2.352−0.424)/20) | **93–99 ms** | 94–101 ms | ~36 ms |

**The comparison logic & verdict.**

- **`execution_time_ms` is a FAIR — and slightly GreptimeDB-conservative — basis.**
  For the **heavy** query (agg) all three GreptimeDB bases agree (~95 ms): execution
  dominates, protocol/transport is noise. So every heavy-query GreptimeDB number in
  this log (metric agg, scans) is **protocol-independent and fair** — not inflated by
  HTTP.
- **For tiny queries the basis matters at the few-ms level, and HTTP slightly
  *over*-states GreptimeDB:** native MySQL-wall ~5 ms vs HTTP `execution_time_ms` ~10 ms
  — i.e. a warm native session amortizes ~4–5 ms of per-request planning/overhead that
  the isolated HTTP path pays each time. **So the anchored-lookup gap was reported
  slightly *against* GreptimeDB**; on the native protocol GreptimeDB's anchor is ~5 ms,
  even closer to ClickHouse's 2–9 ms. The measurement bias runs *toward* ClickHouse,
  never flattering GreptimeDB.
- **The old "GreptimeDB 54 ms HTTP-wall" artifacts (Run 40) were external-network
  client wall, not the protocol** — in-container curl-wall ≈ `execution_time_ms`
  (10 vs 9 ms; loopback ~1–2 ms). Confirms the Run 40/58 corrections: those inflated
  numbers were measurement environment, and the server-side figures are the truth.

**Net:** the record's GreptimeDB-vs-ClickHouse latency gaps are **real, not measurement
artifacts**, and if anything GreptimeDB is marginally faster on tiny queries than the
HTTP basis showed. No prior verdict number needs reversing on fairness grounds; the
non-comparability caveat is upgraded from "close enough to read direction" to
"validated: heavy-query-identical, tiny-query slightly GreptimeDB-conservative."
**Status: fairness item closed.**

Caveat: MySQL-wall is a warm-session amortized per-query (20-batch); a cold single
native query would sit between ~5 ms and the HTTP ~10 ms. ClickHouse `--time` is
likewise warm. Container-startup (~0.42 s) excluded by subtraction; 8M/1M smoke scale.

**Reproduce (copy-paste).**

```bash
NET=parallax-bench_default; TR=3fb2d84c0a2032fa7681cde05c2051e9
QA="SELECT count(*) FROM spans_idx WHERE trace_id='$TR'"; A20=$(for i in $(seq 20); do printf '%s;' "$QA"; done)
b=$(date +%s.%N); docker run --rm --network $NET mysql:8 mysql -h parallax-bench-greptimedb-1 -P4002 -uroot -N -e "SELECT 1" >/dev/null 2>&1; e=$(date +%s.%N); python3 -c "print($e-$b)"   # baseline
b=$(date +%s.%N); docker run --rm --network $NET mysql:8 mysql -h parallax-bench-greptimedb-1 -P4002 -uroot -N -e "$A20" >/dev/null 2>&1; e=$(date +%s.%N); python3 -c "print(($e-$b))"  # /20 minus baseline = per-query
docker exec parallax-bench-greptimedb-1 curl -s -w 'WALL=%{time_total}' localhost:4000/v1/sql?db=public --data-urlencode "sql=$QA"   # exec vs wall
```

### Run 61 — 2026-05-25 — Dynamic-attribute JSON path query (the "ClickHouse wins dynamic attrs" edge, now a number)

**Pass target.** Rotate onto stale subsystem #10 (schema/dynamic columns, Run 18 ~pass
38). Run 18 established the *mechanism* (ClickHouse JSON = typed columnar subcolumns;
GreptimeDB JSON = binary blob) but no latency. Measure the dynamic-attribute **path
query** — the load-bearing "dynamic-attr → ClickHouse" verdict edge.

**Environment.** Main stack, GreptimeDB `v1.0.2` / ClickHouse `v26.5.1.882` (re-pinned
— latest, no bump). Both: a `JSON` column `attrs` over **100k rows**, `{user_id (unique),
tenant}`. **Matched shape, not identical bytes**: ClickHouse built from `numbers(100000)`
(tenant `t0–t9`, 10 buckets); GreptimeDB built from `logs_b1` `span_id` (tenant = `t`+first
hex char, 16 buckets). Both scan all 100k and extract the path, so the extraction *work*
is comparable; the filter match-count differs (CH `t3`=10000, GreptimeDB `t3`=6253) —
documented, immaterial to the per-row-parse cost being measured.

**Measured (warm, min of 7 / 3):**

| Query | ClickHouse (JSON subcolumn) | GreptimeDB (`json_get_string`, blob) |
| --- | --- | --- |
| filter `tenant='t3'` | **~6 ms** | **~78 ms** → **~13× slower** |
| group-by `tenant` | **~5 ms** (needs cast `attrs.tenant.:String`) | **~79 ms** (plain `String`, no cast) |
| storage (100k) | **1.00 MiB** | **1.10 MiB** (≈ tie) |

`EXPLAIN actions=1` on ClickHouse confirms a **subcolumn read**: `INPUT: attrs.tenant
Dynamic` + `equals(attrs.tenant, 't3')` — it reads only the `tenant` path, not the whole
document. GreptimeDB's `json_get_string(attrs,'tenant')` parses each row's JSON blob.

**The comparison logic & verdict.**

- **ClickHouse wins dynamic-attr path queries ~13×** (6 ms vs 78 ms) — the columnar
  typed-subcolumn JSON reads only the queried path; GreptimeDB blob-parses every row.
  Confirms + **quantifies** the Run-18 mechanism. Real edge **if Parallax filters/groups
  by unpredictable attribute paths at volume.**
- **Two-sided (fairness):** ClickHouse's subcolumns are **`Dynamic`-typed** → a raw
  `GROUP BY attrs.tenant` **errors** (`Variant/Dynamic not allowed in GROUP BY keys`);
  needs `attrs.tenant.:String` (then 5 ms) or `allow_suspicious_types_in_group_by=1`. An
  aggregation ergonomics wrinkle GreptimeDB's plain-`String` `json_get_*` avoids (slow but
  no cast). Storage is a **tie** at 100k (1.00 vs 1.10 MiB) — the columnar split doesn't
  cost extra here.
- **GreptimeDB's intended fast path is NOT the blob** — it is promoting a *known* hot
  attribute to a typed column / `SKIPPING INDEX` (impl principle 6), columnar like
  ClickHouse but **manual** (you choose which) vs ClickHouse's **automatic** per-path
  subcolumns. So for a *fixed* set of hot attrs both reach columnar speed; the ClickHouse
  edge is specifically for **ad-hoc/unpredictable** attribute paths. Status: edge
  **confirmed + quantified (~13×), with the casting and promote-on-demand caveats.**
  Reinforces, does not flip, the verdict.

Caveat: 100k warm smoke; matched-shape not identical bytes; the gap likely grows at more
rows (per-row parse scales). At-volume dynamic-attr query is owed to the harness if it
becomes a Parallax hot path.

**Reproduce (copy-paste).**

```bash
docker exec parallax-bench-clickhouse-1 clickhouse-client --multiquery -q "CREATE TABLE js_ch (id UInt32, attrs JSON) ENGINE=MergeTree ORDER BY id; INSERT INTO js_ch SELECT number, concat('{\"user_id\":\"u',toString(number),'\",\"tenant\":\"t',toString(number%10),'\"}') FROM numbers(100000); OPTIMIZE TABLE js_ch FINAL;"
docker exec parallax-bench-clickhouse-1 clickhouse-client --time -q "SELECT count() FROM js_ch WHERE attrs.tenant='t3' FORMAT Null"
docker exec parallax-bench-clickhouse-1 clickhouse-client -q "EXPLAIN actions=1 SELECT count() FROM js_ch WHERE attrs.tenant='t3'"   # attrs.tenant subcolumn
docker exec parallax-bench-clickhouse-1 clickhouse-client --time -q "SELECT attrs.tenant.:String t,count() FROM js_ch GROUP BY t FORMAT Null"   # cast required
docker exec parallax-bench-greptimedb-1 curl -s localhost:4000/v1/sql?db=public --data-urlencode "sql=CREATE TABLE js_gt (ts TIMESTAMP(3) TIME INDEX, attrs JSON) WITH (append_mode='true')"
docker exec parallax-bench-greptimedb-1 curl -s localhost:4000/v1/sql?db=public --data-urlencode "sql=INSERT INTO js_gt SELECT ts, CONCAT('{\"user_id\":\"', span_id, '\",\"tenant\":\"t', SUBSTR(span_id,1,1), '\"}') FROM logs_b1 LIMIT 100000"
docker exec parallax-bench-greptimedb-1 curl -s localhost:4000/v1/sql?db=public --data-urlencode "sql=SELECT count(*) FROM js_gt WHERE json_get_string(attrs,'tenant')='t3'"   # execution_time_ms
# cleanup: DROP TABLE js_ch / js_gt
```

### Run 62 — 2026-05-25 — PromQL/metrics-native re-verified (the verdict's #1 pillar, no drift)

**Pass target.** Re-verify the verdict's load-bearing capability claim — "metrics/PromQL
**GA-native** on GreptimeDB vs **experimental** on ClickHouse" (Runs 23/24/44, ~17 passes
stale). A version-drift here (e.g. ClickHouse promoting `TimeSeries` to stable, or
GreptimeDB PromQL regressing) is decision-critical, so re-check live.

**Environment.** Main stack, GreptimeDB `v1.0.2` / ClickHouse `v26.5.1.882` (re-pinned —
latest, no bump).

**Measured / verified:**

| Check | GreptimeDB | ClickHouse |
| --- | --- | --- |
| PromQL zero-setup, real value | `/v1/prometheus/api/v1/query?query=avg(metrics_hc)` → `success`, `50.77`; `TQL EVAL … avg(metrics_hc)` → `49.98` — on a **plain `mito` table**, no metric-engine table needed | n/a (no PromQL HTTP endpoint) |
| Experimental gate | PromQL GA + default-on | `allow_experimental_time_series_table=0` (off by default) |
| PromQL compute path | native planner (`InstantManipulate`/`RangeManipulate`/…) | `prometheusQuery`/`prometheusQueryRange` table functions exist |
| `TimeSeries` engine ingest/query | n/a | created with flag, but **`INSERT` → "not supported by storage TimeSeries yet"**, **`SELECT` → "not supported … yet"** (NOT_IMPLEMENTED) → ingest **remote-write-only**, query **table-function-only** |

**Verdict.** **No drift — pillar STABLE.** GreptimeDB PromQL is GA, default-on, served
over the standard Prometheus HTTP API (drop-in Grafana datasource), on plain tables.
ClickHouse PromQL is a **real shipping capability** (experimental-counts-as-stable: the
functions exist and the engine is creatable) but **maturity-gated and ergonomically
constrained** — off by default, no direct `INSERT`/`SELECT` on the `TimeSeries` engine
(reproduces Run 24 exactly at 26.5), feed via remote-write only, query via table
functions only. So the gap remains **"GA-ergonomic (GreptimeDB) vs
experimental-off-by-default-setup-heavy (ClickHouse)", not present-vs-absent** — exactly
as Runs 23/24/44 found. Status: **confirmed, the metrics-native recommendation basis
holds at current versions.**

Caveat: capability/ergonomics check, not a speed run (PromQL speed was Run 44: GreptimeDB
native PromQL ~5× slower than its own SQL at 40k series — a `SeriesNormalize` fixed cost,
so metrics→GreptimeDB is a *capability/ergonomics* win, never a raw-speed one).

**Reproduce (copy-paste).**

```bash
docker exec parallax-bench-greptimedb-1 curl -s "http://localhost:4000/v1/prometheus/api/v1/query" --data-urlencode "query=avg(metrics_hc)" --data-urlencode "time=2024-05-18T03:00:00Z"
docker exec parallax-bench-greptimedb-1 curl -s localhost:4000/v1/sql?db=public --data-urlencode "sql=TQL EVAL (1716000000,1716000000,'60s') avg(metrics_hc)"
docker exec parallax-bench-clickhouse-1 clickhouse-client -q "SELECT value FROM system.settings WHERE name='allow_experimental_time_series_table'"   # 0
docker exec parallax-bench-clickhouse-1 clickhouse-client --allow_experimental_time_series_table=1 -q "CREATE TABLE ts_probe ENGINE=TimeSeries; INSERT INTO ts_probe(metric_name,tags,timestamp,value) VALUES ('m',map('s','a'),now(),1.0)"   # INSERT not supported yet
docker exec parallax-bench-clickhouse-1 clickhouse-client --allow_experimental_time_series_table=1 -q "DROP TABLE ts_probe"
```

### Run 63 — 2026-05-25 — Why the cold anchored read pulls the whole SST: scatter vs cluster (resolves Run 55's caveat)

**Pass target.** Run 55 found a cold anchored `trace_id` lookup read ~the whole 21 MiB
SST from S3 and flagged it as *possibly* a small-SST artifact ("at scale GreptimeDB should
row-group-prune"). Resolve it: does GreptimeDB prune row groups for the anchored query, or
read all of them? Use `EXPLAIN ANALYZE` locally (no S3 needed).

**Environment.** Main stack, GreptimeDB `v1.0.2` / ClickHouse `v26.5.1.882` (re-pinned —
latest, no bump). `spans_idx` = 1M spans, `trace_id STRING INVERTED INDEX`, **`PRIMARY KEY
(service, name)`** (the recommended Parallax design — trace_id indexed, *not* the sort key,
to avoid series-cardinality blowup).

**Measured (`EXPLAIN ANALYZE`, anchored `trace_id` lookup, 14 rows of 1M):**

| Table | sort key (PK) | scan_cost | file_ranges | output_rows |
| --- | --- | --- | --- | --- |
| `spans_idx` (recommended design) | `(service, name)` → **trace_id scattered** | **39 ms** | 10 | 14 |
| `spans_tidpk` (built this run) | `(trace_id)` → **trace_id clustered** | **14 ms** | 10 | 14 |
| unindexed `span_id` on spans_idx (Run 58 ref) | — (full scan) | ~52 ms | 10 | 1 |

**The mechanism & verdict.**

- **`file_ranges:10` is the parallelism partition count, NOT bytes read** — it is 10 in
  *all* cases. The real signal is **scan_cost**: clustering the anchor (`PRIMARY
  KEY(trace_id)`) cut it **39 ms → 14 ms (~2.8×)** for the identical query. So GreptimeDB
  *does* read less when the anchor is the sort key — the rows are localized to fewer row
  groups.
- **Run 55's whole-SST cold read is NOT a small-SST artifact — it is a scatter
  consequence.** The recommended Parallax design indexes `trace_id` (inverted) but keys on
  low-card `service` (to avoid 71k-series cardinality). So a trace's 14 spans **scatter
  across all ~10 row groups** → an anchored read must touch ~every row group → cold = read
  ~the whole SST (the 23 MiB Run 55 measured). **At a larger SST this persists/grows**
  (more row groups, all touched) — so the cold-selective-egress gap vs ClickHouse is
  **real and would scale**, not an artifact. *Caveat retired.*
- **The structural asymmetry that decides it:** ClickHouse `ORDER BY (trace_id, ts)`
  **clusters by the high-card anchor at zero cardinality cost** (sort key ≠ series), so its
  14 spans sit in ~1 granule → cold read = 294 KiB (Run 55). GreptimeDB's **PK is also its
  series identity**, so clustering by `trace_id` (which *would* prune cold reads — proven
  here, 39→14 ms) **explodes series cardinality** — the very reason the design avoids it.
  So GreptimeDB faces a **cluster-vs-cardinality tradeoff that ClickHouse does not**: it
  can have anchor-clustered cheap cold reads *or* bounded series, not both for free. This
  is the mechanism behind the cold-selective-egress disadvantage — a genuine,
  design-rooted ClickHouse edge for **cold** anchored reads (mitigated for GreptimeDB by
  its persistent local cache making most reads warm, Run 55).

**Status:** Run 55 caveat **resolved** — whole-SST cold read is scatter-driven, persists
at scale; cold-selective-egress favors ClickHouse by a sort-key/cardinality mechanism, not
a small-data fluke. Warm/cached re-reads still favor GreptimeDB (Run 55). The precise
cold-egress *number* at large SST is still owed to the sized harness; the *mechanism* is
now settled.

Caveat: scan_cost is warm local (not cold S3 bytes) — it proves the locality mechanism;
the cold byte count at scale is inferred from it + Run 55, owed for an exact figure.

**Reproduce (copy-paste).**

```bash
TR=3fb2d84c0a2032fa7681cde05c2051e9
docker exec parallax-bench-greptimedb-1 curl -s localhost:4000/v1/sql?db=public --data-urlencode "sql=EXPLAIN ANALYZE SELECT span_id FROM spans_idx WHERE trace_id='$TR'"   # scattered: scan_cost ~39ms
docker exec parallax-bench-greptimedb-1 curl -s localhost:4000/v1/sql?db=public --data-urlencode 'sql=CREATE TABLE spans_tidpk ("ts" TIMESTAMP(3) TIME INDEX,"trace_id" STRING,"span_id" STRING,"service" STRING,"name" STRING,"duration_ms" DOUBLE,"status" STRING, PRIMARY KEY("trace_id")) WITH (append_mode='"'"'true'"'"')'
docker exec parallax-bench-greptimedb-1 curl -s localhost:4000/v1/sql?db=public --data-urlencode 'sql=INSERT INTO spans_tidpk SELECT "ts","trace_id","span_id","service","name","duration_ms","status" FROM spans_idx'
docker exec parallax-bench-greptimedb-1 curl -s localhost:4000/v1/sql?db=public --data-urlencode "sql=ADMIN compact_table('spans_tidpk')"
docker exec parallax-bench-greptimedb-1 curl -s localhost:4000/v1/sql?db=public --data-urlencode "sql=EXPLAIN ANALYZE SELECT span_id FROM spans_tidpk WHERE trace_id='$TR'"   # clustered: scan_cost ~14ms
docker exec parallax-bench-greptimedb-1 curl -s localhost:4000/v1/sql?db=public --data-urlencode "sql=DROP TABLE spans_tidpk"
```

### Run 64 — 2026-05-25 — TTL/retention re-verified: both ClickHouse merge paths + GreptimeDB read-time filter

**Pass target.** Rotate onto stale cost-axis subsystem #5 (retention/TTL, Run 17 ~pass
37). Re-verify the load-bearing claim "GreptimeDB whole-SST drop (no rewrite) vs
ClickHouse row-rewrite TTL" on controlled tables, and characterize *when* ClickHouse
rewrites vs drops.

**Environment.** Main stack, GreptimeDB `v1.0.2` / ClickHouse `v26.5.1.882` (re-pinned —
latest, no bump). ClickHouse tables: `TTL ts + INTERVAL 1 HOUR`, `ttl_only_drop_parts=0`
(default), `merge_with_ttl_timeout=0` (eager). Rows with `ts = now()-1 DAY` are immediately
expired.

**Measured (`system.part_log`):**

| Case | ClickHouse `merge_reason` | read_rows | rows written | meaning |
| --- | --- | --- | --- | --- |
| Wholly-expired part (separate insert) | **`TTLDropMerge`** | 16,384 | **0** | whole part dropped, **no rewrite** — even at default settings |
| Mixed expired+alive part (one insert, 50/50) | **`TTLDeleteMerge`** | **1,000,000** | **500,000** | part **read in full, rewritten** with survivors → write-amp ∝ survivors |

GreptimeDB (`ttl='1h'`, 500k rows loaded with year-old `ts`): **0 live rows
*immediately*, before any compaction** — TTL is a **read-time filter** (expired rows
invisible at query/flush, `compactor.rs:581` whole-SST drop on compaction, no rewrite).
`region_statistics` SST stayed 0 (the all-expired data never materialized as live SSTs).

**The comparison logic & verdict.**

- **Re-verifies Run 17 + refines it:** ClickHouse's TTL rewrite penalty (`TTLDeleteMerge`,
  read 1M / write 500k confirmed) hits **only parts that mix expired+alive rows** — a
  **wholly-expired part drops wholesale (`TTLDropMerge`, 0 rewritten) even at default
  `ttl_only_drop_parts=0`.** So the "row-level rewrite" cost is specifically a
  **boundary-part** cost, and how often it occurs depends on whether parts are
  time-aligned (which `PARTITION BY` time fixes).
- **GreptimeDB avoids it by construction:** TWCS time-windows SSTs → expiry is whole-SST
  (no mixed SST to rewrite) **and** TTL is read-time (expired invisible immediately, not
  waiting for the drop). ClickHouse expired rows stay physically present + queryable until
  the TTL merge runs (≥ `merge_with_ttl_timeout`, 4h default; 0 here only because forced).
- **Net (cost axis):** equal *capability*, unequal *defaults* — GreptimeDB cheap retention
  with zero tuning; ClickHouse cheap **if** `PARTITION BY` time + `ttl_only_drop_parts=1`,
  else boundary-part rewrites. Confirms the retention-cost edge to GreptimeDB **as an
  ergonomics/defaults edge**, not a capability gap. Status: **confirmed + refined.**

Caveat: smoke (1M rows), `merge_with_ttl_timeout=0` forces eager TTL (production default
4h); the write-amp *magnitude* at retention volume is owed to the harness.

**Reproduce (copy-paste).**

```bash
# CH mixed-part rewrite
docker exec parallax-bench-clickhouse-1 clickhouse-client --multiquery -q "CREATE TABLE ttl_mix (ts DateTime, v UInt64) ENGINE=MergeTree ORDER BY v TTL ts + INTERVAL 1 HOUR SETTINGS ttl_only_drop_parts=0, merge_with_ttl_timeout=0; INSERT INTO ttl_mix SELECT if(number%2=0, now()-INTERVAL 1 DAY, now()), number FROM numbers(1000000);"
docker exec parallax-bench-clickhouse-1 clickhouse-client -q "OPTIMIZE TABLE ttl_mix FINAL"; docker exec parallax-bench-clickhouse-1 clickhouse-client -q "SYSTEM FLUSH LOGS"
docker exec parallax-bench-clickhouse-1 clickhouse-client -q "SELECT merge_reason, read_rows, rows FROM system.part_log WHERE table='ttl_mix' AND event_type='MergeParts' ORDER BY event_time DESC LIMIT 2"   # TTLDeleteMerge 1000000 -> 500000
# GT read-time TTL filter
docker exec parallax-bench-greptimedb-1 curl -s localhost:4000/v1/sql?db=public --data-urlencode "sql=CREATE TABLE ttl_gt (ts TIMESTAMP(3) TIME INDEX, v BIGINT) WITH (ttl='1h', append_mode='true')"
docker exec parallax-bench-greptimedb-1 curl -s localhost:4000/v1/sql?db=public --data-urlencode "sql=INSERT INTO ttl_gt SELECT ts, 1 FROM logs_b1 LIMIT 500000"
docker exec parallax-bench-greptimedb-1 curl -s localhost:4000/v1/sql?db=public --data-urlencode "sql=SELECT count(*) FROM ttl_gt"   # 0 immediately (read-time TTL)
```

### Run 65 — 2026-05-25 — No clustering-independent-of-PK in GreptimeDB (the Run-63 gap is architectural)

**Pass target.** Run 63 found GreptimeDB can't cluster by a high-card anchor (`trace_id`)
without making it the PK (→ series blowup). Confirm there is no *other* lever — a sort /
clustering / `order_by` table option independent of the `PRIMARY KEY` — and feed the
parity-roadmap (Improvement #5).

**Environment.** GreptimeDB `v1.0.2` / ClickHouse `v26.5.1.882` (re-pinned — latest, no bump).

**Verified (live):**

- `CREATE TABLE … PRIMARY KEY(tid) WITH (order_by='tid')` → **`Unrecognized table option
  key: order_by`**. GreptimeDB exposes **no clustering/secondary-sort option** — the
  `PRIMARY KEY` is the *only* control over physical row order within a region, and it is
  simultaneously the **series identity** (cardinality driver) and the **dedup key**.
- (Source corroboration from prior passes, now stale-cloned but cited in the notes: no
  `PROJECTION` keyword in the SQL parser; `AlterTableOperation` has no ordering variant —
  `greptimedb-parity-roadmap.md` #5.)

**Consequence.** Confirms the Run 63 finding is **architectural, not a config miss**:
ClickHouse decouples physical sort (`ORDER BY`) from row/series identity, so it clusters by
`trace_id` free; GreptimeDB conflates PK = sort = series identity, so anchor-clustering
costs series cardinality. This is the root of both the alternate-scan-order gap (Run 28) and
the cold-selective-read egress loss (Run 55/63). Routed into parity-roadmap Improvement #5
(Tier A Flow-copy workaround / Tier B mito2 alternate-sorted copy; full sort/identity
decoupling = redesign). **Still a footnote for Parallax** — the persistent read cache keeps
the common (recent, warm) anchored path fast regardless; only frequent **cold selective**
re-reads would justify the build.

**Reproduce.** `docker exec parallax-bench-greptimedb-1 curl -s localhost:4000/v1/sql?db=public --data-urlencode "sql=CREATE TABLE t (ts TIMESTAMP(3) TIME INDEX, tid STRING, PRIMARY KEY(tid)) WITH (order_by='tid')"` → error.

### Run 66 — 2026-05-25 — Deletes/mutations re-verified + UPDATE-statement precision

**Pass target.** Rotate onto stale slice (deletes/mutations, Run 29 ~pass 51). Re-verify
"DELETE ≈ parity (both read-immediate); UPDATE → GreptimeDB" and pin down GreptimeDB's
exact UPDATE capability.

**Environment.** Main stack, GreptimeDB `v1.0.2` / ClickHouse `v26.5.1.882` (re-pinned —
latest, no bump).

**Measured / verified:**

| Operation | ClickHouse | GreptimeDB |
| --- | --- | --- |
| DELETE read-immediacy | `DELETE FROM del_ch WHERE id<50000` → plain `count()` **50000 immediately** (lightweight mask; part `all_1_1_0`→`all_1_1_0_2`, 100k physical rows masked) | `DELETE FROM del_gt WHERE id=2` → plain SELECT **`[1],[3],[4]` immediately** (tombstone + `filter_deleted`, no compaction) |
| UPDATE statement | `ALTER UPDATE` (heavy rewrite) GA; **lightweight `UPDATE` rejected**: *"Lightweight updates are not supported … only for tables with materialized `_block_number`"* (gated, Run 29 reproduces) | **NO `UPDATE` statement** — *"SQL statement is not supported"* |
| UPDATE via upsert (GreptimeDB) | — | re-insert **same `(id=1, ts=1000)`** → overwrote (`sameTS`); re-insert **new `ts=2000`** → **two versions** `[1000,'sameTS'],[2000,'newTS']` (time-series, not in-place) |

**Verdict.** **DELETE = parity reproduced** — both read-immediate (CH `_row_exists` mask,
GreptimeDB tombstone+`filter_deleted`); the old "CH deletes are expensive rewrites"
critique stays softened. **UPDATE → GreptimeDB, with a sharpened nuance:** GreptimeDB has
**no `UPDATE` DML at all** — correction is INSERT-upsert, and the overwrite is **(PK, ts)-keyed**
(same ts overwrites; a new ts is a new time-series version). So a Parallax "current-state"
update is modeled as *re-write the same `(PK, ts)`* or *append + query latest* — simpler
and cheaper than ClickHouse's gated/heavy update, but it is an **append/upsert model, not a
relational row-update**. ClickHouse lightweight UPDATE still experimental + per-table-gated
(reproduces Run 29). Status: **confirmed + sharpened.** Reinforces the LSM-native correction
ergonomics theme; doesn't move the verdict (corrections are occasional for append-mostly
Parallax).

Caveat: small smoke (2-row update semantics, 100k delete); the rewrite-vs-mask-vs-tombstone
cost at GB scale is owed to the harness.

**Reproduce (copy-paste).**

```bash
docker exec parallax-bench-clickhouse-1 clickhouse-client -q "CREATE TABLE del_ch (id UInt32, v String) ENGINE=MergeTree ORDER BY id"; docker exec parallax-bench-clickhouse-1 clickhouse-client -q "INSERT INTO del_ch SELECT number,'x' FROM numbers(100000)"
docker exec parallax-bench-clickhouse-1 clickhouse-client -q "DELETE FROM del_ch WHERE id<50000"; docker exec parallax-bench-clickhouse-1 clickhouse-client -q "SELECT count() FROM del_ch"   # 50000
docker exec parallax-bench-greptimedb-1 curl -s localhost:4000/v1/sql?db=public --data-urlencode 'sql=CREATE TABLE upd_gt (ts TIMESTAMP(3) TIME INDEX, "id" BIGINT, v STRING, PRIMARY KEY("id"))'
docker exec parallax-bench-greptimedb-1 curl -s localhost:4000/v1/sql?db=public --data-urlencode 'sql=UPDATE upd_gt SET v='"'"'x'"'"' WHERE "id"=1'   # "SQL statement is not supported"
docker exec parallax-bench-greptimedb-1 curl -s localhost:4000/v1/sql?db=public --data-urlencode 'sql=INSERT INTO upd_gt VALUES (1000,1,'"'"'a'"'"'),(1000,1,'"'"'b'"'"')'   # same (id,ts) -> b wins
```

### Run 67 — 2026-05-25 — Metric-agg gap re-verified (~2–3× warm) + verdict-currency pass

**Pass target.** Re-verify the core "ClickHouse faster on metric aggregation" claim
(Run 37, ~2× warm) and fold the accumulated Runs 55–66 findings into the standing verdict.

**Environment.** Main stack, GreptimeDB `v1.0.2` / ClickHouse `v26.5.1.882` (re-pinned —
latest, no bump).

**Measured (warm, `GROUP BY service` over `metrics_hc` 8M, min of 7):**

| | ClickHouse | GreptimeDB | ratio |
| --- | --- | --- | --- |
| Run 37 | 50 ms | 107 ms | ~2× |
| **Run 67** | **32 ms** | **99 ms** | **~3×** |

**Verdict.** ClickHouse leads metric aggregation — **direction stable**; the ratio is now
**~3×** (was 2× in Run 37). The shift is **ClickHouse getting faster** (50→32 ms, JIT/warm),
not GreptimeDB regressing (99–107 ms, stable). So state it as **~2–3× warm** going forward,
not a flat 2×. Both sub-100 ms → not hot-path-critical; GreptimeDB's metrics edge stays
*capability* (PromQL-native, Run 62), never agg speed.

**Verdict-currency fold (this pass):** added two ClickHouse edges to
`verdict-which-to-choose.md` Decision-Q2 — **cold selective object-store reads**
(scatter-vs-cluster, Runs 55/63) and **dynamic-attr path queries ~13×** (Run 61, with the
`Dynamic`-cast + promote-to-typed-column caveats) — and updated the metric-agg figure to
~2–3×. No recommendation change: the offsetting GreptimeDB wins (full-text cost tie,
non-blocking concurrency, object-count + warm-cache re-reads, Q6 not-latency-bound, native
ingest, upsert/DELETE ergonomics, PromQL GA, cheap retention) all re-confirmed Runs 51–66.

**Reproduce.** `for i in $(seq 7); do docker exec parallax-bench-clickhouse-1 clickhouse-client --time -q "SELECT service,avg(value) FROM metrics_hc GROUP BY service FORMAT Null"; done` vs GreptimeDB `/v1/sql` `execution_time_ms`.

### Run 68 — 2026-05-25 — Span-tree recursion: GreptimeDB v1.0.2 FAILS the table-self-join CTE (corrects Run 27)

**Pass target.** Rotate onto the traces span-tree slice (Run 27 ~pass 49). Re-verify the
flat anchored fetch + the in-DB recursive-CTE claim ("works on both").

**Environment.** Main stack, GreptimeDB `v1.0.2` / ClickHouse `v26.5.1.882` (re-pinned —
latest, no bump).

**Measured / verified:**

| Check | ClickHouse | GreptimeDB |
| --- | --- | --- |
| Flat anchored fetch (`WHERE trace_id=X`, 14 spans) | ~2 ms | ~15 ms (re-verified, anchored-lookup floor) |
| Recursive CTE — **counter** (`SELECT 1 … n+1 … n<5`) | ✓ | ✓ (`n=5`) |
| Recursive CTE — **table self-join span-tree** (`… c JOIN t ON c.pid=t.sid`) | ✓ **3 rows / depth 2** (clean root→child→grandchild) | ✗ **`Schema error: project index out of bounds`** (both 1-col and 2-col recursive projections) |
| Root empty-parent representation | `''` | **`NULL`** (CSV empty → NULL; base case must use `IS NULL`) |

**The comparison logic & verdict.**

- **CORRECTS Run 27.** Run 27 logged "the recursive join *ran* on both" — that was the
  **counter** form. The **span-tree pattern** (recursive term joins the base table to the
  recursive relation) **errors on GreptimeDB v1.0.2** (`project index out of bounds`,
  reproduced 1-col + 2-col), while ClickHouse runs it correctly. So **in-DB span-tree
  recursion is a ClickHouse capability edge, not a tie** — a DataFusion recursive-CTE
  projection limitation in this GreptimeDB version.
- **Practical impact LOW.** The dominant span-tree pattern is the **flat anchored fetch +
  app-side tree build** (what Jaeger/Tempo do) — re-verified working on both (CH ~2 ms /
  GT ~15 ms). In-DB recursion is only needed for server-side tree analytics
  (critical-path, descendant rollups). So this **does not block Parallax** (it builds
  trees app-side) and **does not move the verdict** — but it is a genuine, mechanism-grounded
  ClickHouse edge to record honestly, and a GreptimeDB upstream-fix candidate.
- Also corrected: the earlier synthetic-`spans` recursion returned degenerate counts
  because (a) the synthetic parent links don't form a connected tree from the root, and
  (b) GreptimeDB stores the root's empty parent as `NULL`. The clean 3-node tree isolates
  the real capability difference.

Caveat: `v1.0.2`-specific (DataFusion recursive CTE is young — re-check on bumps); recursion
latency-vs-depth unmeasured (moot for GreptimeDB until the form is supported).

**Reproduce (copy-paste).**

```bash
# CH: clean tree recursion works
docker exec parallax-bench-clickhouse-1 clickhouse-client --multiquery -q "CREATE TABLE tree_ch (sid String, pid String, nm String) ENGINE=MergeTree ORDER BY sid; INSERT INTO tree_ch VALUES ('s1','','root'),('s2','s1','child'),('s3','s2','grandchild'); WITH RECURSIVE t AS (SELECT sid,pid,nm,0 d FROM tree_ch WHERE pid='' UNION ALL SELECT c.sid,c.pid,c.nm,t.d+1 FROM tree_ch c JOIN t ON c.pid=t.sid) SELECT count() n, max(d) FROM t"   # 3, 2
# GT: counter works, table-self-join errors
docker exec parallax-bench-greptimedb-1 curl -s localhost:4000/v1/sql?db=public --data-urlencode "sql=WITH RECURSIVE t AS (SELECT 1 AS n UNION ALL SELECT n+1 FROM t WHERE n<5) SELECT count(*),max(n) FROM t"   # 5,5
docker exec parallax-bench-greptimedb-1 curl -s localhost:4000/v1/sql?db=public --data-urlencode "sql=CREATE TABLE tree_gt (ts TIMESTAMP(3) TIME INDEX, sid STRING, pid STRING, nm STRING, PRIMARY KEY(sid))"
docker exec parallax-bench-greptimedb-1 curl -s localhost:4000/v1/sql?db=public --data-urlencode "sql=INSERT INTO tree_gt VALUES (1000,'s1','','root'),(2000,'s2','s1','child'),(3000,'s3','s2','grandchild')"
docker exec parallax-bench-greptimedb-1 curl -s localhost:4000/v1/sql?db=public --data-urlencode "sql=WITH RECURSIVE t AS (SELECT sid,0 AS d FROM tree_gt WHERE pid='' UNION ALL SELECT c.sid,t.d+1 FROM tree_gt c JOIN t ON c.pid=t.sid) SELECT count(*) FROM t"   # Schema error: project index out of bounds
```

### Run 69 — 2026-05-25 — WAL/durability re-verified: GreptimeDB has one, ClickHouse's is obsolete (live)

**Pass target.** Re-verify the durability/scaling pillar — "GreptimeDB has a replayable
WAL (Kafka decouples durability → scaling enabler); ClickHouse MergeTree has no functional
WAL" (Run 20 ~pass 41).

**Environment.** Main stack, GreptimeDB `v1.0.2` / ClickHouse `v26.5.1.882` (re-pinned —
latest, no bump).

**Verified live:**

| Check | Result |
| --- | --- |
| GreptimeDB WAL present | `/greptimedb_data/wal/*.raftlog` — 11 raft-engine segments, **~1.4 GB** (grows with writes, purged after flush). The replayable WAL is active. |
| ClickHouse WAL settings status | `system.merge_tree_settings`: `in_memory_parts_enable_wal` and `write_ahead_log_max_bytes` both **`is_obsolete = 1`** (+ `min_rows_for_compact_part` obsolete). |
| ClickHouse part types | only **`Compact` (39) / `Wide` (20)** — **no `InMemory`** part type (the feature the WAL served is gone). |
| ClickHouse WAL files on disk | `find /var/lib/clickhouse -name '*wal*'` → **none**. |
| ClickHouse fsync defaults | `fsync_after_insert=0`, `fsync_part_directory=0` (throughput-over-durability). |

**Verdict.** **No drift — pillar confirmed + precisely sourced.** GreptimeDB has a real,
active replayable WAL (raft-engine local; Kafka remote decouples durability from the
datanode = the compute/storage-separation behind the scaling verdict). ClickHouse has
**no functional WAL** — the lingering `in_memory_parts_enable_wal`/`write_ahead_log_*`
settings are runtime-flagged **`is_obsolete=1`** (the in-memory-parts feature is removed:
no `InMemory` parts, no WAL files), and durability is the un-fsynced part write
(`fsync_after_insert=0`), with crash-safety delegated to `ReplicatedMergeTree`+Keeper.
Both default throughput-over-fsync; **only GreptimeDB has a replay log.** Status:
**confirmed.** (Strengthens Run 20's source cite with live `is_obsolete=1` + part-type +
no-wal-file evidence.)

Caveat: durability is mechanism/config-verified, not crash-tested (a real crash-recovery
benchmark is harness territory).

**Reproduce.**

```bash
docker exec parallax-bench-greptimedb-1 sh -c 'ls -la /greptimedb_data/wal; du -sh /greptimedb_data/wal'
docker exec parallax-bench-clickhouse-1 clickhouse-client -q "SELECT name,is_obsolete FROM system.merge_tree_settings WHERE name IN ('in_memory_parts_enable_wal','write_ahead_log_max_bytes')"
docker exec parallax-bench-clickhouse-1 clickhouse-client -q "SELECT part_type,count() FROM system.parts WHERE active GROUP BY part_type"
docker exec parallax-bench-clickhouse-1 sh -c "find /var/lib/clickhouse -name '*wal*'"
```

### Run 70 — 2026-05-25 — Rollup re-verified (Flow vs MV): correctness tie + a freshness tilt to ClickHouse

**Pass target.** Rotate onto rollup/continuous-aggregation (Run 43 ~pass 43). Re-verify
GreptimeDB Flow and ClickHouse MV+AggregatingMergeTree both produce correct rollups, and
characterize freshness.

**Environment.** Main stack, GreptimeDB `v1.0.2` / ClickHouse `v26.5.1.882` (re-pinned —
latest, no bump). 4 source rows → minute+service rollup (`avg`, `count`).

**Measured:**

| | GreptimeDB Flow | ClickHouse MV + AggregatingMergeTree |
| --- | --- | --- |
| DDL | `CREATE FLOW f SINK TO sink AS SELECT date_bin('1 minute', ts), svc, avg(val), count(val) GROUP BY` (sink **auto-created**) | sink `AggregateFunction(avg)`+`SimpleAggregateFunction(sum)` + `CREATE MATERIALIZED VIEW … avgState/count()` |
| Result (api m0 / web m0 / api m1) | **15·2 / 5·1 / 30·1** | **15·2 / 5·1 / 30·1** (identical) |
| Read form | plain values (`avg_val=15`) | `avgMerge(avg_state)`, `sum(n)` (the `-State`/`-Merge` ceremony) |
| **Freshness** | **batched** — sink empty until `ADMIN FLUSH_FLOW` (streaming mode is low-latency; default/batching path is interval/flush) | **synchronous on INSERT** — 3 sink rows present immediately, no flush (push-MV runs per insert block) |

**Verdict.** **Both correct (tie), opposite tilts reproduce (Run 43).** GreptimeDB Flow:
cleaner, metric-native (`date_bin`, auto-sink, plain-value reads), younger. ClickHouse MV:
more ceremony (`-State`/`-Merge`, manual typed sink) but **fresher on the rollup path** —
the push-MV materializes inside the INSERT, while GreptimeDB Flow is batched (flush/interval).
**New sharpening:** for *real-time* rollup reads (dashboard refreshing a downsample seconds
after ingest) ClickHouse's MV is fresher; for eventually-consistent downsamples both fine.
A freshness tilt to ClickHouse **on the rollup path specifically** (raw-write freshness is
still a tie, Run 5). Neither moves the verdict. Status: **confirmed + freshness sharpened.**

Caveat: 4-row smoke; GreptimeDB *streaming* Flow mode (laminar) may narrow the freshness gap
vs the batching path tested here — re-check if real-time rollups become a Parallax need.

**Reproduce (copy-paste).**

```bash
docker exec parallax-bench-greptimedb-1 curl -s localhost:4000/v1/sql?db=public --data-urlencode "sql=CREATE TABLE flow_src (ts TIMESTAMP(3) TIME INDEX, svc STRING, val DOUBLE)"
docker exec parallax-bench-greptimedb-1 curl -s localhost:4000/v1/sql?db=public --data-urlencode "sql=CREATE FLOW f_rollup SINK TO flow_sink AS SELECT date_bin('1 minute'::INTERVAL, ts) AS t, svc, avg(val) AS avg_val, count(val) AS n FROM flow_src GROUP BY t, svc"
docker exec parallax-bench-greptimedb-1 curl -s localhost:4000/v1/sql?db=public --data-urlencode "sql=INSERT INTO flow_src VALUES (1716000001000,'api',10),(1716000002000,'api',20),(1716000003000,'web',5),(1716000061000,'api',30)"
docker exec parallax-bench-greptimedb-1 curl -s localhost:4000/v1/sql?db=public --data-urlencode "sql=ADMIN FLUSH_FLOW('f_rollup')"; docker exec parallax-bench-greptimedb-1 curl -s localhost:4000/v1/sql?db=public --data-urlencode "sql=SELECT * FROM flow_sink ORDER BY t,svc"
# ClickHouse: MV materializes synchronously (no flush)
docker exec parallax-bench-clickhouse-1 clickhouse-client --multiquery -q "CREATE TABLE mv_src (ts DateTime, svc String, val Float64) ENGINE=MergeTree ORDER BY ts; CREATE TABLE mv_sink (t DateTime, svc String, avg_state AggregateFunction(avg,Float64), n SimpleAggregateFunction(sum,UInt64)) ENGINE=AggregatingMergeTree ORDER BY (t,svc); CREATE MATERIALIZED VIEW mv TO mv_sink AS SELECT toStartOfMinute(ts) t, svc, avgState(val) avg_state, toUInt64(count()) n FROM mv_src GROUP BY t,svc; INSERT INTO mv_src VALUES (toDateTime('2024-05-18 03:00:01'),'api',10),(toDateTime('2024-05-18 03:00:02'),'api',20),(toDateTime('2024-05-18 03:00:03'),'web',5),(toDateTime('2024-05-18 03:01:01'),'api',30);"
docker exec parallax-bench-clickhouse-1 clickhouse-client -q "SELECT t,svc,avgMerge(avg_state),sum(n) FROM mv_sink GROUP BY t,svc ORDER BY t,svc"
```

### Run 71 — 2026-05-25 — Projections re-verified (~1.9× storage, optimizer-picked) + Run-63 link

**Pass target.** Rotate onto projections/access-paths (Run 28 ~pass 50). Re-verify "CH
projection = 2nd physical ORDER BY, optimizer-picked, ~2× storage; GreptimeDB has none."

**Environment.** Main stack, GreptimeDB `v1.0.2` / ClickHouse `v26.5.1.882` (re-pinned —
latest, no bump). 500k rows; base `ORDER BY (a,ts)`, projection `p_b (ORDER BY b)`.

**Measured:**

| Check | Result |
| --- | --- |
| Storage no-projection | **2.41 MiB** |
| Storage with-projection | **4.52 MiB** → **~1.9×** (projection is a near-full 2nd copy; matches Run 28's 2→4 MiB) |
| Optimizer picks projection for `WHERE b=…` | `EXPLAIN indexes=1` → **`ReadFromMergeTree (p_b)` Granules 1/62** (transparent) |
| Latency (b-filter) | 2 ms with projection vs 4 ms without (alternate-key scan accelerated) |
| GreptimeDB `PROJECTION` DDL | **rejected** — *"Cannot use keyword 'PROJECTION'"* (no equivalent) |

**Verdict.** **Reproduces Run 28, no drift.** ClickHouse projections give a second physical
`ORDER BY` inside each part, optimizer-transparently chosen, at **~1.9× storage**;
GreptimeDB has no projection (uses secondary indexes = row positions, not a 2nd physical
order). **Link to Run 63 (cold reads):** a `trace_id`-ordered projection is exactly how
ClickHouse *could* also get anchor-clustering for cheap cold selective reads — the
alternate-physical-order GreptimeDB structurally lacks (PK=sort=series identity, Run 65).
So projections are the storage-vs-locality lever behind both the multi-ordering-scan edge
**and** the cold-egress edge: ClickHouse can pay ~2× storage for anchor locality,
GreptimeDB cannot. Parallax's anchored pattern doesn't need a 2nd scan order, and its
GreptimeDB inverted index is leaner for anchored point/filter — so **neither moves the
verdict** (parity #5 footnote). Status: **confirmed.**

Caveat: 500k smoke; scan-with-projection vs index-lookup at GB scale unmeasured.

**Reproduce.** `CREATE TABLE proj_yes (a String,b String,ts DateTime,v UInt64, PROJECTION p_b (SELECT * ORDER BY b)) ENGINE=MergeTree ORDER BY (a,ts)` + `INSERT … numbers(500000)` + `EXPLAIN indexes=1 SELECT count() FROM proj_yes WHERE b='b500'` (→ ReadFromMergeTree(p_b)); GreptimeDB `CREATE TABLE … PROJECTION …` → rejected.

### Run 72 — 2026-05-25 — Index file formats re-verified (.puffin vs per-part zoo) + text-index decomposition

**Pass target.** Rotate onto indexing internals (Run 22 ~pass 43). Re-verify the on-disk
index-format contrast (GreptimeDB `.puffin` sidecar vs ClickHouse per-part skip-index files)
and decompose what the ClickHouse `text` index actually is.

**Environment.** Main stack, GreptimeDB `v1.0.2` / ClickHouse `v26.5.1.882` (re-pinned —
latest, no bump). Live filesystem inspection.

**Verified:**

- **GreptimeDB:** `.puffin` files in an `index/` subdir per region —
  `…/public/<table_id>/<region>/index/<uuid>.puffin` (UUID matches the SST). One `.puffin`
  per indexed SST, holding *all* that SST's indexes as named blobs. So an indexed table =
  `.parquet` + `.puffin` = **2 files per SST** (the spans_idx puffin = 5.8 MiB, matches the
  Run-54 inverted-index size).
- **ClickHouse `logs_b1` part `all_1_5_1`:** `primary.cidx` (2.5 KB sparse primary) + the
  `text` index as **a cluster of files** — `skp_idx_idx_msg.idx` (238 KB skip) +
  **`skp_idx_idx_msg.dct.idx` (97 MB term dictionary)** + **`skp_idx_idx_msg.pst.idx`
  (81 MB posting lists)** + `.cmrk2` mark file each + per-column `.bin`/`.cmrk2`. **37 files
  in one part.**

**The comparison logic & verdict.**

- **`text` is a true dict+postings inverted index (Lucene-shaped), decomposed live:** the
  `.dct.idx` (97 MB) + `.pst.idx` (81 MB) ≈ 178 MB raw are the bulk of the 170 MiB
  text-index measured in Run 51. So ClickHouse's GA `text` is a real inverted index (term
  dictionary + posting lists), not merely a bloom skip — confirms + decomposes the Run 51
  size.
- **File count is the root of the object-store gap (links Run 22 ↔ Run 54):** GreptimeDB =
  **2 files/SST** (`.parquet` + `.puffin`); ClickHouse = **37 files/part** (per-column +
  per-index dict/postings/skip + marks). On object storage each file → an object, so this
  *is* the mechanism behind Run 54's CH 74 objects vs GreptimeDB 3 — index format and object
  count are two views of the same file-per-everything-vs-few-large-files difference.
- GreptimeDB's index toolkit stays richer/more precise (FST+roaring inverted, tantivy
  full-text, configurable-granularity bloom) in **one** sidecar; ClickHouse spreads a
  sparse primary + per-skip-index file clusters across the part. **Status: confirmed +
  decomposed; no drift.**

Caveat: file inspection (exact structure), not a latency run; the index *speed* findings are
Runs 6/22/48–49.

**Reproduce.**

```bash
docker exec parallax-bench-greptimedb-1 sh -c 'find /greptimedb_data/data -name "*.puffin"'   # index/ subdir, 1 per SST
P=$(docker exec parallax-bench-clickhouse-1 clickhouse-client -q "SELECT path FROM system.parts WHERE active AND table='logs_b1' LIMIT 1")
docker exec parallax-bench-clickhouse-1 sh -c "ls -la '$P' | grep skp_idx"   # .idx + .dct.idx + .pst.idx + .cmrk2
docker exec parallax-bench-clickhouse-1 sh -c "ls '$P' | wc -l"   # 37
```

### Run 73 — 2026-05-25 — Per-column codec compression re-verified (the stalest numeric claim, exact)

**Pass target.** Re-verify the oldest load-bearing numeric claim — ClickHouse's per-column
codec ratios (Run 4 ~pass 8: gauge Gorilla 78×, counter DoubleDelta 7.3×) behind "CH wins
tuned numeric columns" + the per-pattern compression wash.

**Environment.** Main stack, GreptimeDB `v1.0.2` / ClickHouse `v26.5.1.882` (re-pinned —
latest, no bump). `metrics_real` (864k rows, counter+gauge), `system.parts_columns`.

**Measured (CH per-column, reproduces Run 4):**

| Column | Codec | Compressed | Ratio (Run 4 → Run 73) |
| --- | --- | --- | --- |
| `gauge` | Gorilla, ZSTD | 84.7 KiB | 78× → **79.7×** |
| `counter` | DoubleDelta, ZSTD | 922 KiB | 7.3× → **7.3×** (exact) |
| `ts` | DoubleDelta, ZSTD | 10.1 KiB | 668× → **668×** (exact) |
| `service` | LowCardinality | 4.2 KiB | dict → **199×** |
| `instance` | LowCardinality | 10.0 KiB | dict → **85×** |

**Table total:** CH **1.09 MiB** vs GreptimeDB **1.89 MiB** → **CH ~1.7× smaller** on this
tuned-numeric table (reproduces Run 4's 1.09 vs 1.9 exactly).

**Verdict.** **No drift — exact reproduction at ~pass 109.** ClickHouse's hand-tuned codecs
hit the same ratios (Gorilla ~80× on flat gauge, DoubleDelta 7.3× on monotonic counter, 668×
on regular-step ts, LowCardinality 85–199× on low-card strings), and it stays ~1.7× smaller
than GreptimeDB on the tuned-numeric metrics table. "CH wins hand-tuned numeric columns" is
**stable**; the per-pattern wash holds (GreptimeDB's automatic Parquet+ZSTD wins dict-friendly
+ noisy-float, Run 10). Confirms the cost-axis note. Status: **confirmed.**

Caveat: `metrics_real` is synthetic (regular 30 s step → the 668× ts is best-case); real
jittered timestamps compress less. The *direction* (CH tuned-numeric edge) is robust.

**Reproduce.** `docker exec parallax-bench-clickhouse-1 clickhouse-client -q "SELECT column, formatReadableSize(sum(column_data_compressed_bytes)), round(sum(column_data_uncompressed_bytes)/sum(column_data_compressed_bytes),1) ratio FROM system.parts_columns WHERE active AND table='metrics_real' GROUP BY column ORDER BY 2 DESC"` vs GreptimeDB `information_schema.region_statistics` `sst_size` for `metrics_real`.

### Run 74 — 2026-05-25 — Distributed/scaling mechanism re-verified live (the OSS-scale-out-is-manual side)

**Pass target.** Rotate onto the last un-rotated slice (distributed/scaling, Run 34 ~pass
57). Multi-node *hold* is harness-gated, but the single-node-checkable scale-out mechanism
claims can be runtime-confirmed.

**Environment.** Main stack, GreptimeDB `v1.0.2` / ClickHouse `v26.5.1.882` (re-pinned —
latest, no bump).

**Verified live:**

| Claim | Result |
| --- | --- |
| `SharedMergeTree` is Cloud-only (not in OSS) | `CREATE … ENGINE=SharedMergeTree` → **`Unknown table engine SharedMergeTree (UNKNOWN_STORAGE)`** |
| `ReplicatedMergeTree` needs Keeper | `CREATE … ENGINE=ReplicatedMergeTree(...)` (no Keeper) → **`Can't create replicated table without ZooKeeper (NO_ZOOKEEPER)`** |
| Zero-copy replication off by default | `allow_remote_fs_zero_copy_replication = 0` (→ N× S3 copies, Run 34) |
| GreptimeDB single-node mode | `information_schema.cluster_info` = one **`STANDALONE`** peer (all roles in one binary; cluster split is multi-node) |

**Verdict.** **No drift — the OSS-ClickHouse-scale-out-is-manual claims are runtime-confirmed.**
OSS ClickHouse has **no SharedMergeTree** (the elastic compute/storage-separated engine is
Cloud-proprietary), its HA `ReplicatedMergeTree` **requires a separate Keeper**, and zero-copy
replication is **off by default** (each replica a full S3 copy). So OSS horizontal scale-out =
manual sharding + Keeper + N× storage. GreptimeDB's designed-in region/Metasrv/object-store-shared
model (1× S3, region rebalance, Run 34/57) is the "topology change not rewrite" answer — but its
multi-node **hold** (p95 flat as nodes added) stays **harness-gated** (can't test on one standalone
node). Status: **mechanism confirmed; multi-node hold owed to the harness.**

Caveat: this confirms *capability/architecture* (what OSS lacks), not a multi-node performance run —
the scaling *hold* is the standing open question #4 in `verdict-which-to-choose.md`.

**Reproduce.**

```bash
docker exec parallax-bench-clickhouse-1 clickhouse-client -q "CREATE TABLE smt (id UInt32) ENGINE=SharedMergeTree ORDER BY id"   # UNKNOWN_STORAGE
docker exec parallax-bench-clickhouse-1 clickhouse-client -q "CREATE TABLE rmt (id UInt32) ENGINE=ReplicatedMergeTree('/x','r1') ORDER BY id"   # NO_ZOOKEEPER
docker exec parallax-bench-clickhouse-1 clickhouse-client -q "SELECT value FROM system.merge_tree_settings WHERE name='allow_remote_fs_zero_copy_replication'"   # 0
docker exec parallax-bench-greptimedb-1 curl -s localhost:4000/v1/sql?db=public --data-urlencode "sql=SELECT peer_type FROM information_schema.cluster_info"   # STANDALONE
```

### Run 75 — 2026-05-25 — B15: strict-durability ingest cost (WAL-append fsync vs part fsync) — an open question advanced

**Pass target.** Advance a harness-gated open question (B15, strict-durability throughput):
what does *fsync-on-every-write* cost each engine? GreptimeDB `sync_write=true` (fsync each
WAL append) vs ClickHouse `fsync_after_insert=1` (fsync each part).

**Environment.** GreptimeDB `v1.0.2` / ClickHouse `v26.5.1.882` (re-pinned — latest, no
bump). orbstack overlay fs (slow fsync — inflates absolutes, but the *delta* is the cost).
Method: time N small writes with strict-durability OFF vs ON; the per-write **delta**
isolates the fsync cost (docker-exec overhead ~58–88 ms cancels). ClickHouse via two tables
(`fsync_after_insert`/`fsync_part_directory` are *table* settings, not query settings — first
attempt erred); GreptimeDB via two throwaway containers (`sync_write` is a `[wal]` config —
injected by create+cp+start; verified applied).

**Measured (per-write/part delta = the strict-durability cost):**

| Engine | durability OFF (default) | strict ON | **fsync delta** | what gets fsynced |
| --- | --- | --- | --- | --- |
| ClickHouse (`fsync_after_insert=1`, `fsync_part_directory=1`) | 88 ms/insert | 106 ms/insert | **~+18 ms/part** (~20%) | the whole **part** — multiple column files + the part directory |
| GreptimeDB (`sync_write=true`) | 59 ms/write | 60 ms/write | **~+1.7 ms/write** (~3%) | one **sequential WAL append** to the raft-engine log |

**Verdict.** **Strict-durable ingest is ~10× cheaper on GreptimeDB** (~1.7 ms WAL-append
fsync vs ~18 ms whole-part fsync). The mechanism is architectural: GreptimeDB fsyncs **one
sequential WAL append** per write (cheap, append-only log), while ClickHouse's
`fsync_after_insert` must fsync **a whole part** — its column files + the directory (many
fsyncs). So the WAL is not just a *replay* advantage (Run 20/69) but a **strict-durability
*throughput* advantage**: GreptimeDB can run fsync-on-write at ~3% cost, ClickHouse pays
~20% per part. For a Parallax tier that needs no-loss-on-crash ingest, GreptimeDB's WAL
makes it cheap; ClickHouse's realistic answer stays replica-redundancy (Keeper +
`ReplicatedMergeTree`), not per-part fsync. **Advances open question (B15 / verdict open #7)
from "owed" to "directionally measured: GreptimeDB ~10× cheaper strict-durable ingest."**

Caveat: orbstack overlay-fs fsync is slow (inflates both absolutes); on NVMe both shrink but
the **ratio** (sequential-WAL-append-fsync ≪ whole-part-fsync) is architectural, not
disk-specific. Smoke rate (60 writes); the *sustained* strict-durable throughput ceiling is
still a sized-harness number. docker-exec overhead dominates the absolute per-write time —
only the OFF→ON delta is the result.

**Reproduce (copy-paste).**

```bash
# ClickHouse: table-level fsync settings
docker exec parallax-bench-clickhouse-1 clickhouse-client -q "CREATE TABLE fon (id UInt64,v String) ENGINE=MergeTree ORDER BY id SETTINGS fsync_after_insert=1, fsync_part_directory=1"
docker exec parallax-bench-clickhouse-1 clickhouse-client -q "CREATE TABLE foff (id UInt64,v String) ENGINE=MergeTree ORDER BY id"
# time 60 small INSERTs into each (delta = fsync cost)
# GreptimeDB: throwaway container with [wal] sync_write=true vs default; time 60 influx writes each
printf '[wal]\nprovider = "raft_engine"\nsync_write = true\n' > /tmp/s.toml
docker run -d --name gt-def greptime/greptimedb:v1.0.2 standalone start --http-addr 0.0.0.0:4000
docker create --name gt-sync greptime/greptimedb:v1.0.2 standalone start --http-addr 0.0.0.0:4000 -c /sync.toml; docker cp /tmp/s.toml gt-sync:/sync.toml; docker start gt-sync
# for c in gt-def gt-sync: time 60x  curl -XPOST .../v1/influxdb/write --data-binary "m,svc=a v=$i"
```

### Run 76 — 2026-05-25 — B13: high-cardinality metric storage (200k series) + the LowCardinality cliff refined

**Pass target.** Advance B13 (sized high-card metric storage, open Q #8): does ClickHouse's
`LowCardinality` 8,192-dict cliff (Run 26) inflate storage at high series count vs
GreptimeDB? Generate 200k distinct series, compare.

**Environment.** GreptimeDB `v1.0.2` / ClickHouse `v26.5.1.882` (re-pinned — latest, no
bump). 1M rows, `series` with **200,000 distinct values** (`'svc-'||number%200000`),
identical data both engines (CH-generated, CSV-loaded into GreptimeDB).

**Measured (total on disk, OPTIMIZE/compact):**

| Table | total | `series` column |
| --- | --- | --- |
| ClickHouse `LowCardinality(String)` | **9.64 MiB** | 1.53 MiB |
| ClickHouse `String` (plain) | 10.11 MiB | 1.99 MiB |
| GreptimeDB plain mito table (`series` PK) | **11.99 MiB** | — |

**Verdict — two findings, one caveat.**

- **The `LowCardinality` "cliff" is GRACEFUL, not a storage explosion (refines Run 26).**
  At 200k distinct (≫ the 8,192 dict cap), `LowCardinality` is **still smaller than plain
  `String`** (col 1.53 vs 1.99 MiB; total 9.64 vs 10.11). Overflowing the dict = *losing
  the peak dict benefit*, not regressing below `String` (helped by `ORDER BY series`
  per-granule locality + ZSTD). So "the cliff" is a don't-expect-magic caveat, not a
  storage footgun.
- **On a *plain* table, ClickHouse wins high-card series storage ~1.24×** (LC 9.64 vs GT
  11.99 MiB) — consistent with the tuned-codec-on-high-card-strings edge (Run 1).
- **⚠ Caveat — NOT GreptimeDB's high-card path.** The GT table stored `series` as a full
  string; the **metric engine** identifies series by a u64 `__tsid` hash (not the
  `'svc-N'` string), potentially far more compact. The metric-engine high-card storage
  comparison is **owed** (physical `ENGINE=metric` table creates; loading 200k series via
  logical tables/OTLP is the follow-up). So this measures *plain-table* GT, likely
  overstating GT's high-card storage.

**Net:** refines the verdict's "high-card → GreptimeDB" to: **ingest ergonomics** (no
cardinality cap, no `ORDER BY` tuning) → GreptimeDB; **raw plain-table storage** →
ClickHouse `LowCardinality` (~1.24×); **aggregation latency** → ClickHouse (Run 26); the
metric-engine `__tsid` storage is the owed tiebreaker. Status: **B13 partially advanced
(plain-table storage measured + cliff refined; metric-engine path owed).**

Caveat: 200k series / 1M rows smoke; metric-engine path owed; CH ORDER BY series gives it
sorted-column locality (fair — it's the recommended high-card schema).

**Reproduce.** CH `CREATE TABLE hc_lc (series LowCardinality(String), ts DateTime, val Float64) ENGINE=MergeTree ORDER BY (series,ts)` + `INSERT … 'svc-'||toString(number%200000) … numbers(1000000)`; dump `FORMAT CSVWithNames`, `COPY` into GreptimeDB `hc_gt (series STRING, ts_ms TIMESTAMP(3) TIME INDEX, val DOUBLE, PRIMARY KEY(series))`; compare `system.parts` vs `region_statistics`.

### Run 77 — 2026-05-25 — B13 complete: metric-engine `__tsid` high-card storage (CH wins, corrects Run 26)

**Pass target.** Close the Run-76-owed fair tiebreaker: does GreptimeDB's **metric engine**
(series as a u64 `__tsid` hash) store 200k high-card series more compactly than the plain
table (11.99 MiB) or CH `LowCardinality` (9.64 MiB)?

**Environment.** GreptimeDB `v1.0.2` / ClickHouse `v26.5.1.882` (re-pinned — latest, no
bump). Same 200k-series / 1M-row data. Built the metric engine: physical `hc_phy`
(`ENGINE=metric WITH('physical_metric_table'='')`) + logical `hc_log`
(`ENGINE=metric WITH('on_physical_table'='hc_phy')`, `series STRING PRIMARY KEY`); loaded via
a staging table + `INSERT … SELECT`.

**Measured (200k series, 1M rows, full ladder):**

| Storage | total |
| --- | --- |
| ClickHouse `LowCardinality(String)` | **9.64 MiB** |
| ClickHouse `String` | 10.11 MiB |
| GreptimeDB plain mito table | 11.99 MiB |
| **GreptimeDB metric engine** (`__tsid`) | **12.63 MiB** |

**Verdict — B13 storage COMPLETE; corrects Run 26.** The metric engine is **not smaller**
— it is *slightly larger* than the plain table (12.63 vs 11.99 MiB). `__tsid` (the u64
label-set hash) + `__table_id` are stored **in addition to** the label columns (the physical
table keeps labels for query), so the hash is **overhead for fast series identity +
multi-metric sharing, not a storage replacement.** **ClickHouse `LowCardinality` wins
high-card metric *storage* ~1.3×** over both GreptimeDB layouts. So the Run-26 "high-card
*storage* → GreptimeDB" is **refuted on raw bytes**: GreptimeDB's high-card edge is purely
**ingest ergonomics/operability** (no `LowCardinality` cap, no `ORDER BY` tuning, many
logical metrics → one physical table, label-set hashing) — **not** storage size (→ CH) and
**not** aggregation latency (→ CH ~2–3×, Run 26/67). Status: **B13 storage complete; verdict
high-card cell corrected.**

Caveat: 200k series / 1M rows smoke; the metric engine's *operational* wins (cap-free
ingest, multi-metric consolidation, repartition growth) are real and not about bytes — this
measures bytes only. Method gotcha logged: GreptimeDB COPY-CSV matches columns **by name**
(header `ts_ms` vs column `ts` → "missing column ts"); name them to match.

**Reproduce.** Build `hc_phy`/`hc_log` (metric engine), stage the 200k-series CSV in a plain
table, `INSERT INTO hc_log (ts,val,series) SELECT …`, `ADMIN flush_table('hc_phy')`, read
`region_statistics.sst_size` for `hc_phy` (12.63 MiB) vs CH `system.parts` (9.64 MiB LC).

### Run 78 — 2026-05-25 — Full-text selective latency re-verified (the most-corrected headline holds)

**Pass target.** Drift-watch the single most-corrected claim: selective full-text search
is **~2× ClickHouse, not the originally-reported ~18×** (Runs 48–49) — the artifact was
`matches()` (tantivy query-syntax fn) on a `backend='bloom'` index full-scanning. Re-verify
both the correct pairing and the artifact reproduce.

**Environment.** Main stack, GreptimeDB `v1.0.2` / ClickHouse `v26.5.1.882` (re-pinned —
latest, no bump). `logs_b1` (5M, full-text on `message`), unique term `0835d162` (1 row).

**Measured (warm, min of 7):**

| Query | latency | note |
| --- | --- | --- |
| ClickHouse `hasToken(message,'…')` (text index) | **~3 ms** | posting-list prune + vectorized confirm |
| GreptimeDB `matches_term(message,'…')` (bloom backend) | **~8 ms** | bloom prune + scan confirm → **~2.7× CH** |
| GreptimeDB `matches(message,'…')` (bloom, **wrong** pairing) | **~157 ms** | full-scans 5M (no index push) — the **18× artifact** |

**Verdict.** **Reproduces exactly, no drift.** Correct pairing: CH ~3 ms vs GreptimeDB
~8 ms = **~2.7×** (the corrected ~2–3× band, both sub-perceptible). The **wrong** pairing
(`matches()` on a bloom index) still full-scans ~157 ms — the exact artifact that produced
the false ~18× when compared against the old GreptimeDB number. So the headline correction
(selective full-text is interactive-fast on both, ~2–3×; the residual real gap is broad-term
analytics, Run 48) **holds at current versions.** Status: **confirmed.** The verdict's
log-search cell stands.

Caveat: warm cache-resident smoke (5M); broad-term (many-row) full-text scan latency at
volume remains the residual gap (~12×, Run 48 / `query-execution-engine.md`).

**Reproduce.** `docker exec …clickhouse… --time -q "SELECT count() FROM logs_b1 WHERE hasToken(message,'0835d162') FORMAT Null"` (~3 ms) vs GreptimeDB `…matches_term(message,'0835d162')` (~8 ms) and `…matches(message,'0835d162')` (~157 ms, full-scan).

### Run 79 — 2026-05-25 — High-card storage CURVE: a CROSSOVER (CH wins low-mid, GreptimeDB wins extreme)

**Pass target.** Complete B13's sized curve (open Q #8 remainder): does the Run-76/77
"ClickHouse wins high-card storage ~1.3×" hold across cardinality, or shift? Bracket the
200k point with 1k (high-repeat) and 1M (all-unique series).

**Environment.** GreptimeDB `v1.0.2` / ClickHouse `v26.5.1.882` (re-pinned — latest, no
bump). Fixed **1M rows**, distinct series ∈ {1k, 200k, 1M}; CH `LowCardinality(String)`
`ORDER BY (series,ts)` vs GreptimeDB plain mito table `PRIMARY KEY(series)`; identical data.

**Measured (total on disk):**

| distinct series | ClickHouse `LowCardinality` | GreptimeDB plain | winner |
| --- | --- | --- | --- |
| 1,000 (1000 rows/series) | **8.18 MiB** | 9.18 MiB | ClickHouse ~1.12× |
| 200,000 (Run 76) | **9.64 MiB** | 11.99 MiB | ClickHouse ~1.24× |
| 1,000,000 (1 row/series, all-unique) | 16.51 MiB | **12.36 MiB** | **GreptimeDB ~1.34×** |

**Verdict — there is a CROSSOVER; "CH wins high-card storage" is cardinality-dependent.**
ClickHouse `LowCardinality` wins at **low-to-mid** cardinality (1k–200k), but at
**extreme** cardinality (1M distinct, every series unique) it **blows up to 16.51 MiB**
while GreptimeDB grows gently to 12.36 — **GreptimeDB wins ~1.34× at 1M series.** Mechanism:
`LowCardinality`'s dict caps at 8,192 and gives diminishing returns as values stop
repeating; at all-unique it is pure dict overhead over near-raw values, so CH's storage
climbs steeply (9.64 → 16.51 from 200k → 1M). GreptimeDB's Parquet dict/RLE + ZSTD over the
`series`-sorted data degrades more gracefully (11.99 → 12.36). So **the Run-76/77 "CH wins
high-card storage" holds only up to ~hundreds-of-thousands of series; past ~1M unique series
GreptimeDB wins** — which is exactly the regime GreptimeDB's metric engine is designed for.

**Decision-useful framing for Parallax.** If metric series cardinality is **moderate**
(service × instance × endpoint ≈ thousands–100k), ClickHouse stores ~1.1–1.25× smaller. If
it is **extreme** (per-user / per-request / per-fingerprint labels → ~1M+ unique series),
GreptimeDB stores smaller **and** ingests cap-free (no `LowCardinality` 8,192 management).
So GreptimeDB's high-card edge is real specifically at the **very-high-cardinality** end +
ingest operability; ClickHouse wins the moderate-cardinality storage. Status: **B13 curve
complete — crossover at ~hundreds-of-k → 1M series.**

Caveat: smoke (1M rows); the 1M case is also 1M distinct timestamps (both compress ts well —
the `series` column is the driver). Per-column attribution of the crossover (series vs ts vs
val) is a detail; the total-storage crossover is the result. A true sized run (1M rows ×
many series counts at larger row volume) is the harness extension.

**Reproduce.** CH `INSERT … 'svc-'||toString(number % N) … numbers(1000000)` for N ∈
{1000, 200000} and `'svc-'||toString(number)` for all-unique; `OPTIMIZE FINAL`; compare
`system.parts` bytes vs GreptimeDB `region_statistics.sst_size` after CSV-load + compact.

### Run 80 — 2026-05-25 — Logs selective filter re-verified + refined (keyed = tie, broad = CH ~2×)

**Pass target.** Re-verify the logs-signal selective-filter claim (Run 1: CH 3 ms vs GT
9 ms) on the bigger keyed table, and characterize how it varies with selectivity/keying.

**Environment.** Main stack, GreptimeDB `v1.0.2` / ClickHouse `v26.5.1.882` (re-pinned —
latest, no bump). `logs_b1` (5M; 12 services × 3 levels). CH `ORDER BY (service, ts)`,
GreptimeDB `PRIMARY KEY (service, level)` — **both key the filter columns** (unlike Run 1's
original `logs` table, where GreptimeDB left them un-keyed).

**Measured (warm):**

| Filter | rows matched | ClickHouse | GreptimeDB | ratio |
| --- | --- | --- | --- | --- |
| `service='svc-0' AND level='ERROR'` (selective, keyed) | 50,096 / 5M | **~4 ms** | **~5 ms** | **~tie** (CH prunes 51/611 granules) |
| `level='ERROR'` (broader, level not sort-prefix) | 599,916 / 5M | **~6 ms** | **~12 ms** | **~2× CH** |

**Verdict — refines Run 1.** The Run-1 "CH 3 ms vs GT 9 ms" log-filter gap was a
**key-placement** effect (the original `logs` table left GreptimeDB's filter columns
un-keyed → scan). On `logs_b1`, where **both engines key `service`(+`level`)**, a highly
selective keyed filter is a **near-tie** (CH 4 ms / GT 5 ms — both prune to the keyed
range). The gap **reappears as the filter broadens** (level-only, 600k rows: CH ~6 ms vs GT
~12 ms, ~2×) — once many rows match, ClickHouse's vectorized scan over the matched set wins
(consistent with the scan-engine findings, Run 58). So: **keyed + highly-selective log
filter = tie; broad/scan-heavy log filter = ClickHouse ~2×.** The "CH wins logs" claim is
specifically the *broad-scan / un-keyed* case, not the anchored/keyed selective one.
Status: **confirmed + refined.**

Caveat: warm 5M smoke; the ratio at the broad end should grow with row volume (scan
throughput, cold). Both sub-15 ms here — not interactive-perceptible.

**Reproduce.** `docker exec …clickhouse… --time -q "SELECT count() FROM logs_b1 WHERE service='svc-0' AND level='ERROR' FORMAT Null"` (~4 ms) / `… WHERE level='ERROR' …` (~6 ms); GreptimeDB same via `/v1/sql` `execution_time_ms` (~5 ms / ~12 ms); `EXPLAIN indexes=1` on CH → `Granules 51/611`.

### Run 81 — 2026-05-25 — Q4 cross-tier join: GreptimeDB does NOT push the anchor into the join (corrects Run 30)

**Pass target.** Re-verify Q4 (cross-tier `spans LEFT JOIN error_events` anchored on
`trace_id`; Run 30: CH 5 ms / GT 59 ms, claimed "both prune the anchor before joining, GT
gap = HTTP floor + repartition artifact"). Check the GT number + plan with the validated
`execution_time_ms` basis (Run 60).

**Environment.** Main stack, GreptimeDB `v1.0.2` / ClickHouse `v26.5.1.882` (re-pinned —
latest, no bump). Anchor `trace_id=3fb2d84c…` (14 spans, 1 error). CH `spans`, GT `spans_idx`.

**Measured (warm):** correctness 14 rows / 1 matched error, both.

| Form | ClickHouse | GreptimeDB |
| --- | --- | --- |
| Q4 direct `LEFT JOIN` (WHERE on left `trace_id`) | **~4 ms** | **~54 ms** |
| GreptimeDB Q4 with **subquery pre-filter** (`FROM (SELECT * … WHERE trace_id=…) s LEFT JOIN …`) | — | **~21 ms** |

**`EXPLAIN ANALYZE` — the mechanism (corrects Run 30):**

- **GreptimeDB direct join: `UnorderedScan` on `spans_idx` `output_rows: 1,000,000`** — it
  **full-scans all 1M spans**; the `WHERE s.trace_id='X'` is **NOT pushed into the
  left-table scan** (the inverted index is not applied inside the join plan). That is the
  ~54 ms (a 1M-row scan + join), **not** an HTTP/repartition artifact (this is server-side
  `execution_time_ms`, fair per Run 60).
- **GreptimeDB subquery rewrite: scan `output_rows: 14`** — pre-filtering the left table
  forces the inverted-index prune → ~21 ms (still has the index-lookup floor + join, ~5×
  CH).
- **ClickHouse direct join prunes automatically** (Run 30 EXPLAIN: `Granules 1` + PREWHERE
  `trace_id`) → ~4 ms.

**Verdict — CORRECTS Run 30.** Run 30's "both anchor-prune-before-join; GT gap = HTTP floor
+ 10-way repartition of a toy input" is **wrong for GreptimeDB**: GreptimeDB's optimizer
**does not push a left-side equality filter through a LEFT JOIN into the indexed scan**, so
it **full-scans the 1M-row left table** (~54 ms, ~13× CH). It is a genuine **predicate-
pushdown-into-join optimizer limitation**, not a measurement artifact. **Workarounds:**
(a) pre-filter the left table in a subquery → forces the prune (~21 ms, ~2.5× better);
(b) Parallax's app-side correlation — anchored fetch each signal (Q1, ~15–21 ms) + join in
the app — avoids the in-DB join entirely. ClickHouse handles the direct join fine (~4 ms).
So **cross-tier correlation as a direct in-DB join favours ClickHouse** (auto-prune); for
GreptimeDB, rewrite or assemble app-side. Both stay < the 300 ms gate, but this is a real
~5–13× gap and a corrected mechanism. Status: **corrected; GreptimeDB join-pushdown gap is a
new parity-roadmap candidate.**

Caveat: 1M-row left table smoke; the gap scales with the un-pruned left-table size (worse at
volume) — exactly why the subquery rewrite / app-side join matters.

**Reproduce.** `EXPLAIN ANALYZE` the direct join on GreptimeDB (`spans_idx` scan
`output_rows: 1000000`) vs the subquery form (`output_rows: 14`); CH `--time` direct join
~4 ms.

### Run 82 — 2026-05-25 — Join-pushdown gap characterized (INNER + LEFT) → parity Improvement #8

**Pass target.** Deepen Run 81's GreptimeDB join-pushdown gap: is it LEFT-JOIN-specific or
general? Feed the parity-roadmap.

**Environment.** GreptimeDB `v1.0.2` / ClickHouse `v26.5.1.882` (re-pinned — latest, no
bump). `spans_idx` (1M, `trace_id` inverted), anchor `trace_id=3fb2d84c…`; `EXPLAIN ANALYZE`
the `spans_idx` scan `output_rows` (14 = inverted-index pruned; 1,000,000 = full scan).

**Measured (GreptimeDB `spans_idx` scan output_rows):**

| Query form | scan output_rows | pruned? |
| --- | --- | --- |
| plain `WHERE trace_id='X'` (no join) | **14** | ✓ index used |
| `LEFT JOIN … WHERE s.trace_id='X'` | **1,000,000** | ✗ full scan |
| `INNER JOIN … WHERE s.trace_id='X'` | **1,000,000** | ✗ full scan |
| `LEFT JOIN … WHERE s.trace_id='X' AND e.trace_id='X'` | **1,000,000** | ✗ full scan |

**Verdict.** **General join-pushdown gap (both INNER and LEFT).** GreptimeDB's inverted
index prunes a standalone anchored query (14 rows) but is **not consulted when the table is
a join input** (full-scans 1M) — the pushed `trace_id='X'` filter lands as a post-scan
`FilterExec` on the `MergeScanExec` output, not as an index-eligible scan predicate. ClickHouse
prunes the same join input (`Granules 1`, Run 30/81). **Added as parity-roadmap Improvement
#8** (push an equality filter into an indexed join input): Tier-A workaround today = subquery
pre-filter (Run 81: prunes to 14, ~21 ms) or Parallax's app-side correlation; Tier-B fix =
the optimizer reaching the region scan's index path for join-pushed filters
(`src/query/src/optimizer` + DataFusion `push_down_filter`). **Integration, not architecture**
— the index works; the pushdown plumbing into the join-input scan is the gap. Footnote-priority
for Parallax (its bundle assembly is app-side), real for in-DB-join users. Status: **gap
characterized + roadmapped.**

Caveat: 1M-row left table; the full-scan cost scales with the un-pruned table size (worse at
volume) — the workaround/fix matters more as data grows.

**Reproduce.** `EXPLAIN ANALYZE` the join forms above on GreptimeDB, read the `spans_idx`
`UnorderedScan output_rows` (1,000,000 = no pushdown; 14 = pruned).

### Run 83 — 2026-05-25 — Write-path freshness re-verified (axis #1 tie: both visible-on-write)

**Pass target.** Re-verify the top-axis (#1, ingest-to-queryable) load-bearing tie: a write
is **visible-on-write on both engines, no flush barrier** (Run 5).

**Environment.** Main stack, GreptimeDB `v1.0.2` / ClickHouse `v26.5.1.882` (re-pinned —
latest, no bump).

**Verified:** insert 1 row → immediately `SELECT count() WHERE v='marker'`:

| Engine | immediate count | mechanism |
| --- | --- | --- |
| ClickHouse | **1** (visible) | `async_insert=1` + `wait_for_async_insert=1` (live defaults) → ack blocks until the buffer flushes to a part → visible on ack (no separate merge) |
| GreptimeDB | **1** (visible) | row in the mutable memtable, visible via `committed_sequence` (no flush) |

**Verdict.** **Freshness tie reproduces — no drift.** Both make an acked write queryable
immediately, no flush/merge required. (Mechanism nuance, Run 33/56: ClickHouse's default
`async_insert=1`/`wait=1` means the ack *absorbs* the ≤200 ms buffer window — visible on ack
but the ack waits; `wait=0` would give a fast ack + a brief invisible/lossy window.
GreptimeDB's memtable is visible+durable on write with no window.) Axis-#1 freshness stays a
tie; the write-path *differences* that favour GreptimeDB are small-write absorption + native
OTLP/Prom ingest (`write-path-and-ingestion.md`), not freshness latency. Status: **confirmed.**

**Method gotcha (logged):** GreptimeDB v1.0.2 reserves **`id`** as a keyword — `CREATE TABLE …
(id …)` errors ("Cannot use keyword 'id'"); quote it (`"id"`). Joins the reserved set
(`service`/`name`/`status`/`level`/`value`/`v`-ok). Matters for hand-written DDL.

**Reproduce.** `INSERT INTO t VALUES (…)` then immediately `SELECT count() FROM t WHERE …`
on each — count=1 on both, no flush.

### Run 84 — 2026-05-25 — High-card INGEST rate: GreptimeDB cardinality-insensitive, ClickHouse ~2.6× (closes high-card)

**Pass target.** The last owed high-card piece: does extreme series cardinality slow *ingest*
more on one engine? GreptimeDB's PartitionTree/metric engine is claimed "built for high-card
ingest" — measure it.

**Environment.** GreptimeDB `v1.0.2` / ClickHouse `v26.5.1.882` (re-pinned — latest, no
bump). Ingest 1M rows at **1k** vs **1M** distinct series. CH: `INSERT SELECT numbers(1M)`
server-side (`--time`, min of 3). GreptimeDB: `COPY` the same CSV (wall + `execution_time_ms`).
The comparable metric is each engine's **own 1k→1M slowdown** (method difference cancels).

**Measured:**

| Engine | 1k-series ingest | 1M-series ingest | **cardinality slowdown** |
| --- | --- | --- | --- |
| ClickHouse (`INSERT SELECT`, server) | ~0.11 s | ~0.28 s | **~2.6×** |
| GreptimeDB (`COPY`, exec_time_ms) | 357 ms | 381 ms | **~1.07× (flat)** |

**Verdict — closes the high-card picture.** **GreptimeDB ingest is cardinality-INSENSITIVE**
(1k→1M series: 357→381 ms, ~7% — the PartitionTree memtable absorbs 1M distinct series with
negligible slowdown, no `LowCardinality`-style cap or `ORDER BY` re-tuning). **ClickHouse
ingest slows ~2.6×** at extreme cardinality (`LowCardinality` dict overflow + many more
distinct `ORDER BY` keys → more granule boundaries / dict + part management). So the
"GreptimeDB is built for high-cardinality *ingest*" claim is **confirmed with a number** —
its high-card edge is real and largest on the **ingest** axis. **Full high-card picture now:**
ingest → **GreptimeDB** (cardinality-insensitive vs CH 2.6×); storage → **crossover** (CH wins
≤200k, GreptimeDB wins ~1M, Run 79); aggregation latency → **ClickHouse** (~2–3×, Run 67);
operability (no cap) → **GreptimeDB**. Status: **high-card complete across all axes.**

Caveat: GreptimeDB COPY (wall+parse) vs CH INSERT-SELECT (server) — not cross-comparable on
absolutes; the *within-engine* 1k→1M slowdown ratio is the result (each engine's own baseline
cancels the method). 1M-row smoke; the slowdown ratios should hold/sharpen at volume.

**Reproduce.** CH `INSERT INTO t SELECT 'svc-'||toString(number%N), … FROM numbers(1000000)`
`--time` for N∈{1000, 1000000}; GreptimeDB `COPY` the dumped CSVs, compare each engine's
1M/1k time ratio.

### Run 85 — 2026-05-25 — Reserved-keyword scan: GreptimeDB rejects ~28/42 observability column names, ClickHouse 1

**Pass target.** A DDL-ergonomics gap repeatedly hit (Run 45: 7 reserved; Run 83: `id`).
Systematically scan Parallax's likely column names against each parser → the complete
quoting list for the buildable blueprint.

**Environment.** GreptimeDB `v1.0.2` / ClickHouse `v26.5.1.882` (re-pinned — latest, no
bump). `CREATE TABLE … (col STRING)` per candidate; "reserved" = parser rejects unquoted.

**Measured (42 Parallax-relevant names tested):**

| Engine | rejected unquoted | examples |
| --- | --- | --- |
| **GreptimeDB** | **~28 / 42** | `id, value, timestamp, user, name, status, level, message, service, release, url, method, count, type, source, target, date, start, end, key, index, group, order, table, version, event, action, result` |
| **ClickHouse** | **1 / 28** (same set) | only `index` |

GreptimeDB **not** reserved: `host, duration, environment, project, fingerprint, error_type,
span_id, trace_id, kind, attributes, labels, tags, time`.

**Verdict.** **A real DDL-ergonomics papercut for GreptimeDB.** Most common observability
column names — `value`, `timestamp`, `user`, `status`, `level`, `message`, `service`,
`name`, `id`, `type`, `source`, `target`, `event`, `action`, `result`, `method`, `url` — are
**reserved keywords** in GreptimeDB v1.0.2 and must be quoted (`"value"`); ClickHouse accepts
**all but `index`** unquoted. **Not a blocker** (quoting works — the whole blueprint built
live, Run 45), but Parallax's GreptimeDB DDL must **quote column identifiers defensively**,
while ClickHouse's DDL is cleaner. A small ClickHouse authoring-ergonomics edge, offsetting
GreptimeDB's *ingest* ergonomics edges (native protocols, schema-on-write, cap-free high-card).
Corrects the blueprint's incomplete "7 reserved" note → the full set. Status: **logged;
blueprint quoting rule fixed.**

Caveat: tested 42 names; GreptimeDB's full reserved list is larger (it inherits SQL-standard
+ DataFusion keywords). Rule of thumb for the blueprint: **quote every column identifier.**

**Reproduce.** `for col in value timestamp user status …; do CREATE TABLE t (ts TIMESTAMP TIME INDEX, $col STRING); done` on each — GreptimeDB errors "Cannot use keyword '$col'" on ~28; ClickHouse only on `index`.

### Run 86 — 2026-05-25 — Native OTLP traces structure verified (closes Run 57) + a partition-by-trace_id finding

**Pass target.** Close the one owed native-structure gap (Run 57: OTLP traces is
protobuf-only, couldn't verify the native trace table live). Hand-build a minimal OTLP-trace
protobuf, ingest, inspect the auto-created table.

**Environment.** GreptimeDB `v1.0.2` / ClickHouse `v26.5.1.882` (re-pinned — latest, no bump).
Built a 100-byte `ExportTraceServiceRequest` protobuf by hand (one span: trace_id/span_id/
name/kind/start/end + a `service.name` resource attr), `docker cp` + `curl -X POST
.../v1/otlp/v1/traces -H 'Content-Type: application/x-protobuf'`.

**Verified:**

- **OTLP traces is NOT zero-config — it requires a pipeline.** Bare POST → `"Pipeline is
  required for this API"`; only **`x-greptime-pipeline-name: greptime_trace_v1`** worked
  (others → `"Unsupported pipeline for trace"`). (Contrast: Influx metrics / `greptime_identity`
  logs auto-create with no pipeline, Run 57.)
- **Auto-created 3 tables:** `opentelemetry_traces` (+ `_operations`, `_services` companions —
  these back the native Jaeger query API, Run 55). The span landed (`count=1`).
- **`opentelemetry_traces` native schema:** `timestamp` TIMESTAMP(9) TIME INDEX,
  `timestamp_end`, `duration_nano`; `trace_id` / `parent_span_id` / `service_name` each a
  **BLOOM `SKIPPING INDEX`** (fpr 0.01, granularity 10240); **`PRIMARY KEY (service_name)`**;
  full OTLP fields (`span_id`, `span_kind`, `span_name`, `span_status_code`/`_message`,
  `trace_state`, `scope_name`/`_version`); **`span_events` / `span_links` as `JSON`**; and
  **`PARTITION ON COLUMNS (trace_id)`** — a **16-way partition by `trace_id` first hex char**.

**Verdict — ADOPT native for traces; + a partition-by-trace_id mechanism finding.**

- **Adopt-vs-custom (traces): ADOPT the native `opentelemetry_traces`.** It is a complete,
  well-designed OTLP trace model — `trace_id` bloom-skipping-indexed **and** the table is
  **partitioned by `trace_id`**, so an anchored `trace_id` lookup prunes to **1 of 16
  partitions** then bloom-skips within → good anchored retrieval, plus `service_name`
  PK/index for service queries and JSON events/links. Better-designed for traces than the
  hand-rolled `spans_idx` (PK `service,name` + `trace_id` inverted) — Parallax should adopt
  it (custom only to add `fingerprint`/cross-signal columns the native model lacks).
- **Refines Run 63/65 (cluster-vs-cardinality).** I'd concluded GreptimeDB "cannot cluster
  by the high-card anchor without making it the PK (→ series blowup)" and has no `order_by`.
  But `PARTITION ON COLUMNS (trace_id)` is a **distribution-level anchor mechanism**: it
  buckets traces 16-way by `trace_id` **without** `trace_id` being the PK/series identity —
  so an anchored cold read touches **~1/16 of the data**, not the whole table. It is coarse
  (16 buckets, not per-trace sort locality), so it **partially** mitigates the Run-55/63
  cold-egress scatter (16× fewer bytes, not granule-level), at **no series-cardinality cost.**
  So GreptimeDB *does* have an anchor-locality lever (partitioning) — coarser than ClickHouse
  `ORDER BY` but real and cardinality-free. Updates parity #5 / the cold-read story.

Caveat: 1-span smoke; the partition-prune cold-egress benefit (1/16) is structural (from the
DDL), not yet measured at volume on S3 — route the sized native-trace cold-read to the harness.

**Reproduce.** Build the OTLP protobuf (see the Python in this run's history), `curl -X POST
.../v1/otlp/v1/traces -H 'Content-Type: application/x-protobuf' -H 'x-greptime-pipeline-name:
greptime_trace_v1' --data-binary @trace.pb`; `SHOW CREATE TABLE opentelemetry_traces`.

### Run 87 — 2026-05-25 — PARTITION ON COLUMNS(trace_id) prunes anchored reads (cardinality-free anchor locality)

**Pass target.** Test the Run-86 hypothesis on bulk data: does partitioning by `trace_id`
actually prune an anchored query to the matching partition (mitigating the Run-63 cold
scatter), and at what cost?

**Environment.** GreptimeDB `v1.0.2` / ClickHouse `v26.5.1.882` (re-pinned — latest, no
bump). Built `spans_part` = the 1M-span data, `PRIMARY KEY(service,name)` (trace_id **not**
keyed, no trace_id index), **`PARTITION ON COLUMNS (trace_id)`** (8-way hex ranges). Loaded
via `INSERT … SELECT FROM spans_idx`. `EXPLAIN ANALYZE` an anchored `trace_id='3fb2d84c…'`.

**Measured (anchored `trace_id` scan, `EXPLAIN ANALYZE`):**

| Table | partitions touched | scan_cost | output_rows |
| --- | --- | --- | --- |
| `spans_idx` (no partition, `PK(service,name)`) — Run 63 | all (`file_ranges: 10`) | **39 ms** | 14 |
| `spans_part` (`PARTITION ON COLUMNS(trace_id)`, 8-way) | **1 matching** (`count: 2, file_ranges: 2`) | **11 ms** | 14 |

**Verdict — partition-prune is real, ~3.5× here, and cardinality-free.** Partitioning by
`trace_id` pruned the anchored scan to the **one matching partition** (~1/8 of the data,
2 file ranges) → **11 ms vs 39 ms (~3.5×)** — *without* `trace_id` being the PK and with
**no inverted index on it** (pure partition-pruning). So **GreptimeDB DOES have a
cardinality-free anchor-locality lever: `PARTITION ON COLUMNS(<anchor>)`.** This:

- **Confirms the Run-86 mitigation with a number:** a cold anchored read on a trace_id-
  partitioned table touches ~**1/N** of the SSTs (N = partition count; native traces use
  16-way → ~1/16), materially shrinking the Run-55/63 whole-SST cold egress (coarse, not
  ClickHouse's granule-level, but real).
- **Refines Run 63/65:** the earlier "GreptimeDB cannot cluster by the anchor without PK-
  cardinality blowup / has no `order_by`" stands for *sort* locality, but **partitioning is
  the cheap coarse alternative** — proven here. Combine with a `trace_id` index for
  within-partition pruning too.
- **Blueprint rule:** Parallax's GreptimeDB spans/logs/error tables should
  `PARTITION ON COLUMNS(trace_id)` (as the native `opentelemetry_traces` does) for
  anchored-read + cold-egress locality at no series-cardinality cost.

Caveat: 8-way here (native is 16-way); finer partitioning = finer prune but more regions to
manage. Warm scan_cost (the cold S3 egress reduction to ~1/N is structural, owed for the
sized number). Partition count is a fixed schema choice (not adaptive).

**Reproduce.** `CREATE TABLE spans_part (… PRIMARY KEY("service","name")) PARTITION ON
COLUMNS ("trace_id") (trace_id < '1', …)`; `INSERT … SELECT FROM spans_idx`; `EXPLAIN ANALYZE
SELECT span_id FROM spans_part WHERE trace_id='…'` → `partition_count count:2`, scan_cost
~11 ms vs spans_idx ~39 ms.

### Run 88 — 2026-05-25 — Cold S3 egress with trace_id partitioning: ~8× less (closes the cold-egress thread)

**Pass target.** Measure the *cold S3 egress* of an anchored read on a trace_id-partitioned
table (Run 87 measured warm prune; Run 55 measured the non-partitioned whole-SST 23 MiB).
Does partitioning cut cold egress to ~1/N on object storage?

**Environment.** Isolated S3 stack (MinIO + GreptimeDB(S3) `v1.0.2`). Versions re-pinned —
latest, no bump. Loaded the 1M-span set into `spans_part` = **`PARTITION ON COLUMNS(trace_id)`
16-way** (like native `opentelemetry_traces`), `PK(service,name)`, no trace_id index → 22 MiB
across **48 objects** (16 partition-regions). Cold = `rm` the local read cache + restart;
`mc admin trace` the anchored `trace_id='3fb2d84c…'` query.

**Measured (cold anchored read):**

| Table | per-query parquet GETs | parquet egress | vs ClickHouse (Run 55) |
| --- | --- | --- | --- |
| GreptimeDB non-partitioned (Run 55) | 5 | **~23 MiB** (whole SST) | ~80× CH |
| GreptimeDB **16-way trace_id-partitioned** (this run) | **3** | **~2.8 MiB** (the matching partition) | **~10× CH** |
| ClickHouse (Run 55, granule prune) | 18 | **~294 KiB** | 1× |

**Verdict — partitioning cuts cold egress ~8×; closes the thread.** The cold anchored read
fetched only the **matching partition's parquet (~2.8 MiB)**, not the whole table (~23 MiB) —
**~8× less cold S3 egress**, confirming Run 87's warm prune translates to object-store egress.
So `PARTITION ON COLUMNS(trace_id)` (which the native `opentelemetry_traces` ships 16-way)
**substantially closes the Run-55/63 cold-egress gap**: GreptimeDB cold-selective egress goes
23 MiB → 2.8 MiB, narrowing the gap to ClickHouse's granule-level 294 KiB from **~80× to
~10×** (finer partitioning — e.g. 64-way — would narrow further). ClickHouse still wins
cold-*selective* egress (granule beats partition), but by ~10×, not ~80×, and GreptimeDB's
**persistent read cache keeps the common warm path at ~0 S3** regardless. **Cold-egress
thread resolved: the partition lever (cardinality-free) is the mitigation; adopt the
16-way-partitioned native trace table + partition Parallax's custom tables.**

Caveat: 51 *total* data/ GETs include one-time **per-partition manifest reopen** (16
partitions each read their manifest on restart) — that is region-open overhead, not
per-query; the per-query cold data egress is the ~2.8 MiB parquet (3 GETs). 16-way gave ~1/8
here (uneven partition sizes), not exactly 1/16. Single-node smoke.

**Reproduce.** Load `spans_part` (16-way `PARTITION ON COLUMNS(trace_id)`) into GreptimeDB(S3);
`rm -rf /greptimedb_data/cache/*` + restart; `mc admin trace` the anchored query; sum parquet
`GetObject` sizes on `data/` (~2.8 MiB vs Run 55's ~23 MiB non-partitioned).

### Run 89 — 2026-05-25 — The COST of trace_id partitioning (completes the tradeoff)

**Pass target.** Runs 87/88 measured the *benefit* of `PARTITION ON COLUMNS(trace_id)`
(anchored prune 3.5× warm, cold egress ~8×). Measure the *cost* (regions, ingest, full-scan
fan-out) so the blueprint principle is balanced.

**Environment.** Main GreptimeDB `v1.0.2` / ClickHouse `v26.5.1.882` (re-pinned — latest, no
bump). `spans_p16` (1M spans, 16-way `PARTITION ON COLUMNS(trace_id)`) vs `spans_idx` (1
region, no partition).

**Measured:**

| | spans_p16 (16-way) | spans_idx (1 region) |
| --- | --- | --- |
| Regions | **16** | 1 |
| Ingest `INSERT…SELECT 1M` | 2040 ms | (single region, less routing overhead) |
| Warm full-scan agg `GROUP BY service` | **~17 ms** | **~12 ms** |
| Anchored `trace_id` lookup (Run 87) | **11 ms** (1 partition) | 39 ms (all) |
| Cold S3 egress, anchored (Run 88) | **~2.8 MiB** (1 partition) | ~23 MiB (whole SST) |

**Verdict — partitioning is a real tradeoff, net-positive for Parallax's anchored workload.**
**Benefit:** anchored `trace_id` reads prune to 1 partition → **~3.5× faster warm + ~8× less
cold egress** (Runs 87/88). **Cost:** **16× the regions** (each its own memtable/SST/compaction
unit + manifest), **~1.4× slower full-table aggregation** (~17 vs ~12 ms — the query fans out
to 16 partitions and merges), higher ingest routing overhead, and per-partition manifest
reopen on restart (Run 88). For Parallax the **anchored `trace_id` lookup is the dominant
query**, so the tradeoff **favours partitioning** (speed up the hot path + cut cold egress) at
the cost of slower full scans (which Parallax runs less). **Key nuance:** the ~1.4% full-scan
penalty is a **single-node fan-out artifact** — at multi-node the 16 partitions **distribute
across datanodes**, turning the fan-out into parallelism (the scaling design). So partition
for anchored locality + future distribution; don't over-partition (16-way native default is a
reasonable balance — more partitions = finer prune but more region overhead + slower
single-node scans). Status: **partition tradeoff complete; blueprint principle 8 balanced.**

Caveat: single-node smoke; the multi-node "fan-out becomes parallelism" claim is
arch-reasoned (owed to a cluster run). Ingest 2040 ms not compared to a timed non-partitioned
INSERT-SELECT baseline (the region-routing overhead is the directional point).

**Reproduce.** Build `spans_p16` (16-way) + `spans_idx`; compare `region_statistics` region
count (16 vs 1), `GROUP BY service` `execution_time_ms` (~17 vs ~12 ms), anchored lookup
(Run 87: 11 vs 39 ms).

### Run 90 — 2026-05-25 — PREWHERE applies but its benefit is conditional (not a blanket selective-scan win)

**Pass target.** Re-verify ClickHouse's PREWHERE late-materialization (Run 2) — a
CH-favourable read-path mechanism — and quantify its actual benefit (balance the recent
GreptimeDB-deep passes).

**Environment.** Main stack, GreptimeDB `v1.0.2` / ClickHouse `v26.5.1.882` (re-pinned —
latest, no bump). `logs_b1` (5M; `ORDER BY (service, ts)`, `message` wide String).

**Verified:**

- **PREWHERE applies** — `EXPLAIN actions=1 SELECT message … WHERE service='svc-0' AND
  level='ERROR'` → `Prewhere filter column: and(level, service) (removed)`. The filter is
  moved ahead of the `message` read. ✓ mechanism present at 26.5.
- **But latency ON vs OFF is a TIE, and read_bytes is IDENTICAL** (`optimize_move_to_prewhere`
  1 vs 0): on `WHERE level='ERROR'` (~600k of 5M, ~12%, evenly distributed) both read
  **301.56 MiB / 5,000,000 rows** and ran ~80 ms. **PREWHERE skipped nothing** — because
  `level='ERROR'` leaves survivors in *every* granule, so `message` is read for every granule
  regardless.

**Verdict — PREWHERE is real but its benefit is conditional; don't overstate it.** Late
materialization only helps when the filter **empties whole granules** (so the wide column's
reads are skipped for those granules) — i.e. low/clustered selectivity — and most visibly at
**cold/disk-bound** scale (skipping *disk* reads). At a 12%-evenly-distributed filter on
cache-resident smoke it is a **no-op** (read_bytes identical). The smoke-scale selective-scan
pruning is really the **sort-key granule-skip** (`service` in `ORDER BY` → only svc-0's
granules read), with PREWHERE secondary. So ClickHouse's "selective-scan edge" = granule-skip
(sort key) + PREWHERE-when-it-empties-granules + the vectorized scan over survivors; PREWHERE
alone is not a blanket win. **Refines (doesn't overstate) the CH read-path advantage.**
Status: **PREWHERE mechanism re-verified; benefit characterized as conditional.**

Caveat: warm cache-resident 5M — PREWHERE's *disk*-read-skipping benefit is a cold/at-scale
effect the smoke tier can't show; the granule-emptying condition is the other gate.

**Reproduce.** `EXPLAIN actions=1 SELECT message FROM logs_b1 WHERE level='ERROR'` (shows
Prewhere); compare `read_bytes` in `system.query_log` for `optimize_move_to_prewhere=1` vs `0`
(identical at 12% even selectivity).

### Run 91 — 2026-05-25 — Replication-economics re-verified precisely: ClickHouse zero-copy present-but-off + guard-railed

**Pass target.** Re-verify the replication-economics pillar (1× vs N× S3 copies, Run 34/74)
precisely — the exact live status of ClickHouse zero-copy replication (what decides whether
N OSS replicas store N× or 1× on S3). Full multi-replica $ measurement (B14) needs a
Keeper+2-replica setup (harness-gated); this nails the deciding mechanism.

**Environment.** Main stack, GreptimeDB `v1.0.2` / ClickHouse `v26.5.1.882` (re-pinned —
latest, no bump). `system.merge_tree_settings` zero-copy family.

**Verified (CH 26.5 zero-copy settings):**

| Setting | value | obsolete? |
| --- | --- | --- |
| `allow_remote_fs_zero_copy_replication` | **0 (off)** | no |
| `disable_detach_partition_for_zero_copy_replication` | 1 | no |
| `disable_fetch_partition_for_zero_copy_replication` | 1 | no |
| `disable_freeze_partition_for_zero_copy_replication` | 1 | no |
| `remote_fs_zero_copy_zookeeper_path` | `/clickhouse/zero_copy` | no |

**Verdict — pillar holds, precisely.** Zero-copy replication **exists** in OSS 26.5 (the
whole settings family is present, **none obsolete**) but is **off by default** and wrapped in
`disable_{detach,fetch,freeze}_partition_for_zero_copy_replication=1` guardrails (those ops
are unsafe with shared S3 data) — consistent with the source "not production-ready"/EXPERIMENTAL
flag (Run 34). So:

- **OSS ClickHouse default = N× S3 copies** — with zero-copy off, each `ReplicatedMergeTree`
  replica fetches and stores its own parts on S3 → N replicas ≈ N× storage. To get ~1× you
  must enable the experimental/guard-railed zero-copy.
- **GreptimeDB = 1× by default** — object-store-native: SSTs live in S3 once; HA is via
  metadata/leadership (Metasrv) + region reopen-from-storage, not data copies (Run 34/57).

So the cost/scaling pillar "GreptimeDB 1× shared S3 vs OSS ClickHouse N× (unless the
not-ready zero-copy is enabled)" is **re-verified at the setting level.** The exact N×
*bytes* at 2–3 replicas (B14) remains the Keeper+multi-replica harness measurement, but the
deciding switch (zero-copy off + guard-railed by default) is confirmed live. Status:
**replication-economics pillar re-verified precisely; full B14 $ owed to the harness.**

Caveat: setting-level verification (not a 2-replica byte measurement); GreptimeDB's 1×-shared
HA is cluster-mode (single-node `cluster_info` = STANDALONE here) — source/arch-established,
not single-node-testable.

**Reproduce.** `SELECT name,value,is_obsolete FROM system.merge_tree_settings WHERE name LIKE '%zero_copy%'` → `allow_remote_fs_zero_copy_replication=0`, not obsolete, + the `disable_*` guardrails.

### Run 92 — 2026-05-25 — GreptimeDB PromQL vs its own SQL re-verified (~5×, "capability not speed")

**Pass target.** Re-verify the load-bearing metrics nuance (Run 44): GreptimeDB's native
PromQL path is **~5× slower than its own SQL** at high series cardinality — so metrics→
GreptimeDB is a *capability* win (Grafana-native PromQL), not a speed one; use SQL for hot
aggregations (a Tier-A parity insight).

**Environment.** Main stack, GreptimeDB `v1.0.2` / ClickHouse `v26.5.1.882` (re-pinned —
latest, no bump). `metrics_hc` (8M rows, 40k series). Same `avg by service`.

**Measured (warm):**

| GreptimeDB path | latency |
| --- | --- |
| **SQL** `SELECT service, avg(value) … GROUP BY service` | **~104 ms** (`execution_time_ms`) |
| **PromQL** instant `avg by(service)(metrics_hc)` | **~550 ms** (curl wall ≈ server, HTTP ~negligible Run 60) |
| PromQL range (2 h / 60 s step) | ~700 ms |

**Verdict — reproduces Run 44, no drift.** GreptimeDB native PromQL (~550 ms) is **~5×
slower than GreptimeDB SQL (~104 ms)** for the identical aggregation — the
`SeriesNormalize` / per-series PromQL-planner fixed cost over 40k series (`promql-and-metrics-query.md`).
So **metrics → GreptimeDB is a *capability* win (PromQL over the standard Prometheus HTTP
API, drop-in Grafana datasource), never a *speed* win**: GreptimeDB's *own SQL* beats its
PromQL ~5×, and ClickHouse SQL beats GreptimeDB SQL ~2–3× warm (Run 67). For **hot** metric
aggregations the Tier-A move is **SQL, not PromQL**, on GreptimeDB; reserve PromQL for
Grafana compatibility / ad-hoc PromQL. Status: **confirmed; metrics verdict (capability, not
speed) holds.**

Caveat: PromQL timed by curl wall (in-container, HTTP ~negligible per Run 60) vs SQL
`execution_time_ms` — the ~5× direction is robust (Run 44 measured it server-side too). Warm
8M/40k-series smoke.

**Reproduce.** GreptimeDB SQL `SELECT service,avg(value) FROM metrics_hc GROUP BY service`
(~104 ms) vs `GET /v1/prometheus/api/v1/query?query=avg by(service)(metrics_hc)&time=…` (~550 ms).

### Run 93 — 2026-05-25 — Often-run single-user queries: recent-logs tail + metric panel refresh (fair = ~2.6×; two artifacts caught)

**Pass target.** Round out "what the single user actually feels" with two common
debugging queries not yet isolated: the **recent-logs tail** and the **metric panel
refresh** — directly relevant to the live verdict reconsideration.

**Environment.** Main stack, GreptimeDB `v1.0.2` / ClickHouse `v26.5.1.882` (re-pinned —
latest, no bump). `logs_b1` (5M), `metrics_hc` (8M / 40k series, ts span ~1h40m).

**Measured (warm):**

| Query | ClickHouse | GreptimeDB | ratio |
| --- | --- | --- | --- |
| Recent-logs tail (`service='svc-0' ORDER BY ts DESC LIMIT 100`) | ~3 ms | ~15 ms | ~5× (both interactive) |
| Metric panel refresh — **literal** time bound (Grafana-realistic), `ts >= '…03:19:30'` (4.84M rows, 40 services) | **~36 ms** | **~93 ms** | **~2.6×** |
| Metric panel refresh — `max(ts) - INTERVAL` **subquery** form | ~52 ms | **~1100 ms** | ~21× ⚠ artifact |

**Two artifacts caught (honesty):**

1. A first attempt showed "GreptimeDB ~0–2 ms" — a **0-row glitch**: `WHERE ts >= (SELECT
   max(ts) FROM metrics_hc) - 3600000` is invalid on a GreptimeDB `TIMESTAMP` (integer
   subtraction, not `INTERVAL`) → empty result, ~0 ms of nothing. **Not a win.**
2. The corrected **subquery** form (`… - INTERVAL '1 hour'`) ran in **~1100 ms** on
   GreptimeDB vs ~52 ms on ClickHouse (~21×) — but that is a **query-shape artifact**: the
   uncorrelated `max(ts)` subquery is **not folded/pushed** by GreptimeDB's optimizer
   (same family as the join-pushdown gap, Run 81), so it pays ~12× over the literal form;
   ClickHouse folds it. Grafana sends **literal** time bounds, so the **fair** number is the
   literal form.

**Verdict — no surprise; metrics story holds.** Fairly measured (literal bound, how
dashboards actually query), the metric panel refresh is **CH ~36 ms vs GreptimeDB ~93 ms =
~2.6×** — the same ~2–3× metric-aggregation gap (Run 67). So **ClickHouse is faster on the
common dashboard metric query too**; there is **no GreptimeDB metric-speed win** (reinforces
"metrics→GreptimeDB = capability/PromQL-native, not speed"). Recent-logs tail is ~5× but both
sub-20 ms (interactive). **New GreptimeDB gotcha for the blueprint:** use **literal /
app-computed time bounds**, not a `max(ts)` subquery, in metric-panel queries — GreptimeDB
doesn't fold the subquery (~12× penalty). Status: **two artifacts corrected; fair metric-panel
= ~2.6× CH; verdict unchanged.**

Caveat: warm cache-resident smoke; ~60%-of-data window so neither pruned dramatically. The
subquery-fold gap is a GreptimeDB optimizer wrinkle (cf. Run 81 join-pushdown).

**Reproduce.** Metric panel: `… WHERE ts >= '<literal>' GROUP BY service` on both (CH ~36 ms /
GreptimeDB ~93 ms); the `… >= (SELECT max(ts) …) - INTERVAL '1 hour'` subquery form is ~1100 ms
on GreptimeDB (artifact — use a literal).

### Run 94 — 2026-05-25 — Anchored-lookup scale attempt (confounded by dedup) → re-confirms cold-read scatter, warm tie stable

**Pass target.** Stress the load-bearing "anchored hot path tie" toward scale — does
GreptimeDB's anchored `trace_id` lookup stay fast as the table grows (the operator's
scaling concern)? Build 5M-row spans on both, measure.

**Environment.** GreptimeDB `v1.0.2` / ClickHouse `v26.5.1.882` (re-pinned — latest, no bump).
CH `spans_5m` = `spans` ×5 (`ORDER BY (trace_id,ts)`, `OPTIMIZE FINAL`). GreptimeDB
`spans_5m` = `spans_idx` ×5 INSERT-SELECT (`trace_id` INVERTED, `PK(service,name)`).

**What actually happened (two confounds, logged honestly):**

1. **GreptimeDB deduped to 1M, not 5M.** `PK(service,name)` + same `(service,name,ts)` keys
   across the 5 identical inserts → dedup collapsed to **1M logical rows** (`region_rows:
   1000000`). So this did **not** test 5M-*distinct* scale. *(To test it needs unique
   trace_ids/keys per copy — owed.)*
2. **The "GreptimeDB ~1000 ms anchored lookup" was a COLD-cache warming curve, not a scale or
   compaction finding.** The 7 reps fell monotonically **1890 → 1694 → 1413 → 1084 → 1032 →
   682 → 668 ms**, then after a few seconds settled to **~12 ms** (10–14). SST count was **1**
   throughout (not many-SST merge). So the first reads were **cold** — reading the
   un-partitioned 1M-row SST into the local cache (the scatter effect, Run 63: `trace_id`
   not the sort key → cold read touches ~the whole SST) — warming to ~12 ms. ClickHouse
   `spans_5m` (5M) stayed **~3 ms** even fresh (granule/sort-key, OS page cache).

**Verdict — re-confirms the cold/warm divergence on the anchored path; warm tie stable.**

- **Warm anchored lookup at 1M is ~12 ms on GreptimeDB** (matches `spans_idx` ~15 ms, no
  drift); ClickHouse ~3 ms. The warm tie holds.
- **Cold un-partitioned GreptimeDB anchored read is slow (~1000 ms first read, warming to
  ~12 ms)** — the Run-55/63 scatter again: an un-partitioned table's cold anchored read pulls
  ~the whole SST into cache. ClickHouse cold anchored is ~3 ms (granular). **Decision-relevant:
  if Parallax re-reads *cold/evicted* bundles on an un-partitioned GreptimeDB table, the first
  read pays a big warming cost** — another reason the **`PARTITION ON COLUMNS(trace_id)`**
  design (principle 8, Run 87/88: cold prune to ~1/16) matters; warm (the common recent-bundle
  case, persistent cache) is ~12 ms.
- CH anchored lookup **scales flat** (3 ms at 1M and 5M — sort-key seek is table-size-independent).

**Owed:** a clean 5M-*distinct*-trace scale test (unique keys, partitioned + warm + cold) to
confirm the anchored lookup stays flat at scale — mechanically expected (index/partition
prune is ~table-size-independent), but unverified at distinct scale; harness-tier.

**Reproduce.** Build the 5M tables; note GreptimeDB `PK(service,name)` dedups identical
re-inserts (use unique keys for a true scale test); the first GreptimeDB reads on a fresh
un-partitioned table are cold (~1000 ms warming → ~12 ms).

### Run 95 — 2026-05-25 — Clean 5M-DISTINCT partitioned scale test → anchored tie HOLDS at scale (the Run-94 "owed" closed)

**Pass target.** Close Run 94's owed item: a *clean* 5M-**distinct**-trace scale test (unique
trace_ids, no dedup confound, partitioned + warm) to confirm GreptimeDB's anchored `trace_id`
lookup stays interactive as the table grows — the operator's core scaling concern.

**Environment.** GreptimeDB `v1.0.2` / ClickHouse `v26.5.1.882` (re-pinned — latest stable, no
bump). 5M rows / **~360k unique MD5-hex `trace_id`s** on both (unique keys → no GreptimeDB
dedup, the Run-94 confound removed).

**DDL (the two confounds from Run 94 both fixed):**

- ClickHouse `spans_big`: `ORDER BY (trace_id, ts)`, 5M rows of distinct MD5 trace_ids.
- GreptimeDB `spans_big`: **`append_mode='true'`** (no read-time dedup → all 5M rows kept) +
  **`PARTITION ON COLUMNS("trace_id")`** 8-way on hex prefix. Clause order matters
  (cols → `PARTITION ON COLUMNS` → `ENGINE=mito` → `WITH`), else `SQL statement is not supported`:

  ```sql
  CREATE TABLE spans_big (
    "ts_ms" TIMESTAMP(3) TIME INDEX, "trace_id" STRING INVERTED INDEX, "span_id" STRING,
    "service" STRING, "name" STRING, "duration_ms" DOUBLE, "status" STRING,
    PRIMARY KEY("service","name")
  ) PARTITION ON COLUMNS ("trace_id") (
    trace_id < '2', trace_id >= '2' AND trace_id < '4', trace_id >= '4' AND trace_id < '6',
    trace_id >= '6' AND trace_id < '8', trace_id >= '8' AND trace_id < 'a',
    trace_id >= 'a' AND trace_id < 'c', trace_id >= 'c' AND trace_id < 'e', trace_id >= 'e'
  ) ENGINE=mito WITH (append_mode='true');
  ```

**Anchored lookup `WHERE trace_id='00003e3b9e5336685200ae85d21b4f5e'` (10 warm reps, ms):**

| Engine | 5M-distinct | vs its own 1M (Run 94) |
| --- | --- | --- |
| ClickHouse `spans_big` | **~3 ms flat** | ~3 ms — flat (sort-key seek is table-size-independent) |
| GreptimeDB `spans_big` (partitioned, append) | `9 7 7 10 7 6 7 6 6 11` → **~7 ms warm** | ~12 ms → **~7 ms, i.e. flat-to-faster** |

**Verdict — the load-bearing "anchored tie holds at scale" claim is now CONFIRMED at 5M-distinct.**

- **GreptimeDB anchored lookup stays interactive at 5M distinct (~7 ms warm), and is actually
  *flatter/faster* than the 1M un-partitioned ~12 ms** — because `PARTITION ON COLUMNS(trace_id)`
  prunes to ~1/8 of the data *and* the inverted index seeks within the partition. Partitioning
  helps both the cold-egress axis (Run 88) **and** warm anchored latency at scale.
- **ClickHouse stays ~3 ms flat.** Both engines are single-digit-ms on the anchored hot path at
  5M-distinct — **both ≪ the 300 ms gate**, so the tie (CH ~2× faster, both sub-perceptible)
  **does not widen with scale**. This is mechanically expected (index/partition prune is
  ~table-size-independent) and now empirically held at 5M-distinct, not just smoke.
- Closes the Run-94 confounds: unique trace_ids (no PK dedup) + `append_mode` (all 5M kept) +
  warm reps (the ~1000 ms was cold-warming, not scale). Cold-at-scale + multi-node remain
  harness-gated (open Q#1/#4).

**Reproduce.** Generate 5M rows with unique MD5 `trace_id`s; load both; build GreptimeDB with
`append_mode='true'` + `PARTITION ON COLUMNS("trace_id")` (clause order above); run the anchored
lookup warm ×10. Cleanup: `DROP TABLE spans_big` on both, `rm /tmp/sbig.csv /tmp/trbig.txt`.

### Run 96 — 2026-05-25 — Metric dashboard panels re-verified: agg-gap holds (~2–3× warm) AND is query-shape-dependent (gap narrows as per-row compute grows)

**Pass target.** Re-verify the load-bearing **metric-aggregation warm gap** (~2–3×, Run 37/67)
on the often-run *single-user dashboard panel* — does it still reproduce on the current
containers, and how does it behave on the realistic time-bucketed line-chart shape (not just
the flat group-by)?

**Environment.** GreptimeDB `v1.0.2` / ClickHouse `v26.5.1.882` (re-pinned live — latest stable,
no bump). `metrics_hc` = **8,000,000 rows / 40 services / ~40k series** (service×instance), both
engines, identical data (parity re-confirmed: same count + service count). `value` Float64 (CH
`Gorilla(8),ZSTD`; GT auto Parquet). Method: CH server elapsed via `clickhouse-client --time`
(in-container loopback ~0); GT `execution_time_ms` (server-side) — both engine-time, transport
excluded; warm (containers up 11 h, query repeated).

**Panel A — flat "avg by service" (Run 67 shape, 40 groups), warm reps (ms):**

| Engine | reps | warm median | vs Run 67 |
| --- | --- | --- | --- |
| ClickHouse | `58 43 32 35 39 33 34 39 38 38` | **~38 ms** | matches (Run 67 CH 32) |
| GreptimeDB | `131 111 101 104 118 185 162 135 111 114` | **~116 ms** | matches (Run 67 GT 99) |
| **Ratio** | | **~3.0×** | **reproduces ~2–3× warm** |

**Panel B — time-bucketed line chart `avg per 1-min bucket × service` (4,000 groups; CH
`toStartOfMinute`, GT `date_bin('1 minute'::INTERVAL, ts)`), warm reps (ms):**

| Engine | reps | warm median |
| --- | --- | --- |
| ClickHouse | `64 70 62 67 59 54 69 59` | **~63 ms** |
| GreptimeDB | `127 128 127 125 154 114 120 111` | **~126 ms** |
| **Ratio** | | **~2.0×** |

**Verdict — agg-gap reproduces, and the new finding is that it is query-shape-dependent.**

- **The ~2–3× warm metric-agg gap holds** on current containers (Panel A ~3.0× = Run 67 exactly).
  The load-bearing "GreptimeDB is not faster, even on metrics" claim **re-verified, no drift**.
- **New nuance: the gap NARROWS from ~3× (flat) to ~2× (bucketed) as per-row compute grows.**
  Mechanism (consistent with `query-execution-engine.md`): ClickHouse's edge is its vectorized
  **scan + hash-agg throughput** (65k-row blocks + JIT + SIMD). Panel A's hash table is tiny (40
  groups, L1-resident) so the bottleneck is *pure scan throughput* → ClickHouse's strength
  dominates (~3×). Panel B adds a `date_bin`/`toStartOfMinute` scalar per row **and** a 100×
  bigger hash table (4,000 groups); that added work is more *comparable* across engines (both pay
  the bucket compute), diluting ClickHouse's scan-throughput edge to ~2×. So the often-quoted
  "~2–3×" is real but it is the *ceiling* for scan-bound aggregation; compute-heavier panels
  close toward ~2×.
- **Both panels are sub-300 ms warm on GreptimeDB** (116 / 126 ms) — **interactive on either
  engine.** This reaffirms "fit not speed": even the heaviest common dashboard refresh is well
  inside the interactive gate on GreptimeDB, so the agg-gap costs nothing perceptible on the
  single-user panel; it would matter only for very large ad-hoc aggregations (DQ5 flip trigger).
- **Adopt-native (metrics):** unchanged — these used a plain mito table; the native **metric
  engine** (`ENGINE=metric`) runs the *same* DataFusion agg path, so the panel latencies apply to
  the ADOPT-native design too (the `__tsid` layout is an ingest/cardinality win, not an agg-speed
  one — Run 84). Decision stands: **ADOPT native metric engine** for Parallax metrics.

**Reproduce.** On `metrics_hc` (8M/40-svc): Panel A `SELECT service, avg(value) FROM metrics_hc
GROUP BY service`; Panel B CH `SELECT toStartOfMinute(ts) m, service, avg(value) ... GROUP BY
m,service`, GT `date_bin('1 minute'::INTERVAL, ts)`. Warm ×8–10; CH `--time`, GT
`execution_time_ms`. Expect CH ~38/63 ms, GT ~116/126 ms (~3×/~2×).

### Run 97 — 2026-05-25 — Trace-waterfall hot path re-verified: flat fetch interactive both, in-DB recursive span-tree still GT-broken (Run 68 reproduces, no drift)

**Pass target.** Rotate to the **traces** slice: re-verify (a) the trace-view hot path (flat
anchored span fetch for one `trace_id`, the "open a trace waterfall" query) and (b) the Run-68
load-bearing claim that GreptimeDB v1.0.2 errors on the table-self-join recursive CTE used for
in-DB span-tree building. Confirm the adopt-native-traces decision still holds.

**Environment.** GreptimeDB `v1.0.2` / ClickHouse `v26.5.1.882` (re-pinned live — latest stable,
no bump). `spans` (CH) / `spans_idx` (GT): **1,000,000 rows / 71,429 traces** both (parity
re-confirmed), identical schema incl. `parent_span_id`; `trace_id` INVERTED on GT, `ORDER BY
(trace_id,ts)` on CH; ~14 spans/trace. Method: CH `--time`, GT `execution_time_ms`, warm.

**(a) Trace-waterfall flat fetch** `SELECT span_id,parent_span_id,service,name,ts,duration_ms,
status WHERE trace_id='f6a4…' ORDER BY ts` (14 spans), warm reps (ms):

| Engine | reps | warm median |
| --- | --- | --- |
| ClickHouse | `5 2 3 3 3 3 3 3 3 3` | **~3 ms** |
| GreptimeDB | `36 23 21 24 20 18 15 19 16 23` | **~18–20 ms** |

→ **Both ≪ the 300 ms gate** — the trace view opens instantly on either engine. Anchored-path
"not latency-bound" tie **holds for the trace-waterfall shape** (CH ~6× faster in ratio,
sub-perceptible in absolute; GT un-partitioned 1M ~18 ms matches Run 94's warm anchored figure).

**(b) In-DB recursive span-tree (`WITH RECURSIVE`) — Run 68 reproduces exactly, no drift:**

| Form | ClickHouse | GreptimeDB v1.0.2 |
| --- | --- | --- |
| **Pure recursive** (counter `SELECT n+1 FROM c WHERE n<5`) | works | **works** (`count=5, max=5`) |
| **Table-self-join recursive** (span tree: recursive term JOINs `spans_idx` to the CTE) | works (`count=1, depth=0`*) | **ERRORS — `Schema error: project index 1 out of bounds, max field 1`** |

*\*The synthetic data's `parent_span_id` values don't chain to in-trace `span_id`s, so the tree
is effectively flat (1 root, no matched children) — irrelevant to the support question.*

- **GreptimeDB supports basic `WITH RECURSIVE` but still fails the table-self-join form** that a
  span tree needs — same DataFusion recursive-CTE projection limitation as Run 68, reproduced on
  current containers. **No drift; the claim holds.**
- **Also re-confirmed (Run 68 detail):** GreptimeDB loads the root's empty `parent_span_id` as
  **NULL, not `''`** — a base case of `parent_span_id=''` matched **0** rows on GT (vs CH's 1),
  so a portable span-tree base case must test `parent_span_id IS NULL OR parent_span_id=''`.

**Verdict — practical impact LOW; adopt-native-traces stands.** The dominant trace pattern is the
**flat anchored fetch + app-side tree build** (exactly what Jaeger/Tempo do client-side) — which
is interactive on both engines (~3 ms / ~18 ms). The in-DB recursive walk is the *non-dominant*
path, and it is the only place ClickHouse strictly wins on traces; Parallax does not need it. The
native `opentelemetry_traces` table (Run 86) carries `trace_id`/`span_id`/`parent_span_id`, so the
app-side waterfall build works on the native schema → **ADOPT native traces** unchanged. Only
caveat to carry into the blueprint: don't attempt in-DB recursive span-tree assembly on GreptimeDB
(use the flat fetch), and handle the NULL root marker.

**Reproduce.** Pick a trace_id (`SELECT trace_id FROM spans GROUP BY trace_id ORDER BY count()
DESC LIMIT 1`); flat fetch warm ×10 (CH `--time`, GT `execution_time_ms`). Recursive: pure counter
CTE (works both); table-self-join `WITH RECURSIVE tree AS (SELECT … FROM spans_idx WHERE
span_id='<root>' UNION ALL SELECT s.… FROM spans_idx s JOIN tree t ON s.parent_span_id=t.span_id)`
→ GT `Schema error: project index out of bounds`. Note GT root `parent_span_id` is NULL.

### Run 98 — 2026-05-25 — Full-text log search re-verified end-to-end: selective ~3× (competitive), broad-term ~12×, the `matches()`-on-bloom artifact still 155 ms (Runs 48–49 reproduce, no drift)

**Pass target.** Rotate to the **logs** slice and re-verify the most-corrected load-bearing claim:
the ~18× ClickHouse full-text advantage was a backend/function artifact (Runs 48–49), and with the
correct pairing GreptimeDB selective search is competitive. Confirm all three legs still reproduce
on the current containers, and re-confirm the native-logs adopt decision.

**Environment.** GreptimeDB `v1.0.2` / ClickHouse `v26.5.1.882` (re-pinned live — latest stable, no
bump). `logs_b1` = **5,000,000 rows** both (parity re-confirmed). Index family **matched**: CH
`INDEX … TYPE text(tokenizer='splitByNonAlpha')`; GT `FULLTEXT INDEX WITH(backend='bloom',
analyzer='English', false_positive_rate='0.01')`. Selective token = `6628797f` (a `conn=` id
matching **1 row**); broad token = `timeout` (**698,955 rows**). Method: CH `--time`, GT
`execution_time_ms`, warm.

| Leg | Query | ClickHouse | GreptimeDB | Ratio |
| --- | --- | --- | --- | --- |
| **Selective, correct pairing** | CH `hasToken`; GT **`matches_term`** (bloom) | `3 3 3 3 4 3 3 4 3 4` → **~3 ms** | `25 12 10 8 8 9 10 16 10 12` → **~10 ms** | **~3×** (both sub-perceptible) |
| **Selective, WRONG pairing** | GT **`matches()`** on a bloom index | — | `156 160 155 164 149` → **~155 ms (full scan)** | the artifact |
| **Broad term (699k matches)** | CH `hasToken`; GT `matches_term` | `8 7 7 6 7` → **~7 ms** | `91 96 89 82 85` → **~88 ms** | **~12×** (scan engine) |

**Verdict — Runs 48–49 reproduce exactly; no drift. The full-text record holds:**

- **Selective exact-term search is competitive (~3×, both ≪ perceptible):** CH ~3 ms / GT ~10 ms
  with the **correct** GT pairing (`matches_term` on a bloom index → prunes to the 1 matching row).
  This is the everyday incident-grep case (find a request-id/conn-id) — **not** an 18× gap.
- **The ~18× was 100 % a pairing artifact, re-proven:** `matches()` (tantivy query-syntax fn) on a
  `backend='bloom'` index **does not push to the index → 5M full scan ~155 ms**, flat regardless of
  selectivity. Same root cause as Run 48. So the historical "18×" is a misconfiguration, not an
  engine/index-maturity gap.
- **The real residual is broad-term analytics (~12×):** a term matching 699k rows is scan-bound
  (CH ~7 ms / GT ~88 ms) — this is the genuine ClickHouse lead, and it routes to the scan-engine
  gap (parity-roadmap #2), not full-text. Bites only if Parallax runs frequent *broad* log scans
  (not selective grep).
- **Adopt-native (logs):** unchanged. The native logs path (greptime identity pipeline) creates an
  all-STRING append table with **no message index**; `logs_b1`'s shape here (append + `FULLTEXT
  WITH(backend='bloom')` on `message` + `service`/`level` PK) **is** the ADOPT-then-add-index
  blueprint. Decision stands: **ADOPT native logs structure, ADD a `message` fulltext index** —
  bloom backend + `matches_term` for exact-term grep (the cheap, ~10 ms path), or tantivy backend +
  `matches` for query-syntax/phrase (Run 49 ~6 ms). Carry the pairing rule into the blueprint so a
  bloom index is never queried through `matches()`.

**Reproduce.** On `logs_b1` (5M): pick a selective token via `extract(message,'conn=([0-9a-f]+)')`;
CH `SELECT count() WHERE hasToken(message,'<tok>')` vs GT `… WHERE matches_term(message,'<tok>')`
(expect ~3 / ~10 ms). GT `matches(message,'<tok>')` on the bloom index → ~155 ms full scan (the
artifact). Broad: `'timeout'` → CH ~7 ms / GT ~88 ms (~12×). Warm ×5–10.

### Run 99 — 2026-05-25 — THE load-bearing anchor re-verified: Q6 evidence-bundle composite still not latency-bound on either (CH ~5 ms / GT ~16 ms, both ≪ 300 ms; faster than Run 16, no drift)

**Pass target.** Re-verify the single most load-bearing claim of the whole verdict — the one the
entire "**fit, not speed**" thesis rests on: Parallax's *dominant* query, the **anchored
evidence-bundle assembly** (fetch every signal for one `trace_id`), is **not latency-bound on
either engine** (Run 16: CH ~10 ms / GT ~33 ms, both ≪ the 300 ms interactive gate). If this
ever stops reproducing, the verdict's foundation weakens.

**Environment.** GreptimeDB `v1.0.2` / ClickHouse `v26.5.1.882` (re-pinned live — latest stable,
no bump). Signal tables, parity re-confirmed: `spans`/`spans_idx` 1M (trace_id INVERTED on GT,
`ORDER BY(trace_id,ts)` on CH), `logs` 214,287, `error_events` 2,226. Bundle for one trace_id
present in all three signals (`b1a36ee6…`) = **18 rows** (14 spans + errors + logs). Method: CH
`--time`, GT `execution_time_ms`, warm.

**Q6 composite** — the normalized 3-signal bundle, one query (app would fan these out in parallel;
the UNION is the conservative single-round-trip sum):

```sql
SELECT ts,'span'  k, name    d FROM spans      WHERE trace_id='b1a36ee6…'
UNION ALL SELECT ts,'error' k, message d FROM error_events WHERE trace_id='b1a36ee6…'
UNION ALL SELECT ts,'log'   k, message d FROM logs         WHERE trace_id='b1a36ee6…'
ORDER BY ts
```

| Engine | warm reps (ms) | warm median | vs Run 16 |
| --- | --- | --- | --- |
| ClickHouse | `7 5 5 4 5 4 5 5 5 4` | **~5 ms** | faster (Run 16 ~10 ms) |
| GreptimeDB | `94 16 16 17 18 13 19 18 13 12` | **~16 ms** | faster (Run 16 ~33 ms) |
| **Ratio** | | **~3×** | tie holds, both ≪ 300 ms |

**Verdict — the load-bearing anchor reproduces; no drift, and the absolute numbers are better.**

- **Q6 evidence-bundle assembly is NOT latency-bound on either engine** — CH ~5 ms / GT ~16 ms,
  both **≪ the 300 ms interactive gate**. The whole-bundle round trip is sub-perceptible on
  GreptimeDB. This is the query the entire product is built on, and the "fit, not speed" thesis
  stands: ClickHouse's ~3× engine edge buys **nothing perceptible** on the dominant retrieval.
- **Both faster than Run 16** (CH 10→5, GT 33→16) — warmer containers (12 h uptime, OS page cache
  hot) + the GT first-rep ~94 ms cold artifact warming to ~16 ms (the now-familiar cold/warm
  divergence, not a regression). The ~3× ratio is unchanged.
- **GreptimeDB pruned spans via the `trace_id` INVERTED index** (14 of 1M); `logs`/`error_events`
  are small enough that even an un-indexed `trace_id` scan is cheap here. **At GB-scale logs the
  blueprint's `trace_id` index on logs matters** — already in the adopt-native-logs design (Run 98:
  ADOPT structure + ADD trace_id/message index). Carry it.
- **Adopt-native:** the bundle spans all three native signal tables (metrics/logs/traces each carry
  `trace_id`), assembled **app-side** — works on the native schemas (Runs 86/98). ADOPT stands.

**Reproduce.** Find a trace_id in all three signals (`SELECT trace_id FROM spans WHERE trace_id IN
(SELECT trace_id FROM logs) AND trace_id IN (SELECT trace_id FROM error_events) LIMIT 1`); run the
UNION-ALL composite above (GT uses `spans_idx`), warm ×10. Expect CH ~5 ms / GT ~16 ms, both ≪ 300.

### Run 100 — 2026-05-25 — Storage/compression re-verified across all four signal tables: no blanket winner (pattern-dependent), high-card-metric crossover + Gorilla-codec win both reproduce

**Pass target.** Rotate to the **storage/compression cost** slice (the object-store-economics
pillar) and re-verify the load-bearing "no blanket compression winner — per-column-pattern"
finding (`compression-and-cost.md`, Runs 4/10/79) on the *current* live tables, with real
on-disk sizes rather than synthetic generators.

**Environment.** GreptimeDB `v1.0.2` / ClickHouse `v26.5.1.882` (re-pinned live — latest stable,
no bump). Sizes: CH `system.parts.data_compressed_bytes` (active); GT
`information_schema.region_statistics.disk_size`/`sst_size`. Same row counts both sides (parity).

| Table (rows) | ClickHouse compressed | GreptimeDB disk (SST) | Winner | Mechanism |
| --- | --- | --- | --- | --- |
| `metrics_hc` (8M, **high-card** 40k series, Float64 value) | 57.25 MiB | **38.62 MiB** | **GT 1.48×** | high-card crossover (Run 79): CH `LowCardinality(instance)` + sort overhead bloats at 40k series; GT columnar Parquet + dict compresses the label columns better |
| `metrics_real` (864k, **low-card** 12-svc, gauge+counter) | **1.01 MiB** (21.3× ratio) | 1.89 MiB | **CH 1.87×** | codec win (Run 4 / parity #7): CH `gauge→Gorilla` + `counter→DoubleDelta` crush the floats; GT user columns default to `PLAIN`+ZSTD — **no Gorilla-class float encoding** |
| `logs_b1` (5M, structured `message` + fulltext index) | **228 MiB** | 258 MiB (SST 240) | **CH 1.13×** (~wash) | both ZSTD the text + carry a fulltext index; near-tie, CH marginally ahead on this structured-message data |
| `spans` (1M, high-card id strings) | **27.93 MiB** | 42.86 MiB (SST 37.31 + ~5.5 inverted index) | **CH 1.34× raw / 1.53× w/ index** | CH ZSTD on `trace_id`/`span_id` + sort-key locality; GT also stores the `trace_id` INVERTED index (a read-speed cost, Run 99 anchor) |

**Verdict — the "no blanket winner, pattern-dependent" headline HOLDS; two sub-claims re-confirmed:**

- **High-card metric storage → GreptimeDB wins (1.48×), re-confirming the Run-79 crossover** — now
  visible on a *real* 8M-row table, not just the synthetic cardinality sweep. At 40k series CH's
  `LowCardinality` + ordering overhead exceeds GT's columnar Parquet, even with CH's Gorilla on the
  value column. Strengthens the high-card pillar.
- **Low-card metric storage → ClickHouse wins (1.87×) via codecs** — `Gorilla`/`DoubleDelta` crush
  the 12-service gauge/counter table to 1.01 MiB (21× ratio) vs GT's 1.89 MiB. **Re-confirms
  parity-roadmap #7**: GT defaults user columns to `PLAIN`+ZSTD, missing the Gorilla-class float
  encoding — a real, measured CH storage win on codec-friendly metrics.
- **Logs ≈ wash** (1.13×, CH marginally ahead here); **spans → CH** (high-card id strings compress
  better under CH ZSTD+sort locality; GT additionally carries the inverted index it needs for the
  anchored hot path).
- **Cost-pillar caveat unchanged:** raw bytes is **not** the cost driver — object-store *request
  economics* + fewer objects under active ingest + cheap tiering dominate (`compression-and-cost.md`).
  Even where CH is ~1.5× smaller, GT's object-store-native model is the cost lever, and a 1–2×
  local-byte delta is second-order. **No verdict change.**

**Reproduce.** CH: `SELECT table, formatReadableSize(sum(data_compressed_bytes)) FROM system.parts
WHERE active GROUP BY table`. GT: `SELECT t.table_name, r.disk_size, r.sst_size FROM
information_schema.region_statistics r JOIN information_schema.tables t ON r.table_id=t.table_id`.
Compare per table; expect GT-win on `metrics_hc`, CH-win on `metrics_real`/`spans`, wash on `logs_b1`.

### Run 101 — 2026-05-25 — Ingest cardinality-insensitivity re-verified: GreptimeDB degrades 1.16× (flat) vs ClickHouse 1.53× (String) / 2.6× (LowCardinality) as series go 12→1M

**Pass target.** Rotate to the **ingest** slice (under-covered this session) and re-verify the
load-bearing GreptimeDB write-path pillar: **cardinality-insensitive ingest** (Run 84 — GT ~flat,
CH slows as series cardinality grows). Hold this GT *win* to the same evidentiary bar as the
ClickHouse wins.

**Environment.** GreptimeDB `v1.0.2` / ClickHouse `v26.5.1.882` (re-pinned live — latest stable,
no bump). Method: `INSERT … SELECT` from the existing 1M-row `spans`/`spans_idx` as the row source
(no CSV transport, fully reproducible, identical rows both sides) into a 3-col table
`(ts, series, value)`. **Low card** = `series := service` (12 distinct); **high card** = `series :=
span_id` (1,000,000 distinct). Same 1M rows both cases; CH `ORDER BY (series, ts)`, GT
`PRIMARY KEY(series)` + `append_mode='true'` (no dedup — row counts verified 1M on all four loads).

| Engine | low-card (12 series) | high-card (1M series) | **cardinality slowdown** |
| --- | --- | --- | --- |
| ClickHouse (plain `String` series) | 233 ms | 356 ms | **1.53×** |
| GreptimeDB (`PK(series)`, append) | 588 ms | 683 ms | **1.16× (≈ flat)** |

**Verdict — cardinality-insensitivity reproduces: GreptimeDB degrades far less with series count.**

- **The load-bearing GT pillar holds:** going from 12 → 1,000,000 distinct series, GreptimeDB ingest
  slows only **1.16×** (near-flat — the metric-engine `__tsid`/PartitionTree memtable dict-encodes
  label sets with no per-series cap), while ClickHouse slows **1.53×** even with a plain `String`
  key, and **~2.6× with the idiomatic `LowCardinality` label** (Run 84 — the dict overflows past
  8,192 distinct, then degrades). GreptimeDB has **no such knob to get wrong** — it is insensitive
  to cardinality either way. Re-confirms Run 84's central claim.
- **Honest caveats (no cheerleading):**
  1. **This is `INSERT … SELECT` (read+write), not the native ingest path** — so the *absolute*
     numbers (GT 588–683 ms vs CH 233–356 ms) favour ClickHouse here and are **not** a throughput
     verdict. GreptimeDB's optimized path is native OTLP/gRPC/InfluxDB bulk (Run 5/53: >1M rows/s);
     this run measures the **cardinality-sensitivity ratio**, which is the actual claim, and there
     GT wins (flatter).
  2. **ClickHouse's penalty is schema-dependent** — `String` 1.53× vs `LowCardinality` 2.6×. A real
     observability deployment uses `LowCardinality` for labels (so it hits the worse 2.6×); the
     operator must size label cardinality up front. GreptimeDB removes that design burden.
- **Adopt-native (metrics):** unchanged — the native **metric engine** is exactly the
  cardinality-insensitive ingest path this pillar rests on (`__tsid` label-set hash over a shared
  wide table, no per-series `ORDER BY` tuning). ADOPT stands; the ingest-ergonomics edge is real.

**Reproduce.** `INSERT INTO ing_lc SELECT ts, service, duration_ms FROM spans` (low) and `… span_id
…` (high) on each engine (GT `append_mode='true'`, quote identifiers); time via CH `--time` / GT
`execution_time_ms`. Expect GT ~1.16× low→high, CH ~1.53× (String) / ~2.6× (LowCardinality). Drop
`ing_lc`/`ing_hc` after.

### Run 102 — 2026-05-25 — Unindexed/ad-hoc scan gap re-verified warm: ~2–5× shape-dependent (NOT Run 31's ~10×, which was cold), all ≪ 300 ms at 1M

**Pass target.** Re-verify the **unindexed-scan gap** (Q5, Run 31) — the scan-engine difference
that underlies the DQ5 flip trigger ("if Parallax's mix is ad-hoc-scan-dominated, ClickHouse's
read-path edge becomes central"). Run 31 reported ~10× but Run 40 flagged that figure as
cold/HTTP-wall inflated; re-measure warm and refine the multiplier honestly.

**Environment.** GreptimeDB `v1.0.2` / ClickHouse `v26.5.1.882` (re-pinned live — latest stable,
no bump). `spans`/`spans_idx` = 1M rows, parity. `span_id` is **unindexed on both** (CH `ORDER BY
(trace_id,ts)`; GT only `trace_id` INVERTED) → these are true full-scans. Method: CH `--time`, GT
`execution_time_ms`, warm.

| Scan shape | ClickHouse | GreptimeDB | Ratio |
| --- | --- | --- | --- |
| **Point filter** `count() WHERE span_id='X'` (selective, scan-bound) | `9 3 2 3 4 4 2 2 2 3` → **~3 ms** | `105 20 20 19 14 17 15 17 13 13` → **~15 ms** | **~5×** |
| **Full scan + group** `status, count() GROUP BY status` | `7 5 6 5 6 6 4 6` → **~5.5 ms** | `19 14 12 14 11 13 13 10` → **~13 ms** | **~2.3×** |
| **Full scan + agg** `service, avg(duration_ms) GROUP BY service` | `10 8 7 8 7 6 7 5` → **~7 ms** | `17 23 20 14 14 13 18 13` → **~15 ms** | **~2.1×** |

**Verdict — Run 31's ~10× was cold; warm the gap is ~2–5× and shape-dependent. Correction stands.**

- **The unindexed-scan gap is ~2–5× warm, not ~10×.** The pure point-filter scan is widest (~5×:
  CH's vectorized decode-and-compare on one column is its strongest case), and it **compresses to
  ~2×** as aggregation/grouping work is added (consistent with Run 96 — added per-row compute both
  engines pay dilutes ClickHouse's scan-throughput edge). Confirms the Run 40 correction (Run 31's
  GT 95 ms was a cold/HTTP-wall artifact; warm GT full-scan is ~13–15 ms).
- **All shapes are ≪ the 300 ms gate at 1M** — even *ad-hoc, unindexed* scans are interactive on
  GreptimeDB at this scale. So the DQ5 flip trigger does **not** fire on latency at 1M; it requires
  **GB–TB cold scale**, where the gap widens (Run 58: ~3× agg-bound → ~14× scan-bound at 5M, larger
  cold) **and** a scan-dominated workload. The "~10×" should not be quoted as the warm gap.
- **No verdict change, but the DQ5 number needs updating** (verdict cites Q5 ~10×): the honest warm
  figure is ~2–5× at 1M; the scan-engine gap is real and routes to parity-roadmap #2, but it is not
  a hot-path concern for Parallax's *anchored* pattern (which prunes via index — Run 99).

**Reproduce.** On `spans` (1M, `span_id` unindexed): `SELECT count() WHERE span_id='<id>'` (~3/15 ms);
`SELECT status,count() GROUP BY status` (~5.5/13 ms); `SELECT service,avg(duration_ms) GROUP BY
service` (~7/15 ms). Warm ×8–10; CH `--time`, GT `execution_time_ms`.

### Run 103 — 2026-05-25 — Cross-tier in-DB join pushdown re-verified: CH prunes ~3 ms, GT full-scans ~53 ms (~17×), subquery-prefilter workaround ~19 ms (Run 81 reproduces, no drift)

**Pass target.** Re-verify the load-bearing **cross-tier in-DB join** result (Run 81, parity-roadmap
#8, verdict DQ2): ClickHouse pushes an anchored equality filter through a join into the scan and
prunes; GreptimeDB does **not**, full-scanning the join input — and the Tier-A subquery-prefilter /
app-side workaround neutralises it. Hold this ClickHouse *win* (and the GT optimizer gap) to the bar.

**Environment.** GreptimeDB `v1.0.2` / ClickHouse `v26.5.1.882` (re-pinned live — latest stable, no
bump). `spans`/`spans_idx` 1M (trace_id INVERTED on GT), `error_events` 2,226. Anchored trace
`3fb2d84c…` = 14 spans + 1 error. Query = `spans ⋈ error_events ON trace_id WHERE
s.trace_id='X'`. Method: CH `--time`, GT `execution_time_ms`, warm.

| Query | latency (warm) | vs CH | mechanism |
| --- | --- | --- | --- |
| **CH** direct INNER join (anchored) | `12 3 3 3 3 3 4 3 3 3` → **~3 ms** | 1× | pushes `trace_id='X'` into the scan → prunes before join (`Granules 1` + PREWHERE) |
| **GT** direct INNER join | `155 57 52 51 54 49 53 52 62 54` → **~53 ms** | **~17×** | does **not** push the filter into the `spans_idx` scan → **full-scans 1M** (Run 81 EXPLAIN `output_rows: 1,000,000`); filter lands as a post-scan `FilterExec` |
| **GT** subquery-prefilter workaround | `21 18 16 19 19 18 19 18 21 20` → **~19 ms** | ~6× | `FROM (SELECT * FROM spans_idx WHERE trace_id='X') s …` lands the filter as the scan's own → inverted index prunes to 14 rows, then joins |

**Verdict — Run 81 reproduces exactly, no drift. The gap is a join-pushdown optimizer limitation, neutralised by the workaround.**

- **ClickHouse wins the *direct* in-DB cross-tier join (~17×: 3 ms vs 53 ms)** by pushing the
  anchor into the scan; **GreptimeDB full-scans the join input** because its optimizer does not push
  a join-input equality predicate to the `TableScan` as an index-eligible filter (parity-roadmap #8,
  Tier-A workaround / Tier-B optimizer fix). Mechanism unchanged from Run 81.
- **The subquery-prefilter workaround cuts GT to ~19 ms (~3× faster than the direct join)** — the
  inverted index prunes the spans side to 14 rows before the join. The residual ~6× vs CH is the
  join + HTTP overhead on a tiny row set, **all ≪ the 300 ms gate**.
- **Parallax is unaffected on the hot path:** evidence-bundle assembly is **app-side correlation**
  (anchored fetch each signal + join in the app — Q6 = Q1+Q2+Q3, Run 99 ~16 ms), not an in-DB join.
  So this CH win bites only if Parallax adds *direct in-DB cross-tier joins*; the designed pattern
  sidesteps it. No verdict change; carry the "don't rely on GT join-input pushdown — pre-filter or
  correlate app-side" note into the blueprint.

**Reproduce.** Find a trace_id in both spans+error_events; CH `SELECT count() FROM spans s JOIN
error_events e ON s.trace_id=e.trace_id WHERE s.trace_id='X'` (~3 ms); GT same on `spans_idx`
(~53 ms, full-scan); GT subquery `FROM (SELECT * FROM spans_idx WHERE trace_id='X') s JOIN …`
(~19 ms, pruned). Warm ×10.

### Run 104 — 2026-05-25 — Dynamic-attribute JSON path query re-verified — gap WIDENED to ~57× (CH ~1 ms / GT ~57 ms @200k): CH's new JSON subcolumn read matured, GT's per-row jsonb parse unchanged

**Pass target.** Re-verify the load-bearing ClickHouse *win* on **dynamic-attribute path queries**
(Run 61, parity-roadmap #4, verdict DQ2): CH stores each JSON path as a typed columnar subcolumn;
GreptimeDB stores `Json` as a binary jsonb blob read with per-row `json_get_*`. Production shape: a
user groups/filters by an **undeclared** OTLP attribute.

**Environment.** GreptimeDB `v1.0.2` / ClickHouse `v26.5.1.882` (re-pinned live — latest stable, no
bump). Built `sj (ts, trace_id, service, attributes JSON)` on both, loaded **200,000 rows** from
`spans`/`spans_idx` with `attributes = {"http":{"status_code":<200..600>}}` (CH `JSON` type, GT
`JSON` jsonb via `parse_json`). Query: `GROUP BY` the attribute path. Method: CH `--time`, GT
`execution_time_ms`, warm.

| Query | ClickHouse | GreptimeDB | Ratio |
| --- | --- | --- | --- |
| `GROUP BY attributes.http.status_code` (200k) | `~1 ms` (typed subcolumn) | `56 55 57 57 60 58 56 57 55 59` → **~57 ms** (`json_get_int`, per-row jsonb parse) | **~57×** |

**Verdict — Run 61 reproduces in direction but the gap WIDENED (CH improved); honest update needed.**

- **The dynamic-attr gap is now ~57× warm at 200k (CH ~1 ms / GT ~57 ms)**, vs Run 61's ~13× (CH
  6 ms / GT 78 ms @100k). The change is on **ClickHouse's side**: the 26.x **new `JSON` type**
  matured — a path like `attributes.http.status_code` reads exactly one **typed, dictionary-encoded
  subcolumn** (5 distinct values here) in ~1 ms. GreptimeDB's `json_get_int` still **parses the
  whole jsonb blob per row** (`jsonb::get_by_path`, parity-roadmap #4) → ~57 ms / 200k rows
  (~0.28 µs/row, scan-bound, grows linearly). So the mechanism is unchanged; CH simply got faster at
  it, widening the ratio. **Correct the verdict's "~13×" to "~13–57× (CH's subcolumn read improved
  in 26.x; widens with CH maturity)".**
- **Scope unchanged — this is the *undeclared/arbitrary* attribute case only.** The Tier-A mitigation
  still holds: for **known hot attributes**, GreptimeDB promotes them to real typed columns at
  ingest (schema-on-write auto-adds columns, Run 18) → a normal columnar group-by (~13 ms class,
  Run 102), erasing the gap. Parallax's anchored bundle fetches attributes *for a trace* (already
  pruned), not `GROUP BY` across 200k undeclared paths — so this bites only if Parallax ships heavy
  **ad-hoc arbitrary-attribute analytics**. Then it is a real, now-larger ClickHouse advantage.
- **Adopt-native / blueprint:** carry the rule — **promote hot OTLP attributes to typed columns**
  (don't leave them in the jsonb blob for repeated analytics); reserve the `Json` column for
  genuinely sparse/unpredictable attributes accessed by anchored fetch, not aggregation.
- **parity-roadmap #4 strengthened:** the JSON-shredding improvement (shred paths into Parquet
  subcolumns) now closes a **bigger** gap than measured at Run 61 — but is still **Tier-B
  integration** (Parquet Variant), and Parallax's Tier-A column-promotion covers the common case.

**Reproduce.** Build `sj (… attributes JSON)` both; load 200k with `{"http":{"status_code":N}}`
(GT via `parse_json(concat(...))`); CH `SELECT attributes.http.status_code, count() GROUP BY 1`
(~1 ms); GT `SELECT json_get_int("attributes",'http.status_code'), count(*) GROUP BY 1` (~57 ms).
Drop `sj` after.

### Run 105 — 2026-05-25 — PromQL vs SQL re-verified: GreptimeDB's own PromQL ~5.6× slower than its own SQL (Run 44 reproduces); wide PromQL range is OVER the 300 ms gate — "metrics = capability not speed"

**Pass target.** Re-verify the load-bearing **"metrics → GreptimeDB is capability/ergonomics, not
speed"** claim (Run 44): GreptimeDB's *native* PromQL path is materially slower than its own SQL,
because the PromQL planner pays a near-fixed `SeriesNormalize`/`SeriesDivide` series-sort setup. The
metric-agg ordering should be **CH SQL > GT SQL > GT PromQL**.

**Environment.** GreptimeDB `v1.0.2` / ClickHouse `v26.5.1.882` (re-pinned live — latest stable, no
bump). `metrics_hc` = 8M rows / 40k series (40 svc × instances), parity. Query = `avg by (service)`
over a **60-min range at 60 s step** (~2,400 output points). GT PromQL via `TQL EVAL`; GT/CH SQL via
matched `date_bin`/`toStartOfInterval` bucketed group-by over the same window. Method: GT
`execution_time_ms`, CH `--time`, warm.

| Path | warm reps (ms) | median | ratio |
| --- | --- | --- | --- |
| **CH SQL** | `69 55 54 56 52 54 54 61` | **~55 ms** | 1× (baseline) |
| **GT SQL** (`date_bin`) | `134 118 118 115 105 126 128 123` | **~120 ms** | ~2.2× CH |
| **GT PromQL** (`TQL EVAL`) | `756 670 683 639 675 676 666 724` | **~675 ms** | **~5.6× GT-SQL, ~12× CH** |

**Verdict — Run 44 reproduces exactly, no drift. Ordering CH SQL > GT SQL > GT PromQL confirmed.**

- **GreptimeDB's native PromQL is ~5.6× slower than its own SQL** (675 vs 120 ms) on the same
  `avg by (service)` over the same window — the `SeriesNormalize`/`SeriesDivide` series-sort setup
  the PromQL planner pays, which a streaming SQL hash-agg avoids. So on metrics, GreptimeDB's edge is
  PromQL **maturity/ergonomics** (GA, default-on, range-vector/`rate`/lookback expressiveness), **not
  query speed** — exactly the verdict's framing.
- **Sharp practical caveat (sharper than Run 44 stated):** a **wide** PromQL range over 40k series is
  **~675 ms — OVER the 300 ms interactive gate.** Wide/high-card PromQL range queries are *not*
  interactive on GreptimeDB at this scale. The Tier-A answers apply: for hot/interactive metric
  panels use **SQL** (~120 ms here, Run 96), **Flow pre-aggregation** (Run 43), or **narrow the
  series** with label filters; reserve PromQL for alerting/expressiveness, not wide interactive
  dashboards. *(A real dashboard PromQL query is usually narrower — few series via label matchers,
  shorter range — so it lands faster; the 675 ms is the wide-range worst case.)*
- **CH context:** ClickHouse's own PromQL (26.x `TimeSeries` engine) is experimental/off-by-default,
  so the GA-PromQL comparison still favours GreptimeDB on *capability*; this run is GT-PromQL vs
  GT-SQL (the speed claim), which holds.
- **Adopt-native (metrics):** unchanged — ADOPT the native metric engine, but **drive hot panels
  with SQL/Flow, not wide PromQL**. Carry this into the blueprint.

**Reproduce.** GT PromQL: `TQL EVAL (1716000000, 1716003600, '60s') avg by (service) (metrics_hc)`
(~675 ms). GT SQL: `SELECT date_bin('60 seconds'::INTERVAL, ts) m, service, avg(value) FROM
metrics_hc WHERE ts BETWEEN …::timestamp_ms GROUP BY m, service` (~120 ms). CH SQL:
`toStartOfInterval(ts, INTERVAL 60 SECOND)` equivalent (~55 ms). Warm ×8.

### Run 106 — 2026-05-25 — Vendor-claims audit + live-verified the RC2 "100× TopK" gap-closing claim on our v1.0.2 (GT ~20 ms / CH ~7 ms, both ≪ 300 ms)

**Pass target.** The operator asked to audit GreptimeDB's own marketing/comparison pages
(`greptime.com/compare/click_house` + 15 blogs) for accuracy/manipulation vs our findings, and to
re-verify — not trust — their claims. Full audit in `vendor-claims-audit.md`. The one directly
testable engine claim was the **RC2 "100× faster TopK" (dynamic filter pushdown into the Mito
scan)** — our containers are `v1.0.2` (post-RC2 GA), so it should already be in the binary. Verify.

**Environment.** GreptimeDB `v1.0.2` / ClickHouse `v26.5.1.882` (re-pinned live — latest stable, no
bump). `spans`/`spans_idx` 1M, parity. Query = `ORDER BY duration_ms DESC LIMIT 10` (a TopK over an
unindexed numeric column — a naive full sort of 1M would be ~100 ms+). Method: CH `--time`, GT
`execution_time_ms`, warm.

| Engine | warm reps (ms) | median |
| --- | --- | --- |
| ClickHouse | `17 5 7 6 8 7` | **~7 ms** |
| GreptimeDB | `105 24 22 15 18 21` | **~20 ms** |

**Verdict — the gap-closing claim is REAL and live in our v1.0.2; ~3×, both interactive.**

- **GreptimeDB TopK is ~20 ms on 1M (not a ~100 ms+ full sort)** → the **dynamic-filter-pushdown
  TopK optimization shipped in RC2 is present in v1.0.2**, so all our scan/sort benchmarks already
  benefit from it. CH ~7 ms (~3×), both ≪ the 300 ms gate.
- **Significance for DQ6 (the investment thesis):** this is **independent, live-verified evidence**
  that GreptimeDB closes scan-engine gaps via **DataFusion runtime dynamic filters** — exactly the
  "closable via the DataFusion roadmap / contributable Rust" mechanism the operator's long-term bet
  assumes. Not just a vendor blog number — reproduced on our containers.
- **Audit conclusion (separate deliverable, `vendor-claims-audit.md`):** the compare page sells GT
  on fit/storage/economics/native-protocols — where our runs *also* put GT's wins — and **never
  claims raw-analytical-speed superiority** (the one thing our data would refute). The
  log-monitoring blog *concedes* CH is faster on keyword search; the ingestion benchmark
  independently confirms our **cardinality-insensitivity** win on v1.0 GA. Misleading bits (Poizon
  "seconds→ms" = GT-vs-ETL not GT-vs-CH; structured-keyword "GT faster" = unstated config; TSBS
  "67×" = vs row-store Postgres on agg-stacked workload) are **peripheral** and do not flip the
  decision. Two corrections folded in (disk index-file cache exists *in addition to* the in-memory
  caches; OTel-Arrow is experimental/Phase-2, not GA) + JSON Type v2 (v1.1/Q2) will narrow the
  Run-104 dynamic-attr gap.

**Reproduce.** `SELECT trace_id, duration_ms FROM spans ORDER BY duration_ms DESC LIMIT 10` on each
(GT `spans_idx`), warm ×6. Expect GT ~20 ms (TopK pushdown, not full sort) / CH ~7 ms.

### Run 107 — 2026-05-25 — Log-explorer hot queries (service-tail + errors-in-window): CH ~6–7× via sort-key locality, but both ≪ 300 ms (GT interactive); a concrete instance of the #5 alternate-ordering gap

**Pass target.** Model the **log-explorer / live-tail** hot path — the most-run operational log query
(every incident opens with "tail the logs for service X" and "show errors in the last window").
Production-realistic, single-user, often-run.

**Environment.** GreptimeDB `v1.0.2` / ClickHouse `v26.5.1.882` (re-pinned live — latest stable, no
bump). `logs_b1` = 5M rows / 12 services / INFO 3.5M·WARN 899k·ERROR 600k / ~9 h span, parity. CH
`ORDER BY (service, ts)`; GT `PRIMARY KEY (service, level)` + `TIME INDEX ts`. Method: CH `--time`,
GT `execution_time_ms`, warm.

| Query | ClickHouse | GreptimeDB | Ratio |
| --- | --- | --- | --- |
| **Q1 service tail** `WHERE service='svc-8' ORDER BY ts DESC LIMIT 100` | `6 3 4 3 4 4 4 4` → **~4 ms** | `104 24 22 23 26 33 36 33` → **~28 ms** | **~7×** |
| **Q2 errors-in-window** `WHERE level='ERROR' AND ts >= <last 30 min> ORDER BY ts DESC LIMIT 100` | `10 10 9 9 10 8 10 11` → **~10 ms** | `84 62 61 56 63 57 67 62` → **~60 ms** | **~6×** |

**Verdict — a real ~6–7× ClickHouse win on a common operational query, but both interactive (≪ 300 ms).**

- **ClickHouse's `ORDER BY (service, ts)` sort-key locality is structurally ideal for the time-DESC
  tail:** "recent logs for a service ordered by time" reads the tail of a sorted run directly (~4 ms,
  no sort). GreptimeDB pays a reverse-ordered scan within the service partition (~28 ms). Q2 adds a
  `level` filter + ts window: CH prunes ts granules + filters (~10 ms); GT ~60 ms.
- **This is a concrete manifestation of the #5 alternate-ordering gap** (parity-roadmap): CH decouples
  physical sort order from identity, so `(service, ts)` clustering serves time-DESC-per-service for
  free; GreptimeDB's `PK = sort = series` cannot give the same time-within-service locality. One of
  the larger *warm* ratios measured (vs metric-agg ~2–3×, ad-hoc scan ~2–5×, anchored ~3×) precisely
  because the query is pure sort-key-locality territory.
- **But both are ≪ the 300 ms gate** — GT ~28 ms / ~60 ms is interactive; the log explorer opens
  instantly on either. So this is a fair "CH genuinely better" point, not a Parallax blocker.
- **Adopt-native (logs) refinement:** GreptimeDB's benchmark table uses `PK (service, level)` — the
  `level` in the key does **not** help the time-DESC tail and adds a sort dimension. For a log-tail
  workload, **PK on `service` (drop `level` from the key; keep it a plain/indexed column)** is the
  better blueprint, and for a heavy time-ordered-tail pattern a Flow-maintained `ts`-leading copy
  (#5a) or accepting ~28 ms. Carry into the logs blueprint: key by the anchor you tail on, not by
  `level`.

**Reproduce.** On `logs_b1` (5M): Q1 `SELECT ts,level,message,trace_id WHERE service='svc-8' ORDER BY
ts DESC LIMIT 100`; Q2 `… WHERE level='ERROR' AND ts >= '<max-30min>' ORDER BY ts DESC LIMIT 100`.
Warm ×8; CH `--time`, GT `execution_time_ms`. Expect CH ~4/~10 ms, GT ~28/~60 ms.

### Run 108 — 2026-05-25 — Verified the Run-107 blueprint claim: PK(service) only ~10% faster than PK(service,level) for the log-tail — directionally right, but a MINOR lever (the ~7× gap to CH is structural #5, not PK choice)

**Pass target.** Run 107 claimed GreptimeDB log tables should key by `service` (not `service,level`)
for the time-DESC tail. The brief says **verify claims, don't speculate** — so A/B it directly
rather than leave it as advice.

**Environment.** GreptimeDB `v1.0.2` (re-pinned live — no bump). Built two identical 1M-row tables
from `logs_b1` (same data, `append_mode`): `gt_logs_sl` `PRIMARY KEY(service,level)` vs `gt_logs_s`
`PRIMARY KEY(service)`. Within a GreptimeDB region data is sorted by `(PK…, ts)`, so the hypothesis
was: `PK(service)` → `(service, ts)` order serves the per-service tail directly, while
`PK(service,level)` → `(service, level, ts)` forces a merge across per-level runs for a cross-level
ts-DESC tail. Query = `WHERE service='svc-8' ORDER BY ts DESC LIMIT 100`. Warm ×8 (`execution_time_ms`).

| Layout | warm reps (ms) | median |
| --- | --- | --- |
| `PK(service, level)` | `32 33 29 31 43 29 28 27` | **~30 ms** |
| `PK(service)` | `27 28 27 27 27 26 27 27` | **~27 ms** |

**Verdict — claim is directionally CORRECT but the effect is MINOR (~10%); correct the overstatement.**

- **`PK(service)` is only ~10% faster (~27 vs ~30 ms)** for the cross-level tail — and noticeably more
  *stable* (no variance spikes). So keying by `service` (not `service,level`) is a real but **small**
  optimization; `level` in the PK adds a minor sort-grouping cost with no tail benefit. Prefer
  `PK(service)` for log-tail tables — but it is **not** the main lever.
- **The dominant cost is GreptimeDB's reverse-ordered ts-DESC scan itself, not the PK composition.**
  Neither layout approaches ClickHouse's ~4 ms (Run 107) — the ~7× gap is the **structural #5
  alternate-ordering / sort-key-locality gap** (GT's `PK=sort=series` can't make `ts` the leading
  physical order; `order_by` table option is rejected, Run 65), not something a PK tweak closes.
- **Correction to Run 107:** downgrade "key by `service`" from a "fix" to a "minor optimization." The
  honest blueprint line: prefer `PK(service)` for tail-heavy log tables (small, free win), but expect
  ~27 ms (interactive, ≪ 300 ms) — closing to CH's ~4 ms would require a real alternate-ordering
  structure (#5b, a Tier-B engine build) or a Flow `ts`-leading copy (#5a), which is rarely worth it
  since ~27 ms is already interactive.

**Reproduce.** Build `gt_logs_sl` `PK(service,level)` and `gt_logs_s` `PK(service)`, load same 1M from
`logs_b1` (`append_mode`); run `WHERE service='svc-8' ORDER BY ts DESC LIMIT 100` warm ×8 on each.
Expect ~30 vs ~27 ms. Drop both after.

### Run 109 — 2026-05-25 — Last-value / "current value" stat-panel query: GreptimeDB WINS ~2.4× (GT ~17 ms / CH ~41 ms) — time-sorted layout beats argMax full-scan; the vendor "GT loses lastpoint" does NOT carry to ClickHouse

**Pass target.** Model the **"current value" stat-panel** query (every dashboard's single-stat /
gauge: latest value per series) — very common, single-user. Also test a vendor-audit lead: the
TimescaleDB benchmark showed GreptimeDB *losing* "lastpoint" 8.7× to TimescaleDB — does that
last-value weakness also show vs ClickHouse?

**Environment.** GreptimeDB `v1.0.2` / ClickHouse `v26.5.1.882` (re-pinned live — no bump).
`metrics_hc` 8M rows / 40 svc / 40k series, parity. Query = current value per service over the
window. CH `argMax(value, ts)`; GT `last_value(value ORDER BY ts)` — same semantics (value at max
ts per group). Warm ×8.

| Engine | warm reps (ms) | median |
| --- | --- | --- |
| ClickHouse `argMax(value, ts)` | `48 39 37 49 44 44 37 41` | **~41 ms** |
| GreptimeDB `last_value(value ORDER BY ts)` | `23 15 21 16 19 17 17 15` | **~17 ms** |
| **Winner** | | **GreptimeDB ~2.4×** |

**Verdict — a GreptimeDB WIN on a common metric query; refines "CH always faster."**

- **GreptimeDB is ~2.4× FASTER on last-value** (17 vs 41 ms). Mechanism: GreptimeDB's data is
  physically sorted by `(PK…, ts)`, so "latest value per series" is a cheap **tail read** of each
  series run; ClickHouse's `argMax` must **full-scan 8M rows** tracking max-ts state per group. This
  is the time-series-native layout paying off — a genuine GreptimeDB metric-query win, not just fit.
- **Corrects a naive reading of the vendor audit:** GreptimeDB *losing* lastpoint **to TimescaleDB**
  (Run 106 / their TSBS) does **NOT** mean it loses lastpoint to ClickHouse — vs ClickHouse on this
  metric last-value it **wins**. Different rival, different layout (TimescaleDB is a row-store with a
  last-point optimization; ClickHouse's `argMax` has no such shortcut). So "GreptimeDB slow on
  point/last-value" is **engine-relative** — true vs a tuned TSDB, false vs ClickHouse here.
- **Decision relevance:** the "current value" stat panel is one of the most common dashboard
  queries; GreptimeDB serving it ~2.4× faster than ClickHouse (both ≪ 300 ms) is a small but real
  point in GreptimeDB's favour on the metrics axis — alongside the cardinality-insensitive ingest
  (Runs 84/101) and high-card storage crossover (Run 100). Adds nuance to DQ1: metrics→GreptimeDB is
  *mostly* capability/ergonomics, but **last-value is also a speed win**.
- **Adopt-native (metrics):** unchanged/strengthened — the native metric engine's time-sorted layout
  is exactly what makes last-value cheap; ADOPT stands.

**Reproduce.** On `metrics_hc` (8M/40-svc): CH `SELECT service, argMax(value, ts) FROM metrics_hc
GROUP BY service`; GT `SELECT service, last_value(value ORDER BY ts) FROM metrics_hc GROUP BY
service`. Warm ×8. Expect GT ~17 ms / CH ~41 ms.

### Run 110 — 2026-05-25 — Schema-on-write / OTLP-drift re-verified: GreptimeDB auto-adds columns on ingest (zero migration, NULL-backfill); ClickHouse rejects (`Code: 16 NO_SUCH_COLUMN`)

**Pass target.** Re-verify the GreptimeDB **ingest-ergonomics** pillar (Run 18, source-confirmed
`create_or_alter_tables_on_demand`): a new telemetry attribute lands with zero migration on
GreptimeDB, while ClickHouse rejects unknown-column inserts. Production-realistic — OTLP attribute
sets drift constantly (new SDK/service versions add fields/labels).

**Environment.** GreptimeDB `v1.0.2` / ClickHouse `v26.5.1.882` (re-pinned live — containers 13 h up,
no bump). GreptimeDB via the **InfluxDB line protocol** (`/v1/influxdb/write`, a schema-on-write
ingest path); ClickHouse via SQL `INSERT`.

**GreptimeDB — schema-on-write:**
- Write 1: `drift_test,host=a temp=20.5 …` → schema `host, temp, greptime_timestamp` (HTTP 204).
- Write 2 (drift): `drift_test,host=a,region=us temp=21.0,humidity=55.2 …` → **schema auto-gained
  `region` (String) + `humidity` (Float64)** (HTTP 204, **zero migration**).
- Old row **NULL-backfilled**: `['a', 20.5, None, None]` then `['a', 21.0, 'us', 55.2]`. Correct.

**ClickHouse — rejects:**
- `INSERT INTO drift_test (ts,host,temp,humidity) …` → **`Code: 16 … No such column humidity in table
  … (NO_SUCH_COLUMN_IN_TABLE)`**. Requires a prior `ALTER TABLE … ADD COLUMN` (managed migration) or
  routing dynamic attributes into a `JSON` column.

**Verdict — Run 18 reproduces exactly, no drift. A real GreptimeDB operational-simplicity win.**

- **GreptimeDB absorbs OTLP attribute drift with zero ops** — a new attribute becomes a typed column
  on first sight, history NULL-backfilled. ClickHouse needs either a **managed ALTER pipeline** on
  every drift, or a **`JSON` column** for dynamic attributes — which then carries the **~13–57×
  query penalty** (Run 104). So the ClickHouse "handle drift" options both cost something GreptimeDB
  doesn't: ops complexity (ALTER) or query speed (JSON blob).
- **Decision relevance:** Parallax's telemetry drifts continuously (every new service/SDK adds
  attributes). GreptimeDB's schema-on-write is a genuine ingest-ergonomics + ops-simplicity edge —
  reinforces DQ1's "write ergonomics / schema evolution" rows and the startups-first/no-ops-team
  trajectory. **Caveat to carry:** auto-add is convenient but unbounded auto-columns on extreme drift
  could widen tables; promote only *expected* hot attributes to columns and keep genuinely arbitrary
  ones in a `Json` column (per Run 104's promote-hot-attrs rule).
- **Adopt-native:** the native ingest paths (InfluxDB line here; OTLP/Prom in prod) all schema-on-write
  → ADOPT-native ingest stands; no custom migration layer needed for drift.

**Reproduce.** GT: two `/v1/influxdb/write` lines, the 2nd adding a new tag+field; `DESC TABLE` shows
the auto-added columns, old rows NULL. CH: `CREATE TABLE (ts,host,temp)`, then `INSERT … (…,humidity)`
→ `Code: 16 NO_SUCH_COLUMN_IN_TABLE`. Drop `drift_test` after.

### Run 111 — 2026-05-25 — Retention/TTL REFINED: ClickHouse drops fully-expired parts cheaply (rewrite only at boundary/mixed parts) + GreptimeDB TTL purge is eventual/background — the cost gap is narrower than Run 17's worst-case framing

**Pass target.** Re-verify the GreptimeDB "cheap-by-default retention" pillar (Run 17: GT whole-SST
drop vs CH rewrite-survivors). Test the mechanism on both rather than trust the prior framing.

**Environment.** GreptimeDB `v1.0.2` / ClickHouse `v26.5.1.882` (re-pinned live — 13 h up, no bump).

**ClickHouse (TTL `ts + INTERVAL 1 SECOND`, `merge_with_ttl_timeout=0`):**
- Inserted 100k EXPIRED (2 h-old) in one INSERT + 100k LIVE (future) in another → **two parts**.
  Count *before* any OPTIMIZE = **100,000** → the **fully-expired part was already dropped in the
  background, the live part untouched (NO rewrite).** `OPTIMIZE FINAL` → still 100k live.
- So **CH drops a *fully-expired* part cheaply** (whole-part drop), *not* a rewrite. The Run-17
  "read 1M / rewrote 500k" cost applies to a part that **straddles the TTL boundary** (expired + live
  rows mixed in one part) — then CH must rewrite it to drop the expired subset.

**GreptimeDB (`ttl='1s'`, `append_mode`):**
- Inserted 100k rows with ts ~2 **years** past (TTL-expired by a wide margin). Count = 100,000.
- `ADMIN compact_table('ttl_gt')` returned success (0) — but count **stayed 100,000** after +2 s.
  **GreptimeDB's TTL purge is EVENTUAL/background** (a scheduled job), **not forced by an on-demand
  compaction.** The whole-SST-drop *mechanism* stands (Run 17 + source `compactor.rs:581`), but the
  *timing* is background, not immediate.

**Verdict — refines Run 17; the retention-cost gap is NARROWER than first framed.**

- **Both engines drop *fully-expired* time-chunks cheaply** (CH whole-part drop, verified; GT
  whole-SST drop, mechanism-confirmed) — for **time-ordered** observability ingestion (data arrives
  in time order → old parts/SSTs become fully expired), retention is **cheap on both**. CH's rewrite
  cost (Run 17) is the **boundary part** case (a part straddling the TTL cutoff), not all retention.
- **GreptimeDB's real edge is "cheap-by-default, zero config":** TWCS auto-time-windows SSTs so
  expired windows drop whole with no tuning. **ClickHouse needs the config the blueprint already
  specifies** — `ORDER BY ts` (time-ordered parts) + ideally `PARTITION BY` time + `ttl_only_drop_
  parts=1` — to get the same cheap whole-part/partition drop. So: GT cheap-by-default; CH
  cheap-when-time-partitioned (which you do anyway). **Correct the "CH always rewrites survivors"
  overclaim** — it rewrites only the boundary part, or any part where expired+live are mixed (e.g.
  no time-ordering).
- **New small finding:** GreptimeDB TTL purge is **eventual** (background-scheduled), so freshly-aged
  data lingers until the purge job runs; not an on-demand operation. Fine for retention (eventual is
  expected) but note it — you can't force-reclaim instantly via `compact_table`.
- **Decision relevance:** the "cheap retention" GT pillar is **real but narrower** — it's an
  *ergonomics/zero-config* win, not a 2× cost gap, once ClickHouse is time-partitioned (which the
  Parallax CH blueprint already is). Tempers DQ1's retention row.

**Reproduce.** CH: `TTL ts+INTERVAL 1 SECOND`, insert an all-expired part + a live part separately →
expired part drops whole (count = live, no rewrite); to see the rewrite, interleave expired+live in
ONE part (`ORDER BY v`, single INSERT) at the boundary. GT: `WITH(ttl='1s')`, insert old rows;
`ADMIN compact_table` does NOT force the purge (eventual/background). Drop scratch after.

### Run 112 — 2026-05-25 — Concurrent ingest + query re-verified: neither engine blocks reads under sustained ingest (~1.0× penalty both), neither explodes storage (CH merges→2 parts, GT LSM memtable)

**Pass target.** Re-verify the load-bearing operational claim (Run 53): under **sustained
concurrent ingest**, neither engine blocks/slows reads, and neither suffers storage explosion (CH
"too many parts" vs GT LSM). Production-realistic — Parallax ingests telemetry continuously while
users/agents query.

**Environment.** GreptimeDB `v1.0.2` / ClickHouse `v26.5.1.882` (re-pinned live — 13 h up, no bump).
Method: background loop inserting **50 × 20,000-row batches (1M rows)** into a scratch table while
foreground samples an **anchored query** (`count WHERE trace_id='…'`, 14-row result) on the static
1M-row `spans`/`spans_idx`. Isolates engine contention (ingest CPU/IO vs read latency). CH `--time`,
GT `execution_time_ms`, warm.

| Engine | anchored query, baseline (no load) | anchored query, DURING ingest | penalty | storage state after 1M ingested |
| --- | --- | --- | --- | --- |
| ClickHouse | ~2 ms | `3 2 2 2 3 2 2 2` → **~2 ms** | **~1.0× (none)** | **2 active parts** (50 inserts merged down — no explosion) |
| GreptimeDB | ~10 ms | `14 10 17 10 9 7 7 9 12 11` → **~10 ms** | **~1.0× (none)** | **1M rows in LSM memtable** (`sst_num=0` — absorbed in memory, queryable, no explosion) |

**Verdict — Run 53 reproduces, no drift. Neither blocks reads; neither explodes storage at a realistic rate.**

- **Ingest does NOT slow reads on either engine** — the anchored query stayed flat (CH 2→2 ms, GT
  10→10 ms, ~1.0× penalty both, even tighter than Run 53's 1.0–1.19×). The hot anchored path is
  effectively **immune to concurrent ingest** on both.
- **Neither explodes storage at this sustained-but-realistic rate:** ClickHouse's background merges
  collapsed 50 inserts into **2 active parts** (the "too many parts" failure is a *sustained-overload*
  regime where inserts outrun merges, not normal ingest — confirms Run 7/53); GreptimeDB **absorbed
  1M rows in its LSM memtable** (`sst_num=0`), queryable in-memory, flushing to SSTs later. Two
  different mechanisms (CH merge-on-write parts vs GT LSM memtable), same healthy outcome.
- **Decision relevance:** the "continuous ingest while querying" pattern — Parallax's normal operating
  mode — is **safe on both**. ClickHouse's edge needs a batching/async-insert layer only at
  *overload* rates (streaming tiny writes); GreptimeDB's LSM absorbs small writes natively (no batching
  layer). At realistic Parallax volumes neither degrades. No verdict change; reaffirms the
  write-ergonomics rows (GT no-batching-layer) and that reads are safe under load on both.

**Reproduce.** Create a scratch table each side; background loop `INSERT … 20k rows × 50` (CH `FROM
numbers`, GT `FROM spans_idx LIMIT 20000`); foreground sample the anchored `count WHERE trace_id='X'`
on the static spans table ×8–10 during the load; check CH `system.parts` (active) and GT
`region_statistics.sst_num`. Expect flat query latency + bounded parts/SSTs. Drop scratch after.

### Run 113 — 2026-05-25 — Counter-rate panel (the #1 observability metric query): CH ~12 ms / GT ~19 ms (~1.6×) — smallest agg gap yet, both interactive; completes the metric-panel picture

**Pass target.** Model the **counter-rate panel** — request-rate / error-rate / CPU over time, the
single most common observability metric query (PromQL `rate()`). Completes the metric-panel set
alongside avg-by-service (Run 96, ~3×), bucketed line (Run 96, ~2×), last-value (Run 109, GT wins).

**Environment.** GreptimeDB `v1.0.2` / ClickHouse `v26.5.1.882` (re-pinned live — 13 h up, no bump).
`metrics_real` = 864k rows / 12 svc / monotonic `counter` / 6 h span, parity. Query = per-service
per-5-min-bucket counter delta (`max(counter)-min(counter)`, the rate numerator — same shape both
engines, fair). CH `toStartOfInterval`; GT `date_bin('5 minutes'::INTERVAL, ts)`. Warm ×8.

| Engine | warm reps (ms) | median |
| --- | --- | --- |
| ClickHouse | `12 12 10 11 12 16 27 14` | **~12 ms** |
| GreptimeDB | `97 19 19 17 21 16 20 19` | **~19 ms** |
| **Ratio** | | **~1.6×** |

**Verdict — the most common metric panel is ~1.6× (smallest agg gap measured), both interactive.**

- **~1.6× — smaller than flat avg-by-service (~3×, Run 96)** because the rate query does more per-row
  work (bucket `date_bin` + `max`/`min` per group + delta + more groups), which (consistent with Runs
  96/102) dilutes ClickHouse's scan-throughput edge. Both ≪ 300 ms — fully interactive.
- **Completes the metric-panel picture** across the common dashboard query types:
  - last-value / "current value" → **GreptimeDB wins ~2.4×** (Run 109)
  - counter-rate over time → **~1.6×** (this run)
  - bucketed line chart → **~2×** (Run 96)
  - flat avg-by-service → **~3×** (Run 96)
  - wide PromQL range → GT PromQL ~5.6× its own SQL (Run 105 — use SQL/Flow, not wide PromQL)
  So across real metric dashboards GreptimeDB ranges from **winning to ~3× behind, all interactive**.
- **Decision relevance:** metric dashboards are **interactive on GreptimeDB for every common panel**,
  and the speed gap is **small and shrinks as the query does more per-row work** (the scan-throughput
  edge only dominates flat full-table scans). Reinforces "metrics → GreptimeDB is capability/
  ergonomics; the speed gap is real but sub-perceptible on real panels." **Adopt-native metric engine
  stands** — the same DataFusion path serves all these panels.

**Reproduce.** On `metrics_real` (864k): CH `SELECT service, toStartOfInterval(ts, INTERVAL 5 MINUTE)
m, max(counter)-min(counter) FROM metrics_real GROUP BY service, m`; GT `date_bin('5 minutes'::
INTERVAL, ts)` equivalent. Warm ×8. Expect CH ~12 ms / GT ~19 ms.

### Run 114 — 2026-05-25 — BLUEPRINT GOTCHA: GreptimeDB default-dedup + high-cardinality PK = ~16× slower scans (~80× vs the right design); append_mode + low-card PK is mandatory for event signals

**Pass target.** I've used `append_mode='true'` throughout to dodge dedup confounds — but never
measured the *cost* of GreptimeDB's default (dedup) mode, nor isolated the PK-cardinality effect.
This is a real Parallax design decision: which write mode + PK for which signal. Quantify it.

**Environment.** GreptimeDB `v1.0.2` (re-pinned live — no bump). Three 1M-row tables from
`spans_idx`, identical data, varying only PK-cardinality and `append_mode`. Full-table agg
(`GROUP BY svc`, scan-bound), warm ×8.

| Table | PK | mode | full-scan agg (warm) | vs best |
| --- | --- | --- | --- | --- |
| `spans_idx` | `(service, name)` **low-card** | append | **~15 ms** | 1× (best) |
| `gt_ap` | `(span_id)` **1M-card** | append | **~76 ms** | ~5× |
| `gt_dd` | `(span_id)` **1M-card** | **default (dedup)** | **~1220 ms** | **~80×** |
| Ingest | — | dedup 1099 ms / append 894 ms | — | append ~1.2× faster to load |
| Point lookup `WHERE span_id` | — | dd ~10 ms / ap ~25 ms | (dedup faster on PK point-lookup — secondary) | — |

**Verdict — a critical GreptimeDB blueprint gotcha; two compounding effects isolated.**

- **Dedup on a high-cardinality PK is catastrophic on scans (~16× vs append on the SAME table):** the
  only difference between `gt_dd` (1220 ms) and `gt_ap` (76 ms) is `append_mode`. GreptimeDB's
  **`DedupReader` runs in the scan path** and merge-processes **every series** — with a 1M-distinct PK
  that is 1M single-row series to merge, even though there are **zero actual duplicates**. Append mode
  skips the dedup merge entirely.
- **High-cardinality PK itself costs ~5× on scans** (`gt_ap` 76 ms vs low-card-PK `spans_idx` 15 ms,
  both append) — more series to organize. Compounded with dedup, the naive `PK(span_id)+default` is
  **~80× slower** than the right design.
- **Firm blueprint rule (this quantifies why the existing spans design is correct):** for GreptimeDB
  **append-only event signals (spans / logs / traces)** use **low-cardinality `PRIMARY KEY` +
  `append_mode='true'`**, and keep high-card anchors (`trace_id`/`span_id`) as **`INVERTED`-indexed
  plain columns, NOT in the PK**. This is exactly `spans_idx` (`PK(service,name)` + `trace_id`
  INVERTED + append) — now measured as ~80× faster on scans than `PK(span_id)+dedup`. Reserve **dedup
  mode for genuine upsert signals with a LOW-card key** (issue status by fingerprint, deploy markers,
  metric last-value) where the per-series merge is cheap and the latest-wins semantics are needed
  (Run 19/59). Append also loads ~1.2× faster.
- **Decision relevance:** this is a GreptimeDB *operability* sharp edge (get the PK/mode wrong on a
  high-card event table and scans are ~80× slower) — not a GT-vs-CH point, but **essential for the
  "adopt GreptimeDB" implementation**. ClickHouse has an analogous trap (high-card column first in
  `ORDER BY`), so it's a both-engines schema-discipline requirement; GreptimeDB's is sharper because
  PK = sort = series = dedup-unit all at once.

**Reproduce.** Build `PK(span_id) default` vs `PK(span_id) append_mode='true'` vs `PK(service,name)
append_mode='true'`, load same 1M; `SELECT svc, count(*), avg(dur) GROUP BY svc` warm ×8 → ~1220 /
~76 / ~15 ms. Drop after.

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
