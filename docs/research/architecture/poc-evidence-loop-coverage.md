# PoC Coverage Map: What `poc/evidence-loop` Proves, and What It Does Not

<!-- markdownlint-disable MD013 -->

Research date: 2026-06-11. Status note: this map keeps the claim-wording discipline intact while
the PoC grows. **An executable kernel is not a gate pass.** The PoC demonstrates that a designed
mechanism runs, deterministically, on hand-written fixtures. Every product claim remains governed
by its ledger, and every ledger below still reads `not_measured` (or its equivalent early level).
When a PoC mechanism graduates into the product, its evidence must be re-earned through the
governing gate — SDK-generated fixtures, real telemetry, dated result rows.

## Mechanism → design → governing gate

| # | PoC mechanism (kernel) | Implements (design doc) | Governing gate / ledger | Product claim level |
| --- | --- | --- | --- | --- |
| 1–2 | Error derivation from both exception encodings; convergence to one fingerprint (`derive.rs`) | [capture/otlp.md](../capture/otlp.md) error-evidence mapping | OTLP conformance ledger (L1–L3 fixture runs) | not_measured |
| 3 | Deterministic fingerprinting with token normalization (`fingerprint.rs`) | [capture/rust.md](../capture/rust.md) grouping | Grouping-stability fixtures (SDK-generated, not hand-written) | not_measured |
| 4 | Bundle assembly: anchor, typed nodes, strength-tagged edges, `missing_evidence` (`bundle.rs`) | [evidence-bundle-schema.md](evidence-bundle-schema.md) | A1 bundle value; A4 correlation | not_measured |
| 5 | Redaction with machine-readable report (`redact.rs`) | [capture/redaction.md](../capture/redaction.md) | **A6 red-team (veto power)** — four demo regexes are nowhere near the six-stage pipeline + canary corpus | not_measured |
| 6 | Canonical sorted-key hashing (`bundle.rs`) | Fixer-boundary JCS requirement | Projection-equivalence gates (CLI/HTTP/MCP) | not_measured |
| 7, 15, 16 | Detect chain: deploy-adjacent trigger, EWMA spike kernel, pre-aggregation rollup with >100× size proof (`deploy.rs`, `spike.rs`, `rollup.rs`) | [autonomous-fix-loop.md](autonomous-fix-loop.md) §1, §Cost | Detect trigger ledger (does not exist yet — creating it is the gate work) | not designed→designed; not_measured |
| 8 | Recurrence kernel: Recurred/Silent/WindowOpen (`deploy.rs`) | [autonomous-fix-loop.md](autonomous-fix-loop.md) §5 | Fixer outcome ledger: merge/revert/recurrence tracking | not_measured |
| 9 | Earned autonomy budget, v0 thresholds (`budget.rs`) | [north-star §3](../00-vision/north-star-autonomous-fix-loop.md) | Fixer outcome ledger autonomy levels | not_measured |
| 10 | `parallax.fix_candidate.v0` dispatch payload (`dispatch.rs`) | [autonomous-fix-loop.md](autonomous-fix-loop.md) §3 | Agent access-surface gates | not_measured |
| 11–12 | Learner: outcome-citation edge weights; weights wired into edge ordering; budget demotion on appended revert (`learn.rs`) | [autonomous-fix-loop.md](autonomous-fix-loop.md) §6 | `outcome_feedback_loop_pass` | not_measured |
| 13 | Draft JSON Schemas + emitted-artifact validation (`schema/`) | [integration-contract.md](integration-contract.md) | **A3 schema gate** — these are PoC drafts; A3 needs the real versioned schema, validator, fixtures, compatibility policy | schema_draft |
| 14 | Token-budget bounding with explicit trims (`bound.rs`) | [agent-context-integration.md](agent-context-integration.md) | A1 (bounded-bundle arm); tokenizer is a chars/4 heuristic | not_measured |
| 17 | Cross-tier browser→backend reconstruction in one bundle | [capture/frontend.md](../capture/frontend.md), [capture/correlation.md](../capture/correlation.md) | **A4 correlation on real telemetry** | not_measured |
| 18 | CLI invocation node + run-anchored bundle (`bundle.rs`) | [capture/agent-cli-tracing.md](../capture/agent-cli-tracing.md), [local-first-v1.md](local-first-v1.md) | CLI capture ledger | not_measured |
| 19 | Agent sessions as evidence; edit→deploy→error chain; `agent_edited_failing_file` (`agent.rs`) | [capture/agent-cli-tracing.md](../capture/agent-cli-tracing.md) | Agent-session tracing ledger (`agent_session_linkage`) | not_measured |
| 20 | Ranked, evidence-cited hypotheses + honest `insufficient_evidence` (`hypothesis.rs`) | [causal-reconstruction.md](causal-reconstruction.md) | A1 (hypothesis-quality arm) | not_measured |

## What the PoC deliberately does not prove

No real SDK telemetry (every fixture is hand-written, which the OTLP and Sentry ledgers
explicitly disallow as conformance evidence); no database, network, concurrency, or scale (the
A5/storage gates are untouched — the four-build benchmark rule still owns performance claims); no
real agent runs (agent-session fixtures are invented, so no `agent_session_linkage` evidence); no
real fixer, PR, CI, or merge (all outcome rows are synthetic); redaction is four demonstration
regexes against three seeded canaries — A6's six-stage pipeline, detector toolchain, and canary
corpus remain unbuilt; token counting is a heuristic; the hypothesis rule set is three rules and
a fallback. Determinism itself is partly a PoC artifact (no clock, no I/O) that the product must
re-earn under streaming ingest.

## What the PoC does buy

1. **Executable design review.** Twenty mechanisms from eight design docs run end to end; schema
   shapes, derivation rules, edge semantics, and trim policies survived contact with code.
2. **Contract artifacts.** Draft JSON Schemas other tools can validate against today.
3. **Fixture corpus seed.** Three scenarios (deploy-adjacent backend error, cross-tier
   browser→backend, CLI-run panic) ×4 evidence classes are exactly the shape Phase-0/A1
   hand-assembled bundles need — the eval corpus starts from these, replacing hand-written JSON
   with SDK-generated telemetry.
4. **Convention rehearsal.** The integration-contract conventions (resource attributes, deploy
   events, `parallax.run.id`, trace-ID surfacing) all got exercised by a consumer.

## Graduation path

| PoC kernel | Graduates into | First gate it must pass |
| --- | --- | --- |
| derive/fingerprint | `parallax-server` ingest workers | OTLP conformance L1 (direct Rust SDK fixtures) |
| bundle/bound/hypothesis | Bundler service | A1 Phase-0 eval (hand bundles vs agentic-raw) |
| redact | The real six-stage pipeline | A6 seeded-canary red team |
| deploy/spike/rollup | Detector worker | New Detect trigger ledger (precision/recall on replayed telemetry) |
| budget/dispatch/learn | Dispatcher + Learner workers | Fixer outcome ledger rows |
| agent | Agent-session adapters | Per-tool capture-surface ledger rows |
| schema/ | The A3 versioned schema release | A3 `machine-readable artifacts` level |

The rule stays: dated result rows move claim levels; the PoC moves understanding.
