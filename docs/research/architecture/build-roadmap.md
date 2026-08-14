# Historical Build And Validation Sequence

<!-- markdownlint-disable MD013 -->

Research date: 2026-05-25

> **Status (2026-07-17): historical sequencing and research-gate record, not an
> active implementation roadmap.** Phase 1 shipped. Every unfinished product or
> engineering item is authoritative only in research decisions.
> Research experiments and market-validation protocols remain in their linked
> research ledgers, but no implementation may start from this file. The
> GreptimeDB + Turso decision also supersedes every fallback reference in the
> original projection. Closed plans cited in historical phase notes own no
> current work.

## Recorded Verification Rule

- Browser-based verification in the original sequence used `agent-browser` for
  navigation, interaction, screenshots, DOM assertions, and runtime UI checks.
- The environment record pinned `agent-browser` CLI version `0.31.1` at the time.
- Runs used deterministic named sessions and stored results in the relevant
  validation ledger. Current implementation plans define their own live gates.

## Purpose

The [technical implementation concept](implementation-concept.md) recorded
*what* the research proposed. This file recorded *in what order*, with the order
chosen to **kill the project as cheaply as possible** if it was going to die. It synthesizes the
[verdict](../decisions/go-no-go.md), the [bear case](../decisions/risks-and-bear-case.md), the
[bundle-value evaluation](../validation/a1-bundle-value/bundle-value-evaluation.md), and the benchmark specs
into one de-risking sequence with explicit go/no-go gates.

The governing principle, taken straight from the bear case: **validate the
existential market and product assumptions (A1 bundle value, A2 real users)
before the comfortable engineering (the storage benchmark).** The storage
benchmark is the fun problem; it is not the dangerous one. Do the scary,
cheap experiments first.

