# Plan 071: Fix four confirmed UI bugs — cycle-safe error path, stale-window log paging, bucket race, control-char escaping

> **Executor instructions**: Follow this plan step by step. Run every
> verification command and confirm the expected result before moving to the
> next step. If anything in the "STOP conditions" section occurs, stop and
> report — do not improvise. When done, update the status row for this plan
> in `advisor-plans/README.md` — unless a reviewer dispatched you and told you
> they maintain the index.
>
> **Drift check (run first)**: `git diff --stat dbaba3c..HEAD -- ui/src/lib/trace-tree.ts ui/src/routes/logs.tsx "ui/src/routes/issues.\$fingerprint.tsx" ui/src/lib/api.ts`
> If any in-scope file changed since this plan was written, compare the
> "Current state" excerpts against the live code before proceeding; on a
> mismatch, treat it as a STOP condition. Note: `ui/src/routes/logs.tsx` had
> uncommitted edits at planning time — verify the excerpts match whatever is
> live.

## Status

- **Priority**: P1
- **Effort**: M (four small fixes + tests)
- **Risk**: LOW
- **Depends on**: none (069 recommended first so the new tests run in CI)
- **Category**: bug
- **Planned at**: commit `dbaba3c`, 2026-07-10

## Why this matters

Four confirmed defects in the web console:

1. A self-referential or cyclic `parentSpanId` in a trace hangs the browser
   tab in an infinite loop — two sibling helpers already defend against this,
   one doesn't.
2. Changing the time range while a "Load older" fetch is in flight splices
   rows from the old window into the new one, corrupting the list and the
   paging cursor.
3. Rapidly clicking issue histogram bars can display events for a different
   bucket than the one highlighted (last-response-wins race).
4. The hand-rolled GraphQL string escaper passes ASCII control characters
   through raw, which produces invalid GraphQL and fails the whole request
   (realistic via pasted SQL in the SQL workbench).

## Current state

### Bug 1 — unguarded parent-chain walk

- `ui/src/lib/trace-tree.ts:144-163`:

  ```ts
  export function errorPathSpanIds<T extends ErrorTraceSpan>(
    spans: readonly T[]
  ): Set<string> {
    const byId = new Map(spans.map((span) => [span.spanId, span]))
    const ids = new Set<string>()

    for (const span of spans) {
      if (span.statusCode !== "STATUS_CODE_ERROR") continue

      let current: T | undefined = span
      while (current) {
        ids.add(current.spanId)
        current = current.parentSpanId
          ? byId.get(current.parentSpanId)
          : undefined
      }
    }

    return ids
  }
  ```

  No visited-set. Called from a `useMemo` in
  `ui/src/components/console/trace-waterfall.tsx` on every trace render.

- The defensive pattern to mirror already exists in
  `ui/src/lib/graphql-trace.ts:131-144` (`nearestOperationSpanId`):

  ```ts
  let parentId = span.parentSpanId
  const seen = new Set<string>()
  while (parentId && !seen.has(parentId)) {
    if (operationIds.has(parentId)) return parentId
    seen.add(parentId)
    parentId = byId.get(parentId)?.parentSpanId ?? null
  }
  ```

### Bug 2 — `loadOlder` stale-window append

- `ui/src/routes/logs.tsx` — the loader-reset effect (~line 281):

  ```ts
  useEffect(() => {
    setLogs(keyedDataLogs)
    setExhausted(keyedDataLogs.length < PAGE_SIZE)
  }, [keyedDataLogs])
  ```

  and `loadOlder` (~line 388), which captures `logs.at(-1)` + `range` and,
  after `await graphql(...)`, unconditionally does:

  ```ts
  setLogs((current) => [...current, ...assignLogKeys(more.logs)])
  if (more.logs.length < PAGE_SIZE) setExhausted(true)
  ```

  There is no request token/abort: if the range, service, severity, or query
  changes (loader re-runs, reset effect replaces `logs`) while the fetch is in
  flight, the late result is appended to the new window's rows.

### Bug 3 — bucket filter race

- `ui/src/routes/issues.$fingerprint.tsx:240-267` (`filterBucket`):

  ```ts
  async function filterBucket(tsNanos: string | null) {
    setActionError(null)
    setBucket(tsNanos)
    if (!tsNanos) { setBucketEvents(null); return }
    try {
      const from = BigInt(tsNanos)
      const to = from + 3_600_000_000_000n
      const { issue: scoped } = await graphql<...>(...)
      setBucketEvents(scoped?.events ?? [])
      ...
  ```

  `setBucket` is immediate, the fetch is not cancelled; two rapid clicks →
  whichever response resolves last populates `bucketEvents`, regardless of
  the currently selected `bucket`.

