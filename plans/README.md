# Active Implementation Plans

Execution rules for the complete program live in
[`IMPLEMENTATION.md`](IMPLEMENTATION.md). The self-contained Rust/TypeScript
target lives in [`ENGINEERING-STANDARDS.md`](ENGINEERING-STANDARDS.md). Numbered
plans contain all information required for implementation and never require an
executor to reconstruct how a decision was originally researched. The optional
[`OXC-IMPLEMENTATION.md`](OXC-IMPLEMENTATION.md) contains only official implementation
lookups for refreshing Oxc/TypeScript component status at execution time.
The compact [`GOAL.md`](GOAL.md) brief drives the whole program without
duplicating the numbered plans.

Run the bounded implementation program with:

```text
/goal Follow plans/GOAL.md until its Done condition is mechanically proven.
```

`plans/` is the only home for active Parallax implementation plans. It contains
unfinished work only. Completed, rejected, or superseded work belongs in Git
history and, when durable evidence is useful, under `docs/research/validation/`.

## Lifecycle

1. Use a unique, never-reused numeric ID and a flat
   `plans/NNN-kebab-case.md` path.
2. List only `TODO`, `IN PROGRESS`, or `BLOCKED` files in this index.
3. A plan file contains status metadata, current-state rationale, scope, ordered
   steps, tests, machine-checkable done criteria, STOP conditions, and a
   `Remove When` section.
4. When a plan becomes terminal, record durable evidence if needed, then delete
   its file and index row in the same commit. Do not keep a DONE archive here.
5. Work directly on the single active branch from `AGENTS.md`; commit with DCO
   and exactly one agent-product trailer, then push each durable update.
6. `GOAL.md` is an orchestration brief, not another plan or source of
   architecture. Plan 107 deletes it in the final mechanical closure commit.

The completed historical plan programs were retired on 2026-07-12. Their
closure evidence remains in Git history, not as active plan material.

## Program Constraints

Every plan must preserve these non-negotiable Parallax constraints:

- GreptimeDB + Turso only; no product fallback engine.
- GreptimeDB native raw-signal tables.
- Native TLS only; never an active rustls backend.
- Bun only for JavaScript/TypeScript.
- Decode once and move ownership on the ingest hot path.
- Apache-2.0 throughout.
- One active branch; no per-plan or per-agent branches.

## Execution Preflight (verified live, 2026-07-17)

Facts an executor may rely on without re-deriving; re-verify only on failure:

- **Host**: operator's macOS arm64 machine. Docker 29.4.0 running (16 GB
  RAM, >600 GB free disk) — the playground's 14-container stack fits.
  `mise` 2026.7.7, cargo 1.97.0, bun 1.3.14, cargo-nextest present.
- **Push rights**: the `gh`-authenticated account has admin on
  `tailrocks/parallax`, `tailrocks/parallax-telemetry-playground`, and
  `tailrocks/homebrew-parallax`; direct pushes to `main` succeed (parallax's
  ruleset is bypassed by admin — proven by live pushes 2026-07-17;
  playground has no protection). PR creation + merge via `gh` works.
- **Delivery model (operator, 2026-07-17, final — supersedes the same-day
  branch authorizations)**: everything lands as direct commits to `main` in
  BOTH repositories. No branches, no pull requests, ever, in either repo.
  Push every durable green slice immediately; the parallax ruleset's
  "Bypassed rule violations" push notice is expected. Wave 2 starts only
  after plan 159 completes Wave 1's evidence.
- **Live-engine test lanes**: the real-GreptimeDB tests download and cache
  the engine themselves (`target/greptime-test-bin/`) and are gated behind
  `#[ignore]` — run them with `cargo nextest run --run-ignored all -E
  'binary(/greptime/)'` (or the per-test `cargo test … -- --ignored`
  documented in each test header). A plan's "live engine" verification means
  this shape; a zero-test selection is a command-shape error, not a pass.
- **Browser verification (operator, 2026-07-17): the `agent-browser` CLI is
  the designated tool** — verified installed at
  `/opt/homebrew/bin/agent-browser`, v0.32.1. Before the first browser step,
  run `agent-browser skills get core --full` and follow its patterns. The
  command surface covers everything the plans require: `open`/`snapshot`/
  `click`/`find` for walkthroughs, `screenshot [path]` for evidence,
  `console` + `errors` for the clean-console checklist item,
  `set viewport 1440 900` / `set viewport 375 812` for the layout checks,
  `set media dark|light [reduced-motion]` for theme/motion checks,
  `diff screenshot --baseline` for visual comparisons, and
  `record start|stop` for flow captures. Chrome DevTools MCP is the fallback
  surface only if the CLI is unavailable. Plans 157/159/160/162-168 require
  this evidence; if browser tooling is unavailable, that verification step
  is blocked — do not fake screenshots.
