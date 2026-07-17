# Parallax vs Helicone

> An unbiased, one-to-one comparison. Research date: **2026-07-17** (**pass 42
> trajectory + pricing RESOLVED**). Sources: live [helicone.ai/pricing](https://www.helicone.ai/pricing),
> [Helicone joins Mintlify (2026-03-03)](https://www.helicone.ai/blog/joining-mintlify),
> [github.com/Helicone/helicone](https://github.com/Helicone/helicone) (**5,956★**,
> license **Apache-2.0** per GitHub API; last push 2026-07-05; latest release
> **v2025.08.21-1** 2025-08-21).
>
> **Bottom line up front:** Helicone was a strong **OSS LLM gateway/proxy +
> observability** product (caching, cost analytics, zero LLM-cost markup). **Pass-42
> material trajectory:** **acquired by Mintlify (2026-03-03)**; vendor states services
> remain live in **maintenance mode** (security updates, new models, bug/perf fixes —
> not active feature expansion). On LLM-call proxy/monitoring Helicone remains a
> historical/reference specialist; **it is no longer an actively-shipping competitor
> on a growth roadmap.** Domain overlap with Parallax was always narrow; the acquisition
> **vacates** active LLM-gateway competition, not a Parallax win (Parallax still is not
> an LLM gateway). **License corrected:** GitHub reports **Apache-2.0** (not MIT as
> pass-23 claimed — still permissive OSS).

## What each product is

- **Helicone** (`Helicone/helicone`) — **LLM gateway/proxy + observability**: sits between app and LLM providers; captures prompt/completion/latency/tokens/cost; **caching**, cost analytics, OTel support. **~5,956★** (pass 42; earlier “~12k” was overstated/wrong snapshot). **Apache-2.0** OSS self-host + Cloud. **Acquired by Mintlify 2026-03-03** → **maintenance mode** (services live; not feature-forward). Historical scale claim (vendor blog): 14.2T tokens, 16k orgs.
- **Parallax** — open-source (Apache-2.0), Rust-first, self-hostable **execution-context engine**: OTLP-native traces/logs/metrics + CLI/agent traces, derives `error_event`s, fingerprints, evidence graph, bounded/redacted bundles. GreptimeDB + Turso. **Pre-release.**

**Framing:** different domains (LLM-call proxy vs production-incident evidence). Trajectory now matters more than feature parity: Helicone is **maintenance-mode under Mintlify**, not a building competitor.

## Signal coverage

| Signal | Helicone (shipped / maintained) | Parallax (planned/shipped) |
| --- | --- | --- |
| LLM calls (prompt/completion/tokens/cost) | ✅ **(core — via proxy)** | 🟡🧪 (in code / program) |
| LLM caching | ✅ | ❌ |
| LLM cost analytics | ✅ | ❌ |
| Production app traces/logs/metrics (OTLP) | ❌ (LLM-call-only) | ✅🧪 OTLP-native (shipped, pre-release) |
| Errors / exceptions (production backend) | ❌ | ✅ derived `error_event` + fingerprint (🧪 shipped) |
| Sentry envelope / DSN | ❌ | ✅ shipped |

**Verdict:** Helicone remains **deep but narrow** on LLM-call monitoring. Parallax broader on production telemetry. **Different domains.**

## Ingestion & transport

- **Helicone:** proxy/gateway — point app at Helicone; forwards to provider and logs. OTel support.
- **Parallax:** OTLP gateway + shipped Sentry-envelope adapter.

**Verdict:** on LLM-call proxy/caching, Helicone (historical) wins. On OTLP + Sentry, Parallax’s design is broader (different domain).

## Storage / query / AI-native

- Helicone: LLM-call log store (Cloud or self-host); dashboards for cost/latency/cache.
- Parallax: GreptimeDB + Turso; evidence graph + (unproven A1) bounded bundle.

**Honest AI verdict:** Helicone is FinOps/debug for LLM spend — **not** a bounded prod-incident agent context. Parallax bundle **unproven (A1)**.

## Architecture & deployment / trajectory

- **Helicone Cloud:** live under **Mintlify**, **maintenance mode** ([joining Mintlify](https://www.helicone.ai/blog/joining-mintlify)).
- **Helicone OSS:** Apache-2.0 repo public; last release **2025-08-21**; last push **2026-07-05** (maintenance-level activity).
- **Parallax:** single-binary self-host target, Apache-2.0, pre-release.

**No-bias trajectory note (same pattern as Highlight pass 33):** an acquisition + maintenance mode is **not** a Parallax product win. Helicone’s shipped gateway stack was strong; code remains forkable. Effect = **field thins** on active OSS LLM-gateway productization. Parallax still has **no** LLM gateway.

## Openness, licensing & vendor lock-in

- **Helicone:** **Apache-2.0** (GitHub API 2026-07-17) + Cloud. Self-host viable. Proxy is replaceable.
- **Parallax:** Apache-2.0, OTLP-native, portable bundle.

**Verdict:** **tied on license family** (both Apache-2.0). Cloud path now **Mintlify-coupled**.

## Pricing & economics — RESOLVED pass 42 (live helicone.ai/pricing)

| Plan | Price | Requests | Retention | Notes |
| --- | --- | --- | --- | --- |
| **OSS self-host** | **$0** | unlimited (your infra) | yours | Apache-2.0 |
| **Hobby (Cloud)** | **Free** | **10,000/mo** | **7 days** | 1 seat, 1 org, 1 GB storage, 10 logs/min |
| **Pro** | **$79/mo** + usage | 10K free then usage-based | **1 month** | unlimited seats; 1 org; 1 GB free storage then usage; 1k logs/min |
| **Team** | **$799/mo** + usage | 10K free then usage-based | **3 months** | 5 orgs; 15k logs/min |
| **Enterprise** | custom | 10K free then usage | **Forever** | unlimited orgs; 30k logs/min; HIPAA/SOC2/SAML |

**Zero markup on LLM provider spend** remains the pitch (pay platform tier, not % of API). **Discounts:** startups 50% first year; OSS companies $100 credit; students free.

**Parallax pricing:** **no public number** (pre-release).

**Honest cost read:** Cloud pricing is public and generous at Hobby; not a domain Parallax competes in as a gateway. Maintenance-mode status may change long-term Cloud reliability calculus for buyers.

## Where Helicone plainly wins (historical / maintained product)

- LLM-call proxy/gateway + caching + cost analytics + zero LLM-markup.
- Public Cloud pricing + OSS self-host (Apache-2.0).
- Proven LLM-call volume (vendor historical metrics).

## Where Parallax honestly edges Helicone

- **Production-backend telemetry breadth** (OTLP + Sentry). *(Real domain difference.)*
- **Error events + outcome + bounded bundle** (A1 unproven).
- **Active product development** — Parallax is building; Helicone is maintenance-mode under Mintlify. *(Trajectory fact — not capability superiority on LLM-gateway axes Helicone already shipped.)*

> **Honest summary:** Helicone was (and as maintained software remains) a specialist LLM gateway/proxy. Parallax is a production-incident context engine. They barely overlap. **Pass 42:** Mintlify acquisition + maintenance mode means Helicone is **not an actively expanding competitor**; do **not** spin that as a Parallax win. Closest active peers for LLM-obs remain Langfuse / Phoenix / LangSmith / Axiom AI Engineering.

## Watch triggers

- Mintlify **sunsets Cloud** or stops OSS commits → treat as wound-down (Highlight pattern).
- Mintlify **re-opens feature development** or folds Helicone into docs/agent product → re-score.
- License change under Mintlify.

## Open questions / what measurement would settle

- **Domain overlap** — moot while maintenance-mode and no prod-backend expansion.
- A1 vs Helicone — low priority (different domain).

## Sources (accessed 2026-07-17; pass 42)

- [helicone.ai/pricing](https://www.helicone.ai/pricing) (live tiers).
- [Helicone is joining Mintlify (2026-03-03)](https://www.helicone.ai/blog/joining-mintlify).
- [github.com/Helicone/helicone](https://github.com/Helicone/helicone) — 5,956★, Apache-2.0, last push 2026-07-05, latest release v2025.08.21-1.
- Sibling: [parallax-vs-langfuse.md](parallax-vs-langfuse.md), [parallax-vs-traceloop.md](parallax-vs-traceloop.md).
