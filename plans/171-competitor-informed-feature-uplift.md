# Plan 171: Competitor-informed feature uplift — preview-before-save, agent issue lease, MCP evals, instrumented onboarding

> **Executor instructions**: Follow this plan step by step. This plan is
> spec-first: two of its features require decision-record / implementation-
> spec amendments BEFORE code. Run every verification command. On any "STOP
> conditions" item, stop and report.
>
> **Drift check (run first)**: `git diff --stat f6208070..HEAD -- docs/research/decisions/agent-access-surface.md docs/research/decisions/fixer-boundary.md docs/research/architecture/v1-implementation-spec.md ui/graphql/schema.graphql crates/parallax-mcp/ crates/parallax-server/src/alerting/ crates/parallax-api/src/resolvers/alerts.rs crates/parallax-metadata/ crates/parallax-cli/`
> — on mismatch with the excerpts below, STOP.

## Status

- **Priority**: P2
- **Effort**: L (four independent features; each S–M; land separately)
- **Risk**: MED (two features amend product contracts; gated spec-first)
- **Depends on**: 168/170 QA waves already shipped on `main`; f2 remains
  operator-gated. Playwright `alerts-pilot` dataset + `contracts/alerts.spec.ts`
  exist if 170 landed — otherwise add them before f1 step 4.
  recipe as part of this feature; independent of the playground program
  (162–167)
- **Category**: direction
- **Planned at**: parallax `f6208070`, 2026-08-13

## Why this matters

A deep competitor study (Maple, Makisuo/maple @ fc0bd8e, 2026-08-13)
identified four mechanisms that fit Parallax's wedge (agent-ready context
engine) and are small relative to their differentiation value. **License
boundary (binding)**: Maple is FSL-1.1-ALv2 — Parallax is a competing
product, so Maple code may NOT be copied or adapted; only the *ideas*
below, re-specified in Parallax's own terms and implemented independently
against Parallax's own architecture. (Contrast: foglamp, used by plan 172,
is Apache-2.0.) The four:

1. **Alert rule preview-before-save** — evaluate a draft rule over the
   recent window and show the measured series + would-have-fired points
   before the user saves. Kills the #1 alert-authoring failure (blind
   thresholds). Parallax has every ingredient: the alerting measurement
   path already maps rules onto `service_summaries`/`span_red_series`/
   `log_count_series`/`metric_series`
   (`crates/parallax-server/src/alerting/measurement_source.rs`).
2. **Agent issue-lease loop** — MCP/CLI verbs to claim / heartbeat /
   release an issue so an external coding agent can work a queue without
   double-assignment. Directly serves the fixer-boundary outcome loop
   (fixer_outcomes rows exist in Turso).
3. **MCP LLM evals, cost-gated** — a scored eval suite that proves a real
   model picks the right MCP tool with the right args, run only when a PR
   label opts in. Parallax's MCP has projection-equivalence checks
   (`parallax-mcp check`) but nothing proving *tool-selection* quality.
4. **Instrumented onboarding snippets** — zero-data pages ship
   copy-pasteable, framework-specific OTLP setup snippets (Rust/Java/JS
   tabs) instead of a bare endpoint string. (UI shell for this lands with
   plan 172's empty-state work; this plan owns the snippet content
   contract.)

## Current state (verified)

- GraphQL SDL `ui/graphql/schema.graphql`: 76 queries / 14 mutations; alert
  family = `alertRules`, `alertRule`, `alertRuleStates`, `alertIncidents`,
  `alertIncident`, `alertDestinations`, `alertChecks` + save/delete/enable
  mutations. NO preview operation. SDL is generated (Juniper code-first,
  drift-gated by `cargo xtask ui graphql check`); contract changes go to
  `docs/research/architecture/v1-implementation-spec.md` FIRST (§8 owns the
  GraphQL surface), then code.
- Alerting internals: rule model + evaluator + pure state machine
  (`crates/parallax-server/src/alerting/{evaluator,state_machine,measurement,measurement_source}.rs`);
  measurement is already a pure function of (rule, window) — a preview is
  "run measurement + state machine over the draft without persisting".
- MCP: `docs/research/decisions/agent-access-surface.md` — **closed tool
  catalog, exactly 2 read-only tools** (`parallax_issue_context`,
  `parallax_agent_session_show`), local-stdio only. An issue-lease verb set
  is a WRITE surface → REQUIRES amending that decision record (and the
  trust-boundary record
  `docs/research/decisions/agent-trust-boundary-and-prompt-injection.md`
  constraints: agent-context crates may depend only on parallax-evidence +
  parallax-model — lease writes must therefore go through the HTTP API,
  not direct storage deps).
- Fixer outcome contract: `docs/research/decisions/fixer-boundary.md` — PR
  ≠ success; outcome records append-only (`fixer_outcomes` in Turso;
  `crates/parallax-evidence/src/fixer_outcome.rs` state machine). Lease
  verbs must feed this, not bypass it.
