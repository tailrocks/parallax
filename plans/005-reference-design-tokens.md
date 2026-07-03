# Plan 005: Establish the reference design-token foundation (light+dark, shadow system, squircles, fonts)

> **Executor instructions**: Follow this plan step by step. Run every
> verification command and confirm the expected result before moving to the
> next step. If anything in the "STOP conditions" section occurs, stop and
> report — do not improvise. When done, update the status row for this plan
> in `plans/README.md`.
>
> **Reference project (READ THIS FIRST)**: This redesign copies the look and
> feel of a local reference console designated by the operator. Its name must
> NEVER appear in this repository — not in code, comments, commits, or docs.
> Its absolute path lives in the git-ignored file `plans/.reference-root`.
> Resolve it as: `REF_ROOT="$(cat plans/.reference-root)"` (run from the repo
> root; STOP if the file is missing). Everywhere this plan says `$REF_ROOT`,
> substitute that path. Reference state pinned at its commit `9f028d7`.
>
> **Drift check (run first)**: `git diff --stat ad9115d..HEAD -- ui/src/styles.css ui/src/routes/__root.tsx ui/package.json docs/research/architecture/simple-ui-v2.md ui/AGENTS.md`
> If any in-scope file changed since this plan was written, compare the
> "Current state" excerpts against the live code before proceeding; on a
> mismatch, treat it as a STOP condition.

## Status

- **Priority**: P1
- **Effort**: M
- **Risk**: MED
- **Depends on**: none
- **Category**: tech-debt (design foundation)
- **Planned at**: commit `ad9115d`, 2026-07-03

## Why this matters

The operator has ruled the current Parallax UI visually unacceptable and designated a local
reference console (`$REF_ROOT`, commit `9f028d7`, Apache-2.0, same stack: Tailwind v4 +
shadcn on Base UI + Bun) as the **source of truth for look and feel, to be copied essentially
1:1** while keeping all Parallax functionality. Everything downstream (primitives, shell,
screens — plans 006-018) builds on the token layer this plan installs. The reference's
signature look: an **achromatic neutral palette** (color reserved for status semantics),
**elevation via layered custom shadows instead of borders**, **squircle corners**
(`corner-squircle` via the `@toolwind/corner-shape` Tailwind plugin), Inter + Geist Mono, and
true light+dark themes. Parallax today has a custom dark-only "brand rainbow" theme with
border+shadow double-encoding — the root cause of the "ugly" verdict.

## Current state

- `ui/src/styles.css` — the entire current theme. Key facts (verified at `ad9115d`):
  - Line 4: `@import "@fontsource-variable/inter";` — fonts come from Fontsource (Vite app; no
    Next.js font loader). There is **no mono font** installed and no `--font-mono` token.
  - Lines 8-49 `@theme inline`: maps semantic tokens; radius scale is **multiplicative**
    (`--radius-sm: calc(var(--radius) * 0.6)` …) — the reference's is **additive** (below).
  - Lines 51-96: `:root` is a **dark** palette (background `oklch(0.13 0 0)`), with blue
    `--primary: oklch(0.66 0.19 251.6)`, raw-hex chart tokens (`--chart-1: #0090fd` …),
    a 6-color `--brand-*` palette (lines 84-89), `--surface-raised/sunken`, and a single
    heavy shadow `--custom-shadow: 0 18px 60px oklch(0 0 0 / 45%)` (line 92).
  - Lines 98-120: a `.light` class palette that is **unreachable** (no theme provider ever
    sets a class; grep confirms `.light`/`.dark` never applied).
  - Lines 134-170 `@layer components`: `.parallax-panel` (gradient + `border: 1px solid
    var(--border)` + box-shadow — border+shadow double-encoding), `.parallax-pill`,
    `.parallax-glow-border`.
- `ui/src/routes/__root.tsx` — no theme class is ever set on `<html>`; `next-themes` is in
  `package.json` but only imported by `ui/src/components/ui/sonner.tsx`, so every `dark:`
  utility in the shadcn components is currently inert.
- `ui/package.json` — has `@fontsource-variable/inter`, `next-themes@^0.4.6`, `tailwindcss@^4`,
  `tw-animate-css`; does **not** have `@toolwind/corner-shape`, a mono font, or
  `@tabler/icons-react` (icons are plan 006).
