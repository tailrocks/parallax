# Compression and Cost — On-Disk Layout, Codecs, Retention Economics

<!-- markdownlint-disable MD013 -->

Status: pass 8, extended passes 88–89 (Runs 51–52: full-text index storage cost,
inverted + bloom-vs-bloom). The cost axis (#2). Combines the codec mechanisms from the internals notes with **measured
per-table/per-column sizes** from the live Docker candidates, then ties to retention
$. Builds on `local-benchmark-results.md`.

Pins: GreptimeDB `v1.0.2` (`0ef5451`), ClickHouse `v26.5.1.882-stable` (`5b96a8d8`).

## How each engine compresses (mechanism)

| | GreptimeDB | ClickHouse |
| --- | --- | --- |
| Format | Parquet SST | MergeTree column files (Wide) / packed (Compact) |
| Encoding | **Parquet auto-encoding** (dictionary/RLE chosen by the Arrow writer) + table-wide **ZSTD** (`src/mito2/src/sst/parquet/writer.rs:433`, `Compression::ZSTD`). | Per-column **`CODEC()` chain**, hand-tunable: `LZ4`(default)/`ZSTD` + `DoubleDelta`/`Gorilla`/`Delta`/`T64`/`GCD`/`ALP`/`FPC`, plus `LowCardinality` dictionary wrapper. |
| Control | **Automatic** — no per-column codec DDL exists at v1.0.2 (column options are index types only: fulltext/inverted/skipping). | **Manual per column** — you choose the codec to match each column's pattern. |
| Tradeoff | Less tuning burden, less control. | Maximum control, but you must know the data. |

So the question is empirical: does ClickHouse's hand-tuned codec breadth actually
beat GreptimeDB's automatic Parquet+ZSTD, **per signal**?

## Measured retained size (smoke, all tables flushed / `OPTIMIZE FINAL`)

Identical data loaded into both (1M spans, 214k logs, 2.2k errors, 864k metric
points). GreptimeDB sizes are the per-table SST dir (`/greptimedb_data/data/.../<table_id>/`),
**excluding** the transient WAL.

| Table (rows) | ClickHouse | GreptimeDB | Smaller | Dominant column pattern |
| --- | --- | --- | --- | --- |
| `spans` (1M) | **28.9 MiB** | 38 MiB | ClickHouse ~1.3× | high-cardinality random hex `trace_id`/`span_id` |
| `logs` (214k) | 10.24 MiB | **5.5 MiB** | GreptimeDB ~1.9× | low-cardinality repetitive text (⚠ synthetic) |
| `error_events` (2.2k) | 119.9 KiB | **92 KiB** | GreptimeDB | tiny, low-cardinality (⚠ synthetic) |
| `http_req_latency` (864k, random-walk float) | 6.31 MiB | **5.1 MiB** | GreptimeDB | high-entropy float (defeats Gorilla) |
| `metrics_real` (864k, counter+gauge) | **1.09 MiB** | 1.9 MiB | ClickHouse ~1.7× | monotonic counter + flat gauge |

**This contradicts the pass-2 assumption that ClickHouse's codec breadth wins
compression across the board.** It does not. There is **no blanket winner** —
compression is per-column-pattern:

- **ClickHouse wins where a specialized codec matches the pattern:** monotonic
  counters (`DoubleDelta` → 7.3× on the counter column), flat gauges (`Gorilla` →
  **78×** on the gauge column: 84.7 KiB from 6.59 MiB raw), and high-cardinality
  random strings (ZSTD-tuned, `spans`).
- **GreptimeDB wins where Parquet's automatic encoding fits:** dictionary-friendly
  low-cardinality columns (`logs`, `errors`), and **high-entropy floats where
  Gorilla backfires** (`http_req_latency` random walk — Gorilla's XOR produces
  near-incompressible output on noisy mantissas, so ClickHouse fell back to bulk
  ZSTD and lost to Parquet+ZSTD).

### Per-column proof (ClickHouse `metrics_real`)

| Column | Codec | Compressed | Raw | Ratio |
| --- | --- | --- | --- | --- |
| `gauge` | Gorilla, ZSTD | 84.7 KiB | 6.59 MiB | **78×** |
| `counter` | DoubleDelta, ZSTD | 922 KiB | 6.59 MiB | 7.3× |
| `ts` | DoubleDelta, ZSTD | 10.1 KiB | 6.59 MiB | 668× (regular 30 s step) |
| `service`/`instance` | LowCardinality | ~4–10 KiB | 0.85 MiB | dictionary |

## Realistic-cardinality logs (Run 10 — resolves the synthetic caveat)

Run 4's `logs` result used only ~10 distinct messages (extreme dictionary
friendliness). Re-run with **realistic high-entropy text** (500k rows, **99%
unique** messages — embedded UUIDs/IDs/latencies + stack traces):

