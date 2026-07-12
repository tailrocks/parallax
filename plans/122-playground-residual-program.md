# Plan 122: Close the external telemetry-playground residual program

> **Executor instructions**: This plan spans the companion
> `parallax-telemetry-playground` repository. Do not create a branch or PR in
> either repository. First distinguish shipped evidence from actual residuals;
> never replay completed historical phases or turn benchmark observations into
> product claims.

## Status

- **Priority**: P3
- **Effort**: XL
- **Risk**: HIGH
- **Depends on**: 100, 105, 111, 119
- **Category**: cross-repository playground / validation / demos
- **Planned at**: `eefa4617`, 2026-07-12
- **Status**: BLOCKED
- **Blocker**: The operator has not named an authorized active branch and exact
  residual scope for the companion repository under the current one-branch rule.

## Why

The research corpus contains a large playground sample specification, live
enrichment notes, and an OTLP fan-out lab. Much of their ordered work already
shipped through terminal plans, but the documents still mix completed phases,
unrun/server-scale experiments, and optional future scenarios. This plan is the
only engineering owner for genuine companion-repository residuals; the source
documents remain historical evidence and repeatable research protocols.

## Scope

In scope after the blocker clears:

- A commit-pinned two-repository inventory classifying every old row as shipped,
  obsolete, research-only, or genuinely actionable before edits.
- Only unresolved telemetry-shape scenarios needed by current Parallax contracts:
  native metric names/exemplars, trace links/events, cross-tier propagation,
  agent/CLI sessions, backpressure/drop evidence, and redaction canaries.
- Residual UI/playground fixtures required by plans 100/105/111/119.
- Deterministic one-command startup/progress/readiness and bounded scenario tours.
- OTLP fan-out lab maintenance as comparative research, including any explicitly
  authorized server-scale run, without making another backend a product fallback.
- Cross-language semantic-convention generation once plan 119 is ready.

Out of scope:

- Reimplementing completed ecosystem/story/attribute/link/gap features.
- Adding a new language, broker, database, backend, or topology for breadth alone.
- Running the server-scale benchmark on a laptop or converting comparator labs
  into supported Parallax product modes.
- Changing Parallax product contracts from the companion repository.

## Steps

1. Clear the blocker by recording the companion repository path, exact branch,
   baseline commit, allowed write sets, and operator-selected residual outcomes.
2. Reconcile the historical sample spec, live-verification note, fan-out lab,
   terminal plans, and live source into one machine-readable disposition table.
   Delete no evidence; move only executable residual ownership here.
3. For each retained scenario, define the exact OTel shape, expected native
   Greptime/Turso result, Parallax API/UI assertion, redaction boundary, and
   deterministic failure/reset behavior before implementation.
4. Implement disjoint scenario slices in the companion repo, using latest stable
   language ecosystems and the existing one-command harness. Every long operation
   reports progress/speed and ends with the complete ready surface list.
5. Add cross-repository fixtures/gates rather than screenshot-only completion.
   Integrate Weaver output only after plan 119 has deterministic generation.
6. Run small local comparison/fan-out scenarios; run large four-build/server
   work only on an authorized server and update the required consolidated matrix.
7. Re-audit both repositories and retire this plan when no engineering residual
   remains; leave repeatable experiment protocols in research.

## Test Plan

- Scenario manifest parser/disposition checks preventing completed rows reopening.
- Per-language builds/tests and sanitized OTLP golden fixtures.
- Real Parallax GreptimeDB + Turso ingestion/API/UI assertions for each scenario.
- Failure/restart/backpressure/redaction/cross-tier propagation tests.
- One-command clean-start/reset/readiness smoke on supported environments.
- Small four-backend fan-out and, only when authorized, server-tier benchmark
  evidence with exact versions, hashes, resource profiles, and limitations.

## Done Criteria

- [ ] Operator-approved companion branch/baseline/scope is recorded.
- [ ] Every historical row is classified; no completed phase is replayed.
- [ ] Each retained scenario has an exact cross-repository contract and fixture.
- [ ] Progress/readiness, reset, failure, and redaction behavior is deterministic.
- [ ] No comparator backend becomes a Parallax product fallback.
- [ ] Required Rust/Java/Bun, Parallax integration, and visual/API gates pass.
- [ ] Historical research notes contain evidence/protocols only, not engineering queues.

## STOP Conditions

- The companion repo, exact operator-approved branch, or baseline is unavailable.
- A proposed scenario has no current Parallax contract/test consumer.
- Work requires a new stack component or product claim without operator approval.
- A local run would violate the four-build laptop/server benchmark rule.
- Cross-repository wire behavior drifts without an explicit compatibility decision.

## Remove When

Delete this plan and row after the authorized companion residuals are shipped and
cross-repository evidence is green, or when the operator rejects them and all
remaining material is research protocol rather than engineering work.
