# Plan 172: Design-system uplift and the documented design guide (foglamp-informed)

> **Executor instructions**: Follow this plan step by step. Run every
> verification command and confirm the expected result before moving on. If
> anything in "STOP conditions" occurs, stop and report — do not improvise.
> When done, update this plan's row in `plans/README.md`.
>
> **Drift check (run first)**: `git diff --stat f6208070..HEAD -- ui/src/styles.css ui/AGENTS.md ui/src/shared/console/ ui/src/layout/`
> — on mismatch with the excerpts below, STOP.

## Status

- **Priority**: P2
- **Effort**: L (guide M + 6 independent S/M adoption items)
- **Risk**: MED (visual changes across every surface; staged, token-first,
  visual-lane baselines regenerate)
- **Depends on**: none; plan 170's strengthened Playwright lanes recommended
  first (regression net for visual/behavioral fallout); coordinates with
  plan 171 feature 4 (snippet content)
- **Category**: dx
- **Planned at**: parallax `f6208070`, 2026-08-13

## Why this matters

Operator direction (2026-08-13): professional-designer quality bar, latest
UI/UX practice, and a WRITTEN design guide; foglamp
(github.com/foglamp-labs/foglamp, Apache-2.0) is the named reference for
design/color quality. A deep study of foglamp shows Parallax already shares
its foundation (neutral oklch tokens, Base UI shadcn variant, Inter-class
typography, dark = re-lit not inverted) — the differences are a set of
specific, adoptable mechanisms (elevation-as-shadow, loading discipline,
data-tinting patterns) and, above all, the fact that foglamp's rules are
*written down and enforced* while Parallax's live only as scattered
conventions. The deliverables: `ui/DESIGN.md` as the canonical guide, plus
the highest-impact mechanism adoptions.

**License**: foglamp is Apache-2.0 — patterns AND code are usable; if any
code is copied near-verbatim, preserve attribution per Apache-2.0 (note in
NOTICE). Contrast: Maple (plan 171) is FSL — ideas only, never code.

## Current state (verified)

- `ui/src/styles.css` (326 lines) — Parallax tokens TODAY:
  `--background: oklch(0.995 0 0)`, `--border: oklch(0.922 0 0)`,
  `--chart-1: oklch(0.87 0 0)`, `--radius: 0.625rem`,
  `--border-hairline: 1px` (0.5px on hi-dpi, lines 110-117), `.dark`
  `--background: oklch(0.17 0 0)`, `--border: oklch(1 0 0 / 10%)` —
  i.e. the neutral-canvas system already matches foglamp's philosophy.
  MISSING vs foglamp: layered shadow-elevation tokens (no `--custom-shadow*`
  equivalents; separation is border-based), per-accent shadow variants,
  dark inset-bevel re-derivations.
- `ui/AGENTS.md` — binding rules: theme only through CSS vars in
  `src/styles.css` (rule 11); shadcn components only via lock-local CLI
  (rule 9); Base UI `render` prop not `asChild` (rule 10); charts =
  Recharts in `ChartContainer` with series colors from `--chart-*`
  (rule 12); TanStack Table + shadcn Table split (rule 13); `cn()` for
  conditional classes (rule 14); every chart/list links onward (rule 17);
  React Flow for graphs (rule 24). The guide must ABSORB these, not
  contradict them.
- Existing kit (do not duplicate — refine): `ui/src/shared/console/` —
  stat cards, sparklines, pill meters, heat cells (`heat-cell.tsx`),
  skeletons with delayed-loading (`hooks.ts`), empty states with copyable
  OTLP endpoint, relative time, copy buttons; `ui/src/layout/` — app
  shell, ⌘K palette, theme switcher (System/Light/Dark with motion
  indicator).
- Playwright visual lane: goldens for shell + investigations dark only
  (`ui/tests/e2e/visual/`), Linux-CI-authored, `maxDiffPixels` budget.
- foglamp mechanisms verified in its clone (evidence paths under
  `packages/ui/src/styles/globals.css`, `apps/web/src/components/app/` in
  github.com/foglamp-labs/foglamp @ 2026-08-13):
  1. Shadow-as-border: `--custom-shadow: 0 0 0 0.5px rgba(0,0,0,.06), 0 .5px 2px -.5px rgba(0,0,0,.06), 0 1px 4px rgba(0,0,0,.04)`
     + per-accent tinted variants; dark mode re-derives as
     `inset 0 .75px 0 rgba(255,255,255,.03)` bevel + white ring + stacked
     black drops. Their rule: never a border for separation.
  2. Loading discipline: skeletons only after 700ms delay; one 75ms
     opacity-only page fade per hard load (module-scope gate); skeleton
     rows mirror real column alignment + `table-fixed`.
  3. Nav: per-section hue chips (tinted bg + inset color ring) with
     outline→filled icon crossfade on active (stacked grid cell, 100ms).
  4. Empty states: dashed border + icon at 40% opacity + copy-pasteable
     instrumentation snippet with segmented SDK toggle (sliding pill,
     cross-faded snippets in one grid cell so height never shifts).
  5. StatCard: animated number ticker, delta badges with inverted-good
     semantics, sparkline with dashed trailing not-yet-complete bucket,
     narrative order Volume → Health → Performance → Cost.
  6. One color map per domain concept: a single exported record drives
     badge + bar + chip + icon per type; statuses error=rose,
     aborted=amber distinct.
  7. Micro-polish: `active:scale-[0.97]` buttons, `tabular-nums`
     everywhere numeric, thin `color-mix` scrollbars, `selection:` tint,
     hover-instant-bg / eased-shadow transitions.