| Schema | Total |
| --- | --- |
| GreptimeDB (default ZSTD-all) | **25 MiB** |
| ClickHouse (only `message` ZSTD; ids default LZ4) | 35.5 MiB |
| ClickHouse (**ZSTD on all string cols**) | **24.24 MiB** |

→ **Tie at matched effort** (24.24 vs 25 MiB); **GreptimeDB wins out-of-the-box**.
ClickHouse's per-column default is **LZ4**, which compresses the high-cardinality
hex `trace_id`/`span_id` poorly; switching them to ZSTD closes the gap. So the
earlier GreptimeDB logs win was a **default-codec effect, not a synthetic artifact
and not engine superiority** — it ZSTDs everything automatically, while ClickHouse
needs explicit per-column ZSTD on high-card columns. Confirms the "compression is a
tuning-dependent wash" conclusion on realistic data, with one operational nuance:
**GreptimeDB needs no codec tuning; ClickHouse does.**

## Full-text index storage cost (Runs 51–52 — the cost the column comparison misses)

Column compression is a wash, but for **logs** the full-text *index* is a large,
separate storage cost the table above ignores. Measured on an identical 5M-row log
corpus (1 SST/part; bloom filters sized for matched ~1% fpr):

| Full-text index on `message` | type | index | exact-term | overhead on data |
| --- | --- | --- | --- | --- |
| ClickHouse `text` (`splitByNonAlpha`) | inverted | **170 MiB** | 3 ms | **~75%** |
| GreptimeDB tantivy (Lucene-class) | inverted | **148 MiB** | ~6 ms | **~62%** |
| ClickHouse `tokenbf_v1` (1% fpr) | bloom | **19 MiB** | 8 ms | **~8.4%** |
| GreptimeDB bloom (1% fpr) | bloom | **18 MiB** | 9 ms | **~7.5%** |

**The axis is the index *family*, and it is identical on both engines** — not an
engine advantage either way:

- **Inverted-vs-inverted:** GreptimeDB tantivy is only ~13% smaller than ClickHouse
  `text` (148 vs 170 MiB); both cost **60–75% on top of the column data** and answer
  exact-term in 3–6 ms with phrase/ranking support. Full-text inversion is expensive
  on *both*.
- **Bloom-vs-bloom is a TIE** (CH `tokenbf_v1` 19 MiB / 8 ms vs GreptimeDB bloom 18
  MiB / 9 ms). Bloom size at a fixed fpr is governed by distinct-token count (pure
  math, `m ≈ 9.585·n` bits for 1%), so it is ~equal on the same corpus. *(Run 52
  corrected a prior Run 51 over-claim: the naive live numbers — CH inverted 170 MiB
  vs GreptimeDB bloom 18 MiB — compared different index families, an apples-to-oranges
  ~9×; and a first CH `tokenbf` build measured 57 MiB only because it was ~3×
  oversized for fpr ≪ 1%. At matched fpr the bloom tier ties.)*

**The real cost lever is bloom-vs-inverted, available on both engines:** choosing a
bloom full-text index over an inverted one saves **~55–65% of total log-table size**
(~18 MiB vs ~150–170 MiB index) at a ~2–3× exact-term latency cost (8–9 ms vs 3–6 ms)
and a capability tradeoff (token-membership only, probabilistic 1% fp re-checked at
scan; no phrase/ranking). For Parallax's **anchored** log search (exact
request-id/trace-id grep) the bloom tier is the right cost/latency point — and *both*
engines offer it.

**GreptimeDB's only edge here is ergonomics, not cost or speed:** it exposes both
tiers behind one `FULLTEXT INDEX WITH(backend=bloom|tantivy)` knob with
analyzer/case/phrase semantics; ClickHouse splits them — `text` (inverted, GA) vs
`tokenbf_v1`/`ngrambf_v1` (bloom *skip-index*, token-only, no analyzer-class
features). At log-retention scale the **index-family choice dominates the storage
delta, not the column codec or the engine.** (Source: `local-benchmark-results.md`
Runs 51–52.)

## What actually decides Parallax's storage cost

Local-disk size deltas of 1.3–1.9× are **second-order** for Parallax. The
[retention cost framing](retention-and-ttl.md) points to object-storage retention
being far cheaper than ingest-priced SaaS (a marketing-grade comparison, not a
GreptimeDB-vs-ClickHouse result), and egress pricing (R2/B2 vs S3) dominating a
re-read-heavy context engine. So the cost axis is
decided less by "who compresses spans 1.3× better" and more by:

