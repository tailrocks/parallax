# Grafana Tempo v3.0.0 Architecture Review

<!-- markdownlint-disable MD013 -->

Research date: 2026-05-29
Primary source: [Tempo v3.0.0 release](https://github.com/grafana/tempo/releases/tag/v3.0.0)

## Purpose

Grafana Tempo is the closest mature OSS analogue to one slice of Parallax: a
self-hosted, object-storage-backed, OpenTelemetry-native trace store with a
columnar Parquet block format and a query language over traces. The **v3.0.0**
release is a major architectural cut — it removes the legacy ingester write path,
ships a new Kafka-log-based ingest/write path with a "live-store", graduates
TraceQL Metrics to GA, lands `vParquet5`, and promotes trace **redaction** to a
first-class block-rewrite job.

This note answers the operator's question: **what did Tempo 3.0 change, how does
it differ from what Parallax is researching, and what can we borrow?**

Bottom line up front: Tempo is **not a Parallax-wedge competitor** — it has no
evidence bundle, no deterministic error grouping, no agent/MCP surface, and no
fix-outcome loop, so it does **not** move the [GO verdict](../decisions/go-no-go.md).
But it is a strong **architectural reference** on four fronts Parallax has open
gates on: the object-storage Parquet block format, the Kafka-WAL ingest path,
explicit freshness/lag SLOs, and retroactive redaction as a job. Three borrows
are high-value; the rest is confirmation that Parallax's lean is sound.

## What v3.0.0 Changed

| Area | v3.0.0 change | Source signal |
| --- | --- | --- |
| **Write path** | Legacy ingester module **removed**; new ingest/write path (Kafka log → block-builder → live-store) is the only path. Old deployments must migrate or fail to start. | Breaking change |
| **Storage format** | `vParquet5`: doubled max dedicated string columns, new timestamp columns, faster metrics read path. **v2 block encoding + compactor removed**. | Format change |
| **Query** | **TraceQL Metrics GA** (was experimental). Experimental faster read path via query hints (unsafe hints must be enabled). | Feature GA |
| **Redaction** | Extended `TraceRedactor` interface can hide **complete** traces; new `tempo-cli redact` command submits redaction jobs. | New capability |
| **Cardinality** | `max_cardinality_per_label`; drain-based limiter clusters similar span names to sanitize cardinality. | New control |
| **Freshness/lag** | Recent-data queries **fail** when instances lag; `query_end_cutoff` defaults to 30s; live-store readiness gated by `readiness_target_lag`/`readiness_max_wait`; `fail_on_high_lag` for Kafka lag. | Reliability semantics |
| **Live-store internals** | Lock-free block reads via atomic pointers; two-phase crash-safe deletion; WAL block dedup (`tempo_block_builder_spans_deduped_total`); async Parquet read on WAL completion; removed explicit `runtime.GC()`. | Perf/robustness |
| **Profiling** | `span_profiling: true` attaches pprof labels to OTel spans via otelpyroscope. | Evidence enrichment |
| **Ops** | KEDA HPA via jsonnet; `automemlimit` (auto `GOMEMLIMIT`). | Deploy |
| **Removed** | OpenCensus receiver; metrics-generator localblocks processor; `querier.query_live_store`; `query_frontend.rf1_after` deprecated (all blocks queried regardless of RF); 32-bit ARM archives. | Deprecations |

## How It Differs From What Parallax Is Researching

| Dimension | Tempo 3.0 | Parallax | Read |
| --- | --- | --- | --- |
| Signal scope | Traces (+ TraceQL Metrics derived from spans) | Errors + logs + traces + metrics + CLI/agent execution traces, correlated into a typed **evidence graph** | Parallax is wider and cross-signal; Tempo is trace-specialized |
| Output | Trace search / TraceQL results / span metrics | **Bounded, redacted, schema-valid evidence bundle** + execution/action/outcome graph | Different product: store vs context engine |
| Storage | `vParquet5` on object storage, custom Go | `StorageAdapter` over **GreptimeDB** (lean) / ClickHouse — Parquet-on-object-store either way | Same physics, different engine |
| Ingest | Kafka log → block-builder → live-store | [Messaging/ingestion layer](../storage/streaming/messaging-and-ingestion-layer.md) under study; [replay/backpressure gate](../storage/streaming/ingest-log-replay-and-backpressure-gate.md) open | Tempo just validated the log-WAL shape in prod |
| Redaction | Post-hoc block-rewrite job; can hide whole traces | [A6](../capture/redaction.md): default-deny **pre**-exposure pipeline + red-team gate | Different point in lifecycle — complementary |
| Agent access | None (Grafana UI / API) | CLI/HTTP first, read-only MCP after safety gates | Parallax-only surface |
| Language | Go | Rust-first | — |

## Can We Borrow? — Yes, Three High-Value, Rest Is Confirmation

**Borrow 1 — Kafka-log ingest as the only write path (high value).** Tempo 3.0
*deleted* the in-memory ingester and made the durable log the single source of
truth, with a block-builder reading the log and a "live-store" serving recent
data. This is exactly the shape the [ingest-log replay/backpressure
gate](../storage/streaming/ingest-log-replay-and-backpressure-gate.md) is
evaluating. Borrow the **architecture decision** (durable log → builder →
recent-data store), the crash-safe **two-phase block deletion**, and the **WAL
block dedup at build time** (Parallax ingests retried Sentry envelopes + OTLP
exports → duplicates are guaranteed; a `spans_deduped`-equivalent metric is
cheap insurance). Note Tempo's earlier `rf1`/replication-factor machinery is now
gone — single-replica log + object storage is enough; this de-risks Parallax's
self-hosted tiny-tier simplicity claim.

**Borrow 2 — explicit freshness/lag SLO with fail-on-lag (high value).** Tempo's
`query_end_cutoff: 30s` + `readiness_target_lag` + `fail_on_high_lag` turn
"freshness" into a *contract*: when the live-store is behind, recent-data queries
**fail loudly** instead of silently returning partial results. The
[freshness-and-latency gate](../storage/freshness-and-latency.md) currently
measures freshness but does not specify failure semantics. Borrow the **explicit
cutoff + fail-on-lag** idea: an evidence bundle built while ingest is lagging
must declare staleness (or refuse), never silently omit signals — a stale bundle
handed to a coding agent is worse than a late one. This also informs the agent
trust boundary: completeness/staleness must be machine-declared in the bundle.

**Borrow 3 — redaction as a retroactive, whole-trace job (medium-high value).**
Tempo's `TraceRedactor` + `tempo-cli redact` can hide a *complete already-stored
trace*. A6 today is a **pre-exposure** default-deny pipeline (leak-out at bundle
time). Tempo proves the *complementary* need: a secret that slipped through
ingest must be scrubbable from already-persisted blocks on demand (right-to-erasure,
post-incident purge). Add a **retroactive redaction/erasure job** to the A6
roadmap as a distinct capability from the runtime pipeline — they are not the
same control. Worth a one-line note in [capture/redaction.md](../capture/redaction.md).

**Confirmation, not borrow:**

- **`vParquet5` (dedicated string columns, timestamp columns).** Confirms
  Parallax's Parquet-on-object-store lean and the value of **promoting
  hot attributes to dedicated columns** rather than leaving them in a generic
  attribute map — relevant to the [GreptimeDB-vs-ClickHouse](../storage/greptimedb-vs-clickhouse/)
  column-layout work and [size-and-object-cost](../storage/size-and-object-cost.md).
  Parallax gets dedicated columns "for free" via GreptimeDB/ClickHouse schema;
  Tempo had to build them by hand. Net: don't hand-roll a block format.
- **Drain-based span-name clustering for cardinality.** Echoes Parallax's
  deterministic error grouping. The drain template-mining algorithm is a known,
  borrowable technique if log/span-message grouping needs a fallback — note it,
  don't adopt yet.
- **Span profiling (pprof-on-span).** A future evidence-enrichment idea for the
  evidence graph; out of current scope.

**Do not borrow:** TraceQL/TraceQL-Metrics as a user query surface — Parallax
serves *anchored bundles*, not ad-hoc trace queries, and the
[agent-access-surface](../decisions/agent-access-surface.md) decision keeps the
first surface read-only and projection-equivalent, not a general query language.
TraceQL Metrics is worth watching only as evidence that span→metric derivation is
a solved, expected capability.

## Verdict and Triggers

Tempo 3.0 **validates** Parallax's storage/ingest physics (durable-log write
path + Parquet on object storage + single-replica simplicity) and surfaces three
concrete borrows (log-WAL ingest shape, fail-on-lag freshness contract,
retroactive redaction job). It does **not** pressure the wedge: no bundle, no
grouping, no agent surface, no fix loop.

Reopen as a competitor-watch item only if Grafana ships, on top of Tempo, a
**portable evidence-bundle artifact + read-only agent/MCP context surface + a
fix-outcome loop** — the same trigger that governs the
[competitor watch](../market/competitor-watch.md). Until then this stays a
reference, tracked here and in [agent-observability-review](agent-observability-review.md).
