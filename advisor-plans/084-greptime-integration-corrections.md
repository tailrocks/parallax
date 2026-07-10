# Plan 084: Correct the GreptimeDB integration against engine-verified facts — indexed log search, right index types, deterministic log schema, TTL reconcile, query timeouts, version-pin upgrades

> **Executor instructions**: Follow this plan step by step. Run every
> verification command and confirm the expected result before moving to the
> next step. If anything in the "STOP conditions" section occurs, stop and
> report — do not improvise. When done, update the status row for this plan
> in `advisor-plans/README.md` — unless a reviewer dispatched you and told you
> they maintain the index.
>
> **Drift check (run first)**: `git diff --stat df81d86..HEAD -- crates/parallax-storage/src/greptime.rs crates/parallax-server/src/greptime_supervisor.rs crates/parallax-server/src/serve.rs`
> Note: at planning time the working tree already carried uncommitted edits to
> `greptime.rs` (~74 lines) — the excerpts below were taken from that working
> tree, not from `df81d86`. Compare excerpts against live code; mismatch = STOP.

## Status

- **Priority**: P1
- **Effort**: L
- **Risk**: MED (search semantics change in Step 1; ingest-schema change in Step 4)
- **Depends on**: 070 (rebase hygiene), 074 soft (golden-SQL tests make the SQL diffs reviewable; if 074 has not run, update the inline tests this plan adds instead)
- **Category**: perf
- **Planned at**: commit `df81d86`, 2026-07-10

## Why this matters

Parallax's whole product sits on GreptimeDB, and a 2026-07-10 deep audit
verified the integration line-by-line against GreptimeDB v1.1.x docs and the
v1.1.2 source tag — including live `EXPLAIN ANALYZE` runs against the embedded
engine. Six places are measurably wrong or contradict the engine team's own
guidance: log body search bypasses the FULLTEXT index it creates (measured 4.5×
slower, 14× more scan memory at 500k rows), the `trace_id` index type is the one
GreptimeDB's docs say NOT to use for high-cardinality columns, one deviation is
dead code on engine ≥1.1, the log-table schema is nondeterministic (a race can
permanently promote a per-row-unique timestamp into the PRIMARY KEY), retention
config changes silently never apply, and no timeout exists anywhere on the
query path. Each fix below cites the verified source.

## Current state

All paths relative to the repo root.

- `crates/parallax-storage/src/greptime.rs` — the GreptimeDB adapter. All reads
  are SQL over `POST {base_url}/v1/sql?db=public`; ingest forwards raw OTLP to
  `{base_url}/v1/otlp/v1/...`.
- `crates/parallax-server/src/greptime_supervisor.rs` — resolves/downloads the
  engine binary, spawns `greptime standalone start`.
- `crates/parallax-server/src/serve.rs` — calls `GreptimeStore::connect` +
  `bootstrap` (lines 50-67).

### Fact base (engine-verified 2026-07-10; re-verify in Step 0)

1. **FULLTEXT index is used only by `matches_term(col, 't')` / `matches(col, q)`
   — never by `LIKE`.** Upstream applier has no `Expr::Like` arm
   (github.com/GreptimeTeam/greptimedb, `src/mito2/src/sst/index/fulltext_index/applier/builder.rs`;
   docs: docs.greptime.com/user-guide/logs/fulltext-search). Live measurement on
   the embedded engine (1.1.0, 500,709-row `opentelemetry_logs`):
   `body LIKE '%error%'` → 500,709 rows scanned, 1.3 GiB, 114 ms;
   `matches_term(body,'error')` → 196,819 rows, 91.7 MiB, 25 ms.
2. **`opentelemetry_logs.body` is created with a FULLTEXT (bloom) index by
   default** on engine ≥1.1 (v1.1.2 `src/servers/src/otlp/logs.rs:237-247`;
   docs: "The body column is created with a fulltext index by default").
3. **Inverted index is the wrong type for `trace_id`** — docs recommend a
   SKIPPING (bloom) index for high-cardinality point-lookup columns
   (docs.greptime.com/user-guide/manage-data/data-index), and GreptimeDB's own
   trace pipeline gives `trace_id` a SKIPPING index.
