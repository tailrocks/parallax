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
| Error derivation from OTLP (exception spans, ERROR/FATAL logs) | **shipped** | `crates/parallax-analysis/src/derive.rs`, `fingerprint.rs`, `identity.rs`; Turso upsert `crates/parallax-metadata/src/turso/occurrences.rs` | Deterministic fingerprints; issue identity is `(service, fingerprint)` (Turso PK + GraphQL `issue(service, fingerprint)`). Occurrence events carry `invocationId`/`sessionId`/`serviceVersion`/`environment`. **`regressed` derived** on recurrence: `status = CASE WHEN status = 'resolved' THEN 'regressed' ELSE status END` (keep `resolved_at`). GraphQL filter `open\|resolved\|regressed`; `issueSetStatus` still only `open\|resolved`. Tests `new_occurrence_regresses_resolved_issue`, `resolved_issue_regresses_on_new_occurrence`. UI `ISSUE_STATUS.regressed`. |
| Ingest-time PII scrub of all raw signals | **planned / unproven gate (A6)** | design [capture/redaction.md](capture/redaction.md); product redaction is bundle/metadata path today | Do not claim full ingest scrub as shipped |

---

## 2. Storage stack

| Claim | Status | In-repo pointer | Notes |
| --- | --- | --- | --- |
| GreptimeDB telemetry (native OTLP tables) | **shipped** (mandatory) | `crates/parallax-greptime/`; policy [decisions/native-otel-tables.md](decisions/native-otel-tables.md), [decisions/storage-engine.md](decisions/storage-engine.md) | ClickHouse = research comparator only — **never product fallback** |
| Turso metadata | **shipped** (mandatory) | `crates/parallax-metadata/`; [decisions/metadata-store.md](decisions/metadata-store.md) | No Postgres/rusqlite product fallback |
| `StorageAdapter` / capability ports | **shipped** (test/fake boundary) | `crates/parallax-storage/` | Capability split + test fakes; **not** multi-engine product promise |
| Extension tables (`error_events`, exemplars, etc.) | **partial** | Greptime adapter + [decisions/native-otel-tables.md](decisions/native-otel-tables.md); row-API write still blocked on crates.io `greptimedb-ingester` 0.18.0 | Derived signals only; raw signals stay native |
| Large-server four-way storage cost/latency | **unproven gate** | [storage/greptimedb-vs-clickhouse/](storage/greptimedb-vs-clickhouse/), [research-agenda.md](research-agenda.md) §5 | Local small benches exist; sized server tier deferred |

---

## 3. API / CLI / UI / MCP

