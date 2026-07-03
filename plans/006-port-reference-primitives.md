# Plan 006: Port the reference component recipes into the Parallax primitive set (+ Tabler icons)

> **Executor instructions**: Follow this plan step by step. Run every
> verification command and confirm the expected result before moving on. On any
> STOP condition, stop and report. When done, update this plan's row in
> `plans/README.md`.
>
> **Reference project**: the operator-designated local reference console. Its
> name must NEVER appear in this repository. Resolve its path from the
> git-ignored pointer: `REF_ROOT="$(cat plans/.reference-root)"` (STOP if
> missing). Reference pinned at its commit `9f028d7`. Before every commit run
> the leak check from `plans/README.md` §Reference.
>
> **Drift check (run first)**: `git diff --stat ad9115d..HEAD -- ui/src/components/ui ui/components.json ui/package.json`
> Plan 005 must already be DONE (styles.css carries the new tokens). If
> `ui/src/components/ui/*` changed beyond plan 005's series, compare against
> "Current state" before proceeding; mismatch = STOP.

## Status

- **Priority**: P1
- **Effort**: L
- **Risk**: MED
- **Depends on**: plans/005-reference-design-tokens.md
- **Category**: tech-debt (design system)
- **Planned at**: commit `ad9115d`, 2026-07-03

## Why this matters

The reference console's look lives mostly in its shadcn-derived primitives: pill buttons with
press-scale, tinted badges with per-hue shadows, borderless `rounded-3xl corner-squircle`
cards, tables elevated by shadow with dense rows, tabs as pill-in-track, inputs that are
shadows in light / hairline borders in dark. Parallax has the same component *names* (both
codebases are shadcn on Base UI) but stock/old recipes. Porting the recipes — file by file,
from a same-stack reference — is the highest-leverage visual step after tokens. This plan
also switches the icon system to `@tabler/icons-react` (filled-first), which every later plan
uses.

## Current state

- Parallax primitives (22 files) in `ui/src/components/ui/`: avatar, badge, breadcrumb,
  button, card, chart, checkbox, drawer, dropdown-menu, input, label, select, separator,
  sheet, sidebar, skeleton, sonner, table, tabs, toggle, toggle-group, tooltip. All Base UI
  (`render` prop, not `asChild`), `cn()` from `@/lib/utils`.
- Missing primitives the reference has and later plans need: `empty`, `kbd`, `spinner`,
  `loader`, `scroll-area`, `dialog`, `item`, `field`, `input-group`, `popover`, `switch`,
  `alert-dialog`, `progress` (also `calendar`, `combobox`, `native-select` — deferred).
- `ui/components.json:13` → `"iconLibrary": "lucide"`; `lucide-react` in `ui/package.json`;
  no `@tabler/icons-react`.