4. **`x-greptime-log-extract-keys` promotes a not-yet-existing column to
   `SemanticType::Tag` — i.e. into the PRIMARY KEY** (v1.1.2
   `src/servers/src/otlp/logs.rs` `extract_field_from_attr_and_combine_schema`).
   If the column already exists, its existing semantic type is kept.
5. **`x-greptime-hints` (ttl/append_mode) apply ONLY at table auto-creation**;
   on existing tables they are silently ignored (v1.1.2
   `src/operator/src/insert.rs`, consumed only in
   `get_create_table_expr_on_demand`; docs: docs.greptime.com/user-guide/protocols/http).
   `ALTER TABLE t SET 'ttl'='…'` is the documented way to change TTL later.
6. **`X-Greptime-Timeout: <duration>` request header is supported; the server
   default HTTP timeout is `0s` = disabled**
   (docs.greptime.com/user-guide/protocols/http;
   docs.greptime.com/user-guide/deployments-administration/configuration).
7. **`greptime standalone start` has no `--memory-limit` flag.** Tuning goes
   through `-c <config.toml>` or `GREPTIMEDB_STANDALONE__*` env vars.
8. **ALTER-added indexes are built on flush/compaction** — existing SSTs stay
   unindexed until rewritten; `ADMIN build_index_table(...)` exists at 1.1.2
   for explicit backfill (source-derived, MED confidence — verify live).

### Code excerpts (working tree at planning time)

`greptime.rs:790-801` — body search is LIKE (the comment even names the fix):

```rust
    if let Some(needle) = body_contains {
        // LIKE wildcards in the needle are literal for a substring search;
        // backslash first (it is the escape char), then %, _, then quotes.
        let escaped = escape(
            &needle
                .replace('\\', "\\\\")
                .replace('%', "\\%")
                .replace('_', "\\_"),
        );
        // ESCAPE takes exactly one character — a single backslash in SQL.
        clauses.push(format!(r#""body" LIKE '%{escaped}%' ESCAPE '\'"#));
    }
```

