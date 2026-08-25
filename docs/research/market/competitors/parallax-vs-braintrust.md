# Parallax vs Braintrust

> An unbiased, one-to-one comparison. Research date: **2026-07-17**.
> Sources: live [braintrust.dev/pricing](https://www.braintrust.dev/pricing) (**pass 44
> RESOLVED; pass 61 re-confirm**), [braintrust.dev](https://www.braintrust.dev/),
> [playgrounds-vs-experiments](https://www.braintrust.dev/foundations/playgrounds-vs-experiments).
>
> **Bottom line up front:** Braintrust is the **eval-first LLM observability** platform
> — the specialist on **LLM evaluations + experiments** (datasets, scorers, playgrounds,
> cost-tracked experiments, AI Loop). On **LLM eval/experiment tooling and the eval-driven
> dev loop, Braintrust is far ahead of pre-release Parallax.** The two barely overlap:
> Braintrust = LLM-app eval/experimentation; Parallax = production-incident evidence.
> Parallax's honest edges are **production-backend telemetry breadth** and the *unproven*
> bounded agent bundle (A1) — which is itself an eval question Braintrust's tooling
> could help answer.

## What each product is

- **Braintrust** (braintrust.dev) — an **eval-first AI evaluation + observability platform**: **experiments** (`Eval()` against datasets), **scorers** (LLM/code/human), **datasets**, **playgrounds** vs **experiments**, production tracing/monitoring, cost tracking, and **Loop agent** (built-in AI that runs evals, generates test cases, iterates prompts autonomously). **"Eval-driven development"** specialist. **OSS SDK**; core platform SaaS (Enterprise: on-prem or hosted). Live pricing (pass 44): Starter **$0** / Pro **$249/mo** / Enterprise custom.
- **Parallax** — open-source (Apache-2.0), Rust-first, self-hostable **execution-context engine**: OTLP-native ingest of traces/logs/metrics + CLI/agent traces, derives owned `error_event`s, fingerprints, correlates into a typed evidence graph, serves bounded/redacted evidence bundles to humans and coding agents. GreptimeDB + Turso. **Pre-release.**

Both touch "evaluation" of AI behavior, but **Braintrust = LLM-app eval/experimentation** (improve your LLM app); **Parallax = production-incident evidence for coding agents**. Narrow overlap.

## Signal coverage

| Signal | Braintrust (shipped) | Parallax (planned/shipped) |
| --- | --- | --- |
| **LLM evals (scorers, LLM/code/human)** | ✅ **(the core — eval-first)** | ✅ planned (A1 eval design) |
| **Experiments (Eval() vs datasets)** | ✅ **(distinctive)** | ❌ |
| Datasets / prompt playground | ✅ | ❌ |
| Production LLM tracing/monitoring | ✅ | ✅ (🏗) |
| Cost tracking (integrated w/ experiments) | ✅ | ❌ |
| Production app traces/logs/metrics (OTLP) | 🟡 (LLM-trace-centric) | ✅🧪 OTLP-native (shipped, pre-release) |
| Errors / exceptions (production backend) | ❌ | ✅ derived `error_event` + fingerprint (🧪 shipped) |
| Sentry envelope / DSN | ❌ | ✅ shipped |

**Verdict:** Braintrust is **deep on LLM eval/experiment** — a domain Parallax doesn't target (except conceptually: Parallax's A1 gate is an eval question). On eval/experiment tooling, **Braintrust wins decisively.** On production-telemetry/error semantics, Parallax targets a domain Braintrust doesn't.

## Ingestion & transport

- **Braintrust:** SDK-based (Python/JS) capture of LLM calls + eval runs + experiments; tracing via SDK. Not a general OTLP-telemetry backend.
- **Parallax:** OTLP gateway (logs/metrics/traces native) + shipped Sentry-envelope adapter.

**Verdict:** on **LLM-call/eval capture, Braintrust wins** (its domain). On OTLP-native telemetry + Sentry-envelope, **Parallax's design is broader** (different domain).

## Storage architecture

- **Braintrust:** proprietary SaaS backend (datasets/experiments/traces); SDK is OSS, core is not. Internals not public.
- **Parallax:** GreptimeDB (native OTLP tables) + Turso, self-host single-binary.

**Verdict:** on **self-host + open storage, Parallax wins by design.** On eval/experiment-scale + maturity, Braintrust wins (its niche).

## Query & correlation

- **Braintrust:** experiment comparison (side-by-side prompt/model variants), dataset drill, scorer breakdowns, cost-per-experiment. Mature for the eval workflow.
- **Parallax:** evidence-graph correlation + bounded bundle for agents.

**Verdict:** different domains — **Braintrust wins in eval/experiment analytics**, Parallax in (unproven) agent-incident context.

## Error tracking & workflow

- **Braintrust:** eval-failure detection (not production-backend error events); **no production error-issue lifecycle.**
- **Parallax:** derived `error_event` + fingerprint (**shipped**) + fix-outcome offline residual (**plan 123 DONE**; live value **unproven**).

**Verdict:** **different domains.** Parallax targets production-backend errors Braintrust doesn't cover.

## AI-native / agent-context story

- **Braintrust's position:** an **LLM-app eval/experiment tool** (improve your LLM app via evals/experiments) + **Loop** (AI). A human dev-eval tool; **not a bounded, read-only, redacted agent-context projection for production incidents.**
- **Parallax's claim:** bounded, redacted, agent-use (safety/value unproven) evidence bundle for coding agents (**code-shipped**, A1 value unproven gate).

**Honest verdict:** Braintrust and Parallax serve different jobs even at the "AI evaluation" overlap — Braintrust = LLM-app eval/experimentation; Parallax = production-incident agent context. Parallax's differentiated bundle is **unproven (A1)**. Note: Parallax's **A1 gate (does a bundle beat raw context for agent fix outcomes?) is itself an eval question — Braintrust's eval tooling is exactly the kind of harness that could measure it.** (Useful cross-reference, not a competitive overlap.)

## Architecture & deployment

- **Braintrust:** **SaaS** (braintrust.dev); OSS SDK only — core platform is not self-hostable OSS.
- **Parallax:** single-binary self-host target, local-first, offline/local deployment target (air-gap unverified), Apache-2.0.

**Verdict:** on **self-host / openness, Parallax wins by design** (Braintrust core is SaaS; only the SDK is OSS).

## Scalability / Security / compliance

- **Braintrust:** proven for eval/experiment scale (enterprise AI teams); SSO/RBAC; SOC2 posture (verify).
- **Parallax:** unproven at scale; SSO/RBAC/audit planned; redaction (A6) designed.

**Verdict:** on **shipped maturity, Braintrust wins.**

## Openness, licensing & vendor lock-in

- **Braintrust:** **OSS SDK, closed core platform** (SaaS). Moderate-to-high lock-in (datasets/experiments/scorers live in Braintrust).
- **Parallax:** Apache-2.0, fully open, OTLP-native, portable bundle.

**Verdict:** on **openness and lock-in cost, Parallax wins** (Apache OSS core vs Braintrust's closed core + OSS-SDK-only).

## Pricing & economics — real numbers

| Plan | Price | Notes |
| --- | --- | --- |
| **Starter** | **$0/mo** | $10 credits + tok rates; **1 GB** processed then **$4/GB**; **10k scores** then **$2.50/1k**; **14-day** retention; unlimited users/projects/datasets/playgrounds/experiments |
| **Pro** | **$249/mo** | plan card shows **$100→$249 credits** marketing; Topics table **$249 credits/mo** then tok rates; **5 GB** processed then **$3/GB**; **50k scores** then **$1.50/1k**; **30-day** retention then **$0.50/GB/mo**; custom charts, environments, priority support, basic RBAC; Loop agent |
| **Enterprise** | custom | |

Source: live [braintrust.dev/pricing](https://www.braintrust.dev/pricing) (2026-07-17). Topics metering: **$0.06/mtok input, $0.40/mtok output** after included credits. Enterprise: custom retention/export, SAML SSO, on-prem or hosted, BAA/HIPAA path.

**Parallax pricing:** **no public number** (pre-release).

**Honest cost read:** Braintrust Pro **$249/mo** is a real paid eval platform (vs Langfuse/Phoenix free self-host). Not a domain Parallax competes in — except Braintrust-class tooling can **measure** Parallax A1.

## Where Braintrust plainly wins

- **LLM eval/experiment tooling** (datasets, scorers, `Eval()` experiments, playground — the eval-first specialist, strongest in the AI-obs set on this axis).
- Cost tracking integrated with experiments + AI Loop.
- Eval-driven-development workflow maturity + proven at AI-team scale.

## Where Parallax honestly edges Braintrust

- **Production-backend telemetry breadth** — OTLP-native logs/metrics/traces; Braintrust is LLM-eval/trace-centric. *(Real domain difference.)*
- **Production error events + fix-outcome loop** — Braintrust has neither. *(Real: error events **shipped**; fix-outcome offline residual plan 123 DONE; live value **unproven**.)*
- **Openness** — Apache-2.0 OSS core vs Braintrust's closed core (OSS SDK only). *(Real.)*
- **Sentry-envelope compatibility** — Braintrust has none; Parallax ships it. *(Real.)*
- **Bounded, redacted, agent-use (safety/value unproven) evidence bundle** — Braintrust has none. *(Thesis, unproven, A1 — and Braintrust's eval tooling could help *measure* the A1 gate.)*

> **Honest summary:** Braintrust and Parallax **barely overlap** — Braintrust is the **eval-first LLM eval/experiment specialist** (datasets/scorers/experiments/playground, the strongest on that axis); Parallax is a production-incident evidence engine. On eval/experiment tooling Braintrust is far ahead and Parallax doesn't compete there. Parallax's defensible delta is its **production-incident + agent-bundle** domain (where Braintrust doesn't play) + **Apache-OSS-core** + **Sentry-envelope**. Cross-reference: Parallax's **A1 gate is an eval question — Braintrust-class eval tooling is exactly how to measure whether a Parallax bundle beats raw context for agent fix outcomes.** Not direct competitors; potentially complementary tooling.

## Open questions / what measurement would settle

- **A1 gate tooling** — could Braintrust's eval harness measure the Parallax A1 question (bundle-vs-raw-context for agent fix outcomes)? Worth exploring as a validation method, not a competitive overlap.
- **Braintrust exact pricing + OSS scope (2026)** — **RESOLVED pass 44** on live page (Starter free / Pro $249 / Enterprise custom + overage rates above). Enterprise offers on-prem or hosted; core remains closed SaaS (OSS SDK only).

## Sources (accessed 2026-07-17)

- [braintrust.dev](https://www.braintrust.dev/); [playgrounds-vs-experiments](https://www.braintrust.dev/foundations/playgrounds-vs-experiments).
- [cekura Braintrust pricing 2026](https://www.cekura.ai/blogs/braintrust-pricing); [aitoolsbakery review 2026](https://aitoolsbakery.com/blog/braintrust-review/).
- Parallax side: [validation/a1-bundle-value/](../../validation/a1-bundle-value/), [00-vision/ai-native-observability.md](../../00-vision/ai-native-observability.md).
- Sibling AI-obs deep-dives: [parallax-vs-langfuse.md](parallax-vs-langfuse.md), [parallax-vs-arize-phoenix.md](parallax-vs-arize-phoenix.md), [parallax-vs-langsmith.md](parallax-vs-langsmith.md).
