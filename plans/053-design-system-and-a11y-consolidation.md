# Plan 053: Design-system + a11y consolidation — one chip primitive, centralized formatters, keyboard access, dark-safe tokens

> **Executor instructions**: Follow step by step; run every verification. On
> any STOP condition, stop and report. When done, update the status row in
> `plans/README.md`.
>
> **Drift check (run first)**:
> `git diff --stat 408be17..HEAD -- ui/src/components ui/src/lib`
> On mismatch with the excerpts below, STOP.

## Status

- **Priority**: P3
- **Effort**: M
- **Risk**: MED (visual regressions across console pages)
- **Depends on**: best after plans/039 and /040 (they touch the same files;
  this is the cleanup pass)
- **Category**: tech-debt
- **Planned at**: commit `408be17`, 2026-07-07

## Why this matters

Small divergences are compounding into design drift: two relative-time
formatters, ad-hoc byte/time formatting scattered through components, three
hand-rolled "pill" styles beside the `Badge` primitive, hardcoded palette
colors without dark variants, a dead exported component, and two a11y gaps
on primary interactions (log rows unreachable by keyboard; an unlabeled
icon-only clear button). Each is minor; together they make every future UI
plan (041-052 all add chips/links/tables) inherit inconsistency. One
consolidation pass sets the patterns the rest of the program copies.

## Current state

Verified at commit `408be17`.

- **Duplicate relative-time formatter**: `ui/src/lib/api.ts:38-46`
  (`relativeTime`) duplicates `formatRelative` in `ui/src/lib/format.ts:33`.
  Find `relativeTime` importers before deleting
  (`rtk grep -rn "relativeTime" ui/src`).
- **Scattered inline formatting**:
  - `ui/src/components/logs-table.tsx:81` — inline
    `new Date(BigInt/1_000_000n).toISOString()` timestamp;
  - `ui/src/components/metric-strip.tsx:89` — bytes via
    `p.value / (1024 * 1024)`; `:135` — `toLocaleTimeString`;
  - `ui/src/routes/services.$service.tsx:203` region — similar inline
    formatting (verify on read).
  - `format.ts` has no `formatBytes`.
- **Pill divergence** (all should be `Badge` variants or one `Chip`):
  - `ui/src/components/logs-table.tsx:217` — trace chip:
    `rounded-full border border-border/70 px-2 py-1 font-mono text-[11px] ...`
  - `logs-table.tsx:262,271` — same shape, `text-xs`, different hover;
  - `ui/src/components/live-stream-panel.tsx:42` —
    `<code className="rounded-full bg-muted px-2 py-1 text-xs ...">`;
  - `ui/src/components/console/data-table.tsx` `ToggleChip` (~`:113`).
- **Hardcoded palette without dark variants**:
  `ui/src/components/console/span-kind.tsx:15-19` — `chip:` values
  (`text-sky-600` etc., no `dark:`); contrast `heat-cell.tsx:3-9` which
  ships dark variants. The `chip` field is **never consumed** anywhere, and
  `SpanKindBadge` (`span-kind.tsx:40-42`) is exported but unused
  (repo-wide grep: definition only).
- **A11y gaps**:
  - `ui/src/components/logs-table.tsx:188` — `<TableRow onClick>` opens the
    log document sheet; no `tabIndex`/`role`/key handler → keyboard users
    cannot inspect logs.
  - `ui/src/components/console/data-table.tsx:93-101` — SearchInput clear
    is an icon-only Button with `<IconX />` and no accessible name
    (contrast `ClearFiltersButton` `:161-164`, which has text). Note the
    dashboards delete button DOES have `sr-only` text
    (`dashboards.index.tsx:309`) — that's the pattern.
- Theme system: shadcn/Base UI tokens + Tailwind; `Badge` variants seen in
  use: `secondary`, `outline`, `rose`, `emerald`, `blue`, `violet`,
  `amber` (from `span-kind.tsx` + routes) — read
  `ui/src/components/ui/badge.tsx` for the full variant set before adding
  any.

## Commands you will need

| Purpose | Command (from `ui/`) | Expected |
|---------|----------------------|----------|
| Typecheck | `bun run typecheck` | exit 0 |
| Lint | `bun run lint` | exit 0 |
| Tests | `bun run test` | all pass |
| Build | `bun run build` | exit 0 |

## Scope

**In scope**:
- `ui/src/lib/api.ts` (delete `relativeTime`), `ui/src/lib/format.ts`
  (add `formatBytes`; ensure one blessed timestamp helper)
- `ui/src/components/logs-table.tsx`, `live-stream-panel.tsx`,
  `metric-strip.tsx`, `console/data-table.tsx`, `console/span-kind.tsx`
- `ui/src/routes/services.$service.tsx` (inline-format call sites only)
- A new tiny `ui/src/components/console/chip.tsx` ONLY if Badge variants
  can't express the mono-rounded chip (prefer Badge)
- test files

