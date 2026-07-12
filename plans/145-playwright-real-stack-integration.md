# Plan 145: Prove critical UI flows against managed GreptimeDB and isolated Turso

> **Executor instructions**: Extend the Playwright foundation from plans 132 and
> 144 with one distinct real-stack project. Run Parallax in its product storage
> composition: a managed GreptimeDB process for telemetry and a fresh isolated
> Turso/libSQL metadata database. Seed raw signals through public OTLP and drive
> metadata changes through public GraphQL/UI boundaries. Do not reuse the
> fixture-backed adapter, intercept happy-path responses, or weaken eventual-
> consistency assertions to make the suite pass.
>
> **Drift check (run first)**:
> `git diff --stat e3e7997..HEAD -- .github/workflows/ci.yml .github/workflows/storage-integration.yml ui/package.json ui/playwright.config.ts ui/test-matrix.json ui/tests/e2e crates/parallax-server crates/parallax-test-support crates/parallax-xtask mise.toml ratchet.toml`
> Reconcile moved crates and final public facades before editing. If managed
> engine ports or metadata composition changed, update the isolation design and
> rerun occupied-port/process-cleanup proofs before continuing.

## Status

- **Priority**: P1
- **Effort**: L
- **Risk**: HIGH
- **Depends on**: 093, 101, 132, 144
- **Category**: tests / integration / GreptimeDB / Turso
- **Planned at**: `e3e7997`, revised 2026-07-12
- **Status**: TODO

## Why This Matters

Fixture-backed contracts prove browser behavior and API wiring but not native
GreptimeDB tables, telemetry visibility delay, Turso persistence, or real
process lifecycle. The existing managed-engine test at
`crates/parallax-server/tests/m1_greptime.rs:45-125` demonstrates real OTLP,
GreptimeDB, and metadata visibility but never drives the browser. The scheduled
storage workflow runs ignored Rust tests only and the required CI aggregate has
no real-stack browser lane owner.

GreptimeDB's managed supervisor uses fixed ports 24000-24003 at
`crates/parallax-server/src/greptime_supervisor.rs:20-23`. The first suite must
therefore use one worker per host and prove cleanup rather than pretending
worker-local data directories make the engine listeners parallel-safe.

## Fixed Decisions

1. The full-stack project uses managed GreptimeDB plus an isolated Turso/libSQL
   metadata database in a unique temporary Parallax data directory. There is no
   memory adapter, fallback engine, external shared database, or product-mode
   substitution.
2. Raw traces, logs, and metrics enter through Parallax's public OTLP HTTP/gRPC
   listeners using canonical SDK/protobuf helpers. Tests never insert raw rows
   directly or create custom raw-signal tables.
3. This foundation proves Turso persistence with one issue-status mutation
   derived from the public OTLP seed. Plans 134-143 and 150 own their specific
   investigation, dashboard, saved-view, snippet, SQL, and other metadata cases.
   All are created/mutated through GraphQL/UI boundaries unless no public setup
   path exists. A direct metadata seed is allowed only for an otherwise
   unreachable precondition and must be isolated behind test support, documented
   in the matrix, and followed by a public read/write postcondition.
4. The suite uses fixed logical UTC timestamps and stable product IDs, but real
   engine/process readiness uses monotonic deadlines. Browser time is controlled
   only where product clock behavior requires it.
5. Eventual visibility is asserted by polling a named GraphQL predicate with a
   bounded deadline and diagnostic samples. No fixed sleep and no assumption
   that GreptimeDB visibility implies Turso visibility.
6. One Playwright worker owns one stack on a host. Parallelism is allowed only
   across separate CI jobs/hosts with separate data and artifact directories.
7. Required critical flows run with zero status-clearing retries. A diagnostic
   rerun may collect evidence but cannot turn the original job green.
8. Xtask owns engine/server/seed lifecycle only. Playwright owns browser
   fixtures, contexts, locators, assertions, projects, reports, and artifacts.
9. Shared named `@storage` cases use stable `platform/storage` and
   `playwright/full-stack` owner IDs. Delegated rows use their final feature/
   layout owner plus `playwright/full-stack`; numeric 145 or 134-143/150 appears
   only as temporary `delivery_plan`.

## Target Ownership

