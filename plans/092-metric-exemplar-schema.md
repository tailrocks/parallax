# Plan 092: Correct the metric exemplar primary key

> **Executor instructions**: This schema correction is independent of blocked
> gRPC plan 089. Update the implementation spec/decision evidence before code,
> preserve existing rows, and test against the shipped GreptimeDB engine.

## Status

- **Priority**: P1
- **Effort**: M
- **Risk**: HIGH
- **Depends on**: none
- **Category**: storage / schema correctness
- **Planned at**: `eefa4617`, 2026-07-12
- **Status**: IN PROGRESS

## Why

`metric_exemplars` currently declares `trace_id` and `span_id` in its primary
key. Those values are effectively per-event unique and create high-cardinality
series tags with no query benefit. Reads filter by metric name, optional
service, time range, and recency. The correct low-cardinality primary key is
`("service", "name")`; trace/run identifiers should be fields with indexes only
where queries use them.

## Current Evidence

- DDL in `crates/parallax-storage/src/greptime.rs` uses
  `PRIMARY KEY ("service", "name", "trace_id", "span_id")`.
- `run_metric_points` already demonstrates `PRIMARY KEY ("service", "name")`
  with `run_id` as an indexed field.
- Exemplar reads do not group or seek by `trace_id`/`span_id` as a series key.
- `CREATE TABLE IF NOT EXISTS` does not repair existing data directories.

## Scope

In scope:

- The extension-table contract in the implementation spec and native-table
  decision evidence.
- Fresh `metric_exemplars` DDL.
- An idempotent existing-data migration.
- Read/write table indirection if GreptimeDB cannot rename the migrated table.
- Golden SQL, schema inspection, and real-engine migration tests.

Out of scope:

- Raw metric native-table schema.
- gRPC writer transport.
- Other extension-table primary keys.
- Dropping existing rows or requiring a clean data directory.

## Steps

### Step 1: Pin the contract and live engine behavior

Update `docs/research/architecture/v1-implementation-spec.md` and the derived
extension-table section of `docs/research/decisions/native-otel-tables.md`.
Live-test latest stable GreptimeDB for:

- primary-key introspection;
- table rename support;
- `INSERT INTO ... SELECT ...` JSON/timestamp preservation;
- transactional/failure behavior of the migration sequence.

Record the chosen migration in a dated validation note.

### Step 2: Correct fresh DDL

Use:

```sql
CREATE TABLE IF NOT EXISTS metric_exemplars (
  "ts" TIMESTAMP(9) NOT NULL,
  "service" STRING,
  "name" STRING,
  "value" DOUBLE,
  "trace_id" STRING SKIPPING INDEX,
  "span_id" STRING,
  "run_id" STRING SKIPPING INDEX,
  "attributes" JSON,
  TIME INDEX ("ts"),
  PRIMARY KEY ("service", "name")
)
```

Keep append mode and configured metrics TTL.

### Step 3: Migrate old shape idempotently

- Detect only the known old primary-key shape.
- Create a versioned replacement table with the corrected schema.
- Copy every column and verify source/destination counts before cutover.
- Rename when supported. Otherwise use one tested table-name constant and keep
  the old table until the replacement is verified.
- Make restart after every intermediate step safe and convergent.
- Never drop the old table before count/value verification succeeds.

### Step 4: Prove query and ingest compatibility

Run fresh and migrated data through exemplar ingest and newest-first queries.
Verify service/name/time filters, trace links, run IDs, JSON attributes, TTL,
and schema inventory.

## Test Plan

- Golden fresh DDL assertion.
- Pure migration-state decision tests.
- Real-engine old-shape fixture with representative null/JSON/ns values.
- Restart/failure tests at create, copy, verify, cutover, and cleanup stages.
- Query parity before/after migration.

## Done Criteria

- [ ] Fresh DDL primary key is exactly service + name.
- [ ] Trace/run identifiers have only query-justified indexes.
- [ ] Existing old-shape rows migrate without loss or mutation.
- [ ] Migration is idempotent after interruption.
- [ ] Real-engine schema and query assertions pass.
- [ ] Storage conformance, nextest, strict Clippy, and fmt pass.
- [ ] Contract docs and `PROJECT_STRUCTURE.md` remain accurate.

## STOP Conditions

- The live engine cannot preserve rows through any bounded migration path.
- Rename is unavailable and table indirection would require more than a small,
  centralized change.
- Migration changes timestamp precision or JSON values.
- A proposed fix alters Greptime native raw metric tables.

## Remove When

Delete this plan and its index row after fresh and existing-data real-engine
tests are green and the migration evidence is stored under
`docs/research/validation/`.
