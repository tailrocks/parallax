# Plan 078: Split the 5,291-line GraphQL monolith into domain modules

> **Executor instructions**: Follow this plan step by step. Run every
> verification command and confirm the expected result before moving to the
> next step. If anything in the "STOP conditions" section occurs, stop and
> report — do not improvise. When done, update the status row for this plan
> in `advisor-plans/README.md` — unless a reviewer dispatched you and told you
> they maintain the index.
>
> **Drift check (run first)**: `git diff --stat dbaba3c..HEAD -- crates/parallax-api/src/`
> This file is the repo's highest-churn hotspot — expect drift; the split is
> structural, so proceed as long as the anchor points below (Query/Mutation
> impls, ApiContext, check_query_limits) still exist; STOP only if the crate
> has already been modularized.

## Status

- **Priority**: P2
- **Effort**: L
- **Risk**: MED (large mechanical move; behavior must be provably unchanged)
- **Depends on**: 069 (CI runs the full suites that guard this), and do it
  AFTER 072/073/075 land to avoid conflicting with their small edits.
- **Category**: tech-debt
- **Planned at**: commit `dbaba3c`, 2026-07-10

## Why this matters

`crates/parallax-api/src/lib.rs` is 5,291 lines: 47 `#[graphql_object]` impl
blocks, ~380 fns, every wrapper type and resolver for 11 distinct domains in
one file. A 2026-07-07 review deferred splitting it at 2,181 lines expecting
opportunistic modularization; instead it grew 2.4×. Every feature plan and
every fix touches this one file — merge conflicts, slow navigation, reviews
that scroll. The domains have clean seams (verified: resolvers group cleanly
by issues/traces/logs/metrics/services/runs/dashboards/investigations/story/
sql/field-explorer). This is a mechanical, behavior-preserving split.

## Current state

- `crates/parallax-api/src/lib.rs` anchors (at `dbaba3c`):
  - `:1-27` — module doc (Juniper choice is an operator instruction —
    preserve the doc comment verbatim at the crate root) and imports.
  - `:29-35` — `pub struct ApiContext { store, metadata, otlp_grpc_port }` +
    `impl juniper::Context`.
  - `:37-39` — `fn field_err(e) -> FieldError` (used ~119×).
  - `:141-1637` — ~45 `#[graphql_object(context = ApiContext)]` wrapper-type
    impls (Issue, Span, Log, TraceSummary, ServiceOverview, Point, BundleOut,
    …), each preceded by its plain struct definition.
  - `:1639` `pub struct Query;` / `:1642` `impl Query` (~1,440 lines, ~50
    resolvers).
  - `:3085` `pub struct Mutation;` / `:3088` `impl Mutation` (9 resolvers).
  - `:3345` `pub fn check_query_limits(...)` — depth/complexity guard (plan
    024 heritage; has dedicated tests).
  - `:3416` `mod tests` — 28 resolver tests (in-file).
- Juniper constraint: ONE `#[graphql_object]` impl per type — `Query` cannot
  be split across files as multiple impls of the same type. The split
  therefore moves the *wrapper types* (+ their impls) and the *resolver
  bodies* (as free functions or per-domain modules), while `Query`/`Mutation`
  stay single impls that delegate.
- Consumers of this crate's public items (check before changing visibility):
  `crates/parallax-server/src/serve.rs` (schema construction + `check_query_limits`),
  possibly tests under `crates/parallax-server/tests/`. Run
  `rtk cargo doc -p parallax-api --no-deps` or grep server code for
  `parallax_api::` to enumerate the real public surface.
- Conventions: workspace clippy `unwrap_used = warn`, zero-warning gate;
  edition 2024 module style (`mod x;` files, no `mod.rs` needed).

## Commands you will need

| Purpose | Command | Expected on success |
|---------|---------|---------------------|
| Build | `rtk cargo build -p parallax-api` | exit 0 |
| API tests | `rtk cargo nextest run -p parallax-api` | all pass |
| Server integration | `rtk cargo nextest run -p parallax-server` | all pass |
| Full suite | `rtk cargo nextest run --workspace` | all pass |
| Lint | `rtk cargo clippy --workspace --all-targets` | zero warnings |
| Schema-shape guard | see Step 1 | identical before/after |

## Scope

**In scope** (the only files you should modify):
- `crates/parallax-api/src/lib.rs`
- `crates/parallax-api/src/resolvers/*.rs` (create; one per domain)
- `crates/parallax-api/src/types/*.rs` (create; wrapper structs + graphql_object impls, if separating types from resolvers — optional, see Step 3)
- `advisor-plans/README.md` (status row)

**Out of scope** (do NOT touch, even though they look related):
- ANY behavior change: no renamed GraphQL fields, no changed args, no changed
  limits/caps, no resolver-body edits beyond relocation.
