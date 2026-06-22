# SigNoz Deep Research

Research date: 2026-06-22

This note gives SigNoz a Maple-style standalone deep-dive, consolidating what was
previously scattered across `competitor-watch.md`, `competitive-comparison-matrix.md`,
storage evaluation, and the validation ledgers. It is grounded in current primary
sources (signoz.io, docs, GitHub) checked June 2026; facts are flagged where a
source conflict was found and could not be fully reconciled.

## Sources

- [github.com/SigNoz/signoz](https://github.com/SigNoz/signoz) — README, LICENSE, releases, architecture
- [github.com/SigNoz/signoz/blob/main/LICENSE](https://github.com/SigNoz/signoz/blob/main/LICENSE) and [ee/LICENSE](https://github.com/SigNoz/signoz/blob/develop/ee/LICENSE) — license split
- [github.com/SigNoz/signoz/discussions/4231](https://github.com/SigNoz/signoz/discussions/4231) — maintainer license confirmation
- [signoz.io/docs/architecture](https://signoz.io/docs/architecture/), [install/docker](https://signoz.io/docs/install/docker/), [install/self-host](https://signoz.io/docs/install/self-host/)
- [signoz.io/blog/launching-signoz-single-binary](https://signoz.io/blog/launching-signoz-single-binary/), [oss-improvements (SQLite→Postgres)](https://signoz.io/blog/oss-improvements/)
- [signoz.io/docs/ingestion/self-hosted/overview](https://signoz.io/docs/ingestion/self-hosted/overview/), [userguide/exceptions](https://signoz.io/docs/userguide/exceptions/)
- [signoz.io/comparisons/sentry-alternatives](https://signoz.io/comparisons/sentry-alternatives/), [docs/querying/overview](https://signoz.io/docs/querying/overview/)
- [github.com/SigNoz/signoz-mcp-server](https://github.com/SigNoz/signoz-mcp-server) (+ LICENSE), [github.com/SigNoz/agent-skills](https://github.com/SigNoz/agent-skills)
- [signoz.io/docs/ai/signoz-mcp-server](https://signoz.io/docs/ai/signoz-mcp-server/), [ai/use-cases/postmortem-evidence-pack](https://signoz.io/docs/ai/use-cases/postmortem-evidence-pack/), [agent-native-observability](https://signoz.io/agent-native-observability/)
- [signoz.io/pricing](https://signoz.io/pricing/), [signoz.io/blog/signoz-funding](https://signoz.io/blog/signoz-funding/), [TechCrunch funding](https://techcrunch.com/2023/09/28/open-source-datadog-rival-signoz-lands-on-the-cloud-with-6-5m-investment/), [YC W21](https://www.ycombinator.com/companies/signoz)

## What SigNoz Is

SigNoz is an **open-source, OpenTelemetry-native full-stack observability platform** —
unified logs, traces, and metrics in one app, positioned as an open-source Datadog /
New Relic alternative. It also markets APM, distributed tracing, dashboards, alerting,
exception monitoring, and LLM observability. It competes for the same "open, self-hosted,
OTel-native observability backend" ground as Maple and OpenObserve.

- **License (resolved — corrects the common "AGPL" claim):**
  - **Core platform = MIT Expat** (everything outside the enterprise dirs), confirmed in the
    LICENSE file and by maintainer discussion. *Not* AGPL.
  - **`ee/` and `cmd/enterprise/` = proprietary SigNoz Enterprise License.**
  - **`signoz-mcp-server` = Apache-2.0** (separate repo, "Copyright 2025 SigNoz Authors"),
    free and self-hostable — do not conflate with the platform core.
- **Languages:** TypeScript ~53% (React frontend), Go ~37% (backend), Python ~5%.
- **Telemetry storage:** ClickHouse (+ ClickHouse Keeper).
- **Metadata store:** relational, **not** ClickHouse — originally SQLite, **PostgreSQL added 2025**
  (`SIGNOZ_SQLSTORE_PROVIDER`); current default compose ships a PostgreSQL "metastore".
- **Deploy:** Docker Compose/Standalone, Docker Swarm, Kubernetes (Helm), single-binary on VM.
- **Cloud vs self-hosted:** managed SigNoz Cloud + self-hosted Community + Enterprise tier.
- **Company:** SigNoz, **YC W21**; founders Pranay Prateek (CEO), Ankit Nayan (CTO); SF, founded 2021.
- **Funding:** ~$6.5M total ($5.4M announced 2023-09-28 led by SignalFire + ~$1.1M YC). SigNoz
  calls the $5.4M "Series A"; Crunchbase/PitchBook classify it as **seed** — flagged, unresolved.
- **Maturity:** ~27.4k stars (June 2026, aggregators lag); latest **v0.129.0, 2026-06-18**; still
  **pre-1.0**; very fast cadence (~6 minors in ~5 weeks).

## Architecture

### Components (post-consolidation, v0.76 / 2025-03-13)

`query-service` + `frontend` + `alertmanager` were merged into **one Go binary `signoz`**
(bundles React frontend, API/query server, OpAMP server, Ruler, Alertmanager). Still separate
processes/containers:

| Component | Role |
|-----------|------|
| `signoz` (Go binary) | UI + API/query + Ruler + Alertmanager + OpAMP |
| `signoz-otel-collector` | OTLP ingest + processing |
| ClickHouse (+ Keeper) | telemetry store (logs/traces/metrics) |
| PostgreSQL / SQLite | control-plane metadata (orgs, users, dashboards, configs) |
| `schema-migrator` | ClickHouse schema migrations |

The "single binary" consolidation is **only** the control-plane app. ClickHouse and the
collector remain separate. There is **no embedded-ClickHouse single-process local mode.**

### Data Flow

1. App emits OTLP (traces/logs/metrics) via standard OTel SDK/collector.
2. `signoz-otel-collector` receives OTLP (gRPC 4317 / HTTP 4318), processes, writes to ClickHouse.
3. `signoz` binary queries ClickHouse, serves dashboard, MCP, alerts; metadata in Postgres/SQLite.

### Key Technical Choices

| Layer | Choice | Notes |
|-------|--------|-------|
| Telemetry store | ClickHouse + Keeper | column-oriented, all three signals |
| Metadata store | SQLite → PostgreSQL | control-plane only; validates the telemetry/metadata split |
| Backend | Go | consolidated single control-plane binary |
| Frontend | React (TypeScript) | bundled into the Go binary |
| Ingest | signoz-otel-collector | standard OTel Collector distribution |
| Query | Query Builder v5 / ClickHouse SQL / PromQL | GUI primary; SQL all signals; PromQL metrics-only |

## OTLP / OTel Support

- **OTLP-native by design.** Ingests OTLP over **both gRPC (4317) and HTTP (4318)** for **all
  three signals**. Cloud consolidates OTLP onto port 443.
- **Non-OTLP (metrics only):** Prometheus receiver/scrape and Prometheus remote-write/read for
  migration. Logs and traces are effectively **OTLP-only**.
- **Error/exception model = OTel span-events** (`exception.type` / `exception.message` /
  `exception.stacktrace` from `recordException()`/auto-instrumentation). Dedicated **Exceptions
  tab** sortable by Last/First Seen, Count, type, app; drills to parent trace/span.
- **No Sentry-envelope / Sentry-SDK ingestion path.** You cannot point Sentry SDKs at SigNoz;
  migration requires re-instrumenting with OTel. No issue grouping/fingerprinting, no issue
  lifecycle states (resolved/regressed/ignored), no assignment — each exception is a **queryable
  span event**, not a managed work item. The bear-case "SigNoz adds Sentry ingest" trigger has
  **not** fired.
- **Query interfaces (three):** Query Builder v5 (GUI, all signals), raw ClickHouse SQL (all
  signals, mainly dashboard panels), PromQL (metrics-only).

## Local-Run Story

- **One-command install exists** (Foundry installer: `curl -fsSL https://signoz.io/foundry.sh | bash`
  → `foundryctl cast`), **but it spins up a ~5-container stack** (signoz, signoz-otel-collector,
  ClickHouse, ClickHouse Keeper, PostgreSQL).
- **No true single-binary / embedded-engine local mode.** ClickHouse and the collector are always
  separate processes/containers.
- **Footprint:** official minimum **≥4 GB RAM allocated to Docker**; third-party idle baselines
  ~1.5–2 GB.

This is the sharpest contrast with Maple Local (single Bun binary + chDB) and OpenObserve
(single Rust binary, disk-only default): SigNoz has no equivalent zero-Docker local wedge.

## AI / MCP Surface

- **Official `signoz-mcp-server`** (Apache-2.0). Latest **v0.5.1, 2026-06-17**; first announced
  platform changelog v0.121.1 (2026-05-01). A separate **unofficial** DrDroid server exists —
  not the same thing.
  - **Hosting: both** — Cloud-hosted (`https://mcp.<region>.signoz.cloud/mcp`) and self-hosted
    (binary / `go install` / Docker / source).
  - **Transport:** stdio (default) or HTTP (optional OAuth). No SSE.
  - **Clients documented:** Claude Code, Claude Desktop, Cursor, VS Code/Copilot; launch
    changelog also names Codex and Gemini. Windsurf not explicitly documented.
  - **Tools: ~38 total — NOT read-only.** ~25 read tools (`signoz_search_logs/traces`,
    `signoz_query_metrics`, `signoz_get_trace_details`, `signoz_execute_builder_query`,
    `signoz_list/get_alert*`, `signoz_list/get_dashboard`, `signoz_search_docs`, …) **plus 13
    write/destructive tools** — `signoz_create/update/delete_alert`, `..._dashboard`, `..._view`,
    `..._notification_channel`. An agent can mutate and delete alerts, dashboards, saved views,
    and channels.
- **Official `agent-skills` repo** — a Claude Code plugin marketplace ("signoz-skills"),
  CalVer-versioned in manifest, no git tags (latest commit 2026-06-17), **12 skills**. The
  investigation skill is **`signoz-investigating-alerts`**: read-only, three-tier alert RCA flow,
  mandated output sections, every claim must cite an MCP query result, plus eval cases. Companion
  `signoz-explaining-alerts` does static rule decode.
- **"Postmortem Evidence Pack"** — documented use case: an assistant compiles alert history, metric
  inflection points, representative logs, trace search results, and trace details into an incident
  timeline. **"Open investigation format"** appears on the agent-native-observability page.
- **CRITICAL GAP — no portable, versioned evidence-bundle schema.** The Postmortem Evidence Pack
  output is **ad-hoc prose markdown generated per-investigation by the LLM** — no JSON schema, no
  export file format, no version field, no provenance/redaction/raw-ref/query-manifest/missing-
  evidence/outcome semantics. SigNoz's "open/portable" layer is OTel (data) + MCP (protocol) +
  markdown skill files, **not a serializable, hand-off-able evidence artifact.**

## Pricing

- **Community (self-hosted): $0**, fully free, self-managed.
- **Teams Cloud: $49/mo base**, usage-based — **$0.30/GB logs & traces**, **$0.10 / million metric
  samples**; retention 15d–1yr (logs/traces), 1–13mo (metrics). Startup program → $19/mo.
- **Enterprise:** custom, ~$4,000/mo floor.
- **Feature gating:** SSO/SAML = add-on (Teams >$999/mo) or Enterprise; **RBAC + audit logs =
  Enterprise ("coming soon")**; BYOC/SLA/dedicated support = Enterprise.
- **AI/MCP gating:** the **MCP server is Apache-2.0, free, self-hostable** against any instance.
  The in-product "Noz" AI teammate and hosted MCP are listed as Teams-Cloud features; whether
  self-hosted Community users get Noz is **not explicitly stated** (uncertain).

## What SigNoz Does Well

1. **Mature, fast-moving, OTel-native, single unified backend.** Logs/traces/metrics in one app,
   validated telemetry-vs-relational-metadata split, very rapid release cadence.
2. **Real first-party MCP — both hosted and self-hosted** — with a broad (partly mutating) toolset,
   plus a Claude Code skills marketplace and a read-only alert-RCA skill with evals. This is one of
   the strongest current agent/MCP stories in the open self-hosted field.
3. **Permissive licensing on the parts that matter** — MIT-Expat core + Apache-2.0 MCP.
4. **Active postmortem/agent-native framing** — "Postmortem Evidence Pack", "open investigation
   format", on-call lifecycle automation. Strong directional pressure on the evidence narrative.
5. **Usage-based cloud pricing** with no per-seat fees and a large community.

## What SigNoz Does Not Do Well (vs the Parallax wedge)

1. **No portable, versioned evidence-bundle schema.** The evidence pack is throwaway LLM markdown,
   not a durable/shareable/validator-backed artifact. This is the biggest opening — it is precisely
   Parallax's A3 schema/corpus thesis, and SigNoz's "open investigation format" language is product
   copy, not a published spec.
2. **No Sentry-SDK / envelope ingestion and no error-issue lifecycle** (grouping, states,
   assignment). Clean lane for Sentry-migration + error-workflow.
3. **No outcome loop.** Investigation skills produce ranked probable causes but nothing closes the
   loop on resolution / acceptance / revert.
4. **No redaction / PII story surfaced** in any checked source — not a marketed concern.
5. **Heavy local footprint** — ~5-container, ≥4 GB-RAM ClickHouse stack; no single-binary/embedded
   local mode. A single-binary local-first tool (GreptimeDB + Turso) is a direct differentiator.
6. **MCP is partly write/destructive** (create/update/delete alerts, dashboards, views, channels) —
   not the read-only bounded evidence projection Parallax intends.
7. **Still pre-1.0**; RBAC/audit logs not yet GA.

## Comparison: SigNoz vs Parallax

| Dimension | SigNoz | Parallax |
|-----------|--------|----------|
| **Product category** | Full observability platform (logs/traces/metrics/APM/dashboards) | Evidence context engine (not a dashboard suite) |
| **Primary language** | Go (backend) + TypeScript (UI) | Rust (Tokio) |
| **Ingest protocols** | OTLP only (+ Prometheus for metrics) | Sentry envelope + OTLP |
| **Telemetry store** | ClickHouse (+ Keeper) | GreptimeDB behind storage adapter |
| **Metadata** | SQLite / PostgreSQL | Turso (libSQL) |
| **Error model** | OTel span-events, queryable; no issue lifecycle | Derived error events, fingerprint, evidence bundles |
| **Agent surface** | MCP (~38 tools, read + write/destructive) | CLI-first, HTTP underneath, MCP (read-only, after safety gates) |
| **Evidence bundles** | None — ad-hoc LLM markdown "evidence pack" | Bounded, redacted, citable, portable, versioned bundles |
| **Outcome tracking** | None | Fix-outcome loop (accepted/rejected/reverted) |
| **Redaction** | None surfaced | Structured redaction before agent access (A6 gate) |
| **Local mode** | ~5-container Docker stack, ≥4 GB RAM | Single binary (GreptimeDB + Turso) |
| **License** | MIT Expat core / proprietary `ee/` / Apache-2.0 MCP | Apache-2.0 |
| **Maturity** | v0.129.0, pre-1.0, ~27.4k stars, YC + funded | Research/early implementation |

## Threat Assessment for Parallax

**Moderate-to-high pressure on the agent-native narrative; low pressure on the specific wedge.**

SigNoz is the strongest open self-hosted competitor on **agent/MCP maturity** — it ships both
hosted and self-hosted MCP, a skills marketplace, a read-only alert-RCA skill with evals, and
explicit "Postmortem Evidence Pack" / "open investigation format" framing. That directly pressures
any "agent-native observability" positioning.

It does **not** close the Parallax wedge, because the checked sources still show:

1. **No portable evidence-bundle schema** — the evidence pack is generated prose, not a versioned
   artifact with provenance/redaction/raw-ref/query-manifest/missing-evidence/outcome semantics.
2. **No Sentry ingestion** — OTel-native only; migration requires re-instrumentation.
3. **No outcome loop** — investigations end at ranked causes.
4. **No redaction layer.**
5. **No single-binary local wedge** — heavy ClickHouse stack.

### Watch Triggers

Re-evaluate if SigNoz:

- Publishes a **versioned, portable investigation/evidence schema** (with provenance, redaction,
  raw-refs, query manifest, missing-evidence flags, or outcome rows) — would pressure A3.
- Adds **Sentry envelope ingestion** or an error-issue lifecycle — would close the migration lane.
- Adds **fix-outcome tracking** — would close the core-thesis differentiator.
- Ships a **single-binary / embedded-engine local mode** — would close the local-first wedge.
- Adds a **redaction/PII safety layer** on the MCP surface.

## Summary Verdict

SigNoz is the most agent-mature open self-hosted observability platform in the watchlist: real
MCP (hosted + self-hosted), a skills marketplace, evals, and active postmortem/evidence framing,
on a permissive MIT-Expat core. It validates the market for agent-native, OTel-native, self-hosted
observability.

But it is a **full observability platform**, not an evidence context engine. It serves agents
through live MCP queries and generated markdown, not through bounded, redacted, versioned, portable
evidence bundles with an outcome loop — and it has no Sentry-migration path and no single-binary
local wedge. The "open investigation format" language is the closest competitive pressure on
Parallax's A3 thesis, but as of June 2026 it remains product copy over live queries, not a published
schema. The unoccupied ground — Sentry-ingest → OTLP correlation → versioned redacted bundles →
outcome tracking, delivered from a single Rust binary — is still open.
