# Plan 048: Wire Postgres for real — inventory on sqlx, db.* spans, pool metrics, slow-query/N+1/pool-exhaustion scenarios

> **Executor instructions**: Targets the **playground repository**
> (`parallax-telemetry-playground`). Follow step by step; run every
> verification. On any STOP condition, stop and report. When done, update the
> status row in the Parallax repo's `plans/README.md`.
>
> **Drift check (run first)**: in the playground repo,
> `git diff --stat ed1f975..HEAD -- services/inventory deploy libs scenarios`
> On mismatch with the excerpts below, STOP.

## Status

- **Priority**: P2
- **Effort**: L
- **Risk**: MED (new stateful dependency in the request path)
- **Depends on**: plan 036 (traced helpers / span-status conventions)
- **Category**: direction
- **Planned at**: commit `408be17` (Parallax) / `ed1f975` (playground), 2026-07-07

## Why this matters

The compose stack runs `postgres:17` that **no service connects to** —
grep for `sqlx|DATABASE_URL|jdbc|5432` across `services/`, `libs/`, `cli/`
is empty. Meanwhile the "database" scenarios are theater: inventory's slow
query is a `tokio::time::sleep`
(`services/inventory/src/main.rs:30-33`), pool contention is an in-process
mutex, and the N+1 is repeated HTTP calls. The brief (domain E; backlog A25)
wants real `db.*` spans, `db.query.text`, pool metrics
(`db.client.connection.*`), slow queries, and DB-level N+1 — the DB lane of
Parallax's trace/runtime UI has zero genuine evidence until this lands.

## Current state

Verified at playground commit `ed1f975`.

- Dead container: `deploy/docker-compose.yml:70-73`:

  ```yaml
  postgres:
    image: postgres:17
    environment: { POSTGRES_PASSWORD: playground, POSTGRES_DB: playground }
    networks: [pg]
  ```

  (plan 037 parameterizes the password; no service references this host.)

- Fake DB latency — `services/inventory/src/main.rs:28-33`:

  ```rust
  #[tracing::instrument(skip(p), fields(otel.kind = "server"))]
  async fn reserve(Query(p): Query<Reserve>) -> impl IntoResponse {
      if p.slow > 0 {
          tracing::info!(ms = p.slow, "slow db query (chaos)");
          tokio::time::sleep(std::time::Duration::from_millis(p.slow)).await;
  ```

- Fake pool contention: checkout `?lock` holds a `tokio::sync::Mutex`
  (fields doc at `services/checkout/src/main.rs:49-52`).
- HTTP-N+1: `services/checkout/src/main.rs:157-161` loops `reserve()` HTTP
  calls (`?n1=<n>`); stays as-is (it demos service-level N+1); this plan
  adds the DB-level variant inside inventory.
- Workspace conventions: reqwest pinned to `native-tls`
  (`Cargo.toml:34`) — repo + Parallax TLS rule: **never rustls**; sqlx must
  use its `tls-native-tls` feature (or no TLS — in-network Postgres is
  plaintext; simplest correct: no TLS feature, document it).
