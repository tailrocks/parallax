# Plan 079: Shared GraphQL selections/types in the UI + dependency hygiene across both manifests

> **Executor instructions**: Follow this plan step by step. Run every
> verification command and confirm the expected result before moving to the
> next step. If anything in the "STOP conditions" section occurs, stop and
> report — do not improvise. When done, update the status row for this plan
> in `advisor-plans/README.md` — unless a reviewer dispatched you and told you
> they maintain the index.
>
> **Drift check (run first)**: `git diff --stat dbaba3c..HEAD -- ui/package.json ui/src/lib/api.ts Cargo.toml .gitignore bench/otlp-fanout/rotel.env`
> If any in-scope file changed since this plan was written, compare the
> "Current state" excerpts against the live code before proceeding; on a
> mismatch, treat it as a STOP condition.

## Status

- **Priority**: P2 (hygiene) — the rotel.env item is P2-security
- **Effort**: M
- **Risk**: LOW
- **Depends on**: 069 (UI tests in CI make the dedup verifiable); after 071/077
  (both edit the same route files — land those first)
- **Category**: tech-debt / deps / security-hygiene
- **Planned at**: commit `dbaba3c`, 2026-07-10

## Why this matters

- UI routes re-declare GraphQL selection sets and row interfaces inline —
  the log field list is repeated verbatim 3× in `logs.tsx` plus a near-copy in
  `runs.$runId.tsx`; ~54 inline interfaces duplicate shapes that
  `ui/src/lib/api.ts` already exports. Copies have drifted: `traces.index.tsx`'s
  `SpanDoc` lacks `parentSpanId` vs the shared `Span`. Every field change is
  N edits with silent-undefined failure modes.
