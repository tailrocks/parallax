# Ops Burden & Upgrade Surface — GreptimeDB vs ClickHouse (self-host)

<!-- markdownlint-disable MD013 -->

Status: **Run 558 (2026-07-18)** — closes the *engine-layer* half of gap-ledger
item **#7** (operational-complexity full picture). Not a practiced multi-node
fire-drill; not an FTE cost model. Pins: GT **`v1.1.3`** /
nightly **`v1.2.0-nightly-20260713`** (reports `1.2.0`) / CH **`26.6.1.1193`** /
head **`26.7.1.1097`**.

> **Product authority unchanged:** Parallax ships **self-hostable GreptimeDB +
> Turso**. ClickHouse is comparator only. This note scores *what an operator
> must run and watch* for each engine, not whether to flip the stack.

Companion notes:

- Scale-out mechanism: [`distributed-and-scaling.md`](distributed-and-scaling.md)
- Backup primitives: [`backup-and-disaster-recovery.md`](backup-and-disaster-recovery.md)
- Product RPO domains: [`product-rpo-runbook.md`](product-rpo-runbook.md)
- Managed erases ops: [`managed-cloud-vs-self-host.md`](managed-cloud-vs-self-host.md)

## Why this axis matters

The operator’s anti-complexity goal (anti-self-hosted-Sentry) makes **on-call
surface** co-equal with raw query speed. Engine internals research already
showed GT wins 1× object-store HA design and CH wins raw scan/agg; this note
asks: **what breaks at 3 a.m., and how many moving parts must stay healthy?**

## Topology inventory (what you actually run)

| Component | GreptimeDB self-host | ClickHouse OSS self-host |
| --- | --- | --- |
| **Tiny / dev** | One **`standalone`** binary (Frontend+Datanode+Metasrv fused) | One **`clickhouse-server`** (MergeTree, no Keeper) |
| **HA write path** | Cluster: Frontend × N + Datanode × N + **Metasrv** (external **etcd / Postgres / MySQL**) + optional **Kafka remote WAL** + **object store** | **ClickHouse Keeper** (or ZooKeeper) + ≥2 **replicas** (`ReplicatedMergeTree`) + shards as needed |
| **Durable data plane** | Prefer **S3-native SSTs** (OpenDAL); local disk = cache/dev | **Local/attached parts** first; S3 = cold tier or experimental zero-copy (off by default) |
| **Elastic compute/storage** | Designed-in (regions reopen from shared store) | **SharedMergeTree** = **Cloud-only** (not in OSS) |
| **Product metadata (Parallax)** | **Turso** (separate domain — always) | N/A for engine compare; Parallax still needs Turso |

### Live re-verify (Run 558, four-way bench containers ~2.5h up)

| Claim | Live result | Implication |
| --- | --- | --- |
| GT is standalone in harness | `information_schema.cluster_info` → **1 peer `STANDALONE`**, version `1.1.3` / `63ef18a`, uptime ~2.5h | Multi-node Metasrv/region path **not** exercised here |
| CH has no Keeper in harness | `CREATE … ENGINE=ReplicatedMergeTree(...)` → **Code 225 `NO_ZOOKEEPER`** | OSS HA **requires** a separate coordinator process |
| SharedMergeTree not OSS | `ENGINE=SharedMergeTree` → **Code 56 `UNKNOWN_STORAGE`** (26.6.1) | Elastic CH model stays Cloud-proprietary |
| `system.replicas` empty | `count() = 0` | Single-node MergeTree only in harness |
| Meta CLI backends listed | `etcd-store`, `postgres-store`, `mysql-store`, **`raft-engine-store`**, `memory-store` | Enum includes standalone raft, but… |
| Standalone D2 snapshot | `meta snapshot save --backend raft-engine-store --store-addrs /greptimedb_data/metadata` → **`Failed to parse url` / `RelativeUrlWithoutBase`** | **Re-confirms Run 405:** CLI snapshot path is for **URL-shaped external stores**, not the embedded raftlog dir |
| Standalone meta on disk | `/greptimedb_data/metadata/0000000000000001.raftlog` present | Tiny-tier D2 = **filesystem-consistent copy** of metadata (+ data/WAL), not `meta snapshot save` alone |
| Export CLI | `greptime cli data export-v2 {create,list,verify,delete}` present | Logical D1 portability still first-class |
| CH backup observability | `system.backups`, `system.merges`, `system.mutations`, `system.parts`, `system.replicas`, `system.replication_queue` | Richer **engine-native** ops tables than GT OSS exposes |

## On-call surface (failure modes, not full runbooks)

| Failure class | GreptimeDB | ClickHouse OSS |
| --- | --- | --- |
| **Single process death (tiny)** | Restart standalone; risk = local disk without S3 | Restart server; risk = local parts without replica |
| **Lost catalog** | SSTs orphaned without D2 meta; need snapshot **or** raftlog restore (standalone) | Keeper state + `system.replicas` recovery plane separate from `BACKUP TABLE` |
| **Lost object store / disk** | With S3-primary: bucket versioning is the recovery lever | Parts gone unless replica or BACKUP destination survives |
| **HA write unavailability** | Metasrv quorum + (optional) Kafka WAL + datanode region leadership | Keeper quorum + replica catch-up (`replication_queue`) |
| **Scale-out pain** | Add datanode → Metasrv places/migrates **regions** (ownership reopen from store — source: region-migration procedure) | Grow shards = **manual** reshard / rebalance; no OSS auto-reshard |
| **Compaction / merge storms** | TWCS/mito2 compaction (region-local); watch flush/WAL lag | MergeTree merges + mutations; `system.merges` / `system.mutations` |
| **Upgrade** | Bump binary/image; cluster needs coordinated Frontend/Datanode/Metasrv; avoid bare `v1.1.0` (JSON upgrade bug — use ≥`v1.1.1`, pin **`v1.1.3`**) | Bump server; Keeper compatibility matrix; replica rolling restart pattern well-documented in CH ops culture |
| **Auth / multi-tenant** | OSS coarse → **proxy-owned** (Run 172) | SQL grants, row policies, quotas, settings profiles (stronger OSS guardrails) |

