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
Last updated 2026-07-17 after pass 139 (Maple still **v0.0.12 / 1,532★** —
Tinybird decoupling **UNFIRED**; Odigos still **v1.31.2 / 3,668★** — own-store
**UNFIRED**). Pass 138 = auto-merge unclaimed. A1/A2/A4 open; A6 open at mixed
gate only.

**Shipment note:** local visibility and the server product are implemented, including the human UI,
alerting, live streaming, and bounded evidence production. The historical build sequence remains in
[architecture/v1-build-plan.md](architecture/v1-build-plan.md). A1 should now evaluate the real bundle
producer rather than treat its construction as future work. Autonomous fixing remains outside V1.

## Priority queue

| # | Question (what we must learn) | Why it gates the GO | Method | Status | Output / home |
| --- | --- | --- | --- | --- | --- |
| **1** | **Does a bounded bundle beat *raw* context for agent fix-quality, on runtime-dependent bugs?** (A1) | **#1 product-validation risk.** Capable 2026 agents fix repo-logic bugs from raw context; if a bundle doesn't beat agentic-raw on R1–R3 bugs, the schema moat collapses. | Offline eval using the shipped bundle producer: class-labeled corpus → frozen noisy overlay → arms A/B/B′/C/D → hidden-test grading. | **Pass 118:** claim level **`not_measured`**. Design + shipped producer + golden + PoC fixtures **≠** C-vs-B. No `result-ledger.md`/JSONL. Next: freeze manifest → overlays → arms. | [validation/a1-bundle-value/](validation/a1-bundle-value/) ([status recheck](validation/a1-bundle-value/a1-claim-status-recheck-2026-07-17.md), [fair-test](validation/a1-bundle-value/runtime-dependence-and-raw-baseline.md)) |
| **2** | **Is there a sustainable *paying* segment, and what is the product that captures it?** (A2 + business model) | **#1 business risk.** Open self-hosted is structurally non-paying; survivors monetized via managed cloud / enterprise-gating. | Desk + interviews. | **Desk playbook holds** (pass 54/94/106/117/119). **Pass 128:** A2 interview ledger still **zero rows** / gate OPEN — operator runbook execution owed; SO 2025 still live, 2026 not published. | [validation/monetization-and-paying-segment.md](validation/monetization-and-paying-segment.md), [validation/a2-user-demand.md](validation/a2-user-demand.md), [validation/business-model.md](validation/business-model.md), [market/oss-agent-surface-gating-2026-07-17.md](market/oss-agent-surface-gating-2026-07-17.md) |
| **3** | **Will an open standard commoditize the evidence-bundle schema?** (esp. an OTel investigation/incident convention) | Kills the schema moat if it ships before adoption compounds. | Recurring web-watch (OTel semconv repo + Service/Deployment SIG; MCP roadmap). | **Pass 123:** still **not commoditized.** #1185 still **open/idle** (`updated_at` 2025-10-24). No `incident`/`investigation` model dirs; `model/mcp` = tool telemetry; `model/artifact` = SLSA. OCSF still **1.8.0**. Prior: pass 48/85. Note: [architecture/evidence-bundle-schema-commoditization-2026-07-17.md](architecture/evidence-bundle-schema-commoditization-2026-07-17.md). | [architecture/evidence-bundle-schema-commoditization-2026-07-17.md](architecture/evidence-bundle-schema-commoditization-2026-07-17.md), [architecture/evidence-bundle-schema.md](architecture/evidence-bundle-schema.md), [decisions/skeptical-reassessment-2026-05.md](decisions/skeptical-reassessment-2026-05.md) |
| **4** | **Does a wedge-closer ship the full combination first?** (Rustrak/SigNoz/GlitchTip/Traceway add OTLP + portable bundle + outcome) | Closes the technical wedge before Parallax has users → NO-GO trigger. | Recurring web-watch. | **Pass 122+125:** combination **still not closed**. **Traceway** escalator (**1,024★**, v1.9.1). **Bugsink** **1,940★**/v2.4.0 error-only. **Rustrak** **64★**/server 0.9.2 + MCP 0.2.13. No full combo. Note: [market/wedge-closer-lightweight-recheck-2026-07-17.md](market/wedge-closer-lightweight-recheck-2026-07-17.md). | [market/wedge-closer-lightweight-recheck-2026-07-17.md](market/wedge-closer-lightweight-recheck-2026-07-17.md), [market/competitor-watch.md](market/competitor-watch.md) |
| **5** | **Sized storage cost + cold-read latency + self-host-vs-cloud + current stable re-test** | Characterizes the mandatory GreptimeDB engine. **Lower priority** — storage was never the existential risk. | Server-tier benchmark (cannot run in the dev capsule). | **Pass 135:** still **unproven**. Pins **`v1.1.3`** / CH **`v26.6.1.1193`**. Laptop smoke **saturated** (pass 110); concurrent Runs 220+ are re-verifies / packets — **not** A5 server $/GB. Next: **server-tier** + workload-mix. | [decisions/storage-engine.md](decisions/storage-engine.md), [storage/size-and-object-cost.md](storage/size-and-object-cost.md), [open-questions-and-gaps.md](storage/greptimedb-vs-clickhouse/open-questions-and-gaps.md) |
| **6** | **Do the loop-stage designs hold under replay?** (Detect trigger precision/recall, dispatch idempotency, recurrence verdicts on replayed telemetry) | The autonomous-fix-loop needs its own fixture ledger before any Detect/Dispatch claim; PoC kernels exist but a kernel is not a gate pass. | Create the Detect trigger ledger + replay harness over recorded telemetry. | **Executable PoC kernels + offline outcome residual exist (plan 123 DONE); live replay/product measurement remains open.** | [architecture/autonomous-fix-loop.md](architecture/autonomous-fix-loop.md), [architecture/poc-evidence-loop-coverage.md](architecture/poc-evidence-loop-coverage.md), [validation/2026-07-plan-123-fixer-offline](validation/2026-07-plan-123-fixer-offline/README.md) |

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
   **Rechecked 2026-07-17 pass 56:** Seer still self-host-excluded; Datadog no self-hosted backend;
   OSS peers can air-gap but lack portable redacted bundle+outcome. Avoid unscoped "Grafana phones
   home" without primary. Note:
   [market/air-gap-no-phone-home-recheck-2026-07-17.md](market/air-gap-no-phone-home-recheck-2026-07-17.md).

