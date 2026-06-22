# Backend & Data-Flow Comparison — How Each Tool Moves Telemetry

Research date: 2026-06-22

This note answers the infrastructure question across the comparison set: **what storage backend
each tool uses, how logs/metrics/traces flow through it from ingest to query, how fast it can go,
and what each design is best for.** It is the backend companion to the feature map in
[observability-feature-matrix.md](observability-feature-matrix.md). Per-tool data-flow schemas also
live inline in each standalone deep-dive (linked below); this note is the side-by-side.

> **All throughput/latency figures below are vendor claims** — no independent third-party benchmark
> was found for any of the six in this pass. Treat them as design intent, not measured guarantees.

## The one-line answer per engine

| Tool | Telemetry engine | Metadata | Broker | Object store | Deploy | Inverted index | RAM floor |
|---|---|---|---|---|---|---|---|
| **Parallax** | **GreptimeDB** | **Turso (libSQL)** | spool (planned) | via GreptimeDB | **1 binary** | GreptimeDB native | low |
| **Maple** | ClickHouse (Tinybird hosted / chDB local) | libSQL/Turso | none | via ClickHouse | 1 binary local / ~5 svc prod | no | n/a |
| **SigNoz** | ClickHouse (+ **ZooKeeper**, not Keeper yet) | SQLite/Postgres | **no Kafka in OSS** | CH→S3 cold tier | 4+ containers | no (sort-key prune + bloom on errors) | ~4 GB |
| **OpenObserve** | Parquet/object-store + **DataFusion** | SQLite/Postgres | NATS (HA only) | **native** | **1 binary (zero-dep)** | **yes — tantivy `.ttv` + bloom, default on** | ~512 MB |
| **Coroot** | ClickHouse (logs/traces/profiles) + **Prometheus** (metrics) | SQLite/Postgres | none | none (core) | 5 containers | no | agents 100–300 MB; CH/TSDB-bound |
| **Sentry** | ClickHouse via **Snuba** | Postgres (+ nodestore, Redis, Memcached) | **Kafka-centric** | local/S3/GCS/SeaweedFS | **~45–50 containers** | no | **16–32 GB** |
| **Gonzo** | **in-memory ring buffer (no store)** | none | none | none | 1 binary | no | tiny/bounded |

Three architectural families fall out:

1. **ClickHouse-backed platforms** — SigNoz, Coroot (partly), Sentry, Maple. Mature columnar OLAP;
   heavy to operate (ClickHouse + coordinator/broker). Sentry is the extreme (Kafka-centric, ~50
   containers). This is the incumbent design.
2. **Parquet-on-object-storage + query engine** — OpenObserve (DataFusion), and the wider 2026
   cohort below (Parseable, Micromegas, Arc, IceGate, smithclay duckdb-otlp). Cheap retention,
   single-binary, schema-on-read. This is where new tools are converging.
3. **Time-series-native engine** — **GreptimeDB** (Parallax, TMA1, OpenFuse). Purpose-built TSDB
   with native OTLP tables, single binary, object-storage tiering. Parallax's bet; now externally
   validated (see "GreptimeDB in the wild").

## Data-flow schemas (ingest → process → store → query)

### Maple — synchronous OTLP, ClickHouse under the hood

```
OTel SDK ─OTLP gRPC :4317 / HTTP :4318─► apps/ingest (key auth, org enrich)
   ─forward OTLP─► OTel Collector ─► ClickHouse  ┌ cloud: Tinybird (Events API, Pipes/SQL)
                                                 └ local: embedded ClickHouse / chDB
   query: apps/web (TanStack SPA) ─HTTP─► apps/api (Effect) ─► ClickHouse
   metadata off-path: libSQL / Turso
```
- **Write:** no broker, no Maple-level WAL — batching/flush/compaction delegated to ClickHouse MergeTree.
- **Read:** ClickHouse columnar scan; queries as Tinybird Pipes (cloud) / embedded engine (local). No inverted index.
- **Throughput (vendor):** "12.8B rows in 198 ms" (query, not ingest). No ingest rate published.
- **Designed for:** ClickHouse-grade scan speed without running ClickHouse yourself + a real single-binary
  local story. **Not for:** self-controlled engine at scale — fast path coupled to hosted Tinybird, no backpressure broker.

