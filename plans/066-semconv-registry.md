# Plan 066: Shared semantic-convention registry — central attribute constants in both repos, cross-language agreement, Weaver spike

> **Executor instructions**: This plan spans **both repositories** (Parallax
> and `parallax-telemetry-playground`). Follow step by step; run every
> verification. On any STOP condition, stop and report. When done, update
> the status row in `plans/README.md` (Parallax repo).
>
> **Drift check (run first)**:
> Parallax: `git diff --stat ed5b10f..HEAD -- crates/parallax-core/src crates/parallax-storage/src crates/parallax-api/src`
> Playground: `git diff --stat ed1f975..HEAD -- libs/playground-telemetry web/src services`
> On mismatch with the excerpts below, STOP.

## Status

- **Priority**: P3
- **Effort**: L
- **Risk**: MED (touches attribute strings across ingest/storage/API — pure
  refactor, but a typo changes what gets stored)
- **Depends on**: none hard. Sequencing: land AFTER the in-flight plan waves
  that add attribute literals (036/042/047-050/054-056/063) or accept rebase
  churn — the registry migrates whatever exists when it runs.
- **Category**: tech-debt + dx
- **Planned at**: commit `ed5b10f` (Parallax) / `ed1f975` (playground), 2026-07-07

## Why this matters

Telemetry attribute names are Parallax's real public API, and today they are
~142 scattered string literals across ~23 Rust files — `parallax.run.id`
alone appears as a normalize key, a column DDL, and a SELECT JSON path with
no compile-time coupling; `"service.name"` is spelled in six GreptimeDB JSON
paths plus derive and normalize. The playground has the same disease in three
languages, including a real value-level bug this plan fixes in passing: Rust
defaults the environment to `"lab"` while TS hardcodes and compose sets
`"playground"`, so out-of-compose Rust runs split every cross-language trace
across two environments. The research brief prescribes the endgame (a Weaver
registry generating constants for Rust/Java/TS); this plan does the
mechanical consolidation now and spikes Weaver honestly instead of assuming
it.

## Current state

Verified at `ed5b10f` (Parallax) / `ed1f975` (playground).

- Parallax scatter (no constants module exists — grep for a `semconv`
  module → none):
  - `crates/parallax-core/src/normalize.rs:89` —
    `attr_str(resource_attrs, "parallax.run.id")`
  - `crates/parallax-storage/src/greptime.rs:152` —
    `ALTER TABLE opentelemetry_logs ADD COLUMN "parallax.run.id" STRING`
  - `crates/parallax-storage/src/greptime.rs:352` —
    `cols.opt_string("resource_attributes.parallax.run.id", row)`
  - `greptime.rs:376` —
    `json_get_string("resource_attributes", '$."service.name"')`
  - plus `derive.rs` (`"service.name"`, `exception.type/message/stacktrace`
    twice), `lib.rs` metric-name arrays (`REQUEST_DURATION_METRICS` at
    `:700`, CPU/memory candidates `:709-730`, `BUNDLE_WINDOW_METRICS`
    `:1723`), CLI crates. Approx totals: `"service.name"` ~18,
    `parallax.run.id` ~13, `exception.*` ~32 across product+tests.
- Playground scatter:
  - `libs/playground-telemetry/src/lib.rs:63-64` — only SERVICE_NAME/
    SERVICE_VERSION come from the semconv crate; everything else literal.
  - `libs/playground-telemetry/src/lib.rs:116-119` — **the env bug**:

    ```rust
    environment: Some(
        std::env::var("PARALLAX_ENV")
            .unwrap_or_else(|_| "lab".into())
            .into(),
    ),
    ```

    vs `web/src/telemetry.ts:28` — `"deployment.environment.name":
    "playground"` (hardcoded) and compose's `PARALLAX_ENV: "playground"`.
    (Plan 036 Step 1 also writes a `deployment.environment.name` resource
    attr defaulting `"lab"` — if 036 landed, the default lives there too;
    align BOTH to `"playground"`.)
  - `otel.kind` literals in every `#[instrument]`
    (checkout `main.rs:90`, recommendation `main.rs:24`, cli `main.rs:26,38`,
    etc.); `user.tier`/`canary.*` (checkout `:97,119-122`);
    `"catalog.product.queries"` (Java `CatalogApplication.java:54`).
