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
Last updated 2026-07-18 after pass 275 (GO composite reaffirm after **268–274**;
all tracked kills **UNFIRED**; A1/A2/A3/A4 open; A6 mixed open). Pass 274 =
Traceway/Assistant.

**Shipment note:** local visibility and the server product are implemented, including the human UI,
alerting, live streaming, and bounded evidence production. The historical build sequence remains in
[architecture/v1-build-plan.md](architecture/v1-build-plan.md). A1 should now evaluate the real bundle
producer rather than treat its construction as future work. Autonomous fixing remains outside V1.

## Priority queue

| # | Question (what we must learn) | Why it gates the GO | Method | Status | Output / home |
| --- | --- | --- | --- | --- | --- |
| **1** | **Does a bounded bundle beat *raw* context for agent fix-quality, on runtime-dependent bugs?** (A1) | **#1 product-validation risk.** Capable 2026 agents fix repo-logic bugs from raw context; if a bundle doesn't beat agentic-raw on R1–R3 bugs, the schema moat collapses. | Offline eval using the shipped bundle producer: class-labeled corpus → frozen noisy overlay → arms A/B/B′/C/D → hidden-test grading. | **Pass 265:** claim level still **`not_measured`**. Golden **ok** (re-ran); no result ledger; SWE-bench_Lite HF **200**. Design + producer **≠** C-vs-B. Next: freeze → overlays → arms. | [validation/a1-bundle-value/](validation/a1-bundle-value/) ([status recheck](validation/a1-bundle-value/a1-claim-status-recheck-2026-07-17.md), [fair-test](validation/a1-bundle-value/runtime-dependence-and-raw-baseline.md)) |
| **2** | **Is there a sustainable *paying* segment, and what is the product that captures it?** (A2 + business model) | **#1 business risk.** Open self-hosted is structurally non-paying; survivors monetized via managed cloud / enterprise-gating. | Desk + interviews. | **Desk playbook holds** (pass **242/263**). **Pass 271:** A2 interview ledger still **zero rows** / gate OPEN; SO **2026** results still **404**. Operator runbook owed. | [validation/monetization-and-paying-segment.md](validation/monetization-and-paying-segment.md), [validation/a2-user-demand.md](validation/a2-user-demand.md), [validation/business-model.md](validation/business-model.md), [market/oss-agent-surface-gating-2026-07-17.md](market/oss-agent-surface-gating-2026-07-17.md) |
| **3** | **Will an open standard commoditize the evidence-bundle schema?** (esp. an OTel investigation/incident convention) | Kills the schema moat if it ships before adoption compounds. | Recurring web-watch (OTel semconv repo + Service/Deployment SIG; MCP roadmap). | **Pass 268:** still **not commoditized.** #1185 still **open/idle** (`updated_at` 2025-10-24). OCSF GA still **1.8.0**; **1.9.0-dev** only. Prior: pass 48/85/123/157/189/211/243. | [architecture/evidence-bundle-schema-commoditization-2026-07-17.md](architecture/evidence-bundle-schema-commoditization-2026-07-17.md), [architecture/evidence-bundle-schema.md](architecture/evidence-bundle-schema.md), [decisions/skeptical-reassessment-2026-05.md](decisions/skeptical-reassessment-2026-05.md) |
| **4** | **Does a wedge-closer ship the full combination first?** (Rustrak/SigNoz/GlitchTip/Traceway add OTLP + portable bundle + outcome) | Closes the technical wedge before Parallax has users → NO-GO trigger. | Recurring web-watch. | **Pass 270:** TMA1 still alpha12 **24th UNFIRED**. Traceway pass **264**. Error peers **244**. Combo **not closed**. | [market/wedge-closer-lightweight-recheck-2026-07-17.md](market/wedge-closer-lightweight-recheck-2026-07-17.md), [market/competitor-watch.md](market/competitor-watch.md) |
| **5** | **Sized storage cost + cold-read latency + self-host-vs-cloud + current stable re-test** | Characterizes the mandatory GreptimeDB engine. **Lower priority** — storage was never the existential risk. | Server-tier benchmark (cannot run in the dev capsule). | **Pass 266 (API pin only):** still **unproven** for size/cost. Stable Latest **`v1.1.3`**; nightly **`v1.2.0-nightly-20260706`**; CH feature **`v26.6.1.1193-stable`**. Traces still **experimental**. Server-tier owed. | [decisions/storage-engine.md](decisions/storage-engine.md), [storage/size-and-object-cost.md](storage/size-and-object-cost.md), [open-questions-and-gaps.md](storage/greptimedb-vs-clickhouse/open-questions-and-gaps.md) |
| **6** | **Do the loop-stage designs hold under replay?** (Detect trigger precision/recall, dispatch idempotency, recurrence verdicts on replayed telemetry) | The autonomous-fix-loop needs its own fixture ledger before any Detect/Dispatch claim; PoC kernels exist but a kernel is not a gate pass. | Create the Detect trigger ledger + replay harness over recorded telemetry. | **Pass 270:** offline residual + `fixer_outcome` unit tests **3/3 pass**; **no** Detect trigger ledger; live replay **open**. PoC ≠ gate. | [loop-stage status](validation/loop-stage-claim-status-recheck-2026-07-17.md), [autonomous-fix-loop.md](architecture/autonomous-fix-loop.md), [poc-evidence-loop-coverage.md](architecture/poc-evidence-loop-coverage.md), [plan 123](validation/2026-07-plan-123-fixer-offline/README.md) |

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
5. **Air-gapped agent-evidence: Parallax vs incumbents** — confirm the differentiator stays unique.
   **Rechecked 2026-07-18 pass 261** (prior 56/151/158/178/220/251): Seer self-host-excluded; Datadog OPW =
   still **route-to-destinations**; **BYOC Logs** = hybrid customer log store + **SaaS UI/Bits**
   (not offline agent-evidence combination); Grafana Assistant Cloud LLM (pass 245/260);
   OSS peers can air-gap but lack portable redacted bundle+outcome. Note:
   [market/air-gap-no-phone-home-recheck-2026-07-17.md](market/air-gap-no-phone-home-recheck-2026-07-17.md).

