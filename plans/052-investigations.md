# Plan 052: Investigations — save/restore an investigation state (Turso CRUD cloned from dashboards) + pin affordances

> **Executor instructions**: Follow step by step; run every verification. On
> any STOP condition, stop and report. When done, update the status row in
> `plans/README.md`.
>
> **Drift check (run first)**:
> `git diff --stat 408be17..HEAD -- crates/parallax-storage/src/metadata.rs crates/parallax-api/src/lib.rs ui/src`
> On mismatch with the excerpts below, STOP.

## Status

- **Priority**: P3
- **Effort**: M
- **Risk**: LOW
- **Depends on**: plans/038 (URL state IS the captured state — land first)
- **Category**: direction
- **Planned at**: commit `408be17`, 2026-07-07

## Why this matters

The brief's "Investigations/cases" section (I): users (and coding agents)
need to preserve an investigation — time window, filters, pinned
traces/logs/issues/runs, notes — instead of losing it to browser history.
The storage/mutation pattern is already proven end-to-end by dashboards
(Turso row with a JSON blob + save/delete mutations + list query). Because
plans 035/038 put every view's state into the URL, an investigation is
mostly a **named collection of URLs + pins + notes** — deliberately simple
V1.

## Current state

Verified at commit `408be17`.

- The pattern to clone — Turso schema
  (`crates/parallax-storage/src/metadata.rs:31-37`):

  ```sql
  CREATE TABLE IF NOT EXISTS dashboards (
    id          TEXT PRIMARY KEY,
    name        TEXT NOT NULL,
    layout      TEXT NOT NULL,
    created_at  INTEGER NOT NULL,
    updated_at  INTEGER NOT NULL
  );
  ```

  accessors `dashboard_save` (`metadata.rs:519`), `dashboards`
  (`metadata.rs:552`), delete nearby; GraphQL mutations
  `dashboard_save` (`crates/parallax-api/src/lib.rs:1872`),
  `dashboard_delete` (`:1902`); UI CRUD flow in
  `ui/src/routes/dashboards.index.tsx` (create dialog `:188-208`, list
  cards `:289-314`).
- No investigations table exists (`metadata.rs:10-44` schema block).
- URL state: after plan 038, every list/detail view serializes its window +
  filters into search params; TanStack `useRouterState` gives the current
  location for capture.
- Nav shell: `ui/src/components/parallax-shell.tsx` + nav config
  `ui/src/components/nav.ts` — add the Investigations entry the same way
  dashboards appear.

## Commands you will need

| Purpose | Command | Expected |
|---------|---------|----------|
| Build | `rtk cargo build --workspace` | exit 0 |
| Lint | `rtk cargo clippy --workspace --all-targets` | zero warnings |
| Tests | `rtk cargo nextest run` | all pass |
| UI | (from `ui/`) `bun run typecheck && bun run lint && bun run test && bun run build` | exit 0 |

## Scope

**In scope**:
- `crates/parallax-storage/src/metadata.rs` (table + CRUD)
- `crates/parallax-api/src/lib.rs` (`investigations`, `investigation`,
  `investigationSave`, `investigationDelete`)
- `ui/src/lib/api.ts`; new route `ui/src/routes/investigations.index.tsx`
  (+ `investigations.$id.tsx`); `ui/src/components/nav.ts`; a small
  "Pin to investigation" action component used on trace/issue/run detail
  headers
- test files

