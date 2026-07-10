# Plan 070: Fix four confirmed backend bugs — request-rate table lookup, ingest panic, first_seen regression, stuck runs

> **Executor instructions**: Follow this plan step by step. Run every
> verification command and confirm the expected result before moving to the
> next step. If anything in the "STOP conditions" section occurs, stop and
> report — do not improvise. When done, update the status row for this plan
> in `advisor-plans/README.md` — unless a reviewer dispatched you and told you
> they maintain the index.
>
> **Drift check (run first)**: `git diff --stat dbaba3c..HEAD -- crates/parallax-storage/src/greptime.rs crates/parallax-storage/src/metadata.rs crates/parallax-cli/src/commands.rs`
> If any in-scope file changed since this plan was written, compare the
> "Current state" excerpts against the live code before proceeding; on a
> mismatch, treat it as a STOP condition.

## Status

- **Priority**: P1
- **Effort**: M (four small fixes + tests)
- **Risk**: LOW
- **Depends on**: none
- **Category**: bug
- **Planned at**: commit `dbaba3c`, 2026-07-10

## Why this matters

Four independently-confirmed bugs, each small, each user-visible:

1. `ServiceOverview.requestRate` is silently always empty for the standard
   OTel metric names because the query targets a table name that never exists.
2. A failing GreptimeDB statement whose SQL has a multi-byte UTF-8 character
   straddling byte 200 panics the process's **single** ingest worker task,
   silently halting all ingest processing until restart.
3. An issue's `first_seen` timestamp can never move earlier, so out-of-order
   ingestion records a too-late first sighting.
4. A wrapped command that fails to spawn (e.g. binary not found) leaves its
   run stuck in `running` state forever.

## Current state

### Bug 1 — `histogram_count_series` queries a non-existent table

- `crates/parallax-storage/src/greptime.rs:2495-2529` — the function
  interpolates the *display* metric name directly into the table name:

  ```rust
  // greptime.rs:2511-2518
  let rows = self
      .sql_lenient(&format!(
          r#"SELECT CAST(date_bin(INTERVAL '{step_secs} seconds', "greptime_timestamp") AS BIGINT)
                    AS "bucket_ms", SUM("greptime_value") AS "samples"
             FROM "{}_count"
             WHERE "greptime_timestamp" >= {} AND "greptime_timestamp" <= {}{service_clause}
             GROUP BY "bucket_ms" ORDER BY "bucket_ms""#,
          escape_ident(name),
  ```

- The caller iterates dotted OTel names —
  `crates/parallax-core/src/semconv.rs` defines `REQUEST_DURATION_METRICS` as
  `["http.server.request.duration", "rpc.server.duration"]`, and
  `crates/parallax-api/src/lib.rs:1455-1475` (`ServiceOverview::request_rate`)
  calls `histogram_count_series(name, ...)` with each. GreptimeDB's metric
  engine stores the count sibling as the Prometheus-normalized name
  (`http_server_request_duration_count`), so `"http.server.request.duration_count"`
  never exists; `sql_lenient` swallows the table-not-found error and the field
  returns an empty series.

- The correct pattern already exists in the same file — `histogram_quantile`
  at `greptime.rs:1709-1719`:

  ```rust
  let Some(bucket_table) = self.metric_table_for_name(name, Some("_bucket")).await? else {
      return Ok(Vec::new());
  };
  ```

  `metric_table_for_name` (`greptime.rs:495-522`) resolves the real table via
  `metric_table_candidates` (`greptime.rs:80`, which includes the
  `native_metric_base` dotted→underscore normalization) checked against
  `information_schema.tables`.

### Bug 2 — byte-slice panic in SQL error formatting

- `crates/parallax-storage/src/greptime.rs:419-425`:

  ```rust
  if let Some(error) = response.get("error").and_then(|e| e.as_str()) {
      let code = response.get("code").and_then(|c| c.as_i64()).unwrap_or(-1);
      anyhow::bail!(
          "greptime sql failed (code {code}): {error} — sql: {}",
          &sql[..sql.len().min(200)]
      );
  }
  ```

  `&sql[..200]` is a **byte** slice; if byte 200 is not a UTF-8 char boundary
  it panics. Telemetry text (error messages, attribute values — often
  non-ASCII) is interpolated into INSERT statements, so a failing INSERT can
  hit this. The panic propagates into the single ingest worker task spawned at
  `crates/parallax-server/src/serve.rs:287`; the worker loop
  (`crates/parallax-server/src/worker.rs:58-64`) only catches `Err`, not
  panics, so the task dies and all subsequent ingest items queue forever.

### Bug 3 — `first_seen` never lowered

