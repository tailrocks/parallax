# Retention and TTL — How Old Telemetry Expires (Cost Axis #2)

<!-- markdownlint-disable MD013 -->

Status: pass 36. White-box teardown of the **TTL expiry mechanism** in each engine
— *when* old data is dropped, and *what it costs* to drop it. This is a first-class
lever for an observability product: Parallax keeps every signal on a retention
window (spans 30d, logs 30d, metrics 90d–400d, issue history long), and at steady
state the dominant background cost is **expiring old data**, not ingesting new. The
question is not "can it TTL" (both can) but **"does expiry rewrite surviving data or
just drop whole files,"** because that decides retention write-amplification and
object-storage churn. Absorbs the retention-cost references the comparison notes
left dangling. (The repo-wide `docs/research/retention-cost-model.md` cost gate is a
separate, broader artifact — not this note.)

Pins: GreptimeDB `v1.0.2` (`0ef5451`), ClickHouse `v26.5.1.882-stable` (`5b96a8d8`).
Both re-confirmed latest stable on 2026-05-25 (GreptimeDB v1.1.0 is nightly-only;
ClickHouse 26.5.x is the highest line, 26.2.19 is a 26.2 backport).

## GreptimeDB — whole-SST drop, no rewrite (source-confirmed)

TTL is a per-table option (`region_options.ttl`) resolved into the compaction path
(`compaction.rs:716` `find_dynamic_options`). The mechanism:

1. **TWCS time-windows the SSTs.** Time-Window Compaction Strategy buckets SSTs by
   time window, so an *old* window's SSTs cover a contiguous past time range.
2. **`get_expired_ssts`** (`compaction/twcs.rs:219`, def `compaction.rs:1091`) walks
   the levels and calls `level.get_expired_files(now, ttl)` — returning **whole
   `FileHandle`s whose time range is entirely past TTL**.
3. Those files are marked compacting (`twcs.rs:224`) so the picker won't merge them,
   then handed to the compactor as `expired_ssts`.
4. The compactor drops them **without reading or rewriting** — `compactor.rs:581`:

   ```rust
   // Include expired SSTs in removals — these don't depend on merge success.
   compacted_inputs.extend(picker_output.expired_ssts.iter().map(|f| f.meta_ref().clone()));
   ```

   They go straight into `files_to_remove` of the manifest edit (`files_to_add` only
   holds *merge* outputs). The comment "don't depend on merge success" is the tell:
   expiry is a **manifest edit + object DELETE of the whole Parquet SST**, independent
   of any merge work.

**Cost ≈ O(metadata) + one object DELETE per expired SST. No read, no rewrite, no
write-amplification.** Because TWCS already aligns SSTs to time windows, expiry is
*naturally* whole-file — there is no "partially expired part" case to rewrite. On
object storage this reclaims space directly (the S3 object is deleted).

**Caveat (honest):** expiry is **compaction-gated** — `get_expired_ssts` only runs
when a compaction is picked for the region. A region receiving no writes needs a
periodic/triggered compaction for its expired SSTs to actually be removed; expiry is
not a separate always-on timer. So "TTL=30d" means "dropped *at the next compaction
after* 30d," not to-the-second.

## ClickHouse — TTL DELETE merge; row-level by default (rewrites parts)

ClickHouse TTL is **applied during a special merge** ("TTL DELETE merge"), not a
separate reaper. Two source-confirmed settings decide its cost
(`MergeTreeSettings.cpp`):

- **`merge_with_ttl_timeout`** = `3600 * 4` = **4 hours** (line 1669): *"Minimum delay
  in seconds before repeating a merge with delete TTL."* TTL eviction for a partition
  is attempted at most every 4h — expiry is coarse-grained in time.
- **`ttl_only_drop_parts`** = **`false`** by default (line 1675). The source doc is
  explicit:
  > When `ttl_only_drop_parts` is disabled (by default), only the rows that have
  > expired based on their TTL settings are removed.
  > When `ttl_only_drop_parts` is enabled, the entire part is dropped if all rows in
  > that part have expired.

So **by default ClickHouse TTL is row-level**: when a part contains any expired rows,
a TTL merge **reads the part, drops the expired rows, and writes a new part** with the
survivors → **write-amplification proportional to the surviving (non-expired) data**,
repeated every time the merge re-qualifies. On an S3 disk this also churns objects
(rewrite = new object + delete old).

**The cheap path exists but must be configured:**

- Set **`ttl_only_drop_parts = 1`** so a fully-expired part is dropped wholesale (no
  rewrite), and
- **`PARTITION BY` a time bucket** (e.g. `toYYYYMMDD(ts)`) so each part belongs to one
  time bucket and an old bucket's parts become *fully* expired together.

With both, ClickHouse matches GreptimeDB's behavior: drop whole parts, no rewrite.
**Without partition alignment, parts straddle the TTL boundary forever and never fully
expire → perpetual row-level rewrites.** This is the classic ClickHouse retention
footgun.

## Side-by-side: the cost of expiring 1 day out of a 30-day window