```text
ui/tests/e2e/
  datasets/
    real-stack.ts             # expected IDs and public seed manifest
  fixtures/
    real-stack-fixture.ts     # consumes xtask runtime manifest
  full-stack/
    telemetry-discovery.spec.ts
    storage-composition.spec.ts
    live-transport.spec.ts       # one-event @storage lifecycle smoke
  screens/                    # reuse contract screen objects only when useful
crates/<current-test-support-owner>/
  ... public-boundary OTLP seed builders and GraphQL readiness predicates
crates/parallax-xtask/
  ... isolated data dir, engine/server lifecycle, seed and diagnostics commands
```

The test-support crate/module may construct deterministic OTLP payloads and
query public readiness endpoints. It cannot expose storage adapter internals to
Playwright or make the browser suite depend on a concrete Greptime/Turso type.

## Foundation And Delegated Matrix

Plan 145 implements one coherent `@storage` seeded run which proves:

- a trace, logs, metrics, run correlation, error-derived issue, service catalog,
  and ecosystem edge appear from public OTLP ingest;
- overview, services, issues, traces, ecosystem, logs, and runs can navigate to
  one another using the same stable IDs;
- native trace detail, log context, metric summaries, and one post-open SSE
  update show bounded correct data; and
- one error-derived issue status mutation persists through Turso across a fresh
  BrowserContext and public GraphQL read.

The matrix also reserves, but does not implement here, these exact full-stack
scenario owners after the shared project is green. `lane_owner` is always
`playwright/full-stack`; the Plan column is temporary `delivery_plan`:

| Plan | File | Tag |
|---|---|---|
| 134 | `full-stack/investigations.spec.ts` | `@investigations` |
| 135 | `full-stack/sql.spec.ts` | `@sql` |
| 136 | `full-stack/ecosystem.spec.ts` | `@ecosystem` |
| 137 | `full-stack/dashboards.spec.ts` | `@dashboards` |
| 138 | `full-stack/services.spec.ts` | `@services` |
| 139 | `full-stack/issues.spec.ts` | `@issues` |
| 140 | `full-stack/runs.spec.ts` | `@runs` |
| 141 | `full-stack/logs.spec.ts` | `@logs` |
| 142 | `full-stack/traces.spec.ts` | `@traces` |
| 143 | `full-stack/shell.spec.ts` | `@shell` |
| 150 | `full-stack/overview.spec.ts` | `@overview` |

Plan 145 must not pre-create duplicate investigation, SQL, ecosystem,
dashboard, services, issues, runs, logs, traces, shell, or overview behavior.

Keep the cases independently addressable in `ui/test-matrix.json`. Reuse the
same stack within this single-worker project only when each case resets or uses
a unique namespace and can run in any order.

## Commands

| Purpose | Command | Expected result |
|---------|---------|-----------------|
| Exact install | `cd ui && bun ci` | frozen lock, no lifecycle scripts |
| Browser install | `cd ui && bunx --bun --no-install playwright install --with-deps chromium` | locked Chromium installed |
| Engine preflight | `cargo xtask ui-browser full-stack preflight` | engine binary/version/checksum, ports, disk, and cleanup ready |
| Full stack | `cd ui && bun run test:browser:full` | one-worker managed GreptimeDB + Turso project passes |
| Policy | `cargo xtask policy --only ui.browser-full-stack` | storage, seed, worker, polling, lifecycle, matrix, and artifact rules pass |
| Rust integration | `cargo nextest run --locked -p parallax-server --run-ignored only` | managed-engine integration tests pass |
| UI checks | `cd ui && bun run check && bun run lint && bun run typecheck && bun run --bun test:ci && bun run build` | all exit 0 |
| Workflow | `mise exec -- actionlint .github/workflows/*.yml` | exit 0 |
| Full aggregate | `cargo xtask ci --full` | full-stack lane selected and green |

`test:browser:full` must select only the real-stack project and force one
worker. It must fail when zero cases are selected or when the runtime manifest
identifies any storage composition other than managed GreptimeDB plus Turso.

## Scope

In scope:

- Real-stack Playwright project, fixture, datasets, critical specs, and stable
  matrix rows.
- Xtask preflight/start/seed/readiness/stop and redacted diagnostic bundle.
- Public-boundary OTLP builders and GraphQL readiness/postcondition helpers.
- A path-aware required critical full-stack CI job plus scheduled/manual repeat
  coverage in the existing storage integration workflow.
- One-worker/fixed-port enforcement and process/data cleanup tests.

Out of scope:

- In-memory/fixture-backed product contracts (plan 144).
- Cross-browser, mobile, accessibility, and visual breadth (plan 146).
- UI structure changes (plans 134-143, 149, and 150), Query migration (plan
  133), live-data optimization (plan 147), or bundle work (plan 148).