- **No external credentials are required for Waves 1-2.** Optional only:
  a real Slack webhook URL for plan 167's live-delivery test (a local HTTP
  listener otherwise suffices, as the plan specifies). The jackin
  repository/PR #793 is NOT a dependency — the playground simulates the
  contract.
- **No operator-gated plans remain.** The Operator Unblock Directive below
  (2026-07-17) resolved or delegated every pending decision gate and
  rescoped every external block into actionable work. Nothing in the index
  waits on the operator.

## Operator Unblock Directive (2026-07-17)

The operator directed: nothing in this program may sit in an operator-gated
BLOCKED state. Every pending decision is hereby decided or delegated, and
every externally-blocked item is rescoped into actionable work. This section
is authoritative over any stale `Status: BLOCKED` line still inside an
individual plan file — executors treat the index status as current and
update the plan file's status line when they first touch that plan.

Fixed decisions (approver: the operator, alexey@chainargos.com, 2026-07-17):

- **Plan 104**: Option C — versioned envelope around the shipped V1 dossier
  (explicit `contract_version`, compatibility window one minor version,
  migration = envelope-wrap without rewriting stored bundles). Executor
  fills `approved_by`/`approval_date` accordingly.
- **Plan 108**: no destructive history rewrite, ever; scan history, rotate
  any found credential, record evidence.
  **Plan 108 CLOSED (2026-07-17):** scan evidence in
  `docs/research/security/credential-history-scan-2026-07-17.md` — no real
  exposure; no rewrite; optional lab DSN regen only if compose revived.
- **Plan 116**: adopt the plan's own proposed lifecycle contract verbatim as
  the approved contract; executor writes the decision record.
- **Plans 109/115**: V2 auth + server scope opened, minimal recommended
  shapes. Plan 109 minimal slice DONE/retired (2026-07-17); plan 115 residual
  server profile remains.
- **Plans 112/118/120/121/123/124**: scopes opened with defaults — product
  MCP proceeds behind its evidence gates (local transport until 109);
  Sentry-compatible ingest open; first agent-session adapter = Claude Code;
  deploy/change provider = GitHub; CI provider = GitHub Actions; fixer opens
  after its gates.
- **Plan 089**: rescoped to a fix-forward upstream contribution
  (native-TLS/plaintext feature in `greptimedb-ingester`).
- **Plan 128**: rescoped — re-validate on the current toolchain; persistent
  third-party declaration failures get documented shrink-only exceptions
  instead of an indefinite wait.
- **Plan 154**: multi-backend arm runs self-hosted (no external
  credentials), one backend at a time.

Delegation rule for any FUTURE decision gate discovered mid-execution: adopt
the plan's recommended or most conservative option, write the named decision
record stamped with this directive, and proceed — never stall waiting for
the operator. Hard limits that survive this directive: no destructive
history rewrites, no rustls, no engine substitutions, no gate weakening, no
fabricated evidence. A plan may still be marked BLOCKED mid-run ONLY for a
hard external fact (upstream bug, unreachable service), with fresh
reproducible evidence, and work continues elsewhere.

## Active Plans

### Storage

Plan 125 DONE (2026-07-17): drop legacy native fingerprint column
([evidence](../docs/research/validation/2026-07-plan-125-fingerprint/README.md)).


Plan 116 DONE (2026-07-17): retention contract + deterministic prune CLI
([evidence](../docs/research/validation/2026-07-plan-116-retention-prune/README.md)).


| Plan | Title | Priority | Effort | Depends on | Status |
|------|-------|----------|--------|------------|--------|
| [089](089-extension-table-grpc-writes.md) | Move derived extension-table writes to GreptimeDB's row API | P2 | M | upstream `greptimedb-ingester` native-TLS/plaintext feature fix | BLOCKED — crates.io still 0.18.0; upstream PR #58 OPEN not merged (recheck 2026-07-17T13:06Z); HTTP SQL path remains |

### Foundation And Delivery

Plan 102 DONE (2026-07-17): four-target preview `0.1.0-preview.1295+e37a65d`
passed exact-SHA `release-verify` + tap pull
([evidence](../docs/research/validation/2026-07-13-plan-102-release-baseline.md)).

### Quality Tooling And Rust

| Plan | Title | Priority | Effort | Depends on | Status |
|------|-------|----------|--------|------------|--------|

### TypeScript Toolchain, Architecture, Boundary, And Test Foundations

