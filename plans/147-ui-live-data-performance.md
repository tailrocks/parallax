# Plan 147: Make live telemetry updates typed, bounded, and identity-stable

> **Executor instructions**: Start only after feature ownership is stable and
> plan 133 has established one TanStack Query cache. Change live connection,
> buffering, merge, deduplication, polling, and render-input behavior only. Do
> not move feature files, redesign the UI, change Query ownership, or combine
> this work with bundle optimization. Preserve each feature's URL/filter/range
> behavior and use the real-stack Playwright lane to prove SSE behavior.
>
> **Drift check (run first)**:
> `git diff --stat e3e7997..HEAD -- ui/src ui/tests/e2e ui/test-matrix.json ui/package.json ui/vite.config.ts crates/parallax-server crates/parallax-xtask ratchet.toml`
> Resolve the baseline paths through final `platform`, `logs`, `traces`, and
> `runs` facades from plans 100, 140-142, 149, and 151. If the live protocol or
> server stream endpoints changed, re-characterize them before editing.

## Status

- **Priority**: P1
- **Effort**: L
- **Risk**: HIGH
- **Depends on**: 095, 101, 129, 133, 140, 141, 142, 145, 151
- **Category**: performance / correctness / live data
- **Planned at**: `e3e7997`, revised 2026-07-12
- **Status**: BLOCKED — upstream cache, feature, browser, and final UI plans are incomplete

## Contract reconciliation (2026-07-17)

Plans 156/157 rename the live correlation contract before this plan starts:
run-detail streams become invocation-hub streams
(`/v1/logs/stream?invocation_id=`, `/v1/traces/stream?invocation_id=`, plus
`session_id`), `runs.$runId.tsx` becomes `invocations.$invocationId.tsx`, and
the 10 s status poll targets the renamed `invocation` field. Two additional
live shapes exist (background-cycle and job spans inside the traces stream) —
they reuse trace/span identity, no new identity contract. Re-characterize the
baseline paths at the post-157 head; "run" reads as "invocation". See
plans/157-cli-invocation-observability-ui.md.

## Why This Matters

At the baseline, `ui/src/hooks/use-live-stream.ts:36-65` owns an EventSource,
an unbounded array between interval flushes, native reconnect state, and silent
parse-error suppression. Logs and traces prepend reversed arrays and slice on
every flush (`ui/src/routes/logs.tsx:301-311` and
`ui/src/routes/traces.index.tsx:375-383`). Run detail maintains two more streams,
re-sorts loaded plus live logs, and independently polls run state
(`ui/src/routes/runs.$runId.tsx:218-283`).

The visible caps limit final arrays but not arrival buffers, duplicate events,
work per burst, or error/reconnect observability. Typed bounded state and stable
identity are required before high-volume telemetry can be predictable for users,
reviewers, and automated agents.

## Fixed Decisions

1. The shared platform layer owns only EventSource lifecycle, visibility,
   bounded frame buffering, flush scheduling, connection state, and diagnostics.
   Feature modules own runtime decoding, item identity, ordering, deduplication,
   retention capacity, and Query/live reconciliation.
2. Every stream frame enters as `unknown`, passes its feature-owned schema
   instantiated through Plan 153, and becomes a domain value once.
   `JSON.parse(...) as T`, array casts,
   and silent malformed-frame catches are forbidden.
3. Connection state is a discriminated union covering disabled, connecting,
   open, reconnecting/degraded, and stopped/error conditions with attempt/
   diagnostic metadata where observable. Impossible boolean combinations are
   not represented.
4. Every feature declares immutable identity and total ordering. Spans use
   trace/span identity; logs use an approved collision-resistant domain key;
   run-correlated values reuse those identities. Do not invent a lossy log key
   if the server contract cannot distinguish valid repeated events.
5. Each stream declares `maxBufferedItems`, `maxVisibleItems`, flush policy, and
   overflow policy. Overflow increments an observable diagnostic and follows an
   explicit newest/oldest rule; memory growth is never the policy.
6. Merge functions are pure, readonly, deterministic, deduplicating, and linear
   in already ordered inputs. They preserve object references for unchanged
   items, never mutate `incoming` with `reverse()`/`sort()`, and never perform a
   full collection sort per flush.
7. Hidden pages own no EventSource or polling timer. URL/filter/range changes
   close the old stream, cancel pending work, reset only the feature state that
   current contracts reset, and cannot deliver a stale batch.
