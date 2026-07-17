# Parallax vs TMA1

> An unbiased, one-to-one comparison. Research date: **2026-07-17** (pass 63: **14th consecutive watch UNFIRED** — still **v0.2.0-alpha12 / 109★**; recent still install/perf).
> Sources: [github.com/tma1-ai/tma1](https://github.com/tma1-ai/tma1), [tma1.ai](https://tma1.ai/), and the legacy [tma1-deep-research.md](../tma1-deep-research.md) (2026-06-22, a full source-code teardown) as the lead.
>
> **Bottom line up front:** TMA1 is the **single closest architectural competitor
> to Parallax** — a single Go binary that embeds GreptimeDB, ingests OTLP into
> native tables, and serves a **read-only MCP "context bundle" to coding agents**.
> On **local agent-loop specialization and MCP tool breadth for that loop**, TMA1
> remains ahead (even at alpha). **Parallax has also shipped** embedded-style
> GreptimeDB supervision + Turso, OTLP gRPC/HTTP, Sentry-envelope ingest, derived
> error events, local-stdio read-only MCP (`parallax-mcp`), and a code-level
> bounded/redacted bundle assembler — see
> [code-reality-ledger.md](../../code-reality-ledger.md). Remaining gaps are
> **product maturity, local-agent polish, and unproven A1 bundle value**, not
> "architecture only on paper." The honest differentiator is still product
> *intent* and *scope*: TMA1 is **local dev-machine AI-coding-agent loop
> observability**; Parallax targets **production-incident evidence** plus the
> unproven portable redacted-bundle thesis. Critical watch: **if TMA1 adds
> production-error derivation, Sentry ingest, redaction, or an outcome loop, it is
> a direct collision.** As of 2026-07-17, **that trigger has not fired.**

## What each product is

- **TMA1** (`tma1-ai/tma1`) — **local-first observability for LLM/AI coding agents**: a single Go binary that embeds **GreptimeDB** (downloaded as a child process, `minRequiredVersion v1.1.3` — bumped v1.0.2→v1.1.2→v1.1.3, 2026-07-17), records every LLM call on the local machine, and routes observations back into the agent's next turn via hooks + a **read-only 7-tool MCP** (`get_context_bundle`, `get_session_state`, `get_anomalies`, `get_build_status`, `get_external_changes`, `get_project_state`, `get_peer_sessions`) + 6-rule anomaly detection. **Apache-2.0**, Go ~51% + vanilla-JS embedded dashboard (uPlot). **109 GitHub stars, v0.2.0-alpha12 (2026-07-17) — alpha/pre-1.0; 5 alpha bumps since alpha7 (2026-06-08), fast cadence.** Wired into Claude Code / Codex / Copilot CLI / OpenClaw. **No pricing/cloud/SaaS/auth/multi-tenant — explicitly local-only.** Org also ships `openfuse` (Langfuse→GreptimeDB fork). **Latest confirmed tag v0.2.0-alpha12 (2026-07-17, GitHub API).**
- **Parallax** — open-source (Apache-2.0), Rust-first, self-hostable **execution-context engine**: OTLP-native ingest of traces/logs/metrics + CLI/agent traces, derives owned `error_event`s, fingerprints, correlates into a typed evidence graph, serves bounded/redacted/schema-valid evidence bundles to humans and coding agents. GreptimeDB + Turso. **Pre-release.**

These are the closest pair in the set. Both: single binary, embedded GreptimeDB, OTLP-native, read-only MCP for coding agents, Apache-2.0. The differences are scope + a few spine pieces TMA1 lacks.

## The bundle-name trap (must be stated plainly)

**TMA1's "context bundle" and Parallax's "evidence bundle" share a name but are different artifacts.** This is the single most important distinction and a common source of confusion:

- **TMA1 `perception.Bundle`** — a **live, unversioned, unredacted session snapshot** generated at `time.Now()` for agent-loop continuity (what the agent should know this turn). Not portable, not versioned, not redacted, not pinned to evidence.
- **Parallax evidence bundle** — a **portable, versioned, redacted, schema-valid** evidence package pinned to a specific incident/failure, served safely to agents after a redaction gate.

Same shape (a typed object an agent reads); fundamentally different artifact. Do not assume TMA1 ships Parallax's bundle — it does not.

## Signal coverage

| Signal | TMA1 (shipped, alpha) | Parallax (pre-release; ✅🧪=code-shipped) |
| --- | --- | --- |
| LLM / agent call traces (local) | ✅ core (records every LLM call) | ✅ (🏗) |
| Production app traces (OTLP) | 🟡 (any OTel GenAI app can send; secondary) | ✅🧪 OTLP traces (shipped, pre-release) |
| Production logs / metrics | ❌ (not a production-services backend) | ✅🧪 OTLP logs/metrics (shipped, pre-release) |
| Errors / exceptions (production) | ❌ (no production error-event derivation) | ✅🧪 derived `error_event` + fingerprint (shipped, pre-release) |
| Anomaly detection | ✅ 6-rule (cost/sessions/build/anomaly) | 🟡 (🏗) |
| Build / external-change / project state | ✅ (build sensor, external changes) | ✅ CI/deploy/change (🏗) |
| Sentry envelope / DSN | ❌ | ✅ shipped |
| Bounded/redacted/portable bundle | ❌ (live unversioned unredacted) | 🟡🧪 code (A1 unproven) |

**Verdict:** TMA1 is **narrow but deep on the local-agent-loop** (LLM calls + anomalies + build/changes), and ships that today. Parallax is **broader on production telemetry + error workflow** (error/Sentry path shipped; outcome loop unproven). They are scoped to different primary jobs; on raw coverage they barely overlap except at "agent traces."

## Ingestion & transport

- **OTLP:** TMA1 runs an OTLP reverse-proxy on `:14318` into GreptimeDB native tables (logs/traces), uses the Flow engine for continuous aggregations, GreptimeDB-isms (`json_get_string`, `matches_term()` FULLTEXT, `uddsketch`, SKIPPING/INVERTED/FULLTEXT indexes). **Native GreptimeDB usage — the same engine Parallax chose.**
- **Sentry envelope:** TMA1 has **none**. Parallax ships envelope ingest (plan 118 DONE).
- **Capture:** TMA1 records local LLM calls via agent hooks (Claude Code/Codex/Copilot CLI/OpenClaw). Parallax captures CLI/agent/CI sessions + production OTLP.

**Verdict:** on **embedded-GreptimeDB-OTLP-native ingest, tied in design; TMA1 ships it.** On Sentry-envelope + production capture, **Parallax ships envelope ingest** (plan 118 DONE).

## Storage architecture — same engine, different metadata story

- **TMA1:** **embedded GreptimeDB** (child process, `~/.tma1/data`, standalone mode) using **native OTLP tables + Flow engine**. **No separate metadata store** — telemetry + `tma1_*` operational tables + anomaly log + migration ledger all live in the one GreptimeDB instance. In-process bounded write semaphore (no external broker).
- **Parallax:** GreptimeDB (native OTLP tables) **+ Turso (libSQL) metadata** — a deliberate split. Plans an Apache Iggy durable stream (TMA1 has only an in-process semaphore).

**Verdict:** on the **GreptimeDB-native telemetry choice, identical** — strong mutual validation. TMA1 proves embedded GreptimeDB + OTLP + read-only-MCP works in a shipped binary. Parallax's **Turso metadata split + durable-stream backpressure** are design choices TMA1 doesn't make (Parallax argues they're needed for production multi-entity state + replay; unproven at Parallax's scale).

## Query & correlation

- **TMA1:** SQL/DataFusion (via GreptimeDB) + the 7 MCP tools that project a bounded, read-only view (session state, anomalies, build, external changes, project state, peer sessions). Designed so an agent reads a safe slice, not raw queries.
- **Parallax:** evidence-graph correlation + bounded bundle + run_id/invocation stitching + evidence pinning.

**Verdict:** on **read-only-safe agent projection, TMA1 is the closest shipped thing to Parallax's thesis** — its MCP is genuinely read-only and bounded. This is the strongest convergence in the set. Parallax's evidence-graph + pinned bundle is richer/different but **unproven (A1).**

## Error tracking & workflow

- **TMA1:** **no production error-event derivation, no fingerprinting, no issue lifecycle, no fix-outcome loop.** Anomaly resolution is the closest (anomaly → resolved).
- **Parallax:** derived `error_event` + fingerprint (**shipped**) + fix-outcome offline residual (**plan 123 DONE**; live value **unproven**).

**Verdict:** on **production error workflow + outcome loop, Parallax targets cells TMA1 does not occupy** — error derivation **shipped** (pre-release); fix-outcome offline residual **plan 123 DONE**; live product value **unproven.** Real Parallax-favorable axis (gated on value, not code existence).

## AI-native / agent-context story — the convergence point

- **TMA1's position:** it is **literally a context engine for coding agents** — record the agent's LLM calls, detect anomalies, feed a bounded read-only view back into the next turn. This is the closest existing product to Parallax's "context engine, not the fixer" framing. **Read-only by design** (7 tools, none mutate), local, Apache-2.0.
- **Parallax's claim:** bounded, redacted, agent-safe evidence bundle for production incidents (**code-shipped**, A1 unproven) + fix-outcome offline residual (plan **123 DONE**).

**Honest verdict:** TMA1 is **the closest shipped realization of "a read-only context projection for coding agents"** — and it is Apache-2.0, local-first, GreptimeDB-backed, exactly Parallax's substrate. On the *architecture-for-agent-loops*, **TMA1 is ahead (shipped, even alpha).** Parallax's differentiation is entirely: (a) **production-incident scope** (TMA1 is dev-machine-local), (b) **redaction gate** (TMA1's bundle is unredacted), (c) **versioned/portable bundle** (TMA1's is live/unversioned), (d) **fix-outcome loop** (offline residual plan **123 DONE**; live value unproven), (e) **Sentry-envelope compat** (shipped), (f) **Turso metadata** (shipped). Bundle/redaction **code-shipped**; A1 value **unproven.** The honest, uncomfortable read: **a lot of "Parallax's wedge" is already shipped by TMA1 in narrower form.** Parallax's bet is that production-incident + redaction + outcome is a different, valuable job — unproven (A1).

## Architecture & deployment

- **TMA1:** single Go binary (`CGO_ENABLED=0`), GreptimeDB child process, embedded JS dashboard, `~/.tma1/` data. **Local-only, single-user, no auth/multi-tenant.** Apache-2.0.
- **Parallax:** single-binary Rust target, GreptimeDB + Turso, local-first + production tiers, air-gap-capable. Apache-2.0.

**Verdict:** on **local-agent-loop polish and MCP tool breadth for that loop, TMA1 still leads** (purpose-built alpha). Parallax **ships** supervised GreptimeDB + Turso, OTLP, Sentry envelope, UI/CLI/MCP, and production-oriented metadata — different maturity and scope, not "architecture planned-only." TMA1 is local-only by intent; Parallax targets production multi-entity — different deployment tiers.

## Operational footprint

- **TMA1:** trivial local install (download + child GreptimeDB). Zero enterprise ops (it's not enterprise). Alpha-quality at scale.
- **Parallax:** self-hosted GreptimeDB + Turso + engine; single-binary target lowers burden.

**Verdict:** on **local dev-loop install simplicity, TMA1 wins** (it's purpose-built tiny + local). Parallax's target is parity for local, broader for production.

## Scalability & performance

- **TMA1:** alpha, 109 stars, single-machine scale (it's local-first by design — scale = one dev machine). GreptimeDB itself is proven; TMA1's harness is pre-1.0.
- **Parallax:** unproven at production scale; benchmark-dependent.

**Verdict:** not comparable head-to-head — TMA1 is local-single-machine by design; Parallax targets production multi-entity. **Different scale regimes.**

## Security

- **TMA1:** local-only, single-user — **no auth/RBAC/audit** (not needed locally). No redaction gate (bundle is unredacted).
- **Parallax:** SSO/RBAC/audit planned; redaction (A6) designed as first-class.

**Verdict:** TMA1 sidesteps enterprise security (local-only). Parallax's **redaction-before-agent-access** is a real contrast — TMA1 feeds the agent **unredacted** local data (acceptable locally, not for shared/production contexts). Scoped.

## Openness, licensing & vendor lock-in

- **TMA1:** **Apache-2.0**, fully open, local, no commercial product. **Zero lock-in.**
- **Parallax:** Apache-2.0, fully open.

**Verdict:** **tied** — both Apache-2.0, fully open, no lock-in. No edge either way.

## Pricing & economics (incl. hidden cost of "free")

| | TMA1 | Parallax |
| --- | --- | --- |
| **Sticker price** | $0 (Apache-2.0, no commercial product) | **No public number** (pre-release); Apache-2.0 core |
| **Hidden / total cost** | Operator time, GreptimeDB child process, alpha quality risk, no vendor support SLA | Same class of self-host ops (GreptimeDB + Turso + engine), plus incomplete maturity; no paid support yet |
| **Contribute?** | ✅ open; small ecosystem (~100★) — patches welcome, few reviewers | ✅ open; also early ecosystem — contribution path real, throughput unproven |
| **Lock-in** | Low (local, Apache, GreptimeDB native) | Low (OTLP + portable bundle design) |
| **Ecosystem size** | Tiny; agent-hook niche | Tiny; production-incident niche |

**Verdict:** both free at the license layer. **"Free" is not zero-cost** — self-host ops, upgrade risk, and small ecosystems are real costs for both. Not a sticker-price contest; neither has a measured TCO study.

## Where TMA1 plainly wins (or matches)

- **Local agent-loop specialization** — records every LLM call + anomalies + build/external change sensors; MCP catalog (7 tools) is broader for that job than Parallax's current 2-tool local MCP.
- Read-only-safe MCP discipline (7 non-mutating tools) — strong shipped agent-projection discipline; Parallax also ships read-only local MCP with a narrower tool surface.
- Trivial local install + local dev-loop focus.
- Apache-2.0, zero lock-in, zero cost.
- Anomaly detection + build/external-change/project-state capture (loop-feedback signals).

## Where Parallax honestly edges TMA1

- **Production-incident scope** — TMA1 is dev-machine-local; Parallax targets production services. *(Real scope difference.)*
- **Production error-event derivation + fingerprinting** — TMA1 has none. *(Real; Parallax shipped.)*
- **Fix-outcome loop** — TMA1 has none. *(Real unoccupied cell; offline residual plan **123 DONE**; live value **unproven**.)*
- **Bounded, versioned, redacted, portable bundle** — TMA1's bundle is live/unversioned/unredacted. *(Real artifact difference; Parallax **code-shipped**, A1 **value unproven**.)*
- **Sentry-envelope compatibility** — TMA1 has none. *(Real; Parallax shipped.)*
- **Turso metadata split** — production multi-entity state. *(Design choice; scale advantage unproven.)*
- **Rust vs Go** — different substrate. *(Minor; both compile to single binaries.)*

> **Honest summary:** TMA1 is the reference competitor and #1 watch target. It **validates** the embedded-GreptimeDB + OTLP + read-only-MCP pattern in a shipping alpha. Parallax **also ships** that pattern plus production-oriented pieces (error derivation, Sentry envelope, Turso, UI/CLI, redacting bundle assembler). The remaining honest gap is **not "architecture only on paper"** — it is **maturity, local-agent depth, and A1 (does the portable redacted bundle beat raw/TMA1 live context for fix quality?)**. If TMA1 extends to production-errors/Sentry/redaction/outcome, the collision is direct.

## Watch triggers (the point of tracking TMA1)

Re-verify each pass. Direct collision if TMA1 adds any of:

1. **Production error-event derivation / fingerprinting** — from real services, not just agent anomalies.
2. **Sentry envelope / DSN ingest.**
3. **Redaction gate** on the bundle (currently unredacted).
4. **Fix-outcome loop** (accepted/rejected/reverted/recurred).
5. **CI / deploy capture** beyond the build sensor.

**As of 2026-07-17: none fired.** TMA1 remains local-dev-agent-loop scoped. This is the single most important drift signal in the whole comparison.

## Open questions / what measurement would settle

- ~~Exact latest TMA1 release~~ → **pinned v0.2.0-alpha12; 109★.** Watch triggers **14th UNFIRED** (…/59/62/**63**): still install/perf — **zero** prod-error / Sentry / redaction / outcome / deploy / fingerprint hits.
- **A1 gate vs TMA1:** if a team already runs TMA1 for local agent loops, does Parallax's production-incident bundle add measurable value, or does TMA1's narrower scope suffice for their job? Unproven.
- **TMA1 production extension** — track the watch triggers above each pass.

## Sources (accessed 2026-07-17)

- [github.com/tma1-ai/tma1](https://github.com/tma1-ai/tma1); [tma1.ai](https://tma1.ai/).
- Legacy internal teardown: [tma1-deep-research.md](../tma1-deep-research.md) (2026-06-22 — full source-code read: bundle.go, anomaly.go, mcp/tools.go, derive.go, greptimedb/process.go, install.go).
- Parallax side: [00-vision/](../../00-vision/), [architecture/v1-implementation-spec.md](../../architecture/v1-implementation-spec.md), [validation/a1-bundle-value/](../../validation/a1-bundle-value/), [decisions/native-otel-tables.md](../../decisions/native-otel-tables.md).
