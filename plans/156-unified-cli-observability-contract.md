# Plan 156: Replace `parallax.run.id` with the neutral CLI-invocation contract across semconv, ingest, storage, and API

> **Executor instructions**: Follow this plan step by step. Run every
> verification command and confirm the expected result before moving to the
> next step. If anything in the "STOP conditions" section occurs, stop and
> report — do not improvise. When done, update the status row for this plan
> in `plans/README.md`.
>
> **Drift check (run first)**:
> `git diff --stat 39f172c..HEAD -- telemetry/semconv crates/parallax-semconv crates/parallax-ingest crates/parallax-model crates/parallax-greptime crates/parallax-metadata crates/parallax-api crates/parallax-server crates/parallax-cli crates/parallax-analysis crates/parallax-storage crates/parallax-test-support`
> If any in-scope file changed since this plan was written, compare the
> "Current state" excerpts against the live code before proceeding; on a
> mismatch, treat it as a STOP condition.

## Status

- **Priority**: P1
- **Effort**: L
- **Risk**: MED (coordinated rename across ingest → storage → API; native-table
  columns involved)
- **Depends on**: none (deliberately independent of the blocked 128→151 UI
  architecture chain)
- **Category**: migration / product contract
- **Planned at**: commit `39f172c`, 2026-07-17
- **Operator directive (2026-07-17)**: Parallax drops its vendor correlation
  attribute. Generic CLI-application observability replaces the `parallax.run.id`
  special case. Implementation ships on the single authorized branch
  `feature/unified-cli-observability` as ONE Parallax PR (with one linked
  playground PR, plan 158). Plans 156, 157, 158, and 159 are that one PR's
  work packets.

## Why this matters

jackin❯ (the first large external emitter) has landed its unified-OTel cutover
upstream: `parallax.run.id` and every `jackin.*` key are **removed** on its
`feature/unified-otel-observability` branch (its plan 013 is DONE). Its new
neutral, semconv-1.43-aligned vocabulary is:

| Key | Meaning |
|---|---|
| `cli.invocation.id` | opaque UUID per top-level CLI process; **the correlation domain**. Stamped on root spans and logs — never on Resource, never a metric dimension |
| `session.id` / `session.previous_id` | interactive (TUI/attach) ownership window; `session.start` / `session.end` **events** mark the boundaries |
| `app.mode` | `one_shot` \| `interactive` \| `daemon` \| `capsule` |
| `cli.command.name` | bounded registry, dotted subcommand paths (`workspace.env.set`) |
| `ui.screen.entered/exited`, `ui.widget.focused/unfocused` events; `ui.action` root spans; `ui.screen.visit.id`, `ui.navigation.sequence`, `ui.transition.reason` | TUI screens are state (events), actions are bounded root traces |
| `background.cycle` root spans + `background.cycle.name` | substantive periodic daemon work |
| `job.id`, `job.type` + PRODUCER/CONSUMER linked spans | detached jobs |
| `gen_ai.agent.name`, `gen_ai.conversation.id`, `gen_ai.provider.name` | agent panes / conversations |
| `outcome` (`success\|failure\|error\|timeout\|skip\|cancellation`), `error.type` (stable names), `process.exit.code` | bounded result taxonomy |
| Trace shapes | one-shot: one `cli.command` root; interactive: `app.startup` + `app.shutdown` roots; **no lifetime spans** — an invocation is a correlation domain, not one giant trace |

Parallax's whole "runs" pipeline is keyed on the single resource attribute
`parallax.run.id`. When jackin❯ ships, Parallax would show nothing. This plan
makes `cli.invocation.id`/`session.id` the first-class correlation contract in
the backend, keeps a read-only legacy fallback for `parallax.run.id` data still
inside the telemetry TTL, and exposes the new signal families (sessions,
screens, actions, cycles, jobs, conversations) over GraphQL so plan 157 can
build the product surface. It also stops deriving issue fingerprints from the
dead `jackin.operation` key.

## Current state

(verified at `39f172c`)

