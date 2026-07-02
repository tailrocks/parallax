# Plan 004: Keep Branded Shell Visible During Loading And Errors

> **Executor instructions**: Follow this plan step by step. Run every verification command and confirm expected result before next step. If any STOP condition occurs, stop and report. When done, update `plans/README.md`.
>
> **Drift check (run first)**: `rtk git diff --stat 8dde008..HEAD -- ui/src/routes/__root.tsx ui/src/components ui/src/lib/api.ts`
> If any in-scope file changed since this plan was written, compare "Current state" excerpts against live code before proceeding; on mismatch, STOP.

## Status

- **Priority**: P1
- **Effort**: S
- **Risk**: LOW, presentation-only fallback states
- **Depends on**: `plans/001-reference-style-theme-tokens.md`, `plans/002-dark-product-shell.md`
- **Category**: bug
- **Planned at**: commit `8dde008`, 2026-07-03

## Why This Matters

With the API offline, current Parallax renders TanStack Router's default error UI: `Something went wrong!`, a tiny `Hide Error` button, and raw red monospace text. The app shell disappears from the screenshot, so the new reference-style design would vanish exactly when local setup is incomplete. A product-grade observability tool needs branded loading/error states that preserve navigation and explain the failing surface.

## Current State

- `ui/src/routes/__root.tsx:8-36` configures root route but has no `errorComponent` or `pendingComponent`.
- `ui/src/routes/__root.tsx:44-45` wraps normal children in `ParallaxShell`, but the observed error boundary at `/issues` with no API server bypassed this visual shell.
- Browser screenshot on 2026-07-03 against `http://localhost:3000/issues` with no API server showed body text: `Something went wrong!`, `Hide Error`, `parallax api unreachable (502)`.
- `ui/node_modules/@tanstack/react-router/src/Match.tsx` supports route `errorComponent`, `pendingComponent`, and router-level defaults.

Current root excerpt:

```tsx
// ui/src/routes/__root.tsx:8
export const Route = createRootRoute({
  head: () => ({ ... }),
  notFoundComponent: () => (
    <main className="container mx-auto p-4 pt-16">
      <h1>404</h1>
      <p>The requested page could not be found.</p>
    </main>
  ),
  shellComponent: RootDocument,
})
```

## Commands You Will Need

| Purpose | Command | Expected on success |
|---------|---------|---------------------|
| Typecheck | `rtk bun run typecheck` | exit 0 |
| Lint | `rtk bun run lint` | exit 0 |
| Build | `rtk bun run build` | exit 0 |
| Dev | `rtk bun run dev` | Vite local URL available |

## Scope

**In scope**:
- `ui/src/routes/__root.tsx`
- New presentational fallback component under `ui/src/components/`, e.g. `route-fallbacks.tsx`
- `ui/src/lib/api.ts` only if extracting a safe error message helper is needed

**Out of scope**:
- Backend startup or health implementation
- Retrying/fetch policy changes
- Route loader query changes
- Toast/notification system

## Git Workflow

Work on current branch unless operator says otherwise. Commit style follows existing conventional commits. If committing, use `rtk git commit -s` and include `Co-authored-by: Codex <codex@openai.com>`.

## Steps

### Step 1: Add Branded Error Component

Create a component such as `RouteErrorPanel` that accepts TanStack Router error props. It should render inside product styling:

- Compact panel/card using Plan 001/002 tokens.
- Title like `Surface unavailable`.
- Short safe message from error, e.g. `parallax api unreachable (502)`.
- Suggested next action for local mode: verify Parallax API at `127.0.0.1:4000`.
- Optional retry button if TanStack error props expose reset; if uncertain, omit retry instead of guessing.

Do not dump stack traces in normal UI.

**Verify**: `rtk bun run typecheck` -> exit 0.

### Step 2: Add Branded Pending Component

Create `RoutePendingPanel` or similar:

- Skeleton/KPI-card style panel.
- Keeps dark shell visible.
- No spinner-only blank screen.

Use shadcn `Skeleton` if installed; it is present through sidebar imports.

**Verify**: `rtk bun run lint` -> exit 0.

### Step 3: Wire Root Route Fallbacks

In `createRootRoute`, add:

- `errorComponent: RouteErrorPanel`
- `pendingComponent: RoutePendingPanel`
- Replace `notFoundComponent` with a branded shell-compatible panel.

Ensure fallback components render as route children so `RootDocument` still wraps them in `ParallaxShell`. If TanStack behavior still bypasses shell, move shell wrapping to the document level in a way that includes error output.

**Verify**: `rtk bun run typecheck` -> exit 0.

### Step 4: Verify Offline API Scenario

Start dev server with API intentionally offline:

```bash
rtk bun run dev
```

Open `http://localhost:3000/issues`. Expected:

- Shell/nav/header visible.
- Error appears as styled dark panel.
- No raw default `Something went wrong!` boundary visible.
- Error message does not include stack trace.

If using automated Chrome/Playwright, system Chrome may be available at `/Applications/Google Chrome.app/Contents/MacOS/Google Chrome` on this machine, but do not make that path required for the project.

**Verify**: visual/manual check as above.

### Step 5: Run Build Gate

**Verify**:

- `rtk bun run build` -> exit 0
- `rtk bun run test` -> exit 0

## Test Plan

- Add a small component test if project test setup supports React rendering:
  - `RouteErrorPanel` renders safe error message.
  - `RoutePendingPanel` renders skeleton/panel.
- If test setup is too thin, skip new tests and rely on typecheck/build plus manual offline API visual check.

## Done Criteria

- [ ] API-offline `/issues` keeps shell visible.
- [ ] Default TanStack `Something went wrong!` UI no longer appears.
- [ ] Error panel uses Parallax/visual reference-style dark product surface.
- [ ] Stack traces are not shown in normal UI.
- [ ] Pending state is branded and not blank.
- [ ] `rtk bun run typecheck`, `rtk bun run lint`, `rtk bun run test`, and `rtk bun run build` exit 0.

## STOP Conditions

- TanStack Router version requires a different root API than `errorComponent`/`pendingComponent`.
- Shell cannot wrap route errors without large router restructuring.
- Safe error extraction would require changing `graphql()` behavior for all callers.

## Maintenance Notes

This plan makes local failures part of the product experience. Later backend health work can add live status, but this plan should stay presentation-only.
