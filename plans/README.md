# Active Implementation Plans

Execution rules for the complete program live in
[`IMPLEMENTATION.md`](IMPLEMENTATION.md). Both that contract and the Jackin
reference are active plan material and must be retired by plan 107 when the
program closes.

`plans/` is the only home for active Parallax implementation plans. It contains
unfinished work only. Completed, rejected, or superseded work belongs in Git
history and, when durable evidence is useful, under `docs/research/validation/`.

## Lifecycle

1. Use a unique, never-reused numeric ID and a flat
   `plans/NNN-kebab-case.md` path.
2. List only `TODO`, `IN PROGRESS`, or `BLOCKED` files in this index.
3. A plan file contains status metadata, evidence-backed rationale, scope, ordered
   steps, tests, machine-checkable done criteria, STOP conditions, and a
   `Remove When` section.
4. When a plan becomes terminal, record durable evidence if needed, then delete
   its file and index row in the same commit. Do not keep a DONE archive here.
5. Work directly on the single active branch from `AGENTS.md`; commit with DCO
   and exactly one agent-product trailer, then push each durable update.

The completed historical plan programs were retired on 2026-07-12. Their
closure evidence remains in
[`docs/research/validation/2026-07-11-advisor-plans-closure.md`](../docs/research/validation/2026-07-11-advisor-plans-closure.md)
and Git history.

## Reference Basis

The current restructuring program comes from a deep comparison of Parallax
with Jackin and PR #759. The analysis, audited commit, evidence classes,
copy/adapt/reject decisions, and target architecture live in
[`JACKIN-REFERENCE.md`](JACKIN-REFERENCE.md).

Non-negotiable Parallax constraints override every external reference:

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
| [089](089-extension-table-grpc-writes.md) | Move derived extension-table writes to GreptimeDB's row API | P2 | M | External upstream fix | BLOCKED: `greptimedb-ingester` hard-enables rustls through tonic `tls-ring` |
| [092](092-metric-exemplar-schema.md) | Correct the high-cardinality `metric_exemplars` primary key and migrate existing data | P1 | M | None | TODO |
| [125](125-native-trace-fingerprint-deviation.md) | Resolve the unpopulated native trace fingerprint deviation and migration contract | P2 | M | 093, 097, 099, 104 | TODO |

### Foundation And Delivery

| Plan | Title | Priority | Effort | Depends on | Status |
|------|-------|----------|--------|------------|--------|
| [093](093-contract-and-baseline-corrections.md) | Reconcile product contracts, remove the storage fallback, and capture behavioral baselines | P1 | L | None | TODO |
| [094](094-ci-and-security-foundation.md) | Repair CI topology, path routing, advisory gating, permissions, and repository security policy | P1 | L | 093 | TODO |
| [102](102-deterministic-release-pipeline.md) | Unify deterministic preview/stable packaging and release verification | P1 | L | 094, 101 | TODO |

### Quality Tooling And Rust

| Plan | Title | Priority | Effort | Depends on | Status |
|------|-------|----------|--------|------------|--------|
| [095](095-quality-control-plane.md) | Add xtask, architecture policy, one ratchet source, facades, and machine-readable diagnostics | P1 | L | 094 | TODO |
| [096](096-rust-toolchain-and-lints.md) | Pin latest stable Rust and activate a strict measured lint/suppression baseline | P1 | L | 095 | TODO |
| [117](117-documentation-link-integrity.md) | Add a parser-backed required internal Markdown link gate | P2 | S-M | 095 | TODO |
| [119](119-semconv-registry-codegen.md) | Generate checked-in Rust/Java/TypeScript semantic-convention constants from one registry | P2 | M | 095, 096, 100, 101 | TODO |

### Architecture And Boundaries

| Plan | Title | Priority | Effort | Depends on | Status |
|------|-------|----------|--------|------------|--------|
| [097](097-model-test-support-and-dependency-direction.md) | Extract the model leaf, move fakes to test support, split storage capabilities, and enforce direction | P1 | XL | 096 | TODO |
| [098](098-facades-modules-and-api-batching.md) | Seal crate facades, split responsibility hotspots, validate crate docs, and eliminate latent nested-field N+1 paths | P2 | L | 097 | TODO |
| [099](099-boundary-errors-idempotency-and-agent-safety.md) | Add typed errors, explicit retry/idempotency boundaries, an ID pilot, and agent-surface safety | P1 | L | 097, 098 | TODO |

### UI And Product Gaps

| Plan | Title | Priority | Effort | Depends on | Status |
|------|-------|----------|--------|------------|--------|
| [100](100-ui-feature-architecture.md) | Move UI ownership from routes to features and enforce import/data boundaries | P2 | L | 095; 099 soft | TODO |
| [105](105-metric-overview-and-trends.md) | Replace metric stubs and reconcile CLI, native-name, and metric-only service contracts | P2 | M | 097, 099, 100 | TODO |

### Dependencies, Tests, And Performance

| Plan | Title | Priority | Effort | Depends on | Status |
|------|-------|----------|--------|------------|--------|
| [101](101-dependencies-nextest-and-hygiene.md) | Add dependency policy, nextest evidence, native smoke, and staged security hygiene | P1 | L | 094, 095 | TODO |
| [103](103-property-fuzz-and-performance.md) | Add focused property/fuzz corpora and measured performance/allocation gates | P2 | L | 097, 099, 101, 104 | TODO |
| [113](113-ingest-backpressure-observability.md) | Make queue, spool, retry, drop, and drain health observable | P2 | M | 095, 099 | TODO |

