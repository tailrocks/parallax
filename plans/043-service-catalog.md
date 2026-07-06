# Plan 043: Service catalog — identity/runtime/SDK/version surface on the services list and detail

> **Executor instructions**: Follow step by step; run every verification. On
> any STOP condition, stop and report. When done, update the status row in
> `plans/README.md`.
>
> **Drift check (run first)**:
> `git diff --stat 408be17..HEAD -- crates/parallax-storage crates/parallax-api ui/src/routes/services.tsx ui/src/routes/services.\$service.tsx`
> On mismatch with the excerpts below, STOP.

## Status

- **Priority**: P2
- **Effort**: M
- **Risk**: LOW
- **Depends on**: playground plan 036 (emits `service.namespace`,
  `service.instance.id`, and Java `service.version`) for a rich demo; the
  resolver works regardless
- **Category**: direction
- **Planned at**: commit `408be17`, 2026-07-07

## Why this matters

The research brief's Service Catalog section
(`docs/research/architecture/full-observability-ui-and-playground-research.md`,
"E. Service catalog") calls for a service to be more than a name + RED
numbers: identity (version, namespace), runtime/language, SDK, environment,
last-seen. All of that already lands in GreptimeDB as auto-widened
`resource_attributes.*` columns — nothing new is ingested; today the API
simply never reads it (`services` returns bare names, `service_summaries`
returns counts + p95 only). One resolver turns the services page into a
catalog and becomes the anchor for later health/ownership work.

## Current state

Verified at commit `408be17`.

- Resource attrs are columnar: `crates/parallax-storage/src/greptime.rs:315`
  (docs the auto-widening `span_attributes.*`/`resource_attributes.*`
  columns); `reassemble_attrs` at `greptime.rs:452-473` folds
  `resource_attributes.<k>` back into row maps;
  `resource_attributes.parallax.run.id` is read directly at `:352`. Typical
  emitted keys when SDKs are configured: `service.version`,
  `service.namespace`, `service.instance.id`, `telemetry.sdk.name`,
  `telemetry.sdk.language`, `telemetry.sdk.version`,
  `deployment.environment.name`, `process.runtime.*` — presence varies per
  service; columns exist only after first emission (missing-column tolerance
  precedent around `greptime.rs:206-262`).
- Current surface: `services` resolver returns `Vec<String>` names
  (`crates/parallax-api/src/lib.rs:1610`); `service_summaries`
  (`crates/parallax-storage/src/greptime.rs:874`) feeds `serviceList` with
  last-seen/span-count/error/p95-style numbers.
- UI: `ui/src/routes/services.tsx` renders the list (name link `:331`
  region, spans/errors links `:339-359`, HeatCells `:364-372`);
  `ui/src/routes/services.$service.tsx` is the detail (header `:276+`).
- Turso has no ownership metadata table (`crates/parallax-storage/src/metadata.rs:10-44`)
  — ownership is OUT of scope here (deferred; needs product decisions).
- Resolver conventions: single-file `parallax-api/src/lib.rs`, `FieldResult`,
  context adapter calls — model on `serviceList`/`services` (`:1610`
  region). Verification baseline: cargo build/clippy/nextest + UI bun gates.

## Commands you will need

| Purpose | Command | Expected |
|---------|---------|----------|
| Build | `rtk cargo build --workspace` | exit 0 |
| Lint | `rtk cargo clippy --workspace --all-targets` | zero warnings |
| Tests | `rtk cargo nextest run` | all pass |
| UI | (from `ui/`) `bun run typecheck && bun run lint && bun run test && bun run build` | exit 0 |
| Seeded manual check | `cargo run -p parallax-server --example seed` against a dev server (verify the exact invocation in the example header) | data visible |

## Scope

**In scope**:
- `crates/parallax-storage/src/adapter.rs`, `greptime.rs`, `memory.rs` —
  one `service_catalog(from, to)` read method
