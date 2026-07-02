# Plan 001: Establish Reference-Style Theme Tokens

> **Executor instructions**: Follow this plan step by step. Run every verification command and confirm expected result before next step. If any STOP condition occurs, stop and report. When done, update `plans/README.md`.
>
> **Drift check (run first)**: `rtk git diff --stat 8dde008..HEAD -- ui/src/styles.css ui/components.json ui/package.json`
> If any in-scope file changed since this plan was written, compare "Current state" excerpts against live code before proceeding; on mismatch, STOP.

## Status

- **Priority**: P1
- **Effort**: M
- **Risk**: MED, global tokens affect every UI surface
- **Depends on**: none
- **Category**: direction
- **Planned at**: commit `8dde008`, 2026-07-03

## Why This Matters

Parallax currently uses neutral grayscale shadcn defaults. The visual reference's feel comes from a dark-first product surface, white/soft-gray typography, compact rounded controls, strong blue/orange brand accents, and colorful metric colors. Without shared tokens, later shell/page changes will duplicate raw colors and drift.

## Current State

- `ui/components.json` uses shadcn `base-vega`, Tailwind v4, CSS variables, lucide icons, and aliases such as `@/components/ui`.
- `ui/src/styles.css:51-83` defines light defaults as pure neutral white/gray.
- `ui/src/styles.css:86-118` defines dark defaults, but the app does not default to dark and chart colors remain gray.
- `ui/src/styles.css:8-48` exposes Tailwind v4 `@theme inline` tokens.

Relevant current excerpt:

```css
/* ui/src/styles.css:51 */
:root {
  --background: oklch(1 0 0);
  --foreground: oklch(0.145 0 0);
  --card: oklch(1 0 0);
  --primary: oklch(0.205 0 0);
  --chart-1: oklch(0.87 0 0);
  --chart-2: oklch(0.556 0 0);
}
```

Visual reference evidence gathered 2026-07-03:

- Homepage defaults to dark via inline theme script and uses `bg-background`, `bg-card`, `text-muted-foreground`, `rounded-full`, `rounded-3xl`, and `shadow-[var(--custom-shadow...)]`.
- Brand mark uses overlapping dots: dark/white, `#0090FD` blue, `#FF5513` orange.
- Accent glows in homepage HTML include blue `rgba(40, 140, 255, ...)`, orange `rgba(255, 120, 40, ...)`, rose `rgba(255, 50, 100, ...)`, green `rgba(50, 200, 80, ...)`, violet `rgba(100, 70, 255, ...)`, and fuchsia `rgba(240, 50, 180, ...)`.
- visual reference docs define dark background around `#0F0F0F` and primary gray around `#171717`.

## Commands You Will Need

| Purpose | Command | Expected on success |
|---------|---------|---------------------|
| Typecheck | `rtk bun run typecheck` | exit 0 |
| Lint | `rtk bun run lint` | exit 0 |
| Build | `rtk bun run build` | exit 0 |

## Scope

**In scope**:
- `ui/src/styles.css`
- `ui/components.json` only if shadcn metadata must reflect token/theme choice

**Out of scope**:
- React route/page rewrites
- Component API changes
- Package installs
- Backend/API changes

## Git Workflow

Work on current branch unless operator says otherwise. Commit style follows existing conventional commits, e.g. `feat(ui): ...`. If committing, use `rtk git commit -s` and include `Co-authored-by: Codex <codex@openai.com>`.

## Steps

### Step 1: Make Dark Theme Default Without Removing Light Tokens

In `ui/src/styles.css`, set `:root` to reference-style dark tokens and keep `.light` or an equivalent explicit light class for future opt-in. Do not depend on `next-themes`; this repo imports it but currently does not wire a provider. Set body to dark by default through CSS variables.

Target token direction:

- `--background`: near black, around `oklch(0.13 0 0)` / `#0f0f0f`
- `--foreground`: near white, around `oklch(0.96 0 0)`
- `--card`: slightly lifted black, around `oklch(0.17 0 0)`
- `--muted`: dark gray, around `oklch(0.22 0 0)`
- `--muted-foreground`: soft gray, around `oklch(0.68 0 0)`
- `--border`: subtle white alpha, around `oklch(1 0 0 / 10%)`
- `--primary`: visual reference blue `#0090FD` converted to OKLCH or kept via hex only inside CSS variables if needed
- `--destructive`: rose/red accent
- `--radius`: increase slightly to `0.875rem` or equivalent; keep cards at <= 8px only if existing design-system constraint demands, otherwise match the visual reference's rounded product cards selectively with component classes in later plans.

**Verify**: `rtk bun run typecheck` -> exit 0.

### Step 2: Add Parallax Brand Accent Tokens

Still in `ui/src/styles.css`, add semantic custom variables under `:root` and expose only ones needed by Tailwind through normal CSS vars:

```css
--brand-blue: #0090fd;
--brand-orange: #ff5513;
--brand-rose: #ff3264;
--brand-green: #32c850;
--brand-violet: #6446ff;
--brand-fuchsia: #f032b4;
--surface-raised: color-mix(in oklch, var(--card), white 3%);
--surface-sunken: color-mix(in oklch, var(--background), black 8%);
--custom-shadow: 0 18px 60px oklch(0 0 0 / 45%);
--custom-shadow-primary: 0 0 0 1px color-mix(in oklch, var(--brand-blue), transparent 70%), 0 18px 60px oklch(0 0 0 / 45%);
```

Do not add decorative gradient blobs. Accent use should be constrained to borders, icons, metric lines, badges, and focus states.

**Verify**: `rtk bun run typecheck` -> exit 0.

### Step 3: Replace Gray Chart Tokens With reference-like Data Colors

Change `--chart-1..5` from gray ramp to distinct colors:

- `--chart-1`: blue
- `--chart-2`: amber/orange
- `--chart-3`: fuchsia/violet
- `--chart-4`: green
- `--chart-5`: rose/red

Keep contrast acceptable on dark cards.

**Verify**: `rtk bun run build` -> exit 0.

### Step 4: Add Utility Classes For Product Surfaces

Add small CSS utilities under `@layer components` or `@layer utilities`:

- `.parallax-panel`: raised dark card, subtle border, shadow, overflow hidden
- `.parallax-pill`: compact rounded-full control surface
- `.parallax-brand-mark`: layout hook for overlapping three-dot mark if needed by shell
- `.parallax-glow-border`: optional pseudo-element border inspired by the visual reference's beam border; static or reduced-motion-safe, no JS animation required in this plan

Use semantic vars, not hardcoded page-local Tailwind colors.

**Verify**: `rtk bun run lint` -> exit 0.

## Test Plan

- No new unit tests required for token-only CSS.
- Visual verification is mandatory after Plan 002/003, not here.

## Done Criteria

- [ ] `ui/src/styles.css` defaults to dark reference-style tokens.
- [ ] Brand accent and chart tokens exist and are semantically named.
- [ ] `rtk bun run typecheck` exits 0.
- [ ] `rtk bun run lint` exits 0.
- [ ] `rtk bun run build` exits 0.
- [ ] No files outside scope modified.

## STOP Conditions

- `components.json` shows a different shadcn base than `base-vega`.
- Tailwind v4 `@theme inline` structure has been replaced since this plan was written.
- Implementing dark default requires adding a dependency.
- Any change requires touching route/page files; leave that to Plans 002/003.

## Maintenance Notes

Future pages should consume `--chart-*`, `--brand-*`, `.parallax-panel`, and `.parallax-pill`. Reviewers should reject one-off raw color classes when a semantic token exists.
