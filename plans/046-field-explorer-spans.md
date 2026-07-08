# Plan 046: Field explorer phase 1 — `fieldStats` over span/error attributes + explorer drawer on the traces list

> **Executor instructions**: Follow step by step; run every verification. On
> any STOP condition, stop and report. When done, update the status row in
> `plans/README.md`.
>
> **Drift check (run first)**:
> `git diff --stat 408be17..HEAD -- crates/parallax-storage crates/parallax-api ui/src/routes/traces.index.tsx ui/src/components`
> On mismatch with the excerpts below, STOP.

## Status

- **Priority**: P2
- **Effort**: M
- **Risk**: LOW-MED (bounded GROUP BY discipline required)
- **Depends on**: none (playground plan 054's structured-log spike scenario
  enriches the demo; advisor-plans/030's attributeCompare is the sibling
  surface — see Maintenance)
- **Category**: direction
- **Planned at**: commit `408be17`, 2026-07-07

## Why this matters

The Kibana-replacement pillar the research brief ranks highest after search
is **immediate field understanding**: for the data in view, which attribute
keys exist, their coverage, top values, cardinality — with one-click
filter/exclude (brief section C). For spans and error events this is nearly
free in Parallax: span attributes are stored as flattened columnar
`span_attributes.<key>` / `resource_attributes.<key>` columns, so top-values
and coverage are plain bounded GROUP BYs, and key enumeration can reuse the
`information_schema` introspection pattern that `discover_metric_names`
already proves. Log attributes stay opaque JSON — that phase is explicitly
deferred, honestly.

## Current state

Verified at commit `408be17`.

- Span/resource attributes are flattened columns:
  `crates/parallax-storage/src/greptime.rs:315` (doc comment: the
  `span_attributes.*` / `resource_attributes.*` columns auto-widen);
  `reassemble_attrs` (`greptime.rs:452-473`) folds them back per row.
- Key enumeration precedent: `discover_metric_names`
  (`greptime.rs:1498-1532`) queries `information_schema.tables`; the same
  client/pattern against `information_schema.columns` for the spans table
  yields all attribute columns.
- Error events: attributes stored as queryable JSON on `error_events`
  (region `greptime.rs:1089` — `attributes` column; verify exact JSON
  functions used nearby, e.g. `json_get_string` in the log paths around
  `greptime.rs:367-385`).
- Logs (the deferred part): `log_attributes`/`resource_attributes` are
  opaque JSON columns read via `json_get_string(...)` per known key — keys
  are NOT enumerable without scans; the brief's `field_stats_minute`
  materialization is the eventual answer. DO NOT attempt log key discovery
  in this plan.
- Row-cap convention: `MAX_ROWS` in `crates/parallax-api/src/lib.rs:44`
  region — every new query must be window-bounded + LIMITed.
- UI surface: `ui/src/routes/traces.index.tsx` — the traces list with
  filters in URL search (service/errors params seen in its `<Link>`s and
  `validateSearch`); `ui/src/components/console/data-table.tsx` provides
  SearchInput/ToggleChip patterns; sheets/drawers exist via shadcn
  (`ui/src/components/ui/sheet.tsx` — used by logs doc viewer; confirm).
- High-cardinality guardrail (brief cross-cutting rule): explorer may SHOW
  high-cardinality keys but must label them as identifiers; group-by-style
  aggregation UIs must not offer `trace_id`/`run_id`/`session_id`/`user_id`.

## Commands you will need

| Purpose | Command | Expected |
|---------|---------|----------|
| Build | `rtk cargo build --workspace` | exit 0 |
| Lint | `rtk cargo clippy --workspace --all-targets` | zero warnings |
| Tests | `rtk cargo nextest run` | all pass |
| UI | (from `ui/`) `bun run typecheck && bun run lint && bun run test && bun run build` | exit 0 |

## Scope

**In scope**:
- `crates/parallax-storage/src/adapter.rs`, `greptime.rs`, `memory.rs`:
  `span_field_keys(from,to)` + `span_field_stats(key, from, to, filters)`
- `crates/parallax-api/src/lib.rs`: `fieldKeys` + `fieldStats` resolvers
  (entity: SPAN now; ERROR_EVENT if cheap — decide in Step 1)
- `ui/src/lib/api.ts`; `ui/src/components/console/field-explorer.tsx`
  (create); `ui/src/routes/traces.index.tsx` (drawer + filter integration)
- test files

**Out of scope**:
- Log-attribute field stats (JSON key discovery — deferred; record in
  README notes when updating the index).
- The `field_stats_minute` materialization.
- attributeCompare (advisor-plans/030) — different question (selected vs
  baseline); this is "what fields exist in my current view".
- Metric label discovery (plan 044).

## Git workflow

- `main`, Conventional Commits, `git commit -s`, one

## Steps

### Step 1: Verify the introspection + aggregate shapes live