Plan 128 DONE (2026-07-17): strictest-passing TS7 + shrink-only libcheck exceptions
([evidence](../docs/research/validation/2026-07-plan-128-static-safety/README.md)).
Plan 129 DONE (2026-07-17): macOS forced-Bun Vitest dual-run + matrix ownership
([evidence](../docs/research/validation/2026-07-plan-129-macos-vitest/README.md)).

| Plan | Title | Priority | Effort | Depends on | Status |
|------|-------|----------|--------|------------|--------|
| 152 | Establish the generated GraphQL contract foundation | P1 | L | 095, 100, 101, 128, 129, 130 | DONE (2026-07-17) — [evidence](../docs/research/validation/2026-07-plan-152-graphql-contract/README.md) |
| 153 | Establish non-GraphQL runtime boundary foundations | P1 | L | 095, 100, 101, 128, 129, 130 | DONE (2026-07-17) — [evidence](../docs/research/validation/2026-07-plan-153-runtime-boundaries/README.md) |
| 145 | Prove critical UI flows against managed GreptimeDB and isolated Turso | P1 | L | 093, 101, 132, 144 | DONE (2026-07-17) — [evidence](../docs/research/validation/2026-07-plan-145-playwright-real-stack/README.md) |
| 146 | Establish cross-browser, mobile, accessibility, and visual Playwright gates | P1 | L | 101, 132, 144, 145 | DONE (2026-07-17) — [evidence](../docs/research/validation/2026-07-plan-146-playwright-breadth/README.md) |

### TypeScript Capability And Feature Migrations

Plans 152 and 153 establish GraphQL and non-GraphQL runtime boundaries after the
layer graph. Plan 149 DONE (2026-07-17): route-less capability facades
Plan 134 DONE (2026-07-17): investigations feature facade
Plan 142 DONE (see table)
Plan 143 DONE (2026-07-17): app/layout/shell facades
Plan 142 DONE (2026-07-17): traces feature facade

(runtime-metrics, story, time-range, page-header). Plans 134-142 and 150 are split by product owner for parallel work.
Plans 139, 140, and 142 wait for plan 134's public pin facade; plan 140 also
waits for plan 141's public logs facade. Plan 143 moves app/layout/shell only,
and plan 151 verifies zero residual architecture debt without absorbing product
work.

| Plan | Title | Priority | Effort | Depends on | Status |
|------|-------|----------|--------|------------|--------|
| 149 | Establish route-less UI capabilities before feature moves | P1 | L | 100, 129, 152, 153 | DONE (2026-07-17) — [evidence](../docs/research/validation/2026-07-plan-149-route-less-capabilities/README.md) |
| 134 | Migrate investigations behind a strict feature facade | P1 | L | 100, 129, 132, 144, 145, 146, 149, 152, 153 | DONE (2026-07-17) — [evidence](../docs/research/validation/2026-07-plan-134-investigations/README.md) |
| 138 | Move services into one bounded feature | P1 | L | 100, 129, 132, 144, 145, 146, 149, 152, 153 | DONE (2026-07-17) — [evidence](../docs/research/validation/2026-07-plan-138-services/README.md) |
| 139 | Move issues and stacktrace ownership into one feature | P1 | L | 100, 129, 132, 134, 144, 145, 146, 149, 152, 153 | DONE (2026-07-17) — [evidence](../docs/research/validation/2026-07-plan-139-issues/README.md) |
| 141 | Move logs and the reusable log table into one feature | P1 | L | 100, 129, 132, 144, 145, 146, 149, 152, 153 | DONE (2026-07-17) — [evidence](../docs/research/validation/2026-07-plan-141-logs/README.md) |
| 140 | Move runs/invocations, sessions, and live observation into one feature | P1 | L | 100, 129, 132, 134, 141, 144, 145, 146, 149, 152, 153 | DONE (2026-07-17) — [evidence](../docs/research/validation/2026-07-plan-140-invocations/README.md) |
| 142 | Move trace search, analysis, and inspection into one feature | P1 | XL | 100, 129, 132, 134, 144, 145, 146, 149, 152, 153 | DONE (2026-07-17) — [evidence](../docs/research/validation/2026-07-plan-142-traces/README.md) |
| 150 | Move overview into one bounded feature | P1 | L | 100, 129, 132, 144, 145, 146, 149, 152, 153 | DONE (2026-07-17) — [evidence](../docs/research/validation/2026-07-plan-150-overview/README.md) |
| 143 | Move app, layout, and shell behind explicit boundaries | P1 | XL | 134, 135, 136, 137, 138, 139, 140, 141, 142, 145, 146, 149, 150, 152, 153 | DONE (2026-07-17) — [evidence](../docs/research/validation/2026-07-plan-143-app-layout-shell/README.md) |
| 151 | Verify and close the final UI architecture | P1 | L | 134, 135, 136, 137, 138, 139, 140, 141, 142, 143, 149, 150, 152, 153 | DONE (2026-07-17) — [evidence](../docs/research/validation/2026-07-plan-151-ui-architecture-closure/README.md) |

