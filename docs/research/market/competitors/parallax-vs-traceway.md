# Parallax vs Traceway

> An unbiased, one-to-one comparison. Research date: **2026-07-17** (**pass 50**
> first canonical deep-dive). Sources: primary
> [github.com/tracewayapp/traceway](https://github.com/tracewayapp/traceway)
> (README, `backend/go.mod`, `docs/pages/learn/mcp.mdx`, `cli/pkg/mcpserver/*`,
> releases), [tracewayapp.com](https://tracewayapp.com/), pass-49
> [wedge-closer recheck](../wedge-closer-lightweight-recheck-2026-07-17.md).
>
> **Bottom line up front:** Traceway is a **MIT, OTel-native, self-hosted
> full-stack observability product with a first-class agent surface** (CLI +
> skills + **local and remote MCP**). On **shipped multi-signal OTLP platform +
> agent investigation UX maturity, Traceway is ahead of pre-release Parallax.**
> Parallax's remaining edges vs Traceway are **Apache vs MIT is not the story**
> (both open) — real edges are **Sentry-envelope migration**, **Rust-first /
> GreptimeDB mandatory stack**, **portable versioned redacted evidence bundle
> contract**, and **fix-outcome loop** (code/offline residual; A1 **value
> unproven**). Do **not** claim "agent-first CLI/MCP" or "OTel-native self-host"
> as Parallax-unique against Traceway.

## What each product is

- **Traceway** (`tracewayapp/traceway`) — **OpenTelemetry-native observability
  platform**: logs, traces, metrics, exceptions (fingerprinted issues), session
  replay/RUM, AI tracing, experimental profiling. **MIT.** Primary language
  **Go** (~2.5M LoC weight) + Svelte UI + TS. Ingest: **OTLP/HTTP** (protobuf +
  JSON). Deploy: Docker (ClickHouse+Postgres standalone **or** SQLite/DuckDB
  embedded) **or** embeddable Go library (SQLite). **Agent surface (shipped):**
  agent-first `traceway` CLI, Claude/Cursor/Codex **skills**, and **MCP** —
  local stdio (`traceway mcp`) **and** remote HTTP `/mcp` with OAuth + PAT.
  **1,024★, 39 forks, created 2025-12-18, last push 2026-07-17; backend/cli
  v1.9.1 (2026-07-15).** Tagline: "the only tool you need to know what is
  happening and how to fix it."
- **Parallax** — Apache-2.0, Rust-first **execution-context engine**: OTLP
  gRPC/HTTP + Sentry envelope ingest, derived errors, evidence graph, bounded
  redacted **evidence bundles**, CLI/GraphQL/UI, local-stdio MCP (plan 112 DONE;
  remote deferred). GreptimeDB + Turso. **Pre-release.**

**Layer honesty:** both are self-hosted OTel backends with agent access.
Traceway optimizes for **human + agent product surface over a full obs stack**.
Parallax optimizes for **portable evidence contracts + outcome substrate for
coding-agent fix loops**. Overlap is large on the "agent queries production
telemetry" job; divergence is the **artifact/outcome layer**.

## Signal coverage

| Signal | Traceway (shipped) | Parallax (✅🧪 = code-shipped pre-release) |
| --- | --- | --- |
| Traces (OTLP) | ✅ OTLP/HTTP native | ✅🧪 gRPC + HTTP |
| Logs (OTLP) | ✅ | ✅🧪 |
| Metrics (OTLP) | ✅ | ✅🧪 |
| Exceptions / issues | ✅ SHA-256 fingerprint groups + ranking | ✅🧪 derived `error_event` + fingerprint |
| Sentry envelope / DSN | ❌ (not in README/docs; no hits) | ✅ plan 118 DONE |
| Session replay / RUM | ✅ | ❌ |
| AI / LLM traces | ✅ `get_ai_trace` | 🟡🧪 partial modules |
| Profiling | 🟡 experimental (pprof + OTLP profiles) | ❌ |
| CLI / agent invocation audit | ❌ product focus = app telemetry | 🟡🧪 adapter program |
| Evidence bundle (portable, redacted, versioned) | ❌ live query tools, not bundle artifact | 🟡🧪 code (A1 unproven) |
| Fix-outcome / recurrence | ❌ | 🟡 offline residual plan 123 DONE; live unproven |

**Verdict:** **Traceway wins coverage breadth today** (esp. replay + AI traces +
polished exception UX). Parallax wins **Sentry envelope** and the *designed*
bundle/outcome cells (implementation present; value unproven).

## Ingestion & transport

- **Traceway:** OTLP/HTTP only as the public native path (`/api/otel/v1/{traces,metrics,logs}`);
  "no Collector required" marketing. Integrations emit OTLP. **No Sentry path**
  found in primary materials this pass.
- **Parallax:** OTLP gRPC + HTTP (all three signals) + **Sentry envelope HTTP**.

**Verdict:** OTLP-native **both ship**. On Sentry migration depth, **Parallax**.
On "point any OTel SDK and go," **Traceway is polished product**.

## Storage architecture

| Mode | Traceway | Parallax |
| --- | --- | --- |
| Tiny / embedded | SQLite (default) or DuckDB telemetry (`telemetry_duckdb`, CGO) | Managed GreptimeDB standalone + Turso (no product SQLite telemetry) |
| Standalone | ClickHouse telemetry + PostgreSQL config (`transactional_pg telemetry_ch`) | GreptimeDB + Turso (mandatory) |
| Object / S3 | AWS + GCS deps in `go.mod` | GreptimeDB object-store path (design + engine) |

**Verdict:** Traceway is more **deployment-flexible** (SQLite→DuckDB→CH).
Parallax is **opinionated single telemetry engine** (GreptimeDB) for evidence
workloads. Neither choice is "proven better" without Parallax-shaped
benchmarks (unmeasured here). ClickHouse path means Traceway shares the
SigNoz/HyperDX/Maple storage family, not GreptimeDB/TMA1.

## Agent surface (the crux — no-bias)

### Traceway (shipped, mature for the category)

1. **CLI** — agent-first: JSON when piped, stable error codes, `--fields`, mostly
   read-only (archive needs `--yes`).
2. **Skills** — `/traceway-setup`, `/traceway` as `SKILL.md` for Claude Code /
   Cursor / Codex (`npx skills add tracewayapp/traceway`).
3. **MCP (local + remote)** — documented in-repo:
   - Local: `traceway mcp` (stdio; reuses CLI session).
   - Remote: `https://<instance>/mcp` with **OAuth** (dynamic client registration)
     or PAT `twp_...`.
   - Tools (~16): `list_projects`, `list_exceptions`, `get_exception`,
     `get_exception_occurrence`, `query_logs`, `list_endpoints`,
     `get_endpoint_request`, `endpoints_chart`, `get_slow_endpoint_config`,
     `query_metrics`, `get_task`, `get_ai_trace`, `get_session`, `get_trace`,
     `archive_exceptions`, `unarchive_exceptions`.
   - **Read-only annotations** on query tools; **only archive/unarchive mutate**
     (descriptions require explicit user request) — safety model is closer to
     Parallax's read-only posture than Rustrak's "full control" MCP.
   - **Output schemas / `structuredContent`**, investigation **prompts**
     (`debug_issue`, `investigate_performance`, …), and **knowledge resources**
     (`traceway://knowledge/*`) baked into the server.

### Parallax (shipped local; remote deferred)

- Local-stdio **read-only** MCP (`parallax-mcp`, plan 112 DONE).
- CLI + GraphQL as primary surfaces.
- Remote MCP waits on auth/transport (Plan 109).
- Bundle projection is the differentiator *when A1 holds*.

**Honest verdict:** On **agent access maturity**, **Traceway is ahead** —
especially **remote MCP + OAuth + investigation playbooks as prompts/resources**.
Parallax cannot claim "we invented agent-safe production telemetry access."
Parallax can only claim a **different contract** (portable redacted bundle +
outcome), which is **unproven as better for fix quality (A1)**.

**Complementary reading (allowed):** Traceway MCP/CLI could *consume* a Parallax
bundle later, or a team could run both — but that is not current product.

## Pricing / openness

| Axis | Traceway | Parallax |
| --- | --- | --- |
| License | **MIT** | **Apache-2.0** |
| Self-host | ✅ primary path | ✅ primary path |
| Cloud pricing | Public product site; no stable public rate card extracted this pass (homepage fetch thin) — treat cloud economics as **unverified** | No paid cloud yet |
| Contribute | MIT + public monorepo | Apache-2.0 |

**Verdict:** Both open self-host. MIT is slightly more permissive than Apache
for some downstream uses; neither is AGPL. **No Parallax license win vs Traceway.**

## Where each wins (scoped — never unscoped)

| Axis | Winner | Why |
| --- | --- | --- |
| Shipped full-stack OTel UI + multi-signal | **Traceway** | Productized exceptions, endpoints, AI traces, replay |
| Agent investigation surface (CLI/skills/MCP remote) | **Traceway** | Remote OAuth MCP + prompts/resources |
| Read-mostly agent safety model | **Tie-ish** | Both annotate reads; Traceway has 2 mutate tools; Parallax local MCP read-only |
| Sentry envelope migration | **Parallax** | Shipped adapter |
| Portable redacted versioned evidence bundle | **Parallax design** | Code-shipped; A1 unproven; Traceway has none |
| Fix-outcome / earned autonomy substrate | **Parallax design** | Offline residual; live unproven |
| Rust-first / GreptimeDB engine bet | **Parallax** | Different stack; unproven vs ClickHouse/DuckDB |
| Tiny embeddable Go-in-process mode | **Traceway** | Library embed + SQLite |
| OSS license openness | **Slight Traceway** (MIT) or **tie** for self-host practice | |

## Watch triggers (Traceway → collision)

| Trigger | Status 2026-07-17 |
| --- | --- |
| Sentry envelope / DSN ingest | **UNFIRED** |
| Portable versioned redacted investigation **export** (JSON Schema + fixtures) | **UNFIRED** |
| Fix-outcome / recurrence / autonomy budget records | **UNFIRED** |
| MCP gains broad write/automation surface | **UNFIRED** (only archive) |
| Cloud pricing / SaaS lock-in pattern | **Open** (needs next pricing-page pass) |

If Sentry + portable export + outcomes fire, Traceway becomes a **primary
wedge-closer** (not just coverage pressure).

## Falsification

- Claim "Traceway lacks MCP" → **false** as of v1.9.x (this pass).
- Claim "Parallax unique on agent-first self-host OTel" → **false** vs Traceway.
- Claim "Traceway closes the Parallax combination" → **false** until portable
  redacted bundle + outcome (and ideally Sentry) exist.

## Open items (next recheck)

1. Live cloud pricing / hosted tier.
2. Exact exception grouping algorithm vs Sentry fingerprints (desk vs code).
3. Whether DuckDB/SQLite modes support multi-day production retention economics
   (measurement, not desk).
4. A1: does a Parallax bundle beat Traceway MCP tools-over-raw for coding-agent
   **fix** quality on R1–R3 bugs? **Unproven.**

## Related

- [wedge-closer-lightweight-recheck-2026-07-17.md](../wedge-closer-lightweight-recheck-2026-07-17.md)
- [parallax-vs-maple.md](parallax-vs-maple.md) (local OTLP platform peer)
- [parallax-vs-tma1.md](parallax-vs-tma1.md) (agent context / GreptimeDB peer)
- [parallax-vs-signoz.md](parallax-vs-signoz.md) (mature OTLP+MCP OSS peer)