**Out of scope**:
- Any layout/visual redesign beyond consolidating identical-intent styles.
- New Badge variants beyond what the chip consolidation strictly needs.
- Toast system introduction (deferred from plan 035 — decide here ONLY if
  trivial; otherwise leave inline errors).
- Route files other than the named call sites.

## Git workflow

- `main`, Conventional Commits, `git commit -s`, one
  `Co-authored-by: Claude <noreply@anthropic.com>` trailer. Push when done.

## Steps

### Step 1: Formatters

1. Add `formatBytes(bytes: number): string` to `format.ts` (IEC units,
   1 decimal, e.g. `3.2 MiB`) + unit tests.
2. Route the inline sites through `format.ts`: logs-table `:81` (use the
   existing datetime helper — check `formatDateTime`'s output shape fits
   the tooltip use), metric-strip `:89` (`formatBytes`) and `:135` (a
   `formatTimeShort` helper if none fits), services.$service `:203` region.
3. Delete `api.ts:relativeTime`; migrate its importers to
   `formatRelative`.

**Verify**: `bun run test` (format tests green) + `bun run typecheck`;
`rtk grep -rn "relativeTime" ui/src` → no matches;
`rtk grep -n "1024" ui/src/components/metric-strip.tsx` → no matches.

### Step 2: One chip primitive

1. Decide against `badge.tsx`'s variant list: if a `mono`/`chip`-ish
   variant can express `rounded-full border font-mono text-xs px-2 py-1`,
   add ONE variant; else create `console/chip.tsx` with exactly that
   recipe (one component, `asChild`-friendly so it wraps `Link`).
2. Replace the three logs-table chips (`:217,262,271`), the
   live-stream-panel `<code>` pill (`:42`), and keep `ToggleChip` (it's a
   behavioral toggle, not a display chip — align its border radius/spacing
   tokens only).
3. Delete the unused `SpanKindBadge` export and the unused `chip:` field
   from `span-kind.tsx`'s map; while there, add `dark:` variants to any
   `bar` colors that lack them IF badge/bar colors show wrong in dark mode
   (check each against how heat-cell does it, `heat-cell.tsx:3-9`).

**Verify**: `bun run typecheck && bun run test` (waterfall/kit tests cover
span-kind — keep green); visual spot-check both themes on /logs and a trace
(record). `rtk grep -rn "SpanKindBadge" ui/src` → no matches.

### Step 3: Keyboard access

1. Log rows (`logs-table.tsx:188` region): make rows focusable+activatable
   — `tabIndex={0}`, `role="button"`, Enter/Space triggers the same
   handler, visible focus ring (existing focus-visible token). Ensure inner
   links still work (stopPropagation pattern).
2. SearchInput clear (`data-table.tsx:93-101`): add
   `<span className="sr-only">Clear search</span>` (copy the dashboards
   delete pattern).
3. Sweep for other icon-only buttons missing labels:
   `rtk grep -rn "size-\?icon" ui/src/components ui/src/routes` — add
   `sr-only`/`aria-label` where the button has no text child (list what you
   fixed in the report).

**Verify**: `bun run test` — extend the logs-table test: focus a row, press
Enter → document sheet opens (the test file already mounts LogsTable);
`bun run lint` clean.

## Test plan

- `format.test` additions: `formatBytes` cases (0, <1KiB, MiB rounding,
  huge); timestamp helper output pinned.
- Logs-table keyboard test (Step 3).
- Existing kit/waterfall/logs/shell tests stay green throughout — they are
  the visual-regression tripwire at the DOM level.

## Done criteria

- [ ] All four bun gates exit 0
- [ ] `relativeTime` gone from `api.ts`; zero inline `1024` math or ad-hoc
      `toISOString`/`toLocaleTimeString` in the named components
- [ ] One chip recipe (Badge variant or `chip.tsx`) used by all display
      pills named above; `SpanKindBadge` + `chip:` field deleted
- [ ] Log rows keyboard-operable; clear button labeled; icon-only sweep
      recorded
- [ ] Both-themes spot-check recorded
- [ ] `plans/README.md` status row updated

## STOP conditions

- `badge.tsx` variants are generated/synced from a design source (check
  file header) — don't hand-edit; use `chip.tsx` instead.
- The keyboard-row change breaks the existing logs-table test's row
  interaction assumptions in ways that suggest the Sheet trigger needs
  restructuring — report before rearchitecting the row.
- More than ~15 icon-only buttons show up in the sweep — fix the
  components' shared primitives instead of 15 call sites, and report.

## Maintenance notes

- Plans 041-052 add chips/badges/links — after this lands they must use
  the consolidated chip + `format.ts` helpers (their reviewers should
  check).
- Deferred: a toast system (inline errors remain the pattern); full a11y
  audit (this fixes the found gaps, not WCAG certification).
- Reviewer: dark-mode rendering of every touched chip; no visual diff on
  Badge variants used elsewhere.
