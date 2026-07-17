# Parallax vs Sentry

> An unbiased, one-to-one comparison. Research date: **2026-07-17**.
> Sources: live [Sentry pricing](https://sentry.io/pricing/) (accessed 2026-07-17),
> [Sentry OTLP docs](https://docs.sentry.io/concepts/otlp/),
> [Sentry blog: "Sentry vs OpenTelemetry"](https://blog.sentry.io/sentry-opentelemetry-work-together/),
> third-party pricing analyses dated 2026, and the legacy
> [sentry-deep-research.md](../sentry-deep-research.md) (2026-06) as a lead.
>
> **Bottom line up front:** Sentry is the error-tracking incumbent and Parallax's
> explicit benchmark ("be simpler/cheaper than self-hosted Sentry"). On **error
> workflow, SDK breadth, maturity, AI debugging (Seer), profiling, replay, and
> compliance, Sentry is far ahead of pre-release Parallax.** Parallax's honest
> edges are OTLP-native ingest (Sentry is not), a single-binary self-host that is
> genuinely simpler than Sentry's ~20–40-container stack, Apache-2.0 vs
> source-available FSL, and the *unproven* evidence-bundle + fix-outcome thesis.

## What each product is

- **Sentry** (Functional Software, Inc.) — the dominant application-monitoring / error-tracking platform: error monitoring, tracing, logs, **metrics (new in 2026)**, session replay, profiling (UI + continuous), size analysis, cron + uptime monitoring, and an **AI debugging suite called Seer** (Agent, Autofix, AI Code Review, Root Cause Analysis). Source-available under **FSL** (converts to Apache/MIT after 2 years); Python + Rust (Relay) + Kafka/Snuba/ClickHouse backend. Has a 30+ SDK fleet (web, mobile, game). © 2026 pricing page live.
- **Parallax** — open-source (Apache-2.0), Rust-first, self-hostable **execution-context engine**: OTLP-native ingest of traces/logs/metrics + CLI/agent traces, derives owned `error_event`s, fingerprints, correlates into a typed evidence graph, serves bounded/redacted evidence bundles to humans and coding agents. GreptimeDB + Turso. **Pre-release.** **Ships a Sentry-envelope ingest path** (`sentry_http.rs` endpoint + `sentry_envelope.rs` parser + `ErrorSource::SentryEnvelope` derivation, verified in `crates/` 2026-07-17) to absorb Sentry's SDK fleet — landed, not just planned.

These overlap most directly on **error tracking + tracing**. Sentry is a broad product suite; Parallax is a narrow context engine. Compare axis-by-axis.

## Signal coverage

| Signal | Sentry (shipped, 2026) | Parallax (planned) |
| --- | --- | --- |
| Errors / exceptions | ✅ best-in-class grouping + lifecycle + ownership | ✅ derived `error_event` + fingerprint (🏗) |
| Tracing / distributed traces | ✅ | ✅ OTLP traces (🏗) |
| Logs | ✅ | ✅ OTLP logs (🏗) |
| Metrics | ✅ **Metrics (new 2026)** — via Sentry SDK, not OTLP | ✅ OTLP metrics (🏗) |
| Continuous profiling | ✅ UI Profiling + Continuous Profiling | ❌ |
| Session replay | ✅ | ❌ |
| Cron / uptime monitoring | ✅ | ❌ (out of scope) |
| Size analysis (mobile builds) | ✅ | ❌ |
| LLM / agent spans | 🟡 (AI Observability solution listed; not a core LLM-trace product) | ✅ (🏗) |
| CI / deploy / change context | 🟡 releases + deploy tracking | ✅ (🏗) first-class |

**Verdict:** Sentry's surface is far broader and all shipped. Parallax is deliberately narrower. On coverage, **Sentry wins decisively.**

## Ingestion & transport — a real Sentry weakness Parallax targets

- **OTLP:** Sentry **does** ingest OTLP — but **open beta, HTTP-only, traces and logs only; no OTLP metrics, no OTLP gRPC** ([official docs](https://docs.sentry.io/concepts/otlp/), confirmed 2026-07-17). Announced open beta 2025-02-25; **still open beta, not GA**, on both SaaS and self-host (self-hosted native OTLP ingest since ~v25.8.0, [#3830](https://github.com/getsentry/self-hosted/issues/3830) closed 2026-05-19). Sentry is **not OTLP-native in storage**: OTLP data is mapped into Sentry's own envelope/span model and ClickHouse/Snuba backend. This is a genuine, current gap — if your stack standardizes on OTLP metrics, Sentry cannot ingest them.
- **Sentry envelope / DSN:** Sentry's native protocol. **Parallax now ships Sentry-envelope ingest** (`crates/parallax-ingest/src/sentry_envelope.rs` + `crates/parallax-server/src/sentry_http.rs`, verified against code 2026-07-17) — it receives Sentry SDK envelopes and derives `error_event`s (`ErrorSource::SentryEnvelope`). The ingest lane is **no longer "planned."** Sentry still wins on the 30-SDK fleet breadth + issue lifecycle, but Parallax speaks the envelope today.
- **SDK fleet:** Sentry ships 30+ language/platform SDKs (web, iOS, Android, React Native, Flutter, Unity, game consoles) — the broadest in error tracking. Parallax relies on OTel SDKs **plus the shipped Sentry-envelope ingest path**. On SDK breadth, **Sentry still wins decisively** (30+ SDKs vs Parallax's envelope receiver — receiving envelopes ≠ matching the fleet).
- **Self-host ingest:** Sentry ships **Relay** (an open ingestion proxy you can self-host in front of Sentry Cloud). Parallax's whole pipeline is self-hosted by design.

**Verdict:** on OTLP-native ingest (especially metrics), **Parallax's design beats Sentry's current state** (Sentry has no OTLP metrics). On SDK breadth and envelope-native maturity, **Sentry wins decisively.**

## Storage architecture

- **Sentry:** ClickHouse + Kafka + Snuba + Postgres + Redis; object storage for attachments/replays. Self-hosted stack is heavy (legacy note: ~20–40 containers, 16–32 GB RAM locally). Performance is battle-tested at large scale (Sentry's own business proves it).
- **Parallax:** GreptimeDB (telemetry, native OTLP tables) + Turso (metadata). Single-binary target; designed for cheap self-host. Parallax storage performance vs Sentry is **benchmark-dependent and unproven.**

**Verdict:** on proven-at-scale + operational maturity, **Sentry wins.** On self-host simplicity and open native-OTLP storage, **Parallax's target wins (by design).**

## Query & correlation

- **Sentry:** strong cross-signal pivoting (trace ↔ log ↔ error ↔ replay ↔ release ↔ suspect commit), Release Health, Ownership/Code Owners, Suspect Commits (auto-blame). Best-in-class for the error→cause→owner loop.
- **Parallax:** evidence-graph correlation + run_id/invocation stitching + evidence pinning; the bounded bundle is the differentiated artifact (but **unproven**, A1 gate).

**Verdict:** on the shipped error→cause→owner correlation loop, **Sentry wins decisively** (and defines the category). Parallax's evidence-bundle abstraction is a *different* axis (agent-actionable, bounded, redacted), not a like-for-like better Sentry — value unproven.

## Error tracking & workflow — Sentry's moat

- **Sentry:** deterministic grouping/fingerprinting, full issue lifecycle (resolve/regress/ignore/assign), ownership rules + code owners, suspect commits, integrations (Slack/GitHub/Jira), alerting + anomaly detection. This is the category Sentry created and still leads.
- **Parallax:** derives `error_event` + deterministic fingerprint + a *fix-outcome loop* (accepted/rejected/reverted/recurred) — the latter is the genuinely unoccupied cell, but **planned/unproven.**

**Verdict:** on error-workflow maturity, **Sentry wins decisively.** Parallax's only differentiated workflow claim is the fix-outcome loop, which is unshipped and unproven (A1).

## Dashboards & visualization

- **Sentry:** custom dashboards (10/20/unlimited by tier), metric monitors (20/1000/custom), anomaly detection (Business+). Mature.
- **Parallax:** V1 UI = Sentry-grade issues list/detail + predefined/user dashboards (TanStack Start + shadcn). Narrower.

**Verdict:** **Sentry wins** on dashboard maturity; Parallax's V1 is intentionally minimal.

## Alerting & on-call

- **Sentry:** issue alerts, metric monitors, anomaly detection, cron/uptime alerts, integrations to PagerDuty/Slack/etc. Mature (no native on-call/escalation suite — pairs with external paging).
- **Parallax:** minimal alerting in V1 scope; no on-call suite (out of scope).

**Verdict:** **Sentry wins** on alerting breadth. (Neither owns full on-call/incident management; Datadog does.)

## Profiling

- **Sentry:** UI Profiling (+$0.25/hr) + Continuous Profiling (+$0.0315/hr). Shipped.
- **Parallax:** none in V1 scope.

**Verdict:** **Sentry wins.** Out of Parallax's current scope.

## AI-native / agent-context story

- **Sentry's AI (Seer, shipped, additional cost):** Root Cause Analysis, **Fix Generation (Autofix → creates a branch + opens a PR with the fix)**, Error Prediction, AI Code Review, and a **Seer Agent** (Jan 2026: expanded into local-development + code-review debugging). Pricing confirmed: **$40 / active contributor / month, unlimited usage**, add-on for any paid plan (Team/Business/Enterprise) — [BusinessWire announcement](https://www.businesswire.com/news/home/20260127739891/en/Sentry-Adds-Local-Development-and-Code-Review-Debugging-to-Seer) (2026-01) + [sentry.io/product/seer](https://sentry.io/product/seer/). This is shipped, production AI debugging — directly overlapping Parallax's "context → fix" thesis.
- **Parallax's AI claim (planned/unproven):** a bounded, redacted, agent-safe evidence bundle served to coding agents (CLI/HTTP first, MCP after safety gates) — a *context engine*, not a chat/autofix tool.

**Honest verdict:** Seer already does much of "context → root cause → proposed fix → PR" today, from SaaS, at scale. On every *shipped* AI axis, **Sentry is ahead.** Parallax's differentiated claim is the bounded/redacted/agent-safe bundle — unoccupied but **unproven** (A1 gate). The burden of proof that a Parallax bundle beats Seer-as-context for agent fix quality is on Parallax and unmet. Sentry's Seer is a real competitive pressure on the A1 thesis, written plainly.

A Sentry weakness here: Seer is a SaaS, additional-cost, human-facing AI tool — **not a self-hosted, read-only, bounded agent-context projection.** That cell stays unoccupied. But "unoccupied" ≠ "valuable."

## Architecture & deployment model

- **Sentry:** SaaS primary (sentry.io, multi-region, data-residency on Enterprise); **self-hosted** via `getsentry/self-hosted` (single-node Docker Compose; latest **26.4.2** per [github.com/getsentry/self-hosted/releases](https://github.com/getsentry/self-hosted/releases), fetched 2026-07-17; **26.5.0** "Launchpad" noted as the next release). Recent drift: 26.4.2 hotfixes a Relay HTTP/2 auth bug (relay#5913); 26.4.1 patches a **SAML SSO auth vuln (GHSA-rcmw-7mc7-3rj7)** + a taskworker memory leak + an OpenTelemetry-projects crash (#4262); PostgreSQL 14.22 (PG17 migration coming). *(⚠️ the legacy [sentry-deep-research.md](../sentry-deep-research.md) claimed self-hosted 26.6.0 (2026-06-16) — the live releases page shows 26.4.2 newest; unresolved, treat 26.4.2 as verified-latest.)* Architecture: **Relay → Kafka → Snuba → ClickHouse** + Postgres/Redis/Memcached + Celery workers — ~20–40 containers, ~16–32 GB RAM locally (legacy internal measurement; heavy operator burden, widely corroborated). Relay can run on-prem as an ingest proxy. FSL source-available.
- **Parallax:** single-binary self-host target, local-first, air-gap-capable, Apache-2.0. Designed to be *much* simpler to self-host than Sentry.

**Verdict:** on **self-host simplicity, local-first loop, and air-gap, Parallax's target beats Sentry** (Sentry self-host is notoriously heavy — this is Parallax's explicit "simpler than self-hosted Sentry" wedge, and it is *real in design*, though Parallax is pre-release). On **managed SaaS scale/multi-region, Sentry wins; Parallax has no SaaS.**

## Operational footprint

- **Sentry Cloud:** zero backend ops; cost is money. **Sentry self-host:** real Docker/ClickHouse/Kafka ops — documented as heavy.
- **Parallax:** self-hosted GreptimeDB + Turso + Parallax engine; single-binary target lowers burden but production-grade operation is still real work.

**Verdict:** on **operator burden for the self-host path, Parallax's target is lower** (Sentry self-host is heavy). On **SaaS zero-ops, Sentry Cloud wins.** Scoped.

## Scalability & performance

- **Sentry:** proven at very large scale (its business demonstrates it). Specific numbers are vendor claims, not independently measured here.
- **Parallax:** unproven at production scale; **benchmark-dependent.**

**Verdict:** on **proven-at-scale, Sentry wins conclusively.** Parallax cannot yet make a measured scale claim. (Flagged for the benchmark program.)

## Security

- **Sentry:** SSO (Google/GitHub/SAML2/SCIM — Business+), RBAC, Manage PII, Relay on-prem ingest, audit. Mature.
- **Parallax:** SSO/RBAC/audit planned, not shipped. Redaction (A6) designed as first-class.

**Verdict:** on **shipped security posture, Sentry wins decisively.** Parallax's only security-relevant edge is redaction-before-agent-access (narrower, unproven).

## Privacy & compliance

- **Sentry:** **SOC 2, ISO 27001, HIPAA (BAA),** data residency, SSO/SCIM — on Business/Enterprise. Mature.
- **Parallax:** no compliance certifications yet (pre-release). Redaction designed but unattested. Total data ownership via self-host.

**Verdict:** on **compliance certifications, Sentry wins decisively.** On **data ownership/sovereignty (self-host, air-gap), Parallax wins by design.**

## Openness, licensing & vendor lock-in

- **Sentry:** **FSL (Functional Source License)** — source-available, converts to Apache/MIT after 2 years. Self-hostable but heavy. Moderate lock-in: proprietary envelope/DSN format, Snuba/Relay stack; SDK lock-in is the main friction (instruments your code). Less lock-in than Datadog (source is visible, self-host exists), more than Apache-2.0 OSS.
- **Parallax:** Apache-2.0, fully open, OTLP-native (standard format in/out), portable bundle. Lowest lock-in by construction.

**Verdict:** on **openness and lock-in cost, Parallax wins** (Apache-2.0 OTLP-native vs FSL proprietary-format). Sentry's FSL is more open than fully-closed SaaS but less than Apache — a real middle position.

## Extensibility

- **Sentry:** 100+ integrations (Slack/GitHub/Jira/PagerDuty/...), Sentry CLI, **official Sentry MCP**, webhooks, public API, plugins. Mature ecosystem.
- **Parallax:** OTel-native (any OTel instrumentation), CLI/HTTP/MCP surfaces, pipeline/processor model, webhooks (planned). Smaller ecosystem.

**Verdict:** on **integration ecosystem, Sentry wins** (mature, broad). Both ship an MCP surface.

## Pricing & economics — real numbers

Sentry pricing is **public** ([sentry.io/pricing](https://sentry.io/pricing/), accessed 2026-07-17, © 2026). Event-based; each plan includes base quotas, overage is pay-as-you-go:

| Plan | Price (annual) | Users | Errors | Logs | App Metrics | Spans | Replays | Retention |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Developer | **Free** | 1 | 5k | 5 GB | 5 GB | 5M | 50 | 30-day |
| Team | **$26/mo** | unlimited | 50k | 5 GB (+$0.50/GB) | 5 GB (+$0.50/GB) | 5M | 50 | up to 90-day |
| Business | **$80/mo** | unlimited | 50k | 5 GB (+$0.50/GB) | 5 GB (+$0.50/GB) | 5M | 50 | 90-day + sampled |
| Enterprise | **Custom** | unlimited | custom | custom | custom | custom | custom | custom |

Pay-as-you-go overage (Team, per error, descending tiers): 50k–100k **$0.0003625**, 100k–500k **$0.0002188**, 500k–10M **$0.0001875**, 10M–20M **$0.0001625**, 20M+ **$0.0001500** per error. Uptime +$1.00/alert; Cron +$0.78/monitor; UI Profiling +$0.25/hr; Continuous Profiling +$0.0315/hr.

**Seer (AI):** **$40 / active contributor / month, unlimited usage**, add-on for any paid plan ([BusinessWire 2026-01](https://www.businesswire.com/news/home/20260127739891/en/Sentry-Adds-Local-Development-and-Code-Review-Debugging-to-Seer), [sentry.io/product/seer](https://sentry.io/product/seer/)). "Active contributor" = anyone who triggers Seer usage.

**Parallax pricing:** none public yet (pre-release). Stated shape: Apache-2.0 open core + gated enterprise-ops + managed cloud + outcome-priced fixer.

**Honest cost read:** Sentry's entry is cheap (free / $26) and the per-error unit is small, but volume at scale + profiling + replay + Seer add-ons compound (event-based metering across errors/logs/metrics/spans/replays/profile hours). Whether Parallax self-host is cheaper at a given workload is **benchmark-dependent and unmeasured** — do not assert a saving not measured.

## Where Sentry plainly wins

- Error-workflow maturity (grouping, lifecycle, ownership, suspect commits) — category-defining.
- SDK fleet (30+, incl. mobile/game).
- Shipped AI debugging (Seer: RCA + Autofix→PR + AI Code Review).
- Cross-signal pivoting, release health, dashboards, anomaly detection.
- Profiling, session replay, cron/uptime, size analysis.
- Compliance (SOC2/ISO27001/HIPAA), SSO/SCIM, SaaS scale.
- Maturity + proven-at-scale.

## Where Parallax honestly edges Sentry

- **OTLP-native ingest (incl. metrics)** — Sentry ingests OTLP traces+logs but **not metrics**; Parallax is OTLP-native across all three. *(Real, current Sentry gap.)*
- **Self-host simplicity** — single-binary vs Sentry's ~20–40-container self-host stack. *(Real in design; Parallax pre-release. This is the explicit "simpler than self-hosted Sentry" wedge.)*
- **Openness / lock-in** — Apache-2.0 OTLP-native vs Sentry's FSL proprietary-format. *(Real.)*
- **Data sovereignty / air-gap** — Parallax designed for it; Sentry self-host possible but heavy, SaaS can't.
- **Fix-outcome loop + bounded/redacted agent bundle** — unoccupied cells. *(Thesis, **unproven** — A1 gate. Seer already covers much of context→fix from SaaS today.)*

## Open questions / what measurement would settle

- **A1 gate vs Seer:** does a Parallax evidence bundle beat Seer-as-context (or raw context) for agent fix quality, measurably? Unproven; Seer is the direct shipped competitor to the thesis.
- **Sentry OTLP metrics timeline:** metrics are now a first-class Sentry product (2026) but still not via OTLP — track whether OTLP metrics ships.
- **Self-host cost/ops:** a measured Parallax-single-binary vs Sentry-self-hosted deploy + RAM + ops comparison at parity. Benchmark-dependent, unmeasured.

## Sources (accessed 2026-07-17)

- [Sentry Pricing](https://sentry.io/pricing/) — authoritative live price page (all plan + overage numbers).
- [Sentry OTLP docs](https://docs.sentry.io/concepts/otlp/) — "traces and logs via OTLP; does not support OTLP metrics."
- [Sentry blog: Sentry vs OpenTelemetry](https://blog.sentry.io/sentry-opentelemetry-work-together/).
- [BusinessWire: Seer local-dev + code-review, $40/active contributor (2026-01)](https://www.businesswire.com/news/home/20260127739891/en/Sentry-Adds-Local-Development-and-Code-Review-Debugging-to-Seer); [sentry.io/product/seer](https://sentry.io/product/seer/); [Seer docs](https://docs.sentry.io/product/ai-in-sentry/seer/).
- Parallax side: [docs/research/00-vision/](../../00-vision/), [capture/sentry-ingest.md](../../capture/sentry-ingest.md), [validation/a1-bundle-value/](../../validation/a1-bundle-value/).
- Legacy internal note: [sentry-deep-research.md](../sentry-deep-research.md) (source, 2026-06).