- `crates/parallax-storage/src/metadata.rs:164-172`:

  ```rust
  conn.execute(
      "INSERT INTO issues
             (fingerprint, title, error_type, culprit, service,
              first_seen, last_seen, event_count, last_trace_id)
           VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6, 1, ?7)
           ON CONFLICT(fingerprint) DO UPDATE SET
             last_seen = MAX(last_seen, excluded.last_seen),
             event_count = event_count + 1,
             last_trace_id = COALESCE(excluded.last_trace_id, last_trace_id)",
  ```

  `last_seen` takes `MAX`, but there is no `first_seen = MIN(first_seen,
  excluded.first_seen)`, so an earlier-timestamped occurrence processed later
  leaves `first_seen` wrong.

### Bug 4 — wrapper run stuck `running` on spawn failure

- `crates/parallax-cli/src/commands.rs:257-274`:

  ```rust
  let mut cmd = tokio::process::Command::new(&command[0]);
  cmd.args(&command[1..]);
  for (key, value) in &pairs {
      cmd.env(key, value);
  }
  let status = cmd.status().await?;          // <- early-returns on spawn error
  let exit_code = status.code().unwrap_or(-1);

  client
      .graphql(&format!(
          r#"mutation {{ runFinish(runId: "{}", endedAtNanos: "{}", exitCode: {exit_code}) }}"#,
          gql_str(&run_id),
          now_nanos()
      ))
      .await?;
  ```

  By this point the run was already registered as `running` via `runStart`
  earlier in the function. If `cmd.status()` errors (command not found,
  permission denied), the `?` returns before `runFinish`, so the run stays
  `running` forever in the runs list/UI. Helpers: `gql_str` is
  `crates/parallax-cli/src/client.rs:118`, `now_nanos` is
  `crates/parallax-cli/src/commands.rs:172`.

### Conventions

- Errors: `anyhow` everywhere in these crates; match existing style.
- Tests: inline `#[cfg(test)]` modules; async tests use `#[tokio::test]`.
  `metadata.rs` already has a test module near line 1000 using an in-memory
  Turso database — model Bug 3's test on those.
- Zero clippy warnings enforced (`cargo clippy --workspace --all-targets`
  with `-D warnings` in CI).

## Commands you will need

| Purpose | Command | Expected on success |
|---------|---------|---------------------|
| Build | `rtk cargo build --workspace` | exit 0 |
| Tests | `rtk cargo nextest run --workspace` | all pass |
| One test file | `rtk cargo nextest run -p parallax-storage metadata` | pass |
| Lint | `rtk cargo clippy --workspace --all-targets` | zero warnings |
| Format | `rtk cargo fmt --all` | exit 0 |

## Scope

**In scope** (the only files you should modify):
- `crates/parallax-storage/src/greptime.rs` (Bugs 1, 2 + their tests)
- `crates/parallax-storage/src/metadata.rs` (Bug 3 + test)
- `crates/parallax-cli/src/commands.rs` (Bug 4)
- `advisor-plans/README.md` (status row)

**Out of scope** (do NOT touch, even though they look related):
- `crates/parallax-storage/src/memory.rs` — the in-memory adapter has its own
  `histogram_count_series`; its semantics are not part of this fix.
- `crates/parallax-server/src/worker.rs` — making the worker panic-resilient
  or adding retry is Plan 073's territory (spool durability).
- `crates/parallax-api/src/lib.rs` — the `request_rate` caller is correct;
  only the storage lookup is wrong.
- Any SQL query other than the two lines being fixed.

## Git workflow

- Work directly on `main` (repo rule — `BRANCHING.md`).
- One commit per bug or one batch commit; Conventional Commits, DCO signoff:
  e.g. `fix(storage): resolve native count table for request rate`, commit
  with `git commit -s`, trailer `Co-authored-by: Claude <noreply@anthropic.com>`.

## Steps

### Step 1: Fix the count-table lookup (Bug 1)

In `histogram_count_series` (`greptime.rs`, starts ~line 2495), before
building the SQL, resolve the real table exactly as `histogram_quantile` does:

```rust
let Some(count_table) = self.metric_table_for_name(name, Some("_count")).await? else {
    return Ok(Vec::new());
};
```

and change the SQL's `FROM "{}_count"` + `escape_ident(name)` to
`FROM "{}"` + `escape_ident(&count_table)`.

**Verify**: `rtk cargo build -p parallax-storage` → exit 0.

### Step 2: Add a unit test for the candidate expansion (Bug 1)

`metric_table_candidates` (`greptime.rs:80`) is a pure function. Add a test in
the existing `#[cfg(test)]` module at the bottom of `greptime.rs` (near the
existing `escape_ident_doubles_double_quotes_only` test) asserting that
`metric_table_candidates("http.server.request.duration", Some("_count"))`
contains `"http_server_request_duration_count"`. This pins the normalization
the fix relies on.

**Verify**: `rtk cargo nextest run -p parallax-storage greptime` → new test passes.

### Step 3: Fix the byte-slice panic (Bug 2)

Replace the truncation at `greptime.rs:423` with a char-boundary-safe form:

