# Plan 024: Enforce the configured GraphQL depth/complexity limits and add a Host-header guard on the API listener

> **Executor instructions**: Follow this plan step by step. Run every
> verification command and confirm the expected result before moving to the
> next step. If anything in the "STOP conditions" section occurs, stop and
> report — do not improvise. When done, update the status row for this plan
> in `plans/README.md`.
>
> **Drift check (run first)**: `git diff --stat 8bc3f13..HEAD -- crates/parallax-server/src/serve.rs crates/parallax-server/src/config.rs crates/parallax-api/src/lib.rs`
> On excerpt mismatch, STOP.

## Status

- **Priority**: P2
- **Effort**: M
- **Risk**: MED
- **Depends on**: none (independent of 022/023)
- **Category**: security
- **Planned at**: commit `8bc3f13`, 2026-07-07

## Why this matters

`config.rs` defines `graphql_max_depth: 8` and `graphql_max_complexity: 1000`,
but nothing installs them: `build_schema()` constructs a bare
`Schema::new(...)` and the handler never consults `config.limits`. The keys
are dead — worse than missing, because their presence implies a protection
that does not exist. As plans 029/030 add deeply nested graph/story surfaces,
an alias-amplified or deeply nested query has no cost ceiling against
GreptimeDB. Separately, the API binds loopback with no auth by design
(accepted), but a no-auth loopback service still needs a Host/Origin guard:
without one, a web page the developer visits can use DNS rebinding to reach
`http://127.0.0.1:4000/graphql` as same-origin and read all local telemetry
(and drive the SQL surface). That guard is the standard defense for this
posture and is missing.

## Current state

- `crates/parallax-server/src/config.rs:51-57` and `:106-114`:

  ```rust
  pub struct LimitsConfig {
      pub graphql_max_depth: usize,
      pub graphql_max_complexity: usize,
  }
  // defaults:
  graphql_max_depth: 8,
  graphql_max_complexity: 1_000,
  ```

- `crates/parallax-api/src/lib.rs:1929-1933`:

  ```rust
  pub fn build_schema() -> Schema {
      Schema::new(Query, Mutation, EmptySubscription::new())
  }
  ```

- `crates/parallax-server/src/serve.rs:166-188` — the router mounts
  `/graphql`, the SSE routes, and OTLP routes with **no** `CorsLayer` and no
  Host/Origin check; `build_schema()` is called without `config.limits`.
  `crates/parallax-server/src/serve.rs:75-91` — `graphql_handler` executes
  any request. Bind default is `127.0.0.1` (`config.rs:75`).
- The library is **Juniper** (`lib.rs:10`). Juniper does not ship a built-in
  depth/complexity middleware; depth must be enforced by inspecting the query
  AST or by a rule. Confirm the Juniper version in `Cargo.toml` before picking
  an approach (see Step 1).
- Repo conventions: `native-tls` only; axum + tower on the server; zero
  clippy warnings; cargo-nextest.

## Commands you will need

| Purpose | Command (repo root)                                                  | Expected |
|---------|----------------------------------------------------------------------|----------|
| Format  | `rtk cargo fmt --all`                                                | exit 0   |
| Lint    | `rtk cargo clippy --workspace --all-targets --locked -- -D warnings` | exit 0   |
| Tests   | `rtk cargo nextest run --workspace`                                  | all pass |
| Deps    | `grep -n 'juniper' crates/parallax-api/Cargo.toml crates/parallax-server/Cargo.toml` | shows version |

## Suggested executor toolkit

- Before writing depth-limit code, use Context7 (`resolve-library-id`
  "Juniper" → `query-docs`) to confirm how the pinned Juniper version exposes
  query depth (validation rules vs. manual AST walk). Do not guess the API.

## Scope

**In scope**:
- `crates/parallax-server/src/serve.rs`
- `crates/parallax-api/src/lib.rs` (schema wiring / a depth-check helper)
- `crates/parallax-server/tests/` (new test file)

**Out of scope**:
- Adding authentication/RBAC — explicitly not V1 (local-first, accepted).
- OTLP ingest payload limits — a separate low-severity item (see README
  "considered"); not this plan.
- Changing bind defaults away from loopback.
- The `metricSeries`/`sql` injection fixes — plan 022.

## Git workflow

- `main`, Conventional Commits, `git commit -s`, one agent trailer. Push when
  done.

## Steps

### Step 1: Decide the depth-enforcement mechanism (spike, then implement)

Check the Juniper version (`grep juniper crates/*/Cargo.toml`) and confirm via
Context7 whether it exposes a query-depth validation hook. Two acceptable
implementations, pick the one the version supports:

