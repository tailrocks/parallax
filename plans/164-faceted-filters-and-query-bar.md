# Plan 164: Faceted filter sidebars with counts, duration p50/p95 presets, and an autocompleting where-clause editor

> **Executor instructions**: Follow this plan step by step. Read `ui/AGENTS.md`
> (carries the browser-verification checklist — apply after every step against
> playground data). STOP conditions binding. Update this plan's status row in
> `plans/README.md` when done.
>
> **Drift check (run first)**:
> `git diff --stat <wave2-base>..HEAD -- 'ui/src/routes/traces.index.tsx' ui/src/routes/logs.tsx 'ui/src/routes/invocations.index.tsx' ui/src/components/console/data-table.tsx crates/parallax-api`
> `<wave2-base>` = the `main` commit closing Wave 1; plans 162/163 touching shared
> components is expected drift.

## Status

- **Priority**: P1
- **Effort**: L
- **Risk**: MED (touches the three main list surfaces + adds GraphQL fields)
- **Depends on**: plan 162 (DONE 2026-07-17 — ServiceDot/tokens in ui/AGENTS.md rules 19-23); plan 156 (invocation fields)
- **Category**: direction / UI + API / filtering
- **Planned at**: `2288011`, 2026-07-17

## Preliminary work landed (helper agent, 2026-07-17)

**Note (Grok, 2026-07-17):** A mistaken API bridge (`attribute_filters:
Vec::new()` on `TraceQuery` constructors in
`crates/parallax-api/src/resolvers/traces/queries.rs`) was pushed in
`e3a6d81` while the field existed only in concurrent unstaged adapter WIP —
that broke clean `main` (`cargo check -p parallax-api` E0560). **Reverted in
`43ef791`.** When landing Step 1, add the field on `TraceQuery` and the
GraphQL/API wiring in the **same** commit set; do not set the field on API
until storage ships it.

Committed to `main` for the primary executor to verify, deepen, and extend —
statuses intentionally untouched:

- `ui/src/lib/where-clause.ts` + `ui/src/lib/__tests__/where-clause.test.ts`
  (`d0e1005`): AND-only grammar parser (`ident op literal (AND …)*`, ops
  `= != > < >= <= CONTAINS NOT CONTAINS`), quoted strings with escapes and
  unicode, `WhereParseError` start/end offsets for editor squiggles,
  serialize/parse round-trip, `whereClauseFromSearch` URL codec. 26 tests.
- `ui/src/components/console/facet-sidebar.tsx`: controlled `FacetSidebar`/
  `FacetSection` (collapsible, count-annotated `tabular-nums`, inline search,
  `maxVisible` + "Show N more", ServiceDot swatches) plus pure helpers
  `facetSelectionsToParam`/`FromParam`/`toggleFacetValue` (`dim:value` CSV
  URL codec). Codec/toggle tests landed (`fc006e9`,
  `console/__tests__/facet-sidebar.test.tsx`); duration summary tests at
  `626dea0`; editor autocomplete/apply/error tests at `7da35f6`
  (`console/__tests__/where-clause-editor.test.tsx`).
- `ui/src/components/console/duration-filter.tsx`: preset chips
  (`> p50`/`> p95` from injected `DurationStats`, `> 1s`), debounced (400ms)
  min/max ms inputs, collapsed summary chip with inline clear.
- `ui/src/components/console/where-clause-editor.tsx`: monospace input, live
  parse + inline error with position, slot-aware autocomplete
  (keys/ops/values/AND; Ctrl+Space open, Tab/Enter accept, ⌘Enter apply),
  `WhereClauseChips` removable chip row.

- Backend `attributeFilters` core (`9751982`, lands the complete set the
  Grok note above prescribes — field + bridge in one green commit):
  `AttributeFilter`/`AttributeFilterOp` (+ `parse`/`as_token`, pure
  `matches()`) in `parallax-storage/src/adapter.rs`;
  `TraceQuery.attribute_filters` (semantics: a trace matches when ≥1
  in-window span satisfies ALL filters); GreptimeDB compiler
  `parallax-greptime/src/greptime/attribute_filters.rs` (intrinsics → raw
  columns, attribute keys → `span_attributes.*` gated by
  `span_field_key_allowed` else `1 = 0`, escaped string literals, LIKE
  wildcard escaping, numeric literals re-serialized from the parsed
  number; golden unit tests incl. the `x' OR 1=1--` proof);
  `traces_search` participation wiring via a shared
  `trace_participation` helper; in-memory adapter parity
  (`filter_matched_trace_ids`/`filter_observed_value`); API constructors
  pass `Vec::new()` until the GraphQL argument lands. Clippy clean; 103
  tests green across the four crates.

