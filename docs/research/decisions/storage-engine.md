# Storage Engine — GreptimeDB vs ClickHouse

<!-- markdownlint-disable MD013 -->

Decision date baseline: 2026-05-29 (reconciles the engine sub-study to the current operator brief).
Operator re-affirmed focus: 2026-06-03. Operator re-affirmed harder: 2026-06-11 (see below).

> **Decision — current production/server lean GreptimeDB, NOT yet settled.** Keep **both engines behind one
> `StorageAdapter`**; never hard-code engine magic into the schema or the evidence-bundle
> contract. ClickHouse is the fallback and the faster raw analytical engine. The lean is
> GreptimeDB because Parallax's hot path is *anchored* evidence-bundle retrieval (all signals
> for one `trace_id`/`fingerprint`), where **both engines are interactive (≪300 ms at every
> tested scale)** — so ClickHouse's scan-speed lead is off the hot path and the decision turns
> on **cost + Rust + self-hosted**, where GreptimeDB leads. This is finalized only when the
> sized cost numbers and the self-host-vs-managed-cloud call land (below).

The practical server-profile focus is therefore:

> **Implement the first production storage profile around GreptimeDB-shaped assumptions, while preserving the
> ClickHouse adapter boundary.**

This does **not** mean Parallax embeds GreptimeDB into the Parallax process. The local-first V1 should
manage a local GreptimeDB standalone binary for evidence and use Turso/SQLite-like storage for local
metadata/grouping state. This page decides the high-volume self-hosted/server storage profile, where
GreptimeDB is also the first focus. No product contract may depend on GreptimeDB-only behavior. The
bundle schema, context API, grouping semantics, and evidence graph must stay portable enough that
ClickHouse can replace GreptimeDB if the remaining cost/cold-read gates flip.

This is the condensed current verdict. The **full record** — ~170 benchmark runs, a source-level
teardown of both engines, the four-build version matrix, and the per-pass history — lives in
[../storage/greptimedb-vs-clickhouse/](../storage/greptimedb-vs-clickhouse/) (start at
[verdict-which-to-choose.md](../storage/greptimedb-vs-clickhouse/verdict-which-to-choose.md);
history in [run-log.md](../storage/greptimedb-vs-clickhouse/run-log.md); cross-build matrix in
[four-way-version-comparison.md](../storage/greptimedb-vs-clickhouse/four-way-version-comparison.md)).

## Decision questions (DQ1–DQ6), in one table

| # | Question | Answer (mechanism-grounded) |
| --- | --- | --- |
| DQ1 | Where is **GreptimeDB** genuinely better? | Metrics/PromQL-native (GA + default-on); small-write/upsert ingest ergonomics (LSM, no "too many parts"); horizontal scale-out by design (region auto-rebalance, compute/storage separation, no bulk-copy migration); read-time dedup → correct latest-state on a plain query; OTLP schema-drift auto-adds typed columns; retention = whole-SST drop (cheap by default); object-storage-native (fewer objects → wins cold *full* scans); replayable WAL; cardinality-insensitive metric *ingest* (~flat 1k→1M series). |
| DQ2 | Where is **ClickHouse** genuinely better? | Selective log/trace scan + full-text; time-DESC log-tail locality; generic wide-scan/aggregate throughput (decade-tuned C++ vectorized engine, ~2–3× warm metric-agg); per-column codecs; dynamic-attribute JSON path queries (~8× with the required `.:Type` cast); projections (a 2nd physical order); in-DB anchored cross-tier joins; cold *selective* object-store reads (sparse-granule egress); schema-mistake tolerance. The gap **widens with scale** (5M+). |
| DQ3 | Can ClickHouse replace GreptimeDB? | **Yes, technically** — stored every signal, identical bundles — at the cost of a PromQL+OTLP compatibility layer (experimental/collector-only on CH), manual sharding (OSS `SharedMergeTree` is Cloud-only), and an ingest-batching layer. |
| DQ4 | Can GreptimeDB replace ClickHouse? | **Yes** — ran Q1–Q6 with identical results; accept slower heavy ad-hoc log/trace scans. Parallax's anchored hot path is **not latency-bound** (Q6 composite ≪300 ms on both). |
| DQ5 | Which to choose for Parallax today? | **GreptimeDB** on workload fit (metrics-native, ingest/upsert ergonomics, retention cost, scale-out) + the Rust tiebreak; ClickHouse's wins are real but less central to anchored retrieval. |
| DQ6 | Better long-term *investment*? | **GreptimeDB** — the speed gap is **closable engineering, not a physics wall** (seven of eight advantages are pure engineering; the two heaviest ride the shared **DataFusion** scan and **Parquet-Variant** JSON roadmaps), and it is the **Rust, open-source substrate the operator can contribute to** rather than wait on (C++). |

