# Code-reality ledger — research claims vs shipped source

**Date:** 2026-07-17  
**Purpose:** One inventory that maps major research product claims to **what
exists in `main` source today**. Use this before asserting capability in
vision, architecture, market, or agenda pages.

**Status vocabulary**

| Status | Meaning |
| --- | --- |
| **shipped** | Implemented in product crates / UI on `main`; may still be pre-release quality |
| **partial** | Core path exists; residual hardening, coverage, or product polish open |
| **PoC-only** | Mechanism proven under `poc/`; not product authority |
| **planned** | Active ownership only in `plans/` (or closed plan + residual unproven claim) |
| **unproven gate** | Design or code may exist; empirical product/market proof still open (A1–A7, etc.) |

**Discipline:** code existence ≠ scale proof. "Unique" only when competitors
truly lack the combination **and** product value remains marked unproven where
gates say so. Correction welcome: open a PR with primary-source evidence.

**Authority order:** this ledger + `crates/` / `ui/` / `schema/` / active
`plans/` > research prose. Historical research keeps dated banners; it does
not override code.

---

## 1. Ingest

| Claim | Status | In-repo pointer | Notes |
| --- | --- | --- | --- |
| OTLP traces/logs/metrics (gRPC + HTTP) | **shipped** | `crates/parallax-server/src/otlp_grpc.rs`, `otlp_http.rs`; normalize in `crates/parallax-ingest/src/{traces,logs,metrics}.rs` | Ports/config in server serve path |
| Sentry envelope HTTP ingest | **shipped** | `crates/parallax-server/src/sentry_http.rs` (router merge in `serve.rs`); parse `crates/parallax-ingest/src/sentry_envelope.rs`; derive `crates/parallax-analysis/src/sentry.rs` | Plan **118 DONE** — residual multi-SDK compatibility ledger still **unproven** ([validation/2026-07-plan-118-sentry-envelope](validation/2026-07-plan-118-sentry-envelope/README.md)); not a "future adapter" |
| Durable raw-frame spool | **shipped** | `crates/parallax-spool/` | OTLP + Sentry frames; forensic PSPL1 trail |
| Error derivation from OTLP (exception spans, ERROR/FATAL logs) | **shipped** | `crates/parallax-analysis/src/derive.rs`, `fingerprint.rs` | Deterministic fingerprints |
| Ingest-time PII scrub of all raw signals | **planned / unproven gate (A6)** | design [capture/redaction.md](capture/redaction.md); product redaction is bundle/metadata path today | Do not claim full ingest scrub as shipped |

---

## 2. Storage stack

| Claim | Status | In-repo pointer | Notes |
| --- | --- | --- | --- |
| GreptimeDB telemetry (native OTLP tables) | **shipped** (mandatory) | `crates/parallax-greptime/`; policy [decisions/native-otel-tables.md](decisions/native-otel-tables.md), [decisions/storage-engine.md](decisions/storage-engine.md) | ClickHouse = research comparator only — **never product fallback** |
| Turso metadata | **shipped** (mandatory) | `crates/parallax-metadata/`; [decisions/metadata-store.md](decisions/metadata-store.md) | No Postgres/rusqlite product fallback |
| `StorageAdapter` / capability ports | **shipped** (test/fake boundary) | `crates/parallax-storage/` | Capability split + test fakes; **not** multi-engine product promise |
| Extension tables (`error_events`, exemplars, etc.) | **partial** | Greptime adapter + [decisions/native-otel-tables.md](decisions/native-otel-tables.md); open work e.g. [plans/089-extension-table-grpc-writes.md](../../plans/089-extension-table-grpc-writes.md) | Derived signals only; raw signals stay native |
| Large-server four-way storage cost/latency | **unproven gate** | [storage/greptimedb-vs-clickhouse/](storage/greptimedb-vs-clickhouse/), [research-agenda.md](research-agenda.md) §5 | Local small benches exist; sized server tier deferred |

---

## 3. API / CLI / UI / MCP

| Claim | Status | In-repo pointer | Notes |
| --- | --- | --- | --- |
| GraphQL query API | **shipped** | `crates/parallax-api/`; exported schema `ui/graphql/schema.graphql` | **80** Query fields, **15** Mutation fields (counted 2026-07-17 from schema); older "76/14" text is stale |
| CLI (`parallax serve` + client commands) | **shipped** | `crates/parallax-cli/src/main.rs` | serve, invocation, issue, trace, metrics, logs, traces, sql, doctor, prune, uninstall, context |
| TanStack Start UI | **shipped** | `ui/src/routes/`, `ui/src/features/` | Issues, traces, logs, metrics, services, invocations, investigations, dashboards, ecosystem, SQL, alerts, tests, … (~16 feature modules) |
| Local-stdio read-only MCP | **shipped** | `crates/parallax-mcp/`; [validation/2026-07-plan-112-product-mcp](validation/2026-07-plan-112-product-mcp/README.md) | Plan **112 DONE**. Tools: `parallax_issue_context`, `parallax_agent_session_show` |
| Remote MCP / protected transport | **planned** | Plan 109 residual in [validation/2026-07-plan-109-v2-auth](validation/2026-07-plan-109-v2-auth/); design [decisions/agent-access-surface.md](decisions/agent-access-surface.md) | Not product until transport lands |
| Live SSE / alerting | **shipped** (V1-scope) | `crates/parallax-server/src/live/`, `alerting/` | Not on-call suite |
| Read-only SQL against GreptimeDB | **shipped** | GraphQL `sql` + CLI `parallax sql` | SELECT-shaped only |