**Parallax-specific always-on extras (both engines):** OTLP proxy, Turso (D3),
object-store credentials, and product API. Those are **not** neutralized by
picking CH or GT.

## Upgrade story (practical, pin-scoped)

| Axis | GreptimeDB | ClickHouse |
| --- | --- | --- |
| **Pin discipline** | Four-way rule: stable + dated nightly | Feature line (not LTS) + `head` |
| **Current pins (Run 558)** | Stable `1.1.3` / nightly `1.2.0` (`20260713`) | `26.6.1.1193` / head `26.7.1.1097` |
| **Breaking-note in research** | Do not ship `v1.1.0` alone | TimeSeries PromQL still experimental; outer SELECT Code 48 by design (Runs 403–404) |
| **Data migration** | Prefer S3-native + export-v2 for portability; schema-on-write (`greptime_identity`) reduces hand DDL | `BACKUP`/`RESTORE` SQL first-class; projections/indexes are operator DDL debt |
| **Nightly risk** | Dedup-agg regression historically scale-shaped (server 5M owed) | Head tracks feature line closely in this session; PromQL `increase` still **Code 48** on 26.6+26.7 |

## Ops burden scorecard (self-host, honest)

| Dimension | Winner | Because |
| --- | --- | --- |
| **Tiny single-node** | **Tie** | Both one process; GT standalone / CH plain MergeTree |
| **HA storage multiplier** | **GreptimeDB** | 1× shared object store vs N× replica parts (OSS default) |
| **HA process count** | **Slight CH or tie** | CH = server + Keeper; GT cluster = Frontend + Datanode + Metasrv backend (+ optional Kafka) — both multi-process |
| **Scale-out as topology change** | **GreptimeDB** | Region migration / repartition designed-in; CH OSS manual reshard |
| **Backup ergonomics (engine)** | **ClickHouse** | `BACKUP`/`RESTORE` SQL + `system.backup_log`; GT = export + meta snapshot (external store) |
| **Standalone meta backup** | **Neither is “one CLI”** | GT raft-engine snapshot CLI **broken for path addrs** (Runs 405/558); CH single-node has no Keeper state but also no replica |
| **Tenant guardrails OSS** | **ClickHouse** | Quotas / row policies / profiles (Run 172/179) |
| **Observability nativeness** | **GreptimeDB** | OTLP + PromQL + Jaeger OOB (Run 558: Jaeger/Prom **HTTP 200**; identity schema-on-write live) |
| **Ops culture / runbooks** | **ClickHouse** | Decade of production blogs, ClickStack, Cloud patterns |
| **Managed escape hatch** | **Both** | CH Cloud SharedMergeTree + cache; GT Fully-Managed from **$290/mo** (list; Run 558 re-fetch holds) |

**Net for Parallax product (self-host GT + Turso):** ops burden is **accepted**
in exchange for S3-native cost, region scale-out, and native OTEL surfaces.
CH’s lower *engine* ops culture debt and stronger SQL tenancy do **not**
authorize a stack flip; they set **upstream + proxy** work (authz, runbooks,
server-tier drills).

## Concurrent incidental re-verifies (Run 558)

| Check | Result |
| --- | --- |
| Pins | GT `1.1.3` / nightly `1.2.0` / CH `26.6.1.1193` / head `26.7.1.1097` — **no bump** |
| Four-way health | All four containers healthy (~2.5h+) |
| Managed list rates (desk) | GT floor **$290/mo**; CH Basic **$66.52** / Scale **$499.38** / Enterprise **~$2,669** examples; storage **$25.30/TB-mo** math — **no drift** vs Run 221/405 |
| `greptime_identity` | POST → table `run558_id` cols `[greptime_timestamp, level, msg, svc]` schema-on-write |
| Jaeger / Prom HTTP | **200** / **200** |
| last_value / argMax warm | GT ~**8 ms** (after 22 ms cold-ish); CH argMax ~**3 ms** — direction holds, interactive |
| CH PromQL `increase` | Still missing on prior matrix (not re-burned volume); rate path remains the supported CH subset |

## Still open after Run 558

1. **Practiced multi-node ops** — Metasrv etcd/RDS D2 snapshot+restore; CH Keeper+2-replica rolling upgrade.
2. **FTE / on-call hours model** — qualitative only here.
3. **Upgrade soak** — pin bump procedures with real retained TB.
4. Product/server gaps unchanged: workload mix A1–A7, server 1M/5M, trial quotes, GB cold S3, product Turso D3 schema.

**Do not declare comparison done.** Engine ops inventory is now documented;
production drills remain owed.

## Sources (primary / live)

- Live Docker: `parallax-bench-{greptimedb,greptimedb-nightly,clickhouse,clickhouse-head}-1` (Run 558).
- CLI: `greptime cli meta snapshot save --help` (backend enum + S3 flags); `export-v2` subcommands.
- Prior mechanism: Runs 74/91 (Keeper / SharedMergeTree / zero-copy), 174 (BACKUP/export), 405 (raft-engine URL fail), 172/179 (tenancy).
- Managed list: <https://greptime.com/pricing>, <https://clickhouse.com/docs/cloud/manage/billing/overview> (re-fetched Run 558).
