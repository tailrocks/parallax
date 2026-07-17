# Plan 162: Adopt the three-axis semantic color system and data-density craft rules across the Parallax UI

> **Executor instructions**: Follow this plan step by step. Read `ui/AGENTS.md`
> first. Run every verification command; verify each visual change in a real
> browser against playground data per the checklist this plan installs. STOP
> conditions are binding. Update this plan's status row in `plans/README.md`
> when done.
>
> **Drift check (run first)**:
> `git diff --stat <wave2-base>..HEAD -- ui/src/styles.css ui/src/components ui/src/lib`
> where `<wave2-base>` is the `main` commit at which plan 159 (DONE 2026-07-17, evidence commit `0e0e794`) recorded Wave
> 1's completion evidence (plans 156-161; direct-to-main delivery). If
> `/invocations` routes do not exist, STOP.

## Status

- **Priority**: P1
- **Effort**: M
- **Risk**: LOW-MED (visual regression risk; no data-layer changes)
- **Depends on**: plans 156-161 complete on `main` (Wave 1)
- **Category**: direction / design / UI foundation
- **Planned at**: `2288011`, 2026-07-17
- **Wave 2 delivery (operator directives, 2026-07-17)**: Maple
  (github.com/MapleTechLabs/maple) is the design reference. Wave 2 = plans
  162-168, delivered as direct commits to `main` in both repositories — no
  branches, no pull requests (operator delivery model, see plans/README.md
  Execution Preflight). Every feature is verified in the browser against
  the playground before the next step.

## Why this matters

Parallax's UI grew page-by-page; color, density, motion, and empty-state
behavior are decided locally per component. The reference product (Maple)
demonstrates a small set of *system-level* rules that make an observability
UI read as one instrument: color has exactly three semantic axes, numbers
always align, depth is tonal, motion never moves data the user is reading.
Adopting the rules (not the pixels — Parallax keeps its own identity: current
font stack, current accent hues, shadcn base-vega) gives every later Wave-2
plan (timeline, filters, logs, map, alerts, metrics) a shared vocabulary and
kills a class of per-page inconsistencies.

## Reference (self-contained; Maple clone optional)

Concepts adopted from Maple's `DESIGN.md` + `packages/ui` (clone
`https://github.com/MapleTechLabs/maple` and read
`packages/ui/src/styles/tokens.css`, `src/lib/colors.ts`,
`src/lib/semantic-series-colors.ts`, `src/components/service-dot.tsx` if
detail beyond this plan is wanted — everything needed is specified below):

1. **Three semantic color axes**, each with a hard "never repurpose" rule:
   - **Severity ramp** (6 OKLCH tokens: trace/debug/info/warn/error/fatal) —
     used ONLY for log severity and incident/error state; always paired with
     the literal word (a11y: color never the sole signal).
   - **Service categorical palette** — service identity ONLY, never
     sentiment/state/metric. Deterministic: hue from a hash of the service
     NAME (same service = same color on every page), rendered with a small
     squircle "ServiceDot" + service text.
   - **Percentile chart tokens** — `--chart-p50` (blue), `--chart-p95`
     (amber), `--chart-p99` (red-orange) + `--chart-error`,
     `--chart-throughput` (distinct hue from p95) for latency/RED charts;
     generic `--chart-1..5` for arbitrary series.
2. **Semantic series coloring**: series named like severities, HTTP methods,
   status classes (`2xx`…`5xx`), or span status get their semantic color;
   unknown series fall back to golden-angle (137.508°) hue rotation so any
   series count stays distinguishable.
3. **Tabular numerals everywhere numbers stack** (`font-variant-numeric:
   tabular-nums` on every table/duration/count cell).
4. **Hairline borders** (1px; 0.5px at ≥192dpi via a `--border-hairline`
   custom property).
5. **Motion doctrine**: no data that the user is about to read may move —
   content replacing a skeleton fades in opacity-only (~150ms,
   `@starting-style`); row expansion animates opacity+2px translate, never
   height; all motion respects `prefers-reduced-motion`; no loading spinners
   by default (skeletons for measurable waits, one exception: a refresh
   button may spin its own icon).
6. **Empty-state voice**: explain what is missing and what would produce it
   ("No sessions yet — this invocation emitted no session.start event"),
   never marketing copy.
7. **Density baseline**: compact control height, 12px labels for chips/
   column headers, information-dense single rows (the plan-157 surfaces
   already trend this way — codify it).

Explicitly NOT adopted (rejected for Parallax): Maple's amber brand hue,
mono-as-body font inversion (Parallax keeps its current stack; mono stays for
identifiers/durations/code), flat-no-shadow doctrine (Parallax's existing nav
treatment keeps its shadows), scanline/dot-grid textures.

## Current state

(verified at `2288011`; re-verify at wave2-base)

- Theme tokens live in `ui/src/styles.css` (CSS vars, shadcn base-vega);
  severity colors are ad hoc per component (e.g. rose borders for errors in
  runs/issues tables); service color does not exist — services render as
  plain text everywhere (`services.tsx`, `ecosystem-graph.tsx` computes its
  own inline colors).
- Charts: Recharts via `ChartContainer` (`ui/AGENTS.md` rule 13); latency
  series colors picked per chart (`routes/index.tsx` trends,
  `metric-strip.tsx`, `services.$service.tsx`).
- `components/console/relative-time.tsx`, `data-table.tsx`, `stat-card.tsx`
  are the shared row/stat primitives; no tabular-nums discipline.
