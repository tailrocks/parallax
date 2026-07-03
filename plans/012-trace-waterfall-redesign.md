# Plan 012: Trace detail — real waterfall (tree, time axis, span inspector)

> **Executor instructions**: Step by step; verify each; STOP conditions
> binding; update `plans/README.md` when done. This is the product's most
> important screen — favor fidelity over speed.
>
> **Reference project**: operator-designated local reference console — name
> NEVER in this repo. `REF_ROOT="$(cat plans/.reference-root)"` (STOP if
> missing), pinned at its commit `9f028d7`. Leak check before commits.
>
> **Drift check (run first)**: `git diff --stat ad9115d..HEAD -- ui/src/routes/traces.\$traceId.tsx`
> Plans 005-008 and 010 (Span.events) must be DONE.

## Status

- **Priority**: P1 (highest-value screen)
- **Effort**: L
- **Risk**: MED
- **Depends on**: plans/005-008, plans/010
- **Category**: bug + tech-debt (the current rendering is functionally wrong)
- **Planned at**: commit `ad9115d`, 2026-07-03

## Why this matters

The current "waterfall" is upside-down and flat: spans are sorted **newest-first**
(`traces.$traceId.tsx:92-98` — the comment even says "newest on top"), `parentSpanId` is
fetched but never used, so there is no tree, no indentation, no time axis — you cannot read
what called what or where time went. Every serious tracing UI is root-at-top, depth-indented,
time-positioned. The reference implements exactly that with clean math (DFS ordering, shared
3-column grid, proportional rounded bars) plus a span inspector pane. We port its structure
and adapt its LLM-specific semantics to OTel span kinds.

## Current state

`ui/src/routes/traces.$traceId.tsx` (369 lines, verified at `ad9115d`):
- Loader `:37-44` — **interpolates `params.traceId` unescaped** into the GraphQL string
  (`trace(traceId: "${params.traceId}")`, same at `:43`) — every other call site uses
  `gqlString`; fix here (correctness bug).
- Fetches spans (tsNanos service name kind statusCode statusMessage durationNs spanId
  parentSpanId runId links attributes) + `logsByTrace`.
- `:85-98` computes start/end/total, sorts spans newest-first by BigInt ts.
- `:117-133` `PageHeading` "Trace detail"; `:170-221` waterfall Card: flat button rows,
  offset/width % bars `h-2 rounded-full`, colors `bg-(--brand-rose)`/`bg-(--brand-blue)`;
  `:224+` correlated-logs Card (severity+body, **no timestamps**), span-detail right column
  (sticky Card, attributes list, `db.query.text` block `:298-307` without copy), MetricStrip.
- Empty: bare `<p>Trace not found.</p>` (`:82`).
- After plan 010: `Span.events` (JSON string) available.
- Kit (plan 008): CopyButton, span-kind palette (`spanKindBar/Chip/Badge` — error ⇒ rose),
  formatters, EmptyState, ScrollFade.
