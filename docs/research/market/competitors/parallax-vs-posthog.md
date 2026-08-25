# Parallax vs PostHog

> An unbiased, one-to-one comparison. Research date: **2026-07-17** (**pass 65** —
> live pricing re-verify: **Error Tracking + Logs + AI Observability free tiers**
> expanded). Sources: live [posthog.com/pricing](https://posthog.com/pricing),
> [schematic 2026](https://schematichq.com/blog/posthog-pricing), [checkthat.ai](https://checkthat.ai/brands/posthog/pricing), [OpenPanel OSS survey](https://openpanel.dev/articles/open-source-web-analytics).
>
> **Bottom line up front:** PostHog is the leading **open-source product-analytics
> platform** (product analytics, session replay, feature flags, experiments, surveys)
> that has added **LLM observability**, **Error Tracking**, and **Logs** — expanding
> toward full product-OS. On **product analytics, session replay, the experiments/flags
> suite, OSS self-host, large community (~36k★), and generous pricing, PostHog is far
> ahead of pre-release Parallax.** The honest framing: **core domains still differ** —
> PostHog remains product/user-behavior-first; Parallax is production-incident evidence.
> **Pass-65 no-bias update:** Error Tracking (100K free) + Logs (50 GB free) **narrow
> the domain gap slightly** (PostHog now ships exception + log surfaces on Cloud free
> tier) but **do not** make PostHog a production OTLP backend or Sentry-envelope peer.
> Residual Parallax edges = OTLP-native prod telemetry + Sentry envelope + outcome loop
> + bounded/redacted bundle (A1 unproven).

## What each product is

- **PostHog** — the leading **open-source product-analytics platform**: product analytics (funnels, retention, paths), **session replay**, **feature flags**, A/B **experiments**, surveys, CDP, **LLM / AI Observability**, **Error Tracking**, and **Logs**. Self-host OSS **or** Cloud. **~36,093★** (pass 65, 2026-07-17). **License:** **MIT Expat core** + **proprietary `ee/`** ([`ee/LICENSE`](https://github.com/PostHog/posthog/blob/master/ee/LICENSE) — production use requires PostHog Enterprise subscription / seats; dev/test free). Pure-FOSS strip = [`posthog-foss`](https://github.com/PostHog/posthog-foss). **Generous usage pricing** with monthly free tiers across **10+ products** (analytics 1M events, Error Tracking 100K exceptions, Logs 50 GB, AI Observability 100K events, …).
- **Parallax** — open-source (Apache-2.0), Rust-first, self-hostable **execution-context engine**: OTLP-native ingest of traces/logs/metrics + CLI/agent traces, derives owned `error_event`s, fingerprints, correlates into a typed evidence graph, serves bounded/redacted evidence bundles to humans and coding agents. GreptimeDB + Turso. **Pre-release.**

Both OSS, self-hostable, with an LLM/agent-obs surface. But the **core domains differ**: PostHog = product/user-behavior analytics (frontend/product-team); Parallax = production-incident evidence (backend/SRE/coding-agent). The comparison is mostly "different jobs," with overlap at LLM-obs + OSS-self-host.

## Signal coverage — different domains, narrow overlap

| Signal | PostHog (shipped) | Parallax (planned/shipped) |
| --- | --- | --- |
| Product analytics (funnels/retention/paths) | ✅ **(the core)** | ❌ |
| Session replay (web/mobile) | ✅ | ❌ |
| Feature flags / experiments | ✅ | ❌ |
| Surveys / CDP | ✅ | ❌ |
| **LLM / agent observability** | ✅ **AI Observability** (Cloud free 100K events/mo) | 🟡🧪 (agent-session modules; CLI program in flight) |
| **Error Tracking** (exceptions) | ✅ (Cloud free **100K exceptions/mo**; usage after) | ✅ derived `error_event` + fingerprint (🧪 shipped) |
| **Logs** | ✅ (Cloud free **50 GB/mo** ingest; usage after) | ✅🧪 OTLP logs (shipped, pre-release) |
| Production app traces/metrics (OTLP) | 🟡 (event-centric; **not** a full OTLP prod-telemetry backend) | ✅🧪 OTLP-native (shipped, pre-release) |
| Sentry envelope / DSN | ❌ | ✅ shipped |

**Verdict (pass 65):** PostHog remains deep on **product analytics + replay + flags/experiments**. **Error Tracking + Logs free tiers** expand overlap with incident/error workflows (no-bias: PostHog is **closer** than pass-22 framing). Still **not** an OTLP-native production backend or Sentry-envelope peer. On product-analytics domain, **PostHog wins decisively**; on OTLP/Sentry/prod-evidence semantics, Parallax's design targets layers PostHog does not own.

## Ingestion & transport

- **PostHog:** product-event SDKs (web/mobile/server) + replay SDK + LLM-obs SDK; **OpenTelemetry-compatible** for some paths. Event-centric ingest, not a general OTLP-telemetry backend.
- **Parallax:** OTLP gateway (logs/metrics/traces native) + shipped Sentry-envelope adapter.

**Verdict:** on product-event capture + replay, **PostHog wins.** On OTLP-native production telemetry + Sentry-envelope, **Parallax's design is broader** (different domain).

## Storage architecture

- **PostHog:** self-hosted OSS (ClickHouse-backed; event/columnar) or PostHog Cloud. Proven at large event volume.
- **Parallax:** GreptimeDB (native OTLP tables) + Turso, single-binary self-host target.

**Verdict:** on **event-analytics scale + OSS self-host maturity, PostHog wins.** On single-binary + telemetry-native, Parallax's target differs. Both ClickHouse-vs-GreptimeDB-adjacent (benchmark-dependent).

## Query & correlation

- **PostHog:** product-analytics query (funnels, retention, paths, cohorts) + replay + flag/experiment correlation. Mature for product analysis.
- **Parallax:** evidence-graph correlation + bounded bundle for agents.

**Verdict:** different domains — **PostHog wins in product analytics**, Parallax in (unproven) agent-incident context. Not head-to-head.

## Error tracking & workflow

- **PostHog (pass 65):** dedicated **Error Tracking** product on Cloud — free **100K exceptions/mo**, then from **$0.00037/exception** (volume tiers down to ~$0.000115). Combined with session replay for frontend/product errors. Still **not** a Sentry-envelope backend or OTLP-derived production `error_event` store.
- **Parallax:** derived `error_event` + fingerprint (**shipped**) + fix-outcome offline residual (**plan 123 DONE**; live value **unproven**).

**Verdict:** domains **closer than pass 22** (PostHog now prices Error Tracking as a first-class product). Parallax residual edges = **Sentry-envelope + OTLP-derived fingerprint + outcome loop** (latter unproven), not “PostHog has no error product.”

## AI-native / agent-context story — the real overlap

- **PostHog (pass 65):** **AI Observability** free **100K events/mo** (LLM/agent tracing-style surface) + **PostHog AI** free **500 credits** (~$5) then **$0.01/credit**. Overlaps Langfuse/Phoenix territory; product-analytics correlation remains the primary frame. **Not** a bounded, read-only, redacted portable agent-context projection for production incidents.
- **Parallax's claim:** bounded, redacted, agent-use (safety/value unproven) evidence bundle for coding agents (**code-shipped**, A1 value unproven gate).

**Honest verdict:** PostHog ships **AI Observability + PostHog AI** on the free tier — **ahead of pre-release Parallax** on shipped LLM/agent product surface. Parallax's differentiated claim remains the **bounded/redacted/portable prod-incident bundle + outcome loop (A1 unproven)**, not “AI/LLM obs exists.”

## Architecture & deployment

- **PostHog:** **self-host OSS** (ClickHouse-backed, multi-component) **or** PostHog Cloud (managed). Large, mature self-host community.
- **Parallax:** single-binary self-host target, local-first, offline/local deployment target (air-gap unverified), Apache-2.0.

**Verdict:** on **OSS-self-host maturity + community, PostHog wins** (large, proven). On single-binary Rust local-first, Parallax's target is a different ergonomics story. **License:** both permissive-core (PostHog **MIT Expat** + proprietary `ee/`; Parallax **uniform Apache-2.0**, no `ee/`). Parallax’s edge is **uniform OSI openness** (no proprietary EE directory), not “competitive-use clauses” (those do not apply to PostHog core).

## Operational footprint / Scalability

- **PostHog:** proven at large event scale (major OSS product-analytics platform); self-host is multi-component (ClickHouse).
- **Parallax:** unproven; benchmark-dependent.

**Verdict:** on **proven-at-scale + self-host maturity, PostHog wins conclusively.**

## Security / compliance

- **PostHog:** SSO/SAML, RBAC, audit (Cloud/Enterprise); self-host = your posture. SOC2.
- **Parallax:** SSO/RBAC/audit planned; redaction (A6) designed.

**Verdict:** on **shipped security, PostHog wins.**

## Openness, licensing & vendor lock-in

- **PostHog:** **MIT Expat core** + **proprietary `ee/`** (live [LICENSE](https://github.com/PostHog/posthog/blob/master/LICENSE), 2026-07-17). Self-hostable; pure-FOSS via `posthog-foss`. Moderate product lock-in (event schema). Self-host mature at scale.
- **Parallax:** **Apache-2.0**, fully open (no proprietary `ee/` directory), OTLP-native, portable bundle.

**Verdict:** on **uniform OSI openness, Parallax edges** (no paid-feature EE carve-out). On **core permissiveness, both are permissive OSS** (MIT vs Apache-2.0 — **not** a competitive-use fight). PostHog self-host is more mature.

## Pricing & economics — real numbers (pass 65 live re-verify)

PostHog pricing is **public** ([posthog.com/pricing](https://posthog.com/pricing), accessed 2026-07-17 pass 65), **usage-based with generous free tiers on all plans** (resets monthly; free plan = 1 project / 1-year retention; pay-as-you-go = 6 projects / 7-year):

| Product | Free tier / mo | After free (from) |
| --- | --- | --- |
| **Product Analytics** | **1M events** | **$0.00005 / event** (→ ~$0.0000090 at 250M+) |
| **Session Replay** | **5K** recordings (2.5K mobile free) | **$0.005 / recording** (mobile from $0.01) |
| **Feature Flags** | **1M requests** | **$0.0001 / request** |
| **Surveys** | **1,500 responses** | **$0.10 / response** |
| **Error Tracking** | **100K exceptions** | **$0.00037 / exception** (→ ~$0.000115 at 10M+) |
| **Logs** | **50 GB** ingested | **$0.25 / GB** (→ $0.15 at 300+ GB) |
| **AI Observability** | **100K events** | (usage after free; billed as AI Obs product) |
| **PostHog AI** | **500 credits** (~$5) | **$0.01 / credit** |
| **Data warehouse** | **1M rows** + free historical | from **$0.000015 / row** |
| **Data pipelines** | **10K** trigger events + 1M batch rows | realtime from **$0.0005 / event** |
| **Workflows** | **10K** messages/channel | email from **$0.003 / email** |
| **Inbox (beta)** | **3 PRs** | **$15 / PR** |

Identified events can bill higher (~**$0.000248 / event** after free 1M) when person profiles attach. Platform packages (Boost **$250**/mo, Scale **$750**/mo, Enterprise contact) add support/RBAC/SSO enforcement. **No storage fee** for retained events beyond product free tiers (per pricing FAQ framing).

**Among the most generous free tiers + transparent multi-product usage pricing in the set.**

**Parallax pricing:** none public yet (pre-release); self-host = no per-event tax by design.

**Honest cost read (pass 65):** free **Error Tracking + Logs + AI Observability** make PostHog a **cheap default** for product teams that also want light error/log/LLM surfaces — stronger competitive gravity than pass-22 “analytics-only.” Still not a like-for-like vs Parallax self-host prod-telemetry. Parallax’s cost edge remains self-host no-metering in its own domain (unproven at scale).

## Where PostHog plainly wins

- **Product analytics + session replay + feature flags/experiments** (the full product-analytics suite — a domain Parallax doesn't target).
- **AI Observability + PostHog AI free tier** (100K events + 500 credits) — shipped LLM/agent surface ahead of pre-release Parallax.
- **Error Tracking + Logs free tiers** (100K exceptions + 50 GB) — broader free product OS than pass-22 framing.
- OSS self-host maturity + large community (**~36,093★**) + proven-at-scale.
- Generous multi-product usage pricing + SOC2.

## Where Parallax honestly edges PostHog

- **OTLP-native production telemetry backend** — PostHog is still event/product-centric, not a GreptimeDB-class OTLP store. *(Design edge; Parallax pre-release.)*
- **Sentry-envelope compatibility** — PostHog has none; Parallax ships it. *(Real.)*
- **Fix-outcome loop + derived fingerprint semantics** — PostHog Error Tracking ≠ Parallax `error_event` + outcome residual. *(Partial: errors now on both sides; outcome still Parallax-only and **unproven**.)*
- **Uniform OSI openness** — Apache-2.0 with no proprietary `ee/` vs PostHog MIT core + proprietary `ee/`. *(Narrow.)*
- **Bounded, redacted, agent-use (safety/value unproven) evidence bundle** — PostHog has none. *(Thesis, unproven, A1.)*

> **Honest summary (pass 65):** PostHog and Parallax **still differ on primary job** (product analytics OS vs production-incident evidence), but **overlap grew**: Error Tracking, Logs, and AI Observability free tiers make PostHog a stronger adjacent alternative for product teams. On shipped multi-product breadth + community + free-tier economics, **PostHog leads**. Parallax residual delta = **OTLP-native prod backend + Sentry-envelope + outcome loop + portable redacted bundle** (A1 unproven) — **not** “PostHog has no errors/logs/AI.”

## Open questions / what measurement would settle

- **A1 gate:** does a Parallax bundle add value beyond PostHog AI Observability + Error Tracking for coding-agent incident fixes? Unproven.
- ~~PostHog exact license~~ → **RESOLVED pass 41: MIT Expat core + proprietary `ee/`**.
- ~~Which features are EE-only~~ → **pass 57 map (repo + Cloud):** proprietary `ee/` top-level dirs include **`hogai`** (AI assistant / MCP tooling), **`billing`**, **`session_recordings`**, **`surveys`**, **`clickhouse`** (EE ClickHouse paths), **`vercel`**, **`support_sidebar_max`**, **`admin`**, plus EE `api`/`models`/`migrations`. Cloud **Enterprise add-on** (commonly cited **~$2,000/mo** + usage on live pricing/secondary 2026 teardowns) gates compliance/support/RBAC/activity logs — **not identical** to `ee/` file map (some product surface ships in main tree but Cloud-gated). Self-hosters wanting zero proprietary code use **posthog-foss**.
- ~~Free-tier surface (analytics-only? )~~ → **pass 65 RESOLVED:** free tier includes **Error Tracking 100K**, **Logs 50 GB**, **AI Observability 100K**, **PostHog AI 500 credits**, plus analytics/replay/flags/… — multi-product free OS.
- **PostHog → OTLP production backend / Sentry-envelope** — if either lands, domain gap collapses further. **Watch UNFIRED** (still product-event-centric + no Sentry DSN).

## Sources (accessed 2026-07-17; pass 65 live pricing)

- [posthog.com](https://posthog.com/); **[pricing](https://posthog.com/pricing)** (pass 65 free-tier table); [contribute / licensing](https://posthog.com/docs/contribute).
- [PostHog LICENSE (MIT Expat core + ee/ carve-out)](https://github.com/PostHog/posthog/blob/master/LICENSE) — **~36,093★** (GitHub API pass 65).
- [schematic PostHog pricing 2026](https://schematichq.com/blog/posthog-pricing); [checkthat.ai 2026](https://checkthat.ai/brands/posthog/pricing) — secondary; live pricing is primary.
- [OpenPanel OSS analytics survey 2026](https://openpanel.dev/articles/open-source-web-analytics).
- Parallax side: [00-vision/ai-native-observability.md](../../00-vision/ai-native-observability.md), [validation/a1-bundle-value/](../../validation/a1-bundle-value/).
- Sibling AI-obs deep-dives: [parallax-vs-langfuse.md](parallax-vs-langfuse.md), [parallax-vs-arize-phoenix.md](parallax-vs-arize-phoenix.md), [parallax-vs-langsmith.md](parallax-vs-langsmith.md).
