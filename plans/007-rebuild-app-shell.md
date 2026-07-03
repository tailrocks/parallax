# Plan 007: Rebuild the app shell — inset floating console, reference nav grammar, collapsible kept

> **Executor instructions**: Follow step by step; run every verification; STOP
> conditions are binding. Update `plans/README.md` when done.
>
> **Reference project**: the operator-designated local reference console — its
> name must NEVER appear in this repository. `REF_ROOT="$(cat plans/.reference-root)"`
> (git-ignored pointer; STOP if missing). Reference pinned at its commit
> `9f028d7`. Leak check (plans/README.md §Reference) before every commit.
>
> **Drift check (run first)**: `git diff --stat ad9115d..HEAD -- ui/src/components/parallax-shell.tsx ui/src/components/page-heading.tsx ui/src/components/route-fallbacks.tsx ui/src/router.tsx ui/src/routes/__root.tsx`
> Plans 005+006 must be DONE. If parallax-shell.tsx diverges from the excerpts
> below beyond those plans' effects, STOP.

## Status

- **Priority**: P1
- **Effort**: L
- **Risk**: MED
- **Depends on**: plans/005, plans/006
- **Category**: tech-debt (design/UX)
- **Planned at**: commit `ad9115d`, 2026-07-03

## Why this matters

The shell is the frame every screen sits in. The reference console reads "product console":
the sidebar is a quiet canvas (`bg-sidebar`) and the content floats on it as an inset rounded
card (`SidebarInset` + `shadow-(--custom-shadow)` + `rounded-3xl corner-squircle`), with a
per-page header inside the content column — no full-width sticky top bar. Nav items pair an
outline icon (inactive) with a filled icon (active) inside per-section colored squircle chips.
Parallax currently renders an edge-to-edge bordered layout with a sticky header bar, rainbow
nav tiles, and a `text-3xl` hero heading on every page. **One Parallax feature must survive
the port: the collapsible sidebar** (`collapsible="icon"`, trigger button, Cmd/Ctrl+B) — the
operator explicitly keeps it (the reference itself ships no trigger).

## Current state

- `ui/src/components/parallax-shell.tsx` (139 lines, verified at `ad9115d`):
  - `:26-69` `NAV`: 7 items (Issues, Traces, Logs, Services, Runs, Dashboards, SQL), lucide
    icons, per-item `iconClass` brand-color tiles via inline `color-mix`.
  - `:74-78` `<SidebarProvider><Sidebar collapsible="icon" className="border-sidebar-border/80 bg-sidebar/95">`
    — default variant (NOT inset).
  - `:79-94` header: 3-dot brand mark + "Parallax" wordmark (`font-heading text-lg
    font-semibold`, hidden when icon-collapsed).
  - `:103-115` menu buttons: `h-10 rounded-xl … data-active:bg-sidebar-accent/90
    data-active:shadow-[inset_0_0_0_1px_var(--sidebar-border)]`, icon tile `size-6 rounded-lg
    text-white`.
  - `:125-133` sticky `h-16` header: `border-b border-border/50 bg-background/70
    backdrop-blur-sm`, `SidebarTrigger`, separator, `.parallax-pill` status ("Local" +
    `127.0.0.1:4000`, static).
  - `:134` `<main className="flex-1 overflow-auto p-4 sm:p-6">`.
- `ui/src/components/page-heading.tsx`: eyebrow + `text-3xl` title + description + action;
  used by 10 of 11 routes.
