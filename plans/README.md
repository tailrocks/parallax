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

## Active Plans

### Storage

| Plan | Title | Priority | Effort | Depends on | Status |
|------|-------|----------|--------|------------|--------|
| [089](089-extension-table-grpc-writes.md) | Move derived extension-table writes to GreptimeDB's row API | P2 | M | upstream `greptimedb-ingester` native-TLS/plaintext feature fix | BLOCKED: `greptimedb-ingester` hard-enables rustls through tonic `tls-ring` |
| [125](125-native-trace-fingerprint-deviation.md) | Resolve the unpopulated native trace fingerprint deviation and migration contract | P2 | M | 093, 097, 099, 104 | TODO |

### Foundation And Delivery

| Plan | Title | Priority | Effort | Depends on | Status |
|------|-------|----------|--------|------------|--------|
| [094](094-ci-and-security-foundation.md) | Repair CI topology, source formatting enforcement, advisory gating, permissions, and security policy | P1 | L | 093 | IN PROGRESS |
| [102](102-deterministic-release-pipeline.md) | Unify deterministic preview/stable packaging, symbols, and release verification | P1 | L | 094, 096, 101 | TODO |

### Quality Tooling And Rust

| Plan | Title | Priority | Effort | Depends on | Status |
|------|-------|----------|--------|------------|--------|
| [095](095-quality-control-plane.md) | Add xtask, architecture policy, one ratchet source, facades, and machine-readable diagnostics | P1 | L | 094 | TODO |
| [127](127-rust-test-architecture.md) | Separate Rust test bodies and enforce public/private test ownership | P1 | L | 095 | TODO |
| [096](096-rust-toolchain-and-lints.md) | Pin latest stable Rust and activate a strict measured lint/suppression baseline | P1 | L | 095, 127 | TODO |
| [117](117-documentation-link-integrity.md) | Add a parser-backed required internal Markdown link gate | P2 | S-M | 095 | TODO |
| [119](119-semconv-registry-codegen.md) | Generate checked-in Rust/Java/TypeScript semantic-convention constants from one registry | P2 | M | 095, 096, 100, 101, 126 | TODO |

### Architecture And Boundaries

| Plan | Title | Priority | Effort | Depends on | Status |
|------|-------|----------|--------|------------|--------|
| [097](097-model-test-support-and-dependency-direction.md) | Extract model/port/test-support foundations and remove the core-storage inversion | P1 | XL | 096, 127 | TODO |
| [126](126-rust-workspace-decomposition.md) | Decompose Rust into domain, port, adapter, and composition crates | P1 | XL | 097, 101 | TODO |
| [098](098-facades-modules-and-api-batching.md) | Seal crate facades, split responsibility hotspots, validate crate docs, and eliminate latent nested-field N+1 paths | P2 | L | 126 | TODO |
| [099](099-boundary-errors-idempotency-and-agent-safety.md) | Add typed errors, explicit retry/idempotency boundaries, an ID pilot, and agent-surface safety | P1 | L | 097, 098 | TODO |

### TypeScript Toolchain, Architecture, Boundary, And Test Foundations

| Plan | Title | Priority | Effort | Depends on | Status |
|------|-------|----------|--------|------------|--------|
| [130](130-oxfmt-cutover.md) | Replace Prettier with exact-pinned Oxfmt and built-in Tailwind formatting | P1 | M | 094, 101 | TODO |
| [131](131-typescript-7-and-type-aware-oxlint.md) | Adopt TypeScript 7 and replace ESLint with native/type-aware Oxlint | P1 | L | 095, 101 | TODO |
| [128](128-typescript-static-and-runtime-safety.md) | Enforce strict TypeScript static safety | P1 | L | 095, 101, 131 | TODO |
| [129](129-frontend-test-architecture.md) | Build deterministic forced-Bun Vitest characterization and separated test ownership | P1 | L | 094, 101, 128 | TODO |
| [100](100-ui-feature-architecture.md) | Establish the TypeScript layer graph, ownership ledger, facades, and placement policy | P1 | L | 095, 101, 128, 129 | TODO |
| [152](152-graphql-contract-foundation.md) | Establish the generated GraphQL contract foundation | P1 | L | 095, 100, 101, 128, 129, 130 | TODO |
| [153](153-runtime-boundary-foundation.md) | Establish non-GraphQL runtime boundary foundations | P1 | L | 095, 100, 101, 128, 129, 130 | TODO |
| [132](132-playwright-bun-foundation.md) | Establish a Bun-only Playwright test foundation | P1 | L | 094, 101, 128, 129 | TODO |
| [144](144-playwright-product-contracts-and-ci.md) | Make fixture-backed Playwright product contracts a required CI gate | P1 | L | 094, 101, 128, 129, 132 | TODO |
| [145](145-playwright-real-stack-integration.md) | Prove critical UI flows against managed GreptimeDB and isolated Turso | P1 | L | 093, 101, 132, 144 | TODO |
| [146](146-playwright-cross-browser-accessibility-visual.md) | Establish cross-browser, mobile, accessibility, and visual Playwright gates | P1 | L | 101, 132, 144, 145 | TODO |

