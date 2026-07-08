# Plan 041: Releases lane — `deploys` query, issue↔release linkage design, regression badge (design-anchored)

> **Executor instructions**: Design-anchored plan: Step 1 is a spike whose
> output gates the rest. Follow step by step; run every verification. On any
> STOP condition, stop and report. When done, update the status row in
> `plans/README.md`.
>
> **Drift check (run first)**:
> `git diff --stat 408be17..HEAD -- crates/parallax-storage crates/parallax-api crates/parallax-core ui/src`
> On mismatch with the excerpts below, STOP.

## Status

- **Priority**: P2
- **Effort**: L
- **Risk**: MED (the linkage half touches the ingest/derivation path)
- **Depends on**: none API-side; the demo needs plan 042 (playground emits
  real release attrs) to show anything
- **Category**: direction
- **Planned at**: commit `408be17`, 2026-07-07

## Why this matters

"Which deploy introduced this issue?" is a headline capability of the
research brief's Sentry-replacement lane (releases, regression lifecycle —
`docs/research/architecture/full-observability-ui-and-playground-research.md`,
"Issues: Sentry-grade grouping" section). Half of it is cheap: spans already
persist `service.version` and `deployment.environment.name` as auto-widened
columnar resource attributes, so a per-service release timeline is one
resolver. The other half is honestly gated: error events persist **only span
attributes, not resource attributes**, so issues cannot name their release
without either a back-join through `trace_id` or persisting the version onto
the error path. This plan ships the cheap half and produces a decided design
(not code improvisation) for the linkage half.

## Current state

Verified at commit `408be17`.

- Resource attrs are columnar and queryable:
  `crates/parallax-storage/src/greptime.rs:315` documents the auto-widening
  `resource_attributes.*` columns; `reassemble_attrs` (`greptime.rs:452-473`)
  folds `resource_attributes.<k>` columns back into the row's resource map;
  `select_spans` already reads `resource_attributes.parallax.run.id`
  directly (`greptime.rs:352`). So
  `SELECT DISTINCT "resource_attributes.service.version" ... GROUP BY service_name`
  over the spans table is feasible today (columns exist only once some
  service emitted the attr — the storage layer already tolerates missing
  tables/columns, see the not-yet-created tolerance around
  `greptime.rs:206-262`).

- The error path drops resource attrs:
  `crates/parallax-core/src/derive.rs:71` — error events get
  `attributes: attributes_to_json(&span.attributes)` (span attrs only);
  `ErrorEventRow` (`crates/parallax-storage/src/model.rs:79`) has no
  resource/version field; `Issue` (`model.rs:94`) carries no
  version/environment.

- Turso metadata has no deploy/release table — schema at
  `crates/parallax-storage/src/metadata.rs:10-44` defines `issues`, `runs`,
  `dashboards`, `issue_buckets` only. The proven CRUD pattern to copy is
  dashboards: `metadata.rs:519` (`dashboard_save`), `:552` (`dashboards`).

- GraphQL surface: resolvers live in `crates/parallax-api/src/lib.rs`
  (single file; e.g. `services` at `:1610`). UI issue detail:
  `ui/src/routes/issues.$fingerprint.tsx` (header badges at `:253-264`).

- Zero-copy rule (repo `AGENTS.md`): "ingest is zero-copy by design: decode
  once, move ownership forward, never clone telemetry on the hot path." Any
  version-persistence design must respect this — precedent: the traces
  `fingerprint` column was left unpopulated for the same reason (comment at
  `greptime.rs:136-137` region).

## Commands you will need

| Purpose | Command (repo root) | Expected |
|---------|---------------------|----------|
| Build | `rtk cargo build --workspace` | exit 0 |
| Lint | `rtk cargo clippy --workspace --all-targets` | zero warnings |
| Tests | `rtk cargo nextest run` | all pass |
| UI gates | (from `ui/`) `bun run typecheck && bun run lint && bun run test && bun run build` | all exit 0 |

## Scope

**In scope**:
- `crates/parallax-storage/src/adapter.rs` (+`greptime.rs`, `memory.rs`) —
  one new read method for release windows
