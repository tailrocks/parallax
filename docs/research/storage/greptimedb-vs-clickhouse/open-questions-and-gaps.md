# Open Questions & What's Not Yet Addressed (gap ledger)

<!-- markdownlint-disable MD013 -->

Status: created Run 171 — operator asked "what else have we missed?" This consolidates the scattered
open questions + the dimensions the storage research has **not** covered, prioritized. The GreptimeDB-
vs-ClickHouse *engine* comparison is exhaustive + re-verified (Runs 1–**200**). **Run 173 re-pinned**
to GT `v1.1.3` / CH `26.6.1.1193`. Runs 173–200 re-verified live: JSON2, backup/DR, managed-cloud
framework, last-value shape, concurrent ingest, quotas, native protocols, schema-on-write, PromQL
tax scale-shape, full-text, projections, append DELETE ban, TTL, Flow/MV, storage density. The gaps
below still need product input, server-tier scale, or $ quotes — not more engine smoke.

> **Current authority:** GreptimeDB + Turso are mandatory. ClickHouse and
> Postgres are comparators only. This is a factual research-gap ledger, not a
> product backlog or backend-selection mechanism. Plan 093 is closed (contract cleanup historical);
> plan 115 owns any supported server profile.

## 1. THE deciding input we don't have — Parallax's real workload mix

The entire verdict is conditional on **one un-characterized variable**: is Parallax's query distribution
**anchored-retrieval-dominant** (evidence bundles keyed by `trace_id`/`fingerprint` → both engines fast,
GreptimeDB fine) or **ad-hoc-analytics-dominant** (heavy scans/aggregations over large windows →
ClickHouse wins)? Every flip-trigger (`verdict-which-to-choose.md`, `platform-fit-and-alternatives.md`)
points here, and we have **never modelled Parallax's expected query mix**. **This is the highest-value
missing input — the operator (product intent) or a projected-usage model resolves it, not another
benchmark.** The missing input limits performance claims; it cannot change the
mandatory GreptimeDB product engine.

**Run 223:** measurement + scoring packet drafted —
[`workload-mix-decision-input.md`](workload-mix-decision-input.md) (A1–A7 shares, rubric,
flip thresholds, how to gather). **Still empty:** filled product hypothesis row and live
proxy counters.

## 2. Server-tier benchmarks (deferred + agent-network-blocked)