### TypeScript Capability And Feature Migrations

Plans 152 and 153 establish GraphQL and non-GraphQL runtime boundaries after the
layer graph. Plan 149 then establishes shared route-less capability facades
before any feature move. Plans 134-142 and 150 are split by product owner for parallel work.
Plans 139, 140, and 142 wait for plan 134's public pin facade; plan 140 also
waits for plan 141's public logs facade. Plan 143 moves app/layout/shell only,
and plan 151 verifies zero residual architecture debt without absorbing product
work.

| Plan | Title | Priority | Effort | Depends on | Status |
|------|-------|----------|--------|------------|--------|
| [149](149-route-less-capability-foundation.md) | Establish route-less UI capabilities before feature moves | P1 | L | 100, 129, 152, 153 | TODO |
| [134](134-investigations-feature-migration.md) | Migrate investigations behind a strict feature facade | P1 | L | 100, 129, 132, 144, 145, 146, 149, 152, 153 | TODO |
| [135](135-sql-feature-migration.md) | Migrate the SQL workspace behind decoded feature boundaries | P1 | L | 100, 129, 132, 144, 145, 146, 149, 152, 153 | TODO |
| [136](136-ecosystem-feature-migration.md) | Migrate ecosystem topology into a bounded feature | P1 | M | 100, 129, 132, 144, 145, 146, 149, 152, 153 | TODO |
| [137](137-dashboards-feature-migration.md) | Migrate dashboards into decoded model and API boundaries | P1 | L | 100, 129, 132, 144, 145, 146, 149, 152, 153 | TODO |
| [138](138-services-feature-migration.md) | Move services into one bounded feature | P1 | L | 100, 129, 132, 144, 145, 146, 149, 152, 153 | TODO |
| [139](139-issues-feature-migration.md) | Move issues and stacktrace ownership into one feature | P1 | L | 100, 129, 132, 134, 144, 145, 146, 149, 152, 153 | TODO |
| [141](141-logs-feature-migration.md) | Move logs and the reusable log table into one feature | P1 | L | 100, 129, 132, 144, 145, 146, 149, 152, 153 | TODO |
| [140](140-runs-feature-migration.md) | Move runs, sessions, and live observation into one feature | P1 | L | 100, 129, 132, 134, 141, 144, 145, 146, 149, 152, 153 | TODO |
| [142](142-traces-feature-migration.md) | Move trace search, analysis, and inspection into one feature | P1 | XL | 100, 129, 132, 134, 144, 145, 146, 149, 152, 153 | TODO |
| [150](150-overview-feature-migration.md) | Move overview into one bounded feature | P1 | L | 100, 129, 132, 144, 145, 146, 149, 152, 153 | TODO |
| [143](143-app-layout-shell-migration.md) | Move app, layout, and shell behind explicit boundaries | P1 | XL | 134, 135, 136, 137, 138, 139, 140, 141, 142, 145, 146, 149, 150, 152, 153 | TODO |
| [151](151-ui-architecture-final-closure.md) | Verify and close the final UI architecture | P1 | L | 134, 135, 136, 137, 138, 139, 140, 141, 142, 143, 149, 150, 152, 153 | TODO |

### UI State, Performance, And Product Gaps

| Plan | Title | Priority | Effort | Depends on | Status |
|------|-------|----------|--------|------------|--------|
| [133](133-ui-tanstack-query-cache.md) | Replace the UI TTL cache with feature-owned TanStack Query | P1 | L | 095, 101, 128, 129, 132, 144, 145, 151 | TODO |
| [147](147-ui-live-data-performance.md) | Make live telemetry updates typed, bounded, and identity-stable | P1 | L | 095, 101, 129, 133, 140, 141, 142, 145, 151 | TODO |
| [148](148-ui-bundle-performance.md) | Enforce route-owned production chunks and deterministic bundle budgets | P1 | L | 095, 100, 101, 105, 132, 133, 144, 146, 147, 151 | TODO |
| [105](105-metric-overview-and-trends.md) | Replace metric stubs and reconcile CLI, native-name, and metric-only service contracts | P2 | M | 097, 099, 133, 151 | TODO |

### Dependencies, Tests, And Performance

| Plan | Title | Priority | Effort | Depends on | Status |
|------|-------|----------|--------|------------|--------|
| [101](101-dependencies-nextest-and-hygiene.md) | Add Cargo/Bun dependency policy, nextest evidence, native smoke, and staged hygiene | P1 | L | 094, 095, 096 | TODO |
| [103](103-property-fuzz-and-performance.md) | Add focused Rust/UI property/fuzz corpora and measured performance/allocation gates | P2 | L | 097, 099, 101, 104, 133, 147, 148 | TODO |
| [113](113-ingest-backpressure-observability.md) | Make queue, spool, retry, drop, and drain health observable | P2 | M | 095, 099 | TODO |

