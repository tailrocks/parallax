# Plan 093: Correct contracts and freeze behavioral baselines

> **Executor instructions**: Contract changes land in the implementation spec
> before product code. Preserve GraphQL/OTLP/storage behavior while removing the
> already-forbidden product fallback. Capture oracles before any structural
> move and update the active index after each durable state change.

## Status

- **Priority**: P1
- **Effort**: L
- **Risk**: HIGH
- **Depends on**: none
- **Category**: foundation / correctness
- **Planned at**: `eefa4617`, 2026-07-12
- **Status**: TODO

## Why

The repository mandates GreptimeDB + Turso with no product fallback, yet server
config and quickstart still expose storage mode `none` and construct
`MemoryStore`. Large restructuring without contract snapshots would also make
it easy to change GraphQL SDL, normalized rows, SQL, CLI output, or ingest
ownership unintentionally.

## Current Evidence

- `AGENTS.md` restricts the in-memory adapter to tests/dev harnesses.
- `crates/parallax-server/src/config.rs` defines `managed | external | none`.
- `crates/parallax-server/src/serve.rs` constructs MemoryStore for `none`.
- `docs/guide/quickstart.md` advertises that fallback.
- `docs/research/architecture/v1-implementation-spec.md` and `v1-scope.md`
  disagree about whether `none` is a product mode.
- V1 scope still promises a distinct `term` bundle format, profile ingest, a
  metrics CLI, and retention/prune behavior that live surfaces do not all
  implement; plans 105/116 own the retained metric/retention work.
- Current scenario evidence covers PostgreSQL-shaped and GraphQL spans but not
  the promised ClickHouse-shaped query or DataLoader batch spans.
- The worker retries a combined registration, broadcast, telemetry-store, and
  issue-recording operation; late failure behavior is not characterized.

## Scope

In scope:

- Implementation spec, V1 scope, quickstart, config contract, and ready output.
- Internal dependency injection/composition for tests.
- Baseline artifacts for crate graph, SDL, storage contracts, tests, file size,
  suppressions, unsafe, clones, TLS features, and CI jobs.
- A defect-to-gate ledger seeded from known escaped defects.
- Failure-injection characterization of the current worker stages.

Out of scope:

- Moving MemoryStore to a new crate, owned by plan 097.
- Changing retry semantics or typed errors, owned by plan 099.
- Crate/module moves.
- Any fallback database or raw-signal schema change.

## Steps

### Step 1: Capture the exact baseline

Write a dated packet under `docs/research/validation/` containing:

- clean commit and tool versions;
- Cargo package/dependency graph as machine-readable data;
- GraphQL SDL/hash and representative error snapshots;
- V1 acceptance requirement-to-test/evidence mapping, with explicit missing
  ClickHouse-shaped and DataLoader-batch fixtures;
- Rust/UI test and ignored-test inventory;
- native Greptime table/extension table inventory and representative SQL;
- Turso migration/row-mapping inventory;
- Rust/UI line distributions and named hotspots;
- `#[allow]`, `#[expect]`, unsafe, hot-path clone, and public-module census;
- active TLS feature tree and Bun/package-manager file inventory;
- CI job/path-filter/required-check inventory.

Every generated artifact needs a command and schema/version marker.

### Step 2: Make the contract unambiguous

Update `docs/research/architecture/v1-implementation-spec.md` first, then
`v1-scope.md`, quickstart, CLI/config docs, and examples:

- product modes are managed and external GreptimeDB + Turso only;
- in-memory storage is an internal test harness, never a CLI/config mode;
- tests inject capabilities through an internal composition root;
- ready output always names GreptimeDB, Turso, storage mode, and data dir.
- Markdown is the terminal bundle projection unless an independently justified
  `term` format is retained and implemented.
- unsupported profile OTLP ingest is removed from V1 claims until a native
  Greptime profile path and operator-approved scope exist.
- metric CLI and retention/prune divergences are assigned to plans 105 and 116
  with no duplicate promise here.

The reconciliation inventory must explicitly include root `README.md`,
`docs/research/README.md`, `docs/research/architecture/simple-ui-v2.md`, the
historical `v1-build-plan.md`, and
`docs/research/storage/metadata/metadata-store-benchmark-plan.md`. Run a
repository-wide structured/text policy scan for ClickHouse/Postgres fallback,
`storage.mode = "none"`, and npm/pnpm/yarn execution claims so another stale
surface cannot escape this named list.

### Step 3: Add the internal composition seam

Refactor server startup so product entry points validate and construct
mandatory stores while tests can call an internal builder with injected
capabilities. Remove `none` from configuration, CLI help, serve dispatch, and
ready banners. Do not expose a hidden feature/env/flag fallback.

MemoryStore may remain test-only source inside storage until plan 097. First
move production-neutral helpers that Greptime uses, including
`rate_from_buckets`, into an appropriately owned production module. Then gate
only the adapter type, state, constructor, implementations, and exports behind
`cfg(test)` or an explicit dev-only test-support feature that no
normal/build/release graph enables. Do not cfg-gate the whole `memory` module
while production code still references one of its helpers. The adapter must be
unreachable from product config, product crate default features, and release
dependency paths.

### Step 4: Characterize worker failure stages

Add failure injection at registration, live broadcast, telemetry storage, and
issue recording. Record which earlier effects repeat when each later stage
fails. These tests are oracles for plan 099; do not redesign retries here.

### Step 5: Seed the defect ledger

Create a ledger with defect class, escaped symptom, exact regression test ID,
preventing gate, landed commit, owner, status, and evidence date. Seed it from
gzip ingest, paging races, spool durability, redaction bypasses, storage
fallback, and worker retry findings.

## Test Plan

- Config parsing rejects `none` with a clear supported-values message.
- Product startup requires both stores in managed and external modes.
- Internal injected startup preserves current server tests.
- Release feature graph contains no product MemoryStore adapter path, while
  Greptime's extracted production helpers compile in normal/release builds.
- Worker failure-injection tests cover every stage and current replay behavior.
- ClickHouse-shaped database and DataLoader-batch span fixtures either satisfy
  the V1 acceptance contract or a reviewed contract correction explains their
  removal; PostgreSQL-only evidence cannot close the row.
- Baseline JSON/Markdown artifacts validate against their schemas.

## Done Criteria

- [ ] All contract/docs/config surfaces agree on mandatory GreptimeDB + Turso.
- [ ] Product code exposes no `none`/MemoryStore runtime path.
- [ ] Tests use an internal injection seam, not a hidden product mode.
- [ ] Baseline packet includes every listed machine-readable oracle.
- [ ] Worker stage characterization is green and unchanged by later moves.
- [ ] Defect ledger rows name exact tests/gates.
- [ ] Default workspace, real storage, UI, fmt, and Clippy gates pass.

## STOP Conditions

- Removing `none` requires a new public fallback rather than internal injection.
- Product behavior beyond the already-forbidden mode must change.
- GraphQL, OTLP, SQL, persistence, or CLI compatibility cannot be captured.
- A test can pass only by substituting away mandatory production composition.

## Remove When

Delete this plan and row after contracts agree, the product fallback is gone,
and the dated baseline/defect evidence is committed.
