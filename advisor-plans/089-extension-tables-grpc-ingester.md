# Plan 089: Move extension-table writes off text SQL — gRPC ingester for error_events / run_metric_points / metric_exemplars, and fix the exemplar table's high-cardinality primary key

> **Executor instructions**: Follow this plan step by step. Run every
> verification command and confirm the expected result before moving to the
> next step. If anything in the "STOP conditions" section occurs, stop and
> report — do not improvise. When done, update the status row for this plan
> in `advisor-plans/README.md` — unless a reviewer dispatched you and told you
> they maintain the index.
>
> **Drift check (run first)**: `git diff --stat df81d86..HEAD -- crates/parallax-storage/src/greptime.rs crates/parallax-server/src/greptime_supervisor.rs Cargo.toml`
> Excerpts from the working tree at planning time. Mismatch = STOP.

## Status

- **Priority**: P2
- **Effort**: M
- **Risk**: MED (new gRPC dependency + a schema migration for metric_exemplars)
- **Depends on**: 084 (integration corrections land first — same file regions), 070
- **Category**: perf
- **Planned at**: commit `df81d86`, 2026-07-10
- **Execution**: **BLOCKED** at Step 0 (2026-07-11) — see below. SQL path stays.

### Step 0 failure (do not improvise)

Resolved latest stable **`greptimedb-ingester` 0.18.0** (also checked 0.17.0, 0.16.0 —
only published versions on crates.io). Constraints:

| Check | Result |
|-------|--------|
| (a) tonic coexists with workspace `tonic 0.14` | **PASS** — crate pins `tonic = "0.14"` |
| (b) no rustls / TLS features off for plaintext | **FAIL** — hard dep, not feature-gated |
| (c) TIMESTAMP(9) ns + JSON row values | **PARTIAL** — ns OK; no JSON helper (proto has `JsonValue`; plan allows STRING encode) |

**Blocking detail (b)**: every published `greptimedb-ingester` version declares:

```toml
[dependencies.tonic]
version = "0.14"
features = ["tls-ring", "gzip", "zstd"]
```

`tls-ring` pulls `rustls` / `tokio-rustls` / `ring` / `rustls-webpki` into
`parallax-storage` with **no consumer feature flag** to disable TLS. Cargo
cannot strip a dependency's required feature set. Repo TLS policy is
**native-tls always, never rustls**; plan STOP requires halt on unavoidable
rustls (SQL path remains).

Verified: `cargo tree -i rustls` with only `greptimedb-ingester = "0.18.0"`:

```text
rustls → tokio-rustls → tonic → greptimedb-ingester
```

**Upstream ask (fix-forward)**: feature-gate TLS on
`greptimedb-ingester` (default off for plaintext localhost; optional
`tls-native-roots` / native-TLS path instead of hard-coded `tls-ring`). Until
then this plan cannot land without violating TLS policy or forking the crate
(out of scope / STOP: do not improvise).

**Not reached**: Steps 1–4 (gRPC plumbing, row writers, exemplar PK migration,
parity). Drift note: `greptime.rs` / supervisor / root `Cargo.toml` advanced
past `df81d86` via 084/070 (dependencies) — write path still SQL `INSERT` and
`metric_exemplars` still has high-card PK; work remains valid once Step 0
clears.

## Why this matters