### Evidence Contracts And Closure

| Plan | Title | Priority | Effort | Depends on | Status |
|------|-------|----------|--------|------------|--------|
| [104](104-evidence-bundle-contract-reconciliation.md) | Reconcile the research bundle model with shipped `bundle-v1` | P1 | L | 093, 099 | TODO |
| [111](111-redaction-pipeline-and-a6-gate.md) | Build the source-aware fail-closed runtime redactor and prove the A6 gate | P1 | L | 099, 101, 104 | TODO |
| [116](116-retention-and-prune-lifecycle.md) | Reconcile data retention and make `prune` truthfully reclaim eligible data | P1 | L | 093, 097, 099; 105 soft | TODO |
| [106](106-evidence-pinning-ttl-spike.md) | Design and live-test evidence pinning beyond telemetry TTL | P2 | M | 092, 104, 116 | TODO |
| [107](107-program-closure-audits.md) | Run independent source audits and verify the mechanical closure commit | P1 | M | Every other actionable indexed plan; all blockers freshly rechecked | TODO |

### Triggered Or Operator-Blocked Work

These are unfinished and therefore remain as plan files, but execution must not
invent the missing product or operator decision.

| Plan | Depends on | Trigger | Status |
|------|------------|---------|--------|
| [108](108-rotel-credential-history-decision.md) | Operator decision | Operator confirms whether non-default lab credentials ever entered Git history and authorizes any rewrite | BLOCKED: operator-only destructive-history decision |
| [109](109-v2-auth-and-context-management.md) | Operator opens V2 scope | Operator opens V2 authentication and remote-context scope | BLOCKED: V2 not open |
| [110](110-server-profile-ingest-concurrency.md) | 099, 113, 115; measured saturation | Plan 115 ships a supported profile and measurements prove single-worker saturation | BLOCKED: no qualifying profile/measurement |
| [112](112-product-mcp-ship-gates.md) | 099, 104, 111; 109 before any remote transport | Operator opens the product MCP ship/no-ship decision after evidence-safety prerequisites | BLOCKED: product MCP not open |
| [114](114-retire-legacy-spool-reader.md) | A qualifying release cycle and expired legacy segments | A raw-frame release completes one compatibility cycle and all supported legacy segments expire | BLOCKED: no qualifying release cycle |
| [115](115-v2-server-profile.md) | 102, 109; operator opens V2 server scope | Operator opens V2 server scope and approves a supported profile contract | BLOCKED: V2 server scope not open |
| [118](118-sentry-envelope-migration-adapter.md) | 093, 099, 104, 111, 116 | Operator opens Sentry-compatible ingest after evidence makes it the next adoption constraint | BLOCKED: compatibility scope and demand trigger not open |
| [120](120-agent-session-capture-adapters.md) | 099, 104, 111, 119 | Operator selects and opens one coding-agent session capture adapter | BLOCKED: adapter/tool/version/consent scope not open |
| [121](121-deploy-and-change-context-collectors.md) | 099, 104, 109, 111, 116 | Operator selects and opens one deploy/change provider integration | BLOCKED: provider/auth/claim scope not open |
| [122](122-playground-residual-program.md) | 105, 111, 119, 151 | Operator names the companion repository branch and exact remaining cross-repo scope | BLOCKED: cross-repository branch/scope not authorized |
| [123](123-fixer-outcome-loop.md) | 104, 111, 120, 121 | Operator opens a separate fixer after A1/A2/A3/redaction gates | BLOCKED: autonomous-fixer scope and prerequisites not open |
| [124](124-ci-and-flaky-test-evidence-collector.md) | 099, 104, 111, 121 | Operator selects and opens product CI-provider collection | BLOCKED: provider/repository/permission scope not open |

## Dependency Order

The main restructuring path is:

```text
093 -> 094 -> 095 -> 127 -> 096 -> 097
094 + 095 + 096 -----------------------> 101
097 + 101 -----------------------------> 126 -> 098 -> 099 -> 104 -> 111
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
094 + 096 + 101 ------------------------------> 102
097 + 099 + 101 + 104 + 133 + 147 + 148 -----> 103
095 + 099 ------------------------------------> 113
095 ------------------------------------------> 117
095 + 096 + 100 + 101 + 126 ----------------> 119
093 + 097 + 099 ------------------------------> 116
092 + 104 + 116 ------------------------------> 106
093 + 097 + 099 + 104 ------------------------> 125
102 + 109 ------------------------------------> 115
099 + 113 + 115 ------------------------------> 110
099 + 104 + 111 -- operator trigger ----------> 112
093 + 099 + 104 + 111 + 116 -- operator ------> 118
099 + 104 + 111 + 119 -- operator ------------> 120
099 + 104 + 109 + 111 + 116 -- operator ------> 121
105 + 111 + 119 + 151 -- cross-repo ----------> 122
104 + 111 + 120 + 121 -- operator ------------> 123
099 + 104 + 111 + 121 -- operator ------------> 124
all actionable plans --------------------------> 107
```

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
