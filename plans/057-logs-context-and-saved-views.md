# Plan 057: Logs surrounding-context view + named saved views (logsAround resolver, context drawer action, reusable saved-view store)

> **Executor instructions**: Follow this plan step by step. Run every
> verification command. On any STOP condition, stop and report. When done,
> update the status row in `plans/README.md`.
>
> **Drift check (run first)**:
> `git diff --stat ed5b10f..HEAD -- crates/parallax-storage/src/adapter.rs crates/parallax-storage/src/greptime.rs crates/parallax-storage/src/memory.rs crates/parallax-api/src/lib.rs ui/src/components/logs-table.tsx ui/src/routes/logs.tsx crates/parallax-storage/src/metadata.rs`
> On mismatch with the excerpts below, STOP.

## Status

- **Priority**: P2
- **Effort**: M
- **Risk**: LOW
- **Depends on**: plan 035 (inline-error UI conventions); reads get richer
  after plan 055 (event column) but do not require it
- **Category**: direction
- **Planned at**: commit `ed5b10f`, 2026-07-07

## Why this matters

Two of the most common log-triage moves have no product support: "show me
what happened around this line" and "get me back to the filter combination I
always use". Today the doc viewer shows only the single record
(`logs-table.tsx:236-323`), so context means manually re-querying a narrow
window; and the only persisted state anywhere is anonymous SQL history in
localStorage (`sql.tsx:96-107`), so every visit rebuilds
service+severity+query+columns by hand. This plan adds a `logsAround`
resolver (a thin composition of the existing `logs_search`), a "Context"
action in the log doc viewer, and a named saved-view store (Turso-backed,
cloned from the dashboards CRUD) that Logs uses first and other list pages
can adopt later.

## Current state

Verified at commit `ed5b10f`.

- `crates/parallax-storage/src/adapter.rs:227-234` — the existing search:

  ```rust
  async fn logs_search(
      &self,
      service: Option<&str>,
      range: RangeInclusive<u128>,
      severity_min: Option<i32>,
      body_contains: Option<&str>,
      limit: usize,
  ) -> anyhow::Result<Vec<LogRow>>;
  ```

  GreptimeDB impl at `greptime.rs:1319`; in-memory impl in `memory.rs`.

- `crates/parallax-api/src/lib.rs:1138-1202` — the `logs` resolver (every
  filter optional, newest first, `MAX_ROWS`-capped via `clamp_limit`,
  `lib.rs:44-50`). No anchor-around semantics.

- `ui/src/components/logs-table.tsx:236-323` — the doc viewer `Sheet`:
  field rows via `docFields` (`:77-110`), linkified `trace_id`/`run_id`
  (`:296-311`). No context action.

- `ui/src/routes/logs.tsx` — URL-state filters: `service`, `sev`, `q`,
  `cols`, range params (`loadLogs` at `:169-201`); column state parse/
  serialize in `logs-table.tsx:46-61`.

- Saved-state precedent: dashboards are Turso rows with full CRUD —
  `model.rs:180-187` (`Dashboard { id, name, layout, created_at_nanos,
  updated_at_nanos }`), metadata store CRUD in
  `crates/parallax-storage/src/metadata.rs`, resolvers `dashboards`,
  `dashboard`, `dashboardSave`, `dashboardDelete`
  (`lib.rs:1273-1281,1706-1709,1872-1909`). Plan 052 clones the same shape
  for investigations — this plan clones it for saved views and must keep the
  three stores structurally parallel (same CRUD naming, same JSON-payload
  column) so a later consolidation is mechanical.

- Repo conventions: UI errors render inline (plan 035 pattern), Bun-only,
  strict TS; GraphQL via `src/lib/api.ts` only.

## Commands you will need

| Purpose | Command | Expected |
|---------|---------|----------|
| Rust build/lint/test | `rtk cargo build --workspace && rtk cargo clippy --workspace --all-targets && rtk cargo nextest run` | clean |
| UI (from `ui/`) | `bun run typecheck && bun run lint && bun run test && bun run build` | all exit 0 |

## Scope

**In scope**:
- `crates/parallax-api/src/lib.rs` — `logsAround` resolver; `savedViews`/
  `savedViewSave`/`savedViewDelete` resolvers
- `crates/parallax-storage/src/metadata.rs` — `saved_views` Turso table +
  CRUD (clone the dashboards functions)
- `crates/parallax-storage/src/model.rs` — `SavedView` struct
- `ui/src/components/logs-table.tsx` — "Context" action in the doc viewer
- `ui/src/routes/logs.tsx` — context mode + saved-view picker/save UI
- Tests per layer

**Out of scope**:
- Pattern collapse/log clustering — future (needs its own design; noted in
  README deferred list).
- Applying saved views to Traces/Issues pages — the store is generic
  (`page` column) but only Logs consumes it here.
- `logsAround` for the story timeline — advisor-plans/029 can reuse it later.
- Any change to `logs_search` SQL itself.

## Git workflow

- `main`, Conventional Commits, `git commit -s`, one

## Steps

### Step 1: `logsAround` resolver

In `lib.rs` (Query impl, near `logs` at `:1142`):

```rust
/// Logs surrounding one anchor timestamp: `before` + `after` rows around
/// `anchorNanos`, optionally scoped to a service and/or trace. Rows come
/// back time-ascending with the anchor position marked by timestamp.
async fn logs_around(
    context: &ApiContext,
    anchor_nanos: String,
    window_seconds: Option<i32>,   // default 30, clamp 1..=600
    service: Option<String>,
    trace_id: Option<String>,
    limit: Option<i32>,            // default 200, MAX_ROWS-clamped
) -> FieldResult<Vec<LogRecord>>
```

