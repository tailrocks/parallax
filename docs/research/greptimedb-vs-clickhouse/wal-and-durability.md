# WAL and Durability — Crash Safety and the Scaling Enabler

<!-- markdownlint-disable MD013 -->

Status: pass 41, re-verified pass 105 (Run 69 — CH WAL settings `is_obsolete=1` live),
extended pass 111 (Run 75 — **strict-durability cost measured: GreptimeDB ~10× cheaper**) + **Run 146
(SOURCE-grounded: `log-store/src/raft_engine/log_store.rs` — `sync_write`/`sync_period`/`SyncWalTaskFunction`
→ `engine.sync()` over raft-engine's append-only `LogBatch` log; so GT strict-durable = ONE sequential
append-log fsync vs ClickHouse `fsync_after_insert=1` = whole-part (all column files + dir) fsync — the
~10× edge is architectural, not just a smoke number; `sync_period` is a group-commit middle ground, Kafka
remote WAL decouples durability off-datanode)**. White-box teardown of the **durability path** (checklist
#2's WAL/durability sub-item): what makes an acked write survive a crash, the
durability-vs-throughput knobs, and — the load-bearing part for Parallax —
**GreptimeDB's remote (Kafka) WAL as the compute/storage-separation enabler** behind
the horizontal-scaling story. Source-confirmed + live config-checked (Runs 20, 69).

Pins: GreptimeDB `v1.0.2` (`0ef5451`), ClickHouse `v26.5.1.882-stable` (`5b96a8d8`),
re-confirmed latest stable 2026-05-25.

## GreptimeDB — a real WAL, local or remote, tunable durability

Every write goes to a **WAL before the memtable** (`src/log-store`). Two providers:

- **Local: raft-engine** (`log-store/src/raft_engine.rs`; config
  `common/wal/src/config/raft_engine.rs`). Segmented append log (`file_size` 128 MiB,
  `purge_threshold` 1 GiB, `purge_interval` 60 s — purged once the data is flushed to
  SST). **Durability knobs:** `sync_write: bool` (fsync every WAL append) and
  `sync_period: Option<Duration>` (periodic group fsync). **Default `sync_write =
  false`** — so out of the box GreptimeDB does *not* fsync per write; it group-commits
  / relies on periodic + OS flush, trading strict durability for throughput (and you
  can set `sync_write=true` for fsync-on-every-write). **Live (Run 20, re-confirmed Run 69):** the running
  standalone holds `…/wal/00000000000000NN.raftlog` segments ~128–134 MiB each (Run 69:
  11 segments, ~1.4 GB total — grows with writes, purged after flush) — the local
  raft-engine WAL is active.
- **Remote: Kafka** (`common/wal/src/config/kafka/datanode.rs`). Each region's WAL is
  produced to Kafka (`max_batch_bytes` 1 MiB to fit Kafka's default message cap,
  `auto_create_topics`, topic/replication config). **Durability becomes Kafka's
  replication** (acked across brokers), *independent of the datanode's local disk*.

**Why the remote WAL matters beyond durability.** With the WAL in Kafka and SSTs in
object storage, a **datanode holds almost no durable local state**. That is exactly
what makes region migration cheap (pass 34: reopen region from object storage + replay
the Kafka WAL tail — no bulk data copy) and what gives the compute/storage separation
behind the horizontal-scaling verdict. The WAL choice is not just a durability dial;
it is the **scaling architecture**.

**Crash recovery:** on restart the region replays its WAL from the last flushed
sequence → no acked-and-memtable-resident write is lost (to the durability level the
sync knob guarantees). The WAL is the replay log.

## ClickHouse — no MergeTree WAL; durability = part on disk (+ replicas)

ClickHouse MergeTree has **no active write-ahead log**. The old in-memory-parts WAL
(`in_memory_parts_enable_wal`, `write_ahead_log_*`) is **obsolete** in 26.x
(`MergeTreeSettings.cpp:2214-2218` `MAKE_OBSOLETE_MERGE_TREE_SETTING`). **Re-verified
live (Run 69):** `system.merge_tree_settings` reports `in_memory_parts_enable_wal` and
`write_ahead_log_max_bytes` with **`is_obsolete = 1`**; active parts are only `Compact`
(39) / `Wide` (20) — **no `InMemory` part type** — and a `find` for `*wal*` under
`/var/lib/clickhouse` returns **nothing**. So the WAL machinery is a dead vestige, not a
functional log. Durability is the **part write itself**, and fsync is **off by default**:

- `fsync_after_insert = false` (`MergeTreeSettings.cpp:606`; doc: *"Significantly
  decreases performance of inserts"*) — the new part is written to the OS page cache
  and **not fsynced**.
- `fsync_part_directory = false` (`:610`).
- **Live (Run 20):** both `0` on the running server.
- **Async insert (live):** `async_insert = 1` (default on in 26.x) +
  `wait_for_async_insert = 1` — the client ack waits until the server buffer is
  flushed to a part, but that part is still **not fsynced**. With
  `wait_for_async_insert = 0` the ack returns *before* the part exists → an explicit
  data-loss window.

**Crash recovery:** there is **no replay log**. A part that was acked but not yet
flushed to disk by the OS is simply **lost** on power failure. ClickHouse's real
durability answer is **`ReplicatedMergeTree` + Keeper (Raft)** — redundancy across
replicas, not a local WAL. Single-node OSS ClickHouse with default fsync is the
*least* durable config of the two engines.

## Side by side

| | GreptimeDB | ClickHouse (MergeTree) |
| --- | --- | --- |
| Write-ahead log | **Yes** — raft-engine (local) or Kafka (remote) | **No** (in-memory-parts WAL obsolete) |
| Default per-write fsync | **No** (`sync_write=false`; tunable to true) | **No** (`fsync_after_insert=false`) |
| Durable-on-ack out of the box | WAL-appended (replayable), not fsynced by default | part in page cache, not fsynced; async ack waits for part (`wait_for_async_insert=1`) |
| Strict durability option | `sync_write=true` (fsync each WAL record) | `fsync_after_insert=1` (slow) |
| Crash recovery | **Replay WAL** from last flushed sequence | **No replay** — unflushed parts lost; rely on replicas |
| Redundancy model | region replication + remote WAL (Kafka) | `ReplicatedMergeTree` + Keeper |
| Durability decoupled from compute node | **Yes (Kafka WAL + object-store SSTs)** → cheap migration / elastic datanodes | No — local disk or replica-coupled |

## Strict-durability ingest cost — measured (Run 75, B15)

The sync knobs above have a *throughput* cost, and it differs sharply by mechanism.
Measured the per-write **delta** of turning strict durability on (docker/overhead cancels):

| Engine | strict knob | fsync delta | what is fsynced |
| --- | --- | --- | --- |
| **GreptimeDB** | `sync_write=true` | **~+1.7 ms/write (~3%)** | one **sequential WAL append** (raft-engine log) |
| **ClickHouse** | `fsync_after_insert=1` (+`fsync_part_directory=1`) | **~+18 ms/part (~20%)** | the whole **part** — its column files + the directory |

→ **Strict-durable ingest is ~10× cheaper on GreptimeDB.** The WAL is not only a *replay*
advantage; it is a **strict-durability *throughput* advantage** — fsyncing one append-only
log record is far cheaper than fsyncing a multi-file part. So if a Parallax tier needs
no-loss-on-crash ingest, GreptimeDB runs fsync-on-write at ~3% cost; ClickHouse's realistic
no-loss answer stays **replica redundancy** (`ReplicatedMergeTree` + Keeper), not per-part
fsync (which costs ~20% and still isn't a replay log). (orbstack overlay-fs inflates both
absolutes; the *ratio* — sequential-append fsync ≪ whole-part fsync — is architectural.)

## Parallax implication and axis consequence

- **Durability (axis #1, the "see real data" axis's safety side):** both default to
  *throughput over strict fsync*, so neither is strictly durable-on-ack out of the
  box. But **only GreptimeDB has a replayable WAL** — a single-node GreptimeDB
  recovers acked memtable writes after a crash; single-node default ClickHouse loses
  unflushed parts. For Parallax evidence bundles (you don't want to silently lose the
  spans/logs around an incident), GreptimeDB's WAL + tunable `sync_write` is the more
  robust default; matching it on ClickHouse means replication or `fsync_after_insert`
  (a measured perf hit).
- **Scaling (axis #3):** the **Kafka remote WAL is the mechanism** that makes
  GreptimeDB datanodes near-stateless and region migration a copy-free reopen — the
  concrete enabler of the "topology change, not rewrite" scaling verdict
  (`distributed-and-scaling.md`). ClickHouse OSS has no analog; durability and data
  are local-disk/replica-coupled, which is why resharding is heavy.
- **Freshness (axis #1):** unchanged — both visible-on-write (memtable vs part
  commit), already a tie (`write-path-and-ingestion.md`). The WAL adds durability
  *before* visibility on GreptimeDB without delaying it.

Net: this is a **durability + scaling edge to GreptimeDB**, mechanism-confirmed. It
does not touch query speed; it reinforces the freshness/ingest-ergonomics and
horizontal-scaling pillars the verdict already rests on. ClickHouse's
durability-via-replication is perfectly sound at cluster scale — the gap is
single-node default safety and the absence of a Kafka-style durability/compute
decoupling.

## Honest caveats

- Both defaults are tunable; this compares **out-of-the-box** behavior plus the
  available knobs, not a tuned-vs-tuned worst case.
- Not latency-measured: the throughput cost of `sync_write=true` vs
  `fsync_after_insert=1` (the strict-durability configs) is not benchmarked here —
  owed to the harness if Parallax needs strict per-write durability.
- Kafka WAL adds an operational dependency (a Kafka/Redpanda cluster) — it is the
  *distributed* durability/scaling path, not needed for a Tier-1 single node (which
  uses the local raft-engine WAL, as the live bench does).
- ClickHouse's design is deliberate (OLAP, bulk-insert, durability via replicas) — "no
  WAL" is a fit choice for its target workload, not an oversight.

## Source / evidence

- GreptimeDB: `src/log-store/src/{raft_engine.rs,kafka.rs}` (providers);
  `src/common/wal/src/config/raft_engine.rs` (`sync_write` default false, `sync_period`,
  `file_size`/`purge_*`); `src/common/wal/src/config/kafka/datanode.rs`
  (`max_batch_bytes`, topic/replication). Live: `…/wal/*.raftlog` segments.
- ClickHouse: `src/Storages/MergeTree/MergeTreeSettings.cpp:606` (`fsync_after_insert`
  false), `:610` (`fsync_part_directory` false), `:2214-2218` (in-memory-parts WAL
  obsolete). Live: `system.merge_tree_settings` (both fsync `0`), `system.settings`
  (`async_insert=1`, `wait_for_async_insert=1`).
- Empirical: `local-benchmark-results.md` Run 20 (WAL files + fsync defaults), Run 69 (CH WAL settings `is_obsolete=1` live), **Run 75 (B15 strict-durability cost: GreptimeDB `sync_write=true` ~+1.7 ms/write vs ClickHouse `fsync_after_insert=1` ~+18 ms/part → ~10× cheaper)**.
- Cross-refs: `write-path-and-ingestion.md` (freshness, small-write absorption),
  `distributed-and-scaling.md` (region migration, compute/storage separation),
  `greptimedb-internals.md` (WAL in the write path).