8. Classes are allowed only if the EventSource lifecycle genuinely needs one
   mutable invariant-bearing owner. Prefer a pure reducer plus an effect-owned
   adapter; do not create class-per-module wrappers for parsing or merging.
9. Wall-clock gates run in one declared canonical environment. Deterministic
   operation counts, capacity, identity, and cleanup are the cross-platform
   required gates; timing is a ratchet with machine data, not an ad hoc claim.
10. Plan 145's `full-stack/live-transport.spec.ts` and `@storage` stable IDs
    remain the one-event transport/lifecycle smoke. This plan adds distinct
    `@live` rows only for burst capacity, collision-safe identity/order,
    replay/duplicate handling, stale filter/generation rejection, retained
    ownership, and measured performance. It neither copies nor renames the
    `@storage` case.
11. Every `@live` row uses `features/logs`, `features/traces`, or `features/runs`
    as stable `scenario_owner`, `performance/live` as `lane_owner`, and 147 only
    as temporary `delivery_plan` while materializing it.

## Target Ownership

```text
ui/src/platform/sse/
  event-source-connection.ts     # lifecycle adapter
  stream-state.ts                # pure state transitions
  bounded-frame-buffer.ts        # capacity/overflow/flush contract
  tests/
ui/src/features/logs/
  api/log-stream-schema.ts
  model/log-identity.ts
  model/merge-live-logs.ts
  hooks/use-live-logs.ts
  tests/live/
ui/src/features/traces/
  api/span-stream-schema.ts
  model/merge-live-spans.ts
  hooks/use-live-traces.ts
  tests/live/
ui/src/features/runs/
  hooks/use-live-run.ts
  model/merge-run-telemetry.ts
  tests/live/
ui/tests/e2e/full-stack/
  logs-live-performance.spec.ts
  traces-live-performance.spec.ts
  runs-live-performance.spec.ts
ui/tests/e2e/support/live-performance.ts  # measurement helpers; no assertions
```

Use live paths produced by the feature plans when names differ. Do not retain
the baseline generic hook as a compatibility layer after all callers migrate.
Each full-stack spec contains only its feature owner's assertions and matrix
IDs. Shared support may collect typed measurements, but it cannot own a product
scenario, assertion, seed identity, or matrix row.

## Performance Contract

The executor must persist a machine-readable baseline and final ratchet with:

- active connection/timer count by visible surface;
- frame/item decode successes, rejects, buffered count, dropped count, flushes,
  reconnects, stale-generation rejects, and rendered item count;
- merge input/output counts, identity reuse, comparison count, allocations where
  measurable, and duration distribution;
- retained heap or deterministic live-object count after burst/reconnect cycles;
  and
- browser commit/input latency for the approved logs/traces/run scenarios.

Required invariants:

- zero connection/timer while hidden, disabled, or unmounted;
- one owned connection per active feature stream;
- buffered and visible counts never exceed declared capacities;
- merge comparison count is at most a documented constant multiple of `n + m`;
- unchanged domain objects retain reference identity;
- 10,000 ordered current items plus 1,000 incoming items merge within 16 ms p95
  in the canonical environment and no slower than 110% of the accepted final
  baseline; and
- after warmup, 100 burst/reconnect cycles retain no more than configured live
  capacity plus harness overhead and show no rising ownership count.

If the canonical runner cannot produce stable timing, keep the deterministic
gates required and STOP for approval of a different timing environment rather
than raising thresholds inside this plan.

## Commands

| Purpose | Command | Expected result |
|---------|---------|-----------------|
| Live unit/integration | `cd ui && bun run --bun test:ci -- live` | schemas, state, buffer, merge, hook, and feature tests pass |
| Live benchmark | `cd ui && bun run perf:live` | machine report satisfies capacity/operation/timing/identity ratchets |
| Browser contracts | `cd ui && bun run test:browser` | feature live UI contracts pass |
| Real stack | `cd ui && bun run test:browser:full -- --grep @live` | managed-stack ingest/reconnect/dedup/order cases pass |
| Policy | `cargo xtask policy --only ui.live-data` | ownership, schema, capacity, timer, identity, and old-hook rules pass |
| UI checks | `cd ui && bun run check && bun run lint && bun run typecheck && bun run build` | all exit 0 |
| Full aggregate | `cargo xtask ci --full` | exit 0 |

The exact Vitest filter may be implemented as a stable package script if Vitest
does not accept the shown positional filter through Bun. Do not introduce a
second test runner or mutable bunx command.