## Standing watches (cheap, recurring)

- **Engine releases** — re-pin + re-verify load-bearing claims on each new stable (GreptimeDB; ClickHouse feature line). Last: **2026-07-18 pass 266 (API only, no bench)** — GreptimeDB stable **`v1.1.3`** (GitHub **Latest**); latest *named* nightly **release** tag still **`v1.2.0-nightly-20260706`**. ClickHouse **feature** line still **`v26.6.1.1193-stable`** (do not pin **26.5.x** maintenance as feature tip). Server-tier size/cost measurement still owed (agenda #5). **Traces docs:**
  still **experimental** on docs v1.1
  ([Traces overview](https://docs.greptime.com/user-guide/traces/overview/)
  warning reconfirmed pass **266**).
- **Incumbent self-hosted AI** — **Rechecked 2026-07-18 pass 260** (prior 77/126/158/188/210/238/245):
  Seer still **closed / unavailable** on self-hosted
  ([develop.sentry.dev/self-hosted](https://develop.sentry.dev/self-hosted/)
  still lists "Seer and other AI & ML features… closed source"). Self-host release
  still **`26.7.0`** / **64** Compose services. Grafana
  Assistant **UI** on self-managed still requires **Cloud LLM backend**
  ([self-managed setup](https://grafana.com/docs/grafana-cloud/machine-learning/assistant/get-started/self-managed/)).
  Bits Code still **never auto-merges** PRs/MRs
  ([Bits Code docs](https://docs.datadoghq.com/bits_ai/bits_ai_dev_agent/)).
  Full note:
  [market/incumbent-self-hosted-ai-recheck-2026-07-17.md](market/incumbent-self-hosted-ai-recheck-2026-07-17.md).
  **UNFIRED:** Seer self-host GA; Grafana offline/BYO-LLM Assistant; Bits auto-merge.
- **OTel** — any move from per-signal semantics toward incident/investigation/RCA **artifacts**. Last deep recheck: **2026-07-18 pass 268** ([commoditization note](architecture/evidence-bundle-schema-commoditization-2026-07-17.md)): #1185 attribute issue still open/idle (`updated_at` 2025-10-24); no bundle schema; OCSF GA still 1.8.0 (`1.9.0-dev` only).
- **Run-id / invocation-id standardization (active participation, not just a watch)** — no OTel standard for a CLI invocation's cross-trace correlation id (rechecked **2026-07-18 pass 252**; prior 53/92/127/163/207). Parallax ships **`cli.invocation.id`** (+ `session.id`). GenAI session push [semantic-conventions-genai#51](https://github.com/open-telemetry/semantic-conventions-genai/issues/51) still **open/idle** (`updated_at` 2026-05-05). CLI model still process attrs only (`spans.yaml`); semconv code search `cli.invocation` **0**. Full table: [capture/run-id-standardization.md](capture/run-id-standardization.md).
- **Coding-agent capability** — as models improve, the "raw context is enough" threat (item 1) grows; A1 must re-run across model generations.

## How this maps to the kill criteria

Items 1 and 2 are the unresolved assumptions the historical
[skeptical re-assessment](decisions/skeptical-reassessment-2026-05.md) made load-bearing. V1 has
since shipped; failures here would change positioning and investment, not erase implementation
reality. Items 3 and 4 are live strategic triggers from the
[verdict's competitive window](decisions/go-no-go.md) and the
[bear case](decisions/risks-and-bear-case.md). Item 5 characterizes the committed stack rather than
selecting it. Active implementation proceeds only through `plans/`; this agenda does not own it.
