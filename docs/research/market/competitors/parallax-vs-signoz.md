# Parallax vs SigNoz

> One-to-one comparison. **No pro-Parallax bias.** Where SigNoz is ahead, ahead
> is written. Where Parallax's edge is only *planned* or *unproven*, that is
> stated, not hidden.
>
> Research date: **2026-07-17**. Refreshed from current primary sources this
> pass; version/license/pricing drift re-checked. **Pass 101 pin:** GitHub
> **v0.133.0** (2026-07-15), **30,261★**, last push 2026-07-17; README still:
> **“[Noz](https://signoz.io/docs/ai/noz/) is available only on SigNoz Cloud”**
> (MCP self-host path separate). Legacy source:
> [`../signoz-deep-research.md`](../signoz-deep-research.md) (2026-06-22) — kept
> as a lead, corrected here where the market moved.

## TL;DR verdict (scoped per axis)

- **Maturity, breadth, community, shipped dashboards/alerting/SLO/service-map,
  and agent/MCP tool surface: SigNoz wins, plainly.** It is years ahead of
  pre-release Parallax as a *working observability platform*.
- **Single-binary local-first simplicity, Rust-first runtime-error capture,
  Sentry-compatible ingest, and the bounded redacted evidence-bundle +
  fix-outcome thesis: Parallax's intended edges.** Of these, only the
  architectural/local-first shape is real today; Sentry-envelope ingest is shipped;
  bundle value (A1) and outcome loop remain unproven — not full parity.
- **SigNoz pressures the agent-native + "evidence" narrative harder than any
  other OSS tool** (hosted+self-hosted MCP, skills marketplace, evals, "Postmortem
  Evidence Pack" / "open investigation format"). It does **not** ship Parallax's
  specific artifact — confirmed again this pass: the "investigation format" is
  still product copy, **no published versioned schema**.

## SigNoz — what it is (verified 2026-07-17)

Open-source, **OpenTelemetry-native full-stack observability platform**: unified
logs, traces, and metrics in one app. Positioned as an open Datadog / New Relic
alternative. APM, distributed tracing, dashboards, alerting, exception
monitoring, LLM observability.

| | SigNoz | Source |
|---|---|---|
| **Latest version** | **v0.133.0** (2026-07-15); v0.131.0 (2026-07-01) bumped **ClickHouse → 25.12.5** to stay on a supported release | [github.com/SigNoz/signoz/releases](https://github.com/SigNoz/signoz/releases), [signoz.io/changelog](https://signoz.io/changelog/) |
| **Cadence** | Very fast: ~4 minor releases in ~4 weeks (v0.130→v0.133 as of 2026-07-17); still **pre-1.0** | changelog |
| **Stars** | **30,251** (GitHub API, 2026-07-17 pass 59/60 — up from ~27.4k in June 2026) | [github.com/SigNoz/signoz](https://github.com/SigNoz/signoz) |
| **License** | Core platform **MIT-Expat**; `ee/` + `cmd/enterprise/` proprietary; **`signoz-mcp-server` Apache-2.0** (separate repo) | LICENSE, ee/LICENSE, maintainer [discussion #4231](https://github.com/SigNoz/signoz/discussions/4231) |
| **Languages** | TypeScript ~53% (React UI), Go ~37% (backend), Python ~5% | GitHub |
| **Telemetry store** | **ClickHouse** + ClickHouse Keeper (ZooKeeper still the shipped reality — Keeper *supported* but charts/compose not switched: issues signoz#7002, charts#610) | docs + issues |
| **Metadata store** | Relational — **PostgreSQL** (added 2025, default compose) / SQLite; **not** ClickHouse | [signoz.io/blog/oss-improvements](https://signoz.io/blog/oss-improvements/) |
| **Deploy** | Docker Compose / Standalone, Docker Swarm, Kubernetes (Helm), single Go binary on VM | [install/self-host](https://signoz.io/docs/install/self-host/) |
| **Company** | SigNoz, **YC W21**; Pranay Prateek (CEO), Ankit Nayan (CTO); SF, founded 2021 | [YC](https://www.ycombinator.com/companies/signoz) |
| **Funding** | ~$6.5M total ($5.4M 2023-09-28 led by SignalFire; "Series A" per SigNoz vs "seed" per Crunchbase/PitchBook — unresolved) | [TechCrunch](https://techcrunch.com/2023/09/28/open-source-datadog-rival-signoz-lands-on-the-cloud-with-6-5m-investment/) |

### Architecture (post v0.76 / 2025-03-13 consolidation)

`query-service` + `frontend` + `alertmanager` merged into **one Go binary
`signoz`** (bundles React UI, API/query server, OpAMP server, Ruler,
Alertmanager). Still separate processes: `signoz-otel-collector` (OTLP ingest +
processing), **ClickHouse (+ Keeper)**, **PostgreSQL/SQLite** (control-plane
metadata), `schema-migrator`. **The "single binary" is the control-plane app
only** — ClickHouse + collector stay separate. **No embedded-ClickHouse,
single-process local mode exists.**

### Pricing (re-cited 2026-07-17)

| Tier | Price | Notes |
|---|---|---|
| **Community (self-host)** | **$0** | fully free, self-managed |
| **Teams Cloud** | **$49/mo base**, usage: **$0.30/GB logs & traces**, **$0.10 / million metric samples** | retention 15d–1yr (logs/traces), 1–13mo (metrics); startup program → $19/mo |
| **Enterprise** | custom, **~$4,000/mo floor** | — |

Feature gating: SSO/SAML add-on (Teams >$999/mo) or Enterprise; **RBAC + audit
logs = Enterprise ("coming soon")**. MCP server itself is Apache-2.0, free,
self-hostable against any instance. **In-product AI "Noz" = SigNoz Cloud only**
(pass 60: [docs.signoz.io/docs/ai/noz](https://signoz.io/docs/ai/noz/) tagged
**SigNoz Cloud**; README Agent-Native section + concurrent market note
[oss-agent-surface-gating-2026-07-17.md](../oss-agent-surface-gating-2026-07-17.md)).
Self-host keeps free MCP path to external agents; **does not get free Noz**.
Hosted MCP / other Teams-Cloud AI surfaces remain cloud-tiered. **No per-seat
fees.** Verify pricing against live [signoz.io/pricing](https://signoz.io/pricing/)
before quoting — vendor pages change.

> **Economics (multi-angle):** SigNoz Community is **$0 software** with **non-zero
> ops cost** (ClickHouse + collector + Postgres/SQLite; ~5-container local).
> Teams Cloud trades money for reduced ops. Parallax: **no public number**
> (pre-release); closest proxy = self-hosted GreptimeDB+Turso compute + eng time.
> **Contribute:** SigNoz MIT core accepts OSS PRs; large community (30k★). Parallax
> Apache-2.0 also accepts PRs; **tiny** ecosystem today. **Lock-in:** both self-host
> OTLP-friendly; SigNoz ClickHouse stack vs Parallax GreptimeDB+Turso — migration
> cost is real either way. A direct Parallax-vs-SigNoz TCO figure is
> **benchmark-dependent and unmeasured**.

## Axis-by-axis comparison

### Signal coverage

| Signal | SigNoz | Parallax | Who |
|---|---|---|---|
| Logs | ✅ | ✅ (V1) | tie |
| Traces / distributed tracing | ✅ mature | ✅ (V1) | **SigNoz** (depth, service-map) |
| Metrics / dashboards | ✅ full | 🟡 partial / planned | **SigNoz** |
| Errors / exceptions | ✅ OTel span-events (queryable, **no issue lifecycle**) | ✅🧪 derived `error_event` + fingerprint (shipped, pre-release) | **SigNoz** on tooling maturity; **Parallax** on error-as-managed-event model (unproven value) |
| Continuous profiling | 🟡 partial | ❌ | **SigNoz** |
| RUM / session replay | ❌ | ❌ | tie (neither) |
| LLM / agent spans | ✅ LLM observability | 🟡🧪 agent-session + Claude Code modules (partial) | **SigNoz** (shipped product depth) |
| CI / test results | ❌ | 🟡🧪 test explorer + span-derived results (partial; plans 154/155) | **Parallax** (partial code; maturity unproven) |

SigNoz is the broader, more mature signal platform today. Parallax's signal
*model* differs (derived error events, CI/agent evidence): error derivation and
Sentry envelope are **shipped**; test surfaces are **partial**; platform maturity
and A1 remain unproven.

### Ingestion & transport

- **SigNoz: OTLP-native by design** — gRPC (4317) + HTTP (4318) for all three
  signals; Prometheus scrape/remote-write for metrics migration; logs+traces
  effectively OTLP-only. Error model = OTel span-events
  (`exception.type/message/stacktrace`), dedicated Exceptions tab.
  **No Sentry-envelope / Sentry-SDK ingestion; no issue grouping/lifecycle.**
  Migration from Sentry = re-instrument with OTel. Bear-case trigger
  ("SigNoz adds Sentry ingest") **has not fired.**
- **Parallax: OTLP-native in V1; Sentry-envelope ingest is shipped**
  (`sentry_http` + envelope parse/derive/ack; plan 118 DONE
  hardening only). SigNoz has **no** Sentry-envelope path.

> Both are OTLP-native. **Parallax alone claims the Sentry-envelope migration
> lane** with a shipped ingest adapter; plan 118 is DONE; multi-SDK matrix remains unproven.

### Storage architecture

| | SigNoz | Parallax |
|---|---|---|
| Telemetry store | **ClickHouse** + Keeper (columnar, all 3 signals; tiered → S3 cold) | **GreptimeDB** native OTLP tables (columnar) |
| Metadata store | PostgreSQL / SQLite | **Turso (libSQL)** |
| Hot-path ingest | collector pipeline → ClickHouse exporters (`signoz_logs/traces/metrics`); `resource_fingerprint` ORDER BY, per-column codecs (ZSTD/Gorilla/Delta/T64), sparse-PK pruning | decode-once / move-ownership-forward (zero-copy by design) |
| Object-storage cold tier | ClickHouse tiered storage → S3 | 🟡 GreptimeDB object-store tier (planned/unmeasured) |
| Throughput | vendor ~55k logs/sec (2023, logs-only, **~3yr stale**, no current trace/metric numbers) | **no public number**; benchmark-dependent |

ClickHouse is a **proven, scale-hardened** telemetry store with years of
production use across the industry. GreptimeDB is younger; Parallax's bet is on
its native-OTLP-table performance path, which is **benchmark-dependent and not
yet measured head-to-head**. On *proven scale*, SigNoz wins; on *engine fit for
OTLP-native + local-first*, the bet is unmeasured.

### Query & correlation

SigNoz offers three query interfaces: **Query Builder v5** (GUI, all signals),
**raw ClickHouse SQL** (all signals, dashboards), **PromQL** (metrics-only).
Cross-signal trace→log→metric drilldown is mature. **No evidence-pin / typed
correlation artifact.** Parallax's bet is the typed evidence-graph + bundle with
query manifest + raw refs — **partially shipped** (bundle/query surfaces exist; depth not at SigNoz parity).

### Dashboards & visualization

**SigNoz: ✅ mature** dashboard builder, panels, service map, templating.
Parallax: **intentionally minimal, object-centric UI** (not a dashboard suite) —
by design, not a gap to "fix". On dashboard capability, SigNoz plainly wins;
Parallax explicitly cedes that ground.

### Alerting & on-call

**SigNoz: ✅ mature** alert rules, routing, Alertmanager, on-call lifecycle.
Parallax: 🟡 partial/planned. **SigNoz wins.**

### Profiling

**SigNoz: 🟡 partial.** Parallax: ❌. **SigNoz wins** (and Coroot/Sentry win
harder). Parallax has no profiling story.

### Developer experience

SigNoz: polished quickstart, broad docs, large community, fast cadence. Local
friction is the weak spot (see Architecture). Parallax: Rust-first runtime
ergonomics + single-binary local loop intended as a DX wedge — real
architecturally, early in productization.

### AI-native / agent-context story (Parallax's wedge — fastest-moving)

| Capability | SigNoz | Parallax |
|---|---|---|
| Official MCP server | ✅ **Apache-2.0, both hosted + self-hosted**, **v0.8.0** (2026-07-15), stdio + HTTP(OAuth) | ✅🧪 local-stdio read-only shipped (plan 112 DONE; remote deferred) |
| MCP tool surface | **41 tools** (pass 41 recount of [signoz-mcp-server README](https://github.com/SigNoz/signoz-mcp-server) table) — includes write/destructive tools (create/update/delete alerts, dashboards, views, notification channels) | read-only bounded projection (local-stdio MCP shipped; remote 🏗) |
| Skills marketplace | ✅ `agent-skills` repo — 12 skills, incl. read-only `signoz-investigating-alerts` RCA skill **with eval cases** | ❌ |
| AI root-cause | ✅ MCP RCA skill (3-tier, cite-every-claim); **Noz in-product AI = Cloud only** (pass 60) | 🟡 planned |
| Coding-agent clients | ✅ Claude Code, Cursor, VS Code/Copilot, Codex, Gemini | ✅ (intended) |
| **Portable, versioned evidence-bundle schema** | ❌ — "Postmortem Evidence Pack" = **ad-hoc LLM-generated markdown per investigation; no JSON schema, no version/provenance/redaction/query-manifest/outcome fields** | ✅🧪 **code-shipped** (value **unproven A1**) |
| **Fix-outcome loop** | ❌ — investigations end at ranked causes | 🟡🧪 **partial**: offline residual plan **123 DONE**; draft-PR deferred; live value **unproven** |
| Redaction / PII before agent access | ❌ not surfaced | 🟡🧪 **code-shipped** bundle-path (`REDACTION_POLICY_V1`); A6 residual (not full ingest scrub) |

> **SigNoz is the strongest OSS competitor on agent/MCP maturity — by a wide
> margin.** Real hosted+self-hosted MCP, a skills marketplace, evals, and active
> postmortem framing. It pressures *any* "agent-native observability"
> positioning. Parallax's only differentiated AI claim is the **bounded,
> redacted, versioned bundle + outcome loop as a typed artifact** — and that is
> **unproven** (A1: does such a bundle beat raw live-query context for agent fix
> quality?). Do not read "Parallax wins AI" here; on shipped AI surface, SigNoz
> is far ahead.

### Architecture & deployment model

| | SigNoz | Parallax |
|---|---|---|
| Single binary, no Docker | ❌ (~5 containers: signoz, collector, ClickHouse, Keeper, Postgres) | **✅ single binary (GreptimeDB + Turso)** |
| Local RAM floor | **≥4 GB** (Docker); idle baselines ~1.5–2 GB | low (intended) |
| Self-host free tier | ✅ Community $0 | ✅ (OSS self-host) |
| Air-gapped / offline | ✅ | ✅ |
| Multi-tenancy / SSO-RBAC | 🟡 RBAC/audit Enterprise "coming soon", pre-1.0 | 🏗 planned |

**SigNoz's biggest concrete weakness vs Parallax is local-run friction**: no
true single-binary/embedded-engine mode, mandatory ~5-container ClickHouse
stack, ≥4 GB RAM. This is the one axis where Parallax's architectural choice
(GreptimeDB+Turso single binary) is a **real, today differentiator** — assuming
Parallax ships the local loop it designs.

### Operational footprint & scalability

SigNoz: moderate operational burden (multi-container ClickHouse stack, Keeper,
Postgres), proven at moderate scale, vendor-cited ~55k logs/sec (stale).
Parallax: lighter intended footprint, **unmeasured** scale. On proven scale +
operational track record, **SigNoz wins**; on intended lightness, Parallax's bet
is unmeasured.

### Security

| | SigNoz | Parallax |
|---|---|---|
| SSO/SAML | 🟡 Teams >$999 add-on / Enterprise | 🏗 planned |
| RBAC + audit logs | 🟡 **Enterprise, "coming soon", pre-1.0** | 🏗 planned |
| PII scrub / redaction | ❌ not surfaced | 🟡🧪 **code-shipped** bundle-path (`REDACTION_POLICY_V1`); A6 residual (not full ingest scrub) |
| Transport security | standard | standard |
| Compliance (SOC2/etc.) | ❌ self-attest only | ❌ not yet |

Both are immature on enterprise security. SigNoz is slightly ahead on shipped
auth tiers; Parallax intends redaction-as-gate. **Neither has a compliance
certification.**

### Privacy & compliance

SigNoz: no marketed PII/redaction story; data ownership good (self-host).
Parallax: redaction + data-ownership intended as a wedge (self-host, open).
Neither holds SOC2/HIPAA. Roughly even on paper; Parallax's redaction intent is
unproven.

### Openness, licensing & lock-in

- **SigNoz: MIT-Expat core (permissive) + Apache-2.0 MCP** — genuinely open on
  the parts that matter; `ee/` proprietary. ClickHouse is open. Query = standard
  SQL/PromQL (no proprietary lock-in language). Strong openness posture.
- **Parallax: Apache-2.0 (entire repo), GreptimeDB + Turso open.** Also open.

Both score well on openness. SigNoz's edge: maturity + community. Parallax's
edge: fully Apache-2.0 (no proprietary `ee/` split).

### Extensibility

SigNoz: collector pipeline processors, OTel ecosystem, dashboards-as-code, MCP
tools, skills. Mature extensibility. Parallax: intended pipeline/processor model
+ bundle schema. **SigNoz wins** on shipped extensibility.

### Pricing & economics

SigNoz: transparent usage pricing ($0.30/GB logs&traces, $0.10/M metric
samples, $0 self-host, no per-seat). Parallax: **no public number**; proxy =
self-hosted compute. A direct cost comparison is **benchmark-dependent,
unmeasured**. On pricing *transparency*, SigNoz wins today; on *cost ceiling*,
self-hosted Parallax could undercut but that is unmeasured.

## Where SigNoz plainly wins (no bias)

1. **Maturity + community** — **30k+ stars**, years of shipping, fast cadence.
2. **Breadth** — unified mature dashboards/alerting/SLO/service-map/exceptions/LLM obs.
3. **Agent/MCP maturity** — hosted+self-hosted MCP, skills marketplace, evals; the strongest in OSS.
4. **Proven storage scale** — ClickHouse is battle-tested; GreptimeDB is not, head-to-head.
5. **Pricing transparency** — public, usage-based, no per-seat.
6. **Query flexibility** — Builder + SQL + PromQL.

## Where Parallax intends an edge (scoped; code-shipped pieces + unproven value gates)

1. **Single-binary local-first** — the one *real today* architectural edge (GreptimeDB+Turso, no ~5-container stack).
2. **Sentry-compatible ingest lane** — shipped; plan 118 DONE; multi-SDK compatibility ledger unproven.
3. **Portable versioned redacted evidence bundle** — **code-shipped**; value **unproven (A1 gate)**.
4. **Fix-outcome loop** — **partial**: offline residual plan **123 DONE**; draft-PR deferred; live product value **unproven**.
5. **Rust-first runtime-error capture** — real bet, early.
6. **Fully Apache-2.0** — no proprietary `ee/` split (minor vs MIT core).

## Watch triggers — re-evaluate SigNoz if it:

- Publishes a **versioned, portable investigation/evidence schema** (provenance,
  redaction, raw-refs, query manifest, missing-evidence, outcome rows) → would
  pressure Parallax's A3/bundle thesis. **Checked 2026-07-17: still no schema —
  "open investigation format" remains product copy over live MCP queries.**
- Adds **Sentry envelope ingestion** or an **error-issue lifecycle** → closes the
  migration lane.
- Adds **fix-outcome tracking** → closes the core-thesis differentiator.
- Ships a **single-binary / embedded-engine local mode** → closes the local wedge.
- Adds a **redaction/PII layer** on the MCP surface.

## Sources (checked 2026-07-17 unless noted)

- [github.com/SigNoz/signoz](https://github.com/SigNoz/signoz) — README, LICENSE, releases, architecture
- [github.com/SigNoz/signoz LICENSE](https://github.com/SigNoz/signoz/blob/main/LICENSE), [ee/LICENSE](https://github.com/SigNoz/signoz/blob/develop/ee/LICENSE), [discussion #4231](https://github.com/SigNoz/signoz/discussions/4231) (license split)
- [github.com/SigNoz/signoz/releases](https://github.com/SigNoz/signoz/releases) — latest **v0.133.0** (2026-07-15); **30,251 stars** (GitHub API, 2026-07-17 pass 59/60)
- [Noz docs](https://signoz.io/docs/ai/noz/) — **SigNoz Cloud only** (pass 60)
- [OSS agent surface gating note](../oss-agent-surface-gating-2026-07-17.md)
- [github.com/SigNoz/signoz-mcp-server/releases](https://github.com/SigNoz/signoz-mcp-server/releases) — MCP server latest **v0.8.0** (2026-07-15); **41 tools** counted from README tool table (pass 41; was ~38)
- [github.com/SigNoz/signoz/releases](https://github.com/SigNoz/signoz/releases)
- [signoz.io/docs/architecture](https://signoz.io/docs/architecture/), [install/self-host](https://signoz.io/docs/install/self-host/)
- [signoz.io/blog/oss-improvements](https://signoz.io/blog/oss-improvements/) (SQLite→Postgres metadata)
- [signoz.io/docs/ai/signoz-mcp-server](https://signoz.io/docs/ai/signoz-mcp-server/), [ai/use-cases/postmortem-evidence-pack](https://signoz.io/docs/ai/use-cases/postmortem-evidence-pack/), [agent-native-observability](https://signoz.io/agent-native-observability/)
- [github.com/SigNoz/signoz-mcp-server](https://github.com/SigNoz/signoz-mcp-server), [github.com/SigNoz/agent-skills](https://github.com/SigNoz/agent-skills)
- [signoz.io/pricing](https://signoz.io/pricing/)
- [TechCrunch — SigNoz funding](https://techcrunch.com/2023/09/28/open-source-datadog-rival-signoz-lands-on-the-cloud-with-6-5m-investment/), [YC W21](https://www.ycombinator.com/companies/signoz)
- Legacy lead (corrected here): [`../signoz-deep-research.md`](../signoz-deep-research.md) (2026-06-22)
