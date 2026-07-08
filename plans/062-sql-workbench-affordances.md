# Plan 062: SQL workbench — server-side row cap with truncation flag, linkified result cells, named snippets

> **Executor instructions**: Follow this plan step by step. Run every
> verification command. On any STOP condition, stop and report. When done,
> update the status row in `plans/README.md`.
>
> **Drift check (run first)**:
> `git diff --stat ed5b10f..HEAD -- crates/parallax-api/src/lib.rs crates/parallax-storage/src/greptime.rs crates/parallax-storage/src/adapter.rs ui/src/routes/sql.tsx crates/parallax-storage/src/metadata.rs`
> On mismatch with the excerpts below, STOP. Advisor-plans/022 (SQL surface
> hardening) touches the same resolver — reconcile first (see Coordination).

## Status

- **Priority**: P1 (the row-cap half is a real resource hazard)
- **Effort**: M
- **Risk**: LOW-MED (cap changes observable behavior for big SELECTs — by design)
- **Depends on**: none. **Coordination**: advisor-plans/022 edits the same
  `sql` resolver (EXPLAIN ANALYZE rejection) and `greptime.rs` (escape_ident)
  — whichever lands second rebases; no logical conflict.
- **Category**: perf + direction
- **Planned at**: commit `ed5b10f`, 2026-07-07

## Why this matters

The `sql` resolver forwards any allowed statement to GreptimeDB and
materializes **every** returned row in server memory, then ships them through
GraphQL as JSON strings — `SELECT * FROM opentelemetry_traces` over a wide
window is an OOM/latency hazard wired to a text box. The typed resolvers all
cap at `MAX_ROWS = 500`; the raw path uniquely doesn't. Meanwhile the SQL
page's results are dead text: `trace_id`/`run_id` columns — which the page's
own examples SELECT — aren't clickable, and power users get localStorage
history but no named snippets. This plan makes the raw path bounded and
honest, and makes SQL results a pivot surface instead of a copy-paste source.

## Current state

Verified at commit `ed5b10f`.

- `crates/parallax-api/src/lib.rs:1207-1229` — the resolver: prefix
  allowlist (`select|with|show|describe|desc|explain|tql`), single-statement
  check, then

  ```rust
  let result = context.store.raw_sql(trimmed.trim_end_matches(';')).await...
  ```

  No LIMIT injection, no row cap. `MAX_ROWS = 500` (`lib.rs:44`) applies
  only to typed resolvers via `clamp_limit`.

- `crates/parallax-storage/src/greptime.rs:266-298` — `sql_with_schema`
  POSTs verbatim and collects all rows into `Vec<Vec<Value>>`.

- `crates/parallax-api/src/lib.rs:329-347` — `SqlResultOut` exposes
  `columns`, `rows` (JSON-array strings), `rowCount`. No truncation signal.

- `ui/src/routes/sql.tsx` — results table renders raw truncated text cells
  (`:377-388`); `EXAMPLES` hardcoded (`:42-94`, several select `trace_id` /
  `"parallax.run.id"`); history is anonymous localStorage
  (`HISTORY_KEY = "parallax.sql.history"`, `:96-107`). The logs doc viewer
  already linkifies these ids (`ui/src/components/logs-table.tsx:296-311`)
  — the pattern to mirror.

- Named-store precedent: dashboards CRUD in `metadata.rs` + resolvers
  (`dashboardSave`/`dashboardDelete`, `lib.rs:1872-1909`). Plan 057 clones
  it for saved views with a `page` column — **if 057 lands first, reuse its
  `saved_views` table with `page: "/sql"` instead of adding a new table**
  (the state string = the SQL text). If 057 hasn't landed, implement
  snippets on localStorage only and leave the server store to 057 (do NOT
  create a competing table).

## Commands you will need

| Purpose | Command | Expected |
|---------|---------|----------|
| Rust | `rtk cargo build --workspace && rtk cargo clippy --workspace --all-targets && rtk cargo nextest run` | clean |
| UI (from `ui/`) | `bun run typecheck && bun run lint && bun run test && bun run build` | all exit 0 |

## Scope

**In scope**:
- `crates/parallax-api/src/lib.rs` — cap + `truncated` field on
  `SqlResultOut`
- `ui/src/routes/sql.tsx` — truncation notice, cell linkification, snippets
  UI
- `crates/parallax-storage/*` — only if the cap is implemented at the
  `raw_sql` layer (see Step 1 decision)
- Tests

**Out of scope** (do NOT touch):
- The read-only allowlist / EXPLAIN ANALYZE / identifier escaping —
  advisor-plans/022.
