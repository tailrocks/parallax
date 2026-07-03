# Plan 016: Issues — grouped-error console (list + detail) on the reference grammar

> **Executor instructions**: Step by step; verify each; STOP conditions
> binding; update `plans/README.md` when done.
>
> **Reference project**: operator-designated local reference console — name
> NEVER in this repo. `REF_ROOT="$(cat plans/.reference-root)"` (STOP if
> missing), pinned at its commit `9f028d7`. Leak check before commits.
>
> **Drift check (run first)**: `git diff --stat ad9115d..HEAD -- ui/src/routes/issues.index.tsx ui/src/routes/issues.\$fingerprint.tsx`
> Plans 005-008 must be DONE.

## Status

- **Priority**: P1
- **Effort**: L
- **Risk**: MED
- **Depends on**: plans/005-008 (010/012 improve its trace links but are not blockers)
- **Category**: tech-debt (UX redesign)
- **Planned at**: commit `ad9115d`, 2026-07-03

## Why this matters

Issues is the page the spec models on a Sentry workflow the operator uses daily
(`docs/research/architecture/simple-ui-v2.md:33-55`). The list is functionally close but has
dead cells (trend sparkline and events count not clickable — spec says both must be), no
time-range filter (spec requires), tiny link targets (only the title cell navigates), and a
KPI strip pushing rows below the fold. The detail's stacktrace is a raw `<pre>` blob (spec:
frames with file:line), and CLI/ID handoff has no copy affordances (the product's core
agent-era flow). Restyle both on the reference grammar and close those gaps.

## Current state

- `ui/src/routes/issues.index.tsx` (verified): `validateSearch` q/service/status/sort
  (`:47-60` — **no time range**); loader `issues(…, limit: 100) + services` (`:62-88`);
  inline SVG `Sparkline` (`:93-118`, `fill-(--chart-1)`, not clickable); table where only
  the title links; 4 KpiCards; `.parallax-panel` toolbar (Input + 3 Selects); hand-rolled
  empty panel with OTLP snippet.
- `ui/src/routes/issues.$fingerprint.tsx` (audited; verify on read): loader `issue{events(
  limit:20)} + issueTrend`, second query for trace resource/breadcrumbs (`:63-69`);
  `PageHeading` with Resolve/Reopen (`issueSetStatus` mutation); badges row; 4 KpiCards;
  `TrendChart` bar chart **with working click→window drill-down** (`:122-130, 259-277` —
  KEEP this behavior); latest-event Card with raw `<pre>` stacktrace (`:360-364`); tags
  table; context sections; breadcrumbs (severity+body, **no timestamps**, `:404-413`);
  occurrences list; CLI snippet at `:376` (not copyable); empty = bare `<p>Issue not
  found.</p>` (`:241`).
- Kit: table toolkit, RangePicker + `resolveRangeSearch`, HeatCell, RelativeTime,
  CopyButton, EmptyState, skeletons, StatCard, formatters, `CardSparkline`.
- Reference grammar: list = plan 011's toolbar/table/pagination; detail = plan 015's
  header/stat/trend layout; inline-chip links `text-muted-foreground hover:text-foreground`
  + `IconArrowUpRight`; child links inside rows `stopPropagation`.

## Commands you will need

From `ui/`: `rtk bun run typecheck` / `lint` / `test` / `build` → 0; dev + serve + playground
emitting errors. Leak check: plan 005 table.

## Scope

**In scope**: `ui/src/routes/issues.index.tsx`, `ui/src/routes/issues.$fingerprint.tsx`
(rewrites), NEW `ui/src/lib/stacktrace.ts` (frame parser + tests).
**Out of scope**: API (the `issues` query already accepts `fromNanos/toNanos` — use them);
trace pages; runs.

## Git workflow

`main`; `feat(ui): redesign issues console`; `git commit -s`; trailer
`Co-authored-by: Claude <noreply@anthropic.com>`; leak check first.

## Steps

### Step 1: List — URL range + reference table

Extend `validateSearch` with the range schema; pass `fromNanos/toNanos` into the `issues`
query (args exist server-side: `crates/parallax-api/src/lib.rs:768-779` — window on
last-seen). Layout: `PageHeader` (Issues icon, actions=RangePicker) → `Toolbar` (SearchInput
"Search message/type…", service FilterSelect, status FilterSelect open/resolved, sort
FilterSelect LAST_SEEN/FIRST_SEEN/EVENTS/TREND, ClearFiltersButton, right count from
`issues.total`) → `Table` (interactive rows → detail; delete the KPI strip):

