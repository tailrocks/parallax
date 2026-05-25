# Compression and Cost — On-Disk Layout, Codecs, Retention Economics

<!-- markdownlint-disable MD013 -->

Status: pass 8. The cost axis (#2). Combines the codec mechanisms from the
internals notes with **measured per-table/per-column sizes** from the live Docker
candidates, then ties to retention $. Builds on `local-benchmark-results.md`.

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

## Heavy caveat — synthetic cardinality

The `logs`/`errors` GreptimeDB win is **likely a synthetic artifact**: the
generator uses only ~10 distinct log messages and one error message, which is
extremely dictionary-friendly and unrepresentative of real log text (high-entropy,
many unique strings, stack traces). **Real log/trace text would narrow or reverse
the logs result.** Trust the *mechanism* (dictionary vs codec), not the exact
ratio. Re-run with realistic-cardinality text before any cost conclusion — routed
to `benchmarking-the-differences.md` and the harness generator.

## What actually decides Parallax's storage cost

Local-disk size deltas of 1.3–1.9× are **second-order** for Parallax. The
[retention cost model](../retention-cost-model.md) already shows object-storage
retention is ~100× cheaper than ingest-priced SaaS, and that egress pricing
(R2/B2 vs S3) dominates a re-read-heavy context engine. So the cost axis is
decided less by "who compresses spans 1.3× better" and more by:

1. **Object-storage-native vs object-storage-as-policy.** GreptimeDB is
   OpenDAL-native with a default local read cache (`greptimedb-internals.md`);
   ClickHouse uses an S3 disk under a storage policy with TTL-move tiering. For
   cheap, re-readable long retention GreptimeDB's design is the more direct fit.
   **Measured (Runs 8–9), same MinIO, 1M spans:** GreptimeDB **4 objects / 37 MiB**
   vs ClickHouse **74 objects / 63 MiB**. ClickHouse's Wide part writes **one S3
   object per column** (+ marks/metadata) → ~18× more objects → many more GET/PUT
   requests on a cold read; GreptimeDB writes a few large Parquet objects →
   request-efficient. **This is the concrete object-store-economics advantage** for
   GreptimeDB (per-request pricing dominates object-store bills for a re-read-heavy
   engine). Nuance: ClickHouse's *active logical* data was actually smaller (31.82
   vs 37 MiB — tuned codecs), but its raw S3 usage (63 MiB) was inflated by
   un-garbage-collected merge parts (async S3 cleanup) — transient space
   amplification GreptimeDB's flush model avoids. Request-count-during-query is the
   only remaining refinement.
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
- Measured sizes: `local-benchmark-results.md` Run 1/3 + this pass (`system.parts`, `system.parts_columns`, GreptimeDB per-`table_id` `du`).
- Retention $: `../retention-cost-model.md`.