- **(a)** A pre-execution depth check: parse the incoming query with
  `juniper::parser`, walk the selection set, and reject if nesting exceeds
  `graphql_max_depth`. Put the walker in `parallax-api` as
  `pub fn check_query_depth(query: &str, max_depth: usize) -> Result<(), String>`.
- **(b)** If the version has a validation-rule API, register a depth rule in
  `build_schema` / at execution.

If neither is feasible without a large dependency, implement (a) — a bounded
recursive selection-set walk is small and testable. Do **not** pull in a new
GraphQL library.

Change `build_schema()` to accept limits, or add the depth check in the
handler path — whichever the mechanism dictates. If `build_schema`'s
signature changes, update all callers (`grep -rn build_schema crates/`).

**Verify**: `rtk cargo clippy --workspace --all-targets --locked -- -D warnings` → exit 0.

### Step 2: Enforce depth in the handler and return a GraphQL error

In `serve.rs`'s `graphql_handler` (`serve.rs:75-91`), before executing, run
the depth check using `config.limits.graphql_max_depth`. On violation return
a GraphQL error response (HTTP 200 with an `errors` array, or 400 —
match how the existing error path responds). Thread `config.limits` into the
handler state if it is not already there (the handler already has access to
the schema/context; add limits alongside).

Complexity: if the pinned Juniper exposes no complexity metric, approximate
"complexity" as (depth-limit + a field-count cap): reject queries whose total
selected-field count exceeds `graphql_max_complexity`. Document the
approximation in a code comment. Do not over-engineer.

**Verify**: `rtk cargo nextest run --workspace` → all pass.

### Step 3: Add a Host-header guard on the API listener

Add a tower middleware (or an axum `from_fn` layer) on the API router in
`serve.rs:174-188` that rejects requests whose `Host` header is not in an
allowlist of `localhost` / `127.0.0.1` / `[::1]` (with optional `:port`).
Return 403 on mismatch. Apply it to `/graphql` and the SSE routes. Decide
whether OTLP ingest routes are included — telemetry senders may set a
container hostname (`host.docker.internal`), so **exclude** the OTLP routes
from the Host guard (or make the allowlist configurable) to avoid breaking
the playground. Document that choice in a comment.

Make the allowlist derive from the configured bind address so a non-default
bind still works.

**Verify**: `rtk cargo nextest run --workspace` → all pass.

### Step 4: Tests

New integration test file `crates/parallax-server/tests/m5_gates_limits.rs`
(model on an existing `crates/parallax-server/tests/*.rs` that spins the
server/schema over the memory store):

- A query nested deeper than 8 levels returns a GraphQL error mentioning
  depth; a shallow query succeeds.
- A request with `Host: evil.example.com` to `/graphql` gets 403; a request
  with `Host: 127.0.0.1:4000` succeeds.
- (If complexity approximation added) a wide alias-amplified query is
  rejected.

**Verify**: `rtk cargo nextest run --workspace -E 'test(m5_gates_limits)'` →
all pass.

## Test plan

Covered in Step 4. Cases: depth accept/reject, Host accept/reject, optional
complexity reject. Pattern: existing server integration tests over the memory
store.

## Done criteria

- [ ] `rtk cargo fmt --all` no diff; clippy exits 0 with `-D warnings`
- [ ] `rtk cargo nextest run --workspace` exits 0 with new tests present
- [ ] A >8-deep query is rejected (asserted by test)
- [ ] A foreign `Host` header on `/graphql` is rejected (asserted by test)
- [ ] `config.limits.graphql_max_depth`/`graphql_max_complexity` are read at
      runtime (`grep -n "graphql_max_depth" crates/parallax-server/src/serve.rs`
      returns a use site)
- [ ] No out-of-scope files modified (`git status`)
- [ ] `plans/README.md` status row updated

## STOP conditions

Stop and report if:

- Excerpts don't match live code (drift).
- The pinned Juniper version has no viable depth hook and implementing a
  parser-based walk would require a new heavy dependency — report the version
  and options instead of pulling one in.
- The Host guard breaks the OTLP ingest path for the playground even after
  excluding OTLP routes — report so the allowlist can be reconsidered.
- Adding limits changes the response shape in a way existing GraphQL tests
  assert against.

## Maintenance notes

- The complexity metric here is an approximation; if a later Juniper upgrade
  ships a real cost model, replace it and delete the field-count heuristic.
- Reviewer should confirm the Host guard's allowlist tracks the bind config,
  and that OTLP routes are intentionally excluded.
- This unblocks 029/030 safely: the graph/story resolvers can be deep without
  an unbounded-cost risk.
- Deferred: real auth is out of scope by product decision; revisit if
  Parallax ever binds non-loopback or goes multi-user.
