# Plan 068: Command palette — global ⌘K navigation + entity quick-jump (pages, services, recent traces/runs, paste-an-id)

> **Executor instructions**: Follow this plan step by step. Run every
> verification command. On any STOP condition, stop and report. When done,
> update the status row in `plans/README.md`.
>
> **Drift check (run first)**:
> `git diff --stat ed5b10f..HEAD -- ui/src/components/parallax-shell.tsx ui/src/lib/nav.ts ui/src/components/ui ui/package.json`
> On mismatch with the excerpts below, STOP.

## Status

- **Priority**: P3
- **Effort**: M
- **Risk**: LOW (additive overlay; one new dependency)
- **Depends on**: none. Complements plan 053 (a11y) — coordinate if
  simultaneous.
- **Category**: direction (dx/ux)
- **Planned at**: commit `ed5b10f`, 2026-07-07

## Why this matters

Navigation is click-only through the sidebar: no command palette, no global
search, no keyboard entry point to "open trace <id> I just copied from a
terminal". The app already has the raw material — a typed nav model, a `Kbd`
primitive (used only decoratively in the SQL header), fast list resolvers
(`services`, `tracesPage`, `runs`) — and a console product's core audience
lives on the keyboard. The brief's global-UX section names the palette
explicitly. This plan adds a cmdk-style dialog on ⌘K/Ctrl+K: page
navigation, service jump, recent traces/runs, and paste-an-id routing
(trace/run/fingerprint), with the id-shape detection testable and honest.

## Current state

Verified at commit `ed5b10f`.

- Shell: `ui/src/components/parallax-shell.tsx` — sidebar nav + footer
  `StatusPill` + theme toggle; rendered from `__root.tsx`. No palette, no
  global search (`rtk grep -rn "cmdk\|CommandDialog" ui/src` → none).
- Nav model: `ui/src/lib/nav.ts` — typed entries with icons/routes
  (`nav.ts:27-95` per audit; verify the export names when implementing).
- Keyboard precedent: sidebar toggle ⌘B (`ui/src/components/ui/sidebar.tsx:99-109`,
  `SIDEBAR_KEYBOARD_SHORTCUT`); waterfall j/k; SQL ⌘Enter. `Kbd`/`KbdGroup`
  exists (`ui/src/components/ui/kbd.tsx`).
- shadcn convention: components under `ui/src/components/ui/*` are
  shadcn-on-Base-UI ports. shadcn's Command component wraps the `cmdk`
  package — check whether a Base-UI-compatible port already exists in the
  repo's shadcn registry version before hand-rolling (Step 1 decision).
- Data for quick-jump: `services` (list of names, `lib.rs:1610`),
  `tracesPage(sort:, limit:)` (`lib.rs:1384`), `runs` (`lib.rs:1711`),
  `issue(fingerprint)` (`lib.rs:1021`). Id shapes: trace = 32 hex chars,
  span = 16 hex, run id = the CLI's format (READ a real run id from
  `runs` fixtures/tests before pinning a regex — do not guess), fingerprint
  = hex-ish hash (check `fingerprint.rs` output length).
- Bun-only: dependency via `bun add cmdk` (lockfile `bun.lock`).

## Commands you will need

| Purpose | Command (from `ui/`) | Expected |
|---------|----------------------|----------|
| Add dep | `bun add cmdk` | bun.lock updated |
| Gates | `bun run typecheck && bun run lint && bun run test && bun run build` | all exit 0 |

## Scope

**In scope**:
- `ui/package.json` + `bun.lock` (cmdk)
- `ui/src/components/ui/command.tsx` (shadcn-style wrapper)
- `ui/src/components/console/command-palette.tsx` (the feature)
- `ui/src/components/parallax-shell.tsx` (mount + hotkey + a discreet
  trigger button showing `⌘K` via `Kbd`)
- `ui/src/lib/quick-jump.ts` (pure id-shape detection + result shaping)
- Tests

**Out of scope** (do NOT touch):
- Full-text search over telemetry (a search resolver) — the palette
  searches nav + service names client-side and routes ids; a server search
  is a future plan.
- Command actions beyond navigation (mutations like "resolve issue") —
  navigation only in v1.
- Global env/release filters — separate deferred item.
- Restyling the sidebar.

## Git workflow

- `main`, Conventional Commits, `git commit -s`, one
  `Co-authored-by: Claude <noreply@anthropic.com>` trailer. Push when done.

## Steps

### Step 1: Command primitive

