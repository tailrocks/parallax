# Plan 099: Make boundaries typed, idempotent, and agent-safe

> **Executor instructions**: Stabilize ownership through plans 097/098 first.
> Migrate one capability at a time while preserving current human messages.
> Do not expose internal errors, sweep all string IDs, add hot-path coordination
> without measurement, or let an agent transport bypass redacted bundles.

## Status

- **Priority**: P1
- **Effort**: L
- **Risk**: HIGH
- **Depends on**: 097, 098
- **Category**: boundary correctness / security
- **Planned at**: `eefa4617`, 2026-07-12
- **Status**: IN PROGRESS

## Why

Library ports return `anyhow::Result`, GraphQL collapses failures to display
strings, the worker retries a multi-effect operation, and late span/log echoes
have only bounded in-batch dedup. Raw GraphQL/SSE are intentionally unredacted
for the local human UI, but must never become an agent context path. Roughly 97
identifier sites remain primitive strings, making one boundary-first pilot
useful after error ownership stabilizes.

## Scope

In scope:

- Typed storage/config/metadata/server error taxonomies.
- Central GraphQL error extensions and sanitization.
- Explicit worker stage/retry/idempotency contracts.
- Cross-source `error_type` normalization so one failure is not split by
  synthetic span/log channel labels.
- Durable late-echo occurrence idempotency design and implementation.
- One `RunId` or `TraceId` pilot.
- Policy/contract preventing agent access to unredacted raw surfaces.

Out of scope:

- Workspace-wide newtype or typestate sweep.
- Redacting/changing the local human GraphQL/UI contract by default.
- A product MCP implementation.
- Custom raw-signal tables or hot-path telemetry cloning.

## Steps

### Step 1: Add typed library errors

Introduce `thiserror` enums by capability: storage transport/query/schema,
metadata, configuration, and server startup/lifecycle. Preserve source chains
and current human context. Keep `anyhow` only at CLI, xtask, and genuinely
top-level reporting composition with reasoned exceptions.

### Step 2: Centralize client error mapping

Map typed variants to stable GraphQL extensions: invalid input, not found,
conflict, unavailable, timeout, and internal. Snapshot codes/messages and never
expose SQL, paths, credentials, or source chains to clients.

### Step 3: Split worker effect/retry boundaries

Use plan 093's failure-injection oracle. Define ordering, retryability, and
idempotency for registration, broadcast, telemetry persistence, and issue
recording. Completed effects are not replayed after a later failure. Persist or
checkpoint only the minimum needed state; progress/error output remains clear.

### Step 4: Normalize cross-source issue identity

Characterize one structured failure observed as span exception, span status,
log exception, and log severity. Define a stable error-type selection order and
normalize synthetic `span_error`/`log_error` labels before fingerprinting when
structured type evidence exists. Preserve source/channel as evidence fields.
Golden tests must prove the same failure groups once while genuinely different
types/messages/frames remain separate.

### Step 5: Make late echoes durable

Update the implementation spec before schema/code. Design a deterministic
occurrence identity for late span/log repeats, preserving distinct legitimate
occurrences. Evaluate the transaction/retention owner in Turso metadata versus
the derived Greptime extension path; choose from live evidence, not convenience.

Add restart and retry tests for `(trace_id, span_id/log identity, fingerprint)`
duplicates, bounded retention, and concurrent delivery. Do not place mutable
dedup state in Greptime native raw tables.

### Step 6: Pilot one ID type

Choose `RunId` or `TraceId` based on the highest measured confusion/validation
value. Put it in `parallax-model`, validate at GraphQL/CLI/OTLP boundaries, and
keep wire/persisted representations compatible. Measure the remaining primitive
frontier before proposing another plan.

### Step 7: Enforce the agent trust boundary

Document and gate that future agent transports consume only the bounded,
redacted, schema-valid bundle projection. Product MCP/agent code may not call
raw GraphQL/SSE or storage query surfaces. Add architecture/policy fixtures
that fail on such a dependency. Human-local raw surfaces remain a separate
explicit trust domain.

## Test Plan

- Variant/source-chain tests for each typed error.
- GraphQL error-extension snapshots and internal-data leak negatives.
- Failure injection at every worker stage and restart boundary.
- Durable duplicate/concurrent/retention tests on real Turso/Greptime as chosen.
- ID parse/serde/GraphQL/wire compatibility tests.
- Agent-boundary architecture/policy negative fixtures.

## Done Criteria

- [ ] Library capabilities expose typed errors; approved `anyhow` edges are
  enumerated and ratcheted.
- [ ] GraphQL maps/sanitizes variants centrally.
- [ ] No late failure replays a completed worker effect.
- [ ] Cross-source views of one structured failure share stable issue identity.
- [ ] Late echoes are idempotent across restart without collapsing distinct
  occurrences.
- [ ] One ID pilot preserves wire/persistence compatibility.
- [ ] Agent transports are structurally restricted to redacted bundles.
- [ ] Full workspace, real storage, SDL, and strict lint gates pass.

## STOP Conditions

- Error migration requires breaking GraphQL/CLI contracts without a spec change.
- Durable dedup adds unbounded state or clones telemetry on the hot path.
- No store can provide the required identity/atomicity without cross-engine
  distributed coordination.
- An ID pilot changes public wire or stored bytes.
- Agent safety depends only on prose or reviewer memory.

## Remove When

Delete this plan and row after typed boundaries, retry/dedup behavior, one ID
pilot, and agent-surface enforcement have exact green evidence.