- Reference waterfall (read both files fully):
  - `$REF_ROOT/apps/web/src/lib/trace-timeline.ts` — `orderSpans` (`:15-35`): children map
    keyed by parent, roots = no parent or parent missing, DFS `walk(span, depth)` pushing
    `{span, depth}`, children sorted by start; `computeWindow` (`:38-43`): min start / max
    end, span ≥ 1. **Port these two functions almost verbatim**, replacing `toMs(startTime)`
    with BigInt-safe ns math (`Number((BigInt(ts) - BigInt(start)) / 1_000n)` → µs precision
    is enough for percentages; keep BigInt until the final ratio).
  - `$REF_ROOT/apps/web/src/components/app/trace-timeline.tsx` — shared grid
    `grid-cols-[11rem_minmax(0,1fr)_6.5rem]` (`:272`, `:315`); overlays inset
    `left-[11rem] right-[6.5rem]` (`:288-294`); whole-trace root row `:309-366` (icon chip
    `size-5 rounded-md corner-squircle bg-primary/15 text-primary`, full-width
    `bg-primary/70` bar, right column duration `text-[11px] font-medium text-foreground/80
    tabular-nums` over sub-line `text-[10px] text-muted-foreground`); span rows `:368+`
    (indent `paddingLeft: (depth+1)*14 + 4`; bar track `relative h-5`, bar `absolute
    top-1/2 h-2 -translate-y-1/2 rounded-full`, `left:{offset}% width:{width}%`, width
    clamped `min 1.5%`, `≤ 100-offset`); row selection `bg-accent/70`, hover
    `bg-accent/50`; keyboard j/k / arrows (in its detail client `:218-237`).
  - Span inspector: `$REF_ROOT/apps/web/src/app/(app)/traces/[traceId]/trace-detail-client.tsx`
    — `DetailPanel` width-animates to 420px squeezing the timeline (`:522-614`); pane is a
    Card `max-h [calc(100svh-16rem)]`, header = name + type badge + close; sections split by
    `border-b border-border/40`: fields grid (2-col: muted label / `text-sm tabular-nums`
    value), payload blocks, metadata badges; summary stat strip `grid grid-cols-2
    sm:grid-cols-3 lg:grid-cols-6` (`:321-373`); error banner `:375-428` (rose, click →
    select first errored span, copy button); context chips row `:261-307` (pill links
    `rounded-full bg-card px-2.5 shadow-(--custom-shadow)` + `IconArrowUpRight`).
  - **Do NOT port**: replay/scrubber/playhead, TTFT markers, throughput backdrop, model/cost/
    token fields — LLM semantics that don't map to Parallax. Keep the ScrubRuler out too;
    add a static quarter-point time ruler instead (labels at 0/25/50/75/100% of
    `formatDurationNs(total)`).

## Commands you will need

From `ui/`: `rtk bun run typecheck` / `lint` / `test` / `build` → 0; dev + `parallax serve` +
playground traffic for manual checks. Leak check: plan 005 table.

## Scope

**In scope**: `ui/src/routes/traces.$traceId.tsx` (rewrite); NEW
`ui/src/components/console/trace-waterfall.tsx` + `ui/src/lib/trace-tree.ts` (orderSpans/
computeWindow ports + tests).
**Out of scope**: run pages, logs page, MetricStrip internals (consume as-is), API.

## Git workflow

`main`; `feat(ui): real trace waterfall`; `git commit -s`; trailer
`Co-authored-by: Claude <noreply@anthropic.com>`; leak check first.

## Steps

### Step 1: Fix the injection bug + extend the query

Loader: wrap both interpolations in `gqlString(params.traceId)` (`:39`, `:43`). Add `events`
to the span selection (plan 010). Keep `logsByTrace`.

**Verify**: typecheck; a traceId containing `"` no longer breaks the query (unit-test
`gqlString` usage by constructing the query string in a pure helper and asserting escaping).

### Step 2: `ui/src/lib/trace-tree.ts`

Port `orderSpans` + `computeWindow` with ns-string BigInt math; types from the route's
`TraceSpan`. Root = span with empty/absent `parentSpanId` OR parent not in the set. Export
`{ordered: {span, depth}[], window: {startNanos: bigint, totalNanos: bigint}}` helpers +
`positionPct(span, window)` returning `{offset, width}` with the reference clamps (min 1.5,
≤ 100-offset).

**Verify**: unit tests — linear chain depth, orphan parent → root, children sorted by start,
zero-duration trace (total ≥ 1), position clamps.

### Step 3: Waterfall component

`trace-waterfall.tsx`: the 3-column grid (`grid-cols-[11rem_minmax(0,1fr)_6.5rem]`),
quarter-point ruler row, whole-trace root row (icon `IconAffiliate` in `bg-primary/15`
chip, `bg-primary/70` full bar, total duration right), then ordered span rows:
- Label gutter: indent `(depth+1)*14+4`px; `SpanKindChip` (kit) + span name `break-words`;
  error/OK badges below when relevant.
- Bar: `spanKindBar(kind, statusCode)` color (`bg-rose-500` on error), reference geometry.
- Right column: `formatDurationNs(durationNs)` (`text-[11px] font-medium tabular-nums`) over
  service name (`text-[10px] text-muted-foreground`).
- Row = button; selected `bg-accent/70`; hover `bg-accent/50`; gridlines at quarters
  (`bg-border/50` absolute, inset `left-[11rem] right-[6.5rem]`).
- Keyboard: j/k + ArrowUp/Down move selection through `ordered`; ignore when target is an
  input.

**Verify**: with a multi-service playground trace: root at top, children indented under
parents, bars positioned on a shared axis, ruler labels correct.