| Claim | Status | In-repo pointer | Notes |
| --- | --- | --- | --- |
| GraphQL query API | **shipped** | `crates/parallax-api/`; exported schema `ui/graphql/schema.graphql` | **81** Query fields, **15** Mutation fields (R1+R3 restamp: R3 `rumSessions` + `rumSession` 78→80, R1 `sourceMaps` query + `sourceMapUpload` mutation + `ErrorEvent.mappedFrames` 80→81 / 14→15; prior: audit-M1 restamp `chartAnnotations` new in PR71 `ec0eea1c`, 77→78; `service` is now optional). **Count method (SoT):** walk `type Query` / `type Mutation` in the generated SDL; **skip `"""…"""` description blocks** (and single-line `"` descriptions); then match `fieldName(` or `fieldName:`. A naive line regex that does **not** skip descriptions falsely reports extra fields. Re-count before changing this row. `chartAnnotations(fromNanos, toNanos, service: Option)` — omit service collects ≤32; kind `release`, title=version. Trace field `dominantDbQueries`. |
| CLI (`parallax serve` + client commands) | **shipped** | `crates/parallax-cli/src/main.rs` | serve, invocation, issue, trace, metrics, logs, traces, sql, doctor, prune, uninstall, context |
| TanStack Start UI | **shipped** | `ui/src/routes/`, `ui/src/features/` | Issues (`/issues/$service/$fingerprint`, status filter+badge `open\|resolved\|regressed`), traces (`dominantDbQueries` panel), logs, metrics (chart annotation overlay + **Traces around peak**), services, invocations, investigations, dashboards, ecosystem, SQL, alerts (Create-alert from metric/`log_count`/`error_rate`/`p95_latency`), tests, **RUM `/rum`**, … (~17 feature modules). Issue identity is `(service, fingerprint)`. |
| Local-stdio read-only MCP | **shipped** | `crates/parallax-mcp/`; [validation/2026-07-plan-112-product-mcp](validation/2026-07-plan-112-product-mcp/README.md) | Plan **112 DONE**. Tools: `parallax_issue_context`, `parallax_agent_session_show` |
| Remote MCP / protected transport | **planned** | Plan 109 residual in [validation/2026-07-plan-109-v2-auth](validation/2026-07-plan-109-v2-auth/); design [decisions/agent-access-surface.md](decisions/agent-access-surface.md) | Not product until transport lands |
| Live SSE / alerting | **shipped** (V1-scope) | `crates/parallax-server/src/live/`, `alerting/` | Not on-call suite |
| Read-only SQL against GreptimeDB | **shipped** | GraphQL `sql` + CLI `parallax sql` | SELECT-shaped only |
| Typed metric `rate`/`increase` | **shipped** | `crates/parallax-storage/src/adapter_math.rs`; GraphQL `metricQuery` agg `rate`\|`increase` | Reset-clamped; kind-legal. **No PromQL UI** (2026-09-13 decision: keep typed builder). |
| Browser RUM surface | **partial** | `ui/src/features/rum/`, route `/rum/` | Session inbox + timeline over `rumSessions` / `rumSession` (R3) alongside the vitals/errors/journeys projection over `tracesPage` / `histogramQuantile` / `issues`. Minified JS stacks resolve via the R1 artifact store (issue detail `mappedFrames`), not on the RUM surface itself. |
| Issue status `regressed` | **shipped** | Turso `occurrences.rs`; GraphQL `issues(status:)`; UI `ISSUE_STATUS` / issues filter+badge | Derived on recurrence only. Mutation rejects `regressed`. |
| Chart annotations | **shipped** | GraphQL `chartAnnotations`; UI `loadChartAnnotations` + metric-detail `ReferenceLine` | Same store as `releases`. Service optional (omit → ≤32). Kind `release`, title=version. Tests `chart_annotations_are_release_windows`, `chart_annotations_without_service_collect_window`. |
| Trace `dominantDbQueries` | **shipped** | GraphQL `Trace.dominantDbQueries`; UI `trace-detail-query.ts` + panel | Ranked normalized SQL. Page must select GraphQL field, not re-rank client-side. |
| Alert-from-log / alert-from-trace | **shipped** | UI `encodeLogsAlertGraduation` (`log_count`); `encodeTracesAlertGraduation` (`error_rate` if errors-only else `p95_latency`) | Create-alert URLs into `/alerts`. Metric path already had Create alert. |
| Metric spike → related traces | **shipped** | UI `peakWindowFromSeries` / `tracesAroundPeakSearch`; metric detail **Traces around peak** | Peak window ±60s → `/traces` custom range. |
| JS source maps (R1 artifact store) | **shipped** | Turso `source_maps` (`crates/parallax-metadata/src/turso/source_maps.rs`, schema v6); v3 decoder `crates/parallax-analysis/src/sourcemap.rs`; GraphQL `sourceMaps` / `sourceMapUpload` / `ErrorEvent.mappedFrames`; UI mapped-frames render + `SourceMapCard` upload | Keyed (service, version, file) + optional debug id. Map JSON is private: no GraphQL field exposes it. Live proof `crates/parallax-server/tests/sourcemap_greptime.rs` (managed engine: upload → ingest minified → resolved 3:2/`render`, unmapped release stays unresolved). |
| Sampling policy | **absent** | — | Freeze P0 remaining-gap (R2). Do not claim shipped. |

---

## 4. Evidence, redaction, analysis

| Claim | Status | In-repo pointer | Notes |
| --- | --- | --- | --- |
| Bounded evidence bundle assembly | **shipped** (code) / **unproven gate (A1)** | `crates/parallax-evidence/src/bundle/`; schemas `schema/evidence-bundle.v1.schema.json`, `v2` | Code + schema exist; **bundle-vs-raw agent fix quality unproven** |
| Bundle-path redaction policy | **shipped** (code) / **A6 residual** | `crates/parallax-redaction/` (`REDACTION_POLICY_V1` = `redaction-lite-v3`); applied in evidence projection | Not full A6 canary program completion |
| Story / gaps / agent session projections | **shipped** | `parallax-evidence` story/gaps/agent_session; GraphQL `story`, `evidenceGaps`, `agentSession` | Claude Code adapter **shipped** (plan **120 DONE** — [evidence](validation/2026-07-plan-120-claude-code/README.md)). Broader multi-tool adapters are design-only (no active plan owner) |
| Test reporting / flakiness analysis | **partial** | `parallax-analysis` test_*; GraphQL `testCases`/`testCase`; UI tests routes; plans 154/155 retired (Git history); derivation + explorer remain in product source | Derivation + explorer exist; product surface still plan-owned |
| Fixer / outcome loop | **partial** (offline residual **DONE**) / **unproven** product value | `crates/parallax-evidence/src/fixer_outcome.rs`; Turso `fixer_outcomes`; [validation/2026-07-plan-123-fixer-offline](validation/2026-07-plan-123-fixer-offline/README.md); design [decisions/fixer-boundary.md](decisions/fixer-boundary.md) | Plan **123 DONE** offline SM + append-only outcomes. Draft-PR adapter deferred; no measured live outcome ledger; context engine ≠ fixer |
| Autonomous fix-loop kernels | **PoC-only** | `poc/evidence-loop/`; [architecture/poc-evidence-loop-coverage.md](architecture/poc-evidence-loop-coverage.md) | Executable kernels ≠ product gate pass |