(and `greptime.rs:761-763`: "Body search is `LIKE` today; a GreptimeDB FULLTEXT
index + `matches_term` is the planned upgrade for large logs (spec §5 note).")

`greptime.rs:330-349` — the logs deviations:

```rust
    async fn try_logs_deviations(&self) {
        self.try_deviations([
            r#"ALTER TABLE opentelemetry_logs MODIFY COLUMN "trace_id" SET INVERTED INDEX"#
                .to_string(),
            r#"ALTER TABLE opentelemetry_logs MODIFY COLUMN "body" SET FULLTEXT INDEX"#.to_string(),
            format!(
                "ALTER TABLE opentelemetry_logs ADD COLUMN {} STRING",
                wire_attr_ident(semconv::PARALLAX_RUN_ID)
            ),
            ...
```

`greptime.rs:908-928` — `ingest_logs` sends
`x-greptime-log-extract-keys: parallax.run.id,event.name,<LOG_OBSERVED_TS_NANOS>`
on every forward, and only *afterwards* runs `ensure_logs_deviations()`. So on a
fresh data dir, the FIRST logs batch that carries those attributes creates the
columns as **tags** (fact 4) before the ADD COLUMN deviations can create them as
fields. `observed_ts_nanos` as a tag = a per-row-unique Int64 in the PRIMARY
KEY — the exact anti-pattern GreptimeDB's schema guide forbids.

`greptime.rs:764-783` — the log service filter is a per-row JSON extraction
(non-indexable):

```rust
    if let Some(service) = service {
        // Native logs carry no `service_name` column; match on resource JSON.
        clauses.push(format!(
            r#"{} = '{}'"#,
            resource_json_get(semconv::SERVICE_NAME),
            escape(service)
        ));
    }
```

Same `resource_json_get(semconv::SERVICE_NAME)` pattern appears in
`service_names` (`:1259`), `overview_totals` (`:1306`), `observed_runs`
(`:1893`), and as a projection in `select_logs` (`:599`).

`greptime.rs:220-244` — `connect()` builds `reqwest::Client::new()` (no
timeout); `sql()`/`sql_with_schema()` (`:408-493`) send no
`X-Greptime-Timeout` header.

`greptime_supervisor.rs:121-135` — `ensure_binary` returns an existing managed
binary without any version check, so bumping the pin in
`config.rs:92` (`greptime_version: "1.1.2"`) never upgrades an existing
install (confirmed live: repo pins 1.1.2, a long-lived dev instance still ran
1.1.0):

```rust
    let managed = bin_dir.join("greptime");
    if managed.exists() {
        return Ok(managed);
    }
```

`greptime_supervisor.rs:267-282` — spawn passes only addresses + `--data-home`;
no `-c` config file, no env vars: every engine knob is default.

`serve.rs:50-67` — `connect_greptime` passes the retention TTLs into the store;
`bootstrap` creates extension tables with `WITH (ttl = '…')` (`greptime.rs:250-279`)
— `CREATE TABLE IF NOT EXISTS` no-ops on existing tables, and per fact 5 the
hints no-op too, so **editing `[retention]` in config.toml changes nothing on
an existing data dir**.

Repo conventions: strict clippy (zero warnings, `unwrap_used = "warn"`), cargo-nextest,
`rtk` prefix on shell commands, Conventional Commits with DCO (`git commit -s`) and
trailer `Co-authored-by: Claude <noreply@anthropic.com>`, work directly on `main`.

## Commands you will need

| Purpose | Command | Expected on success |
|---------|---------|---------------------|
| Build | `rtk cargo build --workspace` | exit 0 |
| Storage tests | `rtk cargo nextest run -p parallax-storage` | all pass |
| Full suite | `rtk cargo nextest run --workspace` | all pass |
| Lint | `rtk cargo clippy --workspace --all-targets` | zero warnings |
| Format | `rtk cargo fmt --all` | exit 0 |
| Real-engine conformance (gated) | `rtk cargo nextest run -p parallax-server m6_conformance --run-ignored only` | all pass |
| Live engine SQL (needs `parallax serve` running) | `curl -s -XPOST 'http://127.0.0.1:24000/v1/sql?db=public' -d "sql=<QUERY>"` | JSON with `output` |

## Scope

**In scope** (the only files you should modify):
- `crates/parallax-storage/src/greptime.rs`
- `crates/parallax-server/src/greptime_supervisor.rs`
- `crates/parallax-server/src/serve.rs` (TTL reconcile call only)
- `crates/parallax-server/src/config.rs` (only if adding a query-timeout config key)
- `docs/research/decisions/native-otel-tables.md` (record the Step 4 schema decision)
- `advisor-plans/README.md` (status row)

**Out of scope** (do NOT touch, even though they look related):
- `crates/parallax-storage/src/memory.rs` — the memory adapter's `body_contains`
  stays substring; Step 1 documents the divergence instead (see STOP conditions).
- Query-shape rewrites (windowing, aggregation pushdown) — Plan 085.
- `parallax-api` resolvers — Plan 086.
- The OTLP receivers / worker — Plan 087.
- Engine tuning VALUES (write the config channel, do not pick tuning numbers —
  the repo's benchmark rule requires measurement first).

## Git workflow

- Work directly on `main` (BRANCHING.md). Conventional Commits, `git commit -s`,
  trailer `Co-authored-by: Claude <noreply@anthropic.com>`.
- One commit per step is ideal; e.g. `perf(storage): use matches_term for log body search`.

## Steps

### Step 0: Re-verify the fact base against the live embedded engine

Start `parallax serve` (or use a running instance). Confirm:

1. `SELECT version()` via the curl command above → note the version. If < 1.1,
   STOP (fact 2 does not hold).
2. `SHOW CREATE TABLE opentelemetry_logs` → confirm `body` already carries a
   FULLTEXT/bloom index, and note the current semantic types of
   `"parallax.run.id"` / `"observed_ts_nanos"` columns if they exist (TAG vs
   FIELD — visible in the PRIMARY KEY list).
3. `EXPLAIN ANALYZE SELECT count(*) FROM opentelemetry_logs WHERE "body" LIKE '%error%'`
   vs `... WHERE matches_term("body", 'error')` → record rows-scanned for both
   in the commit message. (Requires ingested logs; the sibling
   parallax-telemetry-playground repo generates traffic.)

**Verify**: the numbers reproduce the direction of the audit measurement
(matches_term scans strictly fewer rows). If `matches_term` errors, STOP.

### Step 1: Route log body search through `matches_term`, keep LIKE as explicit escape hatch

In `log_filter_clauses` (`greptime.rs:764-803`):

- Tokenize the needle on whitespace; for each token emit
  `matches_term("body", '<escaped token>')`, AND-combined. Use the existing
  `escape()` for the SQL string literal (single quotes only; `matches_term`
  takes a plain term, no LIKE-wildcard escaping).
- Preserve exact-substring intent behind quoting: if the needle is wrapped in
  double quotes (`"connection reset"`), strip the quotes and emit ONE
  `matches_term("body", 'connection reset')` (phrase term). If the needle
  contains no alphanumeric characters at all (pure punctuation), fall back to
  the current LIKE clause (matches_term terms are word-ish; punctuation-only
  searches would match nothing).
- Document the semantics in a doc comment on `log_filter_clauses`: term match,
  case-insensitive (bloom fulltext default), NOT substring — `error` no longer
  matches inside `errors`.

`log_filter_clauses` drives `logs_search`, `log_count_series`, and
`signal_count_series(Logs)` — one change covers all three; the histogram stays
consistent with the table because they share this function.

Add/extend an inline `#[cfg(test)]` test asserting the generated WHERE clause
for: single term, two terms, quoted phrase, punctuation-only fallback.

**Verify**: `rtk cargo nextest run -p parallax-storage` → all pass, including
the new clause tests. Then live (gated): a logs search for a term present in
seeded logs returns rows, and `EXPLAIN ANALYZE` of the generated WHERE shows
pruned rows vs the Step 0 LIKE baseline.

### Step 2: Fix the logs deviations — SKIPPING index on `trace_id`, delete the dead FULLTEXT ALTER

In `try_logs_deviations` (`greptime.rs:330-349`):

- Replace the `trace_id` line with:
  `ALTER TABLE opentelemetry_logs MODIFY COLUMN "trace_id" SET SKIPPING INDEX`
  (bloom is the default skipping backend). Update the function doc comment:
  the read shape is point lookup (`"trace_id" = '…'` in `logs_by_trace`),
  which a bloom skipping index serves at a fraction of the inverted index's
  build/memory cost per GreptimeDB's data-index guidance.
- Delete the `body SET FULLTEXT INDEX` line and the "(the one native
  shortfall)" comment — the native table has it by default on ≥1.1 (fact 2).
- Note in a comment that an ALTERed index only covers SSTs flushed after the
  ALTER (fact 8); pre-existing installs that care can run
  `ADMIN build_index_table('opentelemetry_logs')` manually.

Existing installs: the previous INVERTED index remains on already-deployed data
dirs; MODIFY COLUMN to a different index type replaces the index setting for
future SSTs. That is acceptable — do not attempt data rewrites.

**Verify**: `rtk cargo build --workspace` → exit 0. Gated live check: restart
serve against a fresh data dir, ingest one logs batch, then
`SHOW CREATE TABLE opentelemetry_logs` shows `SKIPPING INDEX` on `trace_id`
and no duplicate/errored fulltext option on `body`.

### Step 3: Make the log-table schema deterministic — pre-create `opentelemetry_logs` before the first forward

Goal: kill the race in fact 4 AND give logs an indexable service column.

In `GreptimeStore::bootstrap` (`greptime.rs:250-290`), before the deviation
calls, execute a `CREATE TABLE IF NOT EXISTS opentelemetry_logs (...)` that
matches the native OTLP logs schema the engine would auto-create (verify
against `SHOW CREATE TABLE` from a scratch auto-created instance in Step 0 —
column names/types must match EXACTLY or the first forward will fail), with
these deliberate deviations, all as **FIELDs (not tags)** unless noted:

- `"parallax.run.id"` STRING FIELD with SKIPPING INDEX,
- `<semconv::EVENT_NAME>` STRING FIELD,
- `<semconv::LOG_OBSERVED_TS_NANOS>` BIGINT FIELD,
- `"service_name"` STRING **TAG** (low-cardinality; mirrors the native traces
  table where `service_name` is a tag) — populated at ingest via extract-keys.

Then:

- Add `service.name` to the `extract_keys` list in `ingest_logs`
  (`greptime.rs:912-917`). Because the columns now pre-exist with fixed
  semantic types, extract-keys will populate them WITHOUT changing types
  (fact 4: existing semantic type is reused).
- Keep `try_logs_deviations` as a repair path for pre-existing data dirs
  (ADD COLUMN lines stay; they are idempotent).
- Switch the FIVE `resource_json_get(semconv::SERVICE_NAME)` sites
  (`log_filter_clauses:777`, `service_names:1259`, `overview_totals:1306`,
  `observed_runs:1893`, `select_logs:599` projection) to
  `COALESCE("service_name", json_get_string("resource_attributes", '…'))` —
  the COALESCE keeps rows ingested before this change readable. Wrap that
  expression in one helper fn so the fallback lives in one place.
- Record the decision + verification evidence in
  `docs/research/decisions/native-otel-tables.md` (the repo's native-table rule
  requires justifying deviations there; this is an extension via native
  mechanisms — pre-created native schema + extract-keys — not a custom table).

**Verify**: `rtk cargo nextest run -p parallax-storage` → pass. Gated live, on
a FRESH data dir: start serve, ingest logs that carry `observed_ts_nanos`,
then `SHOW CREATE TABLE opentelemetry_logs` → `observed_ts_nanos` is NOT in
the PRIMARY KEY, `service_name` IS a tag, and a `logs_search` filtered by
service returns rows.

### Step 4: Reconcile TTLs on every bootstrap

Retention edits currently never propagate (fact 5). In `bootstrap` (and in the
lazy `ensure_*_deviations` for the native tables, since they may not exist at
bootstrap):

- After table existence is known, read the table's current TTL from
  `information_schema` (`SELECT create_options FROM information_schema.tables WHERE table_name='…'`
  — verify the exact column live in Step 0; if TTL is not exposed there, just
  issue the ALTER unconditionally — it is idempotent and cheap) and issue
  `ALTER TABLE <t> SET 'ttl' = '<configured>'` when it differs (or always).
- Cover: `opentelemetry_traces`, `opentelemetry_logs`, `error_events`,
  `run_metric_points`, `metric_exemplars`. Per-metric native tables are
  excluded (they are created continuously; their TTL keeps riding the hint —
  document this gap in the maintenance notes).

**Verify**: gated live: edit a TTL in config, restart serve,
`SHOW CREATE TABLE error_events` shows the new ttl.

### Step 5: Timeouts on the query path

- In `connect` (`greptime.rs:220-244`): build the client with
  `reqwest::Client::builder().timeout(Duration::from_secs(70)).build()?`.
- In `sql`/`sql_with_schema`/`forward_otlp`: send header
  `X-Greptime-Timeout: 60s` on SQL reads (NOT on `forward_otlp` — ingest
  forwards should not be killed mid-write by a read deadline; give forwards
  the client timeout only).
- If a config knob feels warranted, add `limits.query_timeout_secs` (default
  60) in `config.rs` and thread it through `connect_greptime` in `serve.rs`;
  otherwise constants with a comment are acceptable — choose the config knob
  only if the threading stays under ~20 lines.

**Verify**: `rtk cargo nextest run --workspace` → all pass (the in-memory
tests don't touch reqwest; the gated conformance suite exercises the real
client — run it).

### Step 6: Version-pin upgrades in the supervisor

In `ensure_binary` (`greptime_supervisor.rs:121-135`): when the managed binary
exists, run `<managed> --version` (same pattern as the PATH probe at `:130-135`),
parse the version from its output, and if it does not match the resolved pin
(and the pin is not `"latest"`), re-download: rename the old binary to
`greptime-<oldversion>` (keep as rollback), then proceed with the existing
download path. Log the upgrade at info level (progress-visibility rule: the
user must see "upgrading GreptimeDB vX → vY").

**Verify**: `rtk cargo build --workspace` → exit 0. Manual: place any
executable named `greptime` that prints a different version in the bin dir;
`ensure_binary` with a pinned version triggers the download path (or test the
version-compare logic as a pure function with a unit test — preferred).

### Step 7: Open the engine-config channel (no tuning values)

In `GreptimeSupervisor::spawn` (`greptime_supervisor.rs:262-289`): if
`<data_dir>/greptime-config.toml` exists, pass `-c <that path>`. Do NOT write
tuning values yourself; create nothing by default. Document the file (one
paragraph) in `docs/guides/` if a config guide exists, else in the supervisor
module doc comment: which knobs the perf-tuning docs name
(`region_engine.mito.global_write_buffer_size`, `page_cache_size`,
`query.memory_pool_size`, `wal.*`, TWCS options), with the note that changes
require a four-build benchmark per the repo's benchmarking rule.

**Verify**: `rtk cargo build --workspace` → exit 0; starting serve WITHOUT the
file behaves exactly as before (no `-c` flag passed).

### Step 8: Full gates

**Verify**: `rtk cargo fmt --all` → clean; `rtk cargo clippy --workspace
--all-targets` → zero warnings; `rtk cargo nextest run --workspace` → all
pass; gated conformance suite against the real engine → all pass.

## Test plan

- New inline tests in `greptime.rs`: body-search clause generation (4 cases,
  Step 1); version-compare helper (Step 6).
- Existing storage suite must pass unchanged EXCEPT tests that pin the LIKE
  clause (update those deliberately — the diff is the review artifact).
- Gated real-engine checks as written per step (fresh-data-dir schema check is
  the critical one for Step 3).

## Done criteria

Machine-checkable. ALL must hold:

- [ ] `grep -n "LIKE '%" crates/parallax-storage/src/greptime.rs` → only the
      documented punctuation-fallback site and `span_name` search remain (body
      search default path uses `matches_term`)
- [ ] `grep -n "SET INVERTED INDEX" crates/parallax-storage/src/greptime.rs` → 0 matches
- [ ] `grep -n "body\" SET FULLTEXT" crates/parallax-storage/src/greptime.rs` → 0 matches
- [ ] `grep -cn "resource_json_get(semconv::SERVICE_NAME)" crates/parallax-storage/src/greptime.rs` → sites replaced by the COALESCE helper (helper itself may still call it once)
- [ ] `grep -n "X-Greptime-Timeout\|x-greptime-timeout" crates/parallax-storage/src/greptime.rs` → ≥1 match
- [ ] `grep -n "timeout" crates/parallax-storage/src/greptime.rs` shows a reqwest client timeout in `connect`
- [ ] `rtk cargo nextest run --workspace` exits 0; clippy zero warnings
- [ ] `git status` shows no modified files outside the in-scope list
- [ ] `docs/research/decisions/native-otel-tables.md` records the Step 3 schema decision
- [ ] `advisor-plans/README.md` status row updated

## STOP conditions

Stop and report back (do not improvise) if:

- Step 0's live engine is < 1.1 or `matches_term` is rejected.
- Step 3: the auto-created `opentelemetry_logs` schema (from `SHOW CREATE TABLE`
  on a scratch instance) contains columns/options you cannot reproduce in a
  `CREATE TABLE` statement — pre-creating a half-matching schema would break
  the first forward. Report the exact schema instead.
- Step 3: the first forward into the pre-created table errors (schema
  mismatch) — do not iterate blindly; capture the error + schema and report.
  Per the repo's native-table rule, unresolved conflicts here go to a research
  note + the GreptimeDB team, not workarounds.
- The memory adapter's `body_contains` semantics (substring) would need to
  change to keep any EXISTING cross-adapter test green — the term-vs-substring
  divergence between adapters must be an explicit recorded decision, not a
  silent test edit. Report which test.
- Step 4: `ALTER TABLE ... SET 'ttl'` is rejected by the shipped engine.

## Maintenance notes

- Search semantics changed user-visibly (term match vs substring): release
  notes / commit message must say so; the memory adapter still does substring —
  the conformance suite (Plan 074) should pin the divergence or align them
  later.
- Per-metric native tables still get TTL only via creation hints — a config
  TTL change does not retro-apply to them (recorded gap; revisit with a
  metrics retention pass).
- The Step 3 pre-created schema must be revisited whenever the pinned engine
  version changes the native OTLP logs schema (watch release notes; the
  Step 0 `SHOW CREATE TABLE` comparison is the drift check).
- `span_name LIKE '%…%'` in `traces_search` remains a full-scan (no fulltext
  index on span names) — deliberate; add an index only with measurement.
- Follow-ups deliberately deferred: engine tuning values (needs four-build
  benchmark), `trace_table_partitions` hint for the 16-partition default
  (measure first — see Plan 090), `ADMIN build_index_table` backfill for old
  installs.