- Semconv targets (brief's commitment table): `db.system.name`,
  `db.namespace`, `db.operation.name`, `db.query.summary`, opt-in
  `db.query.text`, `db.client.operation.duration`,
  `db.client.connection.count/pending_requests/timeouts/wait_time/max`.

## Commands you will need

| Purpose | Command (playground root) | Expected |
|---------|---------------------------|----------|
| Build | `rtk cargo build` | exit 0 |
| Lint | `rtk cargo clippy --all-targets -- -D warnings` | exit 0 |
| Compose | `docker compose -f deploy/docker-compose.yml config` | exit 0 |
| Stack | `docker compose -f deploy/docker-compose.yml up --build -d postgres inventory checkout pricing recommendation` | healthy |

## Scope

**In scope** (playground repo):
- `services/inventory/` (sqlx integration, schema bootstrap, chaos knobs)
- workspace `Cargo.toml` (sqlx dep)
- `libs/playground-telemetry/` (a small `db_span` helper + pool-metrics
  gauge task, if factoring is cleaner than inline)
- `deploy/docker-compose.yml` (inventory env: `DATABASE_URL`; postgres
  healthcheck + `depends_on`)
- `scenarios/a25-postgres.sh` (create) + catalog rows

**Out of scope**:
- Wiring catalog (Java/Hikari) to Postgres — valuable follow-up
  (Hikari pool metrics via the agent), deferred to keep one moving part.
- Redis/any new infra (brief rule: cache scenarios stay in-process).
- Removing the checkout HTTP-N+1 (different demo).
- notifications' DB spans (its stub is noted in the repo; separate).

## Git workflow

- Playground repo, `main`, Conventional Commits, `git commit -s`, one

## Steps

### Step 1: sqlx pool + schema bootstrap in inventory

1. Workspace `Cargo.toml`: add
   `sqlx = { version = "<latest stable>", default-features = false, features = ["runtime-tokio", "postgres", "macros"] }`
   — NO TLS feature (in-network plaintext; add a comment citing the repo
   TLS rule: if TLS is ever needed use `tls-native-tls`, never rustls).
2. Inventory `main`: read `DATABASE_URL` (default
   `postgres://postgres:playground@postgres:5432/playground` — matches the
   compose default; the password comes from env in real runs); build a
   `PgPoolOptions` pool with `max_connections(5)` and a 2s
   `acquire_timeout` (small on purpose — the exhaustion scenario needs it).
3. On startup, bootstrap idempotently:
   `CREATE TABLE IF NOT EXISTS stock (sku TEXT PRIMARY KEY, quantity BIGINT NOT NULL)`
   + upsert a handful of seed SKUs (the ones scenarios use: `WIDGET-1` etc.
   — grep scenarios for skus). Startup must announce the DB connection
   (narration rule) and **fail fast with a clear error if Postgres is
   unreachable** (the compose adds a healthcheck below; local no-Docker runs
   get a clear message + `INVENTORY_NO_DB=1` escape hatch that falls back
   to the current in-memory behavior so `cargo run` demos still work).

**Verify**: `rtk cargo build` + clippy exit 0; `docker compose up postgres
inventory` → inventory logs "connected to postgres" and serves `/reserve`.

### Step 2: Real queries with db.* spans

Replace the fake paths in `reserve`:
- Normal: `UPDATE stock SET quantity = quantity - $1 WHERE sku = $2 AND quantity >= $1 RETURNING quantity`
  → out-of-stock when no row (keep the existing 503 + `mark_span_error`
  semantics from plan 036 for `?fail=1`, and make a genuinely-empty sku also
  503).
- Wrap each query in a client span with semconv attrs:
  `db.system.name="postgresql"`, `db.namespace="playground"`,
  `db.operation.name="UPDATE"|"SELECT"`, `db.query.summary` (short form),
  `db.query.text` (full text — lab, opt-in by default ON here; comment why),
  plus `server.address`/`server.port`. Factor a
  `playground_telemetry::db_span(op, summary, text)` helper if used >2
  times.
- `?slow=<ms>` becomes a real slow query: `SELECT pg_sleep($1::float/1000)`
  inside the span (delete the `tokio::time::sleep`).
- DB-N+1 knob: `?db_n1=<n>` runs n sequential `SELECT quantity FROM stock WHERE sku=$1`
  single-row queries (one span each) before the update.

**Verify**: build + clippy; live: a `/reserve?slow=300` trace shows a
`pg_sleep` db span of ~300ms (record trace id).

### Step 3: Pool metrics + exhaustion knob

1. Gauge task (5s interval, same pattern as plan 045's Tokio task):
   `db.client.connection.count` (pool.size()),
   `db.client.connection.idle` (num_idle()), `db.client.connection.max`
   (5), and count acquire timeouts into
   `db.client.connection.timeouts` (increment on `acquire` Err).
2. `?hold_ms=<n>` knob: acquires a connection and holds it `n` ms (bounded
   ≤10s) — concurrent holds exhaust the 5-conn pool; further requests hit
   the 2s acquire timeout → 503 + `mark_span_error("pool_exhausted")` +
   `db.client.connection.pending_requests` visible.

**Verify**: build + clippy; live: 6 parallel `?hold_ms=5000` + 1 normal
reserve → the normal one fails with the pool error; gauges move (record).

### Step 4: Compose + scenario

1. Compose: inventory gets `DATABASE_URL` env; postgres gets a healthcheck
   (`pg_isready`) and inventory `depends_on: postgres: condition: service_healthy`.
2. `scenarios/a25-postgres.sh`: four phases — normal reserves; slow query
   burst (`?slow=400`); DB-N+1 (`?db_n1=12`); pool exhaustion (parallel
   `?hold_ms=4000`). Each phase prints "Check in Parallax": db spans with
   `db.query.text` in trace detail; slow-span duration; N+1 fan of SELECT
   spans; `db.client.connection.*` gauges + pool_exhausted errors
   (runtime lane once Parallax plan 044 lands). Register in
   `scenarios/run.sh` + README.

**Verify**: compose config exit 0; `bash -n` exit 0; live run recorded with
trace ids per phase.

## Test plan

- Rust: keep it integration-shaped — the sqlx paths need a live DB; add
  `#[ignore]`-marked tests only if the repo already has that pattern
  (check; likely not) — otherwise the recorded live phases of a25 are the
  acceptance evidence, plus build/clippy gates.
- The `INVENTORY_NO_DB=1` fallback gets one unit test if it isolates
  cleanly; else verify manually with `cargo run` and record.

## Done criteria

- [ ] inventory uses a real Postgres pool; `INVENTORY_NO_DB=1` fallback
      works for no-Docker runs
- [ ] db spans carry `db.system.name/operation.name/query.summary/query.text`
      (recorded trace id)
- [ ] `db.client.connection.*` gauges emitted; exhaustion produces
      `pool_exhausted` errors (recorded)
- [ ] `a25-postgres.sh` in the catalog with per-phase Parallax checks
- [ ] `rtk cargo build` + clippy zero warnings; compose config green;
      no rustls feature anywhere (`rtk grep -rn rustls Cargo.toml` → only
      absent/comment)
- [ ] Status row updated in Parallax repo `plans/README.md`

## STOP conditions

- sqlx latest-stable's feature set forces a TLS backend choice where
  native-tls is unavailable for the chosen version — report the version
  matrix (TLS rule is hard).
- Pool gauges require sqlx internals not publicly exposed — emit only what
  `PgPool` exposes (`size`, `num_idle`) and report the rest as unavailable;
  don't fork sqlx.
- Startup bootstrap races the healthcheck (migrations flaking in compose) —
  stop after two attempts and report the sequencing that failed.

## Maintenance notes

- Follow-up (deferred): catalog on Hikari + agent-exported
  `db.client.connection.*` for the Java pool story; notifications' DB span
  fleshing (its own TODO).
- Parallax consumers: trace detail already renders `db.query.text`
  (brief inventory); plan 044's runtime lane picks up the
  `db.client.connection.` prefix family — verify the prefix list includes
  it (it does — listed in 044 Step 3).
- Reviewer: the UPDATE must be atomic (no read-then-write); knobs bounded
  (`hold_ms` ≤10s, `db_n1` ≤50); `db.query.text` never interpolates user
  input (bind params only — the knob values are numbers).