### Bug 4 — `gqlString` misses control characters

- `ui/src/lib/api.ts:34-43`:

  ```ts
  export function gqlString(value: string): string {
    // GraphQL string literals cannot contain raw newlines (multi-line SQL).
    return value
      .replace(/\\/g, "\\\\")
      .replace(/"/g, '\\"')
      .replace(/\n/g, "\\n")
      .replace(/\r/g, "")
      .replace(/\t/g, "\\t")
  }
  ```

  U+0000–U+0008, U+000B, U+000C, U+000E–U+001F pass through raw; those are
  not valid GraphQL SourceCharacters inside a quoted string, so the server
  rejects the request with a parse error.

### Conventions

- Strictest TypeScript (`noUncheckedIndexedAccess`, `exactOptionalPropertyTypes`);
  `bun run typecheck` must pass.
- Tests: vitest + @testing-library under `__tests__/` folders; pure-lib tests
  live in `ui/src/lib/__tests__/` (e.g. `trace-tree.test.ts` exists there —
  extend it). Route tests use the `-` filename prefix
  (`ui/src/routes/__tests__/-logs.test.tsx`).
- All data via `graphql()` from `ui/src/lib/api.ts`; nanosecond timestamps are
  strings, compared via `BigInt`.

## Commands you will need

All from `ui/`:

| Purpose | Command | Expected on success |
|---------|---------|---------------------|
| Install | `rtk bun install --frozen-lockfile` | exit 0 |
| Typecheck | `rtk bun run typecheck` | exit 0 |
| Tests | `rtk bun run test` | all pass |
| One file | `rtk bunx vitest run src/lib/__tests__/trace-tree.test.ts` | pass |
| Lint | `rtk bun run lint` | exit 0 |

## Scope

**In scope** (the only files you should modify):
- `ui/src/lib/trace-tree.ts`
- `ui/src/lib/__tests__/trace-tree.test.ts`
- `ui/src/routes/logs.tsx`
- `ui/src/routes/issues.$fingerprint.tsx`
- `ui/src/lib/api.ts`
- `ui/src/lib/__tests__/` (new/extended test files for api.ts if none exists)
- `advisor-plans/README.md` (status row)

**Out of scope** (do NOT touch, even though they look related):
- The SSE/EventSource effects in `logs.tsx`, `traces.index.tsx`,
  `runs.$runId.tsx` — Plan 077 consolidates those into a shared hook; don't
  pre-refactor them here.
- `ui/src/lib/graphql-trace.ts` / `rpc-trace.ts` — already guarded; reference
  only.
- Migrating `gqlString` callers to GraphQL variables — known deferred debt,
  separate decision.

## Git workflow

- Work directly on `main` (repo rule — `BRANCHING.md`).
- Conventional Commits, DCO signoff (`git commit -s`), trailer
  `Co-authored-by: Claude <noreply@anthropic.com>`. Example:
  `fix(ui): guard error-path walk against span cycles`.

## Steps

### Step 1: Cycle-guard `errorPathSpanIds`

In `ui/src/lib/trace-tree.ts`, add a per-walk visited check. Simplest correct
form — stop when the next node was already added:

```ts
let current: T | undefined = span
while (current && !ids.has(current.spanId)) {
  ids.add(current.spanId)
  current = current.parentSpanId ? byId.get(current.parentSpanId) : undefined
}
```

Note `ids` is shared across the outer loop — that is fine and also an
optimization (paths that merge stop early), and it terminates cycles because
every revisited node is already in `ids`.

Add tests in `ui/src/lib/__tests__/trace-tree.test.ts` (file exists — follow
its fixture style): (a) a span whose `parentSpanId === its own spanId`,
(b) a two-node cycle A→B→A where one is `STATUS_CODE_ERROR` — both must
return in finite time with the expected id set.

**Verify**: `rtk bunx vitest run src/lib/__tests__/trace-tree.test.ts` → all pass.

### Step 2: Token-guard `loadOlder` in logs.tsx

Add a generation ref next to the existing state in the route component:

```ts
const logsGeneration = useRef(0)
```

Increment it in the reset effect (the one that does `setLogs(keyedDataLogs)`),
and capture/compare in `loadOlder`:

```ts
const generation = logsGeneration.current
// ...await graphql(...)
if (logsGeneration.current !== generation) return   // window changed; drop result
setLogs((current) => [...current, ...assignLogKeys(more.logs)])
if (more.logs.length < PAGE_SIZE) setExhausted(true)
```

