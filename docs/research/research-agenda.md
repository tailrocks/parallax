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
Last updated 2026-07-17 after pass 96 (Sentry OTLP still **open beta**; **no OTLP
metrics**; self-host **26.7.0**). Pass 95 = HolmesGPT Operator. A1/A2/A4/A6 open.

**Shipment note:** local visibility and the server product are implemented, including the human UI,
alerting, live streaming, and bounded evidence production. The historical build sequence remains in
[architecture/v1-build-plan.md](architecture/v1-build-plan.md). A1 should now evaluate the real bundle
producer rather than treat its construction as future work. Autonomous fixing remains outside V1.

## Priority queue

| # | Question (what we must learn) | Why it gates the GO | Method | Status | Output / home |
| --- | --- | --- | --- | --- | --- |
| **1** | **Does a bounded bundle beat *raw* context for agent fix-quality, on runtime-dependent bugs?** (A1) | **#1 product-validation risk.** Capable 2026 agents fix repo-logic bugs from raw context; if a bundle doesn't beat agentic-raw on R1–R3 bugs, the schema moat collapses. | Offline eval using the shipped bundle producer: class-labeled corpus → frozen noisy overlay → arms A/B/B′/C/D → hidden-test grading. | **Product and PoC machinery exist; comparative agent runs remain owed.** Next: freeze the task manifest, generate overlays, and run the arms against shipped bundles. | [validation/a1-bundle-value/](validation/a1-bundle-value/) ([fair-test](validation/a1-bundle-value/runtime-dependence-and-raw-baseline.md)) |
| **2** | **Is there a sustainable *paying* segment, and what is the product that captures it?** (A2 + business model) | **#1 business risk.** Open self-hosted is structurally non-paying; survivors monetized via managed cloud / enterprise-gating. | Desk + interviews. | **Segment sized + monetization shape designed (2026-05-29)**; **desk recheck pass 54 + pass 94:** hard-boundary thesis holds; O2 EE free ≤**50 GB/day** + AI/SDR Enterprise; SigNoz Noz **Cloud**; Seer self-host-excluded; **A2 interviews still open** (desk cannot size). | [validation/monetization-and-paying-segment.md](validation/monetization-and-paying-segment.md), [validation/a2-user-demand.md](validation/a2-user-demand.md), [validation/business-model.md](validation/business-model.md), [market/oss-agent-surface-gating-2026-07-17.md](market/oss-agent-surface-gating-2026-07-17.md) |
| **3** | **Will an open standard commoditize the evidence-bundle schema?** (esp. an OTel investigation/incident convention) | Kills the schema moat if it ships before adoption compounds. | Recurring web-watch (OTel semconv repo + Service/Deployment SIG; MCP roadmap). | **Rechecked 2026-07-17 (pass 48 + pass 85): still not commoditized.** #1185 still **open/idle** (`updated_at` 2025-10-24). No `incident`/`investigation` model dirs; `model/mcp` is **deprecated MCP tool** telemetry (not a pack); `model/artifact` is SLSA packages. OCSF still **1.8.0** security `incident_finding`. Profile strategy retained. Note: [architecture/evidence-bundle-schema-commoditization-2026-07-17.md](architecture/evidence-bundle-schema-commoditization-2026-07-17.md). | [architecture/evidence-bundle-schema-commoditization-2026-07-17.md](architecture/evidence-bundle-schema-commoditization-2026-07-17.md), [architecture/evidence-bundle-schema.md](architecture/evidence-bundle-schema.md), [decisions/skeptical-reassessment-2026-05.md](decisions/skeptical-reassessment-2026-05.md) |
| **4** | **Does a wedge-closer ship the full combination first?** (Rustrak/SigNoz/GlitchTip/Traceway add OTLP + portable bundle + outcome) | Closes the technical wedge before Parallax has users → NO-GO trigger. | Recurring web-watch. | **Rechecked 2026-07-17 (pass 49 + pass 84 commercial):** combination **still not closed**. **Traceway** still escalator (1,024★, **v1.9.1**, public cloud Free→Enterprise pricing stable, agent skills/CLI); pricing-page **"Archive & resolve workflow (AI)"** = symbolication + agent RCA/archive, **not** portable redacted bundle + outcome. **Rustrak** MCP mutating; **Bugsink** pure Sentry-replacement. Note: [market/wedge-closer-lightweight-recheck-2026-07-17.md](market/wedge-closer-lightweight-recheck-2026-07-17.md), [competitors/parallax-vs-traceway.md](market/competitors/parallax-vs-traceway.md). | [market/wedge-closer-lightweight-recheck-2026-07-17.md](market/wedge-closer-lightweight-recheck-2026-07-17.md), [market/competitor-watch.md](market/competitor-watch.md) |
| **5** | **Sized storage cost + cold-read latency + self-host-vs-cloud + current stable re-test** | Characterizes the mandatory GreptimeDB engine. **Lower priority** — storage was never the existential risk. | Server-tier benchmark (cannot run in the dev capsule). | **Still open.** Pins **`v1.1.3`** / CH **`v26.6.1.1193`**. **Pass 82** TTL mechanism OK. **Pass 91 consume:** Run 189 local **N=100k** density ratios on current pins (logs→GT, spans→CH, shape-dependent) — **smoke only**, not $/GB/object-store/A5. Server-tier + mixed-native + cold-read + egress model still **unproven**. | [decisions/storage-engine.md](decisions/storage-engine.md), [storage/size-and-object-cost.md](storage/size-and-object-cost.md), [compression-and-cost.md](storage/greptimedb-vs-clickhouse/compression-and-cost.md) |
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