## Scope

In scope:

- Shared SSE lifecycle/state/bounded-buffer implementation and diagnostics.
- Logs, traces, and runs runtime schemas, identities, merge functions, live
  hooks, polling coordination, Query reconciliation, and tests.
- Deterministic operation/capacity/identity gates, canonical timing/heap
  harness, browser performance marks, and real-stack live cases.
- Deletion of the old generic hook and duplicated feature parse/merge paths.

Out of scope:

- File ownership moves (plans 100, 134-143, 149-151), Query/cache migration (plan
  133), route chunks/minification (plan 148), backend protocol redesign, metric
  trend implementation (plan 105), or product visual changes.
- WebSocket replacement, service workers, background streaming, unbounded
  history, blanket memoization, arbitrary debounce delays, or benchmark-only
  production branches.

## Git Workflow

- Stay on the one active branch; do not create a branch or PR.
- Land characterization, platform lifecycle/buffer, and one feature at a time as
  separate green commits; delete the old hook only after the final caller moves.
- Use Conventional Commits, DCO, and exactly one agent-product trailer.
- Push every durable green update.

## Steps

### Step 0: Characterize protocol, identities, and current work

Using plan 129's matrix and plan 145's real stack, record for logs, traces, and
runs: endpoint/filter URL, server ordering, duplicate/replay behavior, frame
shape, event ID/reconnect semantics, current visible caps, URL-change reset,
visibility behavior, error UI, and loaded/live reconciliation. Add bursts,
malformed frames, duplicate identities, equal timestamps, out-of-order values,
slow consumer, reconnect, filter change, hidden/visible, and unmount cases.

Instrument the existing implementation in a benchmark-only harness to capture
buffer high-water, flush work, sorts/reversals, object identity, timers,
connections, and retained state. Do not ship instrumentation in product code
until its final owner is defined.

Record plan 145's exact `@storage` live-transport ID/file/assertions as a
read-only prerequisite. Register separate `@live` IDs for only the new defect
classes above; policy rejects an assertion, seed identity, or stable ID owned by
both lanes. Materialize logs, traces, and runs rows only in their corresponding
feature-owned spec named under Target Ownership; reject a row whose
`scenario_owner` differs from its spec's feature owner. Shared support files
cannot appear as scenario evidence.

**Verify**: every live matrix row has an exact identity/order/capacity/reset/
diagnostic expectation and a reproducible baseline report.

### Step 1: Define typed identities and pure bounded merges

In each feature model, define a readonly identity function, total-order
comparator with deterministic tie-breaker, capacity/overflow policy, and pure
merge result containing values plus duplicate/drop counts. Consume already
ordered arrays and use a linear merge with a bounded identity set/index.

For logs, prove the selected key distinguishes legitimately repeated events. If
the wire contract lacks sufficient identity, STOP and request a versioned server
contract; do not hash a lossy subset and call collisions duplicates.

**Verify**: property tests cover arbitrary batches/order/duplicates/capacity and
prove determinism, idempotence, associativity where expected, reference reuse,
no input mutation, exact comparison bound, and stable equal-timestamp order.

### Step 2: Replace the shared connection lifecycle

Create the platform state reducer and effect adapter. Decode transport events
into typed raw frame outcomes, bound the frame/item buffer before allocation can
grow, flush using the declared scheduler, and attach a monotonically increasing
generation so a closed URL cannot deliver later work. Visibility/unmount closes
the EventSource and cancels timers/queued flushes.

Expose structured diagnostics to test/reporting and a bounded user-visible
state, without rendering raw malformed payloads. Preserve native EventSource
retry behavior only if Step 0 proves it satisfies the reconnect contract;
otherwise use the smallest explicit policy compatible with the server.

**Verify**: fake-timer/EventSource tests cover every state transition, burst
overflow, malformed frame, reconnect, generation change, hidden/visible,
unmount, callback change, timer cleanup, and duplicate event.

### Step 3: Migrate logs, traces, and runs independently

For each feature, parse with its runtime schema, call its pure merge, reconcile
the resulting values with the plan 133 Query/live owner, and preserve current
filter/range/reset behavior. Logs retain context/saved-view semantics; traces
retain list/live duration filtering; runs coordinate two streams and status
polling without sorting loaded plus live data on every render.

After each feature migrates, run its Vitest, fixture browser, and feature-owned
real-stack spec before starting the next. Remove duplicated `JSON.parse` casts,
`reverse()`/full-sort update paths, swallowed poll errors, and old imports for
that feature.

