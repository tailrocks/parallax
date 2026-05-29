# Deletes and Mutations — Corrections, GDPR-Erase, Updates

<!-- markdownlint-disable MD013 -->

Status: pass 51, re-verified + sharpened pass 102 (Run 66) + **re-verified Run 167 (exec, no drift):
GreptimeDB `DELETE FROM … WHERE k=…` (affectedrows 1); ClickHouse both `ALTER … DELETE` (sync mutation,
part rewrite) and lightweight `DELETE FROM` (GA mask) remove the row. **GDPR-compliance nuance:** by
default *both* delete **logically** first (GreptimeDB tombstone → read-filter; ClickHouse lightweight
`_row_exists=0` mask → read-filter), with **physical** removal deferred to compaction/merge. So for a
*guaranteed physical-erasure deadline* (right-to-be-forgotten) you must **force** it — GreptimeDB
`ADMIN compact_table` (or wait for TWCS), ClickHouse `OPTIMIZE … FINAL` / heavy `ALTER DELETE` (rewrites
immediately). The data is query-invisible instantly on both; physically gone only after that step. Plan
the redaction path to force compaction, not rely on the default lazy purge.** The "mutations"
ClickHouse system-lead + the delete path on both —
i.e. how each engine handles **deleting and updating already-written data**.
Decision-relevant for Parallax: append-mostly, but occasionally needs a **GDPR erase**
(remove one user's telemetry), a **bad-batch correction**, or an **issue-status
update**. Source + live (Run 29). Connects the dedup (pass 39) and retention/TTL
(pass 36–37) themes — delete is the row-level cousin of TTL.

Pins: GreptimeDB `v1.0.2` (`0ef5451`), ClickHouse `v26.5.1.882-stable` (`5b96a8d8`),
re-confirmed latest stable 2026-05-25.

## GreptimeDB — LSM-native: delete = tombstone, update = upsert

- **DELETE** writes a **tombstone (delete marker)** into the LSM; the read-path
  `DedupReader.filter_deleted` (pass 39) drops the marked `(key, ts)` at every read;
  compaction physically purges it later. **Live (Run 29):** `DELETE FROM gt_del WHERE
  k='b'` → the row was **immediately** gone from queries (`['a','c']`), **no compaction
  forced**. A delete is a cheap LSM write + a read-time filter, not a part rewrite.
- **UPDATE** = re-insert the same `(primary key, ts)` with new values → the dedup
  strategy (`last_row`/`last_non_null`, pass 39) makes the latest write win at read. So
  an update is just an **upsert** — another cheap LSM write, correct at read, GA, no
  special setup. **Precise (Run 66): there is NO `UPDATE` DML statement** — `UPDATE … SET
  … WHERE` returns *"SQL statement is not supported"*. The overwrite is **(PK, ts)-keyed**:
  re-inserting `(id=1, ts=1000, 'sameTS')` overwrote the row (→ `sameTS`), but re-inserting
  with a **new** ts `(id=1, ts=2000, 'newTS')` produced **two versions** (`[1000,'sameTS'],
  [2000,'newTS']`) — time-series semantics, not a relational in-place update. So for a
  Parallax current-state signal (issue status), "update" means **either** re-write the same
  `(PK, ts)` **or** append a new ts and query the latest (`MAX(ts)` / dedup) — never an
  `UPDATE` statement. This is simpler than ClickHouse's mutation but is an append/upsert
  model, not row-update.

Both correction operations ride the same LSM + read-dedup machinery as ordinary writes
and TTL — uniformly cheap, immediate at read, purged at compaction.

## ClickHouse — mutations, with a lightweight delete (and an experimental update)

ClickHouse's historical answer was the **heavy mutation**: `ALTER TABLE … DELETE/UPDATE
WHERE` asynchronously **rewrites every matching part** (read + write surviving
rows/columns) — write-amp ∝ data touched. Two lighter paths now exist:

- **Lightweight `DELETE FROM` — GA-ish, default-on.** **Live (Run 29):** `DELETE FROM
  del_test WHERE id<50000` on a plain table → `system.mutations` shows
  **`UPDATE _row_exists = 0 WHERE id<50000`** and the part bumped `all_1_1_0` →
  `all_1_1_0_2`. So it writes/updates a hidden **`_row_exists` mask** (not a full
  surviving-row rewrite), filters masked rows at read, and purges them at the next
  merge. `lightweight_deletes_sync=2` default. This is a real cheap-ish delete —
  ClickHouse has **caught up** on the delete side.
- **Lightweight `UPDATE` — experimental + per-table setup.** `enable_lightweight_update`
  and `allow_experimental_lightweight_update` are **=1 by default**, but **live (Run
  29)** a plain `UPDATE upd_test SET v='new' …` was **rejected**: *"Lightweight updates
  are supported only for tables with materialized `_block_number` column. Run 'MODIFY
  SETTING enable_block_number_column = 1'."* So the lightweight update needs a per-table
  opt-in column and is gated experimental; without it, `UPDATE` falls back to the
  **heavy `ALTER UPDATE` mutation (full part rewrite)**.

## Side by side

| Operation | GreptimeDB | ClickHouse |
| --- | --- | --- |
| DELETE | tombstone (LSM) → read-filter (`filter_deleted`) → compaction purge | lightweight `DELETE FROM` = `_row_exists=0` mask (GA, default) → read-filter → merge purge; or heavy `ALTER DELETE` rewrite |
| UPDATE | **no `UPDATE` statement** (Run 66); re-insert same `(PK,ts)` → dedup last-wins (**upsert, GA, cheap**); new ts = new version | `ALTER UPDATE` = **full part rewrite**; lightweight `UPDATE` exists but **experimental + needs `enable_block_number_column=1`** (live-rejected, Run 29/66) |
| Cost (delete) | cheap write + read-filter | cheap-ish mask mutation (lightweight) / rewrite (heavy) |
| Cost (update) | **cheap upsert** | **rewrite** (or experimental lightweight w/ setup) |
| Correct-now at read | yes | lightweight delete yes (mask); heavy mutation async |
| Setup needed | none | lightweight update: per-table block-number column |

## The honest result — delete parity, update favors GreptimeDB

- **DELETE ≈ parity.** ClickHouse's lightweight `DELETE FROM` (mask, default-on) gives
  it a cheap, read-immediate delete comparable to GreptimeDB's tombstone — the old
  "ClickHouse deletes are expensive rewrites" critique is **softened** (same
  drift-correction flavor as the PromQL finding, pass 44). Both filter at read and
  purge at merge/compaction. GreptimeDB's is LSM-native (a write); ClickHouse's is a
  mask mutation (part version bump) — mechanically different, practically similar for a
  GDPR erase.
- **UPDATE favors GreptimeDB.** GreptimeDB updates via a **GA, zero-setup upsert**
  (re-insert + dedup); ClickHouse's GA path is the **heavy rewrite mutation**, and its
  lightweight update is **experimental + requires a per-table block-number column**. For
  correction/issue-status-update workloads, GreptimeDB's model is clearly cheaper and
  simpler.

## Axis consequence

- **Cost (axis #2):** for Parallax's correction shapes, GreptimeDB is uniformly cheap
  (tombstone delete + upsert update, all LSM writes); ClickHouse is cheap on
  lightweight delete but pays a rewrite on (GA) update. Updates are the divergence.
- **Speed/operability (axis #1 + ops):** both make a delete visible immediately at read
  (GreptimeDB tombstone filter; ClickHouse `_row_exists` mask). GreptimeDB needs no
  per-table setup for either op; ClickHouse needs opt-in for lightweight update.
- **Net:** reinforces the LSM-native ergonomics theme (dedup, retention, now
  delete/update) — a modest GreptimeDB fit edge on corrections, **strongest on UPDATE**.
  Doesn't move the verdict (Parallax is append-mostly; corrections are occasional), but
  it's a real, mechanism-grounded ergonomics point, and it honestly credits ClickHouse's
  lightweight-delete catch-up.

## Honest caveats

- ClickHouse's lightweight update is **actively maturing** (settings already default-on,
  just gated on a per-table column) — like PromQL, re-check on version bumps; the
  UPDATE-rewrite gap may narrow.
- Not latency-measured at volume: the heavy-mutation rewrite cost vs lightweight-mask
  cost vs GreptimeDB tombstone cost at GB scale is a `benchmarking-the-differences.md`
  candidate if Parallax's correction volume ever matters.
- GreptimeDB tombstones/upserts still cost compaction work to purge (the write-amp
  shows up later at compaction, as with dedup) — "cheap write" ≠ "free."

## Source / evidence

- GreptimeDB: `DELETE FROM` → tombstone + `DedupReader.filter_deleted`
  (`src/mito2/src/read/dedup.rs`, pass 39); UPDATE = upsert via `merge_mode`. Live
  (Run 29): delete immediate at read, no compaction. Live (Run 66): DELETE parity
  re-verified (CH lightweight 100k→50k mask vs GT tombstone, both read-immediate); **no
  `UPDATE` statement ("SQL statement is not supported"); (PK,ts) overwrite confirmed —
  same ts overwrites, new ts = new version**.
- ClickHouse: lightweight `DELETE FROM` → `system.mutations` `UPDATE _row_exists=0`,
  part `_2` bump (Run 29); `lightweight_deletes_sync=2`; `enable_lightweight_update=1` +
  `allow_experimental_lightweight_update=1` but requires `enable_block_number_column=1`
  (live rejection, Run 29); heavy `ALTER UPDATE/DELETE` = part-rewrite mutation.
- Cross-refs: `dedup-and-update-semantics.md` (filter_deleted, upsert/merge_mode),
  `retention-and-ttl.md` (TTL = whole-SST drop, the bulk cousin),
  `promql-and-metrics-query.md` (same experimental-catch-up pattern).