### SigNoz — collector → ClickHouse, no Kafka in OSS

```
SDKs ─OTLP/Jaeger/Zipkin─► signoz-otel-collector (receivers → batch / spanmetrics / logpipeline → CH exporters)
   ─► ClickHouse (+ ZooKeeper) ─split at exporter─► signoz_logs / signoz_traces / signoz_metrics ─► S3 cold
   query: React UI ─► signoz binary :8080 (builder→CH SQL, PromQL engine, ruler, alertmanager)
                       ◄─OpAMP pushes log-pipeline config to collector
   metadata: SQLite / Postgres
```
- **Write:** collector batch → CH exporter → MergeTree; `resource_fingerprint` in ORDER BY packs same-source
  rows; per-column codecs (ZSTD/Gorilla/Delta/T64); TTL → S3.
- **Read:** sparse PK index over fingerprint (log block-scan ~100% → <1%); bloom skip-index kept only on the
  error table. Deliberately moved away from bloom-heavy log indexing toward sort-key pruning.
- **Throughput (vendor, 2023 logs-only, ~3 yr stale):** ~55,000 logs/sec; 500 GB → 207.9 GB (~2.4× compression);
  aggregate query 0.48 s vs Elasticsearch 6.49 s. No current trace/metric numbers.
- **Designed for:** unified high-cardinality OTLP logs/traces/metrics in one ClickHouse store at moderate
  scale, self-hosted. **Not for:** single-binary/edge use, best-in-class full-text point lookup, or OSS Kafka backpressure.

### OpenObserve — WAL → memtable → Parquet → object store, DataFusion query

```
ingest (HTTP / OTLP gRPC) ─► ROUTER (proxy)
   ─► INGESTER (parse, VRL functions, schema evolution, real-time alerts)
        WAL (hourly) + Memtable (Arrow RecordBatch) ─5s─► Parquet ─10s push─► OBJECT STORE (Parquet + .ttv index)
                                                                               ◄─ COMPACTOR (merge ≤2GB, retention, file_list)
   query (SQL / PromQL) ─► ROUTER ─► QUERIER (leader): file_list → partition prune → fan-out to worker QUERIERs (gRPC)
        each worker: tantivy .ttv + bloom skip → scan Parquet via DataFusion
```
- **Write:** WAL (hourly) + Arrow memtable → Immutable at 256 MB → Parquet every 5 s → merge+upload every 10 s →
  Compactor merges to ≤2 GB, updates the `file_list` catalog; tantivy `.ttv` pushed alongside each Parquet.
- **Read:** leader querier → `file_list` → partition-prune (`org/stream/Y/M/D/H` + hash + time) → fan to worker
  queriers → scan via DataFusion using bloom (skip files) + tantivy `.ttv` (full-text/secondary).
