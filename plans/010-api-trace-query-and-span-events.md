# Plan 010: API — trace list sort/total/duration-band, span events, dead storage objects

> **Executor instructions**: Step by step; verify each; STOP conditions
> binding; update `plans/README.md` when done. Pure Rust plan — no UI changes.
>
> **Drift check (run first)**: `git diff --stat ad9115d..HEAD -- crates/parallax-api crates/parallax-storage`
> Re-locate cited symbols by name if lines moved; shape changes = STOP.

## Status

- **Priority**: P1
- **Effort**: M
- **Risk**: MED
- **Depends on**: none (independent of 009; both are prerequisites for UI plans 011/012)
- **Category**: direction (API extension)
- **Planned at**: commit `ad9115d`, 2026-07-03

## Why this matters

The redesigned Traces list (plan 011) needs server-side sort (slowest-first is the core
triage interaction), a result total, and a max-duration bound; today `traces` supports only
newest-first with a limit. The redesigned waterfall (plan 012) needs **span events**
(exceptions, messages) per span; today the `Span` GraphQL type has no `events` field and the
storage row never projects them, so exceptions are only visible by leaving the trace for the
Issues page. This plan also removes dead storage objects that would silently mislead future
work.

## Current state

- `crates/parallax-api/src/lib.rs`:
  - `traces` resolver `:1128-1163` — args service/fromNanos/toNanos/minDurationMs/errorOnly/
    query/limit; builds `parallax_storage::adapter::TraceQuery` (`:1146-1156`); returns
    `Vec<TraceSummary>` (no total).
  - `Span` object `:215-266` — fields tsNanos, service, traceId, spanId, parentSpanId, name,
    kind, statusCode, statusMessage, durationNs, runId, links (JSON string), scopeName,
    attributes (JSON), resource (JSON). **No events field.**
  - `IssueList { items, total }` (`:140`) — the pagination shape to mirror.
- `crates/parallax-storage/src/adapter.rs`:
  - `TraceQuery` `:49-59` — `{service, from_nanos, to_nanos, min_duration_ns, error_only,
    name_contains, limit}`; doc comment `:41-48` explains representative-span semantics.
  - `TraceSummary` `:29-39`.
- `crates/parallax-storage/src/greptime.rs`:
  - `traces_search` orders `ORDER BY ts_nanos DESC` (search the fn by name; agent-audited at
    ~`:1004`).
  - `select_spans` projects `span_links` but no span-events column (~`:355`); `SpanRow` in
    `crates/parallax-storage/src/model.rs:7` has no events field.
  - Dead objects (verified by audit): extension table `rollups_fingerprint_minute` created in
    bootstrap (~`:91`) but never written/read (real rollups live in Turso `issue_buckets`,
    `metadata.rs:198`); post-create `fingerprint` column added to `opentelemetry_traces`
    (~`:142`) never populated; Turso `settings` table (`metadata.rs` SCHEMA `:9` area)
    unused.
- Exceptions today: derived `error_events` rows (`crates/parallax-storage/src/derive.rs:35`)
  — trace-correlated via `error_events_by_traces`.

## Commands you will need

Repo root:

| Purpose | Command | Expected |
|---------|---------|----------|
| Format  | `rtk cargo fmt --all` | clean |
| Lint    | `rtk cargo clippy --workspace --all-targets` | zero warnings |
| Tests   | `rtk cargo nextest run` | all pass |
| Build   | `rtk cargo build --workspace` | exit 0 |
| Live engine (Step 2 column check) | `rtk cargo run -p parallax-cli -- serve` | ready banner |

## Scope

**In scope**:
- `crates/parallax-storage/src/{adapter.rs, greptime.rs, memory.rs, model.rs}`
- `crates/parallax-api/src/lib.rs`
- `crates/parallax-storage/src/metadata.rs` (dead `settings` table removal only)
- Tests in those crates

**Out of scope**:
- UI files; CLI; ingest hot path EXCEPT the narrow option in Step 5 (explicitly bounded).
- `bundle`, issues, runs, dashboards resolvers.

## Git workflow

`main`; Conventional Commits (`feat(api): trace list paging + span events`);
`git commit -s`; trailer `Co-authored-by: Claude <noreply@anthropic.com>`.

## Steps

### Step 1: Extend `TraceQuery` + list shape

`adapter.rs`: add to `TraceQuery`: `max_duration_ns: Option<u128>`, `offset: usize`,
`sort: TraceSort` where `enum TraceSort { StartDesc (default), DurationDesc, DurationAsc,
SpanCountDesc }`. Add `pub struct TraceList { pub items: Vec<TraceSummary>, pub total: u64 }`
and change `traces_search` to return it (update both stores + all callers — grep
`traces_search(` across the workspace; expect greptime, memory, api, possibly CLI).

**Verify**: workspace builds after Steps 1-3 together.

### Step 2: GreptimeDB `traces_search` update

Thread max-duration/offset/sort into the SQL: sort maps to `ORDER BY duration_ns
DESC/ASC` / `span_count DESC` / `ts_nanos DESC`; total = a companion `COUNT(*)` over the same
WHERE (cap the counted scan the same way `IssueList.total` caps — inspect its impl and mirror
the "exact up to N" note in the doc comment). Keep the representative-span semantics
documented at `adapter.rs:41-48` — sorting by duration sorts by the **trace's** duration as
already computed for `TraceSummary`.

