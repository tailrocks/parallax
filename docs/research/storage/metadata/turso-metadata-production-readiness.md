# Turso Metadata Production Readiness

<!-- markdownlint-disable MD013 -->

Research date: 2026-05-25 · **Version/status recheck 2026-07-17 (pass 68)**

> **Current authority (operator, 2026-06-12): Turso is mandatory metadata
> storage in every product profile.** A failed gate means fix forward in
> Parallax or upstream, never switch to Postgres, libSQL, SQLite, or another
> engine. Postgres rows below are preserved as dated comparator evidence only.
> Product contract cleanup is owned by
> [Plan 093 validation](../../validation/2026-07-12-plan-093-baseline/README.md),
> and supported server operations are owned by
> [`plans/115-v2-server-profile.md`](../../../../plans/115-v2-server-profile.md).
> This file is a research and validation protocol, not an implementation queue.
>
> **Pass 68 primary recheck (GitHub + README):**
> - Latest stable Turso Database: **`v0.7.0`** (2026-07-13, non-prerelease) —
>   **supersedes May pin `v0.6.1`**.
> - README FAQ **"Is Turso Database ready for production use?" → Yes** (cites
>   Turso Cloud, Kin, Spice.ai; DST + Antithesis testing). May note's "still
>   beta / not ready for production" **is stale**.
> - libSQL remains the longer-battle-tested lineage; Turso Database is the
>   active rewrite direction (both production-used per README).
> - **Parallax gate still not auto-passed:** vendor "runs in production" ≠
>   Parallax metadata workload crash/MVCC/backup/migration ledger green.
>   Withhold *Parallax* production-readiness claim until Tier A gates on
>   *our* workload pass.

## Purpose

This note tightens proof gate #11 from
[Strategic verdict and research coverage](../../decisions/strategic-coverage.md):

> Turso correctness, backup/restore, concurrency, migration, and degraded-mode
> behavior for metadata, agent session state, CLI invocation state, outcomes,
> and audit records.

The decision is no longer conditional: **Turso is the metadata store.** The
gate controls production-readiness claims and exposes work to fix, not which
engine Parallax selects.

The production claim remains withheld until the local engine, optional sync
path, and backup/restore path are proven on Parallax metadata workloads.

## Current Primary-Source Checks