- Direct native-table writes, custom raw tables, shared cloud state, hidden
  credentials, broad retries, browser response substitution, or Node tooling.

## Git Workflow

- Stay on the one active branch; do not create a branch or PR.
- Land lifecycle/seed support, the critical project, and CI integration as
  separate green commits.
- Use Conventional Commits, DCO, and exactly one agent-product trailer.
- Push every durable green update.

## Steps

### Step 0: Record the exact real-stack contract

Add real-stack rows to `ui/test-matrix.json` for every required critical case.
Record seed signal, stable IDs, product surface, expected Greptime predicate,
expected Turso predicate, mutation postcondition, timeout class, and owning spec.
Capture the current managed engine version resolution, fixed listener ports,
data paths, startup phases, and shutdown behavior in a machine-readable runtime
manifest schema.

**Verify**: matrix/runtime schema positive fixtures pass; missing storage mode,
unbounded timeout, absent predicate, duplicate ID, or fixture-backed project
assignment fails policy.

### Step 1: Build isolated full-stack lifecycle orchestration

Add `cargo xtask ui-browser full-stack preflight|start|seed|status|stop`. Create
one unique temporary data directory and artifact directory per run, verify ports
24000-24003 are free, resolve/check the repository-approved GreptimeDB binary,
start managed Parallax, and wait for narrated engine/bootstrap/API/OTLP readiness.
Expose only sanitized addresses, PIDs, versions, dataset ID, and paths needed by
the owning process.

On success, failure, timeout, signal, or cancellation, stop the browser-owned
server/engine process tree, verify all fixed/dynamic ports are free, and remove
ephemeral data unless failure retention was explicitly requested. Never kill a
foreign PID based only on a port or stale file.

**Verify**: lifecycle tests cover cold/warm engine startup, occupied foreign
port, stale owned PID, startup timeout, test failure, cancellation, double stop,
and two sequential clean runs with no process/port/data leakage.

### Step 2: Seed through public product boundaries

Create deterministic OTLP trace/log/metric fixtures using fixed resource,
scope, service, run, error, and correlation attributes. Export them through the
started Parallax OTLP endpoints and flush exporters. Use a unique dataset prefix
so retained failure data cannot collide with another job.

Poll GraphQL predicates separately for telemetry/native-table visibility and
derived Turso metadata visibility. Log bounded attempts, elapsed time, last
sanitized response shape, and which predicate remains false. Use exponential or
bounded interval scheduling without a blind sleep. Seed metadata through public
mutations/UI as each scenario requires.

**Verify**: seed helper tests prove exact IDs/timestamps, all three signal kinds,
separate visibility predicates, timeout diagnostics, idempotent same-dataset
behavior, and collision-free different datasets.

### Step 3: Add the three foundation scenario groups

Implement only the `@storage` foundation matrix in three spec files:

1. `telemetry-discovery` validates ingest-to-overview/service/issue/trace/log/run/
   ecosystem discovery and cross-route links.
2. `storage-composition` changes the derived issue status through the UI, opens
   a new BrowserContext, and proves the Turso-backed value through visible UI and
   a typed public GraphQL postcondition.
3. `live-transport` is one `@storage` infrastructure smoke: ingest one record
   after the page opens, prove it appears once, exercise one supported
   hide/show or disconnect/reconnect cycle, and prove the same record is not
   rendered twice. It does not own burst buffering, feature identity/order,
   replay classification, filter-generation reset, capacity, heap, timing, or
   performance; plan 147 owns those distinct `@live` cases.

Do not read database files from the test for assertions. Use visible UI state
and typed public GraphQL postconditions. Keep accessibility semantics and
locator rules from plan 144.

Register exact reserved rows for plans 134-143 and 150 without creating their
specs. Each row records its stable feature `scenario_owner`,
`playwright/full-stack` `lane_owner`, numeric `delivery_plan`, exact target
file/tag, that delivery plan's browser-materialization step, and state. Policy
allows the reservation only while the indexed `delivery_plan` is `TODO`, or is
`IN PROGRESS` before that named step. The named delivery step atomically
replaces `reserved` with a discovered non-empty spec row, clears
`delivery_plan`, and does not transfer either durable owner. Fail an orphan,
duplicate, wrong file, terminal/stale/unindexed delivery plan, collapsed or
transferred ownership, or reservation that survives its materialization step.

