# Coroot Deep Research

Research date: 2026-06-22

Coroot was previously tracked only in the `competitor-watch.md` "Coroot MCP and AI RCA Recheck"
ledger and `reference/ai-native-debugging-tools.md`. This note promotes it to a standalone
deep-dive and refreshes stale facts (the ledger cited v1.20.2 / "Claude 3.7"; current is
**v1.22.2** / **Claude Opus 4.6**). Grounded in current primary sources (coroot.com, docs,
GitHub) checked June 2026.

## Sources

- [github.com/coroot/coroot](https://github.com/coroot/coroot) (+ [releases](https://github.com/coroot/coroot/releases)), [coroot/coroot-node-agent](https://github.com/coroot/coroot-node-agent)
- [coroot.com](https://coroot.com/), [overview](https://coroot.com/overview), [editions](https://coroot.com/editions), [about](https://coroot.com/about)
- [docs.coroot.com/installation/architecture](https://docs.coroot.com/installation/architecture/), [docker](https://docs.coroot.com/installation/docker/)
- [docs.coroot.com/ai](https://docs.coroot.com/ai/) (+ [configuration](https://docs.coroot.com/ai/configuration/)), [mcp/overview](https://docs.coroot.com/mcp/overview/)
- [docs.coroot.com/tracing/ebpf-based-tracing](https://docs.coroot.com/tracing/ebpf-based-tracing/), [opentelemetry-go](https://docs.coroot.com/tracing/opentelemetry-go/)
- [peterzaitsev.com — joining Coroot](https://peterzaitsev.com/joining-coroot-as-co-founder/)

## What Coroot Is

Coroot is an **open-source, eBPF-based observability + APM tool with zero-instrumentation** capture
and AI-powered Root Cause Analysis. It combines metrics, logs, traces, continuous profiling, and
SLO-based alerting with predefined dashboards/inspections. Its wedge is **adoption friction**: install
the eBPF agent and a service map appears with no app code changes.

- **License:** **Apache-2.0** core (Community Edition); open-core — a separate **Enterprise Edition**
  is commercial for SSO, RBAC, and AI RCA.
- **Language:** **Go** (~61%), Vue/TS frontend, eBPF C.
- **Company:** Coroot Inc., Palo Alto. Founder Nikolay Sivko; **Peter Zaitsev** (Percona founder) joined
  as co-founder (2024-01-29) with a personal investment. No public seed figure (uncertain).
- **Maturity:** ~7.8k stars. Latest **v1.22.2 (2026-06-15)**; rapid cadence (multiple releases/month).

## Architecture

Three components:

| Component | Role |
|-----------|------|
| **coroot-node-agent** | eBPF agent per node (K8s DaemonSet) — metrics/logs/traces/profiles from containers. Pull (Prometheus) + push (Prometheus Remote Write, OTLP, custom HTTP). |
| **coroot-cluster-agent** | cluster-wide telemetry: DB metrics (Postgres/MySQL), app CPU/mem profiles, AWS infra (RDS, ElastiCache). |
| **Coroot server** | central app, processes/serves telemetry, UI on port 8080. |

**Storage:** **ClickHouse** (logs/traces/profiles, optional metrics, ~10x compression) + **Prometheus**
(primary metrics; compatible with VictoriaMetrics/Thanos/Mimir). Metadata store not separately documented.

**Deployment:** Docker Compose, Kubernetes/Helm, Docker Swarm, Linux. **Not a single self-contained
binary** — multi-container stack.

## OTLP / OTel Support

- **Both eBPF auto-instrumentation AND OTLP ingestion.**
- **Traces:** OTLP **over HTTP** (apps direct or via OTel Collector); eBPF also generates traces exported
  via OTLP/HTTP.
- **Logs:** OTLP/HTTP; node-agent discovers container logs and ships via OTLP/HTTP.
- **Metrics:** Prometheus scrape + **Prometheus Remote Write**.
- **OTLP gRPC ingestion not clearly confirmed** in reviewed docs (HTTP emphasized) — uncertain.
- **No Sentry-compatible path** — no envelope/DSN ingest; not an error-tracking/issue tool.
- **eBPF spans are explicitly partial** — Coroot's own docs note "eBPF-based spans may not provide
  complete traces"; protocol-level only (HTTP/Postgres/MySQL/Redis/Mongo/Memcached). **No app-level error
  chains / panics / stack traces.**

## Local-Run Story

- **Single docker-compose one-liner:**
  `curl -fsS https://raw.githubusercontent.com/coroot/coroot/main/deploy/docker-compose.yaml | docker compose -f - up -d`
- Brings up **5 containers** (Coroot, ClickHouse, Prometheus, node-agent, cluster-agent). UI at
  `localhost:8080`. Service map auto-populates within minutes via eBPF — best zero-instrumentation
  "see something immediately" story in the field.
- **Footprint:** heavier than a single binary (runs ClickHouse + Prometheus). No official CPU/RAM minimums
  published (uncertain), but meaningfully heavier than a Rust single-binary local-first tool.

## AI / MCP Surface

- **AI RCA — two-stage:** **deterministic ML first** (walks the dependency graph from the affected service,
  compares telemetry to anomaly — **no LLM**), then **LLM summarizes + suggests fixes**. RCA result is
  persisted onto the incident.
- **LLM providers:** Anthropic **Claude Opus 4.6** (recommended), OpenAI **GPT-5.2**, any OpenAI-compatible
  API (tested Gemini, DeepSeek).
- **Edition gating:** AI RCA is **Enterprise** ($1/core/mo) **or** via **Coroot Cloud** for Community
  (**10 free investigations/month**).
- **MCP server — 18 tools, all read-only except one** (`resolve_alerts` mutates). Read tools: `list_projects`,
  `select_project`, `list_applications`, `list_alerts`, `list_incidents`, `list_nodes`,
  `get_application_status`, `get_incident_details`, `get_node_details`, `traces_summary`, `traces_errors`,
  `traces_outliers`, `get_trace`, `query_metrics`, `list_metric_names`, `query_logs`. **Enterprise-only:**
  `list_anomalies`, `investigate_anomaly`.
- **Auth:** OAuth 2.0; each user signs in with their own Coroot account; **agent runs with that user's
  RBAC** (server-side authorization) — strong safety posture, the closest of any tracked tool to a
  read-only RBAC-scoped projection.
- **Clients:** Claude Code, Cursor, Codex.
- **No portable evidence-bundle schema** — live-query MCP + workflow (RCA persisted on incident), not an
  exportable versioned redacted artifact.

## Feature Inventory

Service map / dependency graph ✓ · SLO tracking + alerting ✓ · automatic anomaly detection + ML-driven RCA ✓
(LLM summary gated) · eBPF continuous profiling ✓ · logs ✓ · traces (eBPF + OTLP) ✓ · metrics (Prometheus) ✓ ·
**cost monitoring ✓** · **deployment tracking ✓** · predefined dashboards/inspections ✓.

## Pricing

- **Community:** free forever, self-hosted, Apache-2.0. eBPF metrics/traces/logs/profiling, service maps,
  SLO, alerts, cost monitoring, MCP (read-only + `resolve_alerts`).
- **Enterprise:** **$1 per monitored CPU core/month**. Adds AI RCA, agentic anomaly investigation, SSO,
  RBAC, capacity planning, 24×7 support.
- **Coroot Cloud:** managed path for AI RCA for Community (10 free investigations/mo).
- **Gated:** AI RCA, SSO, RBAC, EE MCP tools.

## Strengths and Gaps (vs the Parallax wedge)

**Strengths:**
- **Lowest adoption friction** in the field — eBPF agent → service map, zero app instrumentation.
- **Strongest MCP safety model** — per-user OAuth + RBAC projection, mostly read-only. Closest to Parallax's
  read-only/projection goal.
- Mature broad platform — metrics+logs+traces+profiling+cost+deploy tracking; fast cadence; credible founders.
- **Two-stage RCA** (deterministic ML before LLM) is defensible and accuracy-oriented.

**Gaps:**
1. **eBPF spans are partial** — protocol-level only, no app-level error chains / panics / stack traces.
2. **No Sentry migration path** — no DSN/envelope ingest, not an error-tracking tool.
3. **No portable, versioned, redacted evidence-bundle schema** — RCA persisted in-app, no exportable artifact.
4. **AI RCA not purely OSS/local** in Community — Enterprise or Coroot Cloud (external LLM, credit-metered).
5. **MCP not 100% read-only** — `resolve_alerts` mutates (Parallax's first surface is strictly read-only).
6. **No coding-agent/CLI action audit** and **no accepted/rejected/reverted fix-outcome loop**.
7. **Heavier local footprint** — 5-container stack (ClickHouse + Prometheus) vs a single Rust binary.

## Backend & Data Flow

See [backend-and-data-flow.md](backend-and-data-flow.md) for the side-by-side. Coroot summary — it is **not a
self-contained TSDB**, but an ingest+correlation layer in front of external engines:

- **Engine (per signal):** logs/traces/profiles → **ClickHouse** (`otel_logs`, `otel_traces`, `profiling_*`);
  metrics → **Prometheus** by default (or VictoriaMetrics/Thanos/Mimir with Remote Write Receiver). Config in
  **SQLite/Postgres**. **No broker** (agent-side WAL buffers). No object storage in core.
- **Flow:** agents push to central `coroot:8080`, never to the stores directly —
  `node-agent (eBPF): metrics ─Prom RemoteWrite─► Prometheus | logs/traces ─OTLP/HTTP─► coroot ─SQL─► ClickHouse | profiles ─HTTP─► ClickHouse`;
  `cluster-agent` adds DB metrics/schema; apps may also send OTLP straight to `coroot:8080`.
- **Write/read:** agents buffer in local WAL (survive Coroot outage); metrics→Prometheus head/WAL/blocks;
  logs/traces/profiles→ClickHouse MergeTree (~10× compression claim, TTL). Reads: metrics via PromQL (largely from
  local cache), logs/traces/profiles via ClickHouse SQL. Service map / RED metrics computed by Coroot, not a graph DB.
- **Throughput (vendor):** node-agent at 10k RPS ≈ 200m CPU (~20% of a core); sustained <0.3 cores; eBPF overhead
  ~+15% vs SDK ~+35–38%.
- **Designed for:** zero-instrumentation, low-overhead Kubernetes/Linux observability; ClickHouse gives cheap long
  retention. **Not for:** being its own scalable TSDB/appliance — you operate Prometheus/VM + ClickHouse yourself.

## Comparison: Coroot vs Parallax

| Dimension | Coroot | Parallax |
|-----------|--------|----------|
| **Product category** | eBPF observability + APM (service map, SLO, RCA) | Evidence context engine |
| **Capture model** | eBPF zero-instrumentation (+ OTLP) | OTLP + Sentry envelope, app-level error chains |
| **Primary language** | Go | Rust |
| **Ingest protocols** | OTLP/HTTP + Prometheus remote-write; no Sentry | Sentry envelope + OTLP |
| **Error semantics** | Protocol-level only, partial spans | App-level errors, panics, stack traces, fingerprints |
| **Telemetry store** | ClickHouse + Prometheus | GreptimeDB + Turso |
| **Local mode** | 5-container compose | Single binary |
| **Agent surface** | MCP, 18 tools, ~read-only (1 mutates), OAuth+RBAC | CLI-first + read-only MCP after gates |
| **Evidence bundles** | None (RCA on incident) | Versioned, redacted, portable |
| **Outcome tracking** | None | Fix-outcome loop |
| **AI gating** | Enterprise or Cloud-metered | Open core, local-first |
| **License** | Apache-2.0 core / commercial EE | Apache-2.0 |

## Threat Assessment for Parallax

**Moderate-to-high pressure on adoption friction and MCP safety; low on the specific wedge.**

Coroot is the strongest tool in the watchlist on **two** dimensions Parallax cares about: lowest-friction
local adoption (eBPF service map in minutes) and the **best MCP safety model** (per-user OAuth + RBAC,
mostly read-only). Those directly pressure Parallax's "easy local + safe agent surface" story.

It does not close the wedge: eBPF spans are partial and protocol-level (no app error chains), there is no
Sentry migration path, no portable redacted evidence bundle, AI RCA is Enterprise/Cloud-gated, the MCP has
a mutating tool, and there is no fix-outcome loop.

### Watch Triggers

Re-evaluate if Coroot adds: a **portable redacted evidence-bundle** artifact; **Sentry migration / envelope
ingest**; **app-level error semantics** (panics/stack traces/error chains); a **strictly read-only** agent
projection; or a **fix-outcome loop**. It currently has none.

## Summary Verdict

Coroot is the lowest-friction, safest-agent-surface open observability tool in the watchlist — eBPF
zero-instrumentation gives an instant service map, and its per-user OAuth + RBAC MCP is the closest thing to
Parallax's intended read-only projection. But it is an **infrastructure/APM observability tool**, not an
evidence context engine: its spans are deliberately partial and protocol-level, it has no app-level error
semantics, no Sentry path, no portable redacted bundle, and no outcome loop, and its AI RCA is gated behind
Enterprise/Cloud. The two things to borrow from Coroot are its **adoption-friction model** and its **MCP
RBAC safety posture**; the wedge it cannot reach is the Sentry-ingest → app-error semantics → versioned
redacted bundle → outcome-loop chain.
