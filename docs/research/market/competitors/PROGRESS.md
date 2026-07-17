# PROGRESS — Comparison Status Board

> The living work queue for this folder. Tracks verification state, last-verified
> dates, source links, open uncertainties, and the next highest-value gap. Update
> on **every pass**. State legend:
>
> - ✅ verified — checked against current primary sources this pass.
> - 🟡 inherited — carried from legacy market notes; not yet re-verified against
>   current sources. Treat as a hypothesis.
> - 🔴 stale / missing — aged number, dead source, or absent deep-dive. Highest priority.
> - ⚪ benchmark-dependent — needs measurement before any number is trusted.
>
> Today: **2026-07-17** (pass 1 — bootstrap).

## Pass log

| Pass | Date | What landed | Commit |
| --- | --- | --- | --- |
| 1 | 2026-07-17 | Bootstrap: `README.md` matrix, `comparison-set.md`, `PROGRESS.md`; first no-bias deep-dive **Datadog** (pricing + OTLP + AI/Agent Observability verified against live `datadoghq.com/pricing` + docs). Inherited matrix cells marked 🟡. Legacy matrix/feature-matrix notes left as sources with pointers. | _pending_ |
| 2 | 2026-07-17 | **SigNoz** deep-dive ([parallax-vs-signoz.md](parallax-vs-signoz.md)): version drift re-verified (v0.132.2, ClickHouse 25.12.5), "open investigation format" confirmed still no published schema, pricing re-cited. Legacy signoz-deep-research.md left as lead w/ pointer. | _pending_ |

## Deep-dive status (per product)

| Product | Deep-dive file | State | Last verified | Next gap |
| --- | --- | --- | --- | --- |
| Datadog | [parallax-vs-datadog.md](parallax-vs-datadog.md) | ✅ pass 1 | 2026-07-17 | self-host/agentless reality; gov/FedRAMP posture; exact 2026 OTLP-in-Agent GA scope |
| Sentry | — | 🔴 missing (legacy [sentry-deep-research.md](../sentry-deep-research.md)) | 2026-06 (legacy) | write `parallax-vs-sentry.md`; recheck OTLP GA + Seer pricing |
| Grafana Cloud/LGTM | — | 🔴 missing | n/a | write deep-dive; Tempo v3 + Pyroscope + Mimir cost model |
| Honeycomb | — | 🔴 missing | n/a | write deep-dive; high-cardinality model + pricing |
| New Relic | — | 🔴 missing | n/a | write deep-dive; entity model + NRAI + pricing |
| SigNoz | [parallax-vs-signoz.md](parallax-vs-signoz.md) | 🟡 exists — written by a concurrent pass 2026-07-17; **not yet re-verified by this pass.** Re-read and verify next. | 2026-07-17 (peer) | re-verify MCP "open investigation format" status; confirm ClickHouse/pricing |
| OpenObserve | — | 🟡 inherited | 2026-06 (legacy) | write `parallax-vs-openobserve.md`; AI/MCP gating + free-tier |
| Coroot | — | 🟡 inherited | 2026-06 (legacy) | write `parallax-vs-coroot.md`; eBPF partial-span limit + MCP RBAC |
| Maple | — | 🟡 inherited | 2026-06 (legacy) | write `parallax-vs-maple.md`; Tinybird coupling + local UX |
| TMA1 | — | 🟡 inherited | 2026-06 (legacy) | write `parallax-vs-tma1.md`; bundle artifact drift check |
| Highlight.io | — | 🔴 missing | n/a | write deep-dive; closest SaaS session-replay+errors peer |
| Langfuse | — | 🔴 missing | n/a | write deep-dive (AI wedge); evals + self-host |
| Arize Phoenix | — | 🔴 missing | n/a | write deep-dive (AI wedge); OSS evals |
| LangSmith | — | 🔴 missing | n/a | write deep-dive (AI wedge) |
| Dynatrace / Splunk Obs / Elastic Obs / Sumo / Chronosphere / Observe / Axiom | — | 🔴 missing | n/a | tier 2: one per later pass |
| Uptrace / HyperDX / Odigos | — | 🔴 watch | n/a | brief coverage; verify relevance first |
| PostHog / Helicone / Braintrust | — | 🔴 watch | n/a | brief coverage; AI wedge |

## Overview-matrix cells (from `README.md`)

The wide feature-presence matrix is bootstrapped from legacy notes. **Most cells
are 🟡 inherited** (sources dated 2026-05/06). Priority for re-verification:

1. Every Parallax-column cell — these are most bias-prone (self-assessed). Re-check against shipped code/spec, mark clearly whether the cell reflects *shipped* vs *planned*. (🔴 priority)
2. The AI-native/agent-context column across all competitors — fastest-moving axis, stale within weeks. (🔴 priority)
3. Pricing column — verify each against the live pricing page; mark "no public number" where absent. (🟡→✅ per product)
4. OTLP-native column — was true/false at 2026-06; several products shipped OTLP GA since. (🟡→✅ per product)

## Open uncertainties / questions

- **Datadog self-host posture**: marketing says SaaS-only; clarify whether any production self-host path beyond the Agent + Observability Pipelines Worker exists in 2026. Flagged in Datadog deep-dive.
- **Datadog OTLP-in-Agent GA scope**: confirm which signals (logs/metrics/traces) the Agent OTLP receiver ingests natively vs routes through transform. Flagged.
- **Parallax-column honesty**: several Parallax cells (fix-outcome loop, redacted bundle, MCP) are *planned*, not shipped. Keep the planned/shipped split explicit in every row — do not let "planned" read as "has it".

## Next highest-value gaps (ranked)

1. **Sentry deep-dive** — closest error-workflow incumbent, explicit "simpler than self-hosted Sentry" target, legacy note aging; high strategic value. Recheck OTLP GA (was beta HTTP-only) + Seer autofix pricing.
2. **Langfuse deep-dive** — AI/agent-observability wedge; OSS self-host; directly pressures Parallax's agent-context thesis. Missing entirely.
3. **Grafana Cloud/LGTM deep-dive** — largest OSS-origin managed stack; Tempo v3 + Pyroscope + Mimir cost model unverified.
4. **Honeycomb deep-dive** — defines the high-cardinality event axis; Bubbleuppy AI unverified.
5. **SigNoz cell re-verification** — exact current star count + MCP server version (v0.5.1 last confirmed 2026-06-17); trace/metric throughput (no current public number). Watch triggers in [parallax-vs-signoz.md](parallax-vs-signoz.md).

## Bias audit (this pass)

- ✅ Datadog deep-dive defaults to "Datadog may be better"; written that Parallax is behind on breadth/maturity/scale/enterprise/AI, with evidence.
- ✅ Parallax's only edges (open-source/self-host, cost predictability, data ownership, bundle thesis) are scoped to named axes; bundle thesis flagged unproven (A1 gate).
- ⚠️ The inherited overview matrix is Parallax-framed from legacy notes (columns = "Parallax's wedge"). Acceptable as bootstrap; must re-verify cells product-by-product and reframe verdicts as no-bias in later passes.
- ✅ SigNoz deep-dive (pass 2) defaults to "SigNoz may be better"; written plainly that SigNoz wins on maturity/breadth/MCP/scale/pricing-transparency, and that Parallax's bundle/outcome/Sentry edges are planned or unproven (A1 gate), not parity.
