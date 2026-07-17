# Managed Cloud vs Self-Host — Cost & Ops Calculus

<!-- markdownlint-disable MD013 -->

Status: **Run 175 (2026-07-17)** — gap-ledger item #5. Mechanism synthesis from
public product docs + prior engine findings (`storage-cost-and-tiering.md`,
Runs 155/161). **Not** a full TCO spreadsheet — list prices move; treat $ as
order-of-magnitude. Pins context: GT `v1.1.3` / CH `26.6.1.1193` OSS compared to
**ClickHouse Cloud** and **Greptime managed/Enterprise cloud** offerings as of
research date.

> **Product authority unchanged:** Parallax ships **self-hostable GreptimeDB +
> Turso**. This note scores how *managed* offerings change the *comparison
> calculus*, not whether to abandon the committed stack.

## Why this axis matters

The white-box engine study leaned **self-hosted** economics:

- GreptimeDB: **1× object-store** + near-stateless compute
- ClickHouse OSS: **N× replicas** on local/premium disk (or cold S3 tier with
  cold-read penalty)

**Managed cloud erases or sells back several of those edges** for a fee. Given
the operator’s anti-operational-complexity goal (anti-self-hosted-Sentry), this
can be the *most practical* decision axis after workload mix.

## Architecture map (four cells)

| | **Self-host OSS** | **Managed / Cloud product** |
| --- | --- | --- |
| **GreptimeDB** | mito2 + OpenDAL S3; metasrv; optional Kafka WAL; operator runs HA | Greptime **Fully-Managed** / BYOC Enterprise (list from **~$290/mo** entry; contact for scale). Ops + upgrades owned by vendor; still object-store economics under the hood. |
| **ClickHouse** | MergeTree / ReplicatedMergeTree + Keeper; manual reshard; S3 = cold tier or experimental paths | **ClickHouse Cloud**: **SharedMergeTree** on shared object storage + **local + distributed cache**; elastic compute; automatic backups. Cloud-only engine family for that separation model. |

### ClickHouse Cloud closes what OSS CH loses on object store

Documented Cloud architecture (SharedMergeTree + caches):

- **Shared storage:** parts live in shared object storage; compute nodes are largely
  stateless for durable data (Cloud-native replacement for ReplicatedMergeTree).
- **Local FS cache** on compute + **distributed cache** service so scale-out does
  not cold-miss every new node (blog + Cloud docs; distributed cache productized
  after private preview).
- **Effect on prior thesis:** OSS cold-S3 ~2000× local penalty and **N× storage**
  for HA are **Cloud product problems already paid for** — not free, but no longer
  structural reasons to reject CH *if* the budget is Cloud.

Sources: ClickHouse docs *SharedMergeTree*; blogs *Building a Distributed Cache
for S3*, *No more disks: stateless compute*.

### Greptime managed preserves the S3-native design

Greptime’s managed pitch is the same architectural thesis (object store primary,
elastic compute) with ops removed. Public claims: up to ~70% lower TCO vs
traditional stacks; entry managed pricing from **$290/month** (pricing page,
2026-07 crawl). BYOC (“bring your own cloud”) for data-residency customers.

## Rough $ framing (order-of-magnitude only)

| Component | Self-host GT | Self-host CH OSS | Greptime managed | ClickHouse Cloud |
| --- | --- | --- | --- | --- |
| **Storage $/GB** | S3 ~$0.023 × **1×** (+ egress/R2 games) | EBS/local or S3×**N** (or cold tier) | Bundled; still object-store class | Object storage + Cloud storage SKU (third-party guides cite ~$25–50/TB-mo class — **verify live quote**) |
| **Compute for SLA** | Elastic / scale-to-low possible | Always-on hot tier for interactive | Vendor-sized | CU-hour model (guides cite ~$0.22–0.75/CU-hr; Dev tier entry ~$67/mo class — **verify live quote**) |
| **Ops FTE** | metasrv + upgrades + backup runbook | Keeper + reshard + backup + upgrades | Near-zero engine ops | Near-zero engine ops |
| **Backup** | COPY + meta snapshot + bucket versioning (Run 174) | BACKUP SQL or Cloud auto (Run 174) | Vendor | Cloud automatic (config.xml note) |
| **HA storage multiplier** | **1×** shared | **N×** OSS | 1× design | **1×** shared (SharedMergeTree) |

**Implication:** comparing **self-host GT vs self-host CH** still favors GT on deep
$/GB. Comparing **managed CH Cloud vs managed GT** collapses the storage-multiplier
gap; decision shifts to **query speed / ecosystem / price quote / lock-in**.

## What survives in each pairing

### Self-host GT vs self-host CH (status quo study)

Unchanged: GT wins deep retention economics + cardinality-insensitive ingest +
Rust contributability; CH wins raw analytical scan/agg and BACKUP SQL ergonomics.
Parallax remains on GT.

### Self-host GT vs ClickHouse Cloud

Cloud CH **buys** SharedMergeTree + distributed cache → much of GT’s OSS cost edge
narrows. GT still wins if: (a) you refuse vendor lock-in / need air-gapped, (b)
managed CH quote exceeds self-host GT TCO, (c) native PromQL/OTLP/Jaeger without
collector glue remains valuable. Cloud CH wins if: team will not run DBs and the
query mix is analytics-heavy.

### Managed GT vs ClickHouse Cloud

Closest “fair managed” fight. Both sell object-store + elastic compute. Score on
**price quote, observability nativeness, multi-signal one-engine, support,
egress, regional coverage**. No public apples-to-apples benchmark in this repo yet
— **owed: quote packet + synthetic bundle workload on free tiers if available**.

### Managed anything vs committed self-host product

Parallax’s product shape is a **self-hostable single binary/stack** for customers.
Managed backends are an **operator deployment choice** for Parallax SaaS (if any),
not a customer-required dependency. Do not force customers through GreptimeCloud
or CH Cloud to use Parallax.

## Live re-verify (Run 175 incidental)

On the same Docker pins as Run 173/174 (self-host OSS only — clouds not stood up
here):

- Anchored `trace_id` count still returns (GT + CH); PromQL HTTP **200** on GT.
- No managed endpoints exercised this pass.

## Decision guidance (honest)

1. **Do not flip the product engine** based on managed pricing alone — stack policy
   is GreptimeDB + Turso.
2. **Do re-score risk:** if Parallax SaaS ever standardizes on **ClickHouse Cloud**,
   several OSS-CH disadvantages in this study (S3 cold, N× HA, ops) **do not apply**
   to that deployment; reverse for Greptime managed.
3. **Highest-value next measurement:** get **current** Cloud quotes (GT managed +
   CH Cloud) for a fixed retained volume + QPS profile matching evidence-bundle
   traffic; attach to this note. List prices in third-party blogs go stale fast.
4. **Hybrid** (CH Cloud hot + GT cold) multiplies vendors — still Phase-2 only
   (`storage-cost-and-tiering.md`).

## Gap ledger update

#5 moves from “not modelled” to **framework answered; $ quote packet owed**.
Engine internals comparison still does not self-complete.

## Sources (primary / first-party preferred)

- <https://clickhouse.com/docs/cloud/reference/shared-merge-tree>
- <https://clickhouse.com/blog/building-a-distributed-cache-for-s3>
- <https://clickhouse.com/blog/clickhouse-cloud-stateless-compute>
- <https://greptime.com/pricing>
- <https://greptime.com/product/cloud>
- Prior: `storage-cost-and-tiering.md`, `backup-and-disaster-recovery.md`, Runs 155/161/174
