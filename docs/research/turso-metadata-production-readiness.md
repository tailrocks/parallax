# Turso Metadata Production Readiness

<!-- markdownlint-disable MD013 -->

Research date: 2026-05-25

## Purpose

This note tightens proof gate #11 from
[Strategic verdict and research coverage](strategic-verdict-and-research-coverage.md):

> Turso correctness, backup/restore, concurrency, migration, and fallback
> behavior for metadata, agent session state, CLI invocation state, outcomes,
> and audit records.

The decision: **Turso remains the first metadata-store implementation for the
Rust-first tiny profile, but Parallax must not treat Turso Database as a
production-safe default until a production-readiness gate passes.**

This is a narrow gate, not a reversal of the operator preference. The default
prototype path is still Turso. The production claim is withheld until the local
engine, optional sync path, backup/restore path, and Postgres fallback path are
proven on Parallax metadata workloads.

## Current Primary-Source Checks

| Source | What matters for Parallax |
| --- | --- |
| [Turso Database GitHub repository](https://github.com/tursodatabase/turso) | Latest non-prerelease checked by GitHub API is `v0.6.1` published 2026-05-22. The README still marks Turso Database beta, says to use caution with production data and backups, and says Turso Database is not ready for production use while libSQL is production ready. |
| [Turso Rust SDK reference](https://docs.turso.tech/sdk/rust/reference) | New Rust projects should use the `turso` crate for local/embedded database and sync. The newer engine supports MVCC concurrent writes and push/pull sync; `libsql` remains the remote/existing-codebase option. |
| [Turso concurrent writes](https://docs.turso.tech/tursodb/concurrent-writes) | Default configuration allows one writer. MVCC requires `PRAGMA journal_mode = 'mvcc'` and `BEGIN CONCURRENT`; conflicting same-row transactions must roll back and retry. Parallax must own retry policy and hot-row contention tests. |
| [Turso CDC](https://docs.turso.tech/tursodb/cdc) | CDC records data changes, but it cannot be used together with MVCC on the same connection. Parallax cannot rely on CDC as the primary audit trail if it also depends on MVCC concurrent writes. |
| [Turso Sync usage](https://docs.turso.tech/sync/usage) | Sync writes locally and uses explicit `push()` / `pull()`. First bootstrap needs the remote unless disabled. Stats expose WAL sizes and sync metadata. Sync is an optional topology, not the base local metadata contract. |
| [Turso Sync conflict resolution](https://docs.turso.tech/sync/conflict-resolution) | Sync uses last-push-wins for conflicting changes. During pull with unpushed local changes, Turso rolls back to the last synced state, applies remote changes, and replays local changes. This is unacceptable for Parallax audit/outcome rows unless rows are append-only/idempotent and conflicts are structurally impossible. |
| [Turso Sync checkpoint](https://docs.turso.tech/sync/checkpoint) | Auto-checkpoint is disabled for sync databases; applications must call `checkpoint()` to keep WAL growth bounded. Parallax needs a checkpoint policy if it ever enables sync. |
| [Turso Cloud durability](https://docs.turso.tech/cloud/durability) | Turso Cloud users registered/upgraded after 2025-03-17 get stated 99.999999999% durability with added commit latency up to 100/50/25/10 ms by plan, backed by S3 Express One Zone and S3. These are managed-cloud guarantees, not proof that embedded local files are durable under crash or power loss. |
| [Turso point-in-time recovery](https://docs.turso.tech/features/point-in-time-recovery) | Cloud PITR restores by creating a new database and may have up to a 15-second gap before the requested timestamp. Parallax cannot use PITR alone as an exact local backup or audit recovery guarantee. |
| [Turso database export](https://docs.turso.tech/cli/db/export) | Export creates a SQLite snapshot, but the docs warn it may not contain the latest changes; SDK sync is needed after export for the most recent version. Export is useful for portability, not sufficient by itself as a correctness proof. |
| [Turso Cloud limitations](https://docs.turso.tech/cloud/limitations) | Some SQLite pragmas differ in Cloud: `user_version` is read-only, `journal_mode` is unsupported, and migration tracking should use an explicit `_schema_version` table. Parallax schema management must avoid SQLite-only assumptions. |
| [PostgreSQL MVCC](https://www.postgresql.org/docs/current/mvcc.html), [pg_dump](https://www.postgresql.org/docs/current/app-pgdump.html), [pg_restore](https://www.postgresql.org/docs/current/app-pgrestore.html) | Postgres remains the fallback baseline for mature concurrent metadata writes and boring backup/restore operations. |

## Architecture Decision

Split the metadata decision into three modes:

| Mode | Role | Parallax stance |
| --- | --- | --- |
| Local Turso Database | First prototype and tiny-profile metadata engine. | Preferred implementation path, but production claim gated on crash, backup, concurrency, migration, and fallback tests. |
| Turso Sync / Turso Cloud | Optional sync or managed-cloud metadata topology. | Not part of the self-hosted tiny contract. Use only after conflict, checkpoint, PITR, token rotation, and restore workflows are tested. |
| Postgres | Scale-out and maturity fallback. | Must remain a live adapter, not a theoretical rewrite, because Turso is still beta. |

Do not cite Turso Cloud durability or PITR as proof that local embedded Turso is
safe. They are different operating modes with different failure models.

## Required Product Constraints

Parallax metadata is not disposable cache. It includes issue state, redaction
policy versions, agent-session timelines, CLI invocation records, raw-access
audits, and accepted/rejected fix outcomes. Losing or rewriting it can make an
investigation misleading.

Therefore:

- metadata writes must be idempotent and transactionally grouped around durable
  refs;
- audit rows must be append-only and should not depend on Turso CDC when MVCC is
  enabled;
- schema migrations must use an explicit `_schema_version` table, not
  SQLite-only pragmas;
- hot fingerprint upserts must include bounded retry/backoff behavior for Turso
  MVCC conflicts;
- sync mode must avoid last-push-wins conflicts by construction, preferably
  with append-only globally unique row IDs;
- local backup must be a Parallax-owned operation, not only a raw file copy
  assumption;
- restore must produce a checksum/invariant report before Parallax serves agent
  context from the restored metadata store.

## Production-Readiness Gate

Turso can stay the metadata default only if all Tier A gates pass and Tier B has
a documented operator story.

| Gate | Required evidence |
| --- | --- |
| A1 crash correctness | Kill the process during multi-row issue/event/bundle/agent/audit transactions; reopen and verify no orphan rows, no counter drift, no missing committed audit rows, and no partial bundle metadata. |
| A2 hot write contention | Run concurrent issue upserts and agent-step appends with MVCC enabled; measure conflict rate, retry count, p95/p99 latency, and no duplicate sequence numbers. |
| A3 backup/restore | Produce a local backup while writes continue or are explicitly paused; restore into a fresh store; verify row counts, checksums, foreign keys, and bundle/ref reachability. |
| A4 migration rollback | Fail a schema migration halfway and prove the previous binary can either continue safely or abort with a clear recovery command. |
| A5 Postgres fallback | Export logical rows and import into Postgres with stable IDs; run the same `MetadataStore` query suite and preserve evidence-bundle references. |
| A6 Cloud/sync isolation | If sync is enabled, test last-push-wins conflicts, rollback/replay behavior, checkpoint scheduling, token rotation, PITR restore to new DB, and the documented possible PITR gap. |
| A7 operational observability | Expose metadata-store health, checkpoint lag/WAL size where relevant, backup age, restore test age, migration version, retry rates, and fallback readiness. |

Tier B operator story:

- how often backups run;
- where backups are stored;
- how restore is rehearsed;
- how to migrate to Postgres;
- how long the product can run with metadata writes paused;
- what agent/API features are disabled when metadata is degraded.

## Fallback Triggers

Postgres becomes the metadata default for production installs if any of these
are true:

- Turso crash tests produce invariant violations;
- local backup/restore cannot prove exact logical recovery;
- MVCC conflict retries create unacceptable latency or lock storms for hot issue
  upserts;
- schema migration rollback cannot be made deterministic;
- sync last-push-wins semantics can overwrite audit/outcome rows in a realistic
  multi-writer topology;
- Turso-to-Postgres export cannot preserve stable IDs and bundle refs;
- Turso beta status remains and the product is otherwise ready for production
  users who need stronger support guarantees.

Turso can still remain the local-dev and single-user default even if Postgres
becomes the recommended production metadata store.

## Prototype Updates

Extend `parallax-metadata-bench` from
[Metadata store benchmark plan and prototype](metadata-store-benchmark-plan.md)
with these explicit subcommands:

```text
parallax-metadata-bench
  turso-crash        kill/reopen invariant tests
  turso-mvcc         BEGIN CONCURRENT contention and retry metrics
  turso-backup       backup/restore checksum and ref-reachability report
  turso-migrate      schema upgrade/rollback rehearsal
  turso-export-pg    logical export/import into Postgres
  turso-sync         optional sync conflict/checkpoint/PITR workflow
```

The report should include:

- Turso version and crate version;
- whether MVCC was enabled;
- whether sync was enabled;
- host filesystem and crash method;
- transaction counts and retry counts;
- p50/p95/p99 latency by operation;
- backup size and restore duration;
- invariant failures;
- exact fallback/import errors.

## Relationship To Other Research

- [Metadata store benchmark plan and prototype](metadata-store-benchmark-plan.md)
  remains the runnable benchmark spec. This note adds the production-readiness
  gate and current Turso source interpretation.
- [Technical implementation concept](technical-implementation-concept.md)
  should continue to name Turso as the first implementation while keeping
  Postgres as an active fallback.
- [Risks and bear case](risks-and-bear-case.md) should treat this gate as part
  of A5: the chosen stack holds.
- [A5 stack decision ledger](a5-stack-decision-ledger.md) consumes this gate's
  metadata rows and decides whether Turso is only a prototype metadata default
  or whether Postgres must be the production fallback/default.
- [Agent and CLI execution tracing](agent-and-cli-execution-tracing.md) depends
  on this metadata store for auditability and outcome state.

## Bottom Line

Turso is still the right first implementation for the Rust-first tiny profile,
but the evidence now has to be phrased carefully:

> Turso is the preferred prototype metadata engine. Production readiness is
> unproven until Parallax passes crash, MVCC contention, backup/restore,
> migration rollback, sync-conflict, and Postgres fallback gates on its own
> metadata workload.

That keeps the operator preference intact without turning a beta engine into an
unstated production dependency.