### UI State, Performance, And Product Gaps

| Plan | Title | Priority | Effort | Depends on | Status |
|------|-------|----------|--------|------------|--------|
| 133 | Replace the UI TTL cache with feature-owned TanStack Query | P1 | L | 095, 101, 128, 129, 132, 144, 145, 151 | DONE (2026-07-17) — [evidence](../docs/research/validation/2026-07-plan-133-tanstack-query/README.md) |
| [147](147-ui-live-data-performance.md) | Make live telemetry updates typed, bounded, and identity-stable | P1 | L | 095, 101, 129, 133, 140, 141, 142, 145, 151 | IN PROGRESS — 133 closed; live boundedness |
| [148](148-ui-bundle-performance.md) | Enforce route-owned production chunks and deterministic bundle budgets | P1 | L | 095, 100, 101, 105, 132, 133, 144, 146, 147, 151 | TODO — waits on 133/147 |

### Dependencies, Tests, And Performance

| Plan | Title | Priority | Effort | Depends on | Status |
|------|-------|----------|--------|------------|--------|
| [103](103-property-fuzz-and-performance.md) | Residual Rust/UI property, fuzz, and performance gates | P2 | M | 133, 147, 148 (UI); scheduled samples (ratchets) | IN PROGRESS — Rust properties/fuzz/benches/CI lanes landed; residual trace-tree/serialization/retry properties, UI properties, variance ratchets |

### Evidence Contracts And Closure

Plan 104 DONE (2026-07-17): Option C bundle-v2 envelope
([evidence](../docs/research/validation/2026-07-plan-104-bundle-v2/README.md)).
Plan 106 DONE (2026-07-17): evidence pins (sanitized bundle-v2 in Turso)
([evidence](../docs/research/validation/2026-07-plan-106-evidence-pinning/README.md)).
Plan 111 DONE (2026-07-17): source-aware redaction + A6 public-safe canaries
([evidence](../docs/research/validation/2026-07-plan-111-a6/README.md)).


| Plan | Title | Priority | Effort | Depends on | Status |
|------|-------|----------|--------|------------|--------|
| [107](107-program-closure-audits.md) | Run independent source audits and verify the mechanical closure commit | P1 | M | Every other actionable indexed plan; all blockers freshly rechecked | TODO — runs last, after every other actionable plan (unblock directive 2026-07-17) |

### Unified CLI Observability (operator, 2026-07-17)