- `ui/src/components/route-fallbacks.tsx`: RouteErrorPanel / RoutePendingPanel /
  RouteNotFoundPanel — styled with brand tiles; wired only on the root route
  (`__root.tsx:37-39`); `ui/src/router.tsx` sets no `defaultErrorComponent` /
  `defaultPendingComponent`, so child loader errors fall back to TanStack defaults
  (bug: the shell disappears when a child route's API load fails).
- Reference shell sources (read all, at `9f028d7`):
  - `$REF_ROOT/apps/web/src/components/app/app-shell.tsx` — `ShellBody` (`:261-382`):
    `SidebarProvider className="relative h-svh min-h-0 overflow-hidden"`;
    `<Sidebar variant="inset">`; two nav groups (second labeled); footer;
    `SidebarInset className="min-h-0 overflow-hidden"`; `main className="min-h-0 flex-1
    overflow-y-auto"`; content `div className="flex flex-col gap-6 p-10 2xl:p-16 max-w-380
    mx-auto"` (`:347`). `NavIcon` outline/filled cross-fade (`:206-238`: wrapper `grid
    size-4.5 place-items-center [&_svg]:size-full!`, both icons stacked in
    `[grid-area:1/1]`, opacity tween `duration-100`); `isActive` exact-or-prefix
    (`:191-194`).
  - `$REF_ROOT/apps/web/src/components/app/nav.ts` — nav model
    (`{href,label,icon,activeIcon,iconClassName}`), chip recipe e.g.
    `bg-rose-100 dark:bg-rose-950 rounded-xl p-0.5 corner-squircle text-rose-500
    shadow-[inset_0_0_0_1px_rgba(244,63,94,0.14),0_2px_6px_-2px_rgba(244,63,94,0.25)]
    dark:shadow-(--custom-shadow)`; one item uses a custom brown set
    `bg-[#ede0d4] dark:bg-[#2e211b] … text-[#8b5e34] dark:text-[#c9a888]
    shadow-[inset_0_0_0_1px_rgba(139,94,52,0.14),0_2px_6px_-2px_rgba(139,94,52,0.25)]`.
  - `$REF_ROOT/apps/web/src/components/app/page-parts.tsx:53-116` — `PageHeader`:
    root `flex flex-wrap items-end justify-between gap-4`; `h1` = `flex items-center gap-2
    text-base font-medium tracking-tight` with optional section icon `size-4.5`; back mode
    renders `[icon] Label › Title` (`IconChevronRight` `size-4 shrink-0
    text-muted-foreground/50 stroke-[1.5px]`, parent link `text-muted-foreground
    transition-colors hover:text-foreground`); description `text-sm text-muted-foreground`;
    actions right (`flex items-center gap-2`).
  - `$REF_ROOT/apps/web/src/components/app/route-header.tsx` — thin wrapper adding nav-icon
    lookup + optional range-picker slot.
  - `$REF_ROOT/apps/web/src/components/theme-switcher.tsx` — 3-way segmented pill
    (system/light/dark), spring thumb `{type:"spring", stiffness:400, damping:38}`;
    `mounted` gate for SSR.
- Sidebar primitive after plan 006 already carries the reference inset recipe
  (`shadow-(--custom-shadow) … md:peer-data-[variant=inset]:rounded-3xl corner-squircle`) and
  keeps `collapsible="icon"` support, cookie persistence, and the Cmd/Ctrl+B shortcut.

## Commands you will need

From `ui/`: `rtk bun run typecheck` / `lint` / `test` / `build` (all exit 0);
`rtk bun run dev` (serves :3000). New dep allowed in this plan: `rtk bun add motion`
(theme-switcher thumb / cross-fades; the reference uses `motion` — if you prefer zero-dep,
implement the thumb with CSS transitions and note the deviation). Leak check: see plan 005.

## Scope

**In scope**:
- `ui/src/components/parallax-shell.tsx` (rewrite)
- NEW `ui/src/components/nav.ts`, `ui/src/components/nav-icon.tsx`,
  `ui/src/components/page-header.tsx` (+ optional `route-header.tsx`),
  `ui/src/components/theme-switcher.tsx`
- `ui/src/components/route-fallbacks.tsx` (restyle to new tokens/primitives)
- `ui/src/router.tsx` (`defaultErrorComponent`, `defaultPendingComponent`,
  `defaultNotFoundComponent`)
- `ui/src/routes/__root.tsx` (wiring only)
- `ui/src/components/page-heading.tsx` — keep compiling as a thin adapter over PageHeader
  (routes still import it until their plans land); mark deprecated.

**Out of scope**:
- Route bodies (`ui/src/routes/*` content) — do not restyle pages here; they merely keep
  rendering inside the new frame.
- KPI-strip removal (plans 011-018, per page).
- Overview route + its nav item (plan 013 adds `/overview` and flips `/`).

## Git workflow

`main`; Conventional Commits (`style(ui): rebuild shell as inset console`); `git commit -s`;
trailer `Co-authored-by: Claude <noreply@anthropic.com>`; leak check before each commit.

## Steps

### Step 1: Nav model (`ui/src/components/nav.ts`)

Port the reference `NavItem` shape (`href/label/icon/activeIcon/iconClassName`) with the chip
recipe template `bg-{hue}-100 dark:bg-{hue}-950 rounded-xl p-0.5 corner-squircle
text-{hue}-500 shadow-[inset_0_0_0_1px_rgba(R,G,B,0.14),0_2px_6px_-2px_rgba(R,G,B,0.25)]
dark:shadow-(--custom-shadow)`. Parallax mapping (two groups):

Primary group (unlabeled): Issues rose `rgba(244,63,94,…)` (IconBug/IconBugFilled) · Traces
**the brown set verbatim** (above; IconAffiliate/IconAffiliateFilled) · Logs orange
`rgba(249,115,22,…)` · Services emerald `rgba(16,185,129,…)` · a slot reserved for Overview
(sky `rgba(14,165,233,…)`; plan 013 prepends it).
Group "Workspace" (`SidebarGroupLabel`): Runs violet `rgba(139,92,246,…)` · Dashboards
fuchsia `rgba(217,70,239,…)` · SQL yellow `rgba(234,179,8,…)`.

Icon pairs: Tabler outline+`*Filled`. Suggested: Logs `IconArticle(+Filled)`, Services
`IconServer(+Filled)`, Runs `IconTerminal2` (no filled variant exists — outline for both
states is the sanctioned fallback), Dashboards `IconLayoutDashboard(+Filled)`, SQL
`IconDatabase(+Filled)`. Rule: if TypeScript says a `*Filled` export doesn't exist, use the
outline icon for both states — never invent names. Export a `navItem(href)` lookup.

**Verify**: `rtk bun run typecheck` → exit 0.

### Step 2: NavIcon cross-fade (`ui/src/components/nav-icon.tsx`)

Port the reference `NavIcon` (app-shell.tsx:206-238) verbatim (adjust imports): wrapper
`grid size-4.5 place-items-center [&_svg]:size-full!` + two stacked icons in
`[grid-area:1/1]` with `transition-opacity duration-100` crossfade on `active`.

### Step 3: Rewrite `parallax-shell.tsx`

Target structure (reference `ShellBody` adapted; TanStack `Link`/`useRouterState` instead of
Next):

```tsx
<SidebarProvider className="relative h-svh min-h-0 overflow-hidden">
  <Sidebar variant="inset" collapsible="icon">
    <SidebarHeader>
      {/* row: keep the existing 3-dot Parallax mark + wordmark (wordmark hidden via
          group-data-[collapsible=icon]:hidden) + <SidebarTrigger/> on the right */}
    </SidebarHeader>
    <SidebarContent>
      <SidebarGroup>            {/* primary items: SidebarMenu → NavIcon, isActive */}
      <SidebarGroup>            {/* <SidebarGroupLabel>Workspace</SidebarGroupLabel> */}
    </SidebarContent>
    <SidebarFooter>
      {/* ThemeSwitcher (step 5) + status pill (step 6); both must degrade in icon-collapsed
          mode: group-data-[collapsible=icon]:hidden or reduce to icons */}
    </SidebarFooter>
  </Sidebar>
  <SidebarInset className="min-h-0 overflow-hidden">
    <main className="min-h-0 flex-1 overflow-y-auto">
      <div className="mx-auto flex max-w-380 flex-col gap-6 p-10 2xl:p-16">
        {children}
      </div>
    </main>
  </SidebarInset>
</SidebarProvider>
```

Details: `isActive` = exact-or-prefix match (port the reference helper); menu buttons are the
plain plan-006 `SidebarMenuButton` (no custom h-10/rainbow classes; keep the
`tooltip={label}` prop so icon-collapsed mode shows labels on hover — the retained Parallax
feature). DELETE: the sticky h-16 header bar, its border, and the `.parallax-pill` usage
there. The `SidebarTrigger` lives in the sidebar header row (visible in both states) and
Cmd/Ctrl+B still works (provider ships it).

**Verify**: `rtk bun run typecheck` → exit 0; dev-serve: content floats as a rounded card on
the sidebar canvas (`m-2 ml-0 rounded-3xl` + soft shadow, `ml-2` when collapsed); collapse
via trigger AND Cmd/Ctrl+B; state survives reload (cookie); mobile (<768px) opens the Sheet.

### Step 4: PageHeader + adapter

Create `ui/src/components/page-header.tsx` porting the reference `PageHeader`
(page-parts.tsx:53-116): props `title, titleLeading, titleTrailing, description, actions,
back {href,label,icon,iconClassName}, icon, iconClassName`; `h1` `text-base font-medium
tracking-tight`; back mode `[icon] Label › Title` with `IconChevronRight size-4
text-muted-foreground/50 stroke-[1.5px]`; actions `flex items-center gap-2`. Then rewrite
`ui/src/components/page-heading.tsx` as a deprecated adapter that renders `PageHeader`
(map `eyebrow`→dropped, `title`, `description`, `action`→`actions`) so all existing routes
keep compiling and instantly get the calm header.

**Verify**: `rtk bun run typecheck` → exit 0; `grep -rn "text-3xl" ui/src/components
ui/src/routes` → no matches.

### Step 5: ThemeSwitcher

Port the reference `theme-switcher.tsx` (3-way segmented system/light/dark pill; `useTheme`
from `next-themes` — provider landed in plan 005; `mounted` gate). Mount in the sidebar
footer. If you skipped `motion`, a CSS-transition thumb is acceptable — note it.

**Verify**: dev-serve; switching persists across reload; both themes render correctly.

### Step 6: Status pill, restyled and honest

Rebuild the "Local · 127.0.0.1:4000" pill on plan-006 primitives (chip with
`shadow-(--custom-shadow)`, no border): green dot + "Local" + mono address. Optional (S):
fetch `{ health }` from `/graphql` once on mount and render a rose dot + "offline" when
unreachable — the GraphQL `health` query exists and is currently unused. Place in the sidebar
footer above the ThemeSwitcher.

### Step 7: Router-level fallbacks (shell must survive API failure)

In `ui/src/router.tsx` pass `defaultErrorComponent: RouteErrorPanel`,
`defaultPendingComponent: RoutePendingPanel`, `defaultNotFoundComponent: RouteNotFoundPanel`.
Restyle `route-fallbacks.tsx` on the new primitives: ErrorPanel = `Empty` with rose-tinted
`EmptyMedia` icon chip, the safe error message, mono detail block, and a Retry
`Button variant="outline"`; PendingPanel = header-shaped `Skeleton` stack + `Spinner`;
NotFoundPanel = `Empty` with muted icon. No `--brand-*` references remain in the file.

**Verify**: `rtk bun run typecheck` → exit 0. With the API server stopped, open `/issues`:
the styled error panel renders **inside the shell** (sidebar still visible), not a bare
TanStack error page.

### Step 8: Full gate + both-theme sweep

`rtk bun run typecheck && rtk bun run lint && rtk bun run test && rtk bun run build` → all 0.
Click all 7 nav items in dark + light; collapse/expand; mobile sheet. Leak check → no output.

## Test plan

- Add `ui/src/components/__tests__/shell.test.tsx`: render `PageHeader` with `back` and
  assert the breadcrumb link + `text-base` title class; import the nav model and assert every
  item has `icon` and `activeIcon` defined.
- `rtk bun run test` → all pass.

## Done criteria

- [ ] typecheck / lint / test / build exit 0
- [ ] `grep -n "variant=\"inset\"" ui/src/components/parallax-shell.tsx` → match; and
      `grep -n "collapsible=\"icon\"" …` → match (feature retained)
- [ ] `grep -rn "lucide-react" ui/src/components/parallax-shell.tsx ui/src/components/nav.ts` → none
- [ ] `grep -rn "parallax-pill\|brand-" ui/src/components/parallax-shell.tsx` → none
- [ ] Sticky h-16 header bar gone; content container is `gap-6 p-10 2xl:p-16 max-w-380`
- [ ] API-down renders styled error inside shell
- [ ] Leak check → no output
- [ ] `plans/README.md` row updated

## STOP conditions

- `plans/.reference-root` missing or reference absent at that path.
- Plan 006's sidebar primitive lacks the inset recipe or `collapsible="icon"` support
  (means 006 incomplete — report, don't patch here).
- TanStack Router rejects the `default*Component` option names (version drift) — report the
  actual API, don't guess.
- The `motion` dep conflicts with the build.
- Any route fails to compile after the PageHeading adapter — list the routes, don't restyle
  them here.

## Maintenance notes

- Plan 013 prepends the Overview nav item (sky chip) and flips `/` to `/overview`.
- Screen plans (011-018) each replace their route's `PageHeading` usage with `PageHeader`
  directly; when the last usage is gone, delete `page-heading.tsx` (tracked in plan 018).
- Anything added to the sidebar footer must handle `collapsible="icon"` (hide or iconify).