- **CORRECTION:** OpenObserve **does have an inverted index now** — tantivy integrated 2024 (PR #4733+),
  `.ttv` per Parquet, default on (`ZO_ENABLE_INVERTED_INDEX=true`, fields opt-in) + bloom for high-cardinality
  exact fields. The older "no inverted index, full scan" characterization is outdated.
- **Throughput (vendor):** single-node ~1.8 GB/min ≈ 2.6 TB/day (~31 MB/s on M2); largest cited prod 2+ PB/day;
  "~140×" storage-cost cut vs Elasticsearch; "1 PB scanned in ~2 s". Freshness sub-second–few seconds, durable lag ~10 s.
- **Designed for:** cheap, high-volume, PB-scale observability where storage cost dominates — object-storage Parquet +
  DataFusion + opt-in tantivy/bloom, single binary. **Not for:** ultra-low-latency lookups on un-indexed fields.

### Coroot — agents push to a correlation layer over external engines

```
 coroot-node-agent (eBPF, per-node DaemonSet):
     metrics  ─Prometheus RemoteWrite─►  ┌─────────────┐ ─RW─► Prometheus (or VictoriaMetrics/Thanos/Mimir)
     logs     ─OTLP/HTTP /v1/logs────►  │ coroot:8080 │
     traces   ─OTLP/HTTP /v1/traces──►  │ collector + │ ─SQL─► ClickHouse (otel_logs / otel_traces / profiling_*)
     profiles ─custom HTTP───────────►  │ correlation │
 coroot-cluster-agent (singleton):       │  engine     │ ─config─► SQLite or Postgres
     DB metrics / schema changes ──────► └─────────────┘
 APP (OTel SDK) ─OTLP traces/logs──────────────► coroot:8080
```
- **Write:** agents buffer in local WAL (survive Coroot outage); metrics→Prometheus head/WAL/blocks;
  logs/traces/profiles→ClickHouse MergeTree (~10× compression claim, TTL retention).
- **Read:** metrics via PromQL (largely from local cache); logs/traces/profiles via ClickHouse SQL. Service
  map / RED metrics computed by Coroot, not a graph DB.
- **Throughput (vendor):** node-agent at 10k RPS ≈ 200m CPU (~20% of one core); sustained <0.3 cores; eBPF
  overhead ~+15% vs SDK ~+35–38%.
- **Designed for:** zero-instrumentation, low-overhead Kubernetes/Linux observability; ClickHouse gives cheap
  long retention. **Not for:** being its own scalable TSDB/appliance — you operate Prometheus/VM + ClickHouse yourself.

### Sentry — Kafka eventstream, Relay edge, Snuba→ClickHouse

```
SDK gzip envelope ─► /api/<id>/envelope/ ─► RELAY (Rust): validate DSN, normalize, PII-scrub,
                                            rate-limit (project config from Redis/Memcached), emit outcomes
   ─produce typed msgs─► ═══ KAFKA ingest-* topics (partitioned by project id) ═══
        errors            transactions        profiles            metrics
          ▼                   ▼                  ▼                   ▼
     ingest-consumer     ingest-consumer    ingest-profiles     metrics consumers
       │ preprocess         ▼                  ▼ VROOM (Go)         │
       ▼                                       binary→S3/FS         │
     SYMBOLICATOR (Rust, errors only)                              │
       ▼ save full payload                                         │
     NODESTORE (Postgres self-host / Bigtable SaaS, gzip JSON)     │
                  ═══ KAFKA eventstream (events/transactions/spans) ═══
                           ▼
                     SNUBA CONSUMER (Rust/arroyo) ─batched INSERT─► CLICKHOUSE
                           ▼
                     POST-PROCESS-FORWARDER ─► grouping, alert rules, notifications

 READ: UI/API ─► Sentry web (Django) ─► { Postgres (issues/metadata) | Snuba SnQL/MQL→ClickHouse | nodestore (full body) }
```
- **Write:** envelope → Relay (validate/scrub/rate-limit) → typed Kafka (partitioned by project ID for ordering)
  → consumers symbolicate/group + write full payload to nodestore → Snuba consumers batch Kafka → big ClickHouse
  INSERTs (at-least-once; ReplacingMergeTree dedup → eventually consistent).
- **Read:** issues/metadata from Postgres; aggregates via Snuba SnQL/MQL → ClickHouse SQL; single event from
  nodestore. Spans now in the EAP wide-column store (`spans_v3`), claimed up to 62× faster OLAP.
- **Throughput (vendor):** Relay "hundreds of thousands req/sec" (fleet aggregate); Snuba Rust consumer ~20× vs
  Python. No ingest-to-queryable SLA — eventually consistent, visible lag seconds, grows under backlog.
- **Designed for:** high-cardinality multi-signal *application* observability with a smart edge (Relay normalizes/
  scrubs/quotas at the boundary) + best-in-class error grouping/symbolication. **Not for:** lightweight/single-node —
  a distributed many-service system assuming horizontal scale + dedicated ops.

### Gonzo — no store, in-memory ring buffer

```
Inputs: stdin | file/glob (--follow) | Kubernetes | OTLP receiver (gRPC :4317 / HTTP :4318, logs only)
   ─► Parser (auto-detect JSON/logfmt/plaintext + custom YAML + batch expansion)
   ─► In-memory ring buffer (default 1,000 entries; oldest evicted)
   ─► Analysis: frequency counter (~10,000) | severity classify | Drain3 pattern extraction | attribute discovery
   ─► Output: TUI (Bubble Tea) + embedded web "Dstl8 Lite" (~1s refresh)
   (optional) AI on selected entry → Anthropic / OpenAI / Ollama / LM Studio
```
- **Write:** parse → ring buffer → incrementally update frequency map / severity counters / drain3 templates. No
  WAL, no flush, no compaction — volatile RAM, FIFO eviction.
- **Read:** no query engine; TUI/web renders live aggregates over the in-memory window each ~1 s. No index/SQL/PromQL.
- **Throughput:** none published; RAM bounded by design (ring buffer + frequency map) → predictable small footprint,
  trade-off is zero retention.
- **Designed for:** interactive, ephemeral live log triage at the terminal. **Not for:** persistence, historical
  search, long-window correlation, or trace/metric analysis.

## GreptimeDB in the wild (the storage-bet validation)

Direct evidence that Parallax's engine choice is being made independently by others:

| Project | What it is | GreptimeDB usage | Signal |
|---|---|---|---|
| **TMA1** ([tma1-ai/tma1](https://github.com/tma1-ai/tma1)) | Local-first observability for LLM/AI coding agents | **Embedded GreptimeDB as child process**, data in `~/.tma1/`, single binary, OTLP in (`:14318`), 7 MCP tools incl. `get_context_bundle` | **Near-mirror of Parallax architecture, already shipped.** Closest single competitor/reference found. |
| **OpenFuse** ([tma1-ai/openfuse](https://github.com/tma1-ai/openfuse)) | Langfuse fork, self-hosted LLM observability | **Swaps Langfuse's ClickHouse → GreptimeDB** as source of truth | Proof GreptimeDB works as a drop-in ClickHouse replacement for a major OSS product. Same org as TMA1. |
| **Hebo** ([hebo.ai](https://hebo.ai/blog/260316-greptimedb-observability)) | Embeddable LLM gateway (SaaS) | Observability layer on GreptimeDB; blog "why GreptimeDB instead of ClickHouse" | Second independent team publicly picking GreptimeDB over ClickHouse for telemetry. |
| **greptimedb-mcp-server** ([GreptimeTeam](https://github.com/GreptimeTeam/greptimedb-mcp-server)) | First-party MCP server | AI queries GreptimeDB via SQL/PromQL, read-only enforced, masking, audit log | Greptime itself investing in safe agent/MCP access — aligned with Parallax's read-only projection goal. |

**Implication:** the GreptimeDB bet is no longer contrarian. TMA1 in particular is the most direct
architectural competitor surfaced so far — embedded GreptimeDB + single binary + OTLP-in + MCP-out for
agents. It should get its own deep-dive and a place on the watchlist. See
[missed-similar-tools-2026-06.md](missed-similar-tools-2026-06.md).

## Which design is best for what

- **Cheap PB-scale retention, storage cost dominates** → Parquet-on-object-storage + DataFusion (OpenObserve,
  Parseable, Micromegas, Arc). Single binary, S3-native, schema-on-read.
- **Unified high-cardinality OLAP at moderate-to-large scale, mature ecosystem** → ClickHouse platforms (SigNoz,
  Sentry, Coroot, Maple). Fast aggregates, heavier ops.
- **Error-event workflow + symbolication + grouping** → Sentry's Kafka/Relay/nodestore/Snuba pipeline — purpose-built,
  unmatched for that job, very heavy for everything else.
- **Zero-instrumentation infra/APM** → Coroot's eBPF-agents-over-Prometheus+ClickHouse.
- **Ephemeral live triage, no infra** → Gonzo's in-memory ring buffer.
- **Time-series-native OTLP with native tables + single binary + object tiering** → **GreptimeDB** (Parallax,
  TMA1, OpenFuse). Combines single-binary local-first with a purpose-built TSDB rather than bolting OTLP onto a
  general OLAP store.

## Maintenance note

When a per-tool deep-dive's architecture facts change, update both that doc's data-flow section and the
relevant row here in the same change. The biggest live correction from this pass: **OpenObserve now ships
a tantivy inverted index by default** — the `openobserve-deep-research.md` note has been updated to match.