- `crates/parallax-api/src/lib.rs` — `deploys`/`releases` query resolver
- `ui/src/lib/api.ts`, `ui/src/routes/services.$service.tsx` (release strip),
  `ui/src/routes/issues.$fingerprint.tsx` (regression badge, only if the
  spike's cheap path allows)
- `docs/research/architecture/` — the Step 1 design note
- test files

**Out of scope**:
- Deploy **webhook ingest** (`POST /v1/deploys`) — future; the spike notes
  it.
- Changing the ingest hot path in THIS plan — the spike may propose it; the
  implementation of version-on-error-events is a follow-up plan gated on
  operator approval of the design note.
- Playground changes (plan 042).
- Suspect-commit / VCS attribution.

## Git workflow

- `main`, Conventional Commits, `git commit -s`, one

## Steps

### Step 1 (SPIKE): design note `docs/research/architecture/releases-lane-design.md`

Answer with evidence (read the cited code first):
1. **Release timeline query.** Confirm with a live or fixture query that
   `SELECT service_name, "resource_attributes.service.version" AS version, MIN(ts_nanos), MAX(ts_nanos), COUNT(*) FROM <spans table> WHERE version IS NOT NULL GROUP BY 1,2`
   works on the Greptime schema (use the `sql` GraphQL surface or the seed
   example `crates/parallax-server/examples/seed.rs` + a dev server). Record
   the exact table/column quoting needed.
2. **Issue↔release linkage options**, cost each:
   a. read-time back-join: issue → `last_trace_id` → `spans_by_trace`
      (`greptime.rs:669`) → first span's resource version (cheap, per-issue,
      already the pattern the issue-detail loader uses for `traceRunId` —
      `ui/src/routes/issues.$fingerprint.tsx:110-130`);
   b. persist `service_version` on `ErrorEventRow` at derivation
      (`derive.rs:71` site) — evaluate against the zero-copy rule (the
      resource attrs are already decoded at that point; adding one more
      lookup may be free — verify by reading `derive_from_traces`'s inputs);
   c. a `release_first_seen` Turso rollup.
   Recommend one; the recommendation is the deliverable, not the code.
3. **Regression semantics.** Define "regressed": issue resolved at version X,
   events reappear at version Y > X — what's computable under option (a)
   only?
4. **UI shape.** Release strip on service detail (version segments over the
   time axis) + a `release` badge on issue detail.

STOP after Step 1 if option (b) is recommended — get operator sign-off
before touching derivation (post the design note summary in your report).

**Verify**: design note committed; contains the four sections + a chosen
option with rationale.

### Step 2: `releases(service, from, to)` resolver (the cheap half)

Add adapter method `release_windows(service, from, to)` returning
`{ version, first_seen_nanos, last_seen_nanos, span_count }` rows (Greptime
impl per the spike's verified SQL; memory impl over its in-memory spans).
Expose as GraphQL `releases(service: String!, fromNanos: String!, toNanos: String!)`.
Follow the existing resolver style in `parallax-api/src/lib.rs` (e.g. the
`services` resolver at `:1610` and `serviceRed` nearby — same
context/adapter call pattern, same `FieldResult` error mapping).

**Verify**: `rtk cargo build --workspace && rtk cargo nextest run` → new
storage + resolver tests pass (write them per Test plan).

### Step 3: UI — release strip on service detail

In `ui/src/routes/services.$service.tsx`: fetch `releases` in the loader
(add to the existing loader GraphQL document); render a slim horizontal
strip under the header: one segment per version across the window, labeled
with the version, tooltip = first/last seen + span count. Empty → render
nothing (graceful absence like MetricStrip). Style with existing tokens
(`Badge`, muted borders) — no new design primitives.

**Verify**: (from `ui/`) `bun run typecheck && bun run lint && bun run build`
→ exit 0.

### Step 4: Issue regression badge (only under spike option a)

If the spike chose (a): in the issue-detail loader (which already fetches
the correlated trace — `issues.$fingerprint.tsx:110-130`), also read the
trace's `service.version` from span `resource` JSON (the field is already
fetched there: `spans { resource ... }` — parse it like the existing
resource parse in that file) and render a `release <version>` badge in the
header badge row (`:253-264`). If the spike chose (b) or (c): skip this
step, note it in the report.

**Verify**: `bun run typecheck && bun run test` → exit 0.

## Test plan

- Storage: unit test for `release_windows` on the memory adapter (three
  spans, two versions → two windows with correct min/max/counts); Greptime
  impl covered by the SQL-shape test pattern used by neighboring methods
  (find an existing greptime query test to model — if greptime tests are
  integration-only, the memory test + build gate is the bar; say so in the
  report).
- API: resolver test following an existing query-resolver test in
  `parallax-api` (grep `#[tokio::test]` in `crates/parallax-api/src/`).
- UI: no new harness; gates + manual check with seeded data
  (`crates/parallax-server/examples/seed.rs` emits `service.version` —
  verified at `seed.rs:22`).

## Done criteria

- [ ] Design note committed with a decided linkage option
- [ ] `releases` resolver + adapter methods + tests; `rtk cargo build`,
      clippy zero warnings, `rtk cargo nextest run` all green
- [ ] Service detail renders the release strip with seeded data (manual
      check recorded)
- [ ] Issue regression badge shipped OR explicitly deferred per spike
- [ ] UI gates all exit 0
- [ ] `plans/README.md` status row updated

## STOP conditions

- The `resource_attributes.service.version` column doesn't materialize on
  the spans table for seeded data (schema drift) — report the actual column
  layout.
- Spike recommends option (b) (ingest change) — STOP for operator sign-off
  before implementing anything beyond Steps 2-3.
- The `releases` GROUP BY needs a scan cap the existing `MAX_ROWS`
  convention can't express — report rather than shipping an unbounded scan
  (mind advisor-plans/022's SQL-hardening context).

## Maintenance notes

- Plan 042 makes the playground emit distinct `service.version` values +
  `deployment.*` attrs — the demo for this lane; coordinate.
- Deferred: deploy webhook ingest, `Issue.affectedReleases` list, suspect
  commits, `vcs.*` attribution — each builds on the spike's decision.
- Reviewer: quoting of dotted column names in SQL (`"resource_attributes.service.version"`)
  is the classic breakage; check both Greptime and memory impls agree on
  ordering (newest version last).