- Six UI dependencies have zero imports (`@dnd-kit/*` ×4,
  `@tanstack/react-table`, `date-fns`); `shadcn` sits in runtime deps but is
  only a build-time CSS import; 8 `@tanstack/*` deps are pinned `"latest"`
  (no floor recorded — violates the repo's own version-table policy);
  `recharts` is exact-pinned `3.8.0` while 3.9.x fixed a ResizeObserver
  memory leak. Rust side: `thiserror` is declared in 3 crates and used in
  none; `dirs` is in `[workspace.dependencies]` and absent from `Cargo.lock`.
- `bench/otlp-fanout/rotel.env` is a git-TRACKED env file carrying an
  `Authorization` header (line 24) and an `x-sentry-auth` header (line 44)
  for LOCAL lab instances — lab-only credentials, but the pattern is one
  repo-copy away from committing a real tenant secret, and `.gitignore` has
  no `*.env` rule at all.

## Current state

- Selection-set duplication: the log field list
  `tsNanos eventName observedTsNanos service severityNum severityText body traceId spanId runId scopeName attributes resource`
  appears in `ui/src/routes/logs.tsx` at ~`:229`, ~`:232`, and ~`:405`
  (inside `loadOlder`), and a near-copy in `ui/src/routes/runs.$runId.tsx`
  (~`:137`).
- Type duplication (verified drift): `ui/src/routes/traces.index.tsx:64-74`:

  ```ts
  /** One finished span from the live feed (`/v1/traces/stream`). */
  interface SpanDoc {
    tsNanos: string
    service: string
    traceId: string
    spanId: string
    name: string
    kind: string
    statusCode: string
    durationNs: string
  }
  ```

  vs `ui/src/lib/api.ts:80-91` `export interface Span` — same shape PLUS
  `parentSpanId: string | null`. NOTE the doc comment: `SpanDoc` describes
  the SSE live-feed payload — before merging the two, CHECK what
  `/v1/traces/stream` actually emits (`crates/parallax-server/src/live.rs`,
  the span serializer): if the live feed omits `parentSpanId`, keep a
  distinct live-feed type but derive it: `type SpanDoc = Omit<Span, "parentSpanId">`.
  Other inline copies to consolidate: `routes/index.tsx` (~`:92 IssueRow`,
  ~`:101 TraceRow`), `routes/runs.index.tsx` (~`:55 RunRow`) — for each,
  compare against the exported `lib/api.ts` interfaces and reuse or derive
  (`Pick<...>`) instead of redeclaring.
- `ui/package.json` (facts verified by grep at planning):
  - Zero imports anywhere in `ui/src` (incl. CSS): `@dnd-kit/core`,
    `@dnd-kit/modifiers`, `@dnd-kit/sortable`, `@dnd-kit/utilities`,
    `@tanstack/react-table`, `date-fns`.
  - `shadcn` used ONLY as `@import "shadcn/tailwind.css"` in
    `ui/src/styles.css` → belongs in devDependencies.
  - `"latest"`: `@tanstack/react-devtools`, `react-router`,
    `react-router-devtools`, `react-router-ssr-query`, `react-start`,
    `router-plugin` (deps) + `devtools-vite`, `eslint-config` (devDeps).
  - `"recharts": "3.8.0"` exact; 3.9.1+ is current with a ResizeObserver
    leak fix. Used via `components/ui/chart.tsx` + 8 more files.
  - KEEP (verified used): `@toolwind/corner-shape` (styles.css `@plugin`),
    `@tanstack/react-virtual`, `motion`, `tw-animate-css`.
- Root `Cargo.toml`: `dirs = "6"` in `[workspace.dependencies]` — no crate
  references it (absent from `Cargo.lock`). `thiserror = "2"` in workspace
  deps + `thiserror = { workspace = true }` in
  `crates/parallax-core/Cargo.toml`, `crates/parallax-server/Cargo.toml`,
  `crates/parallax-storage/Cargo.toml` — zero `thiserror::`/`#[derive(Error)]`
  usages in any crate.
- `.gitignore` (root, 327B): no `env` entry of any kind.
- `bench/otlp-fanout/rotel.env`: tracked; headers at `:24`
  (`ROTEL_EXPORTER_OPENOBSERVE_CUSTOM_HEADERS`, an Authorization header) and
  `:44` (`ROTEL_EXPORTER_SENTRY_CUSTOM_HEADERS`). Per `bench/otlp-fanout/README.md`,
  these authenticate only local docker-compose lab instances (the OpenObserve
  value is the base64 of the compose default login). Never copy the values
  anywhere; this plan replaces the file with an example template.

## Commands you will need

| Purpose | Command | Expected on success |
|---------|---------|---------------------|
| UI gates (from `ui/`) | `rtk bun run typecheck && rtk bun run lint && rtk bun run test && rtk bun run build` | all exit 0 |
| Re-lock UI deps (from `ui/`) | `rtk bun install` | exit 0, `bun.lock` updated |
| Rust gates (root) | `rtk cargo build --workspace && rtk cargo nextest run --workspace && rtk cargo clippy --workspace --all-targets` | pass, zero warnings |

## Scope

**In scope** (the only files you should modify):
- `ui/src/lib/api.ts` (add field-list constants; adjust/derive types)
- `ui/src/routes/logs.tsx`, `runs.$runId.tsx`, `traces.index.tsx`,
  `index.tsx`, `runs.index.tsx` (consume shared selections/types)
- `ui/package.json`, `ui/bun.lock`
- Root `Cargo.toml`, `crates/parallax-core/Cargo.toml`,
  `crates/parallax-server/Cargo.toml`, `crates/parallax-storage/Cargo.toml`,
  `Cargo.lock`
- `.gitignore`
- `bench/otlp-fanout/rotel.env` (delete from tracking),
  `bench/otlp-fanout/rotel.env.example` (create),
  `bench/otlp-fanout/README.md` (setup note)
- `advisor-plans/README.md` (status row)

**Out of scope** (do NOT touch, even though they look related):
- GraphQL-client codegen/variables migration — known deferred debt.
- Upgrading `"latest"` deps to NEWER versions — this plan only pins what
  `bun.lock` already resolves (floor-recording, not upgrading).
- Any Rust code (manifest-only changes there).
- Rewriting git history for rotel.env — the values are lab-default creds; the
  file's history stays. If the operator later confirms any value was ever
  non-default/real, rotation on those lab services is their action item —
  note it in the commit message.

## Git workflow

- Work directly on `main` (repo rule — `BRANCHING.md`).
- Separate commits: `refactor(ui): shared graphql field selections`,
  `chore(ui): prune unused deps and pin tanstack floors`,
  `chore: drop unused rust workspace deps`,
  `chore(bench): replace tracked rotel.env with example template`.
- DCO signoff (`git commit -s`), trailer
  `Co-authored-by: Claude <noreply@anthropic.com>`.

## Steps

### Step 1: Shared selection constants + types

In `ui/src/lib/api.ts` add exported field-list constants:

```ts
export const LOG_FIELDS =
  "tsNanos eventName observedTsNanos service severityNum severityText body traceId spanId runId scopeName attributes resource"
export const SPAN_FIELDS =
  "tsNanos service traceId spanId parentSpanId name kind statusCode durationNs"
```

Replace the inline lists: `logs.tsx` ×3, `runs.$runId.tsx` ×1 (interpolate
`${LOG_FIELDS}` into the template literals). Before replacing each, DIFF the
inline list against the constant — if an inline copy has extra/missing
fields, match the constant to what that query's consumer actually reads and
note any intentional narrow selections (a narrower selection may stay inline
if it is deliberate; the goal is one source of truth for the FULL shapes).

Consolidate the row types: check `live.rs`'s SSE payload for `parentSpanId`
(see Current state) and then replace `SpanDoc` with the derived type;
replace `IssueRow`/`TraceRow`/`RunRow` inline interfaces with imports or
`Pick<>` derivations from `lib/api.ts`.

**Verify**: from `ui/`: `rtk bun run typecheck` → 0 (the compiler proves shape
compatibility); `rtk bun run test` → route suites pass;
`grep -rn "observedTsNanos service severityNum" ui/src/routes` → 0 raw
inline copies remain (all via `LOG_FIELDS`).

### Step 2: UI dependency prune + floors

In `ui/package.json`:
1. Remove: `@dnd-kit/core`, `@dnd-kit/modifiers`, `@dnd-kit/sortable`,
   `@dnd-kit/utilities`, `@tanstack/react-table`, `date-fns`.
2. Move `shadcn` to `devDependencies`.
3. Replace every `"latest"` with a caret floor of the CURRENTLY-RESOLVED
   version from `ui/bun.lock` (look each up in the lock; e.g. if the lock has
   `@tanstack/react-start@1.168.25`, write `"^1.168.25"`). Do not chase newer.
4. Bump `recharts` to `"^3.9.1"`.

Run `rtk bun install` (updates the lock), then full UI gates. Chart smoke:
`rtk bun run build` must succeed; if a dev server + backend is available,
eyeball one dashboard chart — otherwise state that only build-level
verification ran.

**Verify**: from `ui/`: typecheck/lint/test/build all exit 0;
`grep -n '"latest"' ui/package.json` → 0 matches;
`grep -n "dnd-kit\|react-table\|date-fns" ui/package.json` → 0 matches.

### Step 3: Rust manifest prune

Remove `dirs = "6"` from root `Cargo.toml` `[workspace.dependencies]`;
remove the `thiserror` lines from the three crate manifests AND the workspace
table. (If a plan executed before this one introduced a real `thiserror`
usage — grep first: `grep -rn "thiserror" crates/*/src` — keep whichever
crates now use it and prune the rest.)

**Verify**: `rtk cargo build --workspace` → exit 0 (also refreshes
`Cargo.lock`); `rtk cargo nextest run --workspace` → pass.

### Step 4: rotel.env → example template

1. Create `bench/otlp-fanout/rotel.env.example`: copy the structure of
   `rotel.env` with every header/credential VALUE replaced by a placeholder
   (`REPLACE_ME`) — do not copy the current values into the example.
2. `git rm --cached bench/otlp-fanout/rotel.env` then delete it from the
   working tree (the local runner re-creates it from the example).
3. Add to `.gitignore`:

   ```
   *.env
   !*.env.example
   ```

4. Update `bench/otlp-fanout/README.md` setup section: `cp rotel.env.example
   rotel.env`, fill in the local lab values (point at the compose defaults it
   already documents).

**Verify**: `git status` shows `rotel.env` deleted + example added;
`git check-ignore bench/otlp-fanout/rotel.env` → path is ignored;
`grep -rn "Authorization" bench/otlp-fanout/rotel.env.example` shows only
placeholder values.

### Step 5: Full gates

**Verify**: UI: typecheck/lint/test/build → 0. Rust: build + nextest + clippy
zero warnings. `git status` clean except intended changes.

## Test plan

- No new tests: Step 1 is proven by the compiler + existing route suites
  (which render these routes and would fail on missing fields).
- Step 2's regression risk (recharts minor bump) is covered by build +
  existing component tests; note in the commit if visual eyeballing wasn't
  possible.

## Done criteria

Machine-checkable. ALL must hold:

- [ ] `grep -rn "LOG_FIELDS" ui/src | wc -l` → ≥5 (definition + 4 uses)
- [ ] `grep -n '"latest"' ui/package.json` → 0
- [ ] `grep -n "recharts" ui/package.json` → shows `^3.9`
- [ ] `grep -n "thiserror\|^dirs" Cargo.toml crates/*/Cargo.toml` → only crates that actually use thiserror (0 expected)
- [ ] `bench/otlp-fanout/rotel.env` not tracked; `.env.example` tracked; `.gitignore` has the `*.env` pair
- [ ] UI typecheck/lint/test/build + Rust build/nextest/clippy all green
- [ ] `git status` shows no modified files outside the in-scope list
- [ ] `advisor-plans/README.md` status row updated

## STOP conditions

Stop and report back (do not improvise) if:

- The SSE live-feed payload genuinely differs from the stored-span shape in
  more than `parentSpanId` — report the actual payload shape before
  consolidating types.
- Removing recharts 3.8.0's pin breaks the build or a chart test (3.9 API
  break in `chart.tsx`) — report; do not patch `components/ui/chart.tsx`
  (shadcn-vendored) beyond what the shadcn upstream ships.
- Any "unused" dep turns out to be referenced somewhere unexpected
  (a config file, a CSS `@plugin`) — re-verify with a whole-`ui/` grep
  including config files before deleting, and report if found.
- `bun install` rewrites the lock with unexpectedly different versions than
  the recorded floors (registry drift between planning and execution).

## Maintenance notes

- New route queries should compose `LOG_FIELDS`/`SPAN_FIELDS` (or add a new
  shared constant) — an inline field list in a route is now a review smell.
- The `"latest"` ban should hold: when TanStack is intentionally upgraded,
  bump the caret floors in the same commit (repo version policy: floors, not
  freezes).
- If drag-and-drop dashboards land later, re-add `@dnd-kit` THEN — with the
  feature, not ahead of it.
- Follow-up owned by the operator: confirm the lab OpenObserve/Sentry
  credentials were always compose-defaults; rotate them on those lab
  instances if not.
