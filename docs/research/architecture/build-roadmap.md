# Build Roadmap and Validation Sequence

<!-- markdownlint-disable MD013 -->

Research date: 2026-05-25

## Purpose

The [technical implementation concept](implementation-concept.md) says
*what* to build. This says *in what order*, and the order is chosen to **kill the
project as cheaply as possible** if it is going to die. It synthesizes the
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
> [V1 build plan](v1-build-plan.md) — are built now for operator-as-user-#1, in parallel with
> these gates, with autonomous fixing deferred to a future nice-to-have. The two tracks feed
> each other: the build's M2 bundle output is the Arm-C generator the Phase-0 A1 eval needs, and
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

### Phase 1 — Tiny tier that makes bundles real (the MVP)

Build only enough to generate the bundle automatically and repeatably:

- Local-first one-command server with managed local GreptimeDB standalone for observability evidence,
  Turso/SQLite-like metadata for grouping/state, short local retention, and `run_id` as the primary
  developer handle.
- OTLP ingest (subset) for traces, logs, and metrics; derive Parallax `error_event` rows from
  exception span events, span error status, and ERROR/FATAL logs; deterministic Rust-focused
  grouping from normalized evidence.
- Direct-SDK and Collector OTLP claim levels controlled by the
  [OTLP conformance ledger](../capture/otlp.md).
- Same-trace and same-run correlation → one real `run context` / `issue context` bundle.
- Storage adapter contract with local GreptimeDB profile implemented first; ClickHouse and Turso-only
  fallback remain interface targets, not Phase-1 blockers.
- CLI (`parallax run inspect …`, `parallax run bundle …`, `parallax issue context …`) + local context
  API; GraphQL is the preferred query/exploration API, with OTLP for ingest and minimal health/version
  endpoints.
- **Gate:** the auto-generated bundle reproduces the Phase-0 hand-bundle quality
  (re-run A1 on real pipeline output); tiny-tier setup is meaningfully simpler
  than self-hosted Sentry (<=15 min) under the
  [self-hosted simplicity gate](../validation/self-hosted-simplicity.md). This is the
  "simpler than Sentry" proof.

### Phase 2 — Prove the engine and start the moat clock

- Implement the GreptimeDB production/server storage profile and run the
  [storage benchmark prototype](../storage/benchmark-plan.md) (GreptimeDB vs ClickHouse) — now
  justified, because local bundles have proven value.
- Add the Sentry-compatible envelope adapter only if the OTLP-first local loop is proven and Sentry
  migration becomes the next highest-value adoption path; compatibility claims remain controlled by
  the [Sentry SDK compatibility ledger](../capture/sentry-ingest.md).
- Validate [retention cost](../storage/size-and-object-cost.md) on real data; pick the
  object store (R2/B2 vs S3 per the egress finding).
- Redaction red-team (A6) before any third-party-model exposure.
- Publish the [open evidence schema](evidence-bundle-schema.md) with the
  machine-readable artifacts and conformance suite required by the
  [schema adoption and corpus moat gate](../validation/a3-schema-corpus.md)
  and [A3 schema adoption and corpus ledger](../validation/a3-schema-corpus.md)
  → starts the A3 adoption clock.
- **Gate:** storage gates pass (freshness/latency/cost) or ClickHouse substitutes;
  redaction leak rate acceptable.

### Phase 3 — Scale seams and breadth

- Add the simple local investigation UI specified in
  [Simple UI V2 Concept](simple-ui-v2.md): TanStack Start + shadcn/ui over the Parallax API, with
  Sentry-style grouped issues, stack traces, run timeline, trace waterfall, log object inspection,
  metric windows, and bundle preview. Do not build a dashboard suite first.
- Tier-2 topology (split ingest/workers, object storage, optional Iggy
  single-node; NATS/Redpanda reserved for Tier-3 clustering per
  [messaging](../storage/streaming/messaging-and-ingestion-layer.md)).
- Add the read-only MCP adapter specified in
  [Agent access surface: CLI, HTTP API, and MCP](../decisions/agent-access-surface.md).
- Add CLI-invocation tracing only after the
  [CLI trace safety ledger](../capture/agent-cli-tracing.md) passes the relevant
  capture/redaction/overhead level.
- Add coding-agent session tracing surface by surface, not as one generic
  feature: Claude OTel and `stream-json`, Codex hooks and `exec --json` JSONL,
  Amp plugins and streaming JSON, and OpenCode run JSON/export/plugin/server/API
  and ACP all require separate rows in the
  [Agent session tracing ledger](../capture/agent-cli-tracing.md).
- Add frontend collection after the privacy and cross-tier gates in
  [frontend collection](../capture/frontend.md).
- **Gate:** scale-out changes topology, not the event/bundle contract; no agent
  tracing wording goes beyond the exact adapter/version/config claim level the
  ledger has passed.

### Phase 4 — Value capture and the feedback loop

- The separate **fixer** component (PR proposals) — the commercial seam from
  [business model](../validation/business-model.md), measured through the
  [fixer outcome ledger](../decisions/fixer-boundary.md) before any value claim feeds
  the [business model validation ledger](../validation/business-model.md).
- Accepted/rejected/reverted fixer outcome capture -> the
  failure/fixer-outcome corpus (A3 moat).
- Use the [fixer component and outcome loop](../decisions/fixer-boundary.md)
  contract and [fixer outcome ledger](../decisions/fixer-boundary.md) so opened PRs are
  not counted as successful fixes until review, validation, and recurrence
  evidence support that label.
- **Gate:** fixes cite evidence, record outcomes, and feed recurrence back.

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
  result contract for turning component benchmarks into stack defaults or
  fallback decisions.
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
  component detail each phase builds.
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

Order the work by how cheaply each step can kill the project. A hand-built bundle
and twenty conversations can falsify Parallax in a week; a storage benchmark
cannot. Build the tiny tier only after the bundle earns its keep, prove the engine
only after bundles matter, and add breadth only after the tiny tier is excellent.
De-risk in assumption-priority order, not in build-comfort order.
