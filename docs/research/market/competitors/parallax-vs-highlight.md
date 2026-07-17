# Parallax vs Highlight.io

> One-to-one comparison. **No pro-Parallax bias.** Where Highlight is ahead,
> ahead is written. Where Parallax's edge is only *planned* or *unproven*, that
> is stated, not hidden.
>
> Research date: **2026-07-17** (trajectory resolved pass 33). Version, license,
> OTLP, pricing, and **company status** re-checked against live primary sources
> this pass. No legacy deep-research note exists for Highlight — this is the
> first canonical comparison.
>
> **🛑 HEADLINE (pass 33): Highlight.io was acquired by LaunchDarkly and its
> standalone SaaS shut down on 2026-02-28.** The OSS repo (`highlight/highlight`,
> Apache-2.0 self-host code) remains public but **unmaintained** — last release
> `docker-v0.5.6` (2025-08-08), last commit 2026-04-16 (LaunchDarkly-migration /
> workspace-allowlist work, not product features). Highlight is therefore **no
> longer an actively-shipping competitor**; its capabilities are described below
> as the **historical/reference** state of the wound-down product.

## TL;DR verdict (scoped per axis)

- **Session replay / RUM, error-monitoring maturity, OSS self-host on Apache-2.0,
  OTLP-native ingest, and the ClickHouse-backed full-stack surface: Highlight
  wins, plainly** over pre-release Parallax — especially session replay, which
  Parallax does not have at all.
- **Highlight is frontend/RUM-centric** (replay → error → why-bugs-happen);
  **Parallax is backend/incident-centric** (production errors → evidence for
  coding agents). Different centers of gravity; narrow overlap on errors + OTLP
  + OSS-self-host.
