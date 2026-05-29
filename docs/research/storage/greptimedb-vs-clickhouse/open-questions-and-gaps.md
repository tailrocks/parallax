# Open Questions & What's Not Yet Addressed (gap ledger)

<!-- markdownlint-disable MD013 -->

Status: created Run 171 — operator asked "what else have we missed?" This consolidates the scattered
open questions + the dimensions the storage research has **not** covered, prioritized. The GreptimeDB-
vs-ClickHouse *engine* comparison is exhaustive + re-verified (Runs 1–170); the gaps below are the parts
that either need data we don't have, are deferred, or sit one layer above the engine.

## 1. THE deciding input we don't have — Parallax's real workload mix

The entire verdict is conditional on **one un-characterized variable**: is Parallax's query distribution
**anchored-retrieval-dominant** (evidence bundles keyed by `trace_id`/`fingerprint` → both engines fast,
GreptimeDB fine) or **ad-hoc-analytics-dominant** (heavy scans/aggregations over large windows →
ClickHouse wins)? Every flip-trigger (`verdict-which-to-choose.md`, `platform-fit-and-alternatives.md`)
points here, and we have **never modelled Parallax's expected query mix**. **This is the highest-value
missing input — the operator (product intent) or a projected-usage model resolves it, not another
benchmark.** Until then the verdict stays "ClickHouse default under the proxy lens; GreptimeDB for the
metrics-cardinality/cost bet," conditioned on this.

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

- **Multi-tenancy / isolation** — Parallax is SaaS-shaped (many customers/projects). How each engine
  isolates tenants (ClickHouse per-DB/table or row-policy; GreptimeDB schemas/catalogs), and the cost of
  isolation at scale. **Decision-relevant + completely untouched.**
- **Auth / access control** — who can read which telemetry; the engines' RBAC vs doing it in the proxy.
- **Backup / disaster recovery** — operational story for each (object-store snapshots, Keeper/metasrv
  state).
- **Rate-limiting / quotas / ingestion protection** — the proxy's protective layer.

## 5. Managed-cloud vs self-host — changes the whole cost/ops calculus

The cost + scaling analysis leaned **self-hosted** (1× vs N× S3, Keeper, manual resharding). But
**ClickHouse Cloud** (`SharedMergeTree` + distributed cache — closes the S3-economics + cold-read gaps,
Run 155/161) and **GreptimeCloud** change the math: they trade $ for ops and erase several of the
self-host edges (e.g. ClickHouse Cloud neutralizes much of GreptimeDB's object-store-economics
advantage). We have **not** modelled managed-vs-self-host — and given the operator's anti-operational-
complexity goal (the anti-self-hosted-Sentry motivation), this may be the *most practically decisive*
axis after workload mix.

## 6. Decided-but-not-designed

- **Metadata store schema** — we decided *where* grouped errors/issues/config live (Turso default /
  Postgres fallback, NOT the columnar engine — `data-model-store-split`), but not the relational model
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

Resolve **#1 (workload mix)** and **#5 (managed-vs-self-host)** first — they are the two inputs that
most move the final decision and need operator/product intent, not more local benchmarking. **#2** is
the server-tier run when the operator is ready. **#3/#4/#6** are the next research/design fronts once the
store decision is locked.
