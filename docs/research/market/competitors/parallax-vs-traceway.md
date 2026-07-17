# Parallax vs Traceway

> An unbiased, one-to-one comparison. Research date: **2026-07-17** (**pass 50**
> first deep-dive). Sources: [github.com/tracewayapp/traceway](https://github.com/tracewayapp/traceway)
> (README + releases API), [tracewayapp.com](https://tracewayapp.com),
> [docs.tracewayapp.com](https://docs.tracewayapp.com), concurrent
> [wedge-closer recheck](../wedge-closer-lightweight-recheck-2026-07-17.md).
>
> **Bottom line up front:** Traceway is a **MIT, OTel-native, multi-signal
> self-host APM** (Go + ClickHouse/SQLite/DuckDB) that ships **agent-first CLI +
> SKILL.md skills + MCP** for coding agents to set up and query production
> telemetry. On **shipped OTLP full-signal breadth + agent-native investigation
> UX + session replay + pure-MIT packaging, Traceway is ahead of pre-release
> Parallax.** It is the **cohort escalator** among lightweight OSS (wedge-closer
> pass): "open self-host OTel + agent-native debug" is **no longer scarce**.
> Parallax edges narrow to **Sentry-envelope ingest (shipped)**, **portable
> versioned redacted evidence bundle** (code-shipped; **A1 unproven**),
> **fix-outcome loop** (offline residual plan 123 DONE; live value unproven),
> GreptimeDB+Turso choice (unproven vs ClickHouse/DuckDB), and Rust substrate.
> **Do not claim agent-first self-host OTel as Parallax-unique — Traceway ships it.**

## What each product is

- **Traceway** (`tracewayapp/traceway`) — **OpenTelemetry-native observability
  platform**: logs, traces, metrics, exceptions (SHA-256 fingerprint grouping +
  source maps), session replay/RUM (web + Flutter), AI/LLM tracing, experimental
  profiling, alerts, multi-tenant orgs + RBAC. **MIT, no BSL, no open-core
  asterisk** (README claim; GitHub `MIT`). **Go 1.25** backend + **SvelteKit**
  frontend. Storage: **ClickHouse+Postgres** (standalone docker compose) or
  **SQLite / DuckDB** embedded modes. Ingest: **OTLP/HTTP** (Protobuf + JSON)
  for traces/metrics/logs — README claims no Collector required. **Agent surface:**
  `npx skills add tracewayapp/traceway` installs `/traceway-setup` + `/traceway`
  skills; **agent-first CLI** (JSON when piped, stable exit codes, mostly
  read-only; archive exceptions needs `--yes`); site also markets **MCP Server**.
  **1,024★**, latest **backend/v1.9.1 + cli/v1.9.1** (2026-07-15); last push
  2026-07-17. Cloud: live [tracewayapp.com/cloud](https://tracewayapp.com/cloud)
  pricing (**pass 54 RESOLVED** — was wrongly “no public $/unit”).
- **Parallax** — open-source (Apache-2.0), Rust-first, self-hostable
  **execution-context engine**: OTLP-native ingest (gRPC+HTTP) of
  traces/logs/metrics, Sentry-envelope ingest, derives owned `error_event`s +
  fingerprints, serves bounded/redacted evidence bundles (**code-shipped**; A1
  value unproven) + local-stdio MCP (plan 112 DONE). GreptimeDB + Turso.
  **Pre-release.**

**Overlap:** both aim at self-host OTel full-signal + agent-readable production
context. Traceway is a **broader productized APM** (replay, endpoints, skills
marketplace-style install). Parallax is a **narrower evidence/bundle engine**.

## Current pin (2026-07-17)

| Field | Value | Source |
| --- | --- | --- |
| Stars | **1,024** | GitHub API |
| Backend / CLI | **v1.9.1** | releases 2026-07-15 |
| License | **MIT** | GitHub SPDX + README |
| Language | Go (+ SvelteKit UI) | GitHub |
| Telemetry store | ClickHouse (standalone); SQLite or DuckDB (embedded) | README tech stack |
| Relational | PostgreSQL (standalone); SQLite (embedded) | README |
| Homepage | [tracewayapp.com](https://tracewayapp.com) | repo |

## Signal coverage

| Signal | Traceway (shipped) | Parallax (pre-release; ✅🧪=code-shipped) |
| --- | --- | --- |
| Traces | ✅ OTLP/HTTP native | ✅🧪 OTLP gRPC+HTTP |
| Logs | ✅ OTLP/HTTP, trace-linked | ✅🧪 OTLP gRPC+HTTP |
| Metrics | ✅ OTLP/HTTP + endpoints/Apdex | ✅🧪 OTLP gRPC+HTTP |
| Exceptions / issues | ✅ fingerprint + ranked issues + symbolication | ✅🧪 derived `error_event` + fingerprint |
| Sentry envelope / DSN | ❌ (no Sentry path in README/docs crawl) | ✅🧪 plan 118 DONE |
| Session replay / RUM | ✅ web + Flutter | ❌ |
| Profiling | 🟡 experimental (pprof + OTLP profiles) | ❌ |
| LLM / AI tracing | ✅ (OpenRouter + OTel AI gateway) | 🟡🧪 modules; incomplete |
| CI / test results | ❌ | ✅🧪 span-derived tests |

**Verdict:** on **breadth of shipped signals (esp. replay + symbolication + AI
tracing productization), Traceway wins.** On **Sentry-envelope migration path,
Parallax ships; Traceway does not.** OTLP multi-signal: both ship (Traceway
HTTP-only per README; Parallax gRPC+HTTP).

## Ingestion & transport

- **Traceway:** native **OTLP/HTTP** endpoints for traces/metrics/logs; marketing
  "no Collector, no glue." Framework middlewares + OTel SDKs. Symbolicator also
  ships as standalone OTel Collector processor (Honeycomb source-map contract
  compatible).
- **Parallax:** OTLP gRPC+HTTP all three signals + **Sentry envelope HTTP**.

**Verdict:** OTLP-native **roughly tied** (protocol form differs). Sentry path =
**Parallax only**. Collector optional for both design stories.

## Storage architecture

- **Traceway:** **ClickHouse** (standalone docker compose with Postgres) or
  **embedded SQLite / DuckDB** (zero/low-deps modes). Not GreptimeDB.
- **Parallax:** **GreptimeDB** native OTLP tables + **Turso** metadata.

**Verdict:** Traceway offers **more storage deployment shapes today** (embedded
SQLite/DuckDB vs full CH+PG). GreptimeDB-vs-ClickHouse/DuckDB performance/cost
is **benchmark-dependent / unproven**. ClickHouse choice aligns Traceway with
SigNoz/HyperDX/Uptrace family.

## Query, dashboards, DX

- **Traceway:** full human dashboard (logs/traces/metrics/issues/endpoints) +
  **agent CLI** designed for pipes/JSON + **SKILL.md** skills for Claude
  Code/Cursor/Codex. Sub-second log search claim (vendor).
- **Parallax:** minimal V1 UI/CLI; evidence-graph + bundle path; local-stdio MCP.

**Verdict:** on **shipped human APM UX + agent-installable skills/CLI, Traceway
wins decisively.** Parallax is pre-release and thinner on productized UX.

## AI-native / agent-context story

- **Traceway (pass 50):** explicitly **AI-First** product positioning.
  - Skills: `/traceway-setup` (wire OTel + verify data), `/traceway` (query
    exceptions/logs/endpoints/metrics for RCA).
  - CLI: agent-first (JSON, stable exit codes); **mostly read-only** (archive
    exceptions requires explicit `--yes`).
  - Site markets **MCP Server** (`/product/mcp`) + **AI Tracing**.
  - **No** portable versioned redacted multi-signal **evidence-bundle schema**
    found (agent query tools ≠ Parallax bundle contract). **No** fix-outcome /
    recurrence loop.
- **Parallax:** bounded/redacted/versioned bundle (**code-shipped**, A1 unproven)
  + local-stdio read-only MCP; AI RCA 🏗; fix-outcome offline residual plan 123
  DONE (live value unproven).

**Honest verdict:** Traceway is **ahead on shipped agent-native production
debug UX** (skills + CLI + MCP marketing). Parallax's only differentiated agent
claim remains the **portable redacted versioned bundle + outcome loop** — value
**unproven (A1)**. Recorded plainly: Traceway pressures the "agent context engine"
framing from the **productized APM + skills** angle the same way TMA1 pressures
from the **embedded GreptimeDB MCP** angle.

## Architecture & deployment

- **Traceway:** docker compose (CH+PG) or single-container SQLite/DuckDB; optional
  **embedded Go library** inside an app process. Multi-tenant orgs + RBAC shipped
  (README). Cosign-signed images.
- **Parallax:** single-binary target supervising GreptimeDB + embeds Turso;
  multi-tenant/SSO 🏗.

**Verdict:** Traceway **ships more deployment modes + tenancy today.** Parallax's
single-binary GreptimeDB story is real in code but pre-release; Traceway's
embedded SQLite/DuckDB is a strong local-dev competitor to Maple/TMA1-class UX.

## Pricing & economics (**pass 54** — live [tracewayapp.com/cloud](https://tracewayapp.com/cloud))

| Mode | Traceway | Parallax |
| --- | --- | --- |
| Self-host | **Free** (MIT, full feature claim — no open-core gate) | Free (Apache-2.0, pre-release) |
| Cloud Starter | **Free**: 10k exceptions + **1 GB** ingest/mo; 3 projects/3 members; 7-day retention | n/a |
| Cloud Pro | **$12.99/mo**: 100k exceptions + **50 GB**; overage **$0.25/GB**; 30-day retention | n/a |
| Cloud Premium | **$24.99/mo**: 1M exceptions + **150 GB**; $0.25/GB beyond; 90-day retention | n/a |
| Cloud Enterprise | **$499.99/mo**: 200M exceptions + **2 TB**; overage **$0.20/GB**; custom retention | n/a |
| Enterprise+ | Custom managed self-host (data stays in your cloud) | n/a |

**No per-host / per-seat** fees (vendor FAQ). HTTP requests and background task
runs **not** metered. Metrics retention: 1-min for 30d, 1-hour rollups 1yr;
profiling 30d (FAQ). **No-bias:** Cloud is **public and cheap** at hobby/small-
team scale vs many incumbents — favors Traceway transparency (prior “no public
$/unit” was a research miss, not a competitor gap).

**Hidden cost:** Traceway self-host TCO = ops for ClickHouse+Postgres (or accept
embedded store limits). Parallax = GreptimeDB+Turso ops (unproven). License:
Traceway **MIT** is a **real marketing edge** vs AGPL peers (OpenObserve/Uptrace).

## Openness & lock-in

- **Traceway:** MIT monorepo; OTel-native formats; skills are plain Markdown in
  repo. Low format lock-in. Cloud optional.
- **Parallax:** Apache-2.0; OTel-native GreptimeDB tables + Turso; bundle schema
  is Parallax-specific (portable by design if A1 holds).

**Verdict:** both open. Traceway's **MIT + no open-core** claim is stronger
marketing than many OSS obs peers; record as competitor strength.

## Where Traceway wins (scoped)

- Shipped multi-signal OTel APM (logs/traces/metrics/exceptions/replay/AI tracing).
- Agent skills + agent-first CLI + MCP productization.
- Session replay / RUM (Parallax has none).
- Symbolication engine (source maps, dSYM, R8, Flutter).
- MIT full-box packaging; multiple storage deployment shapes.
- Product maturity (v1.9.x, 1k★, active) vs Parallax pre-release.

## Where Parallax honestly edges Traceway

- **Sentry-envelope ingest** (shipped) — Traceway has no Sentry migration path
  found. *(Real for teams on Sentry SDKs.)*
- **Portable versioned redacted evidence bundle** — Traceway has live agent
  query, not this artifact. *(Code-shipped; **A1 unproven**.)*
- **Fix-outcome / recurrence loop** — offline residual plan 123 DONE; live
  value unproven. Traceway: ❌.
- **GreptimeDB native OTLP + Turso split** — design choice; scale/perf unproven
  vs Traceway ClickHouse/DuckDB.
- **Rust vs Go** — minor substrate difference.

## Watch triggers

1. Traceway ships **Sentry envelope/DSN** compatibility → direct collision on
   Parallax interop wedge.
2. Traceway publishes a **versioned redacted multi-signal investigation export
   schema** (bundle-like) → A1 must face Traceway head-on.
3. Traceway ships **outcome/recurrence** product → closes more of combination.
4. License or open-core regression (MIT → BSL/EE split).
5. Cloud public rate card appears → re-pin pricing cell.

**Pass 50:** triggers 1–4 **UNFIRED** (README/docs crawl).

## Open questions / measurement

- **A1 vs Traceway:** does a Parallax bounded redacted bundle beat Traceway
  skills/CLI/MCP over live telemetry for coding-agent fix quality? **Unproven**
  (eval program).
- **Cloud pricing:** extract fixed-tier numbers from live Cloud signup or
  published calculator when available.
- **OTLP gRPC:** Traceway README emphasizes HTTP; confirm gRPC absence.
- **MCP tool list / mutability:** re-verify against `/product/mcp` + source when
  expanding agent-safety comparison (Coroot-class safety model?).

## Sources (accessed 2026-07-17; pass 50)

- [github.com/tracewayapp/traceway](https://github.com/tracewayapp/traceway) README + releases (`backend/v1.9.1`, `cli/v1.9.1`).
- [tracewayapp.com](https://tracewayapp.com) (product nav: Agent Skills, MCP Server, AI Tracing, Session Replay).
- [docs.tracewayapp.com/server/docker-compose](https://docs.tracewayapp.com/server/docker-compose) (ClickHouse + PostgreSQL).
- [wedge-closer-lightweight-recheck-2026-07-17.md](../wedge-closer-lightweight-recheck-2026-07-17.md).
- Parallax: [code-reality-ledger.md](../../code-reality-ledger.md), plan 112/118/123.