1. **Object-storage-native vs object-storage-as-policy.** GreptimeDB is
   OpenDAL-native with a default local read cache (`greptimedb-internals.md`);
   ClickHouse uses an S3 disk under a storage policy with TTL-move tiering. For
   cheap, re-readable long retention GreptimeDB's design is the more direct fit.
   **Measured (Runs 8–9, re-verified Run 54), same MinIO, 1M spans:** GreptimeDB
   **3 objects / 21 MiB** vs ClickHouse **74 objects / 57 MiB** (Run 54 — the **74
   reproduced exactly** vs Run 9; ~**25× fewer** objects for GreptimeDB). ClickHouse's
   Wide part writes **one S3 object per column** (+ marks/metadata) **per part** →
   ~18–20 objects for even a single active part, ×N parts until merge-GC; GreptimeDB
   writes **one Parquet SST** (+ manifest) per flush → a handful. So **even fully
   GC'd** it is ~3 vs ~18–20 (~6–7×); the 74 includes transient un-GC'd merge parts
   (S3 lazy cleanup — `OPTIMIZE FINAL` left 2 parts). **This is the concrete
   object-store-economics advantage** for GreptimeDB (per-request pricing dominates a
   re-read-heavy bill). **Size-order nuance updated (Run 54):** on the anchored
   `PRIMARY KEY(trace_id)` schema Parallax actually wants, GreptimeDB's active logical
   data (21.8 MiB) is now **smaller** than ClickHouse's (28.9 MiB) — *reversing* the
   Run-1 local-disk order (CH 28.9 < GT 38 under `PK(service,name)`), because
   trace_id-sorting clusters the high-card hex columns for better Parquet
   dict/RLE+ZSTD. So GreptimeDB is both fewer-objects *and* smaller here. **B10 cold-read now measured (Run 55) — two-sided:**
   for one cold anchored lookup, **request count favours GreptimeDB** (9 vs 18 GETs)
   but **cold egress favours ClickHouse ~80×** (294 KiB granule reads vs ~23 MiB
   whole-SST; small-SST-inflated, at-scale owed). **Warm/repeat favours GreptimeDB**
   (write-through persistent local cache → ~0 S3 after first touch). So object-store
   economics split by regime — per-request + retained-object + warm-amortized re-reads →
   GreptimeDB; cold *selective* egress → ClickHouse (see `caching-and-cold-warm.md`).
2. **Compute per ingested GB and per query** — not yet measured (CPU/RSS sampling
   pending; the harness protocol covers it).
3. **Tiered retention**: both can keep hot data local and cold on object store;
   the cost is the object-store bill + egress on re-read, which the retention
   model already quantifies.

## Cost-axis verdict (provisional)

- **Local compression: a wash, pattern-dependent.** Neither engine dominates;
  ClickHouse for tuned numeric/high-card-string columns, GreptimeDB for
  dictionary-friendly and noisy-float columns. With realistic log text the gap on
  the dominant log/trace volume likely favors ClickHouse's hand-tuned ZSTD.
- **Retention economics likely favor GreptimeDB** on *ergonomics* (object-store
  native), but the $ delta vs ClickHouse S3-disk is **unproven** — both ultimately
  write compressed columns to the same object store, so the bill tracks retained
  bytes × storage price + egress, where the compression wash applies again.
- **Net:** cost is **not a strong differentiator** on current evidence — closer to
  a tie than the pass-2 reasoning implied. The decision should not rest on cost
  unless the MinIO/object-store run surfaces a real gap.

## What we still need (routed to benchmarking + harness)

1. **Realistic-cardinality dataset** (real-shaped log text, high-card attributes)
   — the current synthetic data distorts the log/error compression result.
2. **Object-storage run (MinIO)** for both — measure retained object bytes,
   GET/PUT/LIST counts, and cold-read egress; this is the real cost axis.
3. **Compute per ingested GB / per query** — CPU+RSS sampling per phase.
4. **Bigger tier** so part/SST compression ratios stabilize (smoke parts are tiny).

## Source / evidence

- GreptimeDB compression: `src/mito2/src/sst/parquet/writer.rs:36,391,433` (Parquet + ZSTD; no per-column codec DDL).
- ClickHouse codecs: `src/Compression/CompressionCodec*.cpp` (`clickhouse-internals.md`).
- Measured sizes: `local-benchmark-results.md` Run 1/3/10 (column compression) + Runs 51–52 (full-text index cost: inverted + fair bloom-vs-bloom) via `system.parts`, `system.data_skipping_indices`, GreptimeDB `information_schema.region_statistics`.
- GreptimeDB full-text backends (`bloom` vs `tantivy`): index built per SST in the `.puffin` sidecar; backend chosen in `FULLTEXT INDEX WITH(backend=…)` DDL (`indexing-internals.md`).
- Retention $ + TTL expiry mechanism: `retention-and-ttl.md`.
