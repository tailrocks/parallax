# Open-Source Observability Tools Survey

> Research date: 2026-05-31
> Scope: Open-source, self-hostable tools for observability, error tracking, distributed tracing, log management, AI-powered debugging, MCP-based observability, Sentry-compatible ingestion, and OTLP-native telemetry.
> Excluded: OpenObserve, SigNoz, Coroot, Maple, Bugsink, Rustrak, Traceway, GoSnag, Urgentry, Sentry, Datadog, Grafana/Loki/Tempo/Mimir, New Relic, Dynatrace, Splunk, Dash0, GlitchTip, HyperDX, Uptrace, ClickStack, LangSmith, Langfuse, Phoenix, Braintrust, AgentOps, BuildPulse, Trunk, CloudBees, Colimit, Daxtack, neverbreak, WarpFix, UnfoldCI, Robusta, HolmesGPT, Causely.

---

## Tier 1: Major Platforms (10k+ stars, established)

### 1. Netdata
- **URL**: https://netdata.cloud
- **GitHub**: https://github.com/netdata/netdata
- **Stars**: ~79k
- **What it does**: Real-time, AI-powered full-stack observability with per-second metrics collection, automated anomaly detection, and zero-configuration dashboards. Single binary deployment.
- **Language/Stack**: C, Python, JavaScript
- **License**: GPL v3
- **MCP/AI features**: Yes — has MCP topic tag, AI-powered anomaly detection
- **Sentry protocol**: No
- **OTLP support**: Yes (via OpenTelemetry collector integration)
- **Activity**: Extremely active, updated daily

### 2. Apache SkyWalking
- **URL**: https://skywalking.apache.org
- **GitHub**: https://github.com/apache/skywalking
- **Stars**: ~24.8k
- **What it does**: Production-grade APM platform for distributed systems. Service mesh observability, tracing, metrics, logging, and eBPF agent. CNCF graduated project.
- **Language/Stack**: Java
- **License**: Apache 2.0
- **MCP/AI features**: No
- **Sentry protocol**: No
- **OTLP support**: Yes (OTLP receiver)
- **Activity**: Active, regular releases

### 3. Jaeger
- **URL**: https://jaegertracing.io
- **GitHub**: https://github.com/jaegertracing/jaeger
- **Stars**: ~22.8k
- **What it does**: CNCF graduated distributed tracing platform. End-to-end distributed tracing with storage backends (Elasticsearch, Cassandra, Kafka). Native OpenTelemetry support.
- **Language/Stack**: Go
- **License**: Apache 2.0
- **MCP/AI features**: No
- **Sentry protocol**: No
- **OTLP support**: Yes (native OTLP ingestion)
- **Activity**: Very active