- Consumers that must agree cross-repo: Parallax normalize reads
  `parallax.run.id` / `service.name`; playground (after plans 036/050/056)
  writes them plus `session.id`, `app.screen.*`, event taxonomy names.

## Commands you will need

| Purpose | Command | Expected |
|---------|---------|----------|
| Parallax | `rtk cargo build --workspace && rtk cargo clippy --workspace --all-targets && rtk cargo nextest run` | clean |
| Playground Rust | `rtk cargo build && rtk cargo clippy --all-targets -- -D warnings` | clean |
| Playground web | `cd web && bun run build` | exit 0 |
| Playground Java | service gradlew build | exit 0 |

## Scope

**In scope**:
- Parallax: new `crates/parallax-core/src/semconv.rs` (constants; re-export
  from core so storage/api/cli can depend without a new crate); mechanical
  literal replacement across `crates/` **product code** (tests may keep
  literals — they double as drift detectors)
- Playground: new `libs/playground-telemetry/src/semconv.rs` + literal
  replacement in Rust services; TS: a `web/src/semconv.ts` consts file;
  Java: a `Semconv.java` constants class in each service's shared spot (or
  one per service — smallest honest step)
- Playground env-default fix: `"lab"` → `"playground"` in
  `libs/playground-telemetry/src/lib.rs:118` (+ plan 036's equivalent if
  landed); `web/src/telemetry.ts:28` reads `VITE_PARALLAX_ENV ?? "playground"`
- Weaver spike: a design note
  `docs/research/architecture/semconv-registry-design.md` (Parallax repo)
- Tests: one cross-repo agreement test (see Step 4)

**Out of scope** (do NOT touch):
- Renaming any emitted attribute (pure consolidation — byte-identical
  wire names; the ONLY value change is the environment default, called out
  above).
- Generating code from a Weaver registry in CI — the spike decides; wiring
  is a follow-up plan.
- The `lib.rs` split (advisor-plans README rejected item) — placing consts
  in `parallax-core` deliberately avoids it.
- GreptimeDB column names that are engine-generated (`span_name`,
  `service_name` columns) — those are the engine's schema, not our
  attribute names; do NOT const-ify them in this plan (note: the JSON-path
  fragments like `$."service.name"` DO use our attribute name inside the
  path — build those paths from the const with a helper).

## Git workflow

- Each repo on its own `main`, Conventional Commits, `git commit -s`, one
  both when done.

## Steps

### Step 1: Parallax `semconv.rs`

Create `crates/parallax-core/src/semconv.rs`: doc-commented `pub const`
groups — OTel standard (`SERVICE_NAME`, `SERVICE_VERSION`,
`DEPLOYMENT_ENVIRONMENT_NAME`, `EXCEPTION_TYPE/MESSAGE/STACKTRACE`,
`EVENT_NAME`…), Parallax overlay (`PARALLAX_RUN_ID = "parallax.run.id"`),
and the well-known metric-name arrays currently inlined in `lib.rs`
(`REQUEST_DURATION_METRICS` etc. — move them here). Add helper
`pub fn resource_json_path(attr: &str) -> String` producing the
`$."<attr>"` fragment and
`pub fn resource_column(attr: &str) -> String` producing
`resource_attributes.<attr>` so greptime.rs call sites compose from the
const. Replace product-code literals found by:
`rtk grep -rn '"parallax.run.id"\|"service.name"\|"exception\.' crates/ --include=*.rs`
(skip `tests/`; skip doc comments). The DDL string at `greptime.rs:152`
composes via `format!` from `PARALLAX_RUN_ID`.

**Verify**: full Parallax gate clean; then the drift detector:
`rtk grep -rn '"parallax.run.id"' crates/ --include=*.rs` → hits ONLY in
`semconv.rs` and test files.

