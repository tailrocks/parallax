# Build Roadmap and Validation Sequence

<!-- markdownlint-disable MD013 -->

Research date: 2026-05-25

## Purpose

The [technical implementation concept](technical-implementation-concept.md) says
*what* to build. This says *in what order*, and the order is chosen to **kill the
project as cheaply as possible** if it is going to die. It synthesizes the
[verdict](verdict.md), the [bear case](risks-and-bear-case.md), the
[bundle-value evaluation](bundle-value-evaluation.md), and the benchmark specs
into one de-risking sequence with explicit go/no-go gates.

The governing principle, taken straight from the bear case: **validate the
existential market and product assumptions (A1 bundle value, A2 real users)
before the comfortable engineering (the storage benchmark).** The storage
benchmark is the fun problem; it is not the dangerous one. Do the scary,
cheap experiments first.

## The One Insight That Reorders Everything

You do **not** need the Parallax engine to test Parallax's core claim.

A1 ("a bundle helps an agent fix better than raw context") can be falsified in
days with a **hand-assembled bundle**: take a handful of real incidents, manually
build the evidence bundle a finished Parallax *would* produce, and run the
[bundle-value eval](bundle-value-evaluation.md) arms (repo-only vs raw-dump vs
hand-bundle) against a coding agent. If a hand-built bundle does not beat a raw
telemetry dump, no amount of GreptimeDB tuning will save the product. This is
the cheapest possible test of the most important assumption — do it first.

Likewise A2 ("real users beyond the operator") is tested by **talking to 20
teams**, not by building. Both existential checks cost days and zero
infrastructure.

## Phases And Gates

Each phase has an exit gate tied to a [bear-case](risks-and-bear-case.md)
assumption. Failing a gate sends you back, not forward.

### Phase 0 — Validate the killers (days, ~no build)

- Hand-assemble evidence bundles for 5–10 real incidents (operator's repos +
  public incidents).
- Run the bundle-value eval (arms A/B/C) with these manual bundles, ≥2 models.
- Interview ~20 target teams across the A2 slices: would they deploy? would they
  pay or sustain it? what is their actual debugging pain? Use the
  [user interview and deployment intent gate](user-interview-and-deployment-intent-gate.md)
  so the result is scored by past behavior and concrete commitments, not
  compliments.
- **Gate:** hand-bundle beats raw-dump on fix quality (A1) **and** ≥a handful of
  teams would genuinely deploy (A2). If both fail, **stop or pivot** — this is the
  cheapest NO-GO and the most valuable possible outcome to learn now.

### Phase 1 — Tiny tier that makes bundles real (the MVP)

Build only enough to generate the bundle automatically and repeatably:

- Sentry-envelope + OTLP ingest (subset), deterministic Rust-focused grouping.
- Same-trace correlation → one real `issue context` bundle.
- GreptimeDB standalone + Turso metadata, local WAL, single binary.
- CLI (`parallax issue context …`) + read-only context API.
- **Gate:** the auto-generated bundle reproduces the Phase-0 hand-bundle quality
  (re-run A1 on real pipeline output); tiny-tier setup is meaningfully simpler
  than self-hosted Sentry (≤15 min). This is the "simpler than Sentry" proof.

### Phase 2 — Prove the engine and start the moat clock

- Run the [storage benchmark prototype](storage-benchmark-prototype.md)
  (GreptimeDB vs ClickHouse) — now justified, because bundles have proven value.
- Validate [retention cost](retention-cost-model.md) on real data; pick the
  object store (R2/B2 vs S3 per the egress finding).
- Redaction red-team (A6) before any third-party-model exposure.
- Publish the [open evidence schema](evidence-bundle-and-schema.md) with the
  machine-readable artifacts and conformance suite required by the
  [schema adoption and corpus moat gate](schema-adoption-and-corpus-moat-gate.md)
  → starts the A3 adoption clock.
- **Gate:** storage gates pass (freshness/latency/cost) or ClickHouse substitutes;
  redaction leak rate acceptable.

### Phase 3 — Scale seams and breadth