**Out of scope**:
- Query-history capture, bundle-preview history, evidence-gap lists inside
  investigations (brief's fuller vision — the JSON schema leaves room).
- Multi-user/sharing semantics (local-first single operator).
- Auto-capture/suggestions.

## Git workflow

- `main`, Conventional Commits, `git commit -s`, one

## Steps

### Step 1: Define the state JSON (small design decision, in-code)

`InvestigationState` (serde in Rust mirrors TS type):

```jsonc
{
  "version": 1,
  "window": { "range": "custom|<preset>", "from": "...", "to": "..." },
  "pins": [ { "kind": "trace|issue|run|log|view", "ref": "<id or URL search-string>", "label": "...", "note": "..." } ],
  "notes": "markdown text"
}
```

Store as an opaque `state TEXT` column (same philosophy as dashboards'
`layout`). Server validates: parses as JSON, `version == 1`, pins ≤ 100,
notes ≤ 64KiB — reject otherwise (mirror `dashboard_save`'s layout
validation approach at `lib.rs:1872+` — read how it validates and match the
error style).

### Step 2: Turso table + CRUD

`investigations` table: `id, name, state, created_at, updated_at` (clone
the dashboards DDL shape at `metadata.rs:31-37`); accessors
`investigation_save` (upsert), `investigations` (list, newest first),
`investigation` (by id), `investigation_delete` — model each on the
dashboard functions (`:519+`, `:552+`). Add to the schema-bootstrap block
where the other CREATE TABLEs live (`:10-44`).

**Verify**: `rtk cargo nextest run` — metadata tests (model on existing
dashboard CRUD tests — grep `dashboard` in the storage test modules; if
none exist, write the first: save→get→list→delete round-trip against the
in-memory/temp Turso the tests use — check how existing metadata tests
construct the store).

### Step 3: GraphQL surface

Query `investigations: [Investigation!]!`, `investigation(id: String!)`;
mutations `investigationSave(id: String, name: String!, state: String!): Investigation!`
(id absent → create) and `investigationDelete(id: String!): Boolean!` —
mirror the dashboard resolvers' shapes/naming (`:1872`, `:1902`).

**Verify**: resolver tests green; `rtk cargo build` + clippy clean.

### Step 4: UI — list, detail, pin action

1. `investigations.index.tsx`: list cards (name, pin count, updated),
   create dialog (name only), delete with confirm — copy the dashboards
   index page's structure wholesale (`dashboards.index.tsx`) including the
   error-state pattern from plan 035.
2. `investigations.$id.tsx`: renders the window (link applying it to `/`),
   pins as a list of links (each `kind` icon + label + note inline-editable
   textarea), and a notes textarea; Save persists via `investigationSave`.
3. "Pin" action: a small component (`ui/src/components/console/pin-button.tsx`)
   with a popover: pick an investigation (or create by name) → appends a pin
   with `kind` + `ref` + current URL search; mount it on the trace detail,
   issue detail, and run detail headers. Current-view capture uses
   `useRouterState` location (path + search string) as the `view` pin ref.
4. Nav entry in `nav.ts` (icon consistent with the set used there).

**Verify**: (from `ui/`) all four bun gates exit 0. Manual: pin a trace +
an issue, save, reload, restore each pin → correct pages with correct
windows (record).

## Test plan

- Storage round-trip test (Step 2), resolver tests (Step 3).
- UI: a component test for pin-serialization (given a location, produces
  the expected pin JSON) — pure helper, test like `kit.test.tsx`.
- State-validation tests server-side: bad JSON rejected, >100 pins
  rejected.

## Done criteria

- [ ] cargo build/clippy/nextest green including new metadata + resolver
      tests
- [ ] investigations CRUD live end-to-end; state validated server-side
- [ ] /investigations list + detail routes; pin buttons on 3 detail pages;
      restore round-trip recorded
- [ ] Delete confirm-gated (matches plan 035's dashboard treatment)
- [ ] UI gates exit 0
- [ ] `plans/README.md` status row updated

## STOP conditions

- The metadata store's migration story can't add a table idempotently
  (check how the existing CREATE TABLEs bootstrap — if there's a migration
  version mechanism, follow it; if bare IF NOT EXISTS, that's the pattern)
  — report only if neither applies cleanly.
- Pin refs for logs turn out unstable (logs lack a durable id — plan 040's
  `_key` is client-only): pin logs as a **view** pin (the filtered URL),
  never a row ref; if that compromise fails a use case, report it.

## Maintenance notes

- Future: bundle-preview pins + evidence-gap snapshots (advisor-plans
  023/032 outputs) extend the state JSON via `version` bumps.
- The state blob is the API for future MCP/agent access ("human-readable
  case file" per the brief) — keep it stable and documented in the code.
- Reviewer: XSS discipline on notes rendering (plain text/escaped —
  markdown rendering only if an existing sanitizer is already in the repo);
  pin restore must tolerate deleted targets (dead trace id → graceful
  empty state, which existing routes already provide).
