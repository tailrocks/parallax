# Metric exemplar schema migration validation

- **Date:** 2026-07-12
- **Engine:** GreptimeDB 1.1.2 (`SELECT version()`)
- **Plan:** 092

## Result

GreptimeDB 1.1.2 supports the bounded migration required to replace the legacy
high-cardinality `metric_exemplars` primary key without mutating rows.

| Capability | Live result |
| --- | --- |
| Primary-key inspection | `DESCRIBE table` marked time index and tag columns as `PRI`; tag rows identified the ordered legacy key. |
| Copy | `INSERT INTO destination SELECT * FROM source` copied one representative row. |
| Timestamp fidelity | `CAST(ts AS BIGINT)` remained exactly `1741437296123456789`. |
| JSON fidelity | Nested boolean and numeric values remained equal; key order normalized on rendering. |
| Rename | `ALTER TABLE old RENAME new` succeeded and `SHOW TABLES` exposed only the new name. |
| Atomic transaction | No documented multi-DDL transaction guarantee was found; the implementation therefore does not depend on one. |

The shipped sequence retains the old canonical table until a corrected
replacement has matching counts and no bidirectional `EXCEPT` differences. It
then renames the old table to `metric_exemplars_v1_legacy`, renames the verified
replacement to the canonical name, verifies again, and only then drops the
legacy table. On restart, an old canonical table is recopied from scratch; a
legacy-plus-replacement intermediate state converges by rebuilding and
finishing the canonical rename; a corrected canonical plus legacy state
re-verifies before cleanup.

## Environment note

The repository-managed baseline harness downloaded and checksum-verified the
166 MiB GreptimeDB 1.1.2 Linux ARM64 release, but initially refused to start
because ports 24000–24003 were owned by an earlier Parallax verification
engine. The capability spike used that existing isolated GreptimeDB 1.1.2 HTTP
endpoint with uniquely named `parallax_plan092_probe_*` tables and dropped only
those probe tables afterward. After retiring the stale verification child, the
dedicated `m7_metric_exemplar_migration_greptime` managed-engine fixture passed:
it migrated two legacy rows covering nanosecond timestamps, nested JSON, and
nullable `run_id`; verified the exact two-column key; reran bootstrap to prove
idempotency; and confirmed that no replacement or legacy table remained. The
expanded fixture then reproduced interruption states after replacement create,
partial copy/failed verification, the first rename, the second rename, and
before cleanup. Every restart converged without row loss. Finally, it sent a
new derived exemplar through the production ingest method and queried it through
the production newest-first, metric/service/time-bounded read surface, proving
trace/span links, run ID, and JSON attributes after migration.

`SHOW CREATE TABLE metric_exemplars` on the migrated table mechanically
confirmed `PRIMARY KEY (service, name)`, Bloom skipping indexes on exactly the
query-justified `trace_id` and `run_id` fields, `append_mode = 'true'`, and the
configured `7d` TTL normalized by GreptimeDB to `7days`. The native table
inventory fixture also passed with only the three approved Parallax extension
tables and no migration artifacts.