| | GreptimeDB (default) | ClickHouse (default) | ClickHouse (tuned) |
| --- | --- | --- | --- |
| Unit dropped | Whole SST (TWCS time window) | Expired **rows** within a part | Whole part |
| Reads survivors? | **No** | **Yes** (re-reads the part) | No |
| Rewrites survivors? | **No** | **Yes** (writes a new part) | No |
| Write-amplification | ~0 (manifest edit) | ∝ surviving rows in touched parts | ~0 |
| Reclaims object storage | Directly (object DELETE) | After rewrite + old-part cleanup | On part drop |
| Time granularity | Next compaction after TTL | ≥ every 4h (`merge_with_ttl_timeout`) | ≥ every 4h |
| Config needed | None (TWCS default) | — | `PARTITION BY` time + `ttl_only_drop_parts=1` |

GreptimeDB gets cheap retention **by default** because its storage is already
time-windowed; ClickHouse gets it **only when explicitly partitioned by time and told
to drop parts**. Equal *capability*, unequal *defaults* — and defaults are what a
team actually runs.

## Parallax implication (and a DDL correction)

Parallax is retention-heavy and object-store-first, so retention write-amp is a real
recurring cost, not a one-off:

- **GreptimeDB** — per-table `ttl` + TWCS gives whole-SST drop with no rewrite; aligns
  with the object-store cost story (delete whole Parquet objects). Nothing to tune.
- **ClickHouse** — the seed DDL in `clickhouse-implementation.md` set `TTL … INTERVAL
  N DAY` but **omitted `PARTITION BY` and `ttl_only_drop_parts`**, which means default
  **row-level** expiry: every TTL merge rewrites surviving rows. **Correction applied
  to that note:** add `PARTITION BY toYYYYMMDD(ts)` (or coarser for low-volume tables)
  and `SETTINGS ttl_only_drop_parts = 1` so expiry drops whole parts. The
  `AggregatingMergeTree` rollup (400d) should partition coarser (e.g. `toYYYYMM(ts)`)
  to avoid tiny partitions.

This sharpens the **cost axis (#2)** retention sub-cell: retention is *cheap-by-default
on GreptimeDB, cheap-only-if-configured on ClickHouse*. It does **not** flip the
overall verdict — it is one cost lever, and a competent ClickHouse operator sets these
— but it is a real default-behavior edge for GreptimeDB and an operational gotcha for
ClickHouse, both mechanism-confirmed in source.

## Retention cost framing ($)

Expiry write-amp (above) is one input to retention cost; the standing $ bill has
three drivers, and this note is the canonical home for how they interact:

1. **Retained bytes × $/GB-month.** Set by TTL window × ingest rate × compression.
   Compression is a per-signal wash (`compression-and-cost.md`), so the lever is the
   TTL window, not the engine.
2. **Per-request GET/PUT/LIST cost.** Dominated by object *count* and query shape.
   GreptimeDB writes few large objects (4 per 1M spans, Run 9) vs ClickHouse's
   one-object-per-column-per-part — measured object-count edge to GreptimeDB; cold
   GET *counts* split by query shape (`caching-and-cold-warm.md`, Runs 14–15).
3. **Expiry write-amp.** The mechanism in this note: ~0 for GreptimeDB (whole-SST
   drop) and for tuned ClickHouse; ∝ surviving rows for default ClickHouse. Matters
   most at high churn (short TTL on high-volume signals — exactly Parallax's spans/logs).

The often-cited "**~50–100× cheaper than ingest-priced SaaS**" figure is a
**marketing-grade comparison vs SaaS observability pricing**, not a measured
GreptimeDB-vs-ClickHouse result (see `public-performance-claims.md` claim 8). Both
self-hosted engines get the object-store retention economics; it does **not**
separate them. Treat it as "object storage beats per-GB-ingested SaaS billing,"
directional only.

## Honest caveats

- **Both are background-gated, not instant.** GreptimeDB drops at the next compaction
  past TTL; ClickHouse at the next TTL merge (≥4h apart). Neither guarantees
  to-the-second eviction — relevant only if Parallax has a hard compliance-delete SLA
  (it doesn't, for telemetry).
- **TTL MOVE (tiering) is a separate axis** from TTL DELETE and is covered in
  `caching-and-cold-warm.md` / `compression-and-cost.md`: ClickHouse `TTL … TO DISK
  's3'` moves cold parts to object storage (a rewrite/move), whereas GreptimeDB is
  object-store-native and uses the read cache instead of explicit tiering. This note
  is about *deletion*, not tiering.
- **Not yet measured.** This is a source-confirmed mechanism teardown. A measured run
  (load past-dated data beyond TTL, trigger compaction/merge, observe bytes
  written/objects deleted) would quantify the write-amp gap — owed to the harness.

## Source / evidence

- GreptimeDB: `src/mito2/src/compaction.rs` (`find_dynamic_options:716`,
  `get_expired_ssts:1091`), `src/mito2/src/compaction/twcs.rs:219-224`
  (`get_expired_ssts` + mark-compacting), `src/mito2/src/compaction/compactor.rs:581`
  (`// Include expired SSTs in removals — these don't depend on merge success`).
- ClickHouse: `src/Storages/MergeTree/MergeTreeSettings.cpp:1669` (`merge_with_ttl_timeout`
  = 4h), `:1675` (`ttl_only_drop_parts` default `false`, with the row-vs-part doc text).
- Cross-refs: `compaction-and-merge.md` (TWCS vs SimpleMergeSelector),
  `compression-and-cost.md` (object-storage cost), `clickhouse-implementation.md`
  (DDL correction).