## Commands you will need

| Purpose | Command | Expected |
|---|---|---|
| Typecheck/tests | `cd ui && bun run typecheck && bun run test` | green |
| UI gates | `cargo xtask ui && cargo xtask policy --only ui.architecture && cargo xtask policy --only ui.ratchets` | green |
| Browser lanes | `cargo xtask browser-contracts-serve && cargo xtask browser-full-stack-serve` | green |
| Visual baselines | run visual lane; regenerate goldens per its README when intentional | diffs only where intended |
| Docs links | `cargo xtask docs links` | pass |

## Scope

**In scope**: `ui/DESIGN.md` (new), `ui/src/styles.css` (token additions
only — no removals), `ui/src/shared/console/**` (stat cards, skeletons,
empty states, heat cells refinements), `ui/src/layout/**` (nav chips,
page-fade), a new `ui/src/shared/signal-colors.ts`-style color-map module
+ migration of scattered per-signal colors to it, `ui/tests/e2e/visual/`
golden regeneration, `ui/AGENTS.md` (one pointer line to DESIGN.md),
`NOTICE` (attribution if code copied).

**Out of scope**: route/feature behavior changes; chart library changes
(Recharts stays, rule 12); React Flow internals (styling via tokens only);
squircle corner plugin (Chromium-only — rejected); grayscale-chart-token
copy (Parallax's chart tokens + dataviz palette are stronger for
many-series telemetry — keep); any Tailwind major migration not already in
place.

## Git workflow

PR-only `main`; stage as ~4 PRs (guide+tokens, loading+empty states,
nav+statcards, color-map consolidation); `git commit -s`; Conventional
Commits (`feat(ui): …` / `docs(ui): …`); agent trailer per `COMMITS.md`.

## Steps

### Step 1: Author `ui/DESIGN.md` — the canonical design guide

Sections (all with concrete values from `ui/src/styles.css` and rules
absorbed from `ui/AGENTS.md`):
1. Principles: telemetry owns the chroma (neutral canvas), density with
   hierarchy (Linear-class), dark is re-lit not inverted, every number is
   `tabular-nums`, every chart/list links onward, motion is functional
   (≤150ms, opacity/transform only, respects `prefers-reduced-motion`).
2. Tokens: full palette table (light+dark oklch), radius scale, hairline
   var, NEW elevation tokens (Step 2), chart series tokens + when to use
   the dataviz palette.
3. Typography scale: page title / body / meta / micro with exact classes.
4. Loading & entrance discipline: 700ms skeleton delay, single page fade,
   skeleton-mirrors-columns rule, `table-fixed` pairing.
5. Color maps: one exported record per domain concept (signal type, span
   status, severity, invocation status) — the module from Step 6 is
   normative.
6. Component rules: buttons/badges/tables/empty states/detail panels —
   each with its shared/console exemplar path.
7. Modern-practice appendix: dated (2026-08) rationale for each adopted
   trend (oklch + CSS-first theming, shadow elevation, reduced-motion
   compliance, density-first dashboards, command-palette-centric nav,
   accessible contrast targets APCA-aware) with sources; explicitly a
   living section — re-verify on major redesigns.
Add one line to `ui/AGENTS.md` §Design pointing at DESIGN.md as the
canonical guide (do not duplicate rules).

**Verify**: `cargo xtask docs links` pass; DESIGN.md token table values
grep-match `ui/src/styles.css` (spot-check 5 tokens).

### Step 2: Elevation token system

Add to `ui/src/styles.css`: `--elevation-1/-2` layered shadow tokens
(hairline ring + soft drops) with light values and `.dark` re-derivations
(inset bevel + white-alpha ring + stacked drops), plus accent-tinted
variants for the semantic colors actually used (destructive, success,
warning, info — audit `shared/console` first). Apply to Card-class
surfaces in `shared/console/` (stat cards, panels) REPLACING their border
where visual parity holds. Do NOT ban borders repo-wide — Parallax keeps
`--border` for tables/inputs; DESIGN.md documents when each applies
(elevation for floating surfaces, hairline borders for inline structure).

**Verify**: both browser lanes green; visual goldens regenerated with
intentional diffs only; light + dark manually screenshotted for the PR.

