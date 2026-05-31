# Sentry-Compatible Open-Source Tools

Research date: 2026-05-31

Tools that ingest Sentry envelopes, speak the Sentry event protocol, or serve as drop-in Sentry replacements. Excludes already-known tools: Sentry, GlitchTip, Bugsink, Rustrak, GoSnag, Urgentry.

## Summary Table

| Tool | Language | License | Sentry Protocol | Stars | MCP | OTLP |
|------|----------|---------|-----------------|-------|-----|------|
| **Temps** | Rust + TS | MIT/Apache-2.0 | Yes (envelope) | 459 | Yes | No |
| **ErrorPush** | Python | MIT | No (Rollbar only) | 390 | No | No |
| **Errsole** | JavaScript | MIT | No | 658 | No | No |
| **Sentry Relay** (official) | Rust | FSL | Yes (native) | 379 | No | Partial (relay-otel) |
| **Proof** | Go | Unlicensed | Yes (v4 + v7) | 8 | No | No |
| **Errex** | Rust + Svelte | AGPL-3.0 | Yes (envelope) | 4 | Stub | No |
| **Sentro** | TypeScript | MIT | No (own SDK) | 1 | No | Yes |
| **Kestrel** | Go + TS | MIT | Explicitly no | 0 | Yes | No |
| **Airbag** | Go | MIT | No (own API) | 0 | No | No |
| **edde746/bugs** | Rust + SolidJS | GPL-3.0 | Yes (envelope) | 0 | No | No |

---

## Detailed Findings

### 1. Temps

- **GitHub**: <https://github.com/gotempsh/temps>
- **URL**: <https://temps.sh>
- **What it does**: Self-hosted PaaS (like Vercel/Railway) that bundles deployment, analytics, session replay, and Sentry-compatible error tracking into a single Rust binary. Error tracking is one feature among many, not the sole focus.
- **Language/stack**: Rust (Axum, Sea-ORM, Pingora proxy) + TypeScript (React 19 frontend). PostgreSQL + TimescaleDB.
- **License**: Dual MIT / Apache-2.0
- **Sentry protocol**: Yes. `@temps-sdk/node-sdk` provides `ErrorTracking.init({ dsn })` that is "Sentry-compatible, drop-in replacement." Accepts Sentry-style DSNs.
- **Sentry SDK features supported**: Error capture (`captureException`), DSN-based routing. Not clear how deep the envelope/event protocol support goes — likely covers basic error events.
- **MCP/AI agent features**: Yes. Ships `@temps-sdk/mcp` for AI-agent-driven infrastructure management including deployments, monitoring, and error tracking.
- **OTLP support**: No (not mentioned).
- **Stars/forks/activity**: 459 stars, 25 forks. Very active — 555 commits, latest release v0.0.8 (Mar 2026), frequent releases.
- **Notable limitations**: Error tracking is a secondary feature within a larger PaaS. You get the whole platform, not just error tracking. Still early (v0.0.x). Single-node only.

### 2. ErrorPush

- **GitHub**: <https://github.com/hauxir/errorpush>
- **What it does**: Minimalist error collection service compatible with Rollbar clients. Stores all errors in a single PostgreSQL table. Use Metabase or direct SQL to query.
- **Language/stack**: Python (Flask), PostgreSQL, Docker
- **License**: MIT
- **Sentry protocol**: **No.** Compatible with Rollbar API, not Sentry. Listed here because it appears in "sentry alternative" topic searches and some users swap between the two.
- **Sentry SDK features supported**: None (Rollbar protocol only).
- **MCP/AI agent features**: No.
- **OTLP support**: No.
- **Stars/forks/activity**: 390 stars, 10 forks. Low activity — 22 commits, last update Feb 2026. Mature but essentially unmaintained.
- **Notable limitations**: No dashboard (requires Metabase or raw SQL). Rollbar-only protocol. No grouping/fingerprinting. No alerts. Not Sentry-compatible at all — more of a "store errors in Postgres" tool.

### 3. Errsole

- **GitHub**: <https://github.com/errsole/errsole.js>
- **What it does**: Node.js logger with built-in web dashboard. Collects console logs automatically, provides advanced logging functions, stores in SQLite/MySQL/PostgreSQL/MongoDB. Includes log viewer with filtering, search, auth, and team management.
- **Language/stack**: JavaScript (Node.js), supports SQLite/MySQL/PostgreSQL/MongoDB
- **License**: MIT
- **Sentry protocol**: **No.** Own logging API. Not Sentry-protocol-compatible.
- **Sentry SDK features supported**: None.
- **MCP/AI agent features**: No.
- **OTLP support**: No.
- **Stars/forks/activity**: 658 stars, 58 forks. Active — 815 commits, 40 releases, latest v2.18.2 (Jul 2025).
- **Notable limitations**: Node.js only. Not a Sentry replacement — it's a logger with a viewer. No error grouping/fingerprinting. No distributed tracing.

### 4. Sentry Relay (official)

