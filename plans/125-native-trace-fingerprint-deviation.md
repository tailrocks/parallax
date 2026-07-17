# Plan 125: Resolve the native trace fingerprint deviation

> **Executor instructions**: Treat GreptimeDB's native trace tables as the raw
> signal authority. Do not create a custom raw trace table, clone decoded OTLP,
> or claim the `fingerprint` column is useful while it remains null. Decide the
> correlation contract from live queries and engine behavior before code.

## Status

- **Priority**: P2
- **Effort**: M
- **Risk**: HIGH
- **Depends on**: 093, 097, 099, 104
- **Category**: storage / native tables / correlation correctness
- **Planned at**: `eefa4617`, 2026-07-12
- **Status**: BLOCKED — Plan 104 canonical evidence-contract approval and
  required live stable/nightly GreptimeDB migration evidence are unavailable

## Why

`try_traces_deviations` adds a `fingerprint` column to GreptimeDB's native
`opentelemetry_traces` table, but the OTLP forwarding path never populates it.
The derived `error_events` table already stores fingerprint, trace ID, and span
ID. The null native column therefore advertises an unfinished correlation path,
adds schema drift, and leaves future queries unsure which relation is canonical.

## Current Evidence

- `crates/parallax-storage/src/greptime.rs` applies the column deviation after
  native table creation and identifies population as deferred ingest work.
- Raw traces are forwarded once to GreptimeDB's native OTLP endpoint; Parallax
  must not decode/clone a second raw representation to fill a custom schema.
- Derived error events carry the computed fingerprint and trace/span anchors.
- No current product query was found reading the native trace `fingerprint`
  column; this must be confirmed with structured query/consumer inventory.
- Existing data directories may already contain the nullable column, so merely
  removing fresh-install DDL is not a complete migration decision.
- `f21bc65` stops adding the unpopulated column on fresh installs. It records
  `error_events(fingerprint, trace_id, span_id)` as the canonical relation and
  leaves legacy nullable native columns inert until a live-proven safe removal
  exists; it does not drop, backfill, or duplicate native raw writes.

## Step 1 evidence landed (preliminary, helper agent 2026-07-17)

Both prior blockers are lifted: plan 104 is DECIDED (Option C, unblock
directive 2026-07-17) and Docker is verified on the operator host. Live
probes on **stable v1.1.3** and **nightly v1.2.0-nightly-20260713** plus the
consumer/query inventory are recorded in
[docs/research/validation/2026-07-17-plan-125-fingerprint-probe.md](../docs/research/validation/2026-07-17-plan-125-fingerprint-probe.md)
(raw SQL transcripts alongside). Key results: fresh native tables are clean;
the legacy ADD reproduces; `DROP COLUMN "fingerprint"` succeeds and persists
across restarts on both engines with no data loss; duplicate ADD/DROP fail
closed (4003/4002 — convergence must guard on `information_schema`); and the
never-written legacy column reads **non-NULL** for existing rows on both
engines, so NULL-based consumers were never viable. No product reader of
`opentelemetry_traces.fingerprint` exists at `0b470a4`. Peer/executor owns
Steps 2–4: decision record + spec update, upgraded-real-legacy-dir check, and
the guarded existing-install convergence implementation.

## Current Blocker (rechecked 2026-07-14; superseded above)

- Plan 104 remains fail-closed: its decision record has no operator-selected
  canonical evidence model/version, approver, or approval date. Plan 125's
  dependency list explicitly includes 104, so it cannot publish a final
  fingerprint-to-bundle correlation contract independently.
- The required stable and nightly GreptimeDB fresh/upgrade, drop/index, and
  restart probes need a live engine. This host's Docker client cannot connect
  to a daemon, and no alternate engine or custom raw table is permitted. No
  safe migration can be inferred from SQL syntax or mocked in-memory behavior.

The already-landed fresh-install behavior remains valid: `error_events` is the
only current product correlation relation, no product reader relies on the
nullable native column, and legacy columns remain inert. Resume only after the
Plan 104 approval is committed and a live GreptimeDB-capable host is available
for the named stable/nightly experiments.

## Scope

In scope:

- Native/derived correlation contract and consumer/query inventory.
- Latest stable and nightly GreptimeDB live checks for column introspection,
  safe removal, update/backfill, indexes, and native extension behavior.
- A decision to remove the deviation or populate it through a measured,
  zero-copy-compatible native extension path.
- Fresh-install DDL, existing-install convergence, query fixtures, and docs.

Out of scope:

- A hand-rolled raw trace table or alternate telemetry engine.
- Cloning decoded telemetry or adding another raw write on the ingest hot path.
- Treating a nullable, never-populated column as an index/correlation feature.
- General trace schema/index redesign or unrelated native-table deviations.

## Steps

### Step 1: Characterize the live contract

Inventory every query, GraphQL/CLI/bundle consumer, and derived writer that uses
fingerprint, trace ID, or span ID. Seed a real trace with an error and prove the
current correlation route. Capture `SHOW CREATE TABLE` and column values for a
fresh and an upgraded data directory on latest stable and latest nightly.

Live-test idempotent column removal, index behavior, update/backfill semantics,
and supported native extension points. Record exact versions, commands, plans,
and failure behavior in a dated validation note. Do not infer support from SQL
syntax alone.

### Step 2: Choose one canonical relation

Prefer removing the native column when `error_events` already provides the
required fingerprint-to-trace/span relation within product bounds. Retain and
populate it only if an approved consumer cannot meet its contract through that
derived relation and a supported native extension can populate it without raw
signal duplication or hot-path clones.

Update the implementation spec and native-table decision evidence with the
choice, rejected alternative, migration behavior, query ownership, and engine
evidence before changing product code.

### Step 3: Implement fresh and existing-install behavior

For removal, stop adding the column and converge existing installs only through
a live-proven safe/idempotent migration; if GreptimeDB cannot safely drop it,
leave the legacy nullable column inert and explicitly unsupported while all new
queries ignore it. For retention, populate and index it through the approved
native mechanism, with bounded writes and no decode/clone duplication.

Never drop data, rebuild a native raw table, or hide migration failure. Startup
must remain restart-safe and report any required operator action.

### Step 4: Prove correlation and migration

Test fresh, legacy-null-column, partially migrated, and restarted states against
real GreptimeDB. Assert the chosen canonical query returns the same error,
fingerprint, trace, and span relationship without scanning an unbounded window.
Measure the retained path if it adds writes or indexes.

## Test Plan

- Structured consumer/query inventory with no unowned fingerprint reader.
- Stable/nightly native schema and migration probes.
- Fresh and upgraded real-engine fixtures with seeded trace-linked errors.
- Restart/idempotency and partial-failure migration tests.
- Query-plan/row parity plus hot-path allocation/clone evidence.

## Done Criteria

- [ ] One documented canonical fingerprint-to-trace relation exists.
- [ ] No product query relies on a nullable, unpopulated native column.
- [ ] Fresh and existing data directories converge without raw-table rebuild or loss.
- [ ] Native raw traces remain in GreptimeDB's native tables only.
- [ ] The implementation adds no decoded-telemetry clone or duplicate raw write.
- [ ] Stable/nightly real-engine fixtures and restart tests pass.

## STOP Conditions

- The proposed path requires a custom raw trace table, rustls, or another engine.
- Population requires cloning/re-decoding telemetry or an unbounded per-row update.
- Column removal risks data loss or native helper-table incompatibility.
- No approved consumer requirement justifies retaining/populating the column.

## Remove When

Delete this plan and index row after the native deviation and existing-install
behavior match one tested correlation contract and all real-engine gates pass.