### Step 3: Loading & entrance discipline

Audit `shared/console/hooks.ts` delayed-skeleton timing → align to the
documented 700ms; add the once-per-boot 75ms opacity page-fade (module-
scope gate, `prefers-reduced-motion` respected); ensure table skeletons
mirror real column alignment and their tables use fixed layout where row
identity matters (logs, traces, issues).

**Verify**: `bun run test` green; no skeleton flashes on fast loads
(manual: throttled + unthrottled load of `/logs`).

### Step 4: Instrumented empty states

Extend the existing empty-state component: dashed border + 40%-opacity
icon idiom, and on zero-data surfaces render tabbed (Rust/Java/JS)
copy-pasteable setup snippets sourced from
`docs/guide/instrument-snippets.md` (plan 171 feature 4; if that file
doesn't exist yet, inline the OTLP endpoint + `service.name` env exports as
v1 and leave a TODO referencing plan 171). Segmented toggle = sliding pill
with cross-faded fixed-height content (no layout shift).

**Verify**: contracts lane spec: fresh `shell-empty` dataset → overview
empty state shows tabs, copy button copies the visible snippet.

### Step 5: StatCard + nav polish

- StatCards (`shared/console/`): animated number transitions
  (respect reduced-motion: jump-cut fallback), delta badges with
  inverted-good semantics for error-rate/latency, sparkline dashed
  trailing segment for the current incomplete bucket.
- Nav (`ui/src/layout/app-shell.tsx` + `ui/src/shared/navigation.ts`):
  per-section hue chips (tinted bg + inset ring via the Step 2 accent
  tokens) + outline→filled icon crossfade on active. Keep the existing
  icon set — crossfade needs a filled variant per icon; if the current set
  lacks filled variants, use weight/color emphasis instead (STOP condition
  4 if neither works accessibly).
- Buttons: `active:scale-[0.97]` press feedback on primary actions;
  `tabular-nums` audit across numeric cells.

**Verify**: lanes green; a11y specs (plan 170 Step 5 set) still pass —
contrast of tinted chips checked against WCAG AA at minimum.

### Step 6: One color map per concept

Create a single normative module (e.g. `ui/src/shared/signal-colors.ts`)
exporting records for: signal type (trace/log/metric/error), severity
ladder, span status, invocation status/outcome, test rollup/flaky state —
each mapping to token-based classes for badge/bar/chip/icon. Migrate
scattered inline color choices in `shared/console/` + feature components
to it (grep hex/`text-{color}-` literals; migrate only clear duplicates —
chart series stay on `--chart-*`/dataviz rules).

**Verify**: `grep -rn "#[0-9a-fA-F]\{6\}" ui/src/features ui/src/shared --include=*.tsx | wc -l`
strictly decreases (record before/after in PR); typecheck + lanes green.

## Test plan

Visual lane goldens regenerate per step (intentional diffs documented in
each PR with before/after screenshots, light AND dark). Behavioral specs
from plan 170 are the regression net. New unit tests: color-map module
(exhaustive key coverage per record), empty-state snippet-tab component.

## Done criteria

- [ ] `ui/DESIGN.md` exists covering all 7 sections; `ui/AGENTS.md` points
      to it; token table matches `styles.css`.
- [ ] Elevation tokens in `styles.css` with light+dark+accent variants;
      applied to floating surfaces; border-vs-elevation rule documented.
- [ ] 700ms skeleton delay + once-per-boot page fade + reduced-motion
      compliance in place.
- [ ] Zero-data surfaces show tabbed setup snippets with copy.
- [ ] Nav chips + statcard ticker/dashed-tail shipped, AA contrast held.
- [ ] `signal-colors.ts` normative module; inline hex count reduced and
      recorded.
- [ ] `bun run typecheck|test`, all browser lanes, `cargo xtask ui`,
      visual goldens — green.
- [ ] NOTICE updated if any foglamp code copied near-verbatim.
- [ ] `plans/README.md` row updated.

## STOP conditions

1. Drift check fails on `styles.css` / console kit.
2. Elevation-for-border swap breaks readability on any dense table —
   report with screenshots; tables may stay border-based by design.
3. Visual-lane goldens diff on surfaces you did not touch — bleed;
   investigate token scoping before regenerating.
4. Nav crossfade cannot meet AA contrast with the current icon set —
   report options (filled set, weight emphasis) instead of shipping
   sub-AA.
5. Any step tempts a Tailwind/major-dependency migration — out of scope,
   report.

## Maintenance notes

- DESIGN.md is now the arbiter: future UI PRs that add colors, shadows, or
  motion outside its vocabulary should be blocked in review, and the guide
  amended deliberately (dated changelog section).
- The trend appendix carries dates — re-verify claims at the next major
  design pass; stale trend prose is worse than none.
- Plan 167's agent-browser pass and the visual lane both consume these
  changes — re-run 167's c11 after each PR here.