- `crates/parallax-api/src/lib.rs` — `serviceCatalog` resolver (or extend
  `serviceList`'s row type — decide in Step 1)
- `ui/src/lib/api.ts`, `ui/src/routes/services.tsx` (new columns),
  `ui/src/routes/services.$service.tsx` (identity card)
- test files

**Out of scope**:
- Ownership/team metadata (needs a Turso table + product input — deferred).
- Health scoring / evidence-gap columns (advisor-plans/032 output feeds this
  later).
- The ecosystem graph (advisor-plans/031).
- Any new top-level route — the brief says extend existing surfaces; the
  catalog lives on `/services`.

## Git workflow

- `main`, Conventional Commits, `git commit -s`, one
  `Co-authored-by: Claude <noreply@anthropic.com>` trailer. Push when done.

## Steps

### Step 1: Storage method

Add `service_catalog(from_nanos, to_nanos)` to the adapter returning per
service: `service_name`, latest `service.version`, `service.namespace`,
`deployment.environment.name`, `telemetry.sdk.language`,
`telemetry.sdk.name`, `telemetry.sdk.version`, `last_seen_nanos`,
`instance_count` (distinct `service.instance.id`, 0 when the column is
absent). Greptime impl: one aggregate over the spans table selecting the
resource columns with `MAX(ts_nanos)` and a latest-value strategy — simplest
correct shape: take the row with max ts per service via
`SELECT ... FROM <spans> WHERE ts BETWEEN ... GROUP BY service_name` with
`last_value`-style aggregates if supported, else a two-step (max ts per
service, then fetch those rows). **Decide by testing against a live
GreptimeDB** (dev server + seed) — record the working SQL in a code comment.
Guard every column with the existing missing-column tolerance so services
that never emitted an attr return nulls. Memory impl mirrors over its span
rows. Cap: reuse the row-cap convention (`MAX_ROWS` at
`parallax-api/src/lib.rs:44` region) — bounded GROUP BY over the window.

**Verify**: `rtk cargo build --workspace && rtk cargo nextest run` → memory
impl unit test green (see Test plan).

### Step 2: Resolver

Expose `serviceCatalog(fromNanos: String!, toNanos: String!): [ServiceCatalogRow!]!`
following the neighboring resolver style. Nulls stay nulls (GraphQL optional
fields) — the UI must render absence honestly (the brief's telemetry-quality
principle: missing identity is information).

**Verify**: `rtk cargo nextest run` → resolver test green.

### Step 3: Services list columns

In `ui/src/routes/services.tsx`: fetch `serviceCatalog` in the loader
alongside the existing `serviceList`; join by name. Add columns: `Version`
(monospace, "-" when null), `Runtime` (sdk.language, e.g. `rust`/`java`/
`nodejs`), `Env`. Keep the table lean — put `sdk.name/version` and
`instance_count` in a row tooltip or the detail page, not columns. Columns
sort with the existing data-table plumbing if the table supports it (check
`ui/src/components/console/data-table.tsx` sorting props; skip sorting if
not wired for these).

**Verify**: (from `ui/`) `bun run typecheck && bun run lint` → exit 0.

### Step 4: Identity card on service detail

In `ui/src/routes/services.$service.tsx`: add an "Identity" card (grid of
label/value pairs: version, namespace, environment, SDK, instances,
last seen) using the existing `Card`/`Badge` primitives, placed above or
beside the RED charts. Null values render as muted "not emitted" — this is
deliberate (shows instrumentation gaps).

**Verify**: `bun run typecheck && bun run build` → exit 0; manual with the
seed example (emits `service.version` — `crates/parallax-server/examples/seed.rs:22`):
services list shows a version for the seeded service; a service that lacks
attrs shows "-"/"not emitted". Record the check.

## Test plan

- Storage (memory adapter): three spans across two services, one with full
  identity attrs, one with none → catalog returns both rows, correct
  latest-version pick, nulls for the bare service. Model on existing memory
  adapter tests (grep `mod tests` in `crates/parallax-storage/src/memory.rs`).
- API: `serviceCatalog` resolver test following an existing `#[tokio::test]`
  resolver test in `parallax-api`.
- UI: gates + recorded manual check (no new route harness).

## Done criteria

- [ ] `rtk cargo build`, clippy zero warnings, `rtk cargo nextest run` green
      with the new tests
- [ ] `serviceCatalog` returns identity rows for seeded data; absent attrs
      are null, not fabricated
- [ ] Services list shows Version/Runtime/Env; detail shows the Identity
      card; absence renders as "not emitted"
- [ ] UI gates exit 0
- [ ] `plans/README.md` status row updated

## STOP conditions

- GreptimeDB rejects the latest-value aggregate shape and the two-step
  fallback needs per-service queries (N+1 against the DB) — report the
  working alternatives with measured cost before choosing one.
- Resource columns are not present even for seeded data (schema drift vs the
  `greptime.rs:315` doc) — report actual schema.
- The services loader becomes noticeably slower (>1s on lab data) — report;
  candidate for the `service_catalog_snapshots` materialization the brief
  lists, don't build it ad hoc.

## Maintenance notes

- Playground plan 036 adds namespace/instance/Java-version emission — the
  catalog demo gets rich only after it lands; the "not emitted" states are
  themselves a demo (instrumentation-quality story) until then.
- Later consumers: ownership metadata (Turso), health/evidence-gap columns
  (advisor-plans/032), ecosystem-map node drawers (advisor-plans/031) should
  reuse `ServiceCatalogRow`.
- Reviewer: dotted-column quoting in the Greptime SQL; the name-join in the
  services loader must not drop services present in only one of the two
  queries (full outer join semantics client-side).