Check the repo's shadcn/Base-UI setup for an existing Command port
(`ui/src/components/ui/` inventory + the shadcn registry the repo tracks).
If none: add `cmdk` and write `ui/src/components/ui/command.tsx` following
the repo's existing ui-component style (match `dialog.tsx`/`select.tsx`
conventions — imports, `cn`, data-slot attrs). Keep it a faithful thin
wrapper (Command, CommandDialog, CommandInput, CommandList, CommandGroup,
CommandItem, CommandEmpty, CommandShortcut).

**Verify**: `bun run typecheck && bun run lint` clean.

### Step 2: `quick-jump.ts`

```ts
export type IdGuess =
  | { kind: "trace"; id: string }
  | { kind: "span-in-trace"; id: string }   // 16-hex: explain, don't route blind
  | { kind: "run"; id: string }
  | { kind: "fingerprint"; id: string }
  | null
export function guessId(input: string): IdGuess
```

Rules from verified shapes (READ first: a real run id from the runs
fixtures, fingerprint length from `crates/parallax-core/src/fingerprint.rs`):
32-hex → trace; 16-hex → span (palette shows "Span id — open its trace via
Traces search" guidance, no blind route); run-id pattern → run; fingerprint
pattern → issue. Trim/lowercase input first. Ambiguous matches return the
higher-confidence kind only if patterns are disjoint — if run ids are
hex-shaped and collide with fingerprints, return BOTH as palette entries
(the type becomes a list; adjust) — decide from the real shapes and test it.

**Verify**: `bun run test` — table-driven tests over real-shaped fixtures.

### Step 3: The palette

`command-palette.tsx`:
- Open on ⌘K/Ctrl+K (window keydown listener following the
  `SIDEBAR_KEYBOARD_SHORTCUT` pattern at `sidebar.tsx:99-109`) and via the
  shell trigger button.
- Groups: **Pages** (from `nav.ts`, icons included, always shown filtered);
  **Services** (fetched once per open via the `services` query — lazy, not
  on app load); **Recent** (on open: `tracesPage(sort: START_DESC ... limit: 5)`
  — check the real sort enum values in `lib.rs:1384` region — and
  `runs(limit: 5)`; label with root name/command + relative time);
  **Id jump** (when `guessId` non-null: one entry "Open trace <id>" etc.).
- Selection navigates (TanStack router `useNavigate`) and closes. Loading
  and error states inline within the list (035 conventions; a failed fetch
  shows "Services unavailable" as a disabled item, never blocks Pages).
- Respect reduced-motion; focus-trap comes with the dialog primitive.

**Verify**: `bun run test` — component tests: opens on hotkey (fire
keydown), Pages filter works, id input surfaces the jump entry, Escape
closes. `bun run build` clean.

### Step 4: Shell mount + discoverability

`parallax-shell.tsx`: mount the palette once; add the trigger (a subtle
button in the sidebar footer area near `StatusPill`: "Search… ⌘K" with
`Kbd`). Don't steal ⌘K if a text input/textarea/contenteditable has focus
UNLESS the palette convention says otherwise — standard behavior: ⌘K opens
even from inputs; plain `/` shortcut is NOT added (SQL textarea conflict).

**Verify**: `bun run typecheck && bun run lint && bun run test && bun run
build` all clean; manual: hotkey + trigger both open; navigation works from
`/logs` (record).

## Test plan

- `quick-jump.test.ts` (Step 2 table).
- `command-palette.test.tsx` (Step 3 cases) — harness per the nearest
  existing component test.
- No Rust changes.

## Done criteria

- [ ] All UI gates clean; `bun.lock` contains cmdk (or the found existing
      primitive — stated)
- [ ] ⌘K/Ctrl+K opens; Pages/Services/Recent/Id-jump groups work
- [ ] `guessId` pinned to REAL id shapes with tests citing where each shape
      was verified
- [ ] Failed entity fetches degrade inline; Pages always work offline
- [ ] `plans/README.md` status row updated

## STOP conditions

- cmdk is incompatible with the repo's Base UI dialog/portal setup (focus
  or portal conflicts in practice) — report; hand-rolling a listbox is a
  scope change needing a decision.
- Run-id/fingerprint shapes collide such that `guessId` can't disambiguate
  and the both-entries fallback feels wrong in review — STOP and propose.
- A global keydown listener conflicts with an existing shortcut (test ⌘B
  still toggles the sidebar).

## Maintenance notes

- Future: a server-side entity search resolver would replace the
  client-side service filter and add issues-by-title — the palette's groups
  are the seam; keep fetch logic per-group.
- Plan 053's a11y sweep should include the palette once both land
  (focus-visible, SR labels on groups).
- Reviewer: hotkey handling must not break typing `k` in inputs (only the
  modifier combo listens), and the palette must never block render on a
  slow fetch.
