# Plan 022: Close the SQL-injection surface in storage query builders and tighten the read-only SQL guard

> **Executor instructions**: Follow this plan step by step. Run every
> verification command and confirm the expected result before moving to the
> next step. If anything in the "STOP conditions" section occurs, stop and
> report — do not improvise. When done, update the status row for this plan
> in `advisor-plans/README.md`.
>
> **Drift check (run first)**: `git diff --stat 8bc3f13..HEAD -- crates/parallax-storage/src/greptime.rs crates/parallax-api/src/lib.rs`
> If either file changed since this plan was written, compare the "Current
> state" excerpts against the live code before proceeding; on a mismatch,
> treat it as a STOP condition.

## Status

- **Priority**: P1
- **Effort**: S
- **Risk**: LOW
- **Depends on**: none
- **Category**: security
- **Planned at**: commit `8bc3f13`, 2026-07-07

## Why this matters

The GraphQL `metricSeries`, `histogramQuantile`, and related resolvers pass a
caller-supplied metric `name` into a SQL **identifier** position
(`FROM "{name}"`), but the storage layer's `escape()` helper only doubles
single quotes — it never neutralizes the double quotes that delimit
identifiers. A metric name containing `"` closes the identifier and injects
arbitrary SQL into GreptimeDB's `/v1/sql`. This path has none of the `sql()`
field's guards, so it is a wider injection surface than the intended raw-SQL
feature. Metric names also originate from OTLP data, so a hostile telemetry
producer can plant a malicious name that later detonates when queried.
Separately, the `sql()` field's read-only guard is a prefix allowlist that
admits `EXPLAIN` — and on DataFusion-backed engines `EXPLAIN ANALYZE <stmt>`
*executes* the analyzed plan, defeating the write guard.

## Current state

- `crates/parallax-storage/src/greptime.rs:25-27` — the only escaping helper:

  ```rust
  fn escape(text: &str) -> String {
      text.replace('\'', "''")
  }
  ```

- `crates/parallax-storage/src/greptime.rs:1003-1012` — `metric_series`
  (native branch) interpolates the name inside a double-quoted identifier:

  ```rust
  self.sql_lenient(&format!(
      r#"SELECT CAST(date_bin(INTERVAL '{step_secs} seconds', "greptime_timestamp") AS BIGINT)
                AS "bucket_ms", {sql_agg}("greptime_value") AS "agg_value"
         FROM "{}"
         WHERE "greptime_timestamp" >= {} AND "greptime_timestamp" <= {}{service_clause}
         GROUP BY "bucket_ms" ORDER BY "bucket_ms""#,
      escape(name),
      ...
  ```

  Same pattern: `greptime.rs:1044-1055` (`histogram_quantile`, `FROM "{}_bucket"`),
  and the `metric_series_grouped` / `histogram_count_series` implementations
  (search `FROM "{}` in the file — every hit is in scope).

- `crates/parallax-api/src/lib.rs:1620-1671` — the `metricSeries` resolver
  passes the raw GraphQL `name: String` straight through to
  `store.metric_series(&name, …)` / `store.metric_series_grouped(&name, …)`
  with no validation. `histogramQuantile` (`lib.rs:1681`) does the same.

- `crates/parallax-api/src/lib.rs:1207-1229` — the `sql()` guard:

  ```rust
  let read_only = [
      "select", "with", "show", "describe", "desc", "explain", "tql",
  ]
  .iter()
  .any(|prefix| lowered.starts_with(prefix));
  ...
  if trimmed.trim_end_matches(';').contains(';') {
      return Err(field_err("multiple statements are not allowed"));
  }
  ```

- `crates/parallax-storage/src/adapter.rs:271-274` — `raw_sql` documents that
  "callers enforce the read-only guard"; `greptime.rs` executes verbatim.
- The in-memory store (`crates/parallax-storage/src/memory.rs`) never builds
  SQL, so it needs no change; its `raw_sql` errors by design.
- Repo conventions: zero clippy warnings, cargo-nextest as test runner
  (root `AGENTS.md`), tests co-located in `#[cfg(test)]` modules or
  `crates/parallax-server/tests/`.

