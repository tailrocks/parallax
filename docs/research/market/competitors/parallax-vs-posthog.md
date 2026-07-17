# Parallax vs PostHog

> An unbiased, one-to-one comparison. Research date: **2026-07-17**.
> Sources: [posthog.com](https://posthog.com/) + [pricing](https://posthog.com/pricing), [schematic 2026 pricing](https://schematichq.com/blog/posthog-pricing), [checkthat.ai 2026](https://checkthat.ai/brands/posthog/pricing), [OpenPanel OSS survey](https://openpanel.dev/articles/open-source-web-analytics).
>
> **Bottom line up front:** PostHog is the leading **open-source product-analytics
> platform** (product analytics, session replay, feature flags, experiments, surveys)
> that has added **LLM observability** — converging toward AI-obs. On **product
> analytics, session replay, the experiments/flags suite, OSS self-host, large
> community, and generous pricing, PostHog is far ahead of pre-release Parallax.**
> The honest framing: **the two barely overlap on core domain** — PostHog is
> product/user-behavior analytics; Parallax is production-incident evidence. The
> real overlap is **LLM/agent observability** (PostHog's newer feature) + both
> OSS-self-hostable. Parallax's honest edges there are **production-error + outcome
> semantics** and the *unproven* bounded agent bundle (A1).

## What each product is

- **PostHog** — the leading **open-source product-analytics platform**: product analytics (funnels, retention, paths), **session replay**, **feature flags**, A/B **experiments**, surveys, customer data platform, and (newer) **LLM observability** (tracing/evals for LLM apps). Self-hostable OSS **or** PostHog Cloud. Large OSS community. **License:** PostHog's own open-source license (self-hostable; **note PostHog moved from MIT to its own license terms with competitive-use clauses — verify exact current terms; not pure Apache/MIT**). **Generous usage-based pricing** (Product Analytics: 1M events/mo free, ~$0.00005/event).
- **Parallax** — open-source (Apache-2.0), Rust-first, self-hostable **execution-context engine**: OTLP-native ingest of traces/logs/metrics + CLI/agent traces, derives owned `error_event`s, fingerprints, correlates into a typed evidence graph, serves bounded/redacted evidence bundles to humans and coding agents. GreptimeDB + Turso. **Pre-release.**

Both OSS, self-hostable, with an LLM/agent-obs surface. But the **core domains differ**: PostHog = product/user-behavior analytics (frontend/product-team); Parallax = production-incident evidence (backend/SRE/coding-agent). The comparison is mostly "different jobs," with overlap at LLM-obs + OSS-self-host.

## Signal coverage — different domains, narrow overlap

| Signal | PostHog (shipped) | Parallax (planned/shipped) |
| --- | --- | --- |
| Product analytics (funnels/retention/paths) | ✅ **(the core)** | ❌ |
| Session replay (web/mobile) | ✅ | ❌ |
| Feature flags / experiments | ✅ | ❌ |
| Surveys / CDP | ✅ | ❌ |
| **LLM / agent observability** | ✅ (newer — tracing/evals) | ✅ (🏗) |
| Production app traces/logs/metrics (OTLP) | 🟡 (event-centric, not a prod-telemetry backend) | ✅🧪 OTLP-native (shipped, pre-release) |
| Errors / exceptions (production backend) | 🟡 (frontend errors via replay; not prod error events) | ✅ derived `error_event` + fingerprint (🧪 shipped) |
| Sentry envelope / DSN | ❌ | ✅ shipped |

**Verdict:** PostHog's coverage is deep on **product analytics + replay + flags/experiments** — a domain Parallax doesn't target. The only real overlap is **LLM/agent observability** + OSS-self-host. On coverage breadth within PostHog's domain, **PostHog wins decisively**; on production-telemetry/error semantics, Parallax targets a domain PostHog doesn't.

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

- **PostHog:** frontend/replay errors; **no production-backend error-issue lifecycle** (not its domain).
- **Parallax:** derived `error_event` + fingerprint + (planned) fix-outcome loop.

**Verdict:** **different domains.** Parallax targets production-backend errors PostHog doesn't cover.

## AI-native / agent-context story — the real overlap

- **PostHog's LLM observability:** a newer feature — tracing/evals for LLM apps, overlapping the Langfuse/Phoenix territory. Part of PostHog's product-analytics platform (correlate LLM app behavior with product metrics). **Not a bounded, read-only, redacted agent-context projection for production incidents.**
- **Parallax's claim:** bounded, redacted, agent-safe evidence bundle for coding agents (planned, A1 gate).

**Honest verdict:** PostHog's LLM-obs is a **product-analytics extension** (correlate LLM-app traces with user behavior), not an incident-context engine. Parallax's differentiated bounded-agent-bundle is **unproven (A1)** — and the two serve different primary jobs even at the LLM-obs overlap. On shipped LLM-obs maturity, PostHog (alongside Langfuse/Phoenix) leads pre-release Parallax.

## Architecture & deployment

- **PostHog:** **self-host OSS** (ClickHouse-backed, multi-component) **or** PostHog Cloud (managed). Large, mature self-host community.
- **Parallax:** single-binary self-host target, local-first, air-gap-capable, Apache-2.0.

**Verdict:** on **OSS-self-host maturity + community, PostHog wins** (large, proven). On single-binary Rust local-first, Parallax's target is a different ergonomics story. **License differs** — PostHog's own license (competitive-use clauses) vs Parallax Apache-2.0 (verify PostHog's exact current terms).

## Operational footprint / Scalability

- **PostHog:** proven at large event scale (major OSS product-analytics platform); self-host is multi-component (ClickHouse).
- **Parallax:** unproven; benchmark-dependent.

**Verdict:** on **proven-at-scale + self-host maturity, PostHog wins conclusively.**

## Security / compliance

- **PostHog:** SSO/SAML, RBAC, audit (Cloud/Enterprise); self-host = your posture. SOC2.
- **Parallax:** SSO/RBAC/audit planned; redaction (A6) designed.

**Verdict:** on **shipped security, PostHog wins.**

## Openness, licensing & vendor lock-in

- **PostHog:** **open-source, self-hostable** — but **PostHog's own license** (moved from MIT; competitive-use clauses for some features). **Verify exact current terms.** Moderate lock-in (event schema, product-analytics model). Self-host viable at scale.
- **Parallax:** **Apache-2.0**, fully open (OSI, no competitive-use clauses), OTLP-native, portable bundle.

**Verdict:** on **license permissiveness, Parallax (Apache-2.0) likely edges PostHog** (PostHog's competitive-use clauses are less permissive) — **but verify PostHog's current license before asserting.** Both self-hostable; PostHog's self-host is more mature.

## Pricing & economics — real numbers

PostHog pricing is **public** ([posthog.com/pricing](https://posthog.com/pricing), accessed 2026-07-17), **usage-based with generous free tiers**:

| Feature | Free tier | Price |
| --- | --- | --- |
| **Product Analytics** | **1M events/mo** | **$0.00005 / event** (~$50/1M; down to ~$0.0000090 at 250M+) |
| **Session Replay** | **5,000 recordings/mo** | **$0.005 / recording** (~$5/1K) |
| **Feature Flags** | — | $0.0001 / request |
| **Surveys** | — | $0.10 / response |

Sources: [schematic](https://schematichq.com/blog/posthog-pricing), [checkthat.ai](https://checkthat.ai/brands/posthog/pricing), [userpilot](https://userpilot.com/blog/posthog-pricing/). **Among the most generous free tiers + cheapest event pricing in the set.**

**Parallax pricing:** none public yet (pre-release); self-host = no per-event tax by design.

**Honest cost read:** PostHog's pricing is genuinely generous and cheap for product-analytics workloads — not a domain Parallax competes in. On LLM-obs specifically, PostHog's event pricing competes with Langfuse/Phoenix. Parallax's cost edge applies to its different (production-telemetry-evidence) domain.

## Where PostHog plainly wins

- **Product analytics + session replay + feature flags/experiments** (the full product-analytics suite — a domain Parallax doesn't target).
- **LLM observability** (newer; converging on AI-obs alongside Langfuse/Phoenix).
- OSS self-host maturity + large community + proven-at-scale.
- Generous pricing + SOC2.

## Where Parallax honestly edges PostHog

- **Domain fit** — Parallax is production-incident evidence; PostHog is product/user analytics. *(Different jobs; Parallax where PostHog doesn't play.)*
- **Production error events + fix-outcome loop** — PostHog has neither (not its domain). *(Real: error events shipped; fix-outcome planned/unproven, A1.)*
- **License permissiveness** — Apache-2.0 (likely) vs PostHog's own competitive-use license. *(Narrow; verify PostHog terms.)*
- **Sentry-envelope compatibility** — PostHog has none; Parallax ships it. *(Real.)*
- **Bounded, redacted, agent-safe evidence bundle** — PostHog has none. *(Thesis, unproven, A1.)*

> **Honest summary:** PostHog and Parallax **barely overlap on core domain** — PostHog is the leading OSS product-analytics + replay + flags platform (a job Parallax doesn't do); Parallax is production-incident evidence (a job PostHog doesn't do). The real overlap is **LLM/agent observability** + **OSS-self-host**. On shipped LLM-obs + OSS-self-host maturity + community, **PostHog leads** pre-release Parallax. Parallax's defensible delta is its **production-incident + agent-bundle** scope (where PostHog doesn't play) + likely **Apache-vs-PostHog-license** + **Sentry-envelope**. Don't frame these as direct competitors on the product-analytics axis — they aren't.

## Open questions / what measurement would settle

- **A1 gate:** for LLM/agent observability specifically, does a Parallax bundle add value beyond PostHog's LLM-obs (or Langfuse/Phoenix)? Unproven — and PostHog's LLM-obs + product-analytics correlation is a different angle.
- **PostHog exact license (2026)** — confirm current terms (competitive-use clauses?) vs Apache-2.0; this determines whether Parallax has a real license edge.
- **PostHog → production-backend expansion** — if PostHog adds production-error/OTLP-backend semantics, the domain gap narrows. Track.

## Sources (accessed 2026-07-17)

- [posthog.com](https://posthog.com/); [pricing](https://posthog.com/pricing).
- [schematic PostHog pricing 2026](https://schematichq.com/blog/posthog-pricing); [checkthat.ai 2026](https://checkthat.ai/brands/posthog/pricing); [userpilot](https://userpilot.com/blog/posthog-pricing/).
- [OpenPanel OSS analytics survey 2026](https://openpanel.dev/articles/open-source-web-analytics).
- Parallax side: [00-vision/ai-native-observability.md](../../00-vision/ai-native-observability.md), [validation/a1-bundle-value/](../../validation/a1-bundle-value/).
- Sibling AI-obs deep-dives: [parallax-vs-langfuse.md](parallax-vs-langfuse.md), [parallax-vs-arize-phoenix.md](parallax-vs-arize-phoenix.md), [parallax-vs-langsmith.md](parallax-vs-langsmith.md).