## Why the first production focus is GreptimeDB

Parallax is not choosing the fastest analytical database in the abstract. It is choosing the first
production/server storage substrate for an execution-context product whose critical user action is:

```text
issue / event / trace / fingerprint
  -> fetch related errors, spans, logs, metric windows, deploys, CLI runs, agent actions
  -> assemble one bounded evidence bundle
```

That workload makes GreptimeDB the better first focus for five reasons:

1. **The hot path is anchored, not broad.** ClickHouse's strongest advantage is broad analytical scan,
   log search, dynamic-attribute filtering, and mature SQL throughput. Parallax's first hot path is
   anchored bundle assembly by `trace_id`, `fingerprint`, issue, or narrow time window. Existing local
   benchmark runs show both engines interactive on that path, so ClickHouse's speed lead does not
   decide the server profile.
2. **GreptimeDB matches the observability shape.** Current GreptimeDB docs position it as a unified
   observability database for metrics, logs, and traces, with SQL and PromQL support. That is closer
   to Parallax's retained evidence model than a general analytical warehouse.
3. **Metrics are evidence, not a side quest.** Parallax bundles need metric windows and anomalies beside
   traces/logs/errors. GreptimeDB's PromQL-compatible path makes Prometheus-style metric evidence
   easier to preserve without a separate query layer.
4. **Retention economics matter more than peak scan speed.** Parallax's self-hosted promise depends on
   keeping enough history that bundles remain useful without turning diagnostic data into a cost spike.
   GreptimeDB's cloud-native, disaggregated compute/storage and object-storage-oriented design are the
   reason it gets first focus. This is still a measured claim, not a settled fact, until the sized
   $/GB and cold-read gates close.
5. **Rust is a strategic tie-breaker.** GreptimeDB is the Rust engine the operator can inspect and
   contribute to. ClickHouse is stronger and more mature in many analytical paths, but it is a C++
   substrate. When the hot path is fast enough on both, operator-contributable Rust matters.

This focus should not be misread as "ClickHouse is worse." For Parallax, ClickHouse remains the
fallback when the workload shifts toward heavy ad-hoc analytics, broad log search, or when real
server-tier numbers show no GreptimeDB cost/cold-read advantage.

## Why the lean is GreptimeDB even though ClickHouse is faster

Two lenses once reached opposite defaults; the resolved query mix breaks the tie toward GreptimeDB:

- **Fit + long-term-investment lens → GreptimeDB.** Rust (operator-contributable), object-store-native
  cost, metrics/PromQL-native, scale-out by design; its speed deficits are closable on shared roadmaps.
- **Parallax-as-proxy lens → once leaned ClickHouse.** Because Parallax itself owns OTLP
  ingest/routing/conversion (operator architecture decision, 2026-05-25), GreptimeDB's native-ingest
  edge is neutralized, leaving retrieval speed + build-on-top ecosystem (SigNoz/Uptrace/HyperDX/
  ClickStack) — both ClickHouse wins.