**Verify**: all foundation groups pass individually and in two different orders
against fresh stacks; a deliberately wrong storage mode, seed predicate,
duplicate live event, and duplicate delegated row each fail with the expected
diagnostic.

### Step 4: Add the required and scheduled CI lanes

Add a `browser-full-stack` CI job selected by UI, API, server, storage,
test-support, xtask, Cargo/lock/toolchain, engine-version, and workflow inputs.
Provision Bun/Rust/Playwright explicitly, reuse established Cargo/Bun/engine
binary caches with version/checksum keys, and run one worker. Do not cache engine
data or metadata databases.

Include the critical job in `ci-required`. Extend the scheduled/manual storage
workflow to run the same command from a clean data directory, not a second test
definition. Upload redacted Playwright evidence plus Parallax/Greptime logs and
the runtime/seed manifest on failure. Retention must be bounded.

**Verify**: actionlint and path-routing fixtures pass; required CI fails on
zero-test, wrong storage, engine startup, seed, browser, and cleanup failures;
the scheduled workflow invokes the identical locked command.

### Step 5: Ratchet reliability and duration

Record cold/warm startup, seed-to-Greptime visibility, seed-to-Turso visibility,
scenario, and cleanup durations as machine data. Set per-phase hard deadlines
above measured healthy p99 plus an explicit margin, not one suite-global timeout.
Fail a phase regression beyond the ratchet and print the owning phase.

Run the suite repeatedly in the canonical CI environment to expose leaks and
order dependence. A diagnostic retry retains the original failure and publishes
both attempts; it never changes required status.

**Verify**: repeated-run harness completes the approved count with zero leak,
order, duplicate, timeout, or residual-data failure; deadline negative fixtures
attribute the correct phase.

## Test Plan

- Managed lifecycle, fixed-port, ownership, cancellation, cleanup, and manifest
  tests.
- Public OTLP trace/log/metric seed and separate Greptime/Turso visibility tests.
- Browser telemetry discovery/cross-route, issue-status Turso persistence, and
  one-event SSE transport/reconnect foundation smoke.
- Delegated full-stack owner/file/tag policy fixtures for plans 134-143 and 150.
- Wrong-storage, direct-insert, response-interception, multi-worker, unbounded-
  polling, zero-test, and artifact-redaction policy fixtures.
- Required/scheduled CI path, cache-key, failure, and aggregate cases.

## Done Criteria

- [ ] `test:browser:full` always runs one isolated managed GreptimeDB + Turso
  stack and rejects memory/fallback/shared state.
- [ ] Raw traces, logs, and metrics are seeded through public OTLP and become
  visible through bounded separate Greptime/Turso predicates.
- [ ] Critical discovery, cross-route, issue-status Turso persistence, and live
  transport flows pass from a clean stack without duplicating feature-owned rows.
- [ ] Plans 134-143 and 150 each deliver one reserved full-stack scenario/file/
  tag paired with `playwright/full-stack`; it becomes non-empty, clears the
  temporary delivery plan, and retains durable owners at its exact step.
- [ ] Lifecycle cleanup releases every process, fixed/dynamic port, and data
  owner on success, failure, timeout, and cancel.
- [ ] A path-aware critical full-stack job is aggregated as required and the
  scheduled workflow runs the same command.
- [ ] Failure artifacts are complete, bounded, and redacted.
- [ ] Every command in this plan passes twice from clean state.

## STOP Conditions

Stop and report if:

- the suite can pass with memory storage, a custom raw table, direct raw-signal
  insert, or a shared external database;
- safe parallel workers require changing the product's fixed managed-engine
  ports in this testing plan;
- a public ingest/read/mutation boundary needed by a critical flow does not
  exist and adding it would change product scope;
- eventual consistency cannot be expressed as a bounded observable predicate;
- a failure artifact would retain secrets or unredacted telemetry content;
- Node, a second browser framework, response interception, or status-clearing
  retries appear necessary; or
- a required gate fails twice after a reasonable correction.

## Maintenance And Removal

Storage/API/UI contract changes update the real-stack seed schema, matrix row,
predicate, and browser postcondition together. Engine upgrades rerun cold/warm
lifecycle and native-table inventory evidence before deadline ratchets move.

Delete this plan and its README row only after the managed lifecycle, public
seed/readiness contract, critical browser suite, required/scheduled CI lanes,
duration ratchets, cleanup, and redacted artifacts are durable and green.