- **Semconv registry**: `telemetry/semconv/contract.yaml` (113 lines) is the
  single checked-in language contract rendered by `cargo xtask semconv
  generate` into `crates/parallax-semconv/src/lib.rs`, `ui/src/shared/semconv.ts`,
  and the playground's Rust/TS/Java modules; `telemetry/semconv/registry/` is
  the Weaver overlay. Rows today include
  `{ id: parallax.run.id, … owner: shared }` and
  `{ id: jackin.operation, … owner: parallax }` (contract.yaml:23,26).
  `crates/parallax-semconv/src/lib.rs:25` `PARALLAX_RUN_ID`, `:28`
  `JACKIN_OPERATION`, `:58` `SESSION_ID` (defined, unused), `:68-73`
  `PARALLAX_SESSION_ID`/`PARALLAX_EXECUTION_LAYER`/`PARALLAX_AGENT_ID`/
  `GEN_AI_OPERATION_NAME`/`TOOL_NAME`/`SHELL_COMMAND` (defined, unused by
  ingest/storage/API).
- **Ingest extraction — the single point**: `crates/parallax-ingest/src/lib.rs:39-42`:

  ```rust
  /// Resolve the run id from resource attributes. Parallax intentionally keeps
  /// this to one key so one wrapped command has one lookup id.
  fn run_id(resource_attrs: &[KeyValue]) -> Option<String> {
      attr_str(resource_attrs, semconv::PARALLAX_RUN_ID).map(str::to_string)
  }
  ```

  plus `resource_run_ids()` (`:44-61`). `run_id` is threaded into
  `SpanRow`/`LogRow`/`MetricPointRow`/`MetricExemplarRow`
  (`crates/parallax-model/src/types.rs:17,42,55,67`) and `RunRecord`
  (`types.rs:155`).
- **Native-table promotion**: `crates/parallax-greptime/src/greptime/ingest.rs:36-42`
  builds `x-greptime-log-extract-keys = service.name,parallax.run.id,event.name,observed_ts_nanos`;
  `lifecycle.rs:55-79` pre-creates the promoted `"parallax.run.id"` STRING
  SKIPPING INDEX column on `opentelemetry_logs` (pre-creation keeps extract
  keys out of the PRIMARY KEY); `lifecycle.rs:91-97` creates extension table
  `run_metric_points ("run_id" STRING SKIPPING INDEX, …)`; exemplar DDL carries
  `run_id`. Trace-side identity is read from JSON:
  `resource_attributes."parallax.run.id"` (`run_store.rs:83,114`,
  `trace_store.rs:92,118,170,214`, helper `greptime_sql.rs:15`).
- **Metadata**: Turso `runs` table (`crates/parallax-metadata/src/turso.rs:36-43`):
  `run_id TEXT PRIMARY KEY, command, started_at, ended_at, exit_code, status`;
  CRUD in `turso/runs.rs`. The metadata-store decision record
  (`docs/research/decisions/metadata-store.md`) already names Turso the home of
  "agent-session and CLI-invocation state".
