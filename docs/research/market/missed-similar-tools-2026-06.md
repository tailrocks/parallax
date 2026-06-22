# Missed Similar Tools — Discovery Sweep (June 2026)

Research date: 2026-06-22

A targeted sweep for open-source / self-hostable tools **similar to our comparison set** (Maple,
SigNoz, OpenObserve, Coroot, Sentry, Gonzo) and to Parallax itself, that were **not yet tracked** —
with a focus on **storage backend** (which engine each uses) per the operator's emphasis. The single
biggest finding: **GreptimeDB-backed tools exist in the wild** (TMA1, OpenFuse, Hebo), validating the
Parallax storage bet. Backend details and the side-by-side feed
[backend-and-data-flow.md](backend-and-data-flow.md).

> Star counts are point-in-time (June 2026) and move fast — treat as order-of-magnitude. Items flagged
> "verify" had a license/stars/feature uncertainty at capture time.

## Tier A — GreptimeDB-backed (highest relevance: same engine as Parallax)

| Tool | What it is | Backend | Lang / License / ★ | OTLP / MCP | Deep-dive |
|---|---|---|---|---|---|
| **TMA1** ([tma1-ai/tma1](https://github.com/tma1-ai/tma1)) | Local-first observability for LLM/AI coding agents (cost, sessions, anomalies, replay) | **Embedded GreptimeDB** (child process, `~/.tma1/`), single binary | Go+JS / Apache-2.0 / ~97 | OTLP `:14318` ✅; **7 MCP tools** incl. `get_context_bundle`, `get_anomalies` | **HIGH — near-mirror of Parallax** |
| **OpenFuse** ([tma1-ai/openfuse](https://github.com/tma1-ai/openfuse)) | Langfuse fork, self-hosted LLM observability | **GreptimeDB** (swaps Langfuse's ClickHouse) | TS / MIT (+EE) / ~6 | OTLP ✅ (Langfuse-compatible) | HIGH — GreptimeDB-as-ClickHouse-drop-in proof |
| **Hebo** ([hebo.ai](https://hebo.ai/)) | Embeddable LLM gateway; observability layer on GreptimeDB | **GreptimeDB** | gateway OSS / SaaS platform | — | MEDIUM — external validation, not adoptable |
| **greptimedb-mcp-server** ([GreptimeTeam](https://github.com/GreptimeTeam/greptimedb-mcp-server)) | First-party MCP over GreptimeDB (SQL/PromQL, read-only, masking, audit) | GreptimeDB | Python / MIT / ~28 | MCP ✅ read-only | MEDIUM — Greptime's own agent surface |

**Why it matters:** TMA1 is the closest architectural competitor found anywhere — embedded GreptimeDB +
single binary + OTLP-in + MCP-out for agents, aimed at AI coding agents. OpenFuse proves GreptimeDB drops
in where ClickHouse was. Both are from the same org (`tma1-ai`). Add TMA1 to the active watchlist.

## Tier B — Parquet / DataFusion / DuckDB storage cohort (the 2026 convergence)

These are the architectural cousins of OpenObserve and of Parallax's lean-storage philosophy — Parquet on
object storage with an embedded query engine, usually single-binary, usually Rust.

| Tool | What it is | Backend | Lang / License / ★ | Notes |
|---|---|---|---|---|
| **Parseable** ([parseablehq/parseable](https://github.com/parseablehq/parseable)) | OTel-native columnar data-lake observability, SQL-first | **Parquet/S3 + Arrow + DataFusion** ("ParseableDB"), no ClickHouse | Rust / **AGPL-3.0** / ~2,391 | Most mature in cohort; single binary, BYO bucket. AGPL vs our Apache-2.0. **Deep-dive: HIGH** |
| **Micromegas** ([madesroches/micromegas](https://github.com/madesroches/micromegas)) | Unified high-volume observability | **Postgres (meta) + Parquet/object store + DataFusion**, FlightSQL | Rust+TS / Apache-2.0 / ~47 | **Nearest storage cousin to Parallax** (separate metadata + DataFusion-over-Parquet, ~20ns/event). Monolith mode WIP. **HIGH** |
| **Arc** ([Basekick-Labs/arc](https://github.com/Basekick-Labs/arc)) | Analytical/time-series DB for metrics/logs/traces | **DuckDB over native Parquet** (S3/Azure/MinIO/local) | Go / AGPL-3.0 + commercial / ~610 | Single binary, air-gap. OTLP unconfirmed (Influx line today — verify). **MEDIUM-HIGH** |
| **smithclay stack** ([duckdb-otlp](https://github.com/smithclay/duckdb-otlp) + [otlp2parquet](https://github.com/smithclay/otlp2parquet)) | OTLP→Arrow→Parquet ingest + DuckDB extension | **DuckDB + Parquet (+ Iceberg/DuckLake)** | Rust / C++ / Apache-2.0 / MIT / ~39 + ~66 | Cleanest readable reference for the **zero-copy OTLP→Arrow→Parquet hot path** Parallax cares about. **MEDIUM-HIGH** |
| **IceGate** ([icegatetech/icegate](https://github.com/icegatetech/icegate)) | "Observability data lake engine" | **Apache Iceberg + Parquet/S3**, WAL, Arrow FlightSQL, ACID | Rust / Apache-2.0 / ~28 | **Same lang + license as Parallax.** "Full transactions without a dedicated OLTP DB" — relevant to our Turso choice. Prototype, 5 components. **MEDIUM** |
| **ai-observer** ([tobilg/ai-observer](https://github.com/tobilg/ai-observer)) | Local observability for AI coding assistants | **DuckDB** | Go / MIT / ~243 | Clean small DuckDB+OTLP example, local-first. **MEDIUM** |
| **otel-front** ([mesaglio/otel-front](https://github.com/mesaglio/otel-front)) | Local OTLP viewer w/ flame graphs | **in-memory DuckDB**, single binary | TS+Go / MIT / ~110 | Good zero-config UX model. MEDIUM |
| **otel-gui** ([metafab/otel-gui](https://github.com/metafab/otel-gui)) | Local OTLP viewer | in-memory or **PGlite (Postgres-in-WASM)** | TS/Svelte / MIT / ~145 | Storage curiosity. LOW-MEDIUM |

**Why it matters:** the new-tool center of gravity in 2026 is **Parquet-on-object-storage + DataFusion/DuckDB,
no ClickHouse**. Parallax's GreptimeDB choice is adjacent (TSDB-native rather than raw Parquet) but shares the
single-binary, object-storage, schema-flexible goals. Micromegas and the smithclay stack are the best *reading
references* for the zero-copy ingest path.

## Tier C — AI-native / MCP / RCA / evidence cohort

Mostly "AI agent that investigates and explains" — usually **no own store** (they query existing backends);
differentiation is in knowledge/graph layers and auditable evidence. MCP is now table-stakes here.

| Tool | What it is | Storage / notable | Lang / License / ★ | Deep-dive |
|---|---|---|---|---|
| **HolmesGPT** ([HolmesGPT/holmesgpt](https://github.com/HolmesGPT/holmesgpt)) | AI agent that investigates incidents → RCA (agentic loop) | none (queries Prometheus/Grafana/Loki/Tempo); strongly MCP | Python / Apache-2.0 / ~2,700 · **CNCF Sandbox** | HIGH — most mature open "AI SRE" |
| **OpenSRE** ([Tracer-Cloud/opensre](https://github.com/Tracer-Cloud/opensre)) | Framework + RL env for AI SRE agents; **replays past failures** to score diagnosis | synthetic incident sims; air-gapped via Ollama | Python / Apache-2.0 / ~7,400 (fast-grown — verify) | HIGH — distinctive trajectory/replay pattern |
| **AgentRx** ([microsoft/AgentRx](https://github.com/microsoft/AgentRx)) | Diagnose **AI-agent failures from execution trajectories**; auditable evidence log for an LLM judge | trajectory analysis; 10-category taxonomy | Python / license+stars unconfirmed (new) | **HIGH — purest evidence-bundle/trajectory match** |
| **kagent** ([kagent-dev/kagent](https://github.com/kagent-dev/kagent)) | K8s-native AI-agent framework (CRDs) | ships MCP server; **native OTel tracing of the agents** | Go / Apache-2.0 / ~3,100 · CNCF | HIGH |
| **Keep** ([keephq/keep](https://github.com/keephq/keep)) | Open AIOps / alert management ("GitHub Actions for monitoring") | MCP across 120+ tools; OTel compose | Python / ~Apache-2.0 (verify) / ~12,000 | MEDIUM-HIGH — largest traction |
| **Aurora** ([Arvo-AI/aurora](https://github.com/Arvo-AI/aurora)) | LangGraph agent for autonomous incident RCA | **Weaviate (vector) + Memgraph (infra graph)**; MCP server | Python / Apache-2.0 / ~320 | MEDIUM — graph-of-infra evidence assembly |
| **IncidentFox** ([incidentfox/incidentfox](https://github.com/incidentfox/incidentfox)) | AI SRE with per-team domain agents | RAPTOR KB + Prophet anomaly + blast-radius mapping | Python / Apache-2.0 + BSL-1.1 / ~631 | MEDIUM |
| **Vespper/Merlinn** ([merlinn-co/merlinn](https://github.com/merlinn-co/merlinn)) | AI on-call engineer, RCA in Slack | **ChromaDB** vector store; RAG over Datadog/GitHub/Jira | TS / Apache-2.0 / ~360 | MEDIUM |

**Why it matters:** AgentRx and OpenSRE are the closest things to Parallax's **evidence-bundle / debugging-trajectory**
thesis — auditable, constraint-backed trajectory evidence as an artifact. They validate the direction while still
not shipping a portable telemetry-evidence bundle in the Parallax sense (Sentry-ingest → OTLP correlation → redacted
versioned bundle → outcome loop).

## Backend fact-check on borderline-known tools

| Tool | Storage engine | OTLP | ★ / License | Note |
|---|---|---|---|---|
| **HyperDX** | **ClickHouse** (+ OTel collector) | ✅ | ~9,612 / MIT | Acquired by ClickHouse May 2025; now the UI of ClickStack. |
| **ClickStack** | **ClickHouse** | ✅ | ~106 / — | Bundle/meta repo (ClickHouse + OTel Collector + HyperDX). |
| **Dash0** | **ClickHouse** | ✅ OTel-native | closed SaaS | Confirmed ClickHouse via eng blog. |
| **Uptrace** | **ClickHouse** (events) + **Postgres** (meta) | ✅ | ~4,227 / AGPL-3.0, Go | v2.0 uses ClickHouse JSON type. |
| **Parseable** | **Parquet/S3 + Arrow/DataFusion** (NOT ClickHouse) | ✅ native | ~2,391 / AGPL-3.0, Rust | Single binary, BYO bucket. |
| **Bugsink** | **SQLite** / Postgres / MySQL | ❌ (Sentry envelope only) | ~1,889 / Django | Single container, no Redis. |
| **GlitchTip** | **Postgres** | ❌ raw OTLP | ~350 on GitLab (canonical) | 2026-04 piece on DuckDB+Parquet cold archives (Rust `arro3`). |
| **Tracetest** | none native (connects to Tempo/Jaeger) | ✅ test ingest | ~1,319 / Go | Trace-based *testing*; possibly dormant (last push 2025-06). |
| **Telebugs** | — | ❌ (Sentry SDK) | proprietary, self-hostable ($299) | **Not FOSS** — redistribution prohibited; flag. |

## Cross-cutting takeaways

1. **GreptimeDB-in-the-wild is real** (TMA1 embedded, OpenFuse swap, Hebo choice) — strong validation of the
   Parallax storage decision; no longer a contrarian bet.
2. **2026 new-tool convergence = Parquet-on-object-storage + DataFusion/DuckDB, no ClickHouse** (Micromegas,
   Parseable, Arc, IceGate, smithclay stack). ClickHouse remains the incumbent for mature platforms.
3. **AI cohort shift:** new tools are "AI agent that investigates and explains," rarely shipping their own store;
   differentiation is knowledge/graph layers (Memgraph, Weaviate, RAPTOR, Chroma) and **auditable evidence**
   (AgentRx, OpenSRE). MCP is table-stakes.
4. **Closest single competitor: TMA1.** **Closest storage-architecture cousin: Micromegas.** **Purest
   evidence-bundle match: AgentRx.**

## Recommended next deep-dives (priority order)

1. **TMA1** — direct architectural competitor (embedded GreptimeDB + single binary + OTLP + MCP for agents).
2. **Micromegas** — nearest storage cousin (DataFusion-over-Parquet + separate metadata).
3. **AgentRx** — evidence-bundle / trajectory-evidence pattern reference.
4. **Parseable / Arc / smithclay duckdb-otlp** — Parquet/DataFusion/DuckDB ingest references.
5. **HolmesGPT / OpenSRE / kagent** — AI/MCP SRE cohort.
6. **OpenFuse** — GreptimeDB-swap proof, worth a short note.

## Verify-before-citing flags

AgentRx license/stars; Keep exact license; Arc OTLP support; OpenSRE star velocity (~7.4k grew fast);
IncidentFox/Vespper MCP specifics. Re-confirm these before promoting any to a committed threat assessment.