```rust
let sql_prefix: String = sql.chars().take(200).collect();
anyhow::bail!("greptime sql failed (code {code}): {error} — sql: {sql_prefix}");
```

Add a unit test in the same test module proving the pattern: build a `String`
of 300 multi-byte chars (e.g. `"é".repeat(300)`), apply the same
`chars().take(200).collect::<String>()` expression, assert it doesn't panic
and has 200 chars. (The old `&s[..200]` form would panic on this input — you
can assert `!s.is_char_boundary(200)` to prove the fixture is adversarial.)

**Verify**: `rtk cargo nextest run -p parallax-storage greptime` → passes.

### Step 4: Lower `first_seen` on conflict (Bug 3)

In `metadata.rs:169-172` add one line to the `ON CONFLICT` clause:

```sql
first_seen = MIN(first_seen, excluded.first_seen),
```

Add a test in the existing `metadata.rs` test module: upsert an occurrence at
t=2000ms, then upsert the same fingerprint at t=1000ms, read the issue back,
assert `first_seen == 1000` and `last_seen == 2000`. Model it on the
neighboring upsert tests (in-memory Turso setup).

**Verify**: `rtk cargo nextest run -p parallax-storage metadata` → new test passes.

### Step 5: Always attempt `runFinish` (Bug 4)

In `commands.rs` around lines 257-274, restructure so the `runFinish` mutation
is attempted even when the child fails to spawn:

```rust
let status = cmd.status().await;
let exit_code = match &status {
    Ok(status) => status.code().unwrap_or(-1),
    Err(_) => -1,
};

let finish = client
    .graphql(&format!(
        r#"mutation {{ runFinish(runId: "{}", endedAtNanos: "{}", exitCode: {exit_code}) }}"#,
        gql_str(&run_id),
        now_nanos()
    ))
    .await;

let status = status?;      // propagate the spawn error AFTER finishing the run
finish?;                   // then any finish error
```

Keep the two `println!` lines after this block; on the spawn-error path they
are skipped because `status?` returns first (acceptable: the error message
reaches the user via anyhow).

**Verify**: `rtk cargo build -p parallax-cli` → exit 0. Manual check (only if a
local server is already running — otherwise skip and note it):
`parallax run start -- /nonexistent-binary-xyz` → command errors, then
`parallax run list` shows the run as finished/failed, not `running`.

### Step 6: Full gates

**Verify**: `rtk cargo fmt --all` then
`rtk cargo clippy --workspace --all-targets` → zero warnings, and
`rtk cargo nextest run --workspace` → all pass.

## Test plan

- `greptime.rs` test module: `metric_table_candidates` normalization test
  (Step 2), char-boundary truncation test (Step 3).
- `metadata.rs` test module: `first_seen` MIN regression test (Step 4),
  following the existing in-memory Turso test pattern in that file.
- Bug 4 has no unit harness (spawns a process + needs a server); verified by
  build + optional manual check. Note this in the commit message.
- The real-SQL behavior of Bug 1 is covered end-to-end by the ignored
  integration tests (`m2_metrics_greptime.rs`) only if they assert request
  rate — do not extend those here; Plan 074 owns SQL-layer testing.

## Done criteria

Machine-checkable. ALL must hold:

- [ ] `grep -n 'metric_table_for_name(name, Some("_count"))' crates/parallax-storage/src/greptime.rs` → 1 match
- [ ] `grep -n 'sql\[\.\.' crates/parallax-storage/src/greptime.rs` → 0 matches
- [ ] `grep -n 'MIN(first_seen' crates/parallax-storage/src/metadata.rs` → 1 match
- [ ] `rtk cargo nextest run --workspace` exits 0 with 3 new tests
- [ ] `rtk cargo clippy --workspace --all-targets` → zero warnings
- [ ] `git status` shows no modified files outside the in-scope list
- [ ] `advisor-plans/README.md` status row updated

## STOP conditions

Stop and report back (do not improvise) if:

- `metric_table_for_name` requires `&mut self` or is otherwise uncallable from
  `histogram_count_series` (signature drift).
- The `ON CONFLICT` clause in `metadata.rs` no longer matches the excerpt
  (schema/upsert refactored since planning).
- Turso rejects `MIN(first_seen, excluded.first_seen)` syntax in the conflict
  clause (dialect gap) — report; do not emulate with read-modify-write, that
  changes concurrency semantics.
- The `commands.rs` wrapper flow has been restructured (e.g. already handles
  spawn errors) — re-read and report what remains.

## Maintenance notes

- Bug 2's deeper issue — the ingest worker is a single task with no panic
  isolation — is deliberately deferred to Plan 073 (spool durability); this
  plan only removes the known panic source.
- Bug 3 matters more once spool replay (Plan 073) exists: replay makes
  out-of-order delivery routine.
- Reviewer: check Step 5 preserves the exit-code contract (`run_wrapper`
  returns the child's exit code so `parallax run start -- cmd` proxies it).
