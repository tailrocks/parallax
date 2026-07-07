# Plan 055: Log event identity — carry `event_name` and `observed_ts_nanos` end-to-end (ingest → row → GraphQL → Logs UI)

> **Executor instructions**: Follow this plan step by step. Run every
> verification command and confirm the expected result before moving on. On
> any STOP condition, stop and report — do not improvise. When done, update
> the status row for this plan in `plans/README.md`.
>
> **Drift check (run first)**:
> `git diff --stat ed5b10f..HEAD -- crates/parallax-storage/src/model.rs crates/parallax-core/src/normalize.rs crates/parallax-storage/src/greptime.rs crates/parallax-storage/src/memory.rs crates/parallax-api/src/lib.rs ui/src/components/logs-table.tsx ui/src/routes/logs.tsx`
> If any in-scope file changed since `ed5b10f`, compare the "Current state"
> excerpts against the live code before proceeding; on a mismatch, STOP.

## Status

- **Priority**: P2
- **Effort**: M
- **Risk**: MED (touches the log ingest row shape)
- **Depends on**: none (pairs with plan 056, which makes the playground emit
  typed events; this plan is still verifiable without it via the test suite)
- **Category**: direction
- **Planned at**: commit `ed5b10f`, 2026-07-07

## Why this matters

The OpenTelemetry Logs Data Model is stable and distinguishes a plain log
record from a **typed event** via `EventName`, and an emitted timestamp from
an **observed** one via `ObservedTimestamp`. Parallax currently discards both:
`LogRow` has no `event_name` field at all, and normalize collapses
`time_unix_nano`/`observed_time_unix_nano` into a single `ts_nanos`. That
blocks the research brief's "logs as typed evidence" direction — the Logs page
cannot show a Type/Event column, the story timeline cannot prefer typed events
over body text, and late-arriving logs lose the skew between when they
happened and when they were received. This plan adds both fields end-to-end so
every later log surface (context views, story, quality scoring) can build on
them.

## Current state

All excerpts verified at commit `ed5b10f`.

- `crates/parallax-storage/src/model.rs:30-43` — `LogRow` today:

  ```rust
  pub struct LogRow {
      pub ts_nanos: u128,
      pub service: String,
      pub severity_num: i32,
      pub severity_text: String,
      pub body: String,
      pub trace_id: String,
      pub span_id: String,
      pub run_id: Option<String>,
      pub scope_name: String,
      pub attributes: serde_json::Value,
      pub resource: serde_json::Value,
  }
  ```

  No `event_name`, no observed timestamp.

- `crates/parallax-core/src/normalize.rs:187-191` — the collapse:

  ```rust
  let ts = if record.time_unix_nano != 0 {
      record.time_unix_nano
  } else {
      record.observed_time_unix_nano
  };
  ```

  `record` is the OTLP `LogRecord` proto (crate `parallax-proto`); it carries
  an `event_name` string field in OTLP ≥ 1.5 protos — check the vendored
  proto version first (STOP condition if absent).

- `crates/parallax-storage/src/greptime.rs:367-385` — `select_logs` projects a
  fixed column list from the native `opentelemetry_logs` table (GreptimeDB's
  own OTLP-created schema):

  ```rust
  r#"SELECT CAST("timestamp" AS BIGINT) AS "ts_nanos",
            json_get_string("resource_attributes", '$."service.name"') AS "service",
            "severity_number", "severity_text", "body", "trace_id", "span_id",
            "parallax.run.id", "scope_name",
            json_to_string("log_attributes"),
            json_to_string("resource_attributes")
     FROM opentelemetry_logs WHERE {where_clause}{order}{limit_clause}"#
  ```

  and `log_row_from_row` (`greptime.rs:390-403`) maps positionally. Whether
  the native table already has an `event_name`-like or observed-timestamp
  column is **unknown at planning time** — Step 1 discovers it.

- `crates/parallax-storage/src/greptime.rs:148-155` — the "logs deviations"
  pattern: Parallax already ALTERs the native logs table post-creation
  (inverted index, fulltext index, `parallax.run.id` column). Any missing
  column is added the same way.

- `crates/parallax-api/src/lib.rs:275-312` — `LogRecord` GraphQL object
  exposes exactly the `LogRow` fields (`tsNanos`, `service`, `severityNum`,
  `severityText`, `body`, `traceId`, `spanId`, `runId`, `scopeName`,
  `attributes`, `resource`).

- `ui/src/components/logs-table.tsx:29-41` — `LogDoc` interface mirrors that
  field list; `OPTIONAL_LOG_COLUMNS = ["service", "trace", "scope"]`
  (`logs-table.tsx:43`); the doc viewer flattens fields in `docFields`
  (`logs-table.tsx:77-110`).

- `ui/src/routes/logs.tsx:169-201` — `loadLogs` selects the same fields in
  both the live and windowed GraphQL queries.

- The in-memory adapter (`crates/parallax-storage/src/memory.rs`) stores
  `LogRow` directly — it inherits new fields for free once the struct grows,
  but its tests construct `LogRow` literals that must be updated.