> **Alignment note (operator statement #5, 2026-06-11).** The gates in this sequence govern
> **market claims and further-investment framing**, not the operator's own tool. The operator
> ruled that goals 1+2 — local visibility and the server profile, per the
> [historical V1 build record](v1-build-plan.md) — were prioritized for
> operator-as-user-#1, in parallel with these gates, with autonomous fixing
> deferred. Local V1 subsequently shipped; server work now remains blocked in
> `plans/`. The two tracks feed each other: M2 bundle output is the Arm-C generator the Phase-0 A1 eval needs, and
> A1/A2 still decide what may be *claimed* and whether the market product gets further
> investment. Phase numbering below is unchanged; read "build nothing until the gate passes" as
> "claim nothing and invest no further market effort until the gate passes."

## The One Insight That Reorders Everything

You do **not** need the Parallax engine to test Parallax's core claim.

A1 ("a bundle helps an agent fix better than raw context") can be falsified in
days with a **hand-assembled bundle**: take a handful of real incidents, manually
build the evidence bundle a finished Parallax *would* produce, and run the
[bundle-value eval](../validation/a1-bundle-value/bundle-value-evaluation.md) arms against a coding agent.
Per the [fair-test design](../validation/a1-bundle-value/runtime-dependence-and-raw-baseline.md), the
decisive control is **B′ agentic-raw** (the agent with read tools over an *uncorrelated* telemetry
store), not a static dump — because capable 2026 agents already retrieve from raw telemetry — and the
decisive claim is on **runtime-dependent bugs (classes R1–R3)**, not repo-logic bugs (R0) the agent
fixes from the repo alone. If a hand-built bundle does not beat agentic-raw on runtime-dependent
tasks, no amount of GreptimeDB tuning will save the product. This is the cheapest possible test of the
most important assumption — do it first.

Likewise A2 ("real users beyond the operator") is tested by **talking to 20
teams**, not by building. Both existential checks cost days and zero
infrastructure.

## Phases And Gates

Each phase has an exit gate tied to a [bear-case](../decisions/risks-and-bear-case.md)
assumption. Failing a gate sends you back, not forward.

### Phase 0 — Validate the killers (days, ~no build)

- Hand-assemble evidence bundles for 10–12 seed tasks selected through the
  [bundle-value seed corpus](../validation/a1-bundle-value/bundle-value-seed-corpus.md): current executable
  SWE-style issue/fix/test tasks plus generated Parallax telemetry overlays,
  with operator/public incidents only when they pass the same gates. Generate
  those overlays through the
  [Phase 0 telemetry overlay contract](../validation/a1-bundle-value/phase0-telemetry-overlay-contract.md) so
  raw-dump and bundle arms share the same frozen evidence, then publish results
  through the
  [A1 eval result ledger and model refresh](../validation/a1-bundle-value/a1-eval-result-ledger-and-model-refresh.md).
- Label each seed task by runtime-dependence class (R0 repo-logic … R3 cross-tier) and keep the corpus **≥60% R1–R3**, per the [fair-test design](../validation/a1-bundle-value/runtime-dependence-and-raw-baseline.md).
- Run the bundle-value eval (arms A/B/**B′ agentic-raw**/C/D) with these manual bundles, ≥2 models; report R0 and R1–R3 **separately**.
- Interview ~20 target teams across the A2 slices: would they deploy? would they
  pay or sustain it? what is their actual debugging pain? Use the
  [user interview and deployment intent gate](../validation/a2-user-demand.md)
  and [A2 interview evidence ledger](../validation/a2-user-demand.md) so the
  result is scored by past behavior, redacted evidence rows, and concrete
  commitments, not compliments. Any budget, support, hosted, fixer, or
  enterprise-ops signal also feeds the
  [business model validation ledger](../validation/business-model.md).
- **Gate:** on runtime-dependent tasks (R1–R3), hand-bundle beats **agentic-raw (B′)** on fix quality
  at equal-or-lower cost (A1) **and** ≥a handful of teams would genuinely deploy (A2). If both fail,
  **stop or pivot** — this is the cheapest NO-GO and the most valuable possible outcome to learn now.
  (Per the [2026-05-29 skeptical re-assessment](../decisions/skeptical-reassessment-2026-05.md), A1-vs-raw
  is now the #1 existential gate; lead the product on the **air-gap / no-phone-home** wedge and sequence
  the paying tier — managed cloud + enterprise-ops — after A1, per
  [monetization-and-paying-segment.md](../validation/monetization-and-paying-segment.md).)

### Phase 1 — Tiny tier that made bundles real (shipped MVP)

The original phase built enough to generate the bundle automatically and repeatably:

- Local-first one-command server with managed local GreptimeDB standalone for observability evidence,
  Turso/SQLite-like metadata for grouping/state, short local retention, and `run_id` as the primary
  developer handle.
- OTLP ingest (subset) for traces, logs, and metrics; derive Parallax `error_event` rows from
  exception span events, span error status, and ERROR/FATAL logs; deterministic Rust-focused
  grouping from normalized evidence.
- Direct-SDK and Collector OTLP claim levels controlled by the
  [OTLP conformance ledger](../capture/otlp.md).
- Same-trace and same-run correlation → one real `run context` / `issue context` bundle.
- Storage capability boundaries with local GreptimeDB + Turso. The historical
  ClickHouse/Turso-only fallback projection is superseded and must not be
  implemented.
- CLI (`parallax run inspect …`, `parallax run bundle …`, `parallax issue context …`) + local context
  API; GraphQL is the preferred query/exploration API, with OTLP for ingest and minimal health/version
  endpoints.
- **Gate:** the auto-generated bundle reproduces the Phase-0 hand-bundle quality
  (re-run A1 on real pipeline output); tiny-tier setup is meaningfully simpler
  than self-hosted Sentry (<=15 min) under the
  [self-hosted simplicity gate](../validation/self-hosted-simplicity.md). This is the
  "simpler than Sentry" proof.

### Phase 2 — Engine and evidence gates (partly shipped; ownership migrated)

- GreptimeDB is the committed engine. Comparative benchmark research remains in
  [the storage benchmark](../storage/benchmark-plan.md), while server-profile
  implementation is blocked in plan 115 and cannot introduce a fallback.
- Sentry envelope adapter closed as plan 118 (DONE); the
  [capture note](../capture/sentry-ingest.md) remains design evidence only.
- Retention/prune, evidence pinning, runtime redaction/A6, and evidence-contract
  reconciliation are owned by plans 116, 106, 111, and 104 respectively.
- Schema/corpus adoption remains a research claim gate in
  [A3](../validation/a3-schema-corpus.md), not an implementation checklist here.

The historical gate now means that performance/cost or redaction evidence can
block claims and expose GreptimeDB/Parallax work. It cannot authorize ClickHouse
or any other product fallback.

### Phase 3 — Historical scale-and-breadth projection

This phase originally grouped UI, server topology, MCP, CLI/agent tracing, and
frontend collection. It is not a backlog. The UI and several capture surfaces
subsequently shipped; current UI restructuring ownership is historical plan 100
(closed or superseded by later UI plans), server/auth residual is plans
109/110/115, and local-stdio product MCP graduated plan 112 (DONE; remote → 109).
Capture ledgers continue to constrain product claims. Any future unlisted
implementation must first receive a numbered plan in `plans/`.

### Phase 4 — Historical fixer projection

The separate fixer and outcome loop remain product research in the
[fixer-boundary decision](../decisions/fixer-boundary.md), A3 corpus gate, and
[business-model ledger](../validation/business-model.md). They are not
authorized implementation work. If the operator opens that scope, create a
numbered plan before editing product code; this file supplies no executable
steps or completion claim.

## Assumption → Phase Map

| Assumption (bear case) | Tested in | Cheapest test |
| --- | --- | --- |
| A1 bundle value | Phase 0 (hand), re-check Phase 1 (auto) | [manual bundle + eval](../validation/a1-bundle-value/bundle-value-phase0-runbook.md), days |
| A2 real users | Phase 0 | [20 scored deployment-intent interviews](../validation/a2-user-demand.md) plus the [redacted A2 evidence ledger](../validation/a2-user-demand.md) |
| Business value capture | Phase 0 signal capture → Phase 4 conversion | [business model validation ledger](../validation/business-model.md): budget, hosted, fixer, enterprise ops, support/services, conversion, and paid-pilot rows |
| A6 redaction trust | Phase 2 | [red-team ledger](../capture/redaction.md) over seeded fixtures plus real-data pilot |
| A5 stack holds | Phase 2 | [A5 stack decision ledger](../decisions/stack-decision.md), rolling up storage/metadata/ingest/setup gates |
| A4 correlation reliable | Phase 1–2 | [strong-edge prevalence on real telemetry](../capture/correlation.md) plus the [A4 result ledger](../capture/correlation.md) |
| A3 schema/corpus moat | Phase 2 (publish) → Phase 4 (corpus) | [schema conformance + external adoption + outcome corpus](../validation/a3-schema-corpus.md) |
| Coding-agent trace audit value | Phase 3 | [agent-session tracing ledger](../capture/agent-cli-tracing.md): dated tool/version/config matrix, at least one native OTel adapter and one non-OTel structured adapter, lossiness, redaction, projection, overhead, and audit-value rows |
| A7 scope discipline | enforced by phase order | [A7 scope discipline ledger](../validation/a7-scope.md) stays green and the tiny tier passes the [self-hosted simplicity gate](../validation/self-hosted-simplicity.md) with claim status in the [self-hosted simplicity ledger](../validation/self-hosted-simplicity.md) before breadth |

## What This Sequence Refuses To Do

- Build the storage layer for months before testing A1. (Most common failure
  mode for infra-minded founders; the bear case's "comfortable engineering" trap.)
- Add frontend, MCP, fixer, or Tier-3 before the tiny tier is excellent (A7).
- Treat "coding-agent tracing" as one roadmap milestone or product claim before
  per-surface fixture rows exist.
- Claim bundle value publicly before the
  [Phase 0 bundle eval](../validation/a1-bundle-value/bundle-value-phase0-runbook.md) and Phase 1 automated
  evidence exist.
- Bet Tier-3 on Iggy clustering that does not exist yet.

## Relationship To Other Research

- [Verdict](../decisions/go-no-go.md) and [risks/bear case](../decisions/risks-and-bear-case.md) — the GO and
  the assumptions this sequences.
- [Bundle-value evaluation](../validation/a1-bundle-value/bundle-value-evaluation.md) — the Phase 0/1 gate.
- [Bundle-value seed corpus](../validation/a1-bundle-value/bundle-value-seed-corpus.md) and
  [Bundle-value Phase 0 runbook](../validation/a1-bundle-value/bundle-value-phase0-runbook.md) — the first
  task-source selection and paired run against raw telemetry dumps.
- [Phase 0 telemetry overlay contract](../validation/a1-bundle-value/phase0-telemetry-overlay-contract.md) —
  the no-cheat artifact contract for the telemetry overlay used by that paired
  run.
- [A1 eval result ledger and model refresh](../validation/a1-bundle-value/a1-eval-result-ledger-and-model-refresh.md)
  — the public A1 result artifact and refresh policy for avoiding stale or
  contaminated bundle-value claims.
- [User interview and deployment intent gate](../validation/a2-user-demand.md)
  — the A2 demand-validation runbook for Phase 0.
- [A2 interview evidence ledger](../validation/a2-user-demand.md) — the
  privacy-preserving public artifact that makes the A2 result auditable.
- [Business model validation ledger](../validation/business-model.md) — the
  claim-level contract for adoption, budget, hosted, fixer, enterprise ops,
  support/services, conversion, and paid-pilot evidence.
- [Repo-intent value ledger](../validation/repo-intent.md) — the paired eval for
  whether docs, decisions, tasks, roadmap, and agent instructions improve bundle
  value without weakening runtime-only degraded mode.
- [Schema adoption and corpus moat gate](../validation/a3-schema-corpus.md)
  — the A3 conformance/adoption/corpus runbook for Phase 2 onward.
- [A3 schema adoption and corpus ledger](../validation/a3-schema-corpus.md)
  — the public event ledger for schema reviews, integrations, conformance runs,
  compatibility decisions, and outcome-corpus rows.
- [Correlation reliability on real telemetry gate](../capture/correlation.md)
  — the A4 strong-edge prevalence gate for Phase 1/2 real telemetry.
- [A4 correlation reliability ledger](../capture/correlation.md) —
  the run manifest, per-anchor rows, manual audit rows, claim levels, and
  freshness rules for making A4 pass/fail claims auditable.
- [A6 redaction red-team ledger](../capture/redaction.md) — the
  redaction result artifact for seeded canary leaks, scanner comparisons,
  projection audits, usefulness preservation, and claim freshness before agent
  exposure.
- [A5 stack decision ledger](../decisions/stack-decision.md) — the Phase 2 umbrella
  result contract for testing stack claims and exposing GreptimeDB/Turso risks;
  it no longer authorizes fallback decisions.
- [A7 scope discipline ledger](../validation/a7-scope.md) — the phase budget
  and feature-admission contract that prevents broad roadmap work from entering
  Phase 1 before the tiny bundle proof.
- [Self-hosted simplicity ledger](../validation/self-hosted-simplicity.md) — the
  clean-VM run artifact for install time, service/resource budget, ingest smoke,
  restart durability, backup/restore, upgrade, and redaction proof.
- [Sentry SDK compatibility ledger](../capture/sentry-ingest.md) — the
  claim-level contract for turning real SDK fixture runs into allowed
  Sentry-compatible product wording.
- [OTLP conformance ledger](../capture/otlp.md) — the claim-level
  contract for turning direct-SDK and Collector fixture runs into allowed
  OTLP-native product wording.
- [Agent access surface: CLI, HTTP API, and MCP](../decisions/agent-access-surface.md)
  — the focused answer to the CLI-versus-MCP access-surface question.
- [Agent access surface safety ledger](../decisions/agent-access-surface.md)
  — the claim-level contract for CLI/HTTP/MCP projection equivalence and
  read-only MCP safety.
- [Agent and CLI execution tracing](../capture/agent-cli-tracing.md) — why
  CLI invocations and coding-agent sessions belong in the execution graph.
- [Agent session tracing across real tools](../capture/agent-cli-tracing.md)
  and [Agent session tracing ledger](../capture/agent-cli-tracing.md) — the
  per-tool, per-capture-surface fixture contract before agent-session tracing is
  product wording.
- [CLI trace safety ledger](../capture/agent-cli-tracing.md) — the claim-level
  contract for default-ready CLI capture, redacted excerpts, raw refs,
  child-process policy, and projection safety.
- [Deploy/change context ledger](../capture/deploy-change-context.md) — the
  claim-level contract for release-regression and "what changed?" context.
- [Production database evidence access gate](../capture/production-db-evidence.md)
  — the safety gate before direct production database evidence enters bundles.
- [Production database evidence ledger](../capture/production-db-evidence.md)
  — the claim-level contract for proving least privilege, RLS/view scoping,
  template parsing, redaction, audit, and projection safety.
- [Technical implementation concept](implementation-concept.md) — the
  historical component detail each phase built or projected.
- [Storage benchmark prototype](../storage/benchmark-plan.md),
  [retention cost model](../storage/size-and-object-cost.md) — Phase 2 gates.
- [Business model](../validation/business-model.md) and
  [business model validation ledger](../validation/business-model.md) —
  Phase 4 value capture and the result rows required before it is claimable.
- [Fixer component and outcome loop](../decisions/fixer-boundary.md) —
  Phase 4 fixer boundary, outcome schema, and autonomy gates.
- [Fixer outcome ledger](../decisions/fixer-boundary.md) — Phase 4 result rows and
  claim levels for bundle handoff, PR creation, CI, review, merge/revert,
  recurrence, evidence citation, and allowed fixer wording.

## Bottom Line

The sequencing principle was to order work by how cheaply each step could kill
the project. A hand-built bundle and twenty conversations could falsify Parallax
in a week; a storage benchmark could not. The tiny tier followed bundle value,
engine proof followed bundle relevance, and breadth followed tiny-tier quality.
That assumption-priority principle remains research guidance, not an executable
implementation queue in this file.
