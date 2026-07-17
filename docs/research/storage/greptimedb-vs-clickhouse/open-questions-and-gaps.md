# Open Questions & What's Not Yet Addressed (gap ledger)

<!-- markdownlint-disable MD013 -->

Status: created Run 171 — operator asked "what else have we missed?" This consolidates the scattered
open questions + the dimensions the storage research has **not** covered, prioritized. The GreptimeDB-
vs-ClickHouse *engine* comparison is exhaustive + re-verified (Runs 1–173; **Run 173 re-pinned** to
GT `v1.1.3` / CH `26.6.1.1193` and confirmed JSON2 closes most of the default-JSON analytics gap);
the gaps below are the parts that either need data we don't have, are deferred, or sit one layer
above the engine.

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
- **Rate-limiting / quotas / ingestion protection** — the proxy's protective layer.

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