Against a dev server with seeded/playground data, confirm via the `sql`
surface: (a) `information_schema.columns` lists `span_attributes.*` columns
for the spans table (record table name + filter needed); (b) a bounded
top-values query works:
`SELECT "span_attributes.http.request.method" AS v, COUNT(*) FROM <spans> WHERE ts_nanos BETWEEN x AND y GROUP BY 1 ORDER BY 2 DESC LIMIT 10`;
(c) coverage = `COUNT(v IS NOT NULL)` vs window row count;
(d) approx cardinality — check whether Greptime supports
`APPROX_DISTINCT`/`uddsketch`-style functions; if not, use
`COUNT(DISTINCT ...)` with the row cap and label it exact-up-to-cap.
Decide whether `error_events.attributes` JSON stats are cheap enough with
the available JSON functions — if not, scope this plan to spans only and say
so in the report.

**Verify**: record the four working SQL statements in the plan-execution
notes/commit message.

### Step 2: Storage methods

`span_field_keys(from, to)` → `[ { key, namespace } ]` from
`information_schema.columns` (strip the `span_attributes.` /
`resource_attributes.` prefixes; namespace = first dotted segment, e.g.
`http`, `db`, `parallax`, else `custom`; tag resource keys distinctly).
`span_field_stats(key, from, to, service?)` → coverage pct, distinct count
(capped), top 10 values with counts. Validate `key` against the discovered
column list before interpolation (allowlist — same discipline as plan 044).
Memory impl mirrors over in-memory span attrs.

**Verify**: `rtk cargo nextest run` → memory tests green; clippy clean.

### Step 3: Resolvers

`fieldKeys(fromNanos, toNanos): [FieldKey!]!` and
`fieldStats(key: String!, fromNanos, toNanos, service: String): FieldStats!`
following neighboring resolver conventions. Mark identifier-like keys
(`*.id`, `trace_id`-ish, or distinct-count ≈ row-count) with
`isIdentifier: true` in the payload so the UI can label them.

**Verify**: resolver tests green (`rtk cargo nextest run`).

### Step 4: FieldExplorer drawer on the traces list

Create `ui/src/components/console/field-explorer.tsx`:
- Opens from a "Fields" button in the traces-list header (Sheet/side drawer
  — reuse the primitive the logs doc viewer uses).
- Lists keys grouped by namespace with coverage bars; identifier-labeled
  keys get a muted "id" badge.
- Clicking a key loads `fieldStats`; each top value row has
  include/exclude actions that patch the traces-list URL search. Check what
  attribute filtering `traces`/`tracesPage` support today (read the traces
  route's loader + the resolver args in `parallax-api/src/lib.rs`): if
  arbitrary attribute filters are NOT supported by the list query, wire the
  value actions to the existing supported filters only (service etc.) and
  render a "copy as SQL" action for the rest (build the WHERE clause and
  link to `/sql` prefilled — check how `/sql` accepts a prefilled query via
  URL, `ui/src/routes/sql.tsx`); note the filter-arg gap in the report.
- Carry the active range (plan 038's helper).

**Verify**: (from `ui/`) `bun run typecheck && bun run lint && bun run test && bun run build`
→ exit 0. Manual with playground traffic: explorer lists `http.*`, `db.*`
(post-048), `parallax.*` namespaces; a value include updates the list; a
value exclude via SQL opens `/sql` prefilled. Record it.

## Test plan

- Storage memory tests: key enumeration (namespaces split correctly),
  stats (coverage/top values/caps), allowlist rejection of unknown keys.
- Resolver tests for both queries.
- UI: component test for FieldExplorer rendering from mocked data (model on
  `console/__tests__/kit.test.tsx`); route-level manual check recorded.

## Done criteria

- [ ] cargo build/clippy/nextest green with new tests
- [ ] `fieldKeys`/`fieldStats` bounded (LIMIT + window enforced; allowlist
      validation on `key`)
- [ ] Explorer drawer on `/traces` lists keys with coverage + top values;
      include/exclude actions work for supported filters; identifiers
      labeled
- [ ] UI gates exit 0; manual check recorded
- [ ] `plans/README.md` status row updated (note the logs-phase deferral)

## STOP conditions

- `information_schema.columns` doesn't expose the attribute columns (engine
  drift) — report the actual introspection path.
- Every top-values query on lab data exceeds ~1s — report timings; the
  materialization becomes the prerequisite, don't ship a slow drawer.
- The traces list supports no server-side filters beyond service/errors and
  the "copy as SQL" fallback feels like the whole feature — report; the
  attribute-filter arg on `tracesPage` becomes a prerequisite plan.

## Maintenance notes

- Phase 2 (logs) needs JSON key discovery or `field_stats_minute` — design
  note when attempted; the explorer component is built to take an `entity`
  prop from day one.
- advisor-plans/030 (attributeCompare) should reuse `FieldExplorer`'s
  key-listing + the same allowlist plumbing.
- Playground plan 054 adds the A9b structured-log/field-spike scenario —
  the demo where one field visibly dominates.
- Reviewer: SQL identifier allowlisting; caps on every aggregate; the
  drawer must virtualize or cap its key list if columns exceed ~200.