**Verify**: feature-specific command set passes, unchanged values preserve
references, and real ingest produces exactly one correctly ordered visible item
through disconnect/reconnect and filter changes.

### Step 4: Add durable performance and leak gates

Create a Bun-run deterministic generator and benchmark for small, cap-sized,
and 10k+1k batches with ordered, reversed, duplicate-heavy, and equal-time
inputs. Produce JSON with environment/tool versions, seed, counts, comparisons,
allocations/live objects, p50/p95, and threshold. Add a canonical Chromium
scenario that drives repeated bursts/reconnects while collecting performance
marks and bounded heap/live-owner evidence.

CI compares against checked-in ratchets. Update ratchets only with before/after
evidence and an owner/reason; thresholds shrink or require a separate approved
plan. Benchmark output is never a source file and never affects production
behavior.

**Verify**: intentional quadratic merge, identity churn, buffer overflow, leaked
timer/connection, rising retained ownership, and timing regression fixtures all
fail the owning diagnostic.

### Step 5: Delete compatibility paths and close policy

Delete the old generic hook, duplicated route parsing/merge/polling, temporary
instrumentation, and compatibility exports after every caller moves. Add Oxc/
xtask policy for runtime schema use, explicit capacity, allowed timer/EventSource
owners, no live parsing in routes/components, no mutating reverse/sort on
incoming arrays, and no unowned full-sort update.

Update `ui/AGENTS.md` with the live-data placement decision table and commands.

**Verify**: Oxc graph finds no old/deep caller, all negative policy fixtures
fail correctly, and the complete command table passes twice from clean state.

## Test Plan

- Runtime-schema valid/malformed/oversized frame tests.
- Pure identity/order/merge properties and operation/capacity/reference ratchets.
- State-machine and hook lifecycle tests with fake EventSource/timers/visibility.
- Feature tests for URL/filter reset, Query reconciliation, status/error, loaded
  plus live order, duplicates, and polling coordination.
- Fixture-backed browser behavior plus real Greptime ingest/reconnect/dedup/order.
- Matrix ownership negatives proving `@storage` remains the one-event smoke and
  each feature-owned `@live` spec alone owns its feature's burst/capacity/
  identity/filter-reset/performance evidence; shared support owns no row.
- Canonical timing, retained ownership, and intentional regression fixtures.

## Done Criteria

- [ ] Every live frame is decoded from `unknown` once and malformed/overflow
  events are bounded and observable.
- [ ] Logs, traces, and runs have explicit collision-safe identity, total order,
  capacity, overflow, reset, and Query reconciliation contracts.
- [ ] Hidden/disabled/unmounted surfaces own zero connection/timer and stale
  generations cannot deliver.
- [ ] Pure merges are deterministic, linear, non-mutating, deduplicating,
  capacity-bounded, and reference-stable.
- [ ] Deterministic and canonical timing/leak ratchets pass, including 10k+1k
  p95 at or below the declared threshold.
- [ ] Distinct `@live` real-stack cases prove bounded bursts and stable identity/
  order through replay, reconnect, duplication, and filter-generation change.
- [ ] Old generic/route-level parse-merge-poll paths are deleted.

## STOP Conditions

Stop and report if:

- logs or another signal lacks a collision-safe identity needed for deduplication;
- the server's order/reconnect behavior cannot be characterized through public
  boundaries;
- satisfying live semantics requires unbounded memory, background streaming, a
  second cache, or a backend protocol change;
- benchmark timing is unstable in the canonical environment or meeting the
  ceiling requires changing product results;
- real-stack tests cannot distinguish a replay from a duplicate;
- a proposed `@live` case duplicates plan 145's one-event `@storage` seed,
  stable ID, or sole transport/lifecycle assertion instead of testing a new
  boundedness/identity/reset/performance contract;
- a source move, bundle change, or broad UI redesign becomes necessary; or
- a required gate fails twice after a reasonable correction.

## Maintenance And Removal

New live features must declare schema, identity, order, capacities, overflow,
reset, lifecycle, diagnostics, Query reconciliation, deterministic properties,
and real-stack evidence in the same change. Reviewers reject silent parse/drop,
full-sort-on-flush, object churn, and unowned timers/connections.

Delete this plan and its README row only after all three features migrate, old
paths are removed, performance/leak ratchets and real-stack cases are durable,
and every command is green.
