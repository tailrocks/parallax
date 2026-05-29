# Storage Architecture, Cost, and the Hot/Cold Tiering Question

<!-- markdownlint-disable MD013 -->

Status: pass (Run 161) — answers the operator's framing: *is ClickHouse performance-first (server/
local-disk) while GreptimeDB is S3/cost-first (store-everything-cheaply, read as fast as the
architecture allows)? If so, compare on **cost** (not just speed): how much each costs to run, what
infra each needs. And does a **hybrid** — ClickHouse for live data + GreptimeDB for historical — make
sense?* This note adds the **cost axis** explicitly (we had mostly compared performance) and states
**which system is for what**. Companion to `platform-fit-and-alternatives.md` (proxy lens),
`distributed-and-scaling.md` (Run 155 object-store economics), `compression-and-cost.md` (per-signal
density), and `retention-cost-model.md` (object pricing).

## The thesis — verified (with one correction)

**Operator thesis:** GreptimeDB is built for cloud/object storage (S3) — store everything cheaply, read
back as fast as the architecture allows; ClickHouse is built for the server — local-disk locality and
vectorized real-time compute, with S3 a secondary tier and price not the first priority.

**Verdict: substantially TRUE.** Evidence:

- **ClickHouse is performance-first / local-storage-centric.** Its speed comes from attached fast
  storage (SSD/NVMe) + the sparse primary index + the vectorized C++ engine. On object storage the
  **cold-read penalty is real**: a warm local-cache read is ~250µs vs a **cold S3 read 10–500ms (~2000×
  slower)** — *latency, not bandwidth,* is the bottleneck (a query touching many small files on S3 with
  >50ms link latency is far slower than on local disk). ClickHouse's **own** recommended way to use S3
  is **hot-cold tiering**: recent ("hot") parts on SSD/NVMe, historical ("cold") parts on S3 via a
  storage policy + `TTL … TO VOLUME`. So S3 in OSS ClickHouse is a **cold tier**, not the primary store
  — exactly the operator's read.
- **GreptimeDB is S3-native / cost-first.** SST files are flushed **directly to S3-compatible object
  storage**, with a local cache keeping hot segments fast and **no second system to operate**;
  compute/storage separation is the core design (datanodes near-stateless; `distributed-and-scaling.md`,
  Run 146/155). It is built to **store a lot cheaply** and read back via cache + the (younger) engine.

**The correction — don't overstate "ClickHouse is badly designed for S3."** It is not broken on S3; it
is *tier-vs-native*: (a) local cache hides the penalty for **hot** reads; (b) ClickHouse Cloud's 2026
**distributed cache** pulls data over the network "almost as fast as a shared-nothing SSD server,
without storing anything locally" — largely closing the S3 gap *in Cloud*; (c) the real OSS issues are
the **cold-read latency on uncached data** and that **scaling still replicates data (N× copies) even
though it's in S3** (`distributed-and-scaling.md`, Run 91) — a cost/overhead problem, not an inability.
So: **OSS ClickHouse on raw S3 has a genuine cold penalty + N× storage; ClickHouse's answer is hot-cold
tiering (+ Cloud distributed cache). GreptimeDB treats S3 as the primary store at 1×.**

## The cost axis (what we under-measured) — components

Performance is one axis; **cost to run** is the other the operator now wants. Four components:

| Cost component | ClickHouse | GreptimeDB |
| --- | --- | --- |
| **Compute** (instances $/mo) | Performance-first → wants **CPU + fast local disk/NVMe** (or cache), sized for vectorized scan; the hot tier is **always-on**. More/bigger servers. | Near-stateless datanodes → **smaller, elastic** compute (scale down when idle); WAL/SSTs offloaded to Kafka/S3. **Fewer always-on servers.** |
| **Storage** ($/GB-mo × copies × density) | Hot on **block storage** (EBS gp3 ~**$0.08/GB**) or local NVMe; cold on S3. OSS HA = **N× copies**. | **S3-native** (~**$0.023/GB**, ~3.5× cheaper than EBS) at **1× shared copy**; denser on metrics+logs (Run 159 ~1.5×). |
| **Egress / requests** | S3 GET + egress on cold reads; more objects/parts (Run 9: ~74 vs 4 objects for 1M spans). | Object-count-efficient (4 objects/1M spans, Run 9); still pays egress on re-reads (Parallax re-reads history → R2 zero-egress is attractive, `retention-cost-model.md`). |
| **Operational** | **Keeper** (Raft) for replication; **manual resharding**; or pay for ClickHouse Cloud. | **Metasrv** + optional **Kafka** (remote WAL); auto-rebalance; or GreptimeCloud. |