### Step 2: Playground Rust `semconv.rs` + env fix

Mirror the pattern in `libs/playground-telemetry/src/semconv.rs`
(`PARALLAX_RUN_ID`, `OTEL_KIND = "otel.kind"`, event taxonomy names if plan
056 landed, `canary.*` keys, env default const
`DEFAULT_ENVIRONMENT = "playground"`). Fix `lib.rs:118`'s `"lab"` →
`DEFAULT_ENVIRONMENT` (and plan 036's resource-attr default if present).
Replace service literals mechanically.

**Verify**: playground Rust gate clean;
`rtk grep -rn '"lab"' libs/ services/ cli/` → no environment-default hits.

### Step 3: TS + Java constants

- `web/src/semconv.ts`: export the handful web uses
  (`DEPLOYMENT_ENVIRONMENT_NAME`, event names when 050/056 land); switch
  `telemetry.ts:28` to `import.meta.env.VITE_PARALLAX_ENV ?? "playground"`
  keyed by the const.
- Java: `Semconv.java` with the counter/attr names each service uses (e.g.
  catalog's `"catalog.product.queries"` → `CATALOG_PRODUCT_QUERIES`). One
  file per service is acceptable; note the duplication as what Weaver
  codegen would remove.

**Verify**: web build + Java builds clean.

### Step 4: Cross-repo agreement guard

In the playground, add a small Rust test in `libs/playground-telemetry`
asserting the load-bearing shared names equal their literal wire values
(`assert_eq!(semconv::PARALLAX_RUN_ID, "parallax.run.id")` etc. — the wire
string is the contract; the test freezes it against refactor typos). In
Parallax, same freeze test in `parallax-core`. These two tests are the
manual cross-repo lock until codegen exists.

**Verify**: both test suites green.

### Step 5: Weaver spike (design note, no build-system change)

Write `docs/research/architecture/semconv-registry-design.md`: check the
current OpenTelemetry Weaver release (CLI availability, registry schema,
codegen targets for Rust/Java/TS as of execution date), sketch the registry
layout for the `parallax.*` overlay + event taxonomy, estimate the codegen
integration per language, and end with a go/no-go recommendation and the
exact follow-up plan it implies. Cite versions/links. Do NOT add Weaver to
any build in this plan.

**Verify**: note exists, sourced, with an explicit recommendation line.

## Test plan

- The freeze tests (Step 4) are the core new tests.
- Everything else is covered by the full existing suites in both repos —
  the refactor must be behavior-invisible except the environment default,
  which gets its own test: `PARALLAX_ENV` unset → `"playground"` (extend
  the release-sourcing test home in `libs/playground-telemetry` — plan 042
  adds one; co-locate).

## Done criteria

- [ ] Both repos' full gates clean (Rust/clippy/nextest; web build; Java builds)
- [ ] Product-code literal greps localize to the semconv modules (+ tests)
      in both repos
- [ ] `PARALLAX_ENV`-unset Rust playground services emit
      `environment=playground` (test)
- [ ] Freeze tests pin the shared wire names in both repos
- [ ] Weaver design note committed with a go/no-go recommendation
- [ ] Status row updated in Parallax `plans/README.md`

## STOP conditions

- Any replacement would CHANGE a wire name (grep diff shows a literal that
  two call sites spell differently — e.g. a latent typo discovered
  mid-refactor): STOP and report it as a data-compat finding first; fixing
  a live typo may orphan stored columns.
- In-flight plans (036/042/047-050/054-056/063) are mid-execution in the
  same files — coordinate/rebase; never hand-merge attribute lists blind.
- Weaver's current release can't express the overlay (spike outcome) —
  that's a valid no-go; the hand-rolled modules stand.

## Maintenance notes

- Every future plan that adds an attribute name MUST add it to the semconv
  module(s), not inline — reviewers enforce.
- The Java per-service duplication and the TS/Rust/Java triple maintenance
  are the standing cost that decides the Weaver follow-up.
- The environment-default change is the one observable behavior change;
  release notes / TOUR (plan 054) should mention it.
