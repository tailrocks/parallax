# Distributed Model and Scaling (Axis #3)

<!-- markdownlint-disable MD013 -->

Status: pass 10, mechanism re-verified live pass 110 (Run 74). The scaling axis —
single-node ceiling and **horizontal** scale-out (the operator's primary scaling
concern: startups on a tiny single node that grow to big-company horizontal scale as a
*topology change, not a rewrite*). Mostly architecture-reasoned from source + RFCs;
the single-node-checkable mechanism claims are now **runtime-confirmed (Run 74)**; the
multi-node *hold* (does p95 stay flat as nodes are added) is still harness-gated.

Pins: GreptimeDB `v1.0.2` (`0ef5451`), ClickHouse `v26.5.1.882-stable` (`5b96a8d8`).

## Single-node ceiling (vertical)

| | GreptimeDB | ClickHouse |
| --- | --- | --- |
| Vertical scaling | Good — Rust, async, uses cores; LSM + object store. | **Excellent** — a decade of C++/SIMD tuning; saturates many cores and NVMe; famously high single-box throughput. |
| Tier-1 single node | `standalone` binary, metrics-native. | One `clickhouse-server`, plain MergeTree. |
| Verdict | Both fine for a tiny single-node startup. **ClickHouse has the higher vertical ceiling** on one big box (raw scan/aggregate throughput). | |

So at the *low* end both are trivial; at the *one-huge-box* end ClickHouse scales
up further. The decision is about what happens **when one box is not enough**.

## Horizontal scale-out — designed-in vs bolted-on

| Aspect | GreptimeDB | ClickHouse (OSS) |
| --- | --- | --- |
| Shard unit | **Region** (a table is partitioned into regions by partition rules — range/multi-dim, `src/partition/src` `splitter`, `multi_dim`). | **Shard** (a manually-defined cluster of nodes); table is `Distributed` over shards. |
| Who places shards | **Metasrv** auto-places regions on datanodes and can migrate them (RFC `2023-11-07-region-migration`). | **You do** — sharding key + cluster topology are operator-defined, static. |
| Grow past current size | **Repartition** splits/merges regions as data grows (RFC `2025-06-20-repartition`); Metasrv rebalances. | **Manual resharding** — no automatic cross-shard rebalance in OSS (only intra-node JBOD balancing, `MergeTreeData.h:1183`). Growing shard count = manual data redistribution. |
| Compute/storage separation | **Yes, practical**: object store (OpenDAL) + remote WAL (Kafka) make datanodes hold little durable local state → regions migrate cheaply, datanodes are near-elastic. | **OSS: no.** `SharedMergeTree` (elastic, compute/storage-separated, auto-scaling) is **ClickHouse Cloud proprietary — not in OSS** (confirmed: no `SharedMergeTree` in `src/Storages`). OSS is shared-nothing shard+replica. |
| Replication / consensus | Metasrv (Raft-based metadata) + region replication; remote WAL via Kafka. | `ReplicatedMergeTree` + **ClickHouse Keeper (Raft)** per replica set. |
| Read fan-out | Frontend (stateless, scale freely) → `MergeScanExec` fans sub-plans to region datanodes (`src/query/src/dist_plan`). | `Distributed` engine fans the query to shards, merges on the initiator. Read scaling across replicas via **`max_parallel_replicas`** — still **experimental** in 26.x (`allow_experimental_parallel_reading_from_replicas=0` default). |

## Live mechanism re-verification (Run 74)

The single-node-checkable scale-out claims, confirmed at runtime on ClickHouse 26.5:

- **`SharedMergeTree` is not in OSS** — `CREATE TABLE … ENGINE=SharedMergeTree` →
  **`Unknown table engine SharedMergeTree (UNKNOWN_STORAGE)`**. So the elastic,
  compute/storage-separated, auto-scaling engine that *would* match GreptimeDB's
  object-store-native model is **Cloud-proprietary**; OSS scale-out stays shared-nothing
  shard+replica.
- **`ReplicatedMergeTree` requires a Keeper** — creating one without ZooKeeper/Keeper →
  **`Can't create replicated table without ZooKeeper (NO_ZOOKEEPER)`**. So OSS HA needs a
  separate Keeper/ZooKeeper coordinator stood up alongside.
- **Zero-copy replication is off by default + guard-railed** — `allow_remote_fs_zero_copy_replication
  = 0` (live), so OSS replicas each keep a **full** S3 copy by default (the 1× vs N× S3
  economics, Run 34). **Re-verified precisely (Run 91):** the zero-copy settings family is
  **present but none obsolete**, and wrapped in `disable_{detach,fetch,freeze}_partition_for_zero_copy_replication=1`
  guardrails (those ops are unsafe with shared data) — consistent with the source
  "not production-ready"/experimental flag. So **OSS default = N× S3** (each replica stores
  its own parts); ~1× requires enabling the experimental/guard-railed zero-copy. GreptimeDB
  = 1× native (object-store-shared, HA via metadata). The exact 2–3-replica byte measurement
  (B14) is the harness's; the deciding switch is confirmed off+guard-railed.