- **GraphQL**: `crates/parallax-api/src/lib.rs` query fields `traces_by_run
  (:174)`, `logs_by_run (:177)`, `agent_session (:180)`, `story(trace_id,
  run_id) (:183)`, `evidence_gaps (:186)`, `logs(run_id …) (:202)`, `run
  (:218)`, `observed_runs (:234)` (doc: "any tool exporting `parallax.run.id`
  — e.g. jackin'"), `runtime_snapshot(run_id …) (:270)`, `metric_series(run_id
  …) (:278)`, `bundle (:254)`; mutations `run_start (:311)` / `run_finish
  (:333)`. Types `Run`/`ObservedRun` in `resolvers/runs.rs:15-60`.
  Subscription root is `EmptySubscription` (`schema.rs:10-12`) — live data is
  SSE, deliberately (`parallax-server/src/live.rs:6`).
- **SSE filters**: `crates/parallax-server/src/live.rs:74-93` `StreamFilter
  { service, severity_min, q, trace_id, run_id }`, rendered as `"runId"`
  (`:106`); span-side `SpanStreamFilter.run_id` (`:155`, rendered `:185`).
  Routes `GET /v1/logs/stream`, `GET /v1/traces/stream`
  (`serve.rs:238-239`).
- **Producer side (Parallax's own CLI)**: `crates/parallax-cli/src/commands/forwarding.rs:139-140`
  appends `parallax.run.id={run_id}` to forwarded `OTEL_RESOURCE_ATTRIBUTES`;
  `commands/runs.rs:97-147` calls `run_start`/`run_finish`.
- **Legacy fingerprint input**: `crates/parallax-analysis/src/derive.rs:119,140,142,195`
  uses `semconv::JACKIN_OPERATION` for issue fingerprint/grouping — a key that
  no longer exists upstream.
- **Redaction identity keys**: `crates/parallax-storage/src/adapter_rules.rs:45-81`
  and `adapter.rs:300-330` treat `run_id`/`session.id`/`session_id` as identity
  keys.
- **Binding decision records** (must stay true):
  - `docs/research/decisions/native-otel-tables.md` — raw signals stay in
    GreptimeDB-native tables; custom raw-signal tables are a STOP; derived
    extension tables allowed; `run_id` extraction mechanism documented at
    lines ~85-90 (resource attr on traces via JSON column, extract-keys on
    logs, never a metric tag, `run_metric_points` extension for run-scoped
    points). This file must be updated in the same commit that renames the
    mechanism.
  - `docs/research/decisions/metadata-store.md` — Turso holds CLI-invocation
    and agent-session state; GreptimeDB holds telemetry.

## Contract decisions (fixed — do not relitigate)

1. **Canonical key**: `cli.invocation.id`. Accepted from, in priority order:
   root-span/log **attributes** (jackin's shape — never Resource), then
   **resource attributes** (generic wrapped emitters, Parallax's own wrapper).
   One helper owns this lookup order.
2. **Legacy fallback is read-only**: `parallax.run.id` is still *read* (ingest
   fallback + query-side COALESCE) so pre-cutover data inside the telemetry
   TTL keeps resolving, but Parallax never *emits* it, the UI never names it,
   and no new code path may depend on it. Remove the fallback when the
   playground (plan 158) and jackin are the only supported emitters and one
   TTL window has passed — record that trigger in the index ledger.
3. **`session.id`** becomes a second extracted correlation column on logs and
   a queried span attribute on traces; session boundaries come from
   `session.start`/`session.end` log events (`event.name` column), never from
   a lifetime span.
4. **Identity naming**: Rust/GraphQL/SQL use `invocation_id` (wire:
   `invocationId`); the Turso table becomes `invocations`; the extension table
   becomes `invocation_metric_points`. No abbreviation (`inv_id`) anywhere.
5. **Metrics stay low-cardinality**: `cli.invocation.id`/`session.id` are never
   metric tags; invocation-scoped points continue landing in the (renamed)
   extension table exactly as today (Q6 Approach 2).
6. **`jackin.operation` dies**: fingerprint derivation uses `cli.command.name`
   when present, else the existing non-jackin fallbacks. The contract.yaml row
   is deleted.
7. **New signal families are query-time projections over native tables** —
   no new raw-signal tables. Sessions/screens/actions/cycles/jobs/conversations
   are derived from `opentelemetry_logs` (events) and `opentelemetry_traces`
   (span name/kind/attributes/links) at query time.

## Commands you will need

| Purpose | Command | Expected on success |
|---|---|---|
| Toolchain | `mise install` (repo root) | exit 0 |
| Regenerate semconv | `cargo xtask semconv generate` | exit 0; regenerated Rust/TS modules |
| Semconv check | `cargo xtask semconv check` | exit 0 |
| Build | `cargo check --locked --workspace --all-targets` | exit 0 |
| Tests (no engine) | `cargo nextest run --locked --workspace --all-targets --profile ci` | all pass |
| Live-engine tests | `cargo nextest run --locked -p parallax-server -E 'binary(/greptime/)'` | all pass (needs a live GreptimeDB; the harness in `crates/parallax-server/tests/support/harness.rs` manages it) |
| Lint | `cargo clippy --locked --workspace --all-targets -- -D warnings` | exit 0, zero warnings |
| Fmt | `cargo fmt --all --check` | exit 0 |
| Aggregate | `cargo xtask ci --fast` | exit 0 |

(If `cargo xtask semconv` subcommand names differ, read
`crates/parallax-xtask/src/` for the semconv lane's real CLI before running —
do not guess.)

## Scope

**In scope:**
- `telemetry/semconv/contract.yaml`, `telemetry/semconv/registry/**`,
  `telemetry/semconv/constants.yaml`, regenerated `crates/parallax-semconv/**`
  and `ui/src/shared/semconv.ts` (generation output only — no hand edits).
- `crates/parallax-ingest/**` (extraction, row population, session id).
- `crates/parallax-model/**` (row/record field renames + `session_id`).
- `crates/parallax-greptime/**` (extract-keys, DDL, all run→invocation queries,
  new projection queries).
- `crates/parallax-metadata/**` (Turso `invocations` migration).
- `crates/parallax-api/**` (field renames, new resolvers, SDL).
- `crates/parallax-server/**` (SSE filter params, tests).
- `crates/parallax-cli/**` (wrapper mint/inject/registration).
- `crates/parallax-analysis/src/derive.rs` (fingerprint input swap).
- `crates/parallax-storage/**` (adapter identity keys, `ObservedRun` →
  `ObservedInvocation`).
- `crates/parallax-test-support/**` (memory stores).
- `docs/research/decisions/native-otel-tables.md` (mechanism row update, same
  commit as the rename).

**Out of scope (do NOT touch):**
- `ui/**` beyond the regenerated `semconv.ts` — plan 157 owns every UI change.
- The playground repository — plan 158.
- Retention/prune semantics (plan 116), evidence-bundle contract (plan 104),
  metric summary contract (plan 105) — only rename what they already touch.
- GraphQL subscriptions — SSE stays the live transport by decision.
- Any new raw-signal GreptimeDB table (hard rule; STOP condition).

## Git workflow

- Branch: `feature/unified-cli-observability` (operator-authorized
  2026-07-17; the ONE implementation branch for plans 156–159). Never commit
  to `main`; one Parallax PR ships plans 156+157+159 together.
- Conventional Commits, DCO `-s`, exactly one agent trailer, push after every
  durable commit. Suggested first subject:
  `feat(semconv): adopt neutral cli.invocation.id contract`.

## Steps

### Step 1: Registry first

In `telemetry/semconv/contract.yaml` (and the matching Weaver overlay under
`telemetry/semconv/registry/`):
- Add rows (owner `shared` unless noted): `cli.invocation.id`
  (`CLI_INVOCATION_ID`), `cli.command.name` (`CLI_COMMAND_NAME`), `app.mode`
  (`APP_MODE`) plus value consts (`one_shot`,`interactive`,`daemon`,`capsule`),
  `session.previous_id`, `session.start`/`session.end` event names,
  `ui.screen.entered`/`ui.screen.exited`/`ui.widget.focused`/`ui.widget.unfocused`
  event names, `app.screen.id`, `ui.action.name`, `ui.screen.visit.id`,
  `ui.navigation.sequence`, `ui.transition.reason`, `background.cycle` span
  name + `background.cycle.name`, `job.id`, `job.type`, `outcome` + its six
  values, `gen_ai.agent.name`, `gen_ai.conversation.id`,
  `gen_ai.provider.name`, `process.exit.code`, span names `cli.command`,
  `app.startup`, `app.shutdown`, `ui.action`.
- Keep `parallax.run.id` with a comment marking it **legacy read-only**
  (rename its rust const to `LEGACY_PARALLAX_RUN_ID` if the generator supports
  a rename; otherwise keep the name and add the comment).
- Delete the `jackin.operation` row.
- Run `cargo xtask semconv generate`; commit the regenerated
  `parallax-semconv/src/lib.rs` and `ui/src/shared/semconv.ts` untouched by
  hand.

**Verify**: `cargo xtask semconv check` → exit 0;
`grep -n "jackin.operation" telemetry/semconv/contract.yaml` → no match
(update `derive.rs` in step 6 before the workspace compiles again — or stage
both in one commit).

### Step 2: Ingest identity

In `crates/parallax-ingest/src/lib.rs` replace `run_id()` with:

```rust
/// Resolve the CLI invocation id. Priority: explicit span/log attribute
/// (the jackin shape — ids never live on Resource there), then resource
/// attribute (generic wrapped emitters), then the legacy parallax.run.id
/// resource attribute (read-only fallback inside the telemetry TTL).
fn invocation_id(signal_attrs: &[KeyValue], resource_attrs: &[KeyValue]) -> Option<String>
```

with lookup order `CLI_INVOCATION_ID` in `signal_attrs`, then in
`resource_attrs`, then `PARALLAX_RUN_ID` in `resource_attrs`. Add the parallel
`session_id()` (`SESSION_ID`, same order, no legacy fallback). Thread both
through `normalize_traces`/`normalize_logs`/`normalize_metrics`: rename row
fields `run_id` → `invocation_id`, add `session_id: Option<String>` to
`SpanRow` and `LogRow` (`crates/parallax-model/src/types.rs`). For spans, the
signal attrs of the **root span in each resource-spans group** win; child
spans inherit the group's resolved id (current behavior resolves one id per
resource group — keep that shape, just widen the sources). Update
`serde_contract.rs` guards.

**Verify**: `cargo nextest run --locked -p parallax-ingest -p parallax-model`
→ all pass, including new cases: id on root-span attrs only; id on resource
only; legacy key only; both (new wins); session id present/absent.

### Step 3: Storage columns and queries

`crates/parallax-greptime`:
- `ingest.rs`: extract-keys header becomes
  `service.name,cli.invocation.id,session.id,event.name,observed_ts_nanos,parallax.run.id`
  (legacy key last, kept for the fallback window).
- `lifecycle.rs`: pre-create promoted columns `"cli.invocation.id"` (SKIPPING
  INDEX) and `"session.id"` (SKIPPING INDEX) on `opentelemetry_logs` alongside
  the existing `"parallax.run.id"`; add the repair-ALTER path for existing
  installs (model on the existing `:337` repair). Create
  `invocation_metric_points` with `invocation_id` replacing `run_id`
  (same shape otherwise); on bootstrap, if legacy `run_metric_points` exists,
  `INSERT INTO invocation_metric_points SELECT ts, run_id, service, name,
  value, attributes FROM run_metric_points` then `DROP TABLE
  run_metric_points`. Same rename for the exemplar `run_id` column → new
  canonical exemplar DDL column `invocation_id` (follow the existing
  `migrate_metric_exemplars` pattern at `lifecycle.rs:176-`).
- Query layer: rename `run_store.rs` functions to invocation terms.
  `observed_invocations()` unions:
  - traces: `COALESCE(span-attr column "cli.invocation.id",
    resource_attributes."cli.invocation.id",
    resource_attributes."parallax.run.id")` — confirm the exact column shape
    the `greptime_trace_v1` pipeline produces for span attributes with the
    live-engine test before writing SQL (per
    `docs/research/decisions/native-otel-tables.md` every attribute gets its
    own column; the ident helpers live in `greptime_sql.rs:15-30`);
  - logs: `COALESCE("cli.invocation.id", "parallax.run.id")` over the promoted
    columns.
  `spans_by_invocation` filters the same COALESCE; `logs_by_invocation`
  likewise (`signal_queries.rs`, `query_sql.rs:176`, `transport.rs:316`).
- New projection queries (all bounded by limit + time window, all reading
  native tables only):
  - `sessions_by_invocation(invocation_id)` — from `opentelemetry_logs` where
    `event.name IN ('session.start','session.end')`, paired by `session.id`
    attr; returns session id, previous id, start/end nanos (end NULL = open).
  - `screen_visits(invocation_id | session_id)` — `event.name IN
    ('ui.screen.entered','ui.screen.exited')`, paired by `ui.screen.visit.id`;
    returns screen id, visit id, navigation sequence, entered/exited nanos.
  - `ui_actions(invocation_id, limit)` — root spans named `ui.action` from
    `opentelemetry_traces`; returns action name, screen id, duration, outcome,
    trace id.
  - `background_cycles(invocation_id?, from, to)` — spans named
    `background.cycle` grouped by `background.cycle.name`: count, error count,
    p50/p95 duration, last trace id.
  - `jobs(invocation_id?, from, to)` — spans with a `job.id` attribute:
    grouped by job id → job type, producer span (kind PRODUCER) time, consumer
    attempts (kind CONSUMER) with per-attempt outcome, last trace id.
  - `conversations(invocation_id)` — spans carrying `gen_ai.conversation.id`:
    conversation id, agent name, provider, first/last nanos, span count,
    token sums where `gen_ai.usage.*` metrics/attrs exist.
- Update `adapter.rs`/`adapter_rules.rs` identity-key lists
  (`invocation_id`, keep `session.id`), and the in-memory
  `parallax-test-support` stores to match the new trait surface.

**Verify**: `cargo nextest run --locked -p parallax-greptime -p parallax-storage
-p parallax-test-support` → pass;
`cargo nextest run --locked -p parallax-server -E 'binary(/greptime/)'` → pass
against a live engine, including one new live test proving (a) a span whose
ROOT carries `cli.invocation.id` as a span attribute resolves via
`observed_invocations`, (b) a legacy resource-attr-only emitter still
resolves, (c) the extract-keys promotion fills the `"cli.invocation.id"` log
column, and (d) `invocation_metric_points` receives the migrated legacy rows.

### Step 4: Turso invocations

`crates/parallax-metadata`: new bootstrap DDL

```sql
CREATE TABLE IF NOT EXISTS invocations (
  invocation_id TEXT PRIMARY KEY,
  command       TEXT,
  app_mode      TEXT,
  started_at    INTEGER NOT NULL,
  ended_at      INTEGER,
  exit_code     INTEGER,
  outcome       TEXT,
  status        TEXT NOT NULL DEFAULT 'running'
);
```

with a one-shot migration copying `runs` rows
(`invocation_id=run_id, app_mode=NULL, outcome=NULL`) and dropping `runs`.
Rename `turso/runs.rs` → `turso/invocations.rs`
(`start_invocation`/`finish_invocation`/`invocations()`), extending
`finish_invocation` to accept `outcome`.

**Verify**: `cargo nextest run --locked -p parallax-metadata` → pass,
including a migration test seeding a legacy `runs` row and asserting it
resolves from `invocations`.

### Step 5: GraphQL surface

`crates/parallax-api`:
- Rename: `run`→`invocation(invocation_id)`, `observed_runs`→
  `observed_invocations`, `traces_by_run`→`traces_by_invocation`,
  `logs_by_run`→`logs_by_invocation`, `logs(run_id…)`→`logs(invocation_id…)`,
  `story(run_id…)`/`evidence_gaps(run_id…)`→`invocation_id`,
  `runtime_snapshot(run_id…)`/`metric_series(run_id…)`/`bundle(run_id…)`→
  `invocation_id`; mutations `run_start`/`run_finish`→
  `invocation_start(invocation_id, command, app_mode, started_at_nanos)` /
  `invocation_finish(invocation_id, ended_at_nanos, exit_code, outcome)`.
  Types `Run`→`Invocation` (add `appMode`, `outcome`, derived `status`
  covering `running|finished|failed|stale` — stale = no end record and no
  signal newer than 5 minutes), `ObservedRun`→`ObservedInvocation` (add
  `lastCommand`/`appMode` when derivable from root spans). Pre-release:
  hard rename, no deprecated aliases.
- New query fields, one resolver module each (follow the existing
  one-file-per-domain layout under `resolvers/`): `sessions(invocation_id)`,
  `screen_visits(invocation_id, session_id)`, `ui_actions(invocation_id,
  limit)`, `background_cycles(invocation_id, from_nanos, to_nanos)`,
  `jobs(invocation_id, from_nanos, to_nanos)`,
  `conversations(invocation_id)` — thin mappings over the plan-step-3 storage
  queries. Keep `agent_session` working but re-anchor its argument to
  `invocation_id` (it stays the gen_ai step-timeline projection; the new
  `conversations` field is the summary list).
- Regenerate/commit whatever SDL artifact exists (check `cargo xtask ui
  graphql check` expectations before assuming a path).

**Verify**: `cargo nextest run --locked -p parallax-api` → pass;
`cargo xtask ui graphql check` → exit 0 (or the documented equivalent);
`grep -rn "run_id" crates/parallax-api/src | grep -v invocation` → no
production matches.

### Step 6: SSE, CLI producer, fingerprints

- `parallax-server/src/live.rs`: `StreamFilter.run_id`→`invocation_id` (+ new
  `session_id`), `SpanStreamFilter` likewise; wire-side JSON key
  `"invocationId"`; query params `invocation_id`, `session_id`. Update
  `live/tests.rs`.
- `parallax-cli`: the wrapper mints a UUIDv4 invocation id; injects
  `CLI_INVOCATION_ID` env AND appends `cli.invocation.id=<id>` to
  `OTEL_RESOURCE_ATTRIBUTES` for wrapped generic children
  (`forwarding.rs:139-140` — resource carriage is the pragmatic generic path;
  natively-instrumented apps like jackin❯ stamp their own attrs and ignore
  this); stops writing `parallax.run.id`; registration calls the renamed
  mutations. Rename user-facing flags/args (`parallax logs --run` etc.) to
  `--invocation`; update command help text.
- `parallax-analysis/src/derive.rs`: replace the four `JACKIN_OPERATION`
  sites with `CLI_COMMAND_NAME`-based grouping (same precedence slot);
  update `derive` tests and any fingerprint goldens.

**Verify**: `cargo nextest run --locked -p parallax-server -p parallax-cli
-p parallax-analysis` → pass; one CLI smoke:
`cargo run -p parallax-cli -- run -- echo hi` (or the wrapper's real
invocation shape — read `commands/runs.rs` first) registers an invocation
that `invocations` GraphQL returns.

### Step 7: Workspace closure

Full sweep: `grep -rn "PARALLAX_RUN_ID\|parallax.run.id" crates/ --include='*.rs'`
must show only (a) the semconv constant definition, (b) the ingest legacy
fallback, (c) greptime COALESCE/read paths + legacy DDL column, (d) fixtures
that explicitly test the fallback. Everything else is a defect. Update
`docs/research/decisions/native-otel-tables.md` extraction rows
(`cli.invocation.id`/`session.id` mechanism + legacy note) in the same commit.

**Verify**: `cargo xtask ci --fast` → exit 0; grep audit above clean.

## Test plan

- Unit: ingest priority order (5 cases, step 2); session pairing; projection
  SQL builders (screen visits pairing, job producer/consumer grouping,
  cycle aggregation) against the in-memory store.
- Live-engine (`*_greptime` lane): extract-keys promotion, span-attr column
  resolution, legacy COALESCE, `invocation_metric_points` migration,
  session/screen/action/cycle/job/conversation projections over a seeded
  OTLP corpus (extend `tests/support/harness.rs` seeding; model on
  `m2_api.rs`).
- API: one integration test per new resolver in `m2_api.rs` style.
- Migration: Turso `runs`→`invocations` copy; exemplar column migration.

## Done criteria

- [ ] `cargo xtask ci --fast` exits 0; live-greptime lane green.
- [ ] `grep -rn "jackin" crates/ --include='*.rs' | grep -v "jackin❯"` → no
  attribute-key matches (prose mentions of the product name are fine).
- [ ] Step-7 grep audit shows legacy key only in the allowed fallback sites.
- [ ] GraphQL exposes `invocations`/`invocation`/`sessions`/`screen_visits`/
  `ui_actions`/`background_cycles`/`jobs`/`conversations` and no `run`-named
  field (`grep -n "fn run\|observed_runs" crates/parallax-api/src/lib.rs` → 0).
- [ ] SSE accepts `invocation_id`/`session_id` and rejects nothing silently
  (unknown params ignored as today).
- [ ] `docs/research/decisions/native-otel-tables.md` updated in the rename
  commit.
- [ ] `plans/README.md` status row updated.

## STOP conditions

Stop and report back (do not improvise) if:
- The `greptime_trace_v1` pipeline does NOT materialize root-span attribute
  `cli.invocation.id` as a queryable column/path — that breaks decision 1;
  bring the live-probe evidence and consult before inventing a workaround
  (a custom raw-signal table is forbidden).
- Extract-keys behaves differently for log-record attrs vs resource attrs in
  a way that loses either source (probe both on the live engine first).
- `INSERT INTO … SELECT`/`DROP TABLE` migration of `run_metric_points` is
  unsupported by the pinned GreptimeDB — report the exact error; do not leave
  double-write paths.
- Any consumer outside this plan's scope (UI code beyond semconv.ts) fails to
  compile — that's plan 157's surface; coordinate the commit ordering (156
  backend + 157 UI land on the same branch; the tree may be red only between
  commits, never at a push without noting it).
- Removing `jackin.operation` changes issue fingerprints for stored data in a
  way the existing issue-identity tests reject — report; do not add a shim key.

## Maintenance notes

- The legacy `parallax.run.id` fallback has a removal trigger recorded in the
  index ledger — shrink-only, like every ratchet.
- New extension keys (future jackin registry additions) enter through
  `telemetry/semconv/contract.yaml` + regeneration, never as string literals.
- Reviewer focus: the ingest priority order (signal attr beats resource attr
  beats legacy), COALESCE order in every storage query, and that no metric
  path gained an invocation/session tag.