Parallax writes its own derived rows (error events, run-scoped metric points,
metric exemplars) by building `INSERT INTO … VALUES (…),(…)` SQL strings and
POSTing them to `/v1/sql`. GreptimeDB's official guidance ranks this as the
SLOWEST ingest path: the team's FAQ puts SQL INSERT at 1× baseline, the gRPC
SDK row API at ~16×, gRPC bulk at ~37×
(docs.greptime.com/faq-and-others/faq/), and ships a Rust ingester
(github.com/GreptimeTeam/greptimedb-ingester-rust) for exactly this shape.
These writes sit on the ingest worker's critical path behind every error burst
and metric batch. Separately, `metric_exemplars` declares
`PRIMARY KEY ("service", "name", "trace_id", "span_id")` — per-row-unique ids
as tags — the schema anti-pattern GreptimeDB's design guide explicitly warns
against (high-cardinality tags degrade the metric engine's series bookkeeping).

## Current state

- `crates/parallax-storage/src/greptime.rs:524-534` — the text-SQL insert:

```rust
    async fn insert(&self, table: &str, columns: &str, values: Vec<String>) -> anyhow::Result<()> {
        if values.is_empty() { return Ok(()); }
        let sql = format!("INSERT INTO {table} ({columns}) VALUES {}", values.join(","));
        self.sql(&sql).await?;
        Ok(())
    }
```

  Callers: `ingest_metrics` (`:931-990`, run_metric_points + metric_exemplars,
  per OTLP metrics batch), `write_error_events` (`:992-1018`, per error batch,
  includes full stacktrace strings).
- `crates/parallax-storage/src/greptime.rs:263-278` — the DDL (bootstrap):

```rust
                r#"CREATE TABLE IF NOT EXISTS metric_exemplars (
                   "ts" TIMESTAMP(9) NOT NULL,
                   "service" STRING, "name" STRING, "value" DOUBLE,
                   "trace_id" STRING, "span_id" STRING, "run_id" STRING SKIPPING INDEX,
                   "attributes" JSON,
                   TIME INDEX ("ts"), PRIMARY KEY ("service", "name", "trace_id", "span_id")
                 ) WITH (append_mode = 'true', ttl = '{metrics_ttl}')"#
```

  (`run_metric_points` at `:263-269` is CORRECT per the same guidance —
  PK ("service","name"), `run_id` as SKIPPING-indexed field; use it as the
  model.)
- The supervised engine exposes gRPC on `127.0.0.1:24001`
  (`greptime_supervisor.rs:13 GREPTIME_GRPC_PORT`), currently unused by
  Parallax. External mode (`config.rs storage.greptime_url`) knows only an
  HTTP URL — a gRPC endpoint for external mode needs a config addition.
- Engine facts (verified 2026-07-10): HTTP SQL body limit defaults to 64 MB;
  the ingester supports gRPC compression and row + bulk APIs; TLS policy is
  unaffected on the local hop (plaintext localhost; if the ingester crate
  pulls rustls via default features, DISABLE those features — repo TLS rule:
  never enable rustls; plaintext channel needs no TLS backend).
- Read paths for exemplars: `metric_exemplars` query at `greptime.rs:1755-1792`
  (SELECT with name/ts/service filters, `ORDER BY ts DESC LIMIT`), plus the
  memory adapter equivalent. No query GROUPS BY trace_id/span_id — the PK
  carries no read benefit.
- Version policy: latest stable of any new crate; add via
  `[workspace.dependencies]` in the root `Cargo.toml`.

Conventions: strict clippy, cargo-nextest, `rtk` prefix, Conventional Commits
+ DCO + `Co-authored-by: Claude <noreply@anthropic.com>`, direct on `main`.

## Commands you will need

| Purpose | Command | Expected on success |
|---------|---------|---------------------|
| Build | `rtk cargo build --workspace` | exit 0 |
| Storage tests | `rtk cargo nextest run -p parallax-storage` | all pass |
| Full suite | `rtk cargo nextest run --workspace` | all pass |
| Lint | `rtk cargo clippy --workspace --all-targets` | zero warnings |
| Dep tree check | `rtk cargo tree -p parallax-storage -i rustls 2>&1` | "nothing depends on rustls" / error = good |

## Scope

**In scope**:
- Root `Cargo.toml` + `crates/parallax-storage/Cargo.toml` (ingester dep)
- `crates/parallax-storage/src/greptime.rs` (writes + exemplar DDL)
- `crates/parallax-server/src/serve.rs` + `config.rs` (gRPC URL plumbing)
- `advisor-plans/README.md`

**Out of scope**:
- The OTLP raw forwards (correct as-is; OTLP-over-gRPC was removed upstream).
- Read queries (085/086 own those).
- `memory.rs` (no wire format there).
- Bulk/streaming API adoption — row API only in this plan (bulk requires
  pre-created tables + more machinery; note as follow-up).

## Git workflow

Direct on `main`; Conventional Commits + `git commit -s` + Claude trailer.

## Steps

### Step 0: Verify the ingester crate fits the repo's constraints

Resolve the latest stable `greptimedb-ingester` crate. Check: (a) its tonic
version coexists with the workspace's `tonic 0.14` (cargo tree — duplicate
tonic majors are a STOP), (b) default features do not enable rustls (disable
TLS features entirely — plaintext local channel), (c) the row API supports
TIMESTAMP(9) nanosecond columns and JSON columns (metric_exemplars.attributes
is JSON; if the row API lacks a JSON value type, encode the attributes as a
STRING column write and verify the column type tolerates it — if not, STOP).

**Verify**: `rtk cargo build --workspace` with the dep added → exit 0;
`rtk cargo tree -p parallax-storage | rtk grep -i rustls` → no rustls.

### Step 1: gRPC channel plumbing

- `config.rs`: add `storage.greptime_grpc_url` (default empty = derive:
  managed mode uses `127.0.0.1:24001`; external mode REQUIRES it explicitly —
  error otherwise with a clear message).
- `GreptimeStore::connect` gains the gRPC endpoint parameter and lazily builds
  the ingester client (connect on first write; the engine may still be
  starting).

**Verify**: `rtk cargo nextest run --workspace` → pass (memory-mode tests
untouched); managed-mode serve boots (manual or gated test).