## Commands you will need

| Purpose   | Command (repo root)                                                    | Expected on success |
|-----------|------------------------------------------------------------------------|---------------------|
| Format    | `rtk cargo fmt --all`                                                  | exit 0              |
| Lint      | `rtk cargo clippy --workspace --all-targets --locked -- -D warnings`   | exit 0, no warnings |
| Tests     | `rtk cargo nextest run --workspace`                                    | all pass            |

## Scope

**In scope** (the only files you should modify):
- `crates/parallax-storage/src/greptime.rs`
- `crates/parallax-api/src/lib.rs`

**Out of scope** (do NOT touch, even though they look related):
- `crates/parallax-storage/src/memory.rs` — no SQL there.
- The `/graphql` handler and server config (`crates/parallax-server/`) —
  plan 024 covers server hardening.
- Any change to the GraphQL schema shape (no new/removed fields).
- GreptimeDB engine-level roles/permissions — out of V1.

## Git workflow

- Work directly on `main` (repo rule, `BRANCHING.md`).
- Conventional Commits, DCO signoff, exactly one agent trailer:
  `git commit -s -m "fix(storage): …"` +
  `Co-authored-by: Claude <noreply@anthropic.com>`.
- Push after the plan is done (repo rule in `AGENTS.md`).

## Steps

### Step 1: Add an identifier-escaping helper and use it at every `FROM "{}"` site

In `greptime.rs`, next to `escape()` (line 25), add:

```rust
/// Escape a value for inclusion inside a double-quoted SQL *identifier*.
/// `escape()` protects single-quoted string literals; identifiers need the
/// double quote doubled instead.
fn escape_ident(text: &str) -> String {
    text.replace('"', "\"\"")
}
```

Replace `escape(name)` with `escape_ident(name)` at **every** site where the
value lands inside `FROM "…"` (and only those sites — string-literal
positions keep `escape()`). Find them all with:
`grep -n 'FROM "{}' crates/parallax-storage/src/greptime.rs`
Expected sites (verify against live code): `metric_series` native branch,
`histogram_quantile` (`_bucket`), `metric_series_grouped`,
`histogram_count_series` (`_count`). Check whether the `group_by` argument in
`metric_series_grouped` is also interpolated into an identifier or JSON-path
position; if it is only stripped of `"` characters today, route it through
`escape_ident` (or keep the strip **and** add `escape_ident` — belt and
braces is fine here).

**Verify**: `rtk cargo clippy --workspace --all-targets --locked -- -D warnings` → exit 0.

### Step 2: Reject malformed metric names at the API boundary

In `crates/parallax-api/src/lib.rs`, add a validation helper near
`clamp_limit` (line 46):

```rust
/// Metric names come from OTLP data and flow into SQL identifiers; reject
/// anything outside the OTel metric-name grammar before it reaches storage.
fn validate_metric_name(name: &str) -> FieldResult<()> {
    let ok = !name.is_empty()
        && name.len() <= 255
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-' | '/'));
    if ok { Ok(()) } else { Err(field_err("invalid metric name")) }
}
```

Call it first in `metric_series` (line 1620) and `histogram_quantile`
(line 1681) resolvers. This is defense-in-depth on top of Step 1, and it also
keeps the `_bucket`/`_count` suffix composition in `histogram_quantile` sane.

**Verify**: `rtk cargo nextest run --workspace` → all pass (no behavior change
for legal names).

### Step 3: Close the `EXPLAIN ANALYZE` hole in the `sql()` guard

In the `sql()` resolver (`lib.rs:1207`), after the existing prefix check,
reject the analyze form explicitly:

```rust
if lowered.starts_with("explain") && lowered.contains("analyze") {
    return Err(field_err(
        "EXPLAIN ANALYZE executes the statement and is not allowed; use EXPLAIN",
    ));
}
```

Keep plain `EXPLAIN` working. Do not attempt a full SQL parser in this plan.

**Verify**: `rtk cargo nextest run --workspace` → all pass.

### Step 4: Add a defensive statement check inside `raw_sql`

