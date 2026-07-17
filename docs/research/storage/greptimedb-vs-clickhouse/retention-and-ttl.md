# Retention and TTL — How Old Telemetry Expires (Cost Axis #2)

<!-- markdownlint-disable MD013 -->

Status: **pass 82 recheck 2026-07-17** — mechanism still holds on current stables;
prior history = pass 36 + pass 100 (Run 64 live CH paths + GT read-time filter) +
Run 111 (CH wholly-expired part drop; GT purge eventual) + Run 144 (TWCS structural
whole-SST drop @ `v1.0.2`). White-box teardown of **TTL expiry** — *when* old data
drops and *what it costs*. First-class lever for observability: Parallax retains
signals on TTL windows; at steady state dominant background cost is **expiry**, not
ingest. Question is not "can it TTL" but **"does expiry rewrite survivors or drop
whole files?"** (write-amp + object churn). Product lifecycle contract (logical prune
vs physical reclaim) lives in
[retention-and-prune-contract.md](../../decisions/retention-and-prune-contract.md);
sized $/GB gate in [size-and-object-cost.md](../size-and-object-cost.md).

### Pass 82 recheck (2026-07-17) — pins + primary sources

| Pin / surface | Pass 82 check | Verdict |
| --- | --- | --- |
| GreptimeDB stable | GitHub latest release **`v1.1.3`** (published 2026-07-17) | **Current.** Prior note pin `v1.0.2` is **historical** (Runs 64/111/144). |
| ClickHouse feature line | Source tag **`v26.6.1.1193-stable`** (agenda pin; not LTS) | **Current for comparator.** Prior `v26.5.1.882-stable` was earlier line. |
| GT docs TTL (v1.1) | [Manage data — TTL policies](https://docs.greptime.com/user-guide/manage-data/overview/#manage-data-retention-with-ttl-policies) | Table + DB `WITH ('ttl'=…)` / `ALTER … SET/UNSET 'ttl'`; values duration / `instant` / `forever`. **Expired rows deleted during compaction (async background), not immediately** — explicit doc callout. |
| GT source TTL drop @`v1.1.3` | `src/mito2/src/compaction/twcs.rs` (~245–250): still calls `get_expired_ssts`, marks expired SSTs compacting; `compactor.rs` still **`// Include expired SSTs in removals — these don't depend on merge success`** (~600) and tests `test_expired_ssts_always_removed` | **Mechanism unchanged** vs Run 144: whole-SST removal independent of merge success. |
| CH source defaults @`v26.6.1.1193-stable` | `MergeTreeSettings.cpp`: `merge_with_ttl_timeout = 3600 * 4` (4h); **`ttl_only_drop_parts = false`** default; docs still row-level when disabled | **Defaults unchanged** vs earlier note. |
| Parallax product contract | [retention-and-prune-contract.md](../../decisions/retention-and-prune-contract.md) (approved 2026-07-17) | Already aligned: prune reports logical reclaim separately from physical bytes pending GT compaction/GC. |

**Kept claims (strengthened, not flipped):**

1. GreptimeDB physical TTL reclaim is **compaction-gated** (docs + source) — not a
   synchronous DELETE at expiry second.
2. Cheap physical drop remains **whole expired SSTs** via TWCS + `get_expired_ssts`
   (source @`v1.1.3`).
3. ClickHouse default remains **row-level TTL rewrite** unless
   `ttl_only_drop_parts=1` + time-aligned parts/partitions (source defaults still
   false/4h).
4. Live magnitude numbers from Runs 17/64/111 remain **historical smoke on older
   pins** — **not re-measured this pass** (no benchmark agent run). Treat write-amp
   *magnitude* at production volume as **unproven** on `v1.1.3` / `26.6.x`.

**Product windows (contract, not this engine note):** default product TTLs in the
prune contract are shorter than the old research example windows (traces/logs
`7d`, metrics `14d` in the decision matrix) — engine mechanism is independent of
the numeric window.

**Falsify this pass:** GreptimeDB removes TTL via row-rewrite merges by default;
docs claim synchronous physical delete; CH flips `ttl_only_drop_parts` default to
true; or a live re-measure on `v1.1.3` shows whole-SST drop abandoned.

Historical pins (Runs 64/111/144 era): GreptimeDB `v1.0.2`, ClickHouse
`v26.5.1.882-stable` — mechanism evidence, not current product pins.

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

### Measured live — both ClickHouse merge paths observed (Run 64, re-verifies + refines Run 17)

On a default-TTL table (`ttl_only_drop_parts=0`, `merge_with_ttl_timeout=0`), `part_log`
shows ClickHouse takes **two different paths depending on whether a part is wholly or
partially expired**:

- **Wholly-expired part → `TTLDropMerge`, `read_rows` small, `rows=0` written** — a whole
  part of all-expired rows is **dropped wholesale, no survivor rewrite**, *even at default
  settings*. (When expired and alive rows land in *separate* parts — the time-ordered
  ingest case — old parts age out cheaply.)
- **Mixed expired+alive part → `TTLDeleteMerge`, `read_rows: 1,000,000`, `rows: 500,000`** —
  a part straddling the TTL cutoff is **read in full and rewritten with only the 500k
  survivors** → write-amplification ∝ survivors, exactly as the row-level mechanism predicts.

So the refinement to Run 17: ClickHouse's TTL rewrite penalty bites **only on
boundary/mixed parts**, not on all expiry — wholly-expired parts drop cheap regardless.
Whether parts are wholly-vs-partially expired depends on time-alignment, which is exactly
what `PARTITION BY` time fixes. **GreptimeDB sidesteps this entirely**: TWCS time-windows
SSTs so expiry is whole-SST by construction (no mixed SST to rewrite), **and** its TTL is
also a **read-time filter** — in Run 64 a 500k-row load with year-old timestamps (past a
`ttl='1h'`) showed **0 live rows immediately, before any compaction** (expired rows are
filtered at read/flush, not waiting for the drop). ClickHouse expired rows remain
physically present (and queryable without `FINAL`-like filtering) until the TTL merge runs.

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

- **Background-gated, but the first eviction is prompt.** Both physically drop on a
  background pass, not to-the-second. But `merge_with_ttl_timeout`=4h is a *repeat*
  floor (re-checking the same data), **not** an initial delay — Run 17 saw ClickHouse
  evict within seconds of insert. GreptimeDB additionally filters expired rows on the
  **read path** immediately and drops already-expired rows at **flush**, so query
  results never show expired data even before the compaction drop. Relevant only if
  Parallax had a hard compliance-delete SLA (it doesn't, for telemetry).
- **TTL MOVE (tiering) is a separate axis** from TTL DELETE and is covered in
  `caching-and-cold-warm.md` / `compression-and-cost.md`: ClickHouse `TTL … TO DISK
  's3'` moves cold parts to object storage (a rewrite/move), whereas GreptimeDB is
  object-store-native and uses the read cache instead of explicit tiering. This note
  is about *deletion*, not tiering.
- **Measured (Run 17, smoke).** ClickHouse `system.part_log`: default TTL =
  `TTLDeleteMerge` read 1M rows / rewrote 500k survivors (50 MiB written) to evict
  half; tuned (`ttl_only_drop_parts=1`+partition) = `TTLDropMerge`, 0 rows rewritten.
  GreptimeDB `ttl='5s'`: 1 SST → 0 after aging + `ADMIN compact_table` (Parquet
  deleted, no rewrite file). The mechanism is confirmed numerically; the write-amp
  *magnitude at production volume + sustained churn* is still the prototype's to settle.

## Source / evidence

- **Pass 82 (2026-07-17), current pins:**
  - GreptimeDB **`v1.1.3`** source: `src/mito2/src/compaction/twcs.rs` (`get_expired_ssts`
    + mark-compacting ~245–250), `src/mito2/src/compaction/compactor.rs` (~600:
    expired SSTs in removals independent of merge success; `test_expired_ssts_always_removed`).
  - GreptimeDB docs **v1.1**: [Manage data retention with TTL](https://docs.greptime.com/user-guide/manage-data/overview/#manage-data-retention-with-ttl-policies)
    (compaction-async delete; table/DB TTL; `ALTER SET/UNSET`).
  - ClickHouse **`v26.6.1.1193-stable`**: `MergeTreeSettings.cpp` —
    `merge_with_ttl_timeout = 3600*4`, `ttl_only_drop_parts = false` (row-level default).
  - Product: [retention-and-prune-contract.md](../../decisions/retention-and-prune-contract.md).
- **Historical line numbers (Runs 64/111/144 @`v1.0.2` / CH 26.5):**
  `compaction.rs` `get_expired_ssts`, `twcs.rs` mark-compacting, `compactor.rs`
  removals comment — same control flow re-found on `v1.1.3` at shifted lines.
- Cross-refs: `compaction-and-merge.md`, `compression-and-cost.md`,
  `clickhouse-implementation.md` (DDL correction), `size-and-object-cost.md`.

## Run 187 (2026-07-17) — TTL live on v1.1.3 / 26.6.1.1193

| Engine | Setup | Result |
| --- | --- | --- |
| GT | `WITH (append_mode='true', ttl='1s')`, insert, sleep 2, flush, **`ADMIN compact_table`** | count **1 → 0** after compact |
| CH | `TTL ts + INTERVAL 1 SECOND`, insert past+future rows, **`OPTIMIZE FINAL`** | expired row dropped; remaining `[2]` |

**No drift:** both expire data; GT TTL purge is **compaction-triggered** (background/eventual until
compact), CH boundary/part drop on OPTIMIZE/merge. Cheap retention still favors time-ordered
ingestion + whole-file/part drop (TWCS on GT).

## Run 430 (2026-07-18) — TTL expire live re-verify

| Engine | Setup | Result |
| --- | --- | --- |
| GT v1.1.3 | `ttl='1s'`, insert aged+fresh, `ADMIN FLUSH_TABLE` + `COMPACT_TABLE` | Eventually **0** rows (expired); memtable may briefly hold until compact |
| CH 26.6 | `TTL ts + INTERVAL 1 SECOND`, insert aged+fresh, `MATERIALIZE TTL` + `OPTIMIZE FINAL` | **2 → 1** (fresh kept) |

**No drift** vs Run 187/253: both engines expire; GT needs flush/compact for SST drop;
CH needs MATERIALIZE/OPTIMIZE for immediate physical drop. Query may still see
logical rows until maintenance.