### Step 2: Route the three writers through the row API

Replace the bodies of `write_error_events` and the two `insert` calls in
`ingest_metrics` with ingester row writes (table name, column values, ns
timestamps). Keep `insert()` (the SQL fallback) and add an env-var escape
hatch `PARALLAX_EXT_WRITES=sql` that routes back to SQL — one match at the
three call sites; document it in the fn doc. Delete nothing until Step 4's
verification passes.

Chunk rows defensively (e.g. 5,000 rows per gRPC call) — bursts stay bounded.

**Verify**: gated real-engine test: run the conformance/e2e ingest scenario,
then `SELECT COUNT(*) FROM error_events` / `run_metric_points` /
`metric_exemplars` via raw SQL — counts match the scenario's expectations
(the existing gated suite covers error_events; extend for the other two if
not covered).

### Step 3: Fix the `metric_exemplars` primary key

New DDL (bootstrap):

```sql
CREATE TABLE IF NOT EXISTS metric_exemplars (
  "ts" TIMESTAMP(9) NOT NULL,
  "service" STRING, "name" STRING, "value" DOUBLE,
  "trace_id" STRING SKIPPING INDEX, "span_id" STRING,
  "run_id" STRING SKIPPING INDEX, "attributes" JSON,
  TIME INDEX ("ts"), PRIMARY KEY ("service", "name")
) WITH (append_mode = 'true', ttl = '{metrics_ttl}')
```

Migration for existing data dirs: `CREATE TABLE IF NOT EXISTS` will not touch
the old table. On bootstrap, detect the old shape via
`information_schema.columns`/`SHOW CREATE TABLE` (PK containing trace_id);
if found: `ALTER` cannot demote tags → create `metric_exemplars_v2` with the
new shape, `INSERT INTO metric_exemplars_v2 SELECT ... FROM metric_exemplars`,
drop old, rename is NOT supported the same way everywhere — verify `ALTER
TABLE ... RENAME` exists on the shipped engine; if not, keep the `_v2` name
and point reads/writes at a `const EXEMPLARS_TABLE` — smallest verified path
wins; record which was taken.

**Verify**: fresh data dir → `SHOW CREATE TABLE metric_exemplars` shows
PK ("service","name") and skipping indexes; existing-dir migration test
(gated, manual acceptable): old-shape table with 2 rows migrates, reads
return both rows.

### Step 4: Parity + removal decision

Run the full gated suite with gRPC writes on, then with
`PARALLAX_EXT_WRITES=sql` — identical results. Record in the commit message a
rough local timing of a burst write both ways (informational only; the bench
rule's four-way matrix is not required for a code-path choice, but numbers go
in the commit message, not docs). Keep the SQL fallback for one release cycle
(escape hatch documented) — note removal as follow-up.

**Verify**: both modes green on the gated suite;
`rtk cargo nextest run --workspace` all pass; clippy zero warnings.

## Test plan

- Existing storage/worker tests unchanged (they run on the memory adapter).
- Gated real-engine: counts parity for all three tables in both write modes;
  exemplar migration check.
- Unit: chunking function (row batches split at the cap).

## Done criteria

- [ ] `grep -n "INSERT INTO" crates/parallax-storage/src/greptime.rs` → only inside the documented SQL fallback path
- [ ] `grep -n "PRIMARY KEY (\"service\", \"name\", \"trace_id\"" crates/parallax-storage/src/greptime.rs` → 0 matches
- [ ] `rtk cargo tree -p parallax-storage -i rustls` → not a dependency
- [ ] gated suite green in both write modes
- [ ] `rtk cargo nextest run --workspace` exits 0; clippy zero warnings
- [ ] `git status` clean outside in-scope list
- [ ] `advisor-plans/README.md` status row updated

## STOP conditions

Stop and report back (do not improvise) if:

- Step 0 fails any check (tonic conflict, unavoidable rustls, no JSON/ns
  support) — the SQL path stays; report which constraint failed so the
  operator can raise it upstream (fix-forward rule).
- The migration in Step 3 cannot preserve existing exemplar rows on the
  shipped engine (missing RENAME and `_v2` read-path change exceeds ~10 call
  sites).
- Row-API writes succeed but counts diverge from SQL-mode counts.

## Maintenance notes

- The ingester opens a SECOND channel to the engine; the supervisor's
  health/restart logic only watches HTTP — a gRPC-only outage would surface as
  write errors, not a restart. Acceptable (same process dies together); note
  for future supervisor work.
- Bulk/streaming API (~37×) is the next step if row-API throughput ever
  limits — tables are pre-created, so it is available.
- The SQL fallback + env var should be removed after one release cycle.
- `error_events` PK ("service","fingerprint") is fine (low-cardinality); only
  exemplars needed the fix.