- Parallax `ui/src/components/ui/sidebar.tsx:311` inset recipe is stock
  (`rounded-xl … shadow-sm`) — the reference's is `shadow-(--custom-shadow) … rounded-3xl
  corner-squircle` (its `sidebar.tsx:305-316`).
- Reference component library (**read each file before porting it**):
  `$REF_ROOT/packages/ui/src/components/` at commit `9f028d7`. License Apache-2.0 (its
  `packages/ui/package.json`), same as Parallax — porting authorized by the operator
  (2026-07-03 directive: copy the reference look 1:1). When porting, rewrite its workspace
  imports (`@<scope>/ui/lib/utils`, `@<scope>/ui/components/*`, `@<scope>/ui/hooks/*`) to
  Parallax paths (`@/lib/utils`, `@/components/ui/*`, `@/hooks/*`) — never keep the scoped
  package name.
- Signature recipes you must end up with (verified excerpts from the reference):
  - Button base (`button.tsx:8`): `rounded-full … text-sm font-medium … active:scale-[0.97]
    focus-visible:ring-[1.5px] focus-visible:ring-ring/50 …
    [&_svg:not([class*='size-'])]:size-3.5`; variant `default` = `bg-neutral-800
    dark:bg-neutral-200 shadow-[var(--custom-shadow-primary)]`; `outline` =
    `shadow-(--custom-shadow) bg-background … dark:bg-input/10`; sizes `h-8/h-7/h-6/h-9`,
    `icon size-8`, `icon-xs size-6`, `icon-sm size-8`, `icon-lg size-10`.
  - Badge (`badge.tsx`): `capitalize`, sizes `md h-5 rounded-full px-2 text-xs
    [&>svg]:size-3!` / `lg h-6 rounded-full px-2.5 text-sm`, hue variants
    `bg-{hue}-500/10 text-{hue}-700 shadow-[var(--custom-shadow-{hue})] dark:bg-{hue}-500/15
    dark:text-{hue}-300` for green/blue/amber/orange/emerald/rose/violet, plus
    default/secondary/destructive/outline.
  - Card (`card.tsx:15`): `shadow-(--custom-shadow) … rounded-3xl corner-squircle bg-card
    py-6 text-sm` + `size="sm"` (`py-5 gap-2`), CardTitle `font-heading text-base
    font-medium`, CardHeader/Content `px-6` (sm → `px-5`).
  - Table (`table.tsx`): container `rounded-xl corner-squircle shadow-(--custom-shadow)`,
    table `bg-card/40`, header `bg-muted/50`, rows `border-b border-[#EBEBEB]
    dark:border-[#1E1E1E]` (hardcoded hex is intentional in the reference), cell density
    default `h-11 px-5` / compact `h-10 px-3`, `align="right"` ⇒ `text-right tabular-nums`,
    `TableRow interactive` ⇒ pointer + `hover:bg-muted/50` + focus ring + Enter/Space
    activation, optional sticky header (`sticky top-0 bg-card/95 backdrop-blur`).
  - Tabs (`tabs.tsx`): list `rounded-2xl corner-squircle p-[3px] bg-muted`, trigger active
    `data-active:bg-background data-active:shadow-(--custom-shadow)`; `line` variant with
    `after:` underline bar.
  - Input (`input.tsx:12`): `h-9 rounded-xl corner-squircle shadow-(--custom-shadow)
    dark:shadow-none dark:border dark:border-border/50 bg-transparent … dark:bg-input/30` —
    the recurring dark idiom (shadow in light → hairline border in dark) also applies to
    select trigger, kbd, input-group, empty media.
  - Tooltip: inverted `bg-foreground text-background rounded-2xl corner-squircle text-xs`,
    provider `delay={0}`.
  - Empty (`empty.tsx:10`): `rounded-4xl corner-squircle border-dashed border
    dark:border-border/50 p-12 py-16 text-center` (dashed border = sanctioned exception);
    EmptyTitle `font-heading text-sm font-medium tracking-tight`.
  - Skeleton: `animate-pulse rounded-2xl corner-squircle bg-muted`.
  - Kbd: `h-5 rounded-[14px] corner-squircle shadow-(--custom-shadow) dark:shadow-none
    dark:border dark:border-border/50 text-xs font-medium`.
  - Dialog: overlay `bg-black/20 supports-backdrop-filter:backdrop-blur-[2px]`; content
    `rounded-[40px] corner-squircle p-6 gap-6 shadow-(--custom-shadow) sm:max-w-md` with
    `data-open:animate-in fade-in-0 zoom-in-95`.
  - Sidebar: constants `SIDEBAR_WIDTH "14rem"`, `SIDEBAR_WIDTH_ICON "3rem"`, cookie
    `sidebar_state`, keyboard shortcut `b`; `SidebarInset` `shadow-(--custom-shadow) …
    md:peer-data-[variant=inset]:rounded-3xl corner-squircle`; `SidebarMenuButton`
    `rounded-xl corner-squircle [&_svg]:size-4.5` with `data-active:bg-sidebar-accent`.
  - Chart (`chart.tsx`): container `aspect-video text-xs` + Recharts overrides (ticks
    `fill-muted-foreground`, grid `stroke-border/50`); tooltip content `rounded-lg border
    border-border/50 bg-background px-2.5 py-1.5 text-xs shadow-xl`, values `font-mono
    tabular-nums`.

## Commands you will need

From `/Users/donbeave/Projects/tailrocks/parallax-project/parallax/ui`:

| Purpose   | Command                 | Expected |
|-----------|-------------------------|----------|
| Add deps  | `rtk bun add @tabler/icons-react` | exit 0 |
| Typecheck | `rtk bun run typecheck` | exit 0 |
| Lint      | `rtk bun run lint`      | exit 0 |
| Tests     | `rtk bun run test`      | exit 0 |
| Build     | `rtk bun run build`     | exit 0 |
| Leak check (repo root) | see plan 005 Commands table | no output |

## Scope

**In scope**:
- `ui/src/components/ui/*` (restyle existing 22; add: empty.tsx, kbd.tsx, spinner.tsx,
  loader.tsx, scroll-area.tsx, dialog.tsx, item.tsx, field.tsx, input-group.tsx, popover.tsx,
  switch.tsx, alert-dialog.tsx, progress.tsx — ported from `$REF_ROOT/packages/ui`)
- `ui/components.json` (iconLibrary)
- `ui/package.json` / `bun.lock` (add tabler; keep `lucide-react` installed for now — routes
  still import it until plans 007-017; plan 018 removes it)

**Out of scope**:
- `ui/src/components/*.tsx` app composites (kpi-card, parallax-shell, …) — plans 007-008.
- All `ui/src/routes/*` — later plans. Routes must keep compiling: do not rename exported
  component names or remove variants/props that routes use today without checking usages
  (`grep -rn "<Component" ui/src/routes ui/src/components`); where a route uses a prop the
  new recipe drops, keep a compat prop and note it.
- `calendar`, `combobox`, `command`, `native-select` — deferred to the plans that need them
  (013 range picker, 018 dashboards).

## Git workflow

`main`; Conventional Commits (suggested: one commit per step,
`style(ui): port reference <area> primitives`); `git commit -s` +
`Co-authored-by: Claude <noreply@anthropic.com>`; leak check before each commit.

## Steps

### Step 1: Icon system → Tabler

1. `rtk bun add @tabler/icons-react`
2. `ui/components.json`: `"iconLibrary": "tabler"` (keeps future `shadcn add` output aligned;
   if the CLI rejects the value, leave components.json unchanged and note it in the report).
3. Record the convention in a short `ui/src/components/ui/README.md`: Tabler only, **prefer
   `*Filled`** variants, default glyph size `size-3.5`, `size-3` dense, `size-4`
   menus/inputs, `size-4.5` sidebar; fall back to outline when no `*Filled` export exists.

**Verify**: `rtk bun run typecheck` → exit 0.

### Step 2: Port the existing 22 primitives

For each file in `ui/src/components/ui/` that has a counterpart in
`$REF_ROOT/packages/ui/src/components/` (button, badge, card, table, tabs, input, select,
dropdown-menu, separator, sheet, sidebar, skeleton, tooltip, avatar, breadcrumb, checkbox,
drawer, label, sonner, toggle, toggle-group, chart):

1. Read the reference file in full.
2. Replace the Parallax file's **class recipes and variant/size CVA definitions** with the
   reference's, adjusting imports as described in Current state (workspace scope → `@/…`;
   icons from `@tabler/icons-react`; `use-mobile` already exists at `@/hooks/use-mobile`).
3. Keep Parallax-only exports/props that routes rely on (grep before dropping anything).
   Where the reference file has extra features (Table `maxHeight`/sticky/`interactive`/
   `align`; Button `xs`/`icon-*` sizes; Tabs `line` variant), bring them — later plans use
   them.
4. Order (compile-safety): button → badge → card → separator → skeleton → tooltip → input →
   select → dropdown-menu → tabs → table → sheet → drawer → avatar → breadcrumb → checkbox →
   label → toggle/toggle-group → sonner → chart → sidebar (last, biggest; verify the
   `collapsible="icon"`/offcanvas modes, cookie persistence, Cmd/Ctrl+B shortcut, and the
   inset recipe come across — the Parallax shell relies on `collapsible="icon"`).

**Verify after each file**: `rtk bun run typecheck` → exit 0.
**Verify after all**: `rtk bun run build` → exit 0; `grep -rln "corner-squircle"
ui/src/components/ui | wc -l` → ≥ 15; `grep -rln "custom-shadow" ui/src/components/ui |
wc -l` → ≥ 10.

### Step 3: Add the missing primitives

Port these reference files new into `ui/src/components/ui/` (same import adjustments):
`empty.tsx`, `kbd.tsx`, `spinner.tsx`, `loader.tsx`, `scroll-area.tsx`, `dialog.tsx`,
`item.tsx`, `field.tsx`, `input-group.tsx`, `popover.tsx`, `switch.tsx`, `alert-dialog.tsx`,
`progress.tsx`. If one pulls an npm dependency Parallax lacks (check imports; none of these
should need cmdk/embla/day-picker), STOP and report rather than adding an unplanned dep.

**Verify**: `rtk bun run typecheck && rtk bun run build` → exit 0.

### Step 4: Both-themes smoke check

`rtk bun run dev`; open Issues, Traces, Logs, SQL in dark and light (toggle the `dark` class
on `<html>` manually — the UI switcher arrives in plan 007). Buttons must be pills, cards
borderless-rounded with soft shadow, tables shadow-elevated. Old app composites (KPI cards
etc.) may still look off — expected until plans 007+.

## Test plan

- Existing `rtk bun run test` must stay green.
- Add `ui/src/components/ui/__tests__/primitives.test.tsx` (vitest + @testing-library/react,
  jsdom): render `<Button>`, `<Badge variant="rose">`, `<Card>`, `<Empty>`, `<Kbd>` and
  assert key classes (`rounded-full` on button, `corner-squircle` on card) — a cheap
  regression net for the recipes. There are no existing component tests to copy; keep it
  minimal.

## Done criteria

- [ ] typecheck / lint / test / build all exit 0
- [ ] `ls ui/src/components/ui` includes empty.tsx kbd.tsx spinner.tsx loader.tsx
      scroll-area.tsx dialog.tsx item.tsx field.tsx input-group.tsx popover.tsx switch.tsx
      alert-dialog.tsx progress.tsx
- [ ] `grep -rn "lucide-react" ui/src/components/ui` → no matches (primitives fully Tabler;
      routes may still use lucide until their plans land)
- [ ] `grep -rn "@<scope>" ui/src/components/ui` → no leftover reference workspace imports
      (grep for `/ui/lib/utils"` imports not starting with `@/`)
- [ ] Leak check → no output
- [ ] New primitives test passes
- [ ] `plans/README.md` row updated

## STOP conditions

- `plans/.reference-root` missing, reference absent at that path, or a listed component file
  absent there.
- A route breaks because a primitive dropped an export/prop and the compat shim isn't
  obvious — report the usage list instead of redesigning the route (that's a later plan).
- A ported file needs a new npm dependency not named in this plan.
- Typecheck failures you cannot resolve within the primitive file itself.

## Maintenance notes

- These files are now derived from the reference console; when improving a recipe, diff
  against `$REF_ROOT/packages/ui/src/components/<name>.tsx` rather than upstream shadcn.
- The table's hardcoded divider hexes (`#EBEBEB`/`#1E1E1E`) are intentional reference
  behavior; don't "fix" them to tokens without a matching reference change.
- `lucide-react` remains only for not-yet-redesigned routes; plan 018 removes it — do not add
  new lucide imports anywhere.