**The headline cost number:** object storage is **~3.5× cheaper per GB than block storage** (S3
$0.023 vs EBS gp3 $0.08; Glacier $0.001 is ~80× for archive), and at 10 PB the gap is ~$210k/mo (S3) vs
~$820k/mo (EBS). **Stack the multipliers** for the realistic self-hosted-HA gap on the *bulk* tier:
`(block-vs-S3 ~3.5×) × (N× vs 1× replication) × (per-copy density, GT ~1.5× on metrics+logs)` — which is
why GreptimeDB's "store everything cheaply" is a real, large cost edge for **historical/bulk** data.
ClickHouse's cost is **compute + premium hot storage**; its return is **speed**.

> Caveat: these are list-price, order-of-magnitude planning inputs (see `retention-cost-model.md`); the
> tuned, sized, multi-replica **$ measurement** is owed to the server/harness tier. The *shape* (S3 ≪
> block; 1× ≪ N×; GT compute-light) is architectural and stands.

## Which system is for what (the crisp vision)

- **ClickHouse = the fast, hot, real-time tier.** Pay more (performant compute + block/NVMe storage +
  N× replication) to get the **fastest queries on recent data** — live dashboards, alerting, interactive
  ad-hoc analytics on a bounded hot window. Price is *not* its first priority; **speed is**. Cold/bulk
  on S3 is a bolt-on tier with a real penalty (OSS) or a Cloud-distributed-cache cost.
- **GreptimeDB = the cheap, deep, store-everything tier.** Pay less (S3-native 1×, ~3.5× cheaper medium,
  denser, fewer/elastic servers) to **keep a lot of history affordably**, and read it back as fast as
  the cache + engine allow (interactive on the anchored/keyed hot path; slower on heavy ad-hoc analytics
  — the *engine*, not the storage, `platform-fit-and-alternatives.md`). Its design center is **storage
  cost + elasticity**.

That is the clean split the operator asked for: **ClickHouse optimizes time-to-answer on hot data;
GreptimeDB optimizes $/GB-retained on deep data.**

## The hybrid — ClickHouse (live) + GreptimeDB (historical)

The operator's idea: route **live/recent** telemetry to ClickHouse (fast), age it out to **GreptimeDB**
for cheap historical storage (dump CH→GT by age), and let the **Parallax proxy route queries** by time
range. Assessment:

- **Why it's coherent.** It is the **cross-engine version of ClickHouse's own hot-cold pattern** — but
  instead of CH's *cold S3 tier* (N× cost + cold penalty in OSS) you use **GreptimeDB's 1×-S3-native**
  for cold. So you get **ClickHouse's best hot performance AND GreptimeDB's best cold cost** —
  optimizing *both* ends rather than compromising. Parallax is uniquely placed to do this: it already
  owns ingestion/routing (the proxy), so "write hot to CH, roll cold to GT, fan a time-spanning query to
  both and merge" is in its wheelhouse.
- **What it costs.** **Two storage engines to operate** (their replication/WAL/scaling models, upgrades,
  backups) — against the operator's core anti-complexity goal (the whole anti-self-hosted-Sentry
  motivation). Plus a **roll-over pipeline** (CH→GT by age) and **cross-boundary query federation** (a
  query spanning hot+cold must hit both and union/merge — the proxy must implement this, and ordering/
  dedup at the seam is fiddly).
