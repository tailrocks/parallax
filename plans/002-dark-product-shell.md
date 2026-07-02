# Plan 002: Rebuild App Shell Around Dark Product Surface

> **Executor instructions**: Follow this plan step by step. Run every verification command and confirm expected result before next step. If any STOP condition occurs, stop and report. When done, update `plans/README.md`.
>
> **Drift check (run first)**: `rtk git diff --stat 8dde008..HEAD -- ui/src/components/parallax-shell.tsx ui/src/routes/__root.tsx ui/src/styles.css ui/src/logo.svg`
> If any in-scope file changed since this plan was written, compare "Current state" excerpts against live code before proceeding; on mismatch, STOP.

## Status

- **Priority**: P1
- **Effort**: M
- **Risk**: MED, shell affects every page
- **Depends on**: `plans/001-reference-style-theme-tokens.md`
- **Category**: direction
- **Planned at**: commit `8dde008`, 2026-07-03

## Why This Matters

the visual reference's strongest UI signal is not a table style; it is the full product frame: dark page, compact left nav, translucent top bar, overlapping dot brand, pill controls, and dense content area. Parallax currently has a stock shadcn sidebar with a text-only logo and plain white page surface. This plan makes every Parallax route feel like one coherent product before page-level redesign starts.

## Current State

- `ui/src/components/parallax-shell.tsx:17-25` defines nav labels with no icons.
- `ui/src/components/parallax-shell.tsx:30-67` renders default shadcn Sidebar, text `Parallax`, and a thin header with `local · http://127.0.0.1:4000`.
- `ui/src/routes/__root.tsx:44-56` always wraps routes in `ParallaxShell` and always renders TanStack devtools.

Relevant current excerpt:

```tsx
// ui/src/components/parallax-shell.tsx:31
<Sidebar collapsible="icon">
  <SidebarHeader>
    <div className="px-2 py-1.5 text-base font-semibold tracking-tight group-data-[collapsible=icon]:hidden">
      Parallax
    </div>
  </SidebarHeader>
```

visual reference shell evidence gathered 2026-07-03:

- Header is sticky, 64px tall, `bg-background/70`, backdrop blur, subtle border.
- Nav buttons are compact rounded pills with muted text and hover muted background.
- Brand mark is three overlapping dots; color accents are blue and orange.
- Product screenshot uses a dark left rail with active item pill and colored square icon per nav item.

## Commands You Will Need

| Purpose | Command | Expected on success |
|---------|---------|---------------------|
| Typecheck | `rtk bun run typecheck` | exit 0 |
| Lint | `rtk bun run lint` | exit 0 |
| Build | `rtk bun run build` | exit 0 |
| Dev | `rtk bun run dev` | Vite local URL available |

## Scope

**In scope**:
- `ui/src/components/parallax-shell.tsx`
- `ui/src/routes/__root.tsx` only for devtools gating or body/html classes if needed
- `ui/src/styles.css` only if shell utilities from Plan 001 need small adjustments
- `ui/src/logo.svg` only if replacing with Parallax-owned mark

**Out of scope**:
- Route loader behavior
- Page data layout changes
- Package installs
- visual reference names/product labels

## Git Workflow

Work on current branch unless operator says otherwise. Commit style follows existing conventional commits. If committing, use `rtk git commit -s` and include `Co-authored-by: Codex <codex@openai.com>`.

## Steps

### Step 1: Add Parallax Brand Mark

In `ParallaxShell`, replace text-only header with a compact brand row:

- Three overlapping circles: neutral/white, brand blue, brand orange, adapted for Parallax.
- Text `Parallax` in display/sans semibold.
- Keep collapsed sidebar behavior: hide text when `group-data-[collapsible=icon]`.

Use plain JSX/CSS classes, not copied SVG from visual reference. If using `ui/src/logo.svg`, make it Parallax-owned.

**Verify**: `rtk bun run typecheck` -> exit 0.

### Step 2: Add Icons And Active Pills To Navigation

Use `lucide-react` icons in `NAV` entries. Suggested mapping:

- Issues: `Flame` or `Bug`
- Traces: `GitBranch`
- Logs: `ScrollText`
- Services: `Network`
- Runs: `TerminalSquare`
- Dashboards: `ChartNoAxesCombined`
- SQL: `Database`

Render icon + label in `SidebarMenuButton`. Active item should feel like visual reference screenshot: dark raised/active pill, colored icon tile, muted inactive rows. Use shadcn Sidebar primitives and `SidebarMenuButton`; do not build a custom nav from raw divs.

**Verify**: `rtk bun run lint` -> exit 0.

### Step 3: Make Shell Surface reference-like

Style `SidebarProvider`, `Sidebar`, `SidebarInset`, `header`, and `main` to create:

- Dark app background from Plan 001 tokens.
- Left sidebar as slightly raised/sunken panel with border-right.
- Header sticky/top, translucent `bg-background/70`, backdrop blur, `border-b border-border/50`.
- Main content max width only where pages need it; do not constrain all observability pages too tightly.
- Padding closer to reference product screenshot: desktop `p-6`, mobile `p-4`; no huge blank hero spacing.

Keep sidebar collapsible and mobile Sheet behavior intact.

**Verify**: `rtk bun run typecheck` -> exit 0.

### Step 4: Replace Static Endpoint Text With Product-Aware Status Pill

Replace `local · http://127.0.0.1:4000` with a compact pill:

- small status dot
- label `Local`
- mono endpoint `127.0.0.1:4000`

Use semantic colors. Do not add live health fetch in this plan; Plan 004 handles errors/loading.

**Verify**: `rtk bun run build` -> exit 0.

### Step 5: Gate Devtools Away From Normal Product View

In `ui/src/routes/__root.tsx`, render `TanStackDevtools` only in development and only when an env flag is enabled, or leave it hidden by default if repo convention already exists. Current always-on devtools weakens product feel.

Example condition:

```tsx
{import.meta.env.DEV && import.meta.env.VITE_PARALLAX_DEVTOOLS === "1" ? (
  <TanStackDevtools ... />
) : null}
```

**Verify**: `rtk bun run typecheck` -> exit 0.

## Test Plan

- No new unit tests required unless existing tests cover shell render.
- Manual visual check with dev server:
  - `/issues`, `/runs`, `/traces`, `/logs`, `/services`, `/dashboards`, `/sql` all keep shell.
  - Sidebar active state follows route.
  - Mobile width around 390px keeps nav accessible through Sheet.

## Done Criteria

- [ ] Shell defaults to dark product frame.
- [ ] Brand mark appears, Parallax-owned, reference-inspired but not copied asset.
- [ ] Nav has icons, active pills, and collapsed state still works.
- [ ] Header is sticky/translucent and status pill replaces raw endpoint text.
- [ ] Devtools not visible in normal dev/product screenshots unless explicitly enabled.
- [ ] `rtk bun run typecheck`, `rtk bun run lint`, and `rtk bun run build` exit 0.
- [ ] No out-of-scope files modified.

## STOP Conditions

- Existing shadcn Sidebar API changed from Base UI `render` composition.
- Plan 001 tokens are not present.
- Adding icons requires a dependency; `lucide-react` should already exist.
- You need to alter route loader/data behavior to finish shell styling.

## Maintenance Notes

Keep shell primitives reusable. Later page plans should not duplicate sidebar/header styling. Reviewers should inspect collapsed and mobile states, not only desktop expanded screenshots.