- **GitHub**: <https://github.com/getsentry/relay>
- **What it does**: Official Sentry event forwarding and ingestion proxy. Stands in front of Sentry to handle envelope parsing, normalization, filtering, rate-limiting, and PII scrubbing. Can forward to Sentry SaaS or self-hosted Sentry.
- **Language/stack**: Rust (6,260 commits, extensive workspace), Python bindings via C-ABI
- **License**: FSL (Functional Source License) — not open source in the traditional sense. You can use it, but network-service use of modified versions requires contributing back.
- **Sentry protocol**: Yes — this **is** the reference implementation. Handles envelope parsing, event normalization, and all Sentry protocol versions.
- **Sentry SDK features supported**: All — this is the official ingest pipeline. Envelope parsing, event normalization, filtering, rate-limiting, PII scrubbing, source maps, profiling, replays, OTLP ingestion (via relay-otel), and more.
- **MCP/AI agent features**: No.
- **OTLP support**: Partial. Has a `relay-otel` crate for OpenTelemetry ingestion, translating OTLP to Sentry events.
- **Stars/forks/activity**: 379 stars, 117 forks. Extremely active — 6,260 commits, 206 releases, latest v26.5.1 (May 2026). Production-grade.
- **Notable limitations**: FSL license (not OSI-approved open source). Requires a Sentry backend — Relay does not store or display events itself. Not a standalone Sentry replacement. Processing mode requires Kafka + Redis.

### 5. Proof

- **GitHub**: <https://github.com/scr34m/proof>
- **What it does**: Minimal Sentry alternative / drop-in replacement for local development. Single Go binary with no external dependencies. Supports Sentry protocol versions 4 (old) and 7 (latest). Includes macOS notification support.
- **Language/stack**: Go (single binary, HTML templates for UI)
- **License**: No license file specified (unlicensed)
- **Sentry protocol**: Yes. Supports Sentry protocol v4 and v7. Acts as a drop-in Sentry DSN endpoint.
- **Sentry SDK features supported**: Basic error event ingest. No envelopes, no transactions, no performance monitoring.
- **MCP/AI agent features**: No.
- **OTLP support**: No.
- **Stars/forks/activity**: 8 stars, 0 forks. Low activity — 40 commits, last update Feb 2026. Stable but minimal.
- **Notable limitations**: No license. Dev/local use only — not production-grade. No envelope support (only older protocol versions). No grouping beyond basic display. macOS-focused notifications.

### 6. Errex

- **GitHub**: <https://github.com/TheHoltz/errex>
- **What it does**: Lightweight self-hosted error tracker. Single 5 MB Rust binary with embedded SvelteKit dashboard. SQLite persistence. Sentry-SDK compatible ingest. Fingerprint-based grouping, live WebSocket updates, Slack/Discord/Teams webhooks, regression detection.
- **Language/stack**: Rust (Axum/Tokio) + SvelteKit dashboard embedded in binary. SQLite (WAL mode).
- **License**: AGPL-3.0
- **Sentry protocol**: Yes. Accepts Sentry envelopes at `/api/<project>/envelope/`. Handles gzip + plaintext. Parses, fingerprints, groups.
- **Sentry SDK features supported**: Envelope ingest, error events, DSN auth, per-project rate limits, resolve/mute/ignore/regression. Missing: source maps, multi-tenant orgs, transactions, performance.
- **MCP/AI agent features**: Stub (protocol surface wired, not fully implemented). Advertised as "MCP-ready for AI agents."
- **OTLP support**: No.
- **Stars/forks/activity**: 4 stars, 0 forks. Active development — 84 commits, last update May 2026. Alpha status.
- **Notable limitations**: Alpha quality. No source maps. No multi-tenant. MCP is a stub. Very early — 4 stars suggests limited real-world usage.

### 7. Sentro

- **GitHub**: <https://github.com/yzzztech/sentro>
- **What it does**: Open-source error tracking and observability platform designed for AI agents. Agent-first data model with run tracing, step replay, tool/LLM call monitoring, cost tracking. Has its own TS and Python SDKs.
- **Language/stack**: TypeScript (Next.js 15, Prisma, PostgreSQL, Tailwind, pg-boss)
- **License**: MIT
- **Sentry protocol**: **No.** Uses its own SDK and DSN format (`http://token@host/api/ingest/proj_1`). Not Sentry-protocol-compatible.
- **Sentry SDK features supported**: None — entirely separate protocol and SDK.
- **MCP/AI agent features**: No built-in MCP server, but designed for agent observability (run tracing, step replay, tool/LLM monitoring, cost tracking). Framework integrations for Claude Code, OpenClaw, LangChain, CrewAI, Vercel AI SDK.
- **OTLP support**: Yes. Accepts OTLP traces at `POST /api/v1/traces` with Bearer token. Maps OTLP spans to its data model.
- **Stars/forks/activity**: 1 star, 0 forks. Active development — 89 commits, last update May 2026. Early stage (v0.2.0).
- **Notable limitations**: Not Sentry-compatible — own SDK required. Very early (1 star). PostgreSQL required. No distributed tracing. No source maps yet.

### 8. Kestrel