- Empty states: inconsistent (some good — runs empty state; some bare).
- Browser-verification checklist currently lives only inside plan 157 (a
  file deleted at plan retirement) — it needs a durable home.

## Commands you will need

| Purpose | Command | Expected on success |
|---|---|---|
| Typecheck/lint/format | `cd ui && bun run typecheck && bun run lint && bun run check` | exit 0 |
| Unit tests | `cd ui && bun run --bun test:ci` | all pass |
| Build | `cd ui && bun run build` | exit 0 |
| Stack for browser QA | `parallax serve` + playground `docker compose … up -d` + `scenarios/run.sh --all-corner-cases` | corpus loaded |

## Scope

**In scope (ui/ only):**
- `ui/src/styles.css` — new token blocks: `--severity-*` (6), `--chart-p50/
  p95/p99/error/throughput`, `--border-hairline` (with the ≥192dpi halving).
- New `ui/src/lib/colors.ts` — deterministic service color (hash name → hue,
  2-3 lightness tiers), golden-angle fallback, semantic series-name
  detection (severity/HTTP method/status-class); unit-tested pure functions.
- New `ui/src/components/console/service-dot.tsx` — squircle dot,
  `aria-hidden`, always adjacent to the service name text.
- Sweep: severity rendering (logs table severity chips/dots, issue rows,
  invocation error chips) onto the ramp + word pairing; ServiceDot into
  traces table, logs table, services list, ecosystem nodes, invocation list,
  command palette service rows; percentile tokens into every latency chart;
  `tabular-nums` utility class onto numeric table cells; empty-state copy
  sweep to the "what's missing + what produces it" voice.
- Motion: add the opacity-only `content-enter` utility + apply to
  table-refresh transitions; remove any spinner shown for sub-second loads.
- `ui/AGENTS.md` — append (a) the three-axis color rules + never-repurpose
  clauses, (b) tabular-nums rule, (c) motion doctrine, (d) empty-state
  voice, (e) the **six-item browser verification checklist** (verbatim from
  plan 157's protocol section) as the standing rule for all future UI work.

**Out of scope:** any data-layer/GraphQL change; font-stack changes; nav
redesign; per-page feature work (plans 163-168); shadcn primitive edits
except additive utility classes.

## Git workflow

- Work directly on `main` — no branches, no pull requests (operator
  delivery model, 2026-07-17; see plans/README.md Execution Preflight).
- Commit OFTEN: one small green slice per commit (a step, a component, a
  fixed defect), Conventional Commits, DCO `-s`, exactly one agent trailer.
- **Push to `main` immediately after every commit** — never batch pushes,
  never hold local-only work; never push a slice whose targeted checks are
  red. The parallax ruleset's "Bypassed rule violations" push notice is
  expected.

## Steps

### Step 1: Tokens + pure color lib

Add the token blocks to `styles.css` (both light and dark values; pick
Parallax-fitting OKLCH values — severity hues may follow the conventional
gray/blue/green/amber/red/deep-red ramp; document each choice with a
comment). Implement + test `lib/colors.ts` (same-name-same-color property
test; golden-angle uniqueness for 32 series; semantic detection cases).

**Verify**: `bun run --bun test:ci -- src/lib/__tests__/-colors.test.ts` →
pass; `bun run build` → exit 0.

### Step 2: ServiceDot + severity sweep

Create ServiceDot; sweep the listed surfaces. Severity chips always render
`DOT + WORD` (word visible at all densities).

**Verify**: updated component tests pass; browser walk (playground corpus):
same service shows the same color on traces/logs/services/ecosystem/
invocations pages (screenshot pair proving it); severity words visible.

### Step 3: Charts + numerals + motion + empty states

Percentile tokens into latency charts (p50/p95/p99 legend colors constant
across pages); `tabular-nums` on numeric cells (durations, counts, columns
align vertically); `content-enter` on table refresh; empty-state copy sweep.

**Verify**: unit tests pass; browser walk per checklist — no layout shift
on refresh (record a before/after capture), aligned numerals screenshot,
each empty state exercised via a filter that returns nothing.

### Step 4: Codify in ui/AGENTS.md

Append the rules + checklist. This is the durable home future plans cite.

**Verify**: `grep -n "severity" ui/AGENTS.md` shows the never-repurpose
rules; checklist present.

## Playground verification

Corpus: `scenarios/run.sh --all-corner-cases` suffices (no new scenarios).
Specifically use `l-burst` (severity spectrum), `eco-full` (service color
consistency across pages), `m-shapes` (percentile chart legend). Browser
checklist per page touched; screenshots to
`docs/research/validation/2026-07-wave2/162/`.

## Done criteria

- [ ] All UI gates green (`typecheck`/`lint`/`check`/`test:ci`/`build`).
- [ ] `lib/colors.ts` pure tests pass incl. determinism property.
- [ ] Browser evidence: same-service-same-color across ≥4 pages; severity
  word+color pairing; aligned numerals; no-shift refresh.
- [ ] `ui/AGENTS.md` carries the rules + six-item checklist.
- [ ] `plans/README.md` status row updated.

## STOP conditions

- A severity or service color choice collides with an existing semantic use
  that cannot be reconciled (e.g. the violet Runs accent) — report the
  conflict with screenshots rather than shipping ambiguity.
- Applying `content-enter` breaks virtualized table row identity (DOM
  reuse) — scope it to container-level, never per-virtual-row, and note it.

## Maintenance notes

- Every later Wave-2 plan consumes these tokens/helpers — land this first.
- Reviewer focus: no component keeps a hardcoded severity/service color; the
  three never-repurpose rules hold in new code.