---

## 5. Deploy / CI / agents

| Claim | Status | In-repo pointer | Notes |
| --- | --- | --- | --- |
| Deploy/change context capture | **partial** | server `deploy_backfill`, `github_webhook`; evidence github_deploy modules; [decisions/github-deploy-change-adapter.md](decisions/github-deploy-change-adapter.md) | Adapters present; full product depth varies |
| CI / GitHub Actions evidence | **partial** | evidence github_actions; analysis junit/nextest | Not a full CI product |
| Coding-agent session capture | **partial** (Claude Code **shipped**) | evidence `claude_code`, `agent_session`; CLI import; [plan 120 DONE](validation/2026-07-plan-120-claude-code/README.md) | Claude Code path closed plan 120; other agents design-only / new plan only; MCP consumes sessions when present |
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
| A3 schema adoption | **artifacts shipped** / **adoption open** | [validation/a3-schema-corpus.md](validation/a3-schema-corpus.md), [a3-schema-claim-recheck-2026-07-17.md](validation/a3-schema-claim-recheck-2026-07-17.md) — v1/v2 JSON Schema + tests exist; external adoption ledger empty |
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
| 2026-07-17 | Re-verify: plan **123 DONE** offline — fixer row → partial + validation path (dead `plans/123-*` link removed). |
| 2026-09-13 | GraphQL row recounted per its own method: Query **76→77** (one field added since 2026-07-17), Mutation 14 holds; naive-regex 80/15 warning re-confirmed. |
| 2026-09-13 | Pass 68 (GOAL restamp, HEAD `1cad2a50`): issue identity `(service, fingerprint)` shipped; `rate`/`increase` reset-clamped shipped (`parallax-storage/src/adapter_math.rs`); exp histograms converted at ingest; `/rum` UI is a projection over traces/metrics/issues (not a session store); source maps / sampling / chart annotations / `regressed` **absent**. GraphQL still 77/14. |
| 2026-09-13 | Hotfix restamp: derived issue `regressed` shipped; GraphQL `chartAnnotations` + metric overlay shipped; `dominantDbQueries` UI wired; alert-from-log/trace graduation shipped; spike→traces peak window shipped. Remaining: source maps, sampling, RUM session model, Sentry SDK ledger, release-health rollup. |
| 2026-09-13 | Hotfix restamp HEAD `e0c27c5f` (`goal/final-p0-hotfix`): `regressed` shipped (derived, mutation still open\|resolved); `chartAnnotations` shipped (service optional, omit→≤32, kind=release); `dominantDbQueries` on Trace + UI; alert graduation logs `log_count` / traces `error_rate`\|`p95_latency`; metric spike → traces-around-peak. GraphQL Query count still 77 (`chartAnnotations` already present). Sampling / source maps / RUM session / crash-free release health still absent. |
| 2026-09-13 | Audit M1 fix: GraphQL row **77→78** — `chartAnnotations` is new in PR71 `ec0eea1c` (absent at `cb9a3077`, where SoT recount = 77/14); the `e0c27c5f` entry above wrongly said "already present". Mutation 14 holds. Gap-matrix + feature-inventory restamped to match. M2 parallax-side verified clean: ranked error-event SQL fix + managed-engine test `issue_events_greptime.rs` on main (PR70); no stale defect NOTE on parallax side (playground hop-2 NOTE/fallback left untouched, playground-owned). |
| 2026-09-14 | R3: RUM session model shipped — `rumSessions` / `rumSession` GraphQL (Query **78→80**), `/rum` sessions inbox + timeline (`ui/src/features/rum/`), managed-engine live test `rum_sessions_greptime.rs`; gap-matrix header + feature-inventory restamped to 80/14. |
| 2026-09-14 | R1 shipped (`goal/r1-sourcemaps`): JS source-map artifact store — Turso `source_maps` (schema v6) keyed (service, version, file); owned v3 VLQ decoder (`parallax-analysis/src/sourcemap.rs`); GraphQL `sourceMaps` + `sourceMapUpload` + `ErrorEvent.mappedFrames` (**80→81 / 14→15** per SoT recount on the R3 merge); issue-detail mapped-frames render + `SourceMapCard` upload; map JSON private by construction (no GraphQL field). Live proof `sourcemap_greptime.rs` on managed engine. Remaining P0-next: R2 sampling, R5 release health (R3 sessions + R4 SDK matrix shipped). |
