# Research Agenda — What Is Still Open (and what to compare)

<!-- markdownlint-disable MD013 -->

Living backlog of research still needed to validate and extend the **shipped Parallax V1**, ranked
cheapest-to-kill-first. This is no longer a pre-build gate or implementation backlog: V1 ships as a
17-crate Rust workspace with CLI/server, OTLP and Sentry-envelope ingest (plan 118 DONE), GraphQL
(**76** queries / **14** mutations), GreptimeDB + Turso, evidence/redaction/analysis, alerting and
SSE, TanStack Start UI, and local-stdio MCP (plan 112 DONE). **Present-tense code claims** live in
[code-reality-ledger.md](code-reality-ledger.md) — use that before asserting capability here or in
market notes. Unfinished engineering is owned only by active numbered files in
[`plans/`](../../plans/). The full per-assumption proof-gate list (A1–A7 and the conformance ledgers)
lives in [decisions/strategic-coverage.md → "What Is Still Unproven"](decisions/strategic-coverage.md);
this file is the **prioritized, decision-moving** view plus the explicit **comparisons** still owed.
Last updated 2026-07-17 after V1 shipment + research code-reality audit. A1 and A2 remain
product/market validation risks, not blockers to code that already shipped. Autonomous-loop kernels
remain PoCs; fixer outcome work remains active in plan 123.

**Shipment note:** local visibility and the server product are implemented, including the human UI,
alerting, live streaming, and bounded evidence production. The historical build sequence remains in
[architecture/v1-build-plan.md](architecture/v1-build-plan.md). A1 should now evaluate the real bundle
producer rather than treat its construction as future work. Autonomous fixing remains outside V1.

## Priority queue