Implementation: parse `anchor_nanos` (reuse the existing nanos parsing
helper — grep `from_nanos.parse` / `parse_nanos` in `lib.rs` and match it),
compute `anchor ± window`, and call the existing `logs_search` with that
range; when `trace_id` is set, call `logs_by_trace` and filter client-side
in the resolver to the window (no new adapter method — composition only).
Sort ascending before returning.

**Verify**: `rtk cargo nextest run -p parallax-api` — new test: seed the
in-memory store with logs at T-60s/T-10s/T/T+10s/T+60s, `windowSeconds: 30`
returns exactly the middle three, ascending.

### Step 2: `saved_views` metadata table + CRUD

Clone the dashboards pattern in `metadata.rs` (find `dashboards` DDL +
`dashboard_save`/`dashboard_delete`/`dashboards` functions and mirror them):

- Table `saved_views(id TEXT PK, name TEXT, page TEXT, state TEXT,
  created_at_nanos, updated_at_nanos)` — `state` is the URL search string
  (same capture convention as plan 052's pins: the page's current
  `location.search`), `page` is the route id (`"/logs"`).
- `model.rs`: `SavedView { id, name, page, state, created_at_nanos,
  updated_at_nanos }`.
- `lib.rs`: `savedViews(page: Option<String>)` query +
  `savedViewSave(id: Option<String>, name: String, page: String,
  state: String)` + `savedViewDelete(id: String)` mutations — mirror
  `dashboardSave`/`dashboardDelete` signatures/validation exactly
  (`lib.rs:1872-1909`), including any name-length/row-count caps they
  enforce; if they enforce none, cap saved views at 100 rows per page and
  name ≤ 120 chars.

**Verify**: `rtk cargo nextest run -p parallax-storage -p parallax-api` —
CRUD round-trip test modeled on the existing dashboards test (grep
`dashboard_save` in tests).

### Step 3: UI — Context action

In `logs-table.tsx`, doc viewer footer (after the linkified rows block at
`:296-311`): add a "Show context (±30s)" button. Clicking closes the sheet
and navigates `/logs` with `anchor=<tsNanos>` (+ current service filter
preserved). In `logs.tsx`:
- Accept `anchor` in the search schema; when present, `loadLogs` calls
  `logsAround(anchorNanos: …, windowSeconds: 30, service: …)` instead of
  `logs`, renders the same table, highlights the row(s) whose `tsNanos`
  equals the anchor (background accent), and shows a dismissible banner
  "Context around <time> — Reset" that clears `anchor`.
- The histogram stays on the surrounding window (reuse the existing
  custom-window plumbing — `onWindow` at `logs.tsx:534,554`).

**Verify** (from `ui/`): `bun run typecheck && bun run test` — component
test: anchor row gets the highlight class; banner reset clears the param.

### Step 4: UI — saved views on Logs

In `logs.tsx` toolbar (next to the column menu at `:586-628`):
- "Views" dropdown: lists `savedViews(page: "/logs")` (name, newest first),
  "Save current view…" (name prompt → `savedViewSave` with
  `state = location.search`), per-view delete.
- Selecting a view navigates to `/logs` + the stored search string
  (validate it parses through the route's search schema; on parse failure
  show the inline error pattern and do not navigate — a saved view from an
  older schema version must fail readable, not crash: strip unknown keys
  via the schema's validator if it supports it).

**Verify**: `bun run typecheck && bun run lint && bun run test` clean;
manual: save a view with service+severity+cols set, reload, reselect →
identical URL search.

## Test plan

- API: `logsAround` window/clamp/trace-scoped tests (in-memory store);
  saved-view CRUD + page filter + row-cap tests. Model on the existing
  dashboards resolver tests.
- UI: anchor-highlight + banner test; saved-view dropdown render + select
  test (mock `graphql`). Follow the existing logs-table test file pattern.

## Done criteria

- [ ] Rust build/clippy/nextest clean
- [ ] UI typecheck/lint/test/build clean
- [ ] `logsAround` returns windowed ascending rows (test proves clamps)
- [ ] `saved_views` CRUD works; views scoped by `page`
- [ ] `/logs?anchor=…` renders highlighted context with reset banner
- [ ] Views dropdown saves/restores/deletes named states
- [ ] `plans/README.md` status row updated

## STOP conditions

- The route search schema cannot round-trip an arbitrary saved search string
  (e.g. strict validation rejects unknown params with a crash rather than a
  strip) — report; do not weaken the schema globally to force it.
- `logs_search` cannot serve the anchor window without a new adapter method
  (composition turns out impossible) — report the exact gap first.
- Plan 052 landed first and introduced a generic pinned-state store that
  covers named views — reconcile with it instead of adding a parallel table
  (STOP and propose the merge).

## Maintenance notes

- Traces/Issues pages can adopt the same store later — the `page` column is
  the seam; keep view state = URL-search-string as the only contract.
- Plan 052 (investigations) captures URLs the same way; if both stores
  survive a year, consolidate.
- Reviewer: check the saved-state parse-failure path renders inline (035
  convention), and that `logsAround` clamps both window and limit.
- Deferred: pattern collapse (log clustering) — separate design; ±context by
  trace/run grouping beyond the flat window — revisit after story (029).