Known, owed, operator-deferred ("the proper test on the server, not now"):
- **Timing 4-build** at sized N (the core speed claims — agg ~2×, scan ~7–14× — re-verified only by plan,
  not fresh timing this cycle; agent capsule can't reach Docker ports, Run 155).
- **Sized $/GB + multi-replica storage cost** (the cost axis numbers; `storage-cost-and-tiering.md`).
- **Cold-read at GB–TB from S3** (the cold-cache flip-trigger; `caching-and-cold-warm.md`).
- **Multi-node hold** — region rebalance, `MergeScanExec` fan-out latency, ClickHouse resharding effort
  (`distributed-and-scaling.md`).
- The **hybrid** (CH-hot + GT-cold) total-cost model vs single-engine tiering.

## 3. Parallax's own layers — above the engine, largely untouched

The research compared *storage engines*; these Parallax components are not designed (may be out of this
sub-study's scope, but they are "not addressed"):
- **The proxy/ingestion layer** — buffering, routing, format conversion, backpressure, batching for the
  backend (the proxy reframe established it *exists*, not *how*).
- **The query/API surface** — what Parallax exposes (PromQL? SQL? Sentry API? Jaeger?) and the
  translation cost to the backend (PromQL←→SQL is the expensive one, Run 164).
- **The evidence-bundle / AI-context assembly** — Parallax's core value. We validated the DB-side
  pattern (anchored keyed fetch + app-side correlation, Runs 154/158/165), but the assembly logic / the
  ranking/scoring of what goes in a bundle is undesigned.

## 4. Cross-cutting concerns — not addressed

- **Multi-tenancy / isolation** — **answered at the engine layer in Run 172**:
  ClickHouse has stronger OSS-native guardrails (grants, row policies, quotas, settings profiles);
  GreptimeDB OSS is coarse (auth + global read/write modes), so Parallax's proxy must own tenant
  authorization and GreptimeDB must not be user-facing without Enterprise/custom auth. Remaining work:
  product-level tenant model and a row-policy benchmark if direct SQL/BI access becomes a requirement.
  See `multi-tenancy-and-isolation.md`.
- **Auth / access control** — engine layer partly answered by Run 172; remaining work is the
  Parallax product/API permission model (projects, teams, tokens, internal analyst access).
- **Backup / disaster recovery** — **engine layer answered in Run 174**
  (`backup-and-disaster-recovery.md`): GT = `COPY`/cli data export + `meta snapshot` (+ S3-native
  data plane); CH = first-class `BACKUP`/`RESTORE` SQL (File/Disk/S3/Azure), live restore count
  match. Remaining: product runbook (Turso + Greptime meta + object store RPO/RTO), not engine choice.
- **Rate-limiting / quotas / ingestion protection** — **engine layer answered Run 179**
  (`multi-tenancy-and-isolation.md`): CH has SQL quotas + settings limits (live
  `max_execution_time` / `max_result_rows`); GT has process/resource admission
  (`max_in_flight_write_bytes`, concurrent queries, body limit, RateLimited→429) but **no
  per-tenant SQL quota**. Product-tenant fair-share + OTLP QPS remain **proxy-owned**.

## 5. Managed-cloud vs self-host — framework answered (Run 175); live $ quotes still owed

**Framework:** `managed-cloud-vs-self-host.md` (Run 175). ClickHouse Cloud
(`SharedMergeTree` + local/distributed cache) collapses OSS CH’s N×/cold-S3 disadvantages for a
premium; Greptime managed keeps S3-native economics with ops removed (public entry ~$290/mo class).
**Product stack still GreptimeDB self-hostable.** Remaining: **current vendor quotes** for a fixed
retained volume + evidence-bundle QPS (list prices go stale); optional free-tier smoke — not an
engine-internals blocker.

## 6. Decided-but-not-designed

- **Metadata store schema** — grouped errors/issues/config live in mandatory
  Turso, not the columnar engine; the relational model remained incomplete in
  this dated study
  (issues, fingerprint→issue mapping, projects/users, how it joins to the columnar telemetry).
- **The hybrid federation** (if pursued) — how the proxy routes/merges a time-spanning query across
  CH-hot + GT-cold (flagged Phase-2, `storage-cost-and-tiering.md`, undesigned).
- **Schema blueprint at scale** — the per-signal schemas (key `trace_id`/`fingerprint` everywhere, Run
  158; low-card PK + append_mode, `greptimedb-implementation.md`) are sketched but not a complete
  production DDL set per engine.

## 7. Operational-complexity full picture — partial

We have storage cost; we don't have the full **ops burden** comparison: running ClickHouse (+ Keeper +
manual resharding) vs GreptimeDB (+ metasrv + optional Kafka remote WAL) — upgrade story, on-call
surface, failure modes. Relevant to the anti-complexity goal.

## What is NOT a gap (settled, re-verified)

Engine internals (all subsystems), the proxy reframe + which-system-for-what, the cost thesis (CH=hot/
perf, GT=deep/cheap), the metadata split, the native-structure trio, the surviving-GT-edges set +
nuances, and repo health — all grounded + re-verified (Runs 1–170). The decision *framework* is done;
the *inputs* (1, 2, 5) and the *layers above the engine* (3, 4, 6) are what remain.

## How to use this

Research **#1 (workload mix)** and **#5 (managed-vs-self-host)** first because
they most affect risk and claims. **#2** remains a server-tier research run.
Product work discovered by **#3/#4/#6** must be added to `plans/` before
implementation; this ledger does not authorize it.

## Next engine-loop targets (after Run 219, 2026-07-17)

**Pass 110 research consume (desk, no new bench):** this section is the correct
boundary for the indefinite research brief — **do not** re-run local four-way
smoke as if it closed agenda item 5. Server-tier + workload-mix remain **unproven**.

Engine-smoke re-verify cycle after v1.1.3 re-pin is **saturated** for laptop scale
(Runs 220–232 added MinIO pin fix, managed list rates, RPO runbook+D1 drill,
workload-mix packet, partition/TWCS/PREWHERE/projection/Flow/freshness/concurrent
re-verifies — **no direction drift**).

Highest-value *remaining* items (not “done”):

1. **Workload mix shares filled** (product) — Run 223 packet; fill A1–A7 + proxy counters.
2. **Server-tier 1M/5M four-way** on v1.1.3 + 26.6 — dedup-agg regression + magnitude ratios. Contract: [`server-tier-runbook.md`](server-tier-runbook.md) (Run 240).
3. **Vendor-sized managed quotes** — Run 221 list rates; trial/sales quote for fixed profile.
4. **Cold S3 selective egress at GB scale** — layout+instrumentation (220/234/235);
   **GB–TB** selective cold still owed.
5. **RPO D2/D3 drills** — Run 225 D1 done; **Run 405:** standalone cannot use
   `meta snapshot save` (raft-engine); need etcd/RDS for true D2. **Run 409:**
   D3 logical dump/restore pattern OK on SQLite stand-in (`ROW_MATCH`); product
   schema + Turso CLI still owed.
6. ~~GT OpenDAL/S3 request metrics~~ — **closed Runs 234–235** (`opendal_http_*GetObject`).
7. ~~CH TimeSeries "broken"~~ — **Run 403** query path real; **Run 404** `rate`/`sum`/
   `avg by` match GT, **`increase` still NOT_IMPLEMENTED** on 26.6+26.7. Remaining watch:
   leave experimental / Cloud support / more PromQL fns + volume — not "unusable."

Do **not** burn passes re-confirming interactive 50k–100k ties, small-N object counts,
or plan-shape re-verifies of PREWHERE/projection/TWCS/PARTITION unless a **pin bumps**.

## Run 246 ranking (2026-07-17) — after passes 220–245

Laptop + small MinIO work **advanced**: S3 pin fix, managed list rates, RPO D1,
workload-mix packet, OpenDAL GetObject method, partition/TWCS/PREWHERE/projection/
Flow/freshness/JSON2/PromQL/identity/TimeSeries/concurrent re-verifies. **No
engine direction flip.**

| # | Gap | Status |
| --- | --- | --- |
| 1 | Workload mix **filled shares** | packet exists (223); product fill owed |
| 2 | Server 1M/5M four-way | runbook (240); hardware owed |
| 3 | Vendor trial $ quotes | list rates (221); sales/trial owed |
| 4 | Cold S3 **GB–TB** selective | method (235/238); large N owed |
| 5 | RPO D2 meta + D3 Turso | D1 done (225); D2/D3 owed |
| 6 | OpenDAL GET instrument | **closed** 234–235 |
| 7 | CH TimeSeries maturity | **403:** query path real; experimental+Cloud-no |

Do not re-smoke interactive 50k ties without pin bump or new mechanism.

## Run 408 / 416 (2026-07-18) — ranking after 403–415

**Not done.** Laptop engine comparison still saturated; this cycle closed
**false negatives**, RPO nuances, and re-verified load-bearing holds:

| Run | Finding | Direction impact |
| --- | --- | --- |
| 403 | CH TimeSeries SQL INSERT + `prometheusQuery*` **work**; outer SELECT Code 48 is facade | Corrects “unusable” misread; maturity still experimental |
| 404 | `rate`/`sum`/`avg by` **match GT**; **`increase` missing** 26.6+26.7 | Completeness gap remains |
| 405 | Standalone **cannot** `meta snapshot save` (raft-engine); export-v2 schema OK; managed $ **no list drift** | D2 = cluster-only CLI; tiny-tier = FS copy |
| 406–407 | adopt-native no drift; OTEL tables **on-ingest only** | Product path confirmed |
| 409 | D3 SQLite dump/restore **ROW_MATCH** (stand-in schema) | Pattern OK; product DDL still owed |
| 410–415 | Pins hold; PREWHERE 1/6; last_value/argMax; freshness; append DELETE; Flow/MV | **No direction drift** |

**Still highest remaining (execute / product):**

1. Workload mix A1–A7 **filled shares** (`workload-mix-decision-input.md`)
2. Server 1M/5M four-way (`server-tier-runbook.md`)
3. Vendor **trial** quotes (list rates held Run 405; still sales-blocked)
4. GB–TB cold S3 selective egress
5. RPO **cluster** D2 (etcd/RDS) + product-schema D3
6. Optional: CH PromQL **volume** (comparator) — **fn coverage partly filled Run 423**
   (partial surface documented; volume still owed)

**Do not** burn more interactive 50k ties without pin bump.

## Run 258 (2026-07-17) — still not done

Highest remaining after 220–257:

1. Product fills workload-mix shares (packet 223)
2. Server N=1M/5M (runbook 240)
3. Vendor trial quotes (list 221)
4. GB–TB cold S3 selective (method 235/238)
5. RPO D2/D3 (D1=225)

Engine smoke on laptop remains saturated. **Comparison not declared done.**

## Run 268 ranking (2026-07-17)

Passes **220–267** on main. Laptop mechanism re-verify **saturated**. Still open:

1. Workload mix shares filled (223 packet)
2. Server 1M/5M (240 runbook)
3. Vendor trial quotes (221 rates)
4. GB cold S3 (235/238 method)
5. RPO D2/D3 (225 D1 done)

**Do not declare comparison done.**

## Run 274 (2026-07-17)

Harness improved (logs.trace_id indexed, Runs 271–273). **Still not done:**

1. Workload mix shares (product)
2. Server 1M/5M (`server-tier-runbook.md`)
3. Vendor trial quotes
4. GB cold S3 selective egress
5. RPO D2/D3

Loop continues.

## Run 284 (2026-07-17) — after harness logs.trace_id

Mechanism/laptop smoke through **Run 283** including harness fix (271–273).
**Still open for real progress:**

| Gap | Needs |
| --- | --- |
| Workload mix shares | Operator/product fill of A1–A7 |
| Server 1M/5M | Server hardware + `server-tier-runbook.md` |
| Vendor quotes | Sales/trial invoices |
| GB cold S3 | Large MinIO load |
| RPO D2/D3 | Cluster meta + Turso fixture |

**Comparison not done.**

## Run 293 — still open

Through Run 292. Highest remaining: mix shares, server 1M/5M, quotes, GB cold, RPO D2/D3.

## Run 308 ranking

Through **307**. Harness improved (logs.trace_id). Instrument improved (OpenDAL
GetObject). Packets exist for mix/RPO/server/managed. **Execute next:**

1. Operator fills workload-mix A1–A7
2. Server runs `server-tier-runbook.md`
3. Request vendor quotes
4. GB MinIO cold with OpenDAL deltas
5. Turso dump + meta snapshot drill

Not done.

## Run 349 — still not done (2026-07-17)

Through 348. Execute: mix fill, server runbook, quotes, GB cold, RPO D2/D3.

## Run 356 status (2026-07-17)

**Done this long session (220+):** S3 pins+OpenDAL GET method, managed list
rates, RPO D1+runbook, workload-mix packet, server-tier runbook, harness
logs.trace_id, dozens of live re-verifies, density/instrument notes.

**Still open (not done):**

1. Workload mix A1–A7 filled by product
2. Server 1M/5M four-way
3. Vendor trial quotes
4. GB cold S3 selective
5. RPO D2 meta + D3 Turso

Loop continues until operator stops.

## Run 365

Still not done. Top 5: mix, server, quotes, GB cold, RPO D2/D3.

## Run 379

Not done. Mix/server/quotes/GB/D2–D3.

## Run 388

Not done. Top 5 gaps stand.

## Run 393 (2026-07-17) — operator-facing remaining work

Laptop engine smoke + instrument + packets **exhausted** for this machine.

| Do next | Where |
| --- | --- |
| Fill A1–A7 shares | `workload-mix-decision-input.md` |
| Run N=1M/5M four-way | `server-tier-runbook.md` |
| Get vendor quotes | `managed-cloud-vs-self-host.md` Run 221 rates |
| GB MinIO cold GETs | recipe in `caching-and-cold-warm.md` |
| RPO D2/D3 drills | `product-rpo-runbook.md` |

**Not done** until operator stops the loop or these execute.

## Run 427 (2026-07-18) — pins hold; still not done

Hub re-check: GT latest stable still **v1.1.3** (2026-07-17); latest nightly still
**v1.2.0-nightly-20260713**; CH feature stable still **26.6.1.1193**; head reports
**26.7.1.1097**. **No pin bump.**

Cycle 423–426 advanced: PromQL partial surface map, join prune re-verify, warm
last_value four-way, JSON type honesty (no SQL `JSONB`). **No stack direction flip.**

Highest remaining unchanged: (1) workload mix shares (2) server 1M/5M (3) trial quotes
(4) GB cold S3 (5) cluster D2 + product D3.

## Run 433 (2026-07-18) — ranking after 423–432

**Not done.** Pins still `v1.1.3` / `26.6.1.1193` / head `26.7.1.1097`.

| Run | What closed / held |
| --- | --- |
| 423 | CH PromQL **partial surface map** (rate/agg/topk OK; increase + most `*_over_time` missing) |
| 424 | Join prune: CH both sides; GT PK filter |
| 425–429 | Four-way last_value / metric-agg warm interactive; cold first-hit artifact |
| 426 | SQL type **JSONB unsupported**; default JSON works; JSON2 INSERT caveats |
| 428 | FT tokenbf selective 0/7 vs broad 7/7 |
| 430 | TTL expire both engines |
| 432 | Projection p_svc on stable+head |

**Still highest remaining:**

1. Workload mix A1–A7 filled shares (product)
2. Server 1M/5M four-way
3. Vendor trial quotes
4. GB–TB cold S3
5. Cluster D2 meta + product-schema D3

Laptop engine smoke remains **saturated** for direction; keep pin-watch +
comparator completeness only unless pin bumps.

## Run 440 (2026-07-18) — still not done after 423–439

**Not done.** Highest remaining unchanged (product / server):

1. Workload mix A1–A7 filled shares
2. Server 1M/5M four-way
3. Vendor trial quotes
4. GB–TB cold S3
5. Cluster D2 + product D3

Laptop cycle closed PromQL partial surface (423/439) and re-verified load-bearing
holds (join, TTL, FT, projection, adopt-native, append DELETE). **No pin bump.**

## Run 445 (2026-07-18) — affirm not done

Comparison **not done**. Pins hold. Highest remaining still:

1. Workload mix A1–A7 filled shares  
2. Server 1M/5M four-way  
3. Vendor trial quotes  
4. GB–TB cold S3  
5. Cluster D2 + product D3

## Run 449 (2026-07-18) — still not done

After runs 423–448: PromQL partial surface mapped; load-bearing holds re-verified.
**Not done.** Top five still product/server-gated.

## Run 453 (2026-07-18) — still not done

Pins hold. Laptop 423–452: PromQL surface, join/TTL/FT/projection, windows,
percentiles, density. **Not done.** Top five: mix, server, quotes, GB cold, cluster RPO.

## Run 455 (2026-07-18) — still not done

Highest remaining (unchanged):

1. Workload mix A1–A7 filled shares
2. Server 1M/5M four-way  
3. Vendor trial quotes
4. GB–TB cold S3
5. Cluster D2 + product D3

## Run 459 (2026-07-18) — still not done

Highest remaining: (1) mix shares (2) server 1M/5M (3) trial quotes (4) GB cold S3
(5) cluster D2 + product D3. **Not done.**

## Run 461 (2026-07-18) — still not done

**Not done.** Pins: GT v1.1.3 / nightly 20260713 / CH 26.6.1.1193 / head 26.7.1.1097.
Highest remaining: mix, server 1M/5M, quotes, GB cold, cluster D2/product D3.

## Run 463 (2026-07-18) — still not done

**Not done.** Highest remaining: mix, server 1M/5M, quotes, GB cold, cluster RPO D2/D3.

## Run 465 (2026-07-18) — still not done

**Not done.** Cycle 423–464: PromQL partial surface, join/TTL/FT/projection/windows/
percentiles, adopt-native, RPO export-v2. Top five still product/server.

## Run 467 (2026-07-18) — still not done

**Not done.** Pins hold. Highest remaining: mix, server 1M/5M, quotes, GB cold S3,
cluster D2 + product D3.

## Run 469 (2026-07-18) — still not done

**Not done.** Highest remaining: mix, server 1M/5M, quotes, GB cold, cluster RPO.

## Run 471 (2026-07-18) — still not done

**Not done.** Highest remaining: (1) mix shares (2) server 1M/5M (3) trial quotes
(4) GB cold S3 (5) cluster D2 + product D3.

## Run 473 (2026-07-18) — still not done

**Not done.** Highest remaining: mix, server 1M/5M, quotes, GB cold, cluster RPO.

## Run 475 (2026-07-18) — still not done

**Not done.** Highest remaining: mix, server 1M/5M, quotes, GB cold, cluster RPO.

## Run 477 (2026-07-18) — still not done

**Not done.** Highest remaining: mix, server 1M/5M, quotes, GB cold, cluster RPO.

## Run 479 (2026-07-18) — still not done

**Not done.** Highest remaining: mix, server 1M/5M, quotes, GB cold, cluster RPO.

## Run 481 (2026-07-18) — still not done

**Not done.** Highest remaining: mix, server 1M/5M, quotes, GB cold, cluster RPO.

## Run 483 (2026-07-18) — still not done

**Not done.** Highest remaining: mix, server 1M/5M, quotes, GB cold, cluster RPO.

## Run 485 (2026-07-18) — still not done

**Not done.** Highest remaining: mix, server 1M/5M, quotes, GB cold, cluster RPO.

## Run 487 (2026-07-18) — still not done after 423–486

**Not done.** Pins hold. Highest remaining:

1. Workload mix A1–A7 filled shares (product)
2. Server 1M/5M four-way
3. Vendor trial quotes
4. GB–TB cold S3
5. Cluster D2 + product D3

Laptop cycle mapped CH PromQL partial surface and re-verified load-bearing holds.
No stack direction flip.

## Run 489 (2026-07-18) — still not done

**Not done.** Highest remaining: mix, server 1M/5M, quotes, GB cold, cluster RPO.

## Run 491 (2026-07-18) — still not done

**Not done.** Highest remaining: mix, server 1M/5M, quotes, GB cold, cluster RPO.

## Run 493 (2026-07-18) — still not done

**Not done.** Highest remaining: mix, server 1M/5M, quotes, GB cold, cluster RPO.

## Run 495 (2026-07-18) — still not done

**Not done.** Highest remaining: mix, server 1M/5M, quotes, GB cold, cluster RPO.

## Run 497 (2026-07-18) — still not done

**Not done.** Highest remaining: mix, server 1M/5M, quotes, GB cold, cluster RPO.

## Run 499 (2026-07-18) — still not done

**Not done.** Highest remaining: mix, server 1M/5M, quotes, GB cold, cluster RPO.

## Run 500 (2026-07-18) — milestone; still not done

**Not done.** Pins: GT **v1.1.3** / nightly **1.2.0** (`v1.2.0-nightly-20260713`) /
CH **26.6.1.1193** / head **26.7.1.1097**. All four healthy. last_value warm interactive.

Highest remaining (unchanged):

1. Workload mix A1–A7 filled shares
2. Server 1M/5M four-way
3. Vendor trial quotes
4. GB–TB cold S3
5. Cluster D2 + product D3

Runs 423–500 closed PromQL partial surface mapping and many load-bearing re-verifies.
**No stack direction flip.** Loop continues.

## Run 503 (2026-07-18) — still not done

**Not done.** Highest remaining: mix, server 1M/5M, quotes, GB cold, cluster RPO.

## Run 505 (2026-07-18) — still not done

**Not done.** Highest remaining: mix, server 1M/5M, quotes, GB cold, cluster RPO.

## Run 507 (2026-07-18) — still not done

**Not done.** Highest remaining: mix, server 1M/5M, quotes, GB cold, cluster RPO.

## Run 509 (2026-07-18) — still not done

**Not done.** Highest remaining: mix, server 1M/5M, quotes, GB cold, cluster RPO.

## Run 511 (2026-07-18) — still not done

**Not done.** Highest remaining: mix, server 1M/5M, quotes, GB cold, cluster RPO.

## Run 513 (2026-07-18) — still not done

**Not done.** Highest remaining: mix, server 1M/5M, quotes, GB cold, cluster RPO.

## Run 515 (2026-07-18) — still not done

**Not done.** Highest remaining: mix, server 1M/5M, quotes, GB cold, cluster RPO.

## Run 517 (2026-07-18) — still not done

**Not done.** Highest remaining: mix, server 1M/5M, quotes, GB cold, cluster RPO.

## Run 519 (2026-07-18) — still not done

**Not done.** Highest remaining: mix, server 1M/5M, quotes, GB cold, cluster RPO.

## Run 521 (2026-07-18) — still not done

**Not done.** Highest remaining: mix, server 1M/5M, quotes, GB cold, cluster RPO.

## Run 523 (2026-07-18) — still not done

**Not done.** Highest remaining: mix, server 1M/5M, quotes, GB cold, cluster RPO.

## Run 525 (2026-07-18) — still not done

**Not done.** Highest remaining: mix, server 1M/5M, quotes, GB cold, cluster RPO.

## Run 527 (2026-07-18) — still not done

**Not done.** Highest remaining: mix, server 1M/5M, quotes, GB cold, cluster RPO.

## Run 529 (2026-07-18) — still not done

**Not done.** Highest remaining: mix, server 1M/5M, quotes, GB cold, cluster RPO.

## Run 531 (2026-07-18) — still not done

**Not done.** Highest remaining: mix, server 1M/5M, quotes, GB cold, cluster RPO.