- Turso metadata already has `evidence_claim_rows`
  (`crates/parallax-metadata/` table list) and a proven CAS-claim idiom
  (occurrence claim + alert rule claim, concurrency test at
  `crates/parallax-metadata/src/turso/tests.rs:41`).
- Zero-data onboarding today: the copyable OTLP endpoint lives in feature
  pages, NOT the shared empty-state component —
  `ui/src/features/overview/components/overview-page.tsx:845-865` and
  `ui/src/features/issues/components/issues-page.tsx:225-226`;
  `ui/src/shared/console/empty-state.tsx` has no endpoint/copy affordance.
  No per-framework snippets anywhere.
- No eval harness anywhere in the repo (grep `eval` under crates/parallax-mcp → none).

## Commands you will need

| Purpose | Command | Expected |
|---|---|---|
| Gates | `cargo xtask ci --fast && cargo xtask lint && cargo xtask test && cargo xtask arch` | green |
| Structural policy | `cargo xtask policy --only structural` | green after ratchet.toml rows updated for touched Rust files |
| GraphQL drift | `cargo xtask ui graphql check` | no drift (after SDL export) |
| SDL export | `cargo xtask ui graphql export` | regenerates `ui/graphql/schema.graphql` |
| MCP checks | `cargo run -p parallax-mcp -- check --fingerprint <fp>` | exit 0 |
| Docs links | `cargo xtask docs links` | pass |

## Scope

**In scope**: `docs/research/decisions/agent-access-surface.md` +
`fixer-boundary.md` amendments (feature 2),
`docs/research/architecture/v1-implementation-spec.md` §8 (features 1+2
contracts), `crates/parallax-server/src/alerting/` (preview evaluation
path), `crates/parallax-api/src/resolvers/alerts.rs` (preview query),
`crates/parallax-metadata/` (issue lease rows), `crates/parallax-mcp/`
(lease tools + eval harness), `crates/parallax-cli/` (lease verbs),
`ui/graphql/schema.graphql` (generated), alerts UI dialog (preview panel),
`.github/workflows/` (label-gated eval job),
`docs/guide/` snippet content (feature 4).

**Out of scope**: session replay, anomaly detection, K8s/infra pages, AI
investigations, web analytics (all noted as candidate roadmap items —
record in README, do not build); any Maple code adaptation (license); the
empty-state UI component itself (plan 172).

## Git workflow

PR-only `main`; one PR per feature, spec amendment in the SAME PR as its
implementation (spec commit first); `git commit -s`; Conventional Commits;
agent trailer per `COMMITS.md`.

## Steps

### Feature 1 — Alert preview-before-save

1. Spec: add to v1-implementation-spec §8 a read-only query
   `alertRulePreview(input: AlertRuleInput!, windowMinutes: Int): AlertRulePreview`
   returning the measured series (bounded points), per-group would-fire
   markers from the PURE state machine, and sample-count sufficiency —
   explicitly no persistence, no incident writes.
2. Implement: reuse `measurement_source.rs` + `state_machine.rs` on the
   draft input; resolver in `alerts.rs`; enforce existing depth/complexity
   limits; bound series via the metric-summary contract semantics.
3. UI: preview panel inside the existing New-rule dialog
   (`ui/src/routes/alerts.index.tsx` + `features/alerts/model/alert-rule-form.ts`)
   rendering the series + fire markers before Save.