- A query builder over spans/logs/metrics — future direction (deferred; the
  field explorer's "copy as SQL" from plan 046 is the near-term bridge).
- Schema sidebar behavior (`sql.tsx:125-152`) — unchanged.
- The `EXAMPLES` array content — unchanged (snippets complement it).

## Git workflow

- `main`, Conventional Commits, `git commit -s`, one

## Steps

### Step 1: Server-side cap + `truncated`

Decision (make by reading, not preference): the cap belongs in the resolver
(`lib.rs`) because `raw_sql` is a thin adapter and the in-memory impl just
errors (`adapter.rs:271-274`). Implement:

1. Constant `SQL_MAX_ROWS: usize = 2000` next to `MAX_ROWS` (raw SQL is the
   power surface; 4× the typed cap, still bounded — doc-comment why).
2. In the `sql` resolver, after `raw_sql` returns: if `rows.len() >
   SQL_MAX_ROWS`, truncate to `SQL_MAX_ROWS` and set `truncated: true`.
   (Truncating post-fetch still materializes once inside the storage crate —
   acceptable V1; the honest fix for fetch-time bounding is LIMIT injection,
   which risks breaking user queries with their own LIMIT/OFFSET semantics.
   Record this as the deferred root fix in Maintenance — deliberate,
   named.)
3. `SqlResultOut` gains `fn truncated(&self) -> bool`; keep `rowCount` as
   the RETURNED row count (post-truncation) so UI math stays consistent.

**Verify**: `rtk cargo nextest run -p parallax-api` — test: in-memory store
errors on raw_sql (existing behavior untouched); a GreptimeDB-mocked or
integration-marked test asserting truncation is fine to implement as a pure
unit test on the truncation helper if the resolver body is refactored to
`fn cap_sql_result(result, max) -> (result, truncated)` — do that refactor
so it's testable without a live engine.

### Step 2: UI truncation notice

`sql.tsx`: select `truncated` in the query; when true, render an amber line
under the result header: "Result capped at N rows — refine the query or add
LIMIT/ORDER BY." (inline pattern, no toast).

**Verify**: `bun run test` — render test with a truncated fixture.

### Step 3: Linkified result cells

In the results table (`sql.tsx:377-388`): wrap cell rendering in a
`linkForCell(column, value)` helper:
- column name (case-insensitive, after stripping quotes) `trace_id` →
  `<Link to="/traces/$traceId">`;
- `run_id` or `parallax.run.id` → `/runs/$runId`;
- `span_id` → link to the trace **only when** the row also has a trace_id
  column (link to `/traces/$traceId` with `?span=<id>` if the trace route
  supports a selected-span search param — check `traces.$traceId.tsx`'s
  search schema and use its real param name; if none exists, trace link
  only);
- `fingerprint` → `/issues/$fingerprint`;
- `service`/`service_name` → `/services/$service`.
Non-matching columns render as today. Mirror the styling of the logs doc
viewer links (`logs-table.tsx:296-311`).

**Verify**: `bun run test` — table test: a fixture result with `trace_id`,
`parallax.run.id`, `service_name` columns renders links with correct hrefs;
a value of `""`/`null` stays plain text.

### Step 4: Named snippets

- If plan 057's `saved_views` store exists: a "Snippets" dropdown next to
  the Examples — `savedViews(page: "/sql")`, save-current (name prompt →
  `savedViewSave(page: "/sql", state: <sql text>)`), select → editor set,
  delete per row.
- Else: same UI on localStorage key `parallax.sql.snippets`
  (`Array<{name, sql}>`, cap 50), structured for a later trivial migration
  to the server store. State the chosen path in the commit message.

**Verify**: `bun run typecheck && bun run lint && bun run test` clean;
manual: save, reload, reselect (record).

## Test plan

- Rust: `cap_sql_result` unit tests (under/at/over the cap).
- UI: truncation notice, linkification matrix, snippets save/select — model
  on the existing sql.tsx tests if present, else the nearest route test
  harness.

## Done criteria

- [ ] Rust + UI gates all clean
- [ ] `sql` never returns more than `SQL_MAX_ROWS` rows; `truncated`
      exposed and rendered
- [ ] trace/run/issue/service id cells in SQL results are links (tests)
- [ ] Snippets save/restore works (server- or local-backed, stated)
- [ ] `plans/README.md` status row updated

## STOP conditions

- Advisor-plans/022 landed and moved/renamed the `sql` resolver guard —
  rebase onto its shape first; on structural conflict, STOP.
- Product code turns out to depend on >2000-row `sql` results somewhere
  (grep `sql(` usages in `ui/src` + CLI crates first) — report before
  capping.
- Plan 057 in progress simultaneously — coordinate the store; never create
  two saved-state tables.

## Maintenance notes

- **Deferred root fix (named)**: fetch-time bounding (LIMIT injection or a
  streaming/row-limit parameter on GreptimeDB's HTTP API) — the post-fetch
  truncation still buffers once server-side; revisit if real deployments
  hit memory pressure. Also the audit-noted parameterized-query layer
  (advisor-plans/022's considered item) remains the bug-class fix.
- The `linkForCell` heuristic is column-NAME based by design — reviewers
  should reject value-shape sniffing (16-hex could be a span or a hash).
- Field explorer (plan 046) prefills `/sql` — snippets + prefill compose;
  after both land check the prefill doesn't clobber an unsaved editor
  buffer without confirmation.