- **GitHub**: <https://github.com/wearzdk/kestrel>
- **URL**: <https://kestrel.wearzdk.me>
- **What it does**: Error tracker built specifically for AI agents. Single Go binary, SQLite-only, MCP-native. MCP is the primary interface; web UI is secondary. Agent can list errors, get details, mark resolved with commit SHA.
- **Language/stack**: Go (pure Go SQLite via modernc.org/sqlite, no CGO) + TypeScript (React + Vite frontend). Python and JS SDKs.
- **License**: MIT
- **Sentry protocol**: **Explicitly no.** Non-goals list states: "no Sentry-protocol compatibility." Uses own ingest format and SDKs.
- **Sentry SDK features supported**: None.
- **MCP/AI agent features**: **Yes, this is the primary interface.** MCP server at `/mcp` (StreamableHTTP) and stdio. Tools: `list_errors`, `get_error`, `mark_resolved`, `mark_ignored`, `get_trend`. Designed for Claude Code, Cursor, etc.
- **OTLP support**: No.
- **Stars/forks/activity**: 0 stars, 0 forks. 3 commits. WIP label. Last update May 2026.
- **Notable limitations**: Extremely early (0 stars, 3 commits). Not Sentry-compatible. No distributed tracing, no session replay, no APM, no team RBAC, no alert channels. Scoped very tightly by design.

### 9. Airbag

- **GitHub**: <https://github.com/divinedev111/airbag>
- **What it does**: Self-hosted crash monitoring with structured JSON reports. Go + SQLite. Fingerprint-based deduplication. Go SDK included.
- **Language/stack**: Go, SQLite (WAL mode)
- **License**: MIT
- **Sentry protocol**: **No.** Custom JSON API at `POST /api/ingest/:project_id`. Not Sentry-protocol-compatible.
- **Sentry SDK features supported**: None.
- **MCP/AI agent features**: No.
- **OTLP support**: No.
- **Stars/forks/activity**: 0 stars, 0 forks. 8 commits. Last update Apr 2026.
- **Notable limitations**: Very early. Custom protocol only. No web dashboard (API only). No alerts. No source maps. Go SDK only.

### 10. edde746/bugs

- **GitHub**: <https://github.com/edde746/bugs>
- **What it does**: Lightweight self-hosted error tracking fully compatible with Sentry SDKs. Single Rust binary, SQLite. Source map support, release management, performance monitoring, alerts, user feedback, full-text search, environments, retention policies.
- **Language/stack**: Rust (Axum) + TypeScript (SolidJS frontend). SQLite.
- **License**: GPL-3.0
- **Sentry protocol**: Yes. Full Sentry SDK compatibility. Accepts standard Sentry DSNs.
- **Sentry SDK features supported**: Envelope ingest, error events, source maps, releases/deploys, performance monitoring, alerts, user feedback, full-text search, environments, retention policies.
- **MCP/AI agent features**: No.
- **OTLP support**: No.
- **Stars/forks/activity**: 0 stars, 1 fork. 72 commits. Last update May 2026. Active development.
- **Notable limitations**: GPL-3.0 license (copyleft). No multi-user/teams. No uptime monitoring. Very new (0 stars). But feature-rich relative to other lightweight options — notably has source maps, which Rustrak and Errex lack.

---

## Tools That Are Sentry-Compatible but Not Standalone Servers

### Sentry Relay (getsentry/relay)

See entry #4 above. This is the official Sentry proxy/ingest pipeline. Useful as a reference implementation of the Sentry envelope protocol. Could be used as a building block for a custom ingest system.

---

## Cross-Cutting Observations

### Sentry Protocol Support Spectrum

| Full envelope support | Partial (old protocol) | Own protocol only |
|---|---|---|
| edde746/bugs, Errex, Temps | Proof (v4+v7, no envelopes) | Airbag, Kestrel, Sentro, ErrorPush (Rollbar), Errsole |

### MCP/AI Agent Features

Only a handful of tools have MCP support:
- **Kestrel**: MCP is the *primary* interface (uniquely agent-first design)
- **Errex**: MCP stub (wired but not fully shipped)
- **Temps**: Full MCP server for platform management
- **Sentro**: Agent-observability focus but no MCP server

### OTLP Support

Only **Sentro** has OTLP ingestion. **Sentry Relay** has partial OTLP support (relay-otel crate). None of the lightweight alternatives support OTLP.

### Most Feature-Complete Lightweight Sentry Replacements

1. **edde746/bugs** — fullest feature set per resource (source maps, releases, perf, alerts, ~3 MB RAM, 40 MB image). GPL-3.0.
2. **Errex** — good envelope support, MCP-ready, ~7 MB RAM. Alpha, no source maps. AGPL-3.0.
3. **Temps** — Sentry-compatible error tracking as part of a full PaaS. MIT/Apache.

### Landscape Gaps

- No lightweight Sentry-compatible server supports OTLP natively.
- Source map support is rare in the lightweight space (only edde746/bugs has it).
- MCP integration is very early across the board.
- Most tools with <10 stars are proof-of-concept quality.