### Step 4: Span inspector pane

Right `aside` (animate width to 420px like the reference if `motion` is present — CSS
transition fallback fine): Card with header (span name + `SpanKindBadge` + close), ScrollFade
body, sections split by `border-b border-border/40`:
1. Error banner when `statusCode === "STATUS_CODE_ERROR"` (`bg-destructive/20
   text-destructive` + statusMessage + CopyButton).
2. Fields grid: Started (`formatTimeInRange` vs trace window + absolute tooltip), Duration,
   Service, Kind, Status, Span ID (mono + CopyButton), scopeName.
3. **Events** (new): parse `span.events` JSON → time-ordered list (event name + relative
   offset + attribute table; exception events styled rose with stacktrace in a mono
   `ScrollFade` block + CopyButton).
4. Attributes: key/value rows (mono values); `db.query.text` gets its own block with
   CopyButton (spec flow "span → copy query").
5. Resource attributes (collapsed `<details>` or secondary section).
6. Links: parsed `links` → `/traces/$traceId` Links.
7. Span logs: `logsByTrace` filtered to the span — WITH timestamps
   (`formatTimeInRange`), severity badge, body mono.
Selecting nothing shows a trace-level pane: fields (Trace ID + copy, Started, Duration,
Spans, Services count, Errors count rose-when->0) — mirror the reference's "whole trace"
inspector.

**Verify**: click span → pane opens with sections; exception event renders stacktrace;
`db.query.text` copies; close button and Escape work.

### Step 5: Page composition

`PageHeader back={navItem("/traces")}` + title = root span name, `titleTrailing` =
CopyButton(traceId); context chips row: Run pill (when `runId` present → `/runs/$runId`),
service pills (unique services in trace); summary strip `grid grid-cols-2 sm:grid-cols-3
lg:grid-cols-6`: Spans, Services, Errors (rose when >0), Duration, Logs (count), Started —
plain `Stat`-style (kit), not KpiCards. Error banner above the waterfall when any span
errored (click → select first errored span). Below waterfall: trace-level logs Card (all
`logsByTrace` with timestamps + severity + span link that selects the span) and the
existing `MetricStrip` (unchanged component from plan 008 Step 7). Not-found → `EmptyState`
inside the shell (kills the bare `<p>`).

**Verify**: `grep -n "Trace not found" ui/src/routes/traces.\$traceId.tsx` shows it only
inside the EmptyState; header shows breadcrumb `Traces › <name>` + copy.

### Step 6: Gate

typecheck/lint/test/build 0; both themes; keyboard nav; leak check → no output.

## Test plan

`ui/src/lib/__tests__/trace-tree.test.ts`: ordering/depth/window/position cases from Step 2.
`ui/src/components/console/__tests__/waterfall.test.tsx`: render 4-span fixture (root + 2
children + error grandchild) → assert DOM order equals DFS order, indent style increases
with depth, error bar has rose class; selection callback fires on row click and on `j`.

## Done criteria

- [ ] typecheck / lint / test / build exit 0; new tests pass
- [ ] Root span renders FIRST; children indented; `grep -n "newest first"` in the route → gone
- [ ] `gqlString` wraps both traceId interpolations
- [ ] Span events visible in inspector (or documented fallback path from plan 010)
- [ ] Correlated logs show timestamps; every ID has a CopyButton
- [ ] No `--brand-*` / `.parallax-panel` / lucide in the route
- [ ] Leak check → no output; `plans/README.md` row updated

## STOP conditions

- Plan 010's `Span.events` absent AND the error_events fallback also unavailable.
- A real trace renders >2k spans and the flat DOM list janks — report; virtualization is a
  follow-up decision, not an improvisation.
- The width-animation approach fights the router/layout — fall back to static 420px pane and
  note it (not a STOP, a noted deviation); STOP only if the pane can't render at all.

## Maintenance notes

- The waterfall component will be reused by the run detail (plan 017 may embed per-trace
  mini-waterfalls later); keep it prop-driven (spans in, selection out), no route coupling.
- If span counts grow, add virtualization to the row list (the 3-col grid keeps row height
  fixed — virtualizes cleanly).
- Cross-trace links (span links) currently navigate blind; when plan 015 lands, service
  pills should link to service detail.