| Source | What matters for Parallax |
| --- | --- |
| [Turso Database GitHub](https://github.com/tursodatabase/turso), [v0.7.0](https://github.com/tursodatabase/turso/releases/tag/v0.7.0) | **Pass 68 (2026-07-17):** latest non-prerelease **`v0.7.0`** (2026-07-13). README production FAQ = **Yes**. Historical May pin was `v0.6.1` + beta language — **stale**. | Pin stable vs pre vs `main` separately. Vendor production claim does not replace Parallax Tier A workload gates. |
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
| [PostgreSQL MVCC](https://www.postgresql.org/docs/current/mvcc.html), [pg_dump](https://www.postgresql.org/docs/current/app-pgdump.html), [pg_restore](https://www.postgresql.org/docs/current/app-pgrestore.html) | Postgres is a mature comparison baseline for concurrent metadata writes and backup/restore. It is not a Parallax product fallback. |

## Architecture Decision

Distinguish the supported Turso modes from the comparator:

| Mode | Role | Parallax stance |
| --- | --- | --- |
| Local Turso Database | Local/tiny metadata engine. | Mandatory product path; production claims remain gated on crash, backup, concurrency, and migration tests. |
| Turso Sync / Turso Cloud | Optional sync or managed-cloud metadata topology. | Not part of the self-hosted tiny contract. Use only after conflict, checkpoint, PITR, token rotation, and restore workflows are tested. |
| Postgres | Research comparator. | May calibrate concurrency and recovery expectations; never ships as an adapter or fallback. |

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

Turso production-readiness claims require all Tier A gates and a documented
Tier B operator story. Failure creates Turso/Parallax remediation work.

| Gate | Required evidence |
| --- | --- |
| A1 crash correctness | Kill the process during multi-row issue/event/bundle/agent/audit transactions; reopen and verify no orphan rows, no counter drift, no missing committed audit rows, and no partial bundle metadata. |
| A2 hot write contention | Run concurrent issue upserts and agent-step appends with MVCC enabled; measure conflict rate, retry count, p95/p99 latency, and no duplicate sequence numbers. |
| A3 backup/restore | Produce a local backup while writes continue or are explicitly paused; restore into a fresh store; verify row counts, checksums, foreign keys, and bundle/ref reachability. |
| A4 migration rollback | Fail a schema migration halfway and prove the previous binary can either continue safely or abort with a clear recovery command. |
| A5 comparator portability | Optionally export logical rows to a disposable Postgres comparator to validate schema assumptions and stable IDs; this does not authorize a product adapter. |
| A6 Cloud/sync isolation | If sync is enabled, test last-push-wins conflicts, rollback/replay behavior, checkpoint scheduling, token rotation, PITR restore to new DB, and the documented possible PITR gap. |
| A7 operational observability | Expose metadata-store health, checkpoint lag/WAL size where relevant, backup age, restore test age, migration version, retry rates, and degraded-mode readiness. |

Tier B operator story:

- how often backups run;
- where backups are stored;
- how restore is rehearsed;
- how logical export is verified independently without creating a product
  migration promise;
- how long the product can run with metadata writes paused;
- what agent/API features are disabled when metadata is degraded.

## Failure Response (Supersedes Historical Fallback Triggers)

The original study treated the following as Postgres switch triggers. Current
policy treats each as a release blocker or fix-forward trigger:

- Turso crash tests produce invariant violations;
- local backup/restore cannot prove exact logical recovery;
- MVCC conflict retries create unacceptable latency or lock storms for hot issue
  upserts;
- schema migration rollback cannot be made deterministic;
- sync last-push-wins semantics can overwrite audit/outcome rows in a realistic
  multi-writer topology;
- logical export cannot preserve stable IDs and bundle refs;
- Turso beta status remains and the product is otherwise ready for production
  users who need stronger support guarantees;
- only a moving `main` commit, unreleased pre-release, or vendor benchmark result
  passes a gate that fails or has not been rerun on the latest stable release.

None of these conditions authorizes an alternate product engine. Plan 115 must
block a server-profile claim until the applicable condition is fixed and proven.

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
  turso-export-pg    comparator-only logical export/import validation
  turso-sync         optional sync conflict/checkpoint/PITR workflow
```

The report should include:

- Turso version and crate version;
- release track (`stable`, `pre-release`, or `main`) and exact Git tag or commit;
- whether MVCC was enabled;
- whether sync was enabled;
- host filesystem and crash method;
- transaction counts and retry counts;
- p50/p95/p99 latency by operation;
- backup size and restore duration;
- invariant failures;
- exact comparator export/import errors.

## Relationship To Other Research

- [Metadata store benchmark plan and prototype](metadata-store-benchmark-plan.md)
  remains the runnable benchmark spec. This note adds the production-readiness
  gate and current Turso source interpretation.
- [Technical implementation concept](../../architecture/implementation-concept.md)
  is dated design history; current product authority is mandatory Turso.
- [Risks and bear case](../../decisions/risks-and-bear-case.md) should treat this gate as part
  of A5: the chosen stack holds.
- [A5 stack decision ledger](../../decisions/stack-decision.md) consumes this
  gate's metadata rows to qualify claims and expose remediation work; it cannot
  select Postgres.
- [Agent and CLI execution tracing](../../capture/agent-cli-tracing.md) depends
  on this metadata store for auditability and outcome state.

## Bottom Line

Turso is the mandatory metadata implementation, and the evidence must be
phrased carefully:

> Turso production readiness is unproven until Parallax passes crash, MVCC
> contention, backup/restore, migration rollback, and sync-conflict gates on its
> own metadata workload. Failures block the claim and require a fix; Postgres
> comparison results cannot substitute for Turso evidence.

That keeps the mandatory dependency explicit without overstating the maturity
of the measured Turso release.