Operator directive: jackin❯'s `feature/unified-otel-observability` cutover
removed `parallax.run.id` and every `jackin.*` key upstream (its plan 013 is
DONE; its PR #793 is open). Parallax drops its vendor correlation attribute
and becomes a generic CLI-application observability platform keyed on
`cli.invocation.id`/`session.id` with `app.mode`, `cli.command.name`, `ui.*`
screen/action events, `background.cycle` roots, PRODUCER/CONSUMER `job.id`
traces, `gen_ai.*` conversations, and bounded `outcome`/`error.type`.
Delivery (operator, 2026-07-17, final): direct commits to `main` in BOTH
repositories — no branches, no pull requests. Parallax side = plans 156,
157, 160 (+159 evidence); playground side = plans 158, 161. This vertical is
deliberately independent of the formerly-blocked
128→151 chain: it builds on current UI conventions; plan 140 later migrates
the new surface behind a feature facade.

Second operator directive (2026-07-17), binding on every executor:
- **No legacy support**: `parallax.run.id` (and every `parallax.*`/vendor
  telemetry key) is removed entirely — not read, not written, not COALESCEd.
- **Generic attributes only**: Parallax business functionality exists only
  over generic keys (`cli.*`, `session.*`, `app.*`, `ui.*`, `job.*`,
  `gen_ai.*`, standard semconv). Application-specific attributes are
  display-only opaque data in generic attribute views — never special-cased
  in queries, resolvers, or UI logic.
- **Browser-verified features**: every implemented UI feature is verified in
  a real browser against live playground data before the next step (plan
  157's six-item protocol: data correctness, links, states, layout,
  live behavior, clean console). Known display defects — span rendering
  inside traces foremost — were audited and fixed by plan 160 (DONE,
  2026-07-17) against the plan-161 corner-case corpus; the full grid,
  defect records, and conformance sweep live in
  [docs/research/validation/2026-07-unified-cli-observability/ui-defect-ledger.md](../docs/research/validation/2026-07-unified-cli-observability/ui-defect-ledger.md).

Contract reconciliation for existing plans (binding on their executors):
every `parallax.run.id` / `runId` / `$runId` / `runs`-field reference in
plans 105, 140, 141, 142, 147, 154, and 155 must be read as the plan-156
contract (`cli.invocation.id` → `invocationId`, `/invocations/$invocationId`,
`invocation_metric_points`, `session.id` sessions, renamed GraphQL fields).
Plans 140-142 and 147 re-characterize their baselines after 156/157 land
(their drift checks will fire — that drift is expected and resolved against
the new surface, not reverted). Plan 155's "session = `parallax.run.id`"
statements become `session.id`/`cli.invocation.id`; its result key becomes
`(test_variant_key, cli.invocation.id, attempt)`. Plan 105's undecided
`parallax metrics --run` contract must be authored against `--invocation`.
Plan 154's remaining sweep consumes the plan-158 emitter contract.

Wave 1 (plans 156, 157, 158, 159, 160, 161) is COMPLETE (2026-07-17). The
closing evidence — 27 green GraphQL assertions, the coverage matrix, and
thirteen browser captures with a clean console — lives in
[docs/research/validation/2026-07-unified-cli-observability/README.md](../docs/research/validation/2026-07-unified-cli-observability/README.md).

### Wave 2 — Maple-Informed UI Evolution (operator /improve directive, 2026-07-17)

Reference analysis of `github.com/MapleTechLabs/maple` (deep /improve run,
2026-07-17; three-agent survey of its web UI, design system, and query
layer). Wave 2 executes AFTER Wave 1 (156-161) completes plan 159's
evidence, as direct commits to `main` in both repositories (delivery model
in Execution Preflight — no branches, no PRs), with the playground gaining
the new scenarios
(`f-attrs`, `l-patterns`, `eco-external`, `a-breach-*`, `a-recover`,
`m-labels`). Non-interactive default selection: the seven highest-leverage
adoptions were planned; the deferred list below records what was
consciously not planned. Every plan verifies in the browser against the
playground per the `ui/AGENTS.md` checklist (installed by plan 162), and
respects the generic-attributes-only invariant — no vendor-specific
inference (Maple's Hyperdrive/PlanetScale logic is explicitly not copied).

| Plan | Title | Priority | Effort | Depends on | Status |
|------|-------|----------|--------|------------|--------|
Plans 162 and 163 are DONE (2026-07-17). 162: tokens, colors lib,
ServiceDot, severity ramp, percentile chart tokens, numerals, motion, and
the codified rules + checklist in `ui/AGENTS.md`
([evidence](../docs/research/validation/2026-07-wave2/162/README.md)).
163: viewport reducer + gesture grammar, minimap controller, color-by with
URL round-trip, self-time, and the Flame tab
([evidence](../docs/research/validation/2026-07-wave2/163/README.md)).

Wave 2 (plans 162–168) is DONE (2026-07-17). 164: faceted filters + where editor
([evidence](../docs/research/validation/2026-07-wave2/164/README.md)).
165: Drain logPatterns + Patterns UI
([evidence](../docs/research/validation/2026-07-wave2/165/README.md)).
166: ELK + React Flow service map
([evidence](../docs/research/validation/2026-07-wave2/166/README.md)).
167: alerting v1 + live webhook
([evidence](../docs/research/validation/2026-07-wave2/167/README.md)).

168 is DONE (2026-07-17): metricCatalog + metricQuery explorer
([evidence](../docs/research/validation/2026-07-wave2/168/README.md)).

Considered and deferred (recorded so they are not re-audited): session
replay / rrweb studio (large new subsystem; browser corpus first),
AI triage + chat + MCP tool catalog page (product MCP is operator-gated by
plan 112), anomaly detection (needs alerting v1 baseline first),
org/multi-tenancy + billing surfaces (V2 auth expansions beyond minimal
  slice; plan 115 / future RBAC), Expo mobile
app, demo-seed onboarding (playground covers it), ReactFlow adoption (the
hand-rolled SVG renderer stays; only ELK layout is adopted), Maple's
mono-as-body typography and amber theme (Parallax keeps its identity),
Apdex/SLO pages (revisit after 167 proves value).

### Cross-Repository Playground And Test Reporting

Plan 154 is the operator-directed (2026-07-14) expansion program for the
companion `parallax-telemetry-playground` repository: correctness fixes,
Juniper GraphQL + gRPC matrix completion, per-service test coverage, and
OpenTelemetry test-run visibility (nextest/JUnit 5/Playwright). Plan 122
keeps historical-residual reconciliation and must classify overlapping rows
as owned by 154. Plan 155 is the Parallax-side product surface consuming the
154 W4 payload: a dedicated Tests page with a stable test-identity registry,
attempt chains, failed-vs-broken taxonomy, a flaky state machine, and
failure fingerprints shared with production issues (evidence:
`docs/research/market/test-reporting-ecosystem.md`). Plan 124 keeps
CI-provider API collection; 155 consumes OTLP-ingested telemetry only.

| Plan | Title | Priority | Effort | Depends on | Status |
|------|-------|----------|--------|------------|--------|
| [154](154-playground-capability-and-test-observability.md) | Live multi-backend fan-out acceptance residual | P1 | M | Self-hosted Maple/SigNoz/OpenObserve/Sentry (Parallax arm → plan 159 DONE) | BLOCKED — multi-backend arm only; one self-hosted external at a time on 16 GB host |
| [155](155-test-reporting-surface.md) | Test reporting surface residual | P1 | XL | 149, 152, 153, 140 DONE; soft 121/124 | IN PROGRESS — explorer+testCases GraphQL landed; residual UI/flaky job/adapters/e2e |

### Triggered Or Residual Work

Unfinished residuals only. Operator unblock (2026-07-17) opened scopes; hard
external facts still BLOCKED where noted. Plan 102 and plan 109 retired
2026-07-17 (release four-target proof; minimal auth + context).

| Plan | Depends on | Trigger / residual | Status |
|------|------------|--------------------|--------|
| [110](110-server-profile-ingest-concurrency.md) | 115 + saturation packet | Measured single-worker bottleneck on supported profile | BLOCKED on 115 profile + measurements |
| [112](112-product-mcp-ship-gates.md) | 099, 104, 111 | Claimed-client fixtures, OTel export verify, spike graduation (oversized summary landed) | IN PROGRESS — local-stdio GO; residual ship gates |
| [114](114-retire-legacy-spool-reader.md) | Stable raw-frame release cycle + expired legacy segments | Remove NDJSON reader after cycle | BLOCKED — only rolling `preview` tag (recheck 2026-07-17T13:06Z) |
| [115](115-v2-server-profile.md) | Auth contract + release pipeline (102/109 DONE) | Validated config + rehearsals + load packet (ADR landed) | IN PROGRESS — ADR in decisions/v2-server-profile.md |
| [118](118-sentry-envelope-migration-adapter.md) | 093, 099, 104, 111, 116 | Real SDK fixtures, cross-source identity, bundle/redaction, live gates | IN PROGRESS — parser + HTTP + event-id ledger landed |
| [120](120-agent-session-capture-adapters.md) | 099, 104, 111, 119 | Success-path fixtures, storage/API/UI, consent CLI, loss ledger | IN PROGRESS — Claude Code pure normalizer landed |
| [121](121-deploy-and-change-context-collectors.md) | 099, 104, 111, 116 | Backfill, doctor, claim ledger (HTTP + delivery idempotency landed) | IN PROGRESS — webhook + Turso durable path landed |
| [122](122-playground-residual-program.md) | 105, 151 (111/119 DONE) | Disposition table + retained scenarios only | BLOCKED on 105 + 151 |
| [123](123-fixer-outcome-loop.md) | 120, 121 residual | Offline outcome harness; fixer separate from core | BLOCKED on 120/121 |
| [124](124-ci-and-flaky-test-evidence-collector.md) | 121 durable path (landed) | GHA read-only collect + flaky multi-attempt evidence | TODO — unblocked |

## Dependency Order

The main restructuring path is:

```text
093 -> 094 -> 095 -> 127 -> 096 -> 097
094 + 095 + 096 -----------------------> 101
097 + 101 -----------------------------> 126 -> 099 -> 104 -> 111
095 + 101 -----------------------------> 131 -> 128
094 + 101 -----------------------------> 130
094 + 101 + 128 ----------------------> 129
095 + 101 + 128 + 129 ----------------> 100
095 + 100 + 101 + 128 + 129 + 130 ----> 152, 153
094 + 101 + 128 + 129 ----------------> 132 -> 144
093 + 101 + 132 + 144 ----------------> 145
101 + 132 + 144 + 145 ----------------> 146
100 + 129 + 152 + 153 ----------------> 149
100 + 129 + 132 + 144 + 145 + 146 +
  149 + 152 + 153 --------------------> 134, 135, 136, 137, 138, 141
100 + 129 + 132 + 134 + 144 + 145 +
  146 + 149 + 152 + 153 --------------> 139, 142
100 + 129 + 132 + 134 + 141 + 144 +
  145 + 146 + 149 + 152 + 153 --------> 140
100 + 129 + 132 + 144 + 145 + 146 +
  149 + 152 + 153 --------------------> 150
134..142 + 145 + 146 + 149 + 150 +
  152 + 153 --------------------------> 143
134..143 + 149 + 150 + 152 + 153 ----> 151
095 + 101 + 128 + 129 + 132 +
  144 + 145 + 151 --------------------> 133
095 + 101 + 129 + 133 + 140 + 141 +
  142 + 145 + 151 --------------------> 147
095 + 100 + 101 + 105 + 132 + 133 +
  144 + 146 + 147 + 151 --------------> 148
097 + 099 + 133 + 151 ----------------> 105
094 + 096 + 101 ------------------------------> 102 (DONE 2026-07-17)
097 + 099 + 101 + 104 + 133 + 147 + 148 -----> 103
095 + 099 ------------------------------------> 113
095 ------------------------------------------> 117
095 + 096 + 100 + 101 + 126 ----------------> 119
093 + 097 + 099 ------------------------------> 116
092 + 104 + 116 ------------------------------> 106
093 + 097 + 099 + 104 ------------------------> 125
auth contract + release proof (109/102 DONE) -> 115
099 + 113 + 115 ------------------------------> 110
099 + 104 + 111 -- operator trigger ----------> 112
093 + 099 + 104 + 111 + 116 -- operator ------> 118
099 + 104 + 111 + 119 -- operator ------------> 120
099 + 104 + 111 + 116 -- operator ------------> 121
105 + 151 (111/119 DONE) -- cross-repo -------> 122
104 + 111 + 120 + 121 -- operator ------------> 123
099 + 104 + 111 + 121 -- operator ------------> 124
all actionable plans --------------------------> 107

156 -------------------------------------------> 157, 158
158 -------------------------------------------> 161
156 + 157 + 161 -------------------------------> 160
156 + 157 + 158 + 160 + 161 -------------------> 159

Wave 2 (after 156-161 merge):
159 (Wave 1 merged) ---------------------------> 162
160 + 162 -------------------------------------> 163
156 + 162 -------------------------------------> 164, 166, 167
162 + 164 -------------------------------------> 165, 168
```

The 156→159 vertical (direct-to-main in both repositories)
runs independently of the 128 chain. Plans 105, 140, 141, 142, 147, 154, and
155 additionally consume the plan-156 contract (see the Unified CLI
Observability section note); their own dependency rows are unchanged.

Plan 092 can run in parallel with 093. After 095, Rust test/module extraction
removes structural blockers before the strict lint baseline. After 096,
Cargo/Bun dependency policy can proceed while the model/crate path starts.

After 101 records the two narrow Oxc exceptions, plans 130 and 131 can prepare
their isolated parity evidence in parallel, but their shared manifest/lock
cutovers serialize. Plan 128 tightens declaration and static safety on the final
compiler/linter stack. Plan 094 first makes Vitest genuinely Bun-run;
plan 129 then creates deterministic characterization and the durable risk
matrix. Plans 100 and 132 can proceed in parallel after 129, with shared
ratchet/config writes serialized: plan 100 establishes source architecture and
plan 132 proves the exact macOS/Linux no-Node Playwright matrix. After Plan 100
and the Oxfmt cutover, Plans 152 and 153 establish GraphQL and non-GraphQL
runtime boundaries in parallel while serializing their shared policy/ratchet writes.

Plan 144 adds the fixture-backed contract/CI extension point. Plan 145 adds the
real-stack project before plan 146 adds browser/accessibility/visual breadth;
they are separate rollback units but serialize shared config and CI files. Plan
149 follows both runtime-boundary foundations while the browser chain runs;
every feature move waits for the browser, boundary, and capability foundations.

Plans 134-138, 141, and overview plan 150 can then run concurrently. Only their
feature-local files are parallel; matrix, ratchet, route-tree, and shared
registry updates use the serialized handoff in each plan. Plans 139 and 142
consume plan 134's pin facade, while plan 140 consumes both the pin and plan
141 logs facades. Plan 143 then moves app/layout/shell. Plan 151 performs only
the zero-debt graph, test, ratchet, matrix, and documentation closure.

Behavior changes remain separate and ordered against a stable graph: plan 133
replaces only cache ownership; plans 105 and 147 then change disjoint metric and
live-data behavior; plan 148 records/optimizes bundles only after all three known
graph changes. Plan 103 consumes the final cache/metric/live/bundle owners for
broader property, fuzz, and performance evidence. Prettier and ESLint are
interim baselines, not permitted final tools.

Plan 107 is last. Any BLOCKED plan does not block closure only while a fresh
reproducible external/operator/phase condition still holds and its file contains
no hidden actionable work.

## Shared Verification

Each plan has narrower commands. The final program baseline is:

```text
git diff --check
git diff --cached --check
cargo fmt --all --check
cargo check --locked --workspace --all-targets
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo xtask dependencies --all
cargo xtask ui graphql check
cargo xtask policy --only ui.runtime-boundaries
cargo nextest run --locked --workspace --all-targets --profile ci
cargo test --locked --workspace --doc
cargo xtask ci --full
cd ui && bun ci
cd ui && bun run check
cd ui && bun run lint
cd ui && bun run typecheck
cd ui && bun run --bun test:ci
cd ui && bun run build
cd ui && bun run test:browser
cd ui && bun run test:browser:cross
cd ui && bun run test:browser:a11y
cd ui && bun run test:browser:visual
cd ui && bun run test:browser:full
cd ui && bun run perf:live
cargo xtask ui-bundle analyze
cargo xtask ui-bundle build-twice
mise exec -- actionlint
```

The plain outer `bun run <name>` interface is intentional. Plan 094 makes each
script Bun-only through checked-in `bunfig.toml` (`[run] bun = true`, install
auto-fetch disabled) and exact lock-local commands; plan 101 fixtures executable
ancestry so Node shebangs, mutable `@latest`, and implicit installs fail.

CI source hygiene uses the event's validated base/head range; the two plain
commands above cover local unstaged and staged changes respectively.

The default Rust commands must work from a clean checkout without `ui/dist`.
The dedicated embed partition builds the UI before compiling `embed-ui`.
Long-running engine commands must narrate progress and finish with the required
ready banner.

## Trigger Ledger Without A Plan

These observations are not currently executable work. Reopen them as numbered
plans only when the trigger becomes true:

| Observation | Reopen trigger |
|-------------|----------------|
| `is_missing_table` / `is_missing_column` use substring matching | A GreptimeDB upgrade breaks conformance or exposes structured errors |
| Pre-commit hooks / `.editorconfig` are absent | Operator selects a repository-wide local-hook policy |
| Bench compose pins lag current engines | The next required four-build benchmark run |
| Native log schema may drift | Every GreptimeDB engine upgrade; compare a fresh native `SHOW CREATE TABLE` before release |
| Old native-table indexes need backfill | A supported old install shows unindexed SST query regression; live-test `ADMIN build_index_table` first |
| Trace native table defaults to 16 partitions | A supported at-scale profile exists; rerun the 1-vs-16 partition harness before changing fresh-table hints |
| Raw forward leg is HTTP-only | GreptimeDB restores a supported native OTLP gRPC ingest endpoint with a native-TLS-compatible client path |
| Profiles are not ingested | GreptimeDB ships a native OTLP profile table/path and the operator opens profile scope |
| ExponentialHistogram support | The signal appears in supported SDK traffic and Greptime native handling is verified |
| Broader newtype rollout | The single ID pilot proves value without wire/persistence churn |
| Stable Homebrew formula mutation | Stable-release readiness is explicitly opened |
| External broker / Iggy | A supported server profile proves the current spool/in-process design cannot meet an approved replay/isolation SLO and the operator opens broker scope |

Accepted decisions such as native tables, no rustls, no Node, no docs site,
and no automatic update branches are repository policy, not unfinished plans.

## Findings Considered And Rejected (2026-07-17 restructuring)

- **Keep `parallax.run.id` in any form — primary key, translation shim, or
  read-only fallback** — rejected: jackin❯'s cutover already deleted it
  upstream, and the operator's second directive (2026-07-17) removes support
  entirely; a legacy-only emitter is unsupported by design (plan 159 asserts
  the negative).
- **GraphQL subscriptions for real-time views** — rejected: SSE is a
  deliberate architecture decision (`crates/parallax-server/src/live.rs:6`);
  the real-time toggle rides the existing SSE + poll patterns.
- **Collector-side attribute translation (rewrite legacy keys at ingest into
  the new key)** — rejected: mutating telemetry at ingest violates the
  decode-once/zero-copy hot-path rule; read-path COALESCE achieves the same
  compatibility without rewriting data.
- **Gating the CLI-observability product surface on the 128→151 UI
  architecture chain** — rejected: 128 is blocked on an external TypeScript 7
  declaration issue with no ETA; the vertical lands on current conventions
  and plan 140 (retitled at execution) migrates it later.
- **Building the invocation index as a new GreptimeDB table** — rejected:
  raw-signal tables are a hard STOP per
  `docs/research/decisions/native-otel-tables.md`; invocations are query-time
  projections over native tables plus Turso product state.