- **The resolver — query mix is RESOLVED (operator 2026-05-29): anchored-bundle-retrieval-dominant.**
  The hot path fetches all signals for one `trace_id`/`fingerprint`/issue to assemble a bundle, not
  broad ad-hoc analytics. On that path **both engines are interactive at every tested scale**, so
  ClickHouse's raw-speed lead is **not decisive for Parallax**. The decision therefore turns on
  **cost + Rust**, where GreptimeDB leads — not on analytical-scan speed, where ClickHouse leads.

## Operator re-affirmation 2026-06-11 (with reality checks)

The operator restated the lean with more conviction: GreptimeDB as the engine to invest in because
it is Rust, already built around the columnar/object-storage concepts ClickHouse proves, and —
since AI agents contribute best to Rust codebases — whatever it still misses versus ClickHouse can
be added over time ("absorb the gap upstream") until one engine serves everything in one place.
This strengthens DQ6 (long-term investment) and is consistent with the
[parity roadmap](../storage/greptimedb-vs-clickhouse/greptimedb-parity-roadmap.md). It does **not**
close the cost/cold-read finalizer gates below, and three reality checks bound the
"AI-extends-it-upstream" strategy (checked 2026-06-11):

1. **Upstream is gated.** GreptimeDB requires a CLA and has an explicit AI-assisted-PR policy:
   authors must understand changes end-to-end, "AI dump" PRs may be closed unreviewed, and review
   capacity is stated as "very limited" ([CONTRIBUTING.md](https://github.com/GreptimeTeam/greptimedb/blob/main/CONTRIBUTING.md)).
   Development is overwhelmingly core-team (~136 contributors, external PRs a small share). The
   strategy is therefore *high-quality AI-assisted contributions with human ownership* — with a
   fork as the hedge, which Apache-2.0 permits.
2. **OSS/Enterprise split matters for the loop.** Triggers/alerting, RBAC, audit logging, and
   read replicas are Enterprise-only ([enterprise docs](https://docs.greptime.com/enterprise/overview/)).
   Parallax must own detection/dispatch in its own workers regardless of engine — which the
   adapter boundary already requires.
3. **Release cadence wobble.** v1.1 GA has not shipped (stable remains v1.0.2, 2026-05-14); the
   nightly line stalled at `v1.1.0-nightly-20260525` while the repo stays highly active — a
   publishing gap, not a dev stall, but the v1.1-GA retest trigger has still not fired. Dynamic
   JSON ("JSON2") is the headline in-flight v1.1 feature (merged PRs May–June 2026), which is the
   exact gap behind ClickHouse's ~8× dynamic-attribute win.

## What must close before this is settled

1. **Sized cost numbers on a real server tier** — $/GB retained, per-signal compression, and
   **multi-replica object-storage cost** (GreptimeDB 1× shared S3 vs OSS ClickHouse N× replica
   copies). The operator's #1 priority and the least-measured axis. Evidence:
   [../storage/size-and-object-cost.md](../storage/size-and-object-cost.md).
2. **Cold-read latency at GB–TB from object storage** — the one regime that could still surprise an
   anchored workload. Evidence: [../storage/freshness-and-latency.md](../storage/freshness-and-latency.md).
3. **Self-hosted vs managed cloud** — strictly self-hosted at scale favors GreptimeDB's 1× object copy
   + compute/storage separation; if ClickHouse Cloud (`SharedMergeTree`) is acceptable, that erases
   GreptimeDB's cost-economics edge.
4. **Re-test on GreptimeDB v1.1 GA** (expected Q2 2026 — narrows the dynamic-JSON gap and may move the
   metrics path; the v1.1 *nightly* is uneven, even regressing 5M dedup-aggregation). Re-pin and re-run
   the load-bearing speed/cost benchmarks when it ships.

## The flip rule (honest guardrail)

Absent a surprise in (1)–(2) or a "yes" to managed cloud in (3), the anchored workload + cost + Rust
point at **GreptimeDB**. **But** if the sized cost numbers come back **at parity** *and* a **managed
path is acceptable**, ClickHouse's ecosystem + speed make it the safer pick — let the **numbers**, not
the Rust preference, settle it. A secondary flip (from the sub-study): if the real query mix turns out
**analytics-/ad-hoc-scan-dominated** *and* GreptimeDB's cold-scan latency at GB–TB is materially worse,
ClickHouse's read-path advantage becomes central.

## Standing maintenance

- Keep both engines behind one `StorageAdapter` trait; no engine magic in the schema or bundle contract.
- Query mix is **resolved** (anchored-retrieval-dominant); the remaining finalizers are the sized cost
  numbers and the self-host-vs-managed-cloud call, not another query-shape model.
- Re-pin versions and re-verify load-bearing claims on each new stable release (GreptimeDB v1.1 GA next).
- **Re-verified 2026-06-03 (official docs + GitHub releases):**
  - GreptimeDB docs still describe a unified observability database for metrics, logs, and traces with
    SQL and PromQL support, OpenTelemetry ingestion paths for metrics/logs/traces, and a distributed
    architecture with region-based sharding and disaggregated compute/storage.
  - ClickHouse docs still describe ClickHouse as a highly efficient observability storage engine with
    strong compression and fast query response, but also state that using it as observability storage
    requires a UI and collection framework; current OTLP usage flows through an OpenTelemetry Collector
    exporter into ClickHouse tables.
  - GitHub releases still show GreptimeDB latest GA `v1.0.2` (2026-05-14) with `v1.1.0-nightly-20260525`
    as a pre-release, so the v1.1-GA retest trigger has not fired.
  - GitHub releases still show ClickHouse `v26.5.1.882-stable` (2026-05-21) as the latest stable feature
    line visible on the releases page, while `v26.3.12.3-lts` is marked latest by GitHub release
    metadata because it is the LTS line.

## Source anchors checked on 2026-06-03

- [GreptimeDB introduction](https://docs.greptime.com/) — unified observability database for metrics,
  logs, and traces; SQL/PromQL positioning.
- [GreptimeDB observability ingest overview](https://docs.greptime.com/user-guide/overview) —
  observability scenario support for metrics, logs, and traces via OpenTelemetry-related tooling.
- [GreptimeDB HTTP / PromQL protocol](https://docs.greptime.com/user-guide/protocols/http) —
  PromQL-compatible query surface.
- [GreptimeDB FAQ](https://docs.greptime.com/faq-and-others/faq) — distributed system, region-based
  sharding, unified metrics/logs/traces model, SQL + PromQL, cloud-native architecture with
  disaggregated compute and storage.
- [GreptimeDB releases](https://github.com/GreptimeTeam/greptimedb/releases) — current GA/pre-release
  pin.
- [ClickHouse observability introduction](https://clickhouse.com/docs/use-cases/observability/build-your-own/introduction) —
  efficient observability storage, fast query response, compression, and need for UI/collection
  framework.
- [ClickHouse OpenTelemetry integration](https://clickhouse.com/docs/use-cases/observability/build-your-own/integrating-opentelemetry) —
  OTLP receiver/exporter path through OpenTelemetry Collector into ClickHouse.
- [ClickHouse observability schema design](https://clickhouse.com/docs/use-cases/observability/build-your-own/schema-design) —
  materialized columns and table design for logs.
- [ClickHouse releases](https://github.com/ClickHouse/ClickHouse/releases) — current stable feature/LTS
  release pins.

## Related records

- V1 implementation stance: [v1-storage-adapter-vision.md](v1-storage-adapter-vision.md).
- Stack roll-up that gates this becoming a stack default: [stack-decision.md](stack-decision.md) (A5).
- Relational metadata store (separate from the columnar engine): [metadata-store.md](metadata-store.md).
- Parity/closability analysis and alternatives survey:
  [../storage/greptimedb-vs-clickhouse/greptimedb-parity-roadmap.md](../storage/greptimedb-vs-clickhouse/greptimedb-parity-roadmap.md),
  [../storage/greptimedb-vs-clickhouse/platform-fit-and-alternatives.md](../storage/greptimedb-vs-clickhouse/platform-fit-and-alternatives.md).