- **GreptimeDB here is `STANDALONE`** — `information_schema.cluster_info` reports one
  `STANDALONE` peer (all roles in one binary). The distributed Frontend/Datanode/Metasrv
  split + region rebalance is cluster-mode (multi-node), not exercised single-node — its
  *hold* is the harness-gated open question.

Net: the **OSS-ClickHouse-scale-out-is-manual** side of the verdict (no SharedMergeTree,
Keeper-coupled replication, N× S3 copies) is now runtime-confirmed, not just source-read;
GreptimeDB's designed-in region/Metasrv model still needs a multi-node run to confirm the
*hold* (below).

## The decisive difference for Parallax's growth path

The operator's requirement is **small→large as a topology change, not a rewrite**:

- **GreptimeDB was designed multi-node from the start.** The region is the unit of
  distribution; Metasrv places, migrates, and (via repartition) splits regions as
  load grows. The *same schema* runs on `standalone` (one node) and on a
  Frontend/Datanode/Metasrv cluster — you add datanodes and the control plane
  rebalances. With object storage + remote WAL, datanodes approach stateless, so
  scale-out is closer to "add capacity" than "re-architect." **This is exactly the
  topology-change-not-rewrite property the operator wants.**
  - **Migration mechanism confirmed in source (pass 34):** the region-migration
    procedure (`src/meta-srv/src/procedure/region_migration/`) is **flush_leader →
    downgrade_leader → open_candidate → upgrade_candidate → close_downgraded** — a
    sequence with **no bulk data-transfer step** (its elapsed-time metrics track
    only flush/downgrade/open/upgrade). It flushes the source region to make its
    data durable in shared storage, then the target datanode **opens the same region
    from storage (manifest + SSTs)** and takes over leadership. So migration is
    **ownership reassignment + reopen, not a data copy** — **cheap specifically when
    backed by shared object storage** (the SSTs are already in S3; the target lazy-
    loads them). On local-disk-only storage this property weakens (the target must
    obtain the files). This is the load-bearing mechanism behind cheap rebalancing.
- **ClickHouse OSS scale-out is operator-driven and front-loaded.** You must choose
  the shard count and sharding key up front; there is no automatic resharding in
  OSS, so outgrowing the initial shard layout is a manual, disruptive data-movement
  exercise — closer to a re-architecture than a topology tweak. The elastic answer
  (`SharedMergeTree`) exists only in ClickHouse Cloud, which is **out of scope**
  (proprietary, and the project's self-hosted/open posture + language filter rule
  out depending on a closed cloud engine).

→ **Scaling axis (#3) verdict: GreptimeDB wins the horizontal-scale dimension** —
the operator's stated primary — because scale-out is designed-in (region model +
Metasrv rebalancing + repartition + compute/storage separation), whereas OSS
ClickHouse requires manual sharding decided up front with a painful resharding
wall. **ClickHouse wins the single-node vertical ceiling.** Since the operator
ranks **horizontal first** and vertical-only is a flagged limitation, this axis
favors GreptimeDB for Parallax's startups→big-companies trajectory.

## Replication economics — the zero-copy gap (pass 57)

HA needs replicas; the question is **does each replica multiply the storage cost?**

- **ClickHouse OSS: yes, safely.** `ReplicatedMergeTree` is **shared-nothing** — each
  replica downloads and stores its **own full copy** of every part. On an S3 disk that
  means **N replicas → N× the S3 storage cost**. The fix, **zero-copy replication**
  (replicas share one S3 copy; only metadata + ref-counts replicate via ZooKeeper at
  `/clickhouse/zero_copy`), is **`allow_remote_fs_zero_copy_replication = false` by
  default** and the source itself warns: *"Don't use this setting in production,
  because it is not ready"* (`MergeTreeSettings.cpp:1955`, **EXPERIMENTAL**; live `0`,
  Run 34). Its fragility shows in the surrounding machinery — ZooKeeper-coordinated
  part-removal split/postpone locks, and `freeze`/`detach`/`fetch partition` **disabled**
  under it. The elastic, safe shared-storage engine (**`SharedMergeTree`**) is
  **ClickHouse Cloud-only** (above). So OSS ClickHouse HA on object storage realistically
  pays **N× storage**.
