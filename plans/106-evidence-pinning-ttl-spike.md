# Plan 106: Design and validate evidence pinning beyond telemetry TTL

> **Executor instructions**: Start as a storage-semantics spike against live
> GreptimeDB and Turso. Do not copy raw signals into custom tables or promise
> indefinite retention before ownership, deletion, and cost behavior are
> explicit. Implement only the smallest design that receives a GO decision.

## Status

- **Priority**: P2
- **Effort**: M
- **Risk**: HIGH
- **Depends on**: 092, 104, 116
- **Category**: evidence retention / storage / product contract
- **Planned at**: `eefa4617`, 2026-07-12
- **Status**: BLOCKED — Plan 104 approval and Plan 116 lifecycle approval are absent

## Why

The M5 direction calls for evidence bundles to remain useful after raw
telemetry TTL, but no retention mechanism exists. Pinning crosses native-table
TTL, Turso metadata, bundle versioning, redaction, deletion, storage cost, and
agent-access boundaries. A naive raw-signal copy would violate the native-table
rule and create a second observability store.

## Questions The Spike Must Answer

- What is pinned: immutable sanitized bundle bytes, canonical evidence facts,
  native row references, or a combination?
- Which actor pins/unpins, for how long, and under what authorization?
- What happens when native telemetry expires, is redacted, or must be deleted?
- How are bundle version/hash, provenance, partial evidence, and missing raw
  references represented?
- Which bytes live in Turso versus a future approved artifact store, and what
  are size/retention limits?

## Scope

- Characterize live Greptime TTL and Turso transaction/storage behavior.
- Define retention ownership, lifecycle, bounds, security, and deletion.
- Compare at least sanitized immutable bundle snapshots, compact derived
  evidence records, and native references with explicit tradeoffs.
- Record a GO/NO-GO decision; on GO, implement and test the smallest approved
  V1 path in this plan.

Out of scope:

- Custom raw log/trace/metric/profile tables.
- Silently disabling native table TTL or duplicating unrestricted raw payloads.
- Object-store adoption without a separate approved stack decision.
- Agent access to unsanitized raw GraphQL/SSE data.

## Steps

### Step 1: Pin current contracts and threats

Use plan 104's canonical bundle version and plan 092's exemplar schema. Define
actors, authorization assumption, secret/PII redaction, maximum object/count,
retention/default expiry, deletion semantics, provenance, and behavior when
source telemetry is partially or fully gone.

### Step 2: Run live storage experiments

Against latest stable and latest nightly GreptimeDB where feasible, verify TTL
expiration timing, reference stability, native table identifiers, snapshot
isolation, and relevant extension points. Against current Turso, measure atomic
metadata+payload writes, size/read latency, concurrent pin/unpin, failed
transactions, and recovery. Follow the mandated native-table research and
Greptime consultation path before proposing any unsupported design.

### Step 3: Compare designs and decide

Score each option for native-table compliance, ability to survive TTL,
redaction/version correctness, deterministic hashing, transactionality,
deletion, storage amplification, offline export, and operational complexity.
Record the source evidence, measured limits, rejected alternatives, and an
explicit GO/NO-GO. NO-GO must name the missing upstream/product prerequisite.

### Step 4: Implement only after GO

Add an explicit bounded pin manifest and storage capability. Store only the
approved sanitized/versioned representation, with idempotent pin/unpin,
retention metadata, provenance/hash verification, partial-source state, and
transactional recovery. Keep raw signals in native Greptime tables. Expose
only approved CLI/API operations and clear progress/output for long work.

### Step 5: Prove lifecycle behavior

Test source present, source expired, partial expiration, repeated pin,
concurrent pin/unpin, failed transaction/restart, redaction change, bundle
version change, retention expiry, explicit delete, corrupt bytes/hash, and
maximum bounds. Demonstrate that deleting a pin does not alter native telemetry
and native TTL does not corrupt the pinned representation.

## Test Plan

- Reproducible live Greptime/Turso spike scripts and recorded results.
- Contract/threat/design decision review.
- On GO: unit, adapter, transaction/restart, corruption, authorization, TTL,
  redaction, version, idempotency, and deletion tests.
- Storage amplification and maximum-size measurements.
- Verification that no custom raw-signal table or unredacted agent path exists.

## Done Criteria

- [ ] Live evidence answers TTL, identity, transaction, size, and recovery questions.
- [ ] A reviewed GO/NO-GO resolves the representation and lifecycle explicitly.
- [ ] The chosen path obeys native-table, bundle, redaction, and access policy.
- [ ] On GO, bounded idempotent pin/unpin survives restart and source expiry.
- [ ] Provenance/hash/version and partial/missing source state are verifiable.
- [ ] Expiry/delete/corruption behavior and storage cost are tested and documented.
- [ ] No custom raw-signal table or unredacted agent-access path is introduced.

## STOP Conditions

- The canonical evidence contract from plan 104 is unresolved.
- A design copies raw signals into a custom table or silently disables TTL.
- Retention, deletion, authorization, maximum size, or redaction semantics are
  unspecified.
- Live storage behavior contradicts the design and lacks an upstream resolution.
- An object store or new engine is required without operator approval.

## Remove When

Delete this plan and index row after a source-backed GO/NO-GO is recorded and,
for GO, the approved bounded lifecycle is implemented and verified. If NO-GO
leaves a concrete external trigger, replace this file with a minimal BLOCKED
plan in the same commit rather than keeping completed spike steps.