| # | Question (what we must learn) | Why it gates the GO | Method | Status | Output / home |
| --- | --- | --- | --- | --- | --- |
| **1** | **Does a bounded bundle beat *raw* context for agent fix-quality, on runtime-dependent bugs?** (A1) | **#1 product-validation risk.** Capable 2026 agents fix repo-logic bugs from raw context; if a bundle doesn't beat agentic-raw on R1–R3 bugs, the schema moat collapses. | Offline eval using the shipped bundle producer: class-labeled corpus → frozen noisy overlay → arms A/B/B′/C/D → hidden-test grading. | **Product and PoC machinery exist; comparative agent runs remain owed.** Next: freeze the task manifest, generate overlays, and run the arms against shipped bundles. | [validation/a1-bundle-value/](validation/a1-bundle-value/) ([fair-test](validation/a1-bundle-value/runtime-dependence-and-raw-baseline.md)) |
| **2** | **Is there a sustainable *paying* segment, and what is the product that captures it?** (A2 + business model) | **#1 business risk.** Open self-hosted is structurally non-paying; survivors monetized via managed cloud / enterprise-gating. | Desk + interviews. | **Segment sized + monetization shape designed (2026-05-29)**: paying buyer = hard-boundary (air-gap/classified/sovereign/geo-fenced) self-hoster; product = Apache-2.0 open core (kept consistent) + gated enterprise-ops + managed cloud + outcome-priced fixer. **A2 interviews still open.** | [validation/monetization-and-paying-segment.md](validation/monetization-and-paying-segment.md), [validation/a2-user-demand.md](validation/a2-user-demand.md), [validation/business-model.md](validation/business-model.md) |
| **3** | **Will an open standard commoditize the evidence-bundle schema?** (esp. an OTel investigation/incident convention) | Kills the schema moat if it ships before adoption compounds. | Recurring web-watch (OTel semconv repo + Service/Deployment SIG; MCP roadmap). | **Checked 2026-05-29: none on the roadmap.** **Constructive answer (2026-05-29): define the bundle as a PROFILE over OTel + Sentry-grouping + OCSF + CloudEvents (don't invent) to blunt this risk** — see [architecture/evidence-bundle-schema.md](architecture/evidence-bundle-schema.md). Watch the Feb-2026 HN "incident bundle" tool as prior art. | [decisions/skeptical-reassessment-2026-05.md](decisions/skeptical-reassessment-2026-05.md), [architecture/evidence-bundle-schema.md](architecture/evidence-bundle-schema.md), [capture/otlp.md](capture/otlp.md) |
| **4** | **Does a wedge-closer ship the full combination first?** (Rustrak/SigNoz/GlitchTip add OTLP-native ingest + a portable bundle) | Closes the technical wedge before Parallax has users → NO-GO trigger. | Recurring web-watch. | **Checked 2026-06-11: not closed.** Post-DASH drift (Bits Code GA, Bits Remediation preview, Sentry "self-healing workflow" APIs) accelerates the L2/L3 commodity race, but nobody ships closed-loop app-code fixing or open outcome/recurrence records — the earned-autonomy substrate stays unclaimed. | [market/competitor-watch.md](market/competitor-watch.md) |
| **5** | **Sized storage cost + cold-read latency + self-host-vs-cloud + v1.1 GA re-test** | Finalizes the storage engine. **Lower priority** — storage was never the existential risk. | Server-tier benchmark (cannot run in the dev capsule). | **Blocked/deferred.** v1.1 still not GA (re-verified 2026-06-11: stable `v1.0.2`, nightly line stalled at `v1.1.0-nightly-20260525`; JSON2 is the in-flight headline). | [decisions/storage-engine.md](decisions/storage-engine.md), [storage/size-and-object-cost.md](storage/size-and-object-cost.md) |
| **6** | **Do the loop-stage designs hold under replay?** (Detect trigger precision/recall, dispatch idempotency, recurrence verdicts on replayed telemetry) | The autonomous-fix-loop needs its own fixture ledger before any Detect/Dispatch claim; PoC kernels exist but a kernel is not a gate pass. | Create the Detect trigger ledger + replay harness over recorded telemetry. | **Executable PoC kernels exist; product outcome-loop work is active in plan 123; replay proof remains open.** | [architecture/autonomous-fix-loop.md](architecture/autonomous-fix-loop.md), [architecture/poc-evidence-loop-coverage.md](architecture/poc-evidence-loop-coverage.md) |

## Comparisons still owed (research = compare, then decide)

1. **Bundle vs agentic-raw** (the A1 core): C (Parallax bundle) vs B′ (agent with read tools over an
   uncorrelated telemetry store), per runtime-dependence class. The single most decision-moving
   comparison. → item 1.
2. **Monetization-model comparison**: Grafana Cloud vs SigNoz Cloud vs OpenObserve (open-core +
   enterprise-gate) vs a hosted-Parallax tier — to choose Parallax's actual paying product and what is
   gated vs open. → item 2.
3. **Evidence-bundle schema vs any emerging OTel investigation/incident schema**: structural overlap
   and whether to align with / extend the standard rather than compete. → item 3.
4. **Storage engines on sized cost** (GreptimeDB 1× object store vs OSS ClickHouse N× replicas; cold-read
   GB–TB): characterize the mandatory engine and identify fix-forward work; this no longer reopens the
   committed GreptimeDB + Turso stack. → item 5. (Query mix already resolved: anchored.)
5. **Air-gapped agent-evidence: Parallax vs incumbents** — confirm the "no-phone-home" differentiator
   stays unique (Grafana on-prem still phones cloud; Seer cloud-only; Datadog SaaS). → standing watch in
   competitor-watch.

## Standing watches (cheap, recurring)

- **Engine releases** — re-pin + re-verify load-bearing claims on each new stable (GreptimeDB v1.1 GA next; ClickHouse feature line). Last: 2026-06-11 — no GA, nightly publishing stalled at 20260525 while the repo stays active.
- **Incumbent self-hosted AI** — Sentry Seer self-host (stated FSL intent, no date); Grafana local-inference/BYO-LLM backend; either would erode the wedge.
- **OTel** — any move from per-signal semantics toward incident/investigation/RCA correlation.
- **Run-id standardization (active participation, not just a watch)** — no OTel standard exists for a CLI run's cross-trace correlation id; we intend to propose one (generalize `session.id` per [semantic-conventions#2883](https://github.com/open-telemetry/semantic-conventions/issues/2883), or a `cli.run.id`) and track every thread in [capture/run-id-standardization.md](capture/run-id-standardization.md). Adopt-as-alias the moment anything lands; `parallax.run.id` stays canonical until a standard reaches stability.
- **Coding-agent capability** — as models improve, the "raw context is enough" threat (item 1) grows; A1 must re-run across model generations.

## How this maps to the kill criteria

Items 1 and 2 are the unresolved assumptions the historical
[skeptical re-assessment](decisions/skeptical-reassessment-2026-05.md) made load-bearing. V1 has
since shipped; failures here would change positioning and investment, not erase implementation
reality. Items 3 and 4 are live strategic triggers from the
[verdict's competitive window](decisions/go-no-go.md) and the
[bear case](decisions/risks-and-bear-case.md). Item 5 characterizes the committed stack rather than
selecting it. Active implementation proceeds only through `plans/`; this agenda does not own it.