- Repo conventions: Rust ingest is zero-copy-minded (decode once, move
  forward — AGENTS.md); GraphQL fields are camelCase snake-mapped by Juniper;
  UI TypeScript is strictest-mode; Bun only.

## Commands you will need

| Purpose | Command (repo root unless noted) | Expected |
|---------|----------------------------------|----------|
| Rust build | `rtk cargo build --workspace` | exit 0 |
| Rust lint | `rtk cargo clippy --workspace --all-targets` | exit 0, zero warnings |
| Rust fmt | `rtk cargo fmt --all` | no diff after |
| Rust tests | `rtk cargo nextest run` | all pass |
| UI (from `ui/`) | `bun run typecheck && bun run lint && bun run test && bun run build` | all exit 0 |
| Live schema check | `parallax serve` + SQL page: `DESCRIBE opentelemetry_logs` | column list |

## Scope

**In scope**:
- `crates/parallax-storage/src/model.rs` (`LogRow` + two fields)
- `crates/parallax-core/src/normalize.rs` (`normalize_logs`)
- `crates/parallax-storage/src/greptime.rs` (`select_logs`,
  `log_row_from_row`, `try_logs_deviations`, `logs_search` projection if it
  duplicates the column list)
- `crates/parallax-storage/src/memory.rs` (struct-literal updates only)
- `crates/parallax-api/src/lib.rs` (`LogRecord` object: `eventName`,
  `observedTsNanos` fields)
- `ui/src/components/logs-table.tsx` (`LogDoc`, optional `event` column,
  doc-viewer rows)
- `ui/src/routes/logs.tsx` (query field lists)
- Tests alongside each layer

**Out of scope** (do NOT touch):
- Pattern grouping, saved views, logsAround — plan 057.
- Story timeline consumption of typed events — advisor-plans/029 extension,
  later.
- Issue derivation changes (`derive.rs`) — event-typed derivation is a
  separate decision; do not add `event_name` handling there.
- The playground repo — plan 056 emits the events.
- `severity`/body handling — unchanged.

## Git workflow

- Repo `main` (BRANCHING.md), Conventional Commits, `git commit -s`, exactly
  one `Co-authored-by: Claude <noreply@anthropic.com>` trailer. Push when
  done.

## Steps

### Step 1: Discover the native logs schema

Run `parallax serve` with GreptimeDB storage and execute
`DESCRIBE opentelemetry_logs` (SQL page or `sql` resolver). Record which of
these exist: an event-name column (any of `event_name`, `"event.name"`) and
an observed-timestamp column (`observed_timestamp` or similar). GreptimeDB's
native OTLP logs pipeline versions differ here.

- If a native column exists → Step 3 projects it directly.
- If not → Step 3 adds it via `try_logs_deviations` (the existing ALTER
  pattern at `greptime.rs:148-155`) and Step 2's ingest must ensure the value
  lands there. **If the ingest path is header-driven (`forward_otlp` at
  `greptime.rs:184-199` forwards raw OTLP and GreptimeDB does the mapping)
  and GreptimeDB drops `event_name` entirely, STOP** and report: the fix then
  needs an extract-keys header or upstream issue, not guesswork.

**Verify**: recorded column list in your report.

### Step 2: Grow `LogRow` and `normalize_logs`

1. `model.rs`: add to `LogRow`:

   ```rust
   /// OTel `EventName` — non-empty only for typed log events.
   pub event_name: String,
   /// OTel `ObservedTimestamp` (ns). 0 when the source didn't set it.
   pub observed_ts_nanos: u128,
   ```

2. `normalize.rs` (`normalize_logs`, around lines 178-205): populate
   `event_name: record.event_name.clone()` (confirm the proto field name in
   `parallax-proto`; OTLP calls it `event_name`) and
   `observed_ts_nanos: u128::from(record.observed_time_unix_nano)`. Keep the
   existing `ts_nanos` fallback exactly as is.
3. Fix every `LogRow` literal that no longer compiles (memory.rs, tests,
   fixtures) — mechanical, `event_name: String::new()`,
   `observed_ts_nanos: 0` where the fixture has no opinion.

**Verify**: `rtk cargo build --workspace` → exit 0;
`rtk cargo nextest run -p parallax-core` → pass, including a new
`normalize_logs` test asserting both fields round-trip from a proto fixture.

### Step 3: GreptimeDB read path

In `greptime.rs`:
1. Extend the `select_logs` projection (and any duplicate column list used by
   `logs_search` — grep `severity_number` in the file to find them all) with
   the two columns discovered/added in Step 1, keeping positional order in
   sync with `log_row_from_row`.
2. If Step 1 found no native column: append the ALTER(s) to
   `try_logs_deviations` (same idempotent style as
   `ADD COLUMN "parallax.run.id" STRING`).