- **The simpler alternatives to weigh it against.**
  1. **Single ClickHouse with internal hot-cold tiering** (SSD hot + S3 cold via storage policy/TTL
     MOVE): one system, fast hot, but cold = N× S3 (OSS) + cold penalty; or ClickHouse Cloud (distributed
     cache + SharedMergeTree) which closes both at a $ premium.
  2. **Single GreptimeDB** (S3-native throughout + local cache): one system, cheap storage everywhere,
     hot reads good on the anchored/keyed path but heavy ad-hoc analytics slower (engine).
- **Recommendation.** The hybrid is a **scale optimization, not the starting default.** Start with **one
  engine** (the proxy-lens default leans ClickHouse for retrieval+ecosystem; GreptimeDB if
  cost-at-scale/cardinality dominates) and a single hot/cold tier *inside* it. **Adopt the CH-live +
  GT-historical hybrid only when** (a) the hot window genuinely needs ClickHouse's speed, **and** (b) the
  historical volume is large enough that GT's 1×-S3 economics beat CH's cold tier by more than the cost
  of running a second engine + federation. Parallax's proxy makes the hybrid *feasible* later without a
  rewrite (route-by-age is a proxy policy) — so it can be a **Phase-2 cost optimization**, not a Day-1
  commitment. Decide it on the sized $ numbers (below), not in the abstract.

## What the benchmark must now measure (cost + performance)

We have largely measured **speed**. To decide this, each storage benchmark must **also** capture
**cost** (reframed into the benchmark prompt, Run 161):

1. **$/GB retained** at the tested scale — on-disk size × storage class price × replication factor
   (1× GT vs N× CH-OSS), per signal (metrics/logs/traces; densities differ, Run 159).
2. **Compute footprint** to hold the latency target — instance count/size + whether always-on or
   elastic; the *cost of the SLA*, not just the latency.
3. **Cold-read cost** — latency **and** S3 GET/egress for re-reading historical data (Parallax's AI
   context re-reads history; egress can dominate, `retention-cost-model.md`).
4. **Hybrid total cost** — model `CH(hot window) + GT(historical) + roll-over + federation` vs
   single-engine tiering, at a realistic hot:cold ratio.
   Every speed number should carry its **$ context**: "X ms at $Y/GB-mo on Z compute," so the final
   decision weighs **time-to-answer against cost-to-run**.

## Sources / evidence

- ClickHouse S3 cold penalty + hot-cold tiering: [Separation of storage and compute (ClickHouse Docs)](https://clickhouse.com/docs/guides/separation-storage-compute),
  [S3 as cold storage in ClickHouse](https://oneuptime.com/blog/post/2026-03-31-clickhouse-s3-cold-storage/view),
  [Building a distributed cache for S3 (ClickHouse, 2026)](https://clickhouse.com/blog/building-a-distributed-cache-for-s3),
  [ClickHouse storage tiering best practices](https://chistadata.com/clickhouse-storage-tiering/).
- GreptimeDB S3-native + cost: [GreptimeDB as a ClickHouse alternative](https://greptime.com/tech-content/2026-04-17-clickhouse-alternative-greptimedb),
  [How GreptimeDB cuts IoT storage costs 10× (EMQ)](https://www.emqx.com/en/blog/cloud-native-storage-engine-how-greptimedb-cuts-iot-storage-costs-by-10x),
  [ClickHouse pricing 2026 (cloud vs self-hosted)](https://improvado.io/blog/clickhouse-warehousing-pricing).
- Internal: `distributed-and-scaling.md` (Run 155 object-store economics, 1× vs N×, SharedMergeTree
  Cloud-only), `compression-and-cost.md` (Run 159 per-signal density), `retention-cost-model.md` (S3/R2/
  B2 pricing, egress), `platform-fit-and-alternatives.md` (proxy lens; slower=engine, cheaper=storage-arch).
- Pricing figures are 2026 list prices, order-of-magnitude; sized/tuned $ owed to the server tier.