- **GreptimeDB: no zero-copy concept needed.** Object-store-native + compute/storage
  separation makes storage **inherently shared** — a region's SSTs live once in S3 and
  any datanode opens them (region migration = reopen-from-S3, no bulk copy, pass 34).
  Replication is **region leadership + Metasrv metadata (Raft) + remote WAL (Kafka)**,
  **not data duplication**. So HA does **not** multiply S3 storage; one copy backs the
  region, compute is near-stateless. The "1× storage + elastic compute" economics
  ClickHouse can only reach via an experimental flag (or Cloud) are GreptimeDB's
  **default architecture**.

→ Cost-axis (#2) + scaling-axis (#3) consequence: for **HA at scale on object storage**,
GreptimeDB's shared-storage model is both cheaper (1× vs N× S3) and simpler (no fragile
zero-copy coordination); OSS ClickHouse must choose between N× storage cost, the
not-production-ready zero-copy feature, or Cloud `SharedMergeTree`. Reinforces the
object-store-native advantage (`compression-and-cost.md`, `caching-and-cold-warm.md`)
on the replication dimension. Arch-reasoned + ClickHouse's own source warning (Run 34);
a real multi-replica S3 cost measurement is owed to the harness.

## Honest caveats

- **This is architecture-reasoned, not cluster-measured.** All Docker runs so far
  are single-node smoke. The region-rebalancing smoothness, repartition behavior
  under load, and `MergeScanExec` fan-out latency at scale are **not yet measured**
  — a real multi-node run (both engines) is owed before this is benchmark-confirmed.
- **ClickHouse horizontal scale is proven in production** at enormous scale (many
  shards) — it is not that ClickHouse *can't* scale out, but that doing so is
  operator-managed and resharding is the known pain point. For a fixed, well-sized
  cluster ClickHouse scales horizontally fine.
- **GreptimeDB's distributed maturity is younger.** Region migration/repartition
  are recent (2025 RFCs); their robustness under production churn is less battle-
  tested than ClickHouse's shard model. Designed-in ≠ as-hardened.
- Both use Raft for consensus (Metasrv metadata vs Keeper) — neither has a
  consensus-model disadvantage.

## What still needs a real cluster run

1. **Multi-node read fan-out latency**: `MergeScanExec` (GreptimeDB) vs
   `Distributed` (ClickHouse) on the evidence-bundle queries, 3+ nodes.
2. **Rebalance behavior**: add a datanode to GreptimeDB and observe region
   migration; compare to the manual ClickHouse resharding effort.
3. **Scale-out throughput hold**: does p95 hold as nodes are added (the harness
   scaling protocol)? Owed to `storage-benchmark-prototype.md`.

## Scaling-axis roll-up

| Sub-axis | Winner | Mechanism | Confidence |
| --- | --- | --- | --- |
| Single-node vertical ceiling | ClickHouse | decade-tuned C++ scan/aggregate | arch |
| Horizontal scale-out design | **GreptimeDB** | region model + Metasrv placement/migration + repartition; compute/storage separation via object store + remote WAL | arch |
| Resharding / topology change | **GreptimeDB** | automatic region rebalance vs ClickHouse OSS manual resharding wall | arch |
| Elastic compute/storage separation | **GreptimeDB** (OSS) | OpenDAL + remote WAL; ClickHouse `SharedMergeTree` is Cloud-only | confirmed (OSS absence) |
| Read parallelism across replicas | ~tie | ClickHouse `max_parallel_replicas` (experimental) vs GreptimeDB frontend fan-out | arch |
| **Replication storage economics** | **GreptimeDB** | Object-store-native = one shared S3 copy per region (HA via leadership/metadata, not data copy); OSS ClickHouse `ReplicatedMergeTree` stores N full copies (N× S3), zero-copy is **not-production-ready** + Cloud `SharedMergeTree` only | arch+source (Run 34) |

## Source / evidence

- GreptimeDB: `src/partition/src/{splitter,multi_dim,manager}.rs`; **region-migration procedure `src/meta-srv/src/procedure/region_migration/` (flush_leader → downgrade → open_candidate → upgrade → close; no bulk-copy step)**; RFCs `2023-11-07-region-migration`, `2025-06-20-repartition`, `2025-07-23-global-gc-worker`; dist plan `src/query/src/dist_plan` (`MergeScanExec`); remote WAL `src/log-store/src/kafka`.
- ClickHouse: `src/Storages/StorageDistributed.cpp`, `src/Storages/StorageReplicatedMergeTree.cpp`, `src/Coordination/Keeper*`; `max_parallel_replicas`/`allow_experimental_parallel_reading_from_replicas` (`src/Core/Settings.cpp:7308`); **no** `SharedMergeTree` in `src/Storages` (Cloud-only); **zero-copy replication** `allow_remote_fs_zero_copy_replication=false` + *"Don't use … not ready"* (`MergeTreeSettings.cpp:1955`, EXPERIMENTAL), `remote_fs_zero_copy_zookeeper_path=/clickhouse/zero_copy`, `disable_{freeze,detach,fetch}_partition_for_zero_copy_replication`.