- **Engine releases** — re-pin + re-verify load-bearing claims on each new stable (GreptimeDB; ClickHouse feature line). Last: **2026-07-17 pass 60 + pass 90** — GreptimeDB stable **`v1.1.3`**, nightly **`v1.2.0-nightly-20260706`**; ClickHouse **feature** line still **`v26.6.1.1193-stable`** (2026-06-25). GitHub also shows newer **`v26.5.5.8-stable`** (2026-07-01) and LTS **`v25.8.28.1-lts`** — **not** the feature-line pin (policy: never LTS; prefer highest **feature** major.minor, not latest calendar patch on older line). Measurement still owed on the pins. **Traces docs GA (team Q#2):** still **experimental** on stable + Nightly (**pass 83**).
- **Incumbent self-hosted AI** — **Rechecked 2026-07-17 pass 77:** Seer still
  **closed / unavailable** on self-hosted
  ([develop.sentry.dev/self-hosted](https://develop.sentry.dev/self-hosted/)).
  Grafana Assistant **UI** is available on self-managed OSS/Enterprise (v13+) but
  **requires a Grafana Cloud stack** for the LLM backend (not local-inference /
  air-gap). Full note:
  [market/incumbent-self-hosted-ai-recheck-2026-07-17.md](market/incumbent-self-hosted-ai-recheck-2026-07-17.md).
  **UNFIRED:** Seer self-host GA; Grafana offline/BYO-LLM Assistant.
- **OTel** — any move from per-signal semantics toward incident/investigation/RCA **artifacts**. Last deep recheck: **2026-07-17** ([commoditization note](architecture/evidence-bundle-schema-commoditization-2026-07-17.md)): #1185 attribute issue still open/idle; no bundle schema.
- **Run-id / invocation-id standardization (active participation, not just a watch)** — no OTel standard for a CLI invocation's cross-trace correlation id (rechecked **2026-07-17 pass 53 + pass 92**). Parallax ships **`cli.invocation.id`** (+ `session.id`). Historical tracker link `semantic-conventions#2883` is **dead**; GenAI session push [semantic-conventions-genai#51](https://github.com/open-telemetry/semantic-conventions-genai/issues/51) still **open/idle** (`updated_at` 2026-05-05). Full table: [capture/run-id-standardization.md](capture/run-id-standardization.md).
- **Coding-agent capability** — as models improve, the "raw context is enough" threat (item 1) grows; A1 must re-run across model generations.

## How this maps to the kill criteria

Items 1 and 2 are the unresolved assumptions the historical
[skeptical re-assessment](decisions/skeptical-reassessment-2026-05.md) made load-bearing. V1 has
since shipped; failures here would change positioning and investment, not erase implementation
reality. Items 3 and 4 are live strategic triggers from the
[verdict's competitive window](decisions/go-no-go.md) and the
[bear case](decisions/risks-and-bear-case.md). Item 5 characterizes the committed stack rather than
selecting it. Active implementation proceeds only through `plans/`; this agenda does not own it.