3. Use lenient reads: absent/NULL values map to `String::new()` / `0` (match
   the `opt_str_at`/`u128_at` helpers' behavior at `greptime.rs:390-403`).

**Verify**: `rtk cargo nextest run -p parallax-storage` → pass. With a live
stack: send one OTLP log that has `event_name` set (Step 6 fixture) and
`SELECT event_name FROM opentelemetry_logs LIMIT 5` returns it.

### Step 4: GraphQL surface

In `crates/parallax-api/src/lib.rs`, on the `LogRecord` object
(`lib.rs:277-312`), add:

```rust
/// OTel EventName — set only for typed log events (empty for plain logs).
fn event_name(&self) -> &str { &self.0.event_name }
/// OTel ObservedTimestamp in ns ("0" when the source didn't set it).
fn observed_ts_nanos(&self) -> String { nanos_string(self.0.observed_ts_nanos) }
```

**Verify**: `rtk cargo build -p parallax-api` → exit 0; existing API tests
pass (`rtk cargo nextest run -p parallax-api`).

### Step 5: UI — event column + doc viewer

1. `logs-table.tsx`: add `eventName: string` and `observedTsNanos: string` to
   `LogDoc` (`:29-41`). Add `"event"` to `OPTIONAL_LOG_COLUMNS` (`:43`) — OFF
   by default (the default list at `parseLogColumns:49` stays
   `["service", "trace"]`). Render the column as the raw `eventName` text,
   dash when empty. In `docFields` (`:77-110`): after the `severity` row, add
   `["event.name", log.eventName]` when non-empty; add an
   `["@observed", <ISO>]` row when `observedTsNanos !== "0"` and it differs
   from `tsNanos` by more than 1s (reuse the `@timestamp` formatting at
   `:79-82`).
2. `logs.tsx`: add `eventName observedTsNanos` to both GraphQL selections in
   `loadLogs` (`:179` and `:196-198`) and to the live-stream row mapping if
   the SSE payload is separately typed (grep `severityNum` in the file to
   find every projection).
3. Run detail reuses `LogDoc` (stated at `logs-table.tsx:27-28`) — grep
   `LogDoc` across `ui/src` and update any other query that builds them
   (e.g. `runs.$runId.tsx`).

**Verify** (from `ui/`): `bun run typecheck && bun run lint && bun run test`
→ exit 0; new/updated component test asserts the event column renders when
toggled and the doc viewer shows `event.name`.

### Step 6: End-to-end check

With `parallax serve` running, emit one typed log event (any OTLP source; if
plan 056 landed, `scenarios/run.sh` has one — otherwise use a small
`examples/seed.rs`-style fixture or `curl` an OTLP/HTTP JSON payload with
`event_name` set). Open `/logs`, enable the Event column, confirm the value
appears and the doc viewer shows `event.name` + `@observed`.

**Verify**: screenshot or recorded observation in the commit body.

## Test plan

- `parallax-core`: `normalize_logs` fixture with `event_name` +
  `observed_time_unix_nano` set → both fields populated; unset → empty/0.
- `parallax-storage` (memory): `LogRow` round-trip with the new fields.
- `parallax-api`: `logs` query selects `eventName observedTsNanos` (extend an
  existing logs resolver test — grep `logs(` in `crates/parallax-api/tests/`
  or the in-file `#[cfg(test)]` module and model after it).
- UI: extend the logs-table test (co-located `*.test.tsx` if present; else
  follow the repo's existing component-test pattern — check
  `ui/src/components/*.test.*` for the exemplar).

## Done criteria

ALL must hold:

- [ ] `rtk cargo build --workspace && rtk cargo clippy --workspace --all-targets` → exit 0, zero warnings
- [ ] `rtk cargo nextest run` → all pass, incl. new normalize/storage/api tests
- [ ] `cd ui && bun run typecheck && bun run lint && bun run test && bun run build` → all exit 0
- [ ] `rtk grep -n "event_name" crates/parallax-storage/src/model.rs` → 1 hit in `LogRow`
- [ ] `rtk grep -n "eventName" ui/src/routes/logs.tsx` → present in both query selections
- [ ] Live check (Step 6) recorded, or explicitly reported blocked with reason
- [ ] `plans/README.md` status row updated

## STOP conditions

- The vendored OTLP proto (`parallax-proto`) has no `event_name` field on
  `LogRecord` — report the proto version; upgrading protos is its own change.
- GreptimeDB's native logs pipeline cannot store `event_name` via ALTER +
  forwarded OTLP (Step 1/3 discovery) — report findings; do not switch the
  logs ingest to a custom INSERT path in this plan.
- `select_logs`' positional row mapping turns out to be used by more callers
  than `log_row_from_row` — re-check every caller before reordering columns
  (append-only is safest).
- Any in-scope excerpt no longer matches (drift).

## Maintenance notes

- Plan 056 (playground typed events) makes this visible in demos; plan 057
  (context/saved views) and advisor-plans/029 (story) are the next consumers
  — story should prefer `event_name` beats over body parsing once both land.
- The `observed_ts_nanos` skew row in the doc viewer is deliberately
  threshold-gated (1s) — reviewers should check the threshold reads clearly.
- Deferred: deriving issues from typed exception events by `event_name`
  (today derivation keys on attributes/severity only — `derive.rs`); decide
  when a real producer emits `event.name=exception`.