### Evidence Contracts And Closure

| Plan | Title | Priority | Effort | Depends on | Status |
|------|-------|----------|--------|------------|--------|
| [104](104-evidence-bundle-contract-reconciliation.md) | Reconcile the research bundle model with shipped `bundle-v1` | P1 | L | 093, 099 | TODO |
| [111](111-redaction-pipeline-and-a6-gate.md) | Build the source-aware fail-closed runtime redactor and prove the A6 gate | P1 | L | 099, 101, 104 | TODO |
| [116](116-retention-and-prune-lifecycle.md) | Reconcile data retention and make `prune` truthfully reclaim eligible data | P1 | L | 093, 097, 099; 105 soft | TODO |
| [106](106-evidence-pinning-ttl-spike.md) | Design and live-test evidence pinning beyond telemetry TTL | P2 | M | 092, 104, 116 | TODO |
| [107](107-program-closure-audits.md) | Run independent source audits and verify the mechanical closure commit | P1 | M | Every actionable indexed plan | TODO |

### Triggered Or Operator-Blocked Work

These are unfinished and therefore remain as plan files, but execution must not
invent the missing product or operator decision.

| Plan | Trigger | Status |
|------|---------|--------|
| [108](108-rotel-credential-history-decision.md) | Operator confirms whether non-default lab credentials ever entered Git history and authorizes any rewrite | BLOCKED: operator-only destructive-history decision |
| [109](109-v2-auth-and-context-management.md) | Operator opens V2 authentication and remote-context scope | BLOCKED: V2 not open |
| [110](110-server-profile-ingest-concurrency.md) | Plan 115 ships a supported profile and measurements prove single-worker saturation | BLOCKED: no qualifying profile/measurement |
| [112](112-product-mcp-ship-gates.md) | Operator opens the product MCP ship/no-ship decision after evidence-safety prerequisites | BLOCKED: product MCP not open |
| [114](114-retire-legacy-spool-reader.md) | A raw-frame release completes one compatibility cycle and all supported legacy segments expire | BLOCKED: no qualifying release cycle |
| [115](115-v2-server-profile.md) | Operator opens V2 server scope and approves a supported profile contract | BLOCKED: V2 server scope not open |
| [118](118-sentry-envelope-migration-adapter.md) | Operator opens Sentry-compatible ingest after evidence makes it the next adoption constraint | BLOCKED: compatibility scope and demand trigger not open |
| [120](120-agent-session-capture-adapters.md) | Operator selects and opens one coding-agent session capture adapter | BLOCKED: adapter/tool/version/consent scope not open |
| [121](121-deploy-and-change-context-collectors.md) | Operator selects and opens one deploy/change provider integration | BLOCKED: provider/auth/claim scope not open |
| [122](122-playground-residual-program.md) | Operator names the companion repository branch and exact remaining cross-repo scope | BLOCKED: cross-repository branch/scope not authorized |
| [123](123-fixer-outcome-loop.md) | Operator opens a separate fixer after A1/A2/A3/redaction gates | BLOCKED: autonomous-fixer scope and prerequisites not open |
| [124](124-ci-and-flaky-test-evidence-collector.md) | Operator selects and opens product CI-provider collection | BLOCKED: provider/repository/permission scope not open |

## Dependency Order

The main restructuring path is:

```text
093 -> 094 -> 095 -> 096 -> 097 -> 098 -> 099 -> 104 -> 111
095 -------------------------------> 100 -> 105
099 ---------------------- soft ---> 100
097 --------------------------------------> 105
094 -> 095 -> 101 -> 102
097 + 099 + 101 + 104 ------------------------> 103
095 + 099 ------------------------------------> 113
095 ------------------------------------------> 117
095 + 096 + 100 + 101 ------------------------> 119
093 + 097 + 099 ------------------------------> 116
092 + 104 + 116 ------------------------------> 106
093 + 097 + 099 + 104 ------------------------> 125
102 + 109 ------------------------------------> 115
099 + 113 + 115 ------------------------------> 110
099 + 104 + 111 -- operator trigger ----------> 112
093 + 099 + 104 + 111 + 116 -- operator ------> 118
099 + 104 + 111 + 119 -- operator ------------> 120
099 + 104 + 109 + 111 + 116 -- operator ------> 121
100 + 105 + 111 + 119 -- cross-repo ----------> 122
104 + 111 + 120 + 121 -- operator ------------> 123
099 + 104 + 111 + 121 -- operator ------------> 124
all actionable plans --------------------------> 107
```

Plan 092 can run in parallel with 093. After 095, UI boundary work and
dependency/test telemetry can proceed in parallel with the Rust architecture
chain when their write sets are disjoint. Plan 107 is last. Any BLOCKED plan
does not block closure only while a fresh reproducible external/operator/phase
condition still holds and its file contains no hidden actionable work.

## Shared Verification

Each plan has narrower commands. The final program baseline is:

```text
git diff --check
git diff --cached --check
cargo fmt --all --check
cargo check --locked --workspace --all-targets
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo nextest run --locked --workspace --all-targets --profile ci
cargo test --locked --workspace --doc
cargo xtask ci --full
cd ui && bun ci
cd ui && bun run check
cd ui && bun run lint
cd ui && bun run typecheck
cd ui && bun run test:ci
cd ui && bun run build
mise exec -- actionlint
```

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
