# Managed Cloud vs Self-Host — Cost & Ops Calculus

<!-- markdownlint-disable MD013 -->

Status: **Run 175** framework; **Run 221 (2026-07-17)** — primary-source **quote
packet** from Greptime + ClickHouse pricing pages / billing docs. **Run 405
(2026-07-18)** — re-fetched same sources: **list rates hold** (no drift). Still
**not** a signed commercial quote. Pins: GT `v1.1.3` / CH `26.6.1.1193`.

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
| **Storage $/GB** | S3 ~$0.023 × **1×** (+ egress/R2 games) | EBS/local or S3×**N** (or cold tier) | Pay-for-stored; entry plan “unlimited storage, pay what you stored” (no public $/TB on page) | **$25.30 / TB-mo** compressed (AWS us-east-1; same SKU Scale + Enterprise, 2026-07) |
| **Compute for SLA** | Elastic / scale-to-low possible | Always-on hot tier for interactive | Vendor-sized from **$290/mo** Fully-Managed floor | Metered unit-hr: Scale **$0.2985/unit-hr**, Enterprise **$0.3903/unit-hr** (us-east-1); Basic example floor **~$66.52/mo** @ 6h/day |
| **Ops FTE** | metasrv + upgrades + backup runbook | Keeper + reshard + backup + upgrades | Near-zero engine ops | Near-zero engine ops |
| **Backup** | COPY + meta snapshot + bucket versioning (Run 174) | BACKUP SQL or Cloud auto (Run 174) | Vendor / Enterprise auto-backup | Counted **separately** toward storage; default 1 backup retained 1 day |
| **HA storage multiplier** | **1×** shared | **N×** OSS | 1× design | **1×** shared (SharedMergeTree) |

**Implication:** comparing **self-host GT vs self-host CH** still favors GT on deep
$/GB. Comparing **managed CH Cloud vs managed GT** collapses the storage-multiplier
gap; decision shifts to **query speed / ecosystem / price quote / lock-in**.

## Run 221 — primary-source quote packet (2026-07-17)

Sources (fetch date = research day):

- Greptime: <https://greptime.com/pricing>
- ClickHouse Cloud pricing page: <https://clickhouse.com/pricing> (us-east-1 calculator surface)
- ClickHouse billing docs (worked examples): <https://clickhouse.com/docs/cloud/manage/billing/overview>

### Published list numbers (verbatim class)

| Offer | Public floor / rate | What is included (marketing/docs) | Gap |
| --- | --- | --- | --- |
| **Greptime Enterprise Fully-Managed** | **From $290 / month** | Guaranteed resources; “unlimited data storage and retention, pay for what you stored”; isolated resources; SQL + PromQL + OTel | **No public $/TB or CU-hr** — storage and overage are sales-configured |
| **Greptime Enterprise BYOC** | **Custom pricing** | Deploy in customer cloud / AWS Marketplace; SLA + dedicated TAM | Contact-only |
| **CH Cloud Basic** | **From $66.52 / mo** (docs example) | 1×8 GiB / 2 vCPU, 500 GB compressed + 500 GB backup, 10 GB public egress, 5 GB XR; active **6 h/day** in the $66.52 row | Not for hard multi-AZ SLA |
| **CH Cloud Scale** | Storage **$25.30 / TB-mo**; compute **$0.2985 / unit-hr**; worked **from $499.38 / mo** | Unlimited storage SKU; 2+ AZ; auto vertical scale; private networking; 24h backups | Worked $499.38 = 2×8 GiB always-on + 1 TB + 1 backup + small egress |
| **CH Cloud Enterprise** | Storage **$25.30 / TB-mo**; compute **$0.3903 / unit-hr**; worked from **~$2,669 / mo** | SSO, CMEK, HIPAA/PCI, named support, private regions | Worked example 2×32 GiB + 5 TB |
| **CH Cloud egress** | Public internet **from $0.1152 / GB**; inter-region **from $0.0312 / GB** | Plus ClickPipes **$0.04 / GB** ingest + **$0.20 / hr** per pipe CU | Material if re-read-heavy |

Compute metering (docs): per-minute, in **8 GiB RAM increments** (“units”). Storage =
**compressed** table bytes on object store; **backups billed as storage too**.

### Fixed-profile scenarios (planning envelopes, not invoices)

Assume AWS us-east-1, 30-day month, CH rates above. Greptime cells use the
**$290 floor** when size is unknown and flag “+ storage quote required”.