- `crates/parallax-server/*` except if an import path must update (prefer
  re-exports from `parallax-api` root so server code doesn't change).
- The 28 in-file tests' assertions (they may MOVE with their domain).

## Git workflow

- Work directly on `main` (repo rule — `BRANCHING.md`).
- One commit per extracted domain (reviewable chunks), Conventional Commits,
  DCO signoff (`git commit -s`), trailer
  `Co-authored-by: Claude <noreply@anthropic.com>`. E.g.
  `refactor(api): extract traces domain module`.

## Steps

### Step 1: Capture a schema-shape baseline

Before touching anything, produce a machine-comparable schema artifact.
Juniper exposes the schema; the cheapest robust baseline: add a TEMPORARY
test (or use an existing schema test if one exists — grep `mod tests` for
`schema`) that prints `RootNode::as_sdl()` (juniper 0.17: `schema.as_sdl()`
via `juniper::introspect` or the `RootNode::as_parser_document`/SDL support —
check the juniper 0.17 docs for the exact API; `graphql-parser` feature may be
needed). Write it to `target/schema-before.graphql` (target/ is ignored).

If SDL export proves unavailable in juniper 0.17 without new dependencies,
fall back to: run the FULL test suites before and after as the behavioral
baseline and rely on the compiler for shape (juniper resolvers are typed —
moving code cannot silently rename a GraphQL field unless attribute macros
are edited; your discipline is: never edit an attribute or signature during a
move). Record which baseline you used in the commit message.

**Verify**: baseline artifact exists, or the fallback decision is recorded.

### Step 2: Create the module skeleton

In `crates/parallax-api/src/`, create `resolvers/` with one module per
domain, wired from `lib.rs`:

```
resolvers/
  issues.rs      traces.rs     logs.rs      metrics.rs
  services.rs    runs.rs       dashboards.rs investigations.rs
  story.rs       sql.rs        fields.rs    // field-explorer + attribute_compare + evidence_gaps
```

`lib.rs` keeps: the crate doc comment, `ApiContext`, `field_err` (make it
`pub(crate)`), `Query`/`Mutation` structs + their single impls,
`check_query_limits`, schema constructor, and `pub use` re-exports preserving
today's public paths.

**Verify**: `rtk cargo build -p parallax-api` → exit 0 (empty modules).

### Step 3: Move domains one at a time

Per domain (start with the smallest — `dashboards` or `sql` — to validate the
pattern, then the big ones; `traces` last):

1. Move the domain's wrapper structs AND their `#[graphql_object]` impls into
   `resolvers/<domain>.rs` verbatim (cut-paste; imports adjusted; no other
   edits). Types used across domains (e.g. `Point`, shared range args) go to
   a `resolvers/common.rs`.
2. In `impl Query`, keep each resolver fn but reduce its body to a delegation:
   `resolvers::traces::trace(context, id, ...).await` — move the body into a
   `pub(crate) async fn` in the domain module with the same parameters plus
   `&ApiContext`. (Alternative acceptable shape: leave one-liner bodies in
   place if the body is already ≤3 lines; don't create delegation for
   trivia.)
3. Move the domain's tests from the in-file `mod tests` into
   `resolvers/<domain>.rs`'s own `#[cfg(test)] mod tests` (or a shared
   `tests` module per file), keeping every assertion identical.

**Verify after EACH domain**: `rtk cargo build -p parallax-api` &&
`rtk cargo nextest run -p parallax-api` && `rtk cargo nextest run -p parallax-server`
→ all pass. Commit the domain.

### Step 4: Compare schema shape

Re-run the Step 1 artifact as `target/schema-after.graphql`;
`diff target/schema-before.graphql target/schema-after.graphql` → empty.
(Or, on the fallback path: full workspace suite green.)
Remove the temporary schema-dump test if it was added ad hoc, or keep it as a
permanent regression test if it's ≤30 lines — prefer keeping it, named
`schema_sdl_snapshot`, asserting the SDL contains a few sentinel field names
rather than the whole document (a whole-document assert would churn on every
legitimate schema change).

**Verify**: empty diff (or suite-green fallback); `lib.rs` line count now
< 800 (`wc -l crates/parallax-api/src/lib.rs`).

### Step 5: Full gates

**Verify**: `rtk cargo fmt --all`;
`rtk cargo clippy --workspace --all-targets` → zero warnings;
`rtk cargo nextest run --workspace` → all pass.

## Test plan

- No new behavioral tests — the 28 existing resolver tests + server
  integration suites are the characterization net; they move but do not
  change.
- Optional lasting artifact: the `schema_sdl_snapshot` sentinel test from
  Step 4.

## Done criteria

Machine-checkable. ALL must hold:

- [ ] `wc -l crates/parallax-api/src/lib.rs` → < 800
- [ ] `ls crates/parallax-api/src/resolvers/` → ≥10 domain files
- [ ] Schema baseline diff empty (or fallback recorded in commit message)
- [ ] `rtk cargo nextest run --workspace` exits 0; test COUNT ≥ the count at
      `dbaba3c` (no tests lost in the move — compare `cargo nextest list | wc -l`
      before/after)
- [ ] `rtk cargo clippy --workspace --all-targets` → zero warnings
- [ ] `grep -rn "parallax_api::" crates/parallax-server | sort` unchanged vs
      `dbaba3c` (public surface preserved) — OR the diff is only additive
      re-export paths
- [ ] `git status` shows no modified files outside the in-scope list
- [ ] `advisor-plans/README.md` status row updated

## STOP conditions

Stop and report back (do not improvise) if:

- Juniper 0.17 macro constraints prevent moving a `#[graphql_object]` impl to
  another module without editing the attribute (e.g. context-path resolution
  issues) that can't be fixed by a plain `use` — report the exact error.
- Any test must have its ASSERTION changed to pass after a move.
- The schema diff (Step 4) is non-empty.
- A domain's extraction forces changes in `parallax-server` beyond import
  lines.

## Maintenance notes

- Future resolvers go in their domain module; a new resolver added to
  `lib.rs` is a review-blocking smell after this lands.
- The 119× `.map_err(field_err)` boilerplate and the anyhow-string error
  collapse remain — a typed `StoreError` (storage crate) → GraphQL error-kind
  mapping is the recorded follow-up (see index: deferred findings).
- This split is the prerequisite that makes the deferred "typed errors" and
  any per-domain dataloader work reviewable.