- Backend `traceDurationStats` (`d071943`): `DurationStats` type + trait
  method (duration bounds ignored so presets don't feed back), GreptimeDB
  impl over the traces_search representative set (shared
  `trace_scan_clauses` helper, `approx_percentile_cont`), exact
  nearest-rank in-memory impl, GraphQL `traceDurationStats(p50Ms/p95Ms)`.
  104 tests green; peer wires `duration-filter.tsx` to it + live-verify.

- GraphQL `attributeFilters` argument (`916efac`): `AttributeFilterInput
  {key, op, value}` (operator tokens parsed, invalid rejected with a field
  error) on `tracesPage` and `traceDurationStats`; e2e narrowing test
  against the memory adapter. The UI where-clause path is now open
  end-to-end for traces — peer sends the parsed clause from the route.

Still open (full plan scope): facet-count queries; logs/invocations filter
compilation (the compiler is span-table-only so far); live-engine narrowing
+ injection assertions; route wiring + URL schemas (Step 3); `F` keyboard
shortcut; syntax-highlight overlay (editor currently plain input);
browser evidence. The `f-attrs` playground scenario landed on the
playground's main at `2083a89` (100 spans + 100 logs,
`http.request.method` exactly 70/20/10 GET/POST/DELETE, unit-tested split;
run `scenarios/run.sh f-attrs`) — peer verifies live counts against it.
Preliminary components are controlled/presentational —
verify their contracts against the GraphQL shapes you build and refactor
freely; they carry no route or fetch coupling.

## Why this matters

Parallax filters today are dropdowns plus a free-text box — no visibility
into which values exist or how much data each hides, and no way to express a
compound condition without dropping to the SQL console. The reference
product (Maple) demonstrates the two-tier answer: **facet sidebars** (checkbox
groups with per-value counts, searchable, color-swatched for services) for
the common 90%, and a **where-clause editor** (`service = "checkout" AND
attr.http.route != "/health"`, autocomplete over live keys/values, ⌘Enter to
apply) for the long tail — both URL-encoded so every filtered view is a
permalink. Parallax's backend already has the raw ingredients
(`field_keys`, `field_stats`, `attribute_compare`, `sql`); this plan gives
them a product surface.

## Reference (self-contained)

From Maple (`apps/web/src/components/filters/filter-section.tsx`,
`traces/duration-range-filter.tsx`, `query-builder/where-clause-editor.tsx`;
clone `https://github.com/MapleTechLabs/maple` for detail):
- Facet section: collapsible group; each option = checkbox + optional
  color-dot/icon + label + right-aligned count (`tabular-nums`); `maxVisible`
  with "Show N more"; inline search filtering options; multi-select ORs
  within a facet, ANDs across facets.
- Duration filter: preset chips `> p50` / `> p95` / `> 1s` where p50/p95
  come from the current result window's duration stats; min/max ms inputs
  with ~400ms debounce; collapsed summary chip (`≥ 12ms`) with inline clear.
- Where-clause editor: monospace input with syntax highlight overlay;
  autocomplete listbox for keys (from live field discovery), operators
  (`= != > < >= <= CONTAINS NOT CONTAINS`), and values (top values for the
  chosen key); Ctrl+Space to open, Tab/Enter accept, ⌘Enter apply; active
  clause shown as a removable chip. Grammar: `ident op literal (AND …)*` —
  AND-only in v1, no OR/parens (STOP if tempted to grow grammar).

## Current state

(verified at `2288011`)

- `ui/src/components/console/data-table.tsx` — shared `Toolbar`,
  `SearchInput`, `FilterSelect` (single-select dropdowns), `SortableHead`.
- Traces list `ui/src/routes/traces.index.tsx` — service/errors-only/
  min-duration(ms input)/text filters in the toolbar; live toggle.
- Logs `ui/src/routes/logs.tsx` — service, severity_min, text, range,
  saved views.
- Invocations list (plan 157) — q/mode/status/outcome/range params.
- Backend: `crates/parallax-api/src/lib.rs` — `field_keys(from,to)` and
  `field_stats(key,from,to,service)` (coverage/cardinality/top values) exist
  (`:193-196`); `traces_page`/`logs` accept fixed filter args only; there is
  **no facet-count query and no where-clause argument**. `sql` exists but is
  the power-user escape hatch, not the filter path.
- GreptimeDB storage: span/resource attributes are native columns/JSON —
  key/value filters compile to SQL WHERE clauses in
  `crates/parallax-greptime` query builders.

## Contract decisions (fixed)

1. **Facets are a GraphQL field, not client-side aggregation**: new
   `traceFacets(from,to, baseFilters…)` / `logFacets(…)` /
   `invocationFacets(…)` returning `[{dimension, values: [{value, count}]}]`
   for bounded dimensions only (service, status/severity, app.mode, outcome,
   cli.command.name, http.method, error.type). Facet queries reuse a fixed
   coarse window cap so they stay cheap.
2. **Where-clause is structured, not raw SQL**: the UI parses the clause
   into a typed filter list `[{key, op, value}]` sent as a GraphQL argument
   (`attributeFilters`); the Rust side compiles it to parameterized SQL
   against known column/JSON paths. Raw strings never reach SQL — no
   injection path. Unknown keys are valid (JSON path lookup); values are
   always bound.
3. AND-only grammar in v1.
4. Everything URL-encoded (zod search schemas); permalinks reproduce the
   exact result set.

## Commands you will need