- Tier-2 topology (split ingest/workers, object storage, optional Iggy
  single-node; NATS/Redpanda reserved for Tier-3 clustering per
  [messaging](messaging-and-ingestion-layer.md)).
- Add the read-only MCP adapter specified in
  [Agent access surface: CLI, HTTP API, and MCP](agent-access-surface-cli-api-mcp.md),
  then CLI-invocation + coding-agent session tracing, then frontend collection
  ([frontend](frontend-collection-and-cross-tier-correlation.md)).
- **Gate:** scale-out changes topology, not the event/bundle contract.

### Phase 4 — Value capture and the feedback loop

- The separate **fixer** component (PR proposals) — the commercial seam from
  [business model](business-model-and-economics.md).
- Accepted/rejected-fix outcome capture → the failure/fix corpus (A3 moat).
- **Gate:** fixes cite evidence, record outcomes, and feed recurrence back.

## Assumption → Phase Map

| Assumption (bear case) | Tested in | Cheapest test |
| --- | --- | --- |
| A1 bundle value | Phase 0 (hand), re-check Phase 1 (auto) | [manual bundle + eval](bundle-value-phase0-runbook.md), days |
| A2 real users | Phase 0 | [20 scored deployment-intent interviews](user-interview-and-deployment-intent-gate.md) |
| A6 redaction trust | Phase 2 | red-team on real data |
| A5 stack holds | Phase 2 | storage/metadata benchmarks |
| A4 correlation reliable | Phase 1–2 | [strong-edge prevalence on real telemetry](correlation-reliability-real-telemetry-gate.md) |
| A3 schema/corpus moat | Phase 2 (publish) → Phase 4 (corpus) | [schema conformance + external adoption + outcome corpus](schema-adoption-and-corpus-moat-gate.md) |
| A7 scope discipline | enforced by phase order | tiny tier excellent before breadth |

## What This Sequence Refuses To Do

- Build the storage layer for months before testing A1. (Most common failure
  mode for infra-minded founders; the bear case's "comfortable engineering" trap.)
- Add frontend, MCP, fixer, or Tier-3 before the tiny tier is excellent (A7).
- Claim bundle value publicly before the
  [Phase 0 bundle eval](bundle-value-phase0-runbook.md) and Phase 1 automated
  evidence exist.
- Bet Tier-3 on Iggy clustering that does not exist yet.

## Relationship To Other Research

- [Verdict](verdict.md) and [risks/bear case](risks-and-bear-case.md) — the GO and
  the assumptions this sequences.
- [Bundle-value evaluation](bundle-value-evaluation.md) — the Phase 0/1 gate.
- [Bundle-value Phase 0 runbook](bundle-value-phase0-runbook.md) — the first
  paired run against raw telemetry dumps.
- [User interview and deployment intent gate](user-interview-and-deployment-intent-gate.md)
  — the A2 demand-validation runbook for Phase 0.
- [Schema adoption and corpus moat gate](schema-adoption-and-corpus-moat-gate.md)
  — the A3 conformance/adoption/corpus runbook for Phase 2 onward.
- [Correlation reliability on real telemetry gate](correlation-reliability-real-telemetry-gate.md)
  — the A4 strong-edge prevalence gate for Phase 1/2 real telemetry.
- [Agent access surface: CLI, HTTP API, and MCP](agent-access-surface-cli-api-mcp.md)
  — the focused answer to the CLI-versus-MCP access-surface question.
- [Production database evidence access gate](production-database-evidence-access.md)
  — the safety gate before direct production database evidence enters bundles.
- [Technical implementation concept](technical-implementation-concept.md) — the
  component detail each phase builds.
- [Storage benchmark prototype](storage-benchmark-prototype.md),
  [retention cost model](retention-cost-model.md) — Phase 2 gates.
- [Business model](business-model-and-economics.md) — Phase 4 value capture.

## Bottom Line

Order the work by how cheaply each step can kill the project. A hand-built bundle
and twenty conversations can falsify Parallax in a week; a storage benchmark
cannot. Build the tiny tier only after the bundle earns its keep, prove the engine
only after bundles matter, and add breadth only after the tiny tier is excellent.
De-risk in assumption-priority order, not in build-comfort order.