4. Tests: state-machine-level would-fire cases (unit), resolver test vs
   MemoryStore, one contracts-lane Playwright spec (preview renders before
   save — extends plan 170's `contracts/alerts.spec.ts`).

**Verify**: `cargo xtask ui graphql export && cargo xtask ui graphql check`
→ SDL gains exactly the one query; all gates green.

### Feature 2 — Agent issue lease (spec-first, decision gate)

1. Amend `agent-access-surface.md`: propose extending the closed catalog
   with three write-scoped tools `parallax_issue_claim`,
   `parallax_issue_heartbeat`, `parallax_issue_release` (lease semantics:
   TTL lease keyed by fingerprint + agent identity, CAS acquisition,
   heartbeat extends, release records disposition; expired lease
   reclaimable). Respect the trust boundary: tools call the HTTP API; no
   new storage deps in agent-context crates. **The amendment is an
   operator decision — commit the proposal, open the PR, and STOP feature
   2 until the operator approves the decision-record change.**
2. After approval: Turso lease table (numbered migration if plan 169's
   versioning landed; else follow current schema pattern), CAS claim
   modeled on the occurrence-claim idiom + its concurrency test shape;
   GraphQL mutations (spec §8 first); CLI verbs
   (`parallax issue claim|heartbeat|release`); MCP tools registered in the
   closed catalog with wire-budget enforcement like the existing two.
3. Release verb writes a `fixer_outcomes`-compatible disposition row —
   never marks success (fixer-boundary rule: success requires review +
   non-recurrence evidence).
4. Tests: two-concurrent-claimers (exactly one wins), heartbeat extends,
   expiry reclaims, release disposition recorded; MCP projection
   equivalence extended to the new tools.

**Verify**: decision-record PR approved BEFORE code lands; gates green;
`parallax-mcp check` covers the new tools.

### Feature 3 — MCP LLM evals, label-gated

1. New dev-only eval harness at `crates/parallax-mcp/tests/evals/`
   (decision made: colocate with the crate; the tests are `#[ignore]`d and
   run only when `ANTHROPIC_API_KEY` is set, so no network in normal CI;
   if the trust-boundary build gate rejects the dev-dep, STOP condition 4
   moves it to a standalone tools/ crate): scenario table — a user
   prompt + seeded store state → expected tool + expected key args; driver
   calls a real Claude model (claude-sonnet-5 default) with the MCP tool
   schemas and scores tool-choice + arg accuracy. Read the `claude-api`
   skill/plugin docs available in the executor environment for the SDK
   call shape if unsure.
2. CI: separate workflow job triggered ONLY by PR label `run-mcp-evals`
   (cost gate); requires `ANTHROPIC_API_KEY` secret; job absent-key →
   skip with notice, never fail.
3. Baseline: ≥8 scenarios (issue-context happy path, ambiguous fingerprint,
   session-show, wrong-tool distractors). Threshold: ≥7 of 8 scenarios
   must select the right tool with required args (record the constant in
   the harness); failures print the model transcript.

**Verify**: with a key set,
`cargo nextest run -p parallax-mcp --run-ignored only -E 'test(/eval/)'`
→ ≥8 tests run, ≥7 pass per the scoring assert; the workflow file greps
for the label gate:
`grep -n "run-mcp-evals" .github/workflows/<eval workflow>.yml` shows an
`if: contains(github.event.pull_request.labels.*.name, 'run-mcp-evals')`
condition; a PR without the label shows the job skipped in CI.

### Feature 4 — Framework snippet contract

Write `docs/guide/instrument-snippets.md`: canonical, copy-pasteable OTLP
setup snippets for Rust (tracing + opentelemetry-otlp), Java (OTel
javaagent), JS/browser (sdk-trace-web), each pointing at `:4317`/`:4318`
with `service.name` and the conventions doc's required resource
attributes. Source of truth for plan 172's empty-state tabs. Validation
method (no separate test rig): cross-check each snippet against the
known-working equivalents in the sibling playground checkout
`../parallax-telemetry-playground` — Rust:
`libs/playground-telemetry/src/lib.rs` + a service main; Java:
`deploy/Dockerfile.java` agent wiring + `deploy/docker-compose.yml` OTEL
env block; JS: `web/src/telemetry.ts` — and cite those files + their
versions in a "verified <date> against <playground files @ commit>" line
per snippet. If the sibling checkout is absent, shallow-clone
github.com/tailrocks/parallax-telemetry-playground to a temp dir for the
comparison.

**Verify**: `cargo xtask docs links` pass; every snippet carries the
verified line citing concrete playground files + commit.

## Test plan

Per feature above. Feature 2 adds the concurrency suite; feature 1 extends
alerts unit+resolver+Playwright; feature 3 IS a test harness; feature 4 is
doc-verified-by-run.

## Done criteria

- [ ] `alertRulePreview` in SDL, resolver tested, UI preview panel in the
      rule dialog, Playwright spec green.
- [ ] Decision-record amendment for lease tools approved by operator, then
      lease verbs shipped with concurrency tests — or feature 2 explicitly
      parked at the STOP with the proposal PR open.
- [ ] `run-mcp-evals`-labeled PRs run the eval job; unlabeled PRs don't;
      ≥8 scenarios ≥ threshold.
- [ ] `docs/guide/instrument-snippets.md` exists with 3 snippets, each
      carrying a verified line citing playground files + commit.
- [ ] `cargo xtask ci --fast`, `lint`, `test`, `arch`, `policy --only structural` + `ui graphql check` green.
- [ ] `plans/README.md` row updated (also record the rejected/deferred
      Maple-inspired ideas: session replay, anomaly detection, K8s pages,
      AI investigations, web analytics, digest emails).

## STOP conditions

1. Drift check fails.
2. Feature 2 decision amendment not approved — park feature 2, continue
   others.
3. Preview evaluation cannot reuse the measurement path without persisting
   state — report the coupling; do not fork a second measurement impl.
4. Eval harness cannot run without violating the trust-boundary build gate
   (`ratchet.toml` agent_context deps) — report; likely needs to live
   outside the mcp crate.
5. Any implementation step finds you translating Maple source — stop,
   re-derive from the spec you wrote.

## Maintenance notes

- Preview and evaluator share the state machine — future rule-model fields
  must update both spec §8 and the preview resolver in one PR.
- Lease TTLs interact with prune (leases on pruned issues) — the lease
  table must join the prune plan classes.
- Eval scenarios should grow with every MCP tool addition; the label gate
  keeps cost opt-in.