| Profile | Retained compressed | Compute assumption | **CH Cloud (list math)** | **Greptime managed (list)** |
| --- | --- | --- | --- | --- |
| **A — Dev / spike** | 0.5 TB | 1 unit (8 GiB), **6 h/day** | Docs Basic-class: **~$66–186/mo** depending on active hours (storage alone 0.5×$25.30≈$12.65 if not in Basic bundle) | **≥$290/mo** floor; storage overage unknown publicly |
| **B — Small always-on prod** | 1 TB + 1 backup | 2×8 GiB always-on (Scale) | Docs Example 1: **~$499/mo** (compute ~$437 + storage+backup ~$51 + egress ~$12) | **≥$290/mo** + pay-stored; **likely competitive or cheaper** if vendor sizes lean, **but unproven without quote** |
| **C — Mid retention** | 10 TB + 1 backup | 2×16 GiB always-on (Scale) | Storage+backup ≈11×$25.30≈**$278**; compute 2×(16/8)×$0.2985×720≈**$860**; total **~$1.15–1.3k/mo** + egress | Still **contact**; $290 is not the bill at 10 TB |
| **D — Large** | 20 TB + backup | 2×32 GiB (Enterprise-class) | Docs Enterprise-ish: storage ~$506–1k; compute multi-k; **$5k–10k/mo** class | Enterprise/BYOC custom |

**Self-host GT envelope (same data, rough infra only — not ops FTE):** raw S3 at
~$0.023/GB-mo → 1 TB ≈ **$23/mo** storage (1×), 10 TB ≈ **$230/mo**, 20 TB ≈
**$460/mo**, plus always-on compute (2–3 modest VMs + optional Kafka) often
**$100–400/mo** for small, **$500–2k+** for mid — so **self-host GT storage stays
~order-of-magnitude below CH Cloud storage SKU** ($25.30/TB ≈ $0.025/GB is
*similar to S3 list* before CH markup on backups/replicas of metadata), while
**managed compute is where both clouds charge the real money**.

### Comparison logic (honest)

1. **CH Cloud is transparent and calculator-driven.** Storage + unit-hr + egress
   are public; worked examples land **~$67 (Basic part-time) → ~$500 (Scale 1 TB)
   → multi-k (Enterprise)**. Good for budget modeling without sales.
2. **Greptime Fully-Managed is opaque above the $290 floor.** “Pay for what you
   stored” without a published $/TB means **Run 221 cannot close an apples-to-apples
   $ row** for profiles B–D without a vendor quote or trial invoice.
3. **At profile B (1 TB always-on), CH Scale ~$500 is a concrete comparator**;
   Greptime’s $290 floor *can* undercut it **if** the floor includes enough
   compute for the workload — unknown until sized. Do **not** treat $290 as
   “always cheaper than CH Cloud.”
4. **Self-host GT still wins deep retention math** when ops FTE is accepted
   (product default). Managed CH Cloud **closes OSS CH’s N×/cold-S3 tax** for $
   (SharedMergeTree) — thesis from Run 175 **still holds** with fresher numbers.
5. **Egress** on CH Cloud (from **$0.1152/GB** public) is the sleeper for
   evidence-bundle re-read-heavy SaaS; prefer same-region app + private link;
   GT managed egress not published here.

### Still owed after Run 221 / 405

- **Signed/trial quotes** for a fixed Parallax profile (ingest GB/day, retained
  TB, QPS for Q1–Q6 mix) from both vendors. **Run 405 re-fetch:** public list
  numbers unchanged (`$290` GT floor; CH Basic **$66.52** / Scale **$499.38** /
  Enterprise **$2,669.40** examples; storage still **$25.30/TB-mo** in the docs
  math). Sales/trial still required for apples-to-apples.
- Greptime **published or trial-metered $/TB and CU** (or fixed SKU dimensions).
- Optional: free-tier / trial burn of a synthetic bundle on each cloud (product
  time, not engine-internals).

### Decision guidance update

Highest-value commercial next step is **sales/trial quote with a one-page
workload card** (not more laptop smoke). Engine stack policy remains self-host
GreptimeDB + Turso for the product binary.

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
3. **Highest-value next measurement:** Run 221 attached **primary list rates +
   scenario envelopes**. Still need **vendor-sized quotes** for GT (opaque above
   $290) and a trial burn for a fixed evidence-bundle profile. List prices go
   stale — re-fetch the three URLs above before budgeting.
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