| Column | Treatment |
|---|---|
| Issue | flexible: `errorType` `font-medium` + title `truncate`; sub-line culprit `text-xs text-muted-foreground font-mono`; open rows w/ recent activity get rose left-accent bar |
| Trend | `w-24`; replace the inline SVG with `CardSparkline`-style mini area (rose); **cell links to detail** (spec) |
| Events | right `w-24` sortable(EVENTS) `formatCount`; **links to detail** |
| Age | right `w-28`: first-seen `RelativeTime` |
| Last seen | right `w-32` sortable(LAST_SEEN) `RelativeTime` |
| Tags | `w-56`: top 2 tag chips (Badge secondary size sm) + `+N` |
| Status | `w-24`: Badge (open → rose, resolved → emerald) |

Empty-first-load → EmptyState + OTLP snippet (CopyButton). Filtered-empty variant.

**Verify**: URL holds q/service/status/sort/range; whole-row click navigates; Trend/Events
cells navigate independently (`stopPropagation` semantics correct); range filter narrows the
list.

### Step 2: Stacktrace frames (`ui/src/lib/stacktrace.ts`)

Parser: input raw string → `Frame[] {raw, fn?, file?, line?, col?, isApp?}` supporting the
common shapes (Rust backtrace `N: fn\n at file:line`, Python `File "x", line N, in f`,
Node/V8 `at fn (file:line:col)`, Go `fn()\n\tfile:line`, Java `at pkg.Cls.m(File.java:N)`).
Unknown lines pass through as `{raw}` — the renderer falls back gracefully. `isApp`
heuristic: not under site-packages/node_modules/cargo registry/goroot/jdk.

**Verify**: unit tests with one fixture per language + a garbage fixture (all raw
passthrough, no throw).

### Step 3: Detail — reference layout + copy affordances

Layout top-to-bottom:
1. `PageHeader back={navItem("/issues")}` title=`errorType`, `titleTrailing` =
   CopyButton(fingerprint); description = message; actions = status Button
   (Resolve/Reopen via existing `issueSetStatus` mutation, outline; keep optimistic
   behavior) + RangePicker (drives trend + events windows).
2. Chips row: service pill, status Badge, first/last seen RelativeTime chips.
3. Stat row (4 StatCard sm): Events (`formatCount`, sparkline from trend), Services? no —
   keep: Events, First seen, Last seen, Trend-24h delta (`deltaInverted`).
4. **Trend chart** — KEEP the existing click-bucket→occurrences drill-down behavior exactly
   (it is the one working spec flow); restyle to the plan-006 chart primitive (bars
   `--destructive`, bucket click sets the occurrences window + scrolls to the list).
5. Stacktrace Card: frames from Step 2 — each row `file:line` mono + fn; app frames
   emphasized (`font-medium`), library frames muted + collapsed behind "N library frames"
   toggles; culprit frame highlighted (rose left bar); raw-text fallback `<pre>` when
   parsing yields < 2 structured frames; CopyButton (raw text) in the card header.
6. Tags table + context sections (restyle to Cards; content unchanged).
7. Breadcrumb logs: add per-line timestamps (`formatTimeInRange`) + severity badges.
8. Occurrences list: each row = time RelativeTime + service + trace link chip
   (`IconArrowUpRight`) → `/traces/$traceId`.
9. CLI handoff Card: `parallax issue context <fingerprint>` snippet in mono block with
   CopyButton (spec: hand the agent the reference).
Not-found → EmptyState (kills the bare `<p>`).

**Verify**: trend click still narrows occurrences (regression-check this manually);
stacktrace shows structured frames for a Rust and a Python playground error; every ID/
snippet copies.

### Step 4: Gate

typecheck/lint/test/build; both themes; leak check.

## Test plan

`ui/src/lib/__tests__/stacktrace.test.ts` — per-language fixtures + garbage. Component:
list fixture renders trend/events as links (assert hrefs); detail fixture renders frame rows
+ culprit highlight; status button calls mutation (mock graphql).

## Done criteria

- [ ] typecheck / lint / test / build exit 0; tests pass
- [ ] Issues list: time-range filter live; Trend + Events cells navigate; whole-row click
- [ ] Detail: parsed frames w/ fallback; breadcrumbs have timestamps; CLI snippet + IDs
      copyable; trend click-drill preserved
- [ ] No KpiCard/.parallax-panel/lucide/`"Issue not found."` bare paragraph
- [ ] Leak check → no output; `plans/README.md` row updated

## STOP conditions

- `issues` query rejects fromNanos/toNanos (schema drift vs `lib.rs:768-779`).
- The existing trend drill-down regresses and can't be restored within the restyle — revert
  the chart section to the old implementation + report (the behavior outranks the styling).
- Stacktrace parsing needs >5 language formats to be useful for the operator's actual data —
  report which formats appear in real `error_events` instead of guessing more regexes.

## Maintenance notes

- The trend chart is the template consumer for a future brush (plan 014 pattern) — click
  stays, brush is additive later.
- Frame parser is deliberately UI-side; if it matures, consider moving to the API/bundle
  layer so CLI/agents get frames too.
- Tag chips cap at 2+N; the detail's tag table is the full view — don't grow list chips.