- Reference sources (read these; they are the source of truth):
  - `$REF_ROOT/packages/ui/src/styles/globals.css` — the complete token file to port
    (253 lines). Load-bearing values inlined below.
  - `$REF_ROOT/FRONTEND.md` — the design conventions ("never `border`/`divide-*` for
    separation — use a shadow token"; "neutral is the only grayscale"; "design and verify
    both light and dark").
- Repo rules that bind this plan: Bun only (`AGENTS.md` — never npm/pnpm/yarn; installs via
  `bun add`); TypeScript strictest mode; theme only through CSS variables in `src/styles.css`
  (`ui/AGENTS.md` rule 11).
- **Confidentiality rule (operator, 2026-07-03)**: the reference project's name must never be
  committed to this repository. Always call it "the reference console". Before every commit
  in this and later plans, run the leak check from `plans/README.md` §Reference.

## Commands you will need

All from `/Users/donbeave/Projects/tailrocks/parallax-project/parallax/ui` unless noted:

| Purpose   | Command                  | Expected on success |
|-----------|--------------------------|---------------------|
| Install   | `rtk bun install`        | exit 0              |
| Add dep   | `rtk bun add <pkg>`      | exit 0, `bun.lock` updated |
| Typecheck | `rtk bun run typecheck`  | exit 0              |
| Lint      | `rtk bun run lint`       | exit 0              |
| Tests     | `rtk bun run test`       | exit 0              |
| Build     | `rtk bun run build`      | exit 0              |
| Dev       | `rtk bun run dev`        | Vite serves on `http://localhost:3000/` |
| Leak check (repo root) | `git grep -ril "$(basename "$(cat plans/.reference-root)")" -- . | grep -v '^plans/.reference-root$'` | **no output** |

## Scope

**In scope** (the only files you should modify):
- `.gitignore` (repo root — one line, step 0)
- `ui/src/styles.css` (rewrite)
- `ui/src/routes/__root.tsx` (theme provider only)
- `ui/package.json` + `ui/bun.lock` (via `bun add` only)
- `docs/research/architecture/simple-ui-v2.md` (record the superseding design decision)
- `ui/AGENTS.md` (rule 11 note)
- `plans/README.md` (status row)

**Out of scope** (do NOT touch, even though they look related):
- `ui/src/components/ui/*` — primitive restyle is plan 006.
- `ui/src/components/parallax-shell.tsx` and all route files — plans 007+. They may look
  temporarily off (old classes on new tokens); that is expected and acceptable for one plan.
- Any Rust code.

## Git workflow

- Work directly on `main` (repo rule, `BRANCHING.md`).
- Conventional Commits (`COMMITS.md`), e.g. `style(ui): port reference design tokens`.
- Every commit: `git commit -s` and include exactly one trailer
  `Co-authored-by: Claude <noreply@anthropic.com>`.
- Before each commit: run the leak check (Commands table) — it must print nothing.

## Steps

### Step 0: Git-ignore the reference pointer

Append to the repo-root `.gitignore`:

```
plans/.reference-root
```

**Verify**: `git check-ignore plans/.reference-root` → prints the path (exit 0), and
`git status --short plans/` does not list `.reference-root`.

### Step 1: Add the corner-shape plugin and Geist Mono

```
cd ui
rtk bun add @toolwind/corner-shape @fontsource-variable/geist-mono
```

**Verify**: `rtk bun pm ls | grep -E "corner-shape|geist-mono"` → both listed.

### Step 2: Rewrite `ui/src/styles.css` with the reference token set

Replace the whole file. Target structure (port from `$REF_ROOT/packages/ui/src/styles/globals.css`,
adapting only the font imports and dropping its `@source` lines, which Parallax does not need):

1. Header:
   ```css
   @import "tailwindcss";
   @import "tw-animate-css";
   @import "shadcn/tailwind.css";
   @import "@fontsource-variable/inter";
   @import "@fontsource-variable/geist-mono";
   @plugin "@toolwind/corner-shape";

   @custom-variant dark (&:is(.dark *));
   ```
2. `:root` = the reference's **light** palette + light shadow set, copied verbatim from its
   `globals.css:10-88`. Anchor values (the file is the source): `--background: oklch(0.995 0 0)`,
   `--foreground: oklch(0.145 0 0)`, `--card: oklch(1 0 0)`, `--primary: oklch(0.205 0 0)`,
   `--muted-foreground: oklch(0.556 0 0)`, `--destructive: oklch(0.577 0.245 27.325)`,
   `--border: oklch(0.922 0 0)`, `--ring: oklch(0.708 0 0)`, `--radius: 0.625rem`,
   `--chart-1..5` = grayscale ramp `0.87/0.556/0.439/0.371/0.269`, all `--sidebar-*` tokens,
   and the full shadow set:
   ```css
   --custom-shadow:
     0px 0px 0px 1px rgba(0, 0, 0, 0.06), 0px 1px 2px -1px rgba(0, 0, 0, 0.06),
     0px 2px 4px 0px rgba(0, 0, 0, 0.04);
   --custom-shadow-primary:
     0px 0px 0px 1px rgba(0, 0, 0, 0.6), 0px 1px 2px -1px rgba(0, 0, 0, 0.4),
     0px 2px 4px 0px rgba(0, 0, 0, 0.18);
   ```
   plus `-secondary`, `-destructive`, and the hue variants `-green -blue -amber -orange
   -emerald -rose -slate -violet` (pattern `0 0 0 1px rgba(hue,.25), 0 1px 2px -1px
   rgba(hue,.18), 0 2px 4px 0 rgba(hue,.12)`).
3. `.dark` = the reference's dark palette + dark shadow set, verbatim from its
   `globals.css:90-160`. Anchors: `--background: oklch(0.17 0 0)`, `--card: oklch(0.205 0 0)`,
   `--primary: oklch(0.922 0 0)` (light-on-dark), `--border: oklch(1 0 0 / 10%)`,
   `--input: oklch(1 0 0 / 15%)`, `--sidebar: oklch(0.145 0 0)`, and dark shadows of the form
   ```css
   --custom-shadow:
     inset 0 1px 0 0 rgba(255, 255, 255, 0.03),
     inset 0 0 0 1px rgba(255, 255, 255, 0.03), 0 0 0 1px rgba(0, 0, 0, 0.1),
     0 2px 2px 0 rgba(0, 0, 0, 0.1), 0 4px 4px 0 rgba(0, 0, 0, 0.1),
     0 8px 8px 0 rgba(0, 0, 0, 0.1);
   --custom-shadow-primary:
     inset 0 1px 0 0 rgba(255, 255, 255, 0.45),
     0 0 0 1px rgba(255, 255, 255, 0.55), 0 1px 2px 0 rgba(0, 0, 0, 0.5);
   ```
   (hue variants: `inset 0 1px 0 0 rgba(hue,.03), 0 0 0 1px rgba(hue,.18), 0 1px 2px 0
   rgba(hue,.1)`).
4. `@theme inline` block: copy the reference's (globals.css:162-205) — all `--color-*`
   mappings, the **additive** radius scale (`--radius-sm: calc(var(--radius) - 4px)` …
   `--radius-4xl: calc(var(--radius) + 16px)`), `--font-heading: var(--font-sans)`,
   `--animate-shimmer: shimmer 4s linear infinite` — and set the Parallax font stacks:
   ```css
   --font-sans: "Inter Variable", ui-sans-serif, system-ui, sans-serif;
   --font-mono: "Geist Mono Variable", ui-monospace, SFMono-Regular, monospace;
   ```
   (Note: the reference loads Geist Mono but never maps `--font-mono` — a known drift between
   its FRONTEND.md and code. We implement the *intended* look: `font-mono` resolves to Geist
   Mono.)
5. `@keyframes shimmer` (globals.css:207-214) verbatim.
6. `@layer base` (globals.css:216-253) verbatim: `* { @apply border-border outline-ring/50 }`;
   `body { @apply font-sans bg-background text-foreground selection:bg-primary/10;
   scrollbar-width: thin; scrollbar-color: gray transparent; }`; `html { @apply font-sans }`;
   and the **webkit autofill override** (transition-delay `999999s !important` +
   `-webkit-text-fill-color`/`caret-color: var(--foreground)`) — required because inputs
   become `bg-transparent` in plan 006.
7. **Temporary legacy compat block** (until plan 018 removes it), clearly marked:
   ```css
   /* LEGACY-COMPAT — consumed by pre-redesign routes/components; removed by plan 018. */
   :root {
     --brand-blue: #3b82f6; --brand-orange: #f97316; --brand-rose: #f43f5e;
     --brand-green: #22c55e; --brand-violet: #8b5cf6; --brand-fuchsia: #d946ef;
     --surface-raised: var(--card); --surface-sunken: var(--muted);
   }
   @layer components {
     .parallax-panel { background: var(--color-card); border-radius: var(--radius-xl);
       box-shadow: var(--custom-shadow); }
     .parallax-pill { background: var(--color-muted); border-radius: 999px; }
     .parallax-glow-border { position: relative; }
   }
   ```
   This re-bases the legacy hexes onto Tailwind's standard hues (used by the shadow variants)
   and makes `.parallax-panel` borderless card-like so untouched pages inherit the new look
   instead of breaking. Do **not** port the old gradients/glow.

**Verify**: `rtk bun run build` → exit 0. `grep -c "custom-shadow" src/styles.css` → ≥ 24
(12 tokens × 2 themes). `grep -n "font-mono" src/styles.css` → the `@theme` mapping exists.

### Step 3: Wire the theme provider (light+dark both real, dark default)

In `ui/src/routes/__root.tsx`, wrap the app body with `next-themes`:

```tsx
import { ThemeProvider } from "next-themes"
// inside the root component, around the shell/outlet:
<ThemeProvider attribute="class" defaultTheme="dark" enableSystem disableTransitionOnChange>
  ...existing shell...
</ThemeProvider>
```

This matches the reference (`$REF_ROOT/apps/web/src/components/providers.tsx`:
`attribute="class" defaultTheme="dark" enableSystem disableTransitionOnChange`). TanStack
Start note: if the root route renders the `<html>` element, add `suppressHydrationWarning` on
it (next-themes mutates the class client-side). The visible theme switcher lands in plan 007's
sidebar footer; this step only makes `.dark` real.

**Verify**: `rtk bun run typecheck` → exit 0. `rtk bun run dev`, open
`http://localhost:3000/issues`: `document.documentElement.classList` contains `dark`; in
DevTools, toggling the class to light shows the light palette (white background) — both themes
render.

### Step 4: Record the design decision in the intent docs (no reference name!)

1. `docs/research/architecture/simple-ui-v2.md` — in the "Required Tech Stack" table, the
   Theme row currently reads "**Default theme as-is** — no customization". Amend it to:
   "**Custom token theme** (operator decision 2026-07-03, supersedes 'default theme as-is'):
   neutral achromatic palette, custom-shadow elevation instead of borders, squircle corners
   (`@toolwind/corner-shape`), Inter + Geist Mono, light+dark via `next-themes`. Modeled on a
   local reference console designated by the operator (deliberately not named here). See
   `plans/005…018`."
2. `ui/AGENTS.md` — extend rule 11 with one sentence: theming still only via CSS variables in
   `src/styles.css`; the variable set is the custom token system adopted 2026-07-03, and
   visual separation uses `--custom-shadow*` tokens, not borders.

**Verify**: `grep -n "custom token" docs/research/architecture/simple-ui-v2.md` → hit;
leak check (Commands table) → no output.

### Step 5: Full gate

**Verify**: `rtk bun run typecheck && rtk bun run lint && rtk bun run test && rtk bun run build`
→ all exit 0. Dev-serve and click through Issues, Traces, Logs — pages must render without
console errors (they will look transitional; that is fine). Run the leak check → no output.

## Test plan

This plan is CSS/provider-only; no new unit tests. The gates are the four verification
commands plus the manual light/dark render check in Step 3. Do not add snapshot tests of CSS.

## Done criteria

- [ ] `rtk bun run typecheck` / `lint` / `test` / `build` all exit 0
- [ ] `git check-ignore plans/.reference-root` → exit 0
- [ ] Leak check → no output
- [ ] `grep -n "brand-blue: #0090fd" ui/src/styles.css` → no match (old brand palette gone)
- [ ] `grep -n "0 18px 60px" ui/src/styles.css` → no match (old heavy shadow gone)
- [ ] `grep -n "corner-shape" ui/src/styles.css` → `@plugin` present
- [ ] `grep -n "Geist Mono" ui/src/styles.css` → `--font-mono` mapped
- [ ] `.dark` class present on `<html>` at runtime, light palette reachable
- [ ] `plans/README.md` status row updated

## STOP conditions

Stop and report back (do not improvise) if:
- `plans/.reference-root` is missing, or `$REF_ROOT/packages/ui/src/styles/globals.css` does
  not exist or no longer contains the `--custom-shadow` tokens (reference moved/changed).
- `@toolwind/corner-shape` fails to install or `@plugin` makes the Vite/Tailwind build error —
  report; do NOT ship squircle-less silently.
- `next-themes` cannot set the class on `<html>` under TanStack Start SSR (hydration errors
  that `suppressHydrationWarning` does not resolve).
- The drift check shows `ui/src/styles.css` changed since `ad9115d`.

## Maintenance notes

- Plans 006-018 assume these exact token names; renaming any `--custom-shadow*` token breaks
  the whole series.
- The LEGACY-COMPAT block is a bridge, not a home — plan 018 deletes it; nothing new may
  reference `--brand-*` or `.parallax-panel`.
- Both themes are now real: every subsequent visual change must be checked in light AND dark.
- The reference project stays unnamed in this repo forever; keep using "the reference
  console" and `plans/.reference-root`.