In `greptime.rs`'s `raw_sql` implementation, before executing, repeat the
cheap shape check so the invariant no longer depends solely on callers:
reject if the trimmed lowered query does not start with one of
`select|with|show|describe|desc|explain|tql`, or contains an interior `;`,
or starts with `explain` and contains `analyze`. Return
`anyhow::bail!("raw_sql: read-only statements only")`. Update the trait doc
comment in `adapter.rs:271-274` to say the guard is enforced both at the API
layer and defensively in the adapter.
(`adapter.rs` doc-comment edit is allowed as part of this step even though
the file is not in the modify list — it is a comment-only change.)

**Verify**: `rtk cargo clippy --workspace --all-targets --locked -- -D warnings` → exit 0.

### Step 5: String-level tests (no engine required)

The SQL-building layer is only exercised by `#[ignore]` integration tests
today, so add fast unit tests in a `#[cfg(test)]` module at the bottom of
`greptime.rs`:

- `escape_ident` doubles `"` and leaves other chars alone.
- (If you extracted any pure query-building helper in Step 1, assert a name
  like `x" UNION SELECT` renders inertly inside the identifier.)

And in `lib.rs`'s existing test module (search `#[cfg(test)]` near the bottom;
tests there build a schema over the memory store):

- `sql` rejects `EXPLAIN ANALYZE SELECT 1` with the Step 3 message.
- `sql` still accepts a plain `SELECT` (memory store returns its
  "no SQL surface" error — asserting the guard error *differs* from the
  pass-through error is enough).
- `metricSeries` with name `evil"name` returns the "invalid metric name"
  error; with `http.server.request.duration` it succeeds against the memory
  store.

**Verify**: `rtk cargo nextest run --workspace` → all pass, including the new
tests (name them so `nextest run -E 'test(sql_guard)'` style filtering works).

## Test plan

Covered by Step 5. New cases: identifier escaping, metric-name validation
(accept/reject), `EXPLAIN ANALYZE` rejection, plain `EXPLAIN`/`SELECT`
acceptance. Model resolver-level tests on the existing schema tests at the
bottom of `crates/parallax-api/src/lib.rs`.

## Done criteria

Machine-checkable. ALL must hold:

- [ ] `rtk cargo fmt --all` produces no diff; clippy exits 0 with `-D warnings`
- [ ] `rtk cargo nextest run --workspace` exits 0 with the new tests present
- [ ] `grep -n 'FROM "{}' crates/parallax-storage/src/greptime.rs` — every hit
      uses `escape_ident(` (manual inspection of each hit's argument)
- [ ] `grep -c "escape_ident" crates/parallax-storage/src/greptime.rs` ≥ 5
- [ ] No files outside the in-scope list modified (`git status`), except the
      `adapter.rs` doc comment from Step 4
- [ ] `advisor-plans/README.md` status row updated

## STOP conditions

Stop and report back (do not improvise) if:

- The code at the cited locations doesn't match the excerpts (drift).
- You find a `FROM "{}"`-style site whose value is **not** a metric name or
  group-by key (i.e. a new query builder landed since 8bc3f13) — report it
  instead of guessing its grammar.
- GreptimeDB integration tests (`cargo nextest run --run-ignored all`) are
  being relied on by CI in a way that the new validation breaks.
- Rejecting non-ASCII metric names breaks an existing test fixture — that
  means real metric names are wider than the grammar above; report, don't
  silently widen.

## Maintenance notes

- Any future query builder that interpolates into an identifier must use
  `escape_ident` — a reviewer should grep for new `FROM "{}"`/`"{}"` format
  sites in storage diffs.
- Plan 029/030 (span links, service map) add new storage queries; they must
  follow this pattern.
- Deferred root fix (named per repo rule): the structural enabling condition
  is string-composed SQL. A parameterized-query client or a query-builder
  layer for GreptimeDB's HTTP API would remove the bug class; that is a
  larger change tracked as a "considered" item in `advisor-plans/README.md`, not
  done here.
- The `sql()` guard remains lexical; engine-level read-only enforcement is
  not available in-process. If GreptimeDB grows a read-only session flag,
  adopt it.