| Purpose | Command | Expected on success |
|---|---|---|
| API tests | `cargo nextest run --locked -p parallax-api -p parallax-greptime` | pass |
| Live engine | `cargo nextest run --locked -p parallax-server -E 'binary(/greptime/)'` | pass |
| UI gates | `cd ui && bun run typecheck && bun run lint && bun run check && bun run --bun test:ci && bun run build` | exit 0 |
| Corpus | playground `scenarios/run.sh --all-corner-cases` | loaded |

## Scope

**In scope:**
- `crates/parallax-greptime` — facet-count queries; `attributeFilters`
  compilation (shared helper used by traces/logs/invocations list queries);
  duration-stats (p50/p95 for current filter window) for traces.
- `crates/parallax-api` — `traceFacets`/`logFacets`/`invocationFacets`,
  `durationStats`, `attributeFilters` argument on `traces_page`/`logs`/
  `invocations`; tests.
- `ui/src` — new `components/console/facet-sidebar.tsx` (+ facet section
  component), `components/console/duration-filter.tsx`,
  `components/console/where-clause-editor.tsx` (+ `lib/where-clause.ts`
  parser, pure, tested); wiring into traces/logs/invocations routes; URL
  schemas; ServiceDot in service facets.
- Keyboard: `F` opens the where-clause editor on list pages (registered in
  the palette help).

**Out of scope:** OR/parens grammar, saved-view redesign (logs saved views
keep working, gaining the new params transparently), SQL console, dashboard
filtering, metrics page (plan 168).

## Git workflow

- Work directly on `main` in BOTH repositories — no branches, no pull requests (operator
  delivery model, 2026-07-17; see plans/README.md Execution Preflight).
- Commit OFTEN: one small green slice per commit (a step, a component, a
  fixed defect), Conventional Commits, DCO `-s`, exactly one agent trailer.
- **Push to `main` immediately after every commit** — never batch pushes,
  never hold local-only work; never push a slice whose targeted checks are
  red. The parallax ruleset's "Bypassed rule violations" push notice is
  expected.

## Steps

### Step 1: Backend facets + attributeFilters + durationStats

Storage queries + GraphQL fields + compilation helper (key → column/JSON
path; op → SQL; value bound). Live-engine tests: facet counts match seeded
corpus; an `attributeFilters=[{http.request.method,=,POST}]` narrows
results; injection attempt (`value = "x' OR 1=1--"`) returns zero rows and
parameterization is proven (no string concat in the SQL builder — assert
via the builder's unit test).

**Verify**: `cargo nextest run` lanes above pass.

### Step 2: Parser + editor component

`lib/where-clause.ts`: tokenize/parse/serialize round-trip; error positions
for the editor squiggle. Editor: highlight overlay, autocomplete (keys from
`field_keys`, values from `field_stats.topValues`, cached), chip display.

**Verify**: parser tests (valid/invalid/round-trip/quoting incl. spaces and
unicode); editor component tests (autocomplete open/accept/apply).

### Step 3: Facet sidebar + duration filter + route wiring

Wire all three list routes: sidebar layout (collapsible, count-annotated,
searchable service facet with dots), duration presets from `durationStats`,
URL schemas extended. Multi-select semantics: OR within facet, AND across,
AND with where-clause.

**Verify**: route tests for URL round-trip and filter composition; UI gates
green; browser walk per checklist on all three pages against the corpus —
counts visibly correct (cross-check one count against the SQL console),
permalink reload reproduces state, `> p95` chip narrows `t-wide`-era traces
(screenshots to `docs/research/validation/2026-07-wave2/164/`).

## Playground verification

Existing scenarios suffice: `eco-full` (service facets), `e-burst` (five
error.type facet values), `p-grpc-err` (status facets), `l-bodies`/`l-burst`
(severity + attribute filters), `j-parallel` (invocation facets: mode/
command/outcome). One new scenario to add (extend the playground matrix per
plan-161 discipline, direct on the playground's main): `f-attrs` — spans/logs carrying a
documented attribute set with known value distributions (e.g.
`http.request.method` 70/20/10 GET/POST/DELETE) so facet counts and
where-clause results are exactly assertable.

## Done criteria

- [ ] Backend lanes + live-engine facet/filter tests pass, incl. the
  parameterization proof.
- [ ] Parser round-trip tests pass; UI gates green.
- [ ] Browser evidence: facet counts match a SQL cross-check; where-clause
  `service = "checkout" AND http.request.method = "POST"` narrows; p95
  preset works; permalink reload identical.
- [ ] `plans/README.md` status row updated.

## STOP conditions

- Facet counting over JSON attribute paths is too slow on the live engine
  (>1s on the corpus) — report timings; consider bounded dimensions only,
  do NOT silently drop counts.
- The where-clause grammar tempts OR/parens — v1 is AND-only by decision;
  STOP if a corpus scenario genuinely requires OR and report.
- Any code path concatenates a user value into SQL.

## Maintenance notes

- The `attributeFilters` compiler is the single filter path — future
  surfaces (metrics 168, alert scoping 167) reuse it; never fork a second
  compiler.
- Reviewer focus: parameterization, facet-window cap, URL schema
  back-compat with pre-existing saved views.