---

## 4. Evidence, redaction, analysis

| Claim | Status | In-repo pointer | Notes |
| --- | --- | --- | --- |
| Bounded evidence bundle assembly | **shipped** (code) / **unproven gate (A1)** | `crates/parallax-evidence/src/bundle/`; schemas `schema/evidence-bundle.v1.schema.json`, `v2` | Code + schema exist; **bundle-vs-raw agent fix quality unproven** |
| Bundle-path redaction policy | **shipped** (code) / **A6 residual** | `crates/parallax-redaction/` (`REDACTION_POLICY_V1` = `redaction-lite-v3`); applied in evidence projection | Not full A6 canary program completion |
| Story / gaps / agent session projections | **shipped** | `parallax-evidence` story/gaps/agent_session; GraphQL `story`, `evidenceGaps`, `agentSession` | Claude Code adapter modules; broader capture adapters in [plans/120-agent-session-capture-adapters.md](../../plans/120-agent-session-capture-adapters.md) |
| Test reporting / flakiness analysis | **partial** | `parallax-analysis` test_*; GraphQL `testCases`/`testCase`; UI tests routes; [plans/154…](../../plans/154-playground-capability-and-test-observability.md), [155…](../../plans/155-test-reporting-surface.md) | Derivation + explorer exist; product surface still plan-owned |
| Fixer / outcome loop | **planned** + **unproven** | [plans/123-fixer-outcome-loop.md](../../plans/123-fixer-outcome-loop.md); [decisions/fixer-boundary.md](decisions/fixer-boundary.md) | Context engine ≠ fixer; no measured outcome ledger |
| Autonomous fix-loop kernels | **PoC-only** | `poc/evidence-loop/`; [architecture/poc-evidence-loop-coverage.md](architecture/poc-evidence-loop-coverage.md) | Executable kernels ≠ product gate pass |

---

## 5. Deploy / CI / agents

| Claim | Status | In-repo pointer | Notes |
| --- | --- | --- | --- |
| Deploy/change context capture | **partial** | server `deploy_backfill`, `github_webhook`; evidence github_deploy modules; [decisions/github-deploy-change-adapter.md](decisions/github-deploy-change-adapter.md) | Adapters present; full product depth varies |
| CI / GitHub Actions evidence | **partial** | evidence github_actions; analysis junit/nextest | Not a full CI product |
| Coding-agent session capture | **partial** | evidence `claude_code`, `agent_session`; plan 120 | Local MCP consumes sessions when present |
| SSO / multi-tenant RBAC | **planned** | V2 auth design / plan 109 family | Explicitly not V1 maturity |

---

## 6. Workspace shape (sanity)

| Item | Reality (2026-07-17) |
| --- | --- |
| Product crates | 17 workspace members under `crates/` (see [architecture/rust-workspace-map.md](architecture/rust-workspace-map.md)) |
| Mandatory engines | GreptimeDB + Turso only (`AGENTS.md`, metadata/storage decisions) |
| UI runtime | Bun-only TanStack Start (`ui/`) |
| License | Apache-2.0 (root `LICENSE`) |
| Active implementation ownership | Numbered files under `plans/` only (no dual plan trees) |

---

## 7. Open product/market gates (not code checklist)

| Gate | Status | Home |
| --- | --- | --- |
| A1 bundle value vs raw context | **unproven** | [validation/a1-bundle-value/](validation/a1-bundle-value/) |
| A2 paying segment / demand | **partial desk; interviews open** | [validation/a2-user-demand.md](validation/a2-user-demand.md), monetization notes |
| A3 schema adoption | open | [validation/a3-schema-corpus.md](validation/a3-schema-corpus.md) |
| A4 correlation reliability | design + partial code | [capture/correlation.md](capture/correlation.md) |
| A5 stack | **decided shipped stack**; residual measurement | [decisions/stack-decision.md](decisions/stack-decision.md) |
| A6 redaction red-team | residual | [capture/redaction.md](capture/redaction.md) |
| A7 scope discipline | standing | [validation/a7-scope.md](validation/a7-scope.md) |

---

## 8. How research should use this ledger

1. **Front doors** ([README.md](README.md), [research-agenda.md](research-agenda.md), [00-vision/](00-vision/), decisions marked "current truth") must match this table in present tense.
2. **Market pages** compare peers multi-angle (capability + price/TCO or "no public number" + license/contribute + hidden ops/lock-in/ecosystem cost). Parallax cells that are **shipped** must not read as 🏗 planned; **unproven** claims stay unproven.
3. **Historical notes** keep evidence; they get a **superseded / historical** banner and a pointer here or to the owning decision — no silent contradiction with `main`.
4. **Corrections:** prefer PRs with dated primary sources or crate paths that falsify a row. Bias toward transparency over brand.

---

## Changelog

| Date | Change |
| --- | --- |
| 2026-07-17 | Initial ledger from workspace + schema + plans inventory (research code-reality audit). |