Also skip the `setExhausted(true)` when stale (it is inside the guarded
block above). Leave `finally { setOlderLoading(false) }` unguarded.

**Verify**: `rtk bun run typecheck` → exit 0; `rtk bun run test` → the
existing `-logs.test.tsx` suite still passes.

### Step 3: Guard the bucket fetch in issues.$fingerprint.tsx

In `filterBucket`, after the `await`, only commit the result if the clicked
bucket is still the selected one. Track the in-flight target with a ref:

```ts
const bucketRequestRef = useRef<string | null>(null)
// in filterBucket, before the await:
bucketRequestRef.current = tsNanos
// after the await:
if (bucketRequestRef.current !== tsNanos) return
setBucketEvents(scoped?.events ?? [])
```

Also set `bucketRequestRef.current = null` in the `!tsNanos` early-return
branch so a pending fetch can't resurrect a cleared selection.

**Verify**: `rtk bun run typecheck` → exit 0; `rtk bun run test` → the
existing `-issues.test.tsx` suite still passes.

### Step 4: Complete `gqlString` control-char escaping

In `ui/src/lib/api.ts`, extend the chain (keep the existing replacements —
order matters, backslash first):

```ts
.replace(/[\u0000-\u0008\u000b\u000c\u000e-\u001f]/g, (c) =>
  "\\u" + c.charCodeAt(0).toString(16).padStart(4, "0")
)
```

appended as the last step of the chain. (`\n`, `\r`, `\t` are excluded from
the class because the existing replacements already handle them.) Add unit
tests (create `ui/src/lib/__tests__/api.test.ts` if absent — but check first: if
a test file for api.ts exists, extend it): backslash, quote, newline, tab keep
their existing behavior; a form-feed char becomes the six-char text `\u000c`; a NUL char becomes `\u0000`; a plain ASCII string is returned unchanged.

**Verify**: `rtk bunx vitest run src/lib/__tests__/api.test.ts` → all pass.

### Step 5: Full gates

**Verify**: from `ui/`: `rtk bun run typecheck` → 0, `rtk bun run lint` → 0,
`rtk bun run test` → all pass (including the new tests).

## Test plan

- `trace-tree.test.ts`: 2 new cycle tests (Step 1).
- `api.test.ts`: 4+ new escaping tests (Step 4).
- Bugs 2 and 3 are interaction races — no new route tests required (the route
  test harness has no async-race fixture pattern to follow); the guard logic
  is trivial enough that typecheck + existing route suites are the gate. If
  you can add a focused race test cheaply following an existing pattern in
  `ui/src/routes/__tests__/`, do so; do not build new test infrastructure.

## Done criteria

Machine-checkable. ALL must hold:

- [ ] `grep -n "ids.has" ui/src/lib/trace-tree.ts` → 1 match
- [ ] `grep -n "logsGeneration" ui/src/routes/logs.tsx` → ≥2 matches
- [ ] `grep -n "bucketRequestRef" "ui/src/routes/issues.\$fingerprint.tsx"` → ≥2 matches
- [ ] `grep -n "u0000" ui/src/lib/api.ts` → ≥1 match
- [ ] From `ui/`: `rtk bun run typecheck`, `rtk bun run lint`, `rtk bun run test` all exit 0
- [ ] `git status` shows no modified files outside the in-scope list
- [ ] `advisor-plans/README.md` status row updated

## STOP conditions

Stop and report back (do not improvise) if:

- The excerpts don't match live code (especially `logs.tsx`, which had
  uncommitted edits at planning time — if `loadOlder` or the reset effect has
  been restructured, report what you found instead).
- The existing `-logs.test.tsx` or `-issues.test.tsx` suites fail BEFORE your
  change (pre-existing breakage — report, don't fix).
- `exactOptionalPropertyTypes` or `noUncheckedIndexedAccess` produce errors
  that require changing shared types to satisfy — that widens scope.

## Maintenance notes

- Bug 2's pattern (generation token around await) should be reused by any
  future "load more" affordance; if Plan 077's `useLiveStream` hook grows a
  paging sibling, fold this token pattern into it.
- Bug 4 is a stopgap on the hand-rolled client; the durable fix is GraphQL
  variables/codegen (recorded as deferred debt in `advisor-plans/README.md`).
- Reviewer: in Step 1 confirm the shared-`ids` early-stop doesn't change the
  returned set for legitimate merged paths (it doesn't — ancestors are already
  present by definition).