**Verify**: `rtk cargo nextest run -p parallax-storage` (with Step 6 tests) → pass.

### Step 3: Span events — column discovery, projection, GraphQL

1. **Discover the column** (live engine): `parallax serve` + raw SQL
   `SELECT column_name FROM information_schema.columns WHERE table_name =
   'opentelemetry_traces'`. Look for the span-events column produced by the
   `greptime_trace_v1` pipeline (expected name `span_events`, JSON). 
   - If present: add `events: Option<String>` (raw JSON) to `SpanRow` (`model.rs:7`),
     project it in `select_spans` (greptime.rs ~`:355`), memory store passes through.
   - If absent: **fallback** — resolve events by correlating `error_events` rows for the
     trace (existing `error_events_by_traces`) into a synthesized
     `[{name:"exception", tsNanos, attributes:{message, type, stacktrace}}]` JSON per span
     (match on span_id). Note which path you took in the report; the GraphQL shape is
     identical either way.
2. `parallax-api/src/lib.rs`: add `events: String` field to `Span` (`:215-266`) — JSON
   string, same convention as `links`/`attributes` (`"[]"` default). Doc-comment the shape.

**Verify**: resolver test — a trace whose span carries an exception event returns non-empty
`events` JSON (memory store path); build + clippy clean.

### Step 4: Remove dead storage objects

- Delete `rollups_fingerprint_minute` from the GreptimeDB bootstrap (greptime.rs ~`:91`) and
  from the RESERVED list in `discover_metric_names` (greptime.rs `:1261-1269`) — it was never
  written or read (trend rollups live in Turso `issue_buckets`).
- Delete the unused `settings` table from the Turso SCHEMA (`metadata.rs`) **only if** grep
  confirms no runtime reads/writes (audit found one regression test — update/remove that test
  accordingly).
- Leave `opentelemetry_traces.fingerprint` ADD COLUMN in place but add a `// TODO(plan-010):
  unpopulated;` comment — populating it touches the ingest hot path (zero-copy rule) and is
  explicitly deferred; removing it risks breaking existing tables. Document in report.

**Verify**: `grep -rn "rollups_fingerprint_minute" crates/` → only in migrations/history if
any (expect zero hits); `rtk cargo nextest run` → pass. Fresh-bootstrap smoke: run serve
against a clean data dir → ready banner, no bootstrap errors.

### Step 5: (Bounded option) skip — do NOT populate fingerprint on ingest

Explicit non-goal: the zero-copy ingest rule (`AGENTS.md`) makes fingerprint population a
separate decision. STOP if you find yourself editing `worker.rs`/ingest.

### Step 6: API surface for the list

`traces` resolver: add args `maxDurationMs: Option<f64>`, `offset: Option<i32>`,
`sort: Option<TraceSort>` (GraphQL enum `START_DESC DURATION_DESC DURATION_ASC
SPAN_COUNT_DESC`); return new type `TraceList! { items: [TraceSummary!]!, total: String! }`.
**Compatibility**: the UI currently reads the bare list (`ui/src/routes/traces.index.tsx`);
changing the return type is a breaking schema change — acceptable here because plan 011
updates the only consumer, but sequence it: land this plan, then 011 immediately after.
Alternative (choose if you want zero breakage): keep `traces` as-is and add `tracesPage(...)
:TraceList!`; record the choice in the report and in plans/README so 011 binds to the right
name.

**Verify**: build + clippy + nextest all green.

## Test plan

`parallax-storage` (memory store): sort orders (duration desc/asc, span-count), offset
windowing, max-duration band, total correctness under filters. `parallax-api`: resolver test
for `traces(sort: DURATION_DESC, limit: 2, offset: 1)` shape + `Span.events` presence.
Follow existing test module layout in each crate.

## Done criteria

- [ ] build / clippy(0 warnings) / fmt / nextest all green
- [ ] `traces` (or `tracesPage`) exposes sort + offset + maxDurationMs + total (String)
- [ ] `Span.events` returns JSON (native column or documented fallback)
- [ ] `rollups_fingerprint_minute` gone from bootstrap; `settings` table removed or
      justified in report
- [ ] Fresh-bootstrap smoke passes (serve ready banner, no errors)
- [ ] `plans/README.md` row updated (record which schema-compat choice was made in Step 6)

## STOP conditions

- `information_schema` shows neither a span-events column nor a workable correlation path.
- Changing `traces_search`'s signature breaks a caller outside the expected set
  (greptime/memory/api/CLI) — list them and stop.
- Any edit would land in the ingest hot path (`worker.rs`, OTLP forward) — hard stop
  (zero-copy rule).
- GreptimeDB errors on `ORDER BY` over the aggregated duration in the existing
  representative-span query shape — report the query plan problem, don't restructure the
  whole query silently.

## Maintenance notes

- Plan 011 binds to the Step 6 naming choice — update `plans/README.md` dependency note with
  the chosen query name immediately.
- If span events prove present natively, a later plan may deprecate the derived
  `error_events` duplication for in-trace display; don't do it here.
- The deferred `fingerprint` population is the enabling condition for span↔issue SQL joins;
  revisit when the ingest path gets its next planned change.