### 4. Vector (Datadog)
- **URL**: https://vector.dev
- **GitHub**: https://github.com/vectordotdev/vector
- **Stars**: ~22k
- **What it does**: High-performance observability data pipeline. Collect, transform, and route logs, metrics, and traces. Agent or aggregator mode. 10x faster than alternatives.
- **Language/Stack**: Rust
- **License**: MPL-2.0
- **MCP/AI features**: No
- **Sentry protocol**: No
- **OTLP support**: Yes (OTLP source and sink)
- **Activity**: Very active (maintained by Datadog's Community Open Source team)

### 5. Zipkin
- **URL**: https://zipkin.io
- **GitHub**: https://github.com/openzipkin/zipkin
- **Stars**: ~17.4k
- **What it does**: Classic distributed tracing system. Collects timing data to troubleshoot latency problems in service architectures.
- **Language/Stack**: Java
- **License**: Apache 2.0
- **MCP/AI features**: No
- **Sentry protocol**: No
- **OTLP support**: Limited (via Brave instrumentation + collectors)
- **Activity**: Low-moderate (mature project, fewer updates)

### 6. VictoriaMetrics
- **URL**: https://victoriametrics.com
- **GitHub**: https://github.com/VictoriaMetrics/VictoriaMetrics
- **Stars**: ~17.1k
- **What it does**: Fast, cost-effective time series database and monitoring solution. Prometheus-compatible with long-term storage. Supports OpenTelemetry metrics.
- **Language/Stack**: Go
- **License**: Apache 2.0
- **MCP/AI features**: No
- **Sentry protocol**: No
- **OTLP support**: Yes (OTLP ingestion for metrics)
- **Activity**: Very active

### 7. Dianping CAT
- **URL**: https://github.com/dianping/cat
- **GitHub**: https://github.com/dianping/cat
- **Stars**: ~18.9k
- **What it does**: Real-time APM platform from Meituan-Dianping. Multi-language client support (Java, C/C++, Node.js, Python, Go). Provides metrics, health monitoring, real-time alerting.
- **Language/Stack**: Java
- **License**: Apache 2.0
- **MCP/AI features**: No
- **Sentry protocol**: No
- **OTLP support**: No
- **Activity**: Low (last updated 2025, mostly stable)

### 8. Pinpoint APM
- **URL**: https://pinpoint-apm.github.io
- **GitHub**: https://github.com/pinpoint-apm/pinpoint
- **Stars**: ~13.8k
- **What it does**: APM tool for large-scale distributed systems written in Java/PHP. Provides server-map, transaction tracing, and inspector views.
- **Language/Stack**: Java
- **License**: Apache 2.0
- **MCP/AI features**: No
- **Sentry protocol**: No
- **OTLP support**: No (uses its own agent protocol)
- **Activity**: Moderate

### 9. Nightingale (FlashCat)
- **URL**: https://n9e.github.io
- **GitHub**: https://github.com/ccfos/nightingale
- **Stars**: ~13k
- **What it does**: Monitoring and alerting platform — "what Grafana is to visualization, Nightingale is to monitoring." Fork of Open-Falcon with unified metrics/alerting/dashboarding.
- **Language/Stack**: Go
- **License**: Apache 2.0
- **MCP/AI features**: No
- **Sentry protocol**: No
- **OTLP support**: No (Prometheus-remote-write compatible)
- **Activity**: Active

### 10. Kubeshark
- **URL**: https://kubeshark.co
- **GitHub**: https://github.com/kubeshark/kubeshark
- **Stars**: ~11.9k
- **What it does**: eBPF-powered network observability for Kubernetes. Deep L4/L7 traffic inspection, TLS decryption without keys, AI agent querying via MCP protocol.
- **Language/Stack**: Go
- **License**: Apache 2.0
- **MCP/AI features**: **Yes — MCP server for AI agent querying of network traffic**
- **Sentry protocol**: No
- **OTLP support**: No
- **Activity**: Active

### 11. Quickwit
- **URL**: https://quickwit.io
- **GitHub**: https://github.com/quickwit-oss/quickwit
- **Stars**: ~11.3k
- **What it does**: Cloud-native search engine for observability. Alternative to Elasticsearch/Loki/Tempo for logs and traces. Sub-second search on cloud storage (S3, GCS, Azure). ES-compatible API.
- **Language/Stack**: Rust
- **License**: Apache 2.0
- **MCP/AI features**: No
- **Sentry protocol**: No
- **OTLP support**: Yes (native OTEL for logs and traces)
- **Activity**: Very active

---

## Tier 2: Established Platforms (1k-10k stars)

### 12. Highlight.io
- **URL**: https://highlight.io
- **GitHub**: https://github.com/highlight/highlight
- **Stars**: ~9.3k
- **What it does**: Open-source full-stack monitoring: error monitoring, session replay, logging, and distributed tracing. Frontend and backend coverage.
- **Language/Stack**: TypeScript, Go
- **License**: SSPL-1.0 (source-available, not OSI-approved)
- **MCP/AI features**: No
- **Sentry protocol**: No (own SDK)
- **OTLP support**: Yes (for traces)
- **Activity**: Active
- **Self-hostable**: Yes (hobby Docker, enterprise self-hosted)

### 13. DeepFlow
- **URL**: https://deepflow.io
- **GitHub**: https://github.com/deepflowio/deepflow
- **Stars**: ~4.1k
- **What it does**: eBPF-based zero-code observability. Auto-metrics, auto-tracing, auto-profiling without any code changes. Full-stack from application to infrastructure including GPU/CUDA. ACM SIGCOMM 2023 paper.
- **Language/Stack**: Go, Rust, C
- **License**: Apache 2.0
- **MCP/AI features**: No
- **Sentry protocol**: No
- **OTLP support**: Yes (serves as storage backend for OTEL, Prometheus, SkyWalking)
- **Activity**: Very active

### 14. DeepOps (DeepFlow community spinoff)
- **URL**: https://github.com/deepops-ai/deepops
- **GitHub**: https://github.com/deepops-ai/deepops
- **Stars**: ~4k
- **What it does**: Observe any stack, any service, any data using any UI components. X-factor detection and resolution before they become problems.
- **Language/Stack**: TypeScript
- **License**: Check repo
- **MCP/AI features**: Unknown
- **Sentry protocol**: No
- **OTLP support**: Yes (OpenTelemetry integration)
- **Activity**: Low (last updated March 2025)

### 15. Erda
- **URL**: https://github.com/erda-project/erda
- **GitHub**: https://github.com/erda-project/erda
- **Stars**: ~2.7k
- **What it does**: Enterprise-grade cloud-native application platform for Kubernetes. Includes APM, CI/CD, microservice governance, and observability.
- **Language/Stack**: Go
- **License**: Apache 2.0
- **MCP/AI features**: No
- **Sentry protocol**: No
- **OTLP support**: Partial
- **Activity**: Moderate

### 16. OpenLIT
- **URL**: https://openlit.io
- **GitHub**: https://github.com/openlit/openlit
- **Stars**: ~2.5k
- **What it does**: OpenTelemetry-native AI engineering platform. LLM observability, GPU monitoring, guardrails, evaluations, prompt management, vault. Integrates with 50+ LLM providers, vector DBs, agent frameworks. SDKs for Python, TypeScript, Go.
- **Language/Stack**: TypeScript (controller), Python/TypeScript/Go (SDKs)
- **License**: Apache 2.0
- **MCP/AI features**: **Yes — AI-native observability for LLMs, guardrails, evaluations**
- **Sentry protocol**: No
- **OTLP support**: **Yes — fully OpenTelemetry-native, follows official semantic conventions**
- **Activity**: Very active
- **Self-hostable**: Yes (Docker Compose, Kubernetes Helm)

### 17. Gonzo
- **URL**: https://github.com/control-theory/gonzo
- **GitHub**: https://github.com/control-theory/gonzo
- **Stars**: ~2.7k
- **What it does**: Go-based TUI log analysis tool with AI (OpenAI/Ollama) integration. Analyze logs from terminal with natural language queries.
- **Language/Stack**: Go
- **License**: Check repo
- **MCP/AI features**: **Yes — AI-powered log analysis (OpenAI, Ollama)**
- **Sentry protocol**: No
- **OTLP support**: Yes (OTLP tag)
- **Activity**: Moderate

### 18. SigLens (ARCHIVED)
- **URL**: https://siglens.com
- **GitHub**: https://github.com/siglens/siglens
- **Stars**: ~1.7k
- **What it does**: Single-binary observability for logs, metrics, traces. 100x more efficient than Splunk. Supports Splunk SPL and SQL query languages. ARCHIVED March 2026.
- **Language/Stack**: Go
- **License**: Apache 2.0 (changed at archival)
- **MCP/AI features**: No
- **Sentry protocol**: No
- **OTLP support**: Yes
- **Activity**: Archived (read-only)

### 19. Measure
- **URL**: https://measure.sh
- **GitHub**: https://github.com/measure-sh/measure
- **Stars**: ~1.3k
- **What it does**: Complete mobile app monitoring: crash reporting, ANR tracking, bug reporting, performance tracing, logging. Open-source Firebase Crashlytics alternative. Android, iOS, Flutter, React Native.
- **Language/Stack**: TypeScript, Go, Kotlin, Swift, Dart
- **License**: Apache 2.0
- **MCP/AI features**: No
- **Sentry protocol**: No
- **OTLP support**: No
- **Activity**: Very active

### 20. Evlog
- **URL**: https://github.com/HugoRCD/evlog
- **GitHub**: https://github.com/HugoRCD/evlog
- **Stars**: ~1.4k
- **What it does**: Wide events and structured errors for TypeScript. TypeScript-first observability for every runtime. Sentry-compatible error handling, OTLP export.
- **Language/Stack**: TypeScript
- **License**: Check repo
- **MCP/AI features**: No
- **Sentry protocol**: Sentry-compatible error handling
- **OTLP support**: Yes
- **Activity**: Active

### 21. OpenTelemetry Collector
- **URL**: https://opentelemetry.io/docs/collector/
- **GitHub**: https://github.com/open-telemetry/opentelemetry-collector
- **Stars**: ~7.1k
- **What it does**: Vendor-agnostic telemetry data pipeline. Receives, processes, and exports traces, metrics, and logs. The backbone of modern OTel-native observability stacks.
- **Language/Stack**: Go
- **License**: Apache 2.0
- **MCP/AI features**: No
- **Sentry protocol**: No (but can be extended with receivers)
- **OTLP support**: **Yes — this IS the reference OTLP implementation**
- **Activity**: Extremely active (CNCF project)

---

## Tier 3: Emerging/Niche Tools (<1k stars)

### 22. Tower (Elixir)
- **GitHub**: https://github.com/mimiquate/tower
- **Stars**: ~191
- **What it does**: Vendor-neutral exception/error tracking and reporting in Elixir. Pluggable backends (TowerRollbar, TowerSentry, TowerEmail).
- **Language/Stack**: Elixir
- **License**: Check repo
- **MCP/AI features**: No
- **Sentry protocol**: Yes (via tower_sentry adapter)
- **OTLP support**: No
- **Activity**: Active

### 23. Temps
- **GitHub**: https://github.com/gotempsh/temps
- **Stars**: ~459
- **What it does**: "The PaaS you actually own." Self-hosted deployment platform with session replay and error tracking built in.
- **Language/Stack**: Rust
- **License**: Check repo
- **MCP/AI features**: No
- **Sentry protocol**: No
- **OTLP support**: No
- **Activity**: Active

### 24. Logwell
- **GitHub**: https://github.com/Divkix/Logwell
- **Stars**: ~44
- **What it does**: Self-hosted logging platform with real-time streaming, full-text search, and OTLP-compatible ingestion. PostgreSQL-backed. Deploy in minutes.
- **Language/Stack**: TypeScript
- **License**: Check repo
- **MCP/AI features**: No
- **Sentry protocol**: No
- **OTLP support**: **Yes — OTLP-compatible ingestion**
- **Activity**: Active

### 25. Otelite
- **GitHub**: https://github.com/planetf1/otelite
- **Stars**: ~59
- **What it does**: Lightweight OpenTelemetry receiver and local dashboard for LLM development. Single binary, zero dependencies, SQLite-backed. Built for local dev observability of AI apps.
- **Language/Stack**: Rust
- **License**: Check repo
- **MCP/AI features**: LLM-focused observability
- **Sentry protocol**: No
- **OTLP support**: **Yes — lightweight OTLP receiver**
- **Activity**: Active

### 26. Waggle
- **GitHub**: https://github.com/danielloader/waggle
- **Stars**: ~14
- **What it does**: Local OpenTelemetry viewer with Honeycomb-style query builder. OTLP/HTTP ingest into SQLite, trace waterfall, FTS5 log search. Single static binary.
- **Language/Stack**: Go, React
- **License**: Check repo
- **MCP/AI features**: No
- **Sentry protocol**: No
- **OTLP support**: **Yes — OTLP/HTTP ingestion**
- **Activity**: Active

### 27. Faze
- **GitHub**: https://github.com/ErickJ3/faze
- **Stars**: ~21
- **What it does**: Local-first observability for developers. Rust CLI for viewing metrics, logs, and traces.
- **Language/Stack**: Rust
- **License**: Check repo
- **MCP/AI features**: No
- **Sentry protocol**: No
- **OTLP support**: Yes
- **Activity**: Active

### 28. Crashlens
- **GitHub**: https://github.com/Crashlens/crashlens
- **Stars**: ~14
- **What it does**: Production LLM observability CLI. Detects token waste, retry loops, and model overkill across OpenAI/Anthropic/Gemini. Prometheus metrics and Grafana dashboards included.
- **Language/Stack**: Python
- **License**: Check repo
- **MCP/AI features**: LLM observability focused
- **Sentry protocol**: No
- **OTLP support**: No (Prometheus metrics)
- **Activity**: Low (last updated Dec 2025)

### 29. OTel-Front
- **GitHub**: https://github.com/mesaglio/otel-front
- **Stars**: ~104
- **What it does**: Lightweight OpenTelemetry viewer for local development. View traces, logs, and metrics instantly.
- **Language/Stack**: Go, TypeScript
- **License**: Check repo
- **MCP/AI features**: No
- **Sentry protocol**: No
- **OTLP support**: Yes
- **Activity**: Active

### 30. No-No Debug
- **GitHub**: https://github.com/summerliuuu/no-no-debug
- **Stars**: ~150
- **What it does**: Self-evolution system for AI coding assistants. Makes AI coding agents remember all their bugs and learn from mistakes.
- **Language/Stack**: Python
- **License**: Check repo
- **MCP/AI features**: **Yes — AI agent self-improvement through error memory**
- **Sentry protocol**: No
- **OTLP support**: No
- **Activity**: Active

### 31. Honeybadger MCP
- **GitHub**: https://github.com/vishalzambre/honeybadger-mcp
- **Stars**: ~6
- **What it does**: MCP integration connecting Honeybadger error tracking with Cursor IDE. Fetch, analyze, and manage error reports within dev environment.
- **Language/Stack**: TypeScript, Node.js
- **License**: Check repo
- **MCP/AI features**: **Yes — MCP server for error tracking integration**
- **Sentry protocol**: No (Honeybadger protocol)
- **OTLP support**: No
- **Activity**: Low

### 32. Rails Error Dashboard
- **GitHub**: https://github.com/AnjanJ/rails_error_dashboard
- **Stars**: ~80
- **What it does**: Self-hosted error tracking for Rails. Zero recurring cost, full data ownership. Rails engine.
- **Language/Stack**: Ruby (Rails)
- **License**: Check repo
- **MCP/AI features**: No
- **Sentry protocol**: No
- **OTLP support**: No
- **Activity**: Active

---

## Additional Tools Worth Noting (not found in searches but known)

These tools are relevant but were not surfaced by the GitHub topic searches:

### 33. Parca
- **URL**: https://parca.dev
- **GitHub**: https://github.com/parca-dev/parca
- **What it does**: Continuous profiling for infrastructure and applications. eBPF-based, zero-instrumentation profiling.
- **Language/Stack**: Go
- **License**: Apache 2.0
- **OTLP support**: Yes (can integrate with OTel)

### 34. Fluentd / Fluent Bit
- **URL**: https://www.fluentd.org / https://fluentbit.io
- **GitHub**: https://github.com/fluent/fluentd / https://github.com/fluent/fluent-bit
- **What it does**: Log collection and forwarding. Fluent Bit is the lightweight C-based version. Both are CNCF projects.
- **Language/Stack**: Ruby (Fluentd) / C (Fluent Bit)
- **License**: Apache 2.0
- **OTLP support**: Yes (via plugins)

### 35. Graylog
- **URL**: https://graylog.org
- **GitHub**: https://github.com/Graylog2/graylog2-server
- **What it does**: Log management platform with search, analysis, and alerting. Enterprise features available.
- **Language/Stack**: Java
- **License**: SSPL (source-available)
- **OTLP support**: Yes

### 36. Apache Flume
- **URL**: https://flume.apache.org
- **GitHub**: https://github.com/apache/flume
- **What it does**: Distributed log collection and aggregation system.
- **Language/Stack**: Java
- **License**: Apache 2.0

### 37. Tremor
- **URL**: https://www.tremor.rs
- **GitHub**: https://github.com/tremor-rs/tremor
- **What it does**: Event processing system for observability data pipelines. Rust-based, handles logs, metrics, and traces.
- **Language/Stack**: Rust
- **License**: Apache 2.0
- **OTLP support**: Yes

---

## Summary Matrix: Key Differentiators

| Tool | Category | Stars | Lang | MCP/AI | OTLP | Sentry Protocol | Self-Host | License |
|------|----------|-------|------|--------|------|-----------------|-----------|---------|
| Netdata | Full-stack monitoring | 79k | C | Yes | Partial | No | Yes | GPL-3.0 |
| SkyWalking | APM/Tracing | 24.8k | Java | No | Yes | No | Yes | Apache 2.0 |
| Jaeger | Distributed tracing | 22.8k | Go | No | Yes | No | Yes | Apache 2.0 |
| Vector | Data pipeline | 22k | Rust | No | Yes | No | Yes | MPL-2.0 |
| CAT | APM | 18.9k | Java | No | No | No | Yes | Apache 2.0 |
| Zipkin | Distributed tracing | 17.4k | Java | No | Partial | No | Yes | Apache 2.0 |
| VictoriaMetrics | TSDB/Monitoring | 17.1k | Go | No | Yes | No | Yes | Apache 2.0 |
| Pinpoint | APM | 13.8k | Java | No | No | No | Yes | Apache 2.0 |
| Nightingale | Monitoring/Alerting | 13k | Go | No | No | No | Yes | Apache 2.0 |
| Kubeshark | Network observability | 11.9k | Go | **Yes (MCP)** | No | No | Yes | Apache 2.0 |
| Quickwit | Log/Trace search | 11.3k | Rust | No | Yes | No | Yes | Apache 2.0 |
| OTel Collector | Telemetry pipeline | 7.1k | Go | No | Yes | No | Yes | Apache 2.0 |
| Highlight.io | Full-stack monitoring | 9.3k | TS/Go | No | Yes | No | Yes | SSPL |
| DeepFlow | eBPF observability | 4.1k | Go/Rust | No | Yes | No | Yes | Apache 2.0 |
| OpenLIT | AI/LLM observability | 2.5k | TS/Python | **Yes** | Yes | No | Yes | Apache 2.0 |
| Gonzo | AI log analysis | 2.7k | Go | **Yes** | Yes | No | Yes | - |
| Erda | Platform + APM | 2.7k | Go | No | Partial | No | Yes | Apache 2.0 |
| SigLens | Log management | 1.7k | Go | No | Yes | No | Yes | Apache 2.0 |
| Evlog | Structured logging | 1.4k | TS | No | Yes | Partial | Yes | - |
| Measure | Mobile monitoring | 1.3k | TS/Go | No | No | No | Yes | Apache 2.0 |
| Tower | Elixir error tracking | 191 | Elixir | No | No | Yes | Yes | - |
| Temps | PaaS + error tracking | 459 | Rust | No | No | No | Yes | - |
| Otelite | Lightweight OTel | 59 | Rust | No | Yes | No | Yes | - |
| Logwell | Log management | 44 | TS | No | Yes | No | Yes | - |
| Waggle | Local OTel viewer | 14 | Go | No | Yes | No | Yes | - |
| Faze | Local observability | 21 | Rust | No | Yes | No | Yes | - |
| OTel-Front | Local OTel viewer | 104 | Go/TS | No | Yes | No | Yes | - |
| No-No Debug | AI error memory | 150 | Python | **Yes** | No | No | Yes | - |
| Parca | Continuous profiling | - | Go | No | Partial | No | Yes | Apache 2.0 |
| Tremor | Event processing | - | Rust | No | Yes | No | Yes | Apache 2.0 |
| Graylog | Log management | - | Java | No | Yes | No | Yes | SSPL |

---

## Key Findings

### MCP/AI-Native Tools
Only a handful of tools in the open-source space have explicit MCP or AI agent features:
1. **Kubeshark** — MCP server for querying K8s network traffic via AI agents
2. **Netdata** — AI-powered anomaly detection + MCP topic tag
3. **OpenLIT** — AI-native LLM observability with evaluations and guardrails
4. **Gonzo** — AI-powered TUI log analysis (OpenAI/Ollama)
5. **No-No Debug** — AI agent self-improvement through error memory
6. **Honeybadger MCP** — MCP integration for error tracking in Cursor IDE
7. **Otelite** — LLM development-focused lightweight OTel receiver

### Sentry-Compatible Error Ingestion
Very few tools support the Sentry SDK protocol:
1. **Tower (Elixir)** — via tower_sentry adapter
2. **Evlog** — Sentry-compatible error handling
3. (Most tools in the exclusion list like GlitchTip, Bugsink, Rustrak already cover this niche)

### OTLP-Native Tools (strongest OTel alignment)
1. **Jaeger** — native OTLP ingestion
2. **Quickwit** — native OTEL for logs and traces
3. **OpenLIT** — fully OpenTelemetry-native, follows semantic conventions
4. **DeepFlow** — OTEL storage backend
5. **VictoriaMetrics** — OTLP metrics ingestion
6. **Vector** — OTLP source and sink
7. **OTel Collector** — the reference implementation
8. **Otelite, Waggle, Faze, OTel-Front** — emerging lightweight OTel viewers

### Rust-Based Observability Tools
1. **Vector** (22k stars) — data pipeline
2. **Quickwit** (11.3k) — search engine for observability
3. **DeepFlow** (4.1k) — agent in Rust, server in Go
4. **Temps** (459) — PaaS with error tracking
5. **Otelite** (59) — lightweight OTel receiver
6. **Faze** (21) — local-first observability
7. **Tremor** — event processing

### Gaps in the Market
1. **MCP-based observability** is almost nonexistent as a first-class feature. Kubeshark is the only significant tool with an MCP server.
2. **AI-native debugging** (not just LLM observability) remains nascent. No-No Debug is an interesting early experiment.
3. **Evidence/context engines** for debugging (the kind Causely targets) have no direct open-source equivalent found in this search.
4. **Sentry-compatible ingestion** is a crowded niche but mostly covered by excluded tools. Tower (Elixir) is a notable non-excluded entry.