- **🛑 Trajectory RESOLVED (pass 33):** Highlight.io **was acquired by
  LaunchDarkly**; the **standalone Highlight service shut down 2026-02-28** and is
  being folded into **LaunchDarkly Observability** ("Guarded Releases"). The OSS
  repo stays public (Apache-2.0 self-host code) but is **unmaintained** (no
  release since 2025-08-08; last commit 2026-04-16 = migration/allowlist work).
  Highlight is **no longer an actively-shipping competitor** — treat the
  capability comparison below as the **historical reference** state. Net effect on
  the field: the **OSS session-replay champion is vacated** (no active successor
  at Highlight's maturity); Parallax does **not** "win" this — the field thinned,
  it did not tilt toward Parallax.
- **Parallax's differentiated edges are all unproven (A1 gate):** backend
  production-incident evidence bundle + fix-outcome loop + redaction; single-
  binary local-first; GreptimeDB.

## Highlight.io — what it is (verified 2026-07-17)

Open-source **full-stack monitoring platform** (Highlight, `highlight.io`):
**session replay** (its defining feature), **error monitoring**, console logs,
network requests, distributed tracing, infrastructure monitoring. Positioned as
the open-source FullStory/Sentry-frontend peer; OTLP-native; Apache-2.0
self-host + Highlight Cloud.

| | Highlight.io | Source |
|---|---|---|
| **Repo** | `highlight/highlight`, **9,331 stars** (GitHub API, 2026-07-17) | [github.com/highlight/highlight](https://github.com/highlight/highlight) |
| **Latest release** | **`docker-v0.5.6`** (2025-08-08) — **no release since; repo unmaintained** | GitHub releases |
| **Last commit** | **2026-04-16** — LaunchDarkly-migration / workspace-allowlist work, not product features | GitHub API |
| **License** | **Apache-2.0** (self-host code; GitHub reports `NOASSERTION` due to mixed licensing) | repo + docs |
| **Language** | TypeScript (frontend/SDKs) + Go (backend); ClickHouse store | [clickhouse.com/blog/overview-of-highlightio](https://clickhouse.com/blog/overview-of-highlightio) |
| **Telemetry store** | **ClickHouse** | ClickHouse blog |
| **OTLP ingest** | ✅ **OTLP-native** — OTLP endpoints for traces + logs; native OTel error monitoring via SDKs; OpenTelemetry persistent session mapping | [OneUptime OTLP guide](https://oneuptime.com/blog/post/2026-02-06-otel-highlight-io-otlp-endpoints/view), [docs](https://www.highlight.io/docs/getting-started/native-opentelemetry/error-monitoring) |
| **Session replay** | ✅ **core, best-in-class OSS** *(historical)* | [highlight.io/session-replay](https://highlight.io/session-replay) |
| **Self-host** | ✅ Apache-2.0, Docker *(code remains; unmaintained)* | repo |
| **Pricing** | **standalone SaaS shut down 2026-02-28**; free tier + paid ~$150/mo were the *historical* tiers | [europeanpurpose review](https://europeanpurpose.com/tool/highlight-io) |
| **Company status** | 🛑 **Acquired by LaunchDarkly**; standalone service **shut down 2026-02-28** → LaunchDarkly Observability | [LaunchDarkly blog](https://launchdarkly.com/blog/welcome-highlight-to-launchdarkly/), [Bugsink](https://www.bugsink.com/a-self-hosted-alternative-to-highlight-io/) |

### Pricing (re-cited; verify live page)

| Tier | Price | Notes |
|---|---|---|
| **Free** | $0 | session replay + error monitoring included |
| **Paid** | from **~$150/mo** | usage-based |

> Parallax pricing: **no public number** (pre-release). Direct comparison
> **benchmark-dependent, unmeasured.**

## Axis-by-axis comparison

### Signal coverage

| Signal | Highlight (shipped) | Parallax (planned) | Who |
|---|---|---|---|
| Session replay / RUM | ✅ **best-in-class OSS** | ❌ | **Highlight** (Parallax has none) |
| Errors / exceptions | ✅ error monitoring (incl. OTel) | ✅ derived `error_event` + fingerprint (🏗) | **Highlight** (maturity) |
| Traces / distributed tracing | ✅ OTLP | ✅ OTLP traces (🏗) | tie |
| Logs | ✅ console + OTLP logs | ✅ OTLP logs (🏗) | **Highlight** (maturity) |
| Metrics / infra | ✅ infrastructure monitoring | ✅ OTLP metrics (🏗) | **Highlight** (maturity) |
| Frontend (browser) | ✅ core | ❌ | **Highlight** |
| Profiling | ❌ | ❌ | tie (neither) |
| LLM / agent spans | ❌ | 🟡 planned | **Parallax** (planned) |

**Verdict:** Highlight's coverage is frontend/RUM-led + full-stack, all shipped.
**On session replay + RUM, Highlight wins by default** (Parallax has none). On
backend production telemetry Parallax's *design* is comparable but unshipped.

### Ingestion & transport

- **Highlight: OTLP-native** — OTLP endpoints for traces + logs; native OTel
  error monitoring; OpenTelemetry persistent session mapping (replay ↔ trace).
  Plus proprietary browser/mobile SDKs for replay.
- **Parallax: OTLP-native (all signals) + planned Sentry-envelope.**

> **Both OTLP-native.** Highlight's edge: **replay↔trace correlation** (the
> persistent OTel session mapping) + a mature browser-SDK fleet. Parallax's edge:
> general multi-signal + planned Sentry path (not shipped).

### Storage architecture

Highlight: **ClickHouse** (battle-tested). Parallax: GreptimeDB native OTLP
tables + Turso. ClickHouse is more proven; GreptimeDB is **benchmark-dependent,
unproven** head-to-head.

### Query & correlation

Highlight: replay → error → network → log → trace drilldown (the "why did this
bug happen" loop). Strong frontend-RUM correlation. Parallax: evidence-graph +
bounded bundle (**unproven**, A1).

### Error tracking & workflow

Highlight: error monitoring + grouping + lifecycle (resolve/regress/assign) — a
real error-workflow product (frontend-anchored). Parallax: derived error events
+ fingerprint + (planned) outcome loop. **Highlight wins on shipped error
workflow**; Parallax's outcome loop is the unproven differentiator.

### Dashboards & visualization

Highlight: replay viewer + error console + traces/logs UI. Parallax: minimal V1.
**Highlight wins** within its domain.

### AI-native / agent-context story (Parallax's wedge — be most honest)

- **Highlight:** a **human monitoring platform** (replay, errors, traces for
  engineers debugging). No bounded/redacted agent-context projection, no
  fix-outcome loop, no AI autofix→PR surfaced.
- **Parallax's claim:** bounded, redacted, agent-safe evidence bundle for coding
  agents.

> **Honest verdict:** Highlight is **not** an agent-context engine — it's a human
> RUM/error platform. On the agent-context axis, the two barely overlap.
> Parallax's differentiation (bounded bundle + outcome loop) is **unproven
> (A1)**; but Highlight does not compete on that axis today, so there is little
> direct pressure here. Highlight does, however, occupy the **error-workflow +
> OTLP + Apache-self-host** ground adjacent to Parallax's error wedge.

### Architecture & deployment

Highlight: Apache-2.0 self-host (Docker, ClickHouse) **or** Highlight Cloud.
Parallax: single-binary self-host, Apache-2.0. Both open + self-hostable.
**Highlight shipped/mature; Parallax pre-release.** Parallax's single-binary
local-first is a (design) simplicity edge; Highlight's is a multi-service stack.

### Scalability & performance

Highlight: proven at moderate scale (9.3k★, Cloud customers); ClickHouse-backed.
Specific numbers vendor; not independently measured. Parallax:
**benchmark-dependent, unproven.**

### Security & compliance

Highlight Cloud: standard SaaS security. Self-host = your posture. Parallax:
SSO/RBAC/audit planned; redaction (A6) designed. Roughly even on paper (both
immature vs Datadog/Sentry); verify Highlight's current compliance certs.

### Openness, licensing & lock-in

- **Highlight: Apache-2.0** — genuinely open, self-hostable, OTLP-native. Low
  lock-in (standard formats). **Comparable to Parallax's Apache-2.0.**
- **Parallax: Apache-2.0**, OTLP-native.

> **Verdict:** on openness, **roughly tied** (both Apache-2.0, OTLP-native,
> self-hostable). No lock-in advantage either way. An honest draw.

### Pricing & economics

Highlight: free tier + paid from ~$150/mo; self-host free (Apache). Parallax:
**no public number**. Direct comparison **benchmark-dependent, unmeasured.**

## 🛑 Trajectory / company-status — RESOLVED (pass 33, 2026-07-17)

The pass-14 "verify alive" flag is now **answered: Highlight.io is wound down.**

- **Acquired by LaunchDarkly** ("Welcome Highlight to LaunchDarkly"; "future of
  Guarded Releases"). Standalone Highlight SaaS **shut down 2026-02-28**; users
  migrated into **LaunchDarkly Observability**.
- **OSS repo unmaintained:** last release `docker-v0.5.6` (2025-08-08); last
  commit 2026-04-16 = LaunchDarkly-migration + workspace-allowlist plumbing, not
  product features. Docs site frozen ("no longer being actively updated").
- **License:** self-host code remains Apache-2.0 (GitHub reports `NOASSERTION`
  from mixed licensing) — so a fork/self-host is still *possible*, but no one is
  building it.

**No-bias strategic read:** this is **not a Parallax win.** Highlight's shipped
capabilities (session replay, OTLP-native error workflow, replay↔trace
correlation, Apache-2.0 ClickHouse stack) were genuinely strong and the code
remains. The effect is field-level: the **active OSS session-replay / RUM
competitor at this maturity is vacated** — the next-best active OSS replay options
are weaker/narrower (PostHog replay, HyperDX replay). Parallax still has no
session replay and gains nothing shipped from Highlight's exit; the bar for
"best OSS replay" simply drops. Re-evaluate only if LaunchDarkly re-open-sources
an actively-maintained Highlight fork.

## Where Highlight plainly wins (no bias)

1. **Session replay / RUM** — best-in-class OSS; Parallax has none.
2. **Error-monitoring maturity** — shipped workflow (grouping/lifecycle).
3. **Replay↔trace↔log correlation** (OTel persistent session mapping).
4. **Apache-2.0 OSS** self-host + OTLP-native + ClickHouse.
5. **Full-stack coverage** (frontend + backend + infra), shipped.
6. **Free tier + ~$150/mo** transparent pricing.

## Where Parallax honestly edges Highlight

1. **Backend production-incident focus** — Parallax is evidence-for-coding-agents;
   Highlight is human-RUM/error. *(Different job; not a head-to-head Parallax win.)*
2. **Production error events + fix-outcome loop** — Highlight has neither as a
   managed artifact. *(Thesis, **unproven** — A1 gate.)*
3. **Bounded, redacted, agent-safe evidence bundle** — Highlight is a human
   dashboard, not an agent-context projection. *(Thesis, **unproven** — A1 gate.)*
4. **Single-binary local-first** — Highlight self-host is a multi-service stack.
   *(Design edge.)*
5. **LLM/agent-span ingestion** — Highlight has none; Parallax plans it. *(Planned.)*

## Watch triggers — re-evaluate Highlight if it:

- **LaunchDarkly re-open-sources an actively-maintained Highlight fork** (the only path back to "active competitor" status). Current state: wound down.
- Adds **AI autofix→PR** or a **bounded agent-context artifact** (in the LaunchDarkly Observability product, not Highlight).
- Adds **LLM/agent observability**.
- Adds a **fix-outcome loop**.

> All Highlight watch triggers are **dormant** while the standalone product is
> shut down; re-scan LaunchDarkly Observability's roadmap instead.

## Sources (checked 2026-07-17)

- [github.com/highlight/highlight](https://github.com/highlight/highlight) — **9,331★** (API); latest release `docker-v0.5.6` (2025-08-08); last commit 2026-04-16 (LD migration); license `NOASSERTION` (mixed, self-host code Apache-2.0).
- [highlight.io](https://www.highlight.io/); [session replay](https://highlight.io/session-replay).
- **[LaunchDarkly: Welcome Highlight to LaunchDarkly](https://launchdarkly.com/blog/welcome-highlight-to-launchdarkly/)** — acquisition + "Guarded Releases".
- **[Bugsink: self-hosted alternative to Highlight.io](https://www.bugsink.com/a-self-hosted-alternative-to-highlight-io/)** — confirms standalone service **shut down 2026-02-28**, folded into LaunchDarkly Observability.
- [Highlight docs welcome](https://www.highlight.io/docs/general/welcome) — "Highlight has been acquired by LaunchDarkly!"; docs no longer actively updated.
- [docs: native OTel error monitoring](https://www.highlight.io/docs/getting-started/native-opentelemetry/error-monitoring); [session replay overview](https://www.highlight.io/docs/general/product-features/session-replay/overview).
- [OneUptime — OTLP endpoints to Highlight (2026-02)](https://oneuptime.com/blog/post/2026-02-06-otel-highlight-io-otlp-endpoints/view); [ClickHouse — overview of highlight.io](https://clickhouse.com/blog/overview-of-highlightio).
- [europeanpurpose — pricing review 2026 (~$150/mo)](https://europeanpurpose.com/tool/highlight-io); [Better Stack vs Highlight 2026](https://betterstack.com/community/comparisons/better-stack-vs-highlight-io/).
- Parallax side: [00-vision/ai-native-observability.md](../../00-vision/ai-native-observability.md), [validation/a1-bundle-value/](../../validation/a1-bundle-value/).