## Standing watches (cheap, recurring)

- **Engine releases** — re-pin + re-verify load-bearing claims on each new stable (GreptimeDB; ClickHouse feature line). Last: **2026-07-17 pass 60/90/97 + pass 129 (API only, no bench)** — GreptimeDB stable **`v1.1.3`** (GitHub **Latest**; release date 2026-07-17); latest *named* nightly tag still **`v1.2.0-nightly-20260706`** (no newer `nightly-2026071x` GitHub release tag). ClickHouse **feature** line still **`v26.6.1.1193-stable`** (2026-06-25). Newer **`v26.5.5.8-stable`** (2026-07-01) is a **26.5** patch, **not** the feature-line pin. Server-tier size/cost measurement still owed (agenda #5). **Traces docs GA:**
  still **experimental** on docs v1.1
  ([Traces overview](https://docs.greptime.com/user-guide/traces/overview/)
  warning: "experimental stage and may be adjusted"; **pass 83 + pass 130**).
- **Incumbent self-hosted AI** — **Rechecked 2026-07-17 pass 77 + pass 126:**
  Seer still **closed / unavailable** on self-hosted
  ([develop.sentry.dev/self-hosted](https://develop.sentry.dev/self-hosted/)
  still lists "Seer and other AI & ML features… closed source"). Grafana
  Assistant **UI** on self-managed still requires **Cloud LLM backend** (pass
  77). Full note:
  [market/incumbent-self-hosted-ai-recheck-2026-07-17.md](market/incumbent-self-hosted-ai-recheck-2026-07-17.md).
  **UNFIRED:** Seer self-host GA; Grafana offline/BYO-LLM Assistant.
- **OTel** — any move from per-signal semantics toward incident/investigation/RCA **artifacts**. Last deep recheck: **2026-07-17** ([commoditization note](architecture/evidence-bundle-schema-commoditization-2026-07-17.md)): #1185 attribute issue still open/idle; no bundle schema.
- **Run-id / invocation-id standardization (active participation, not just a watch)** — no OTel standard for a CLI invocation's cross-trace correlation id (rechecked **2026-07-17 pass 53 + 92 + 127**). Parallax ships **`cli.invocation.id`** (+ `session.id`). Historical `semantic-conventions#2883` **redirects to genai#51** (not a CLI issue). GenAI session push [semantic-conventions-genai#51](https://github.com/open-telemetry/semantic-conventions-genai/issues/51) still **open/idle** (`updated_at` 2026-05-05). CLI model still process attrs only. Full table: [capture/run-id-standardization.md](capture/run-id-standardization.md).
- **Coding-agent capability** — as models improve, the "raw context is enough" threat (item 1) grows; A1 must re-run across model generations.

## How this maps to the kill criteria

Items 1 and 2 are the unresolved assumptions the historical
[skeptical re-assessment](decisions/skeptical-reassessment-2026-05.md) made load-bearing. V1 has
since shipped; failures here would change positioning and investment, not erase implementation
reality. Items 3 and 4 are live strategic triggers from the
[verdict's competitive window](decisions/go-no-go.md) and the
[bear case](decisions/risks-and-bear-case.md). Item 5 characterizes the committed stack rather than
selecting it. Active implementation proceeds only through `plans/`; this agenda does not own it.
