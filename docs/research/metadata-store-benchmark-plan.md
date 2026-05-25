# Metadata Store Benchmark Plan And Prototype

<!-- markdownlint-disable MD013 -->

Research date: 2026-05-25

## Purpose

Parallax needs a separate benchmark and runnable prototype for low-volume product
metadata. This is not the GreptimeDB/ClickHouse observability-storage benchmark.
The metadata store holds control-plane and audit state:

- users, projects, DSNs, tokens, and teams;
- issue status, grouping state, assignments, and comments;
- redaction policy and raw-access policy;
- context bundle records;
- agent sessions, agent steps, tool calls, and patch refs;
- CLI invocation records;
- fix outcomes, human reviews, recurrence state, and audit logs.

The current operator decision is:

> Prefer Turso Database for metadata. Do not default to C SQLite. Keep Postgres
> only as a scale-out fallback if Turso fails the technical gates.

## Current Source Snapshot

As of 2026-05-25:

- The `tursodatabase/turso` repository describes Turso Database as an
  in-process SQL database written in Rust and compatible with SQLite.
- The public README marks Turso Database beta and warns to use caution with
  production data and backups.
- Public docs distinguish legacy libSQL embedded replicas from newer Turso
  Database/Turso Sync direction.
- The Rust quickstart says the Rust SDK is built on the Turso Database engine, a
  ground-up SQLite rewrite with MVCC/concurrent writes, async I/O, and native
  Rust async/await support.

Sources:

- [Turso Database GitHub repository](https://github.com/tursodatabase/turso)
- [Turso Rust quickstart](https://docs.turso.tech/sdk/rust/quickstart)
- [Turso embedded replicas](https://docs.turso.tech/features/embedded-replicas/introduction)
- [Turso libSQL overview](https://docs.turso.tech/libsql)
- [Turso database export](https://docs.turso.tech/cli/db/export)
- [Turso point-in-time recovery](https://docs.turso.tech/features/point-in-time-recovery)
- [PostgreSQL pgbench](https://www.postgresql.org/docs/current/pgbench.html)
- [PostgreSQL concurrency control](https://www.postgresql.org/docs/current/mvcc.html)
- [PostgreSQL pg_dump](https://www.postgresql.org/docs/current/app-pgdump.html)
- [PostgreSQL pg_restore](https://www.postgresql.org/docs/current/app-pgrestore.html)

## Benchmark Question

Can Turso safely serve as the tiny/local Parallax metadata store while preserving
a clean fallback path to Postgres for larger installs?

This benchmark should not ask whether Turso can store high-volume telemetry. It
should ask whether Turso is good enough for product state and audit records while
keeping Parallax simpler and more Rust-native than a mandatory Postgres
deployment.

## Candidate Roles

| Store | Role | Why |
| --- | --- | --- |
| Turso Database | Preferred tiny/local metadata engine. | Rust-first, SQLite-compatible, in-process, simple operational shape, aligned with the project's Rust/agent-contributable lens. |
| Postgres | Scale-out fallback. | Mature concurrent relational database, strong ecosystem, boring production behavior, but adds an external service and non-Rust operational surface. |
| C SQLite | Compatibility baseline only. | Proven embedded reliability, but not the operator's preferred default and not Rust-native. |

## Workload Model

Metadata is low volume but correctness-sensitive.

Tables to model:

```text
projects
project_tokens
issues
issue_events
redaction_policies
context_bundles
agent_sessions
agent_steps
agent_tool_calls
agent_patches
cli_invocations
fix_outcomes
audit_log
```

Primary write paths:

1. ingest creates or updates issue rows;
2. evidence bundle creation records source refs and redaction report;
3. agent session records steps/tool calls/patch/outcome;
4. CLI invocation records command metadata and exit state;
5. human review updates issue and outcome state;
6. audit log appends policy/access/action events.

## Benchmark Dimensions

### Correctness And Durability

- crash during transaction;
- crash during WAL/checkpoint;
- crash while appending audit records;
- power-loss simulation if available;
- backup and restore integrity;
- migration rollback behavior;
- corruption detection and recovery path.

### Concurrency

- many issue updates against the same fingerprint;
- concurrent agent sessions writing steps and tool calls;
- concurrent CLI invocations writing stdout/stderr excerpt refs;
- read-heavy dashboard/API access while ingest writes continue;
- lock behavior for long-running reads.

### Latency

- p50/p95/p99 insert latency for audit records;
- issue lookup/update latency by fingerprint and project;
- context bundle metadata lookup latency;
- agent session timeline query latency;
- CLI invocation lookup by repo/commit/command/exit.

### Operational Shape

- single-binary or in-process local setup;
- file backup and restore workflow;
- schema migration tooling;
- observability of the metadata store itself;
- data export portability;
- upgrade path between versions;
- path from Turso to Postgres if needed.

### Agent And Audit Fit

- append-only audit-log ergonomics;
- foreign-key and transaction behavior for agent session graphs;
- ability to snapshot evidence metadata before and after an agent action;
- query ergonomics for "what happened before this incident?";
- redaction policy versioning and raw-access audit.

## Test Matrix

| Tier | Purpose | Data shape |
| --- | --- | --- |
| Local smoke | Validate schema, migrations, and simple queries. | 10 projects, 1k issues, 100 agent sessions, 500 CLI runs. |
| Startup realistic | Early deployment. | 100 projects, 100k issues/events, 10k agent sessions, 50k CLI runs. |
| Stress | Prove fallback point. | Millions of issue events, agent steps, CLI invocations, and audit records. |

## Runnable Prototype Spec

Build one Rust binary, `parallax-metadata-bench`, separate from the observability
storage harness:

```text
parallax-metadata-bench
  ├── schema      run migrations against candidate
  ├── gen         deterministic product/audit workload generator
  ├── load        concurrent writer/reader workload
  ├── crash       crash/restart and backup/restore probes
  ├── migrate     Turso -> Postgres export/import check
  └── report      results.json + markdown summary
MetadataStore (trait)
  ├── TursoStore      local file, optional sync mode when testing cloud sync
  └── PostgresStore   sqlx or tokio-postgres against local container
```

The trait keeps Parallax's metadata API independent of the backend:

```rust
#[async_trait]
pub trait MetadataStore: Send + Sync {
    fn name(&self) -> &'static str;
    async fn migrate(&self) -> Result<()>;
    async fn create_project(&self, project: ProjectSeed) -> Result<ProjectId>;
    async fn upsert_issue_event(&self, event: IssueEventSeed) -> Result<IssueId>;
    async fn record_bundle(&self, bundle: BundleSeed) -> Result<BundleId>;
    async fn append_agent_step(&self, step: AgentStepSeed) -> Result<()>;
    async fn append_cli_invocation(&self, inv: CliInvocationSeed) -> Result<()>;
    async fn append_audit(&self, audit: AuditSeed) -> Result<()>;
    async fn query_issue_context_meta(&self, issue: IssueId) -> Result<IssueContextMeta>;
    async fn query_agent_timeline(&self, session: AgentSessionId) -> Result<Vec<AgentStep>>;
    async fn query_resource_audit(&self, resource: ResourceRef) -> Result<Vec<AuditRecord>>;
    async fn backup(&self, path: &Path) -> Result<BackupReport>;
    async fn restore(&self, path: &Path) -> Result<RestoreReport>;
}
```

The benchmark must run every query through this trait. If Turso requires SQL that
Postgres cannot execute, or the reverse, the adapter owns the dialect difference.
Parallax product code should not care.

## Schema Prototype

Use the same logical schema for Turso and Postgres. Dialect differences should
be limited to primary-key generation, JSON type, and timestamp defaults.

```sql
CREATE TABLE projects (
  id TEXT PRIMARY KEY,
  slug TEXT NOT NULL UNIQUE,
  created_at TEXT NOT NULL
);

CREATE TABLE issues (
  id TEXT PRIMARY KEY,
  project_id TEXT NOT NULL REFERENCES projects(id),
  fingerprint TEXT NOT NULL,
  status TEXT NOT NULL,
  first_seen TEXT NOT NULL,
  last_seen TEXT NOT NULL,
  event_count INTEGER NOT NULL DEFAULT 0,
  grouping_algo TEXT NOT NULL,
  UNIQUE(project_id, fingerprint)
);

CREATE TABLE issue_events (
  id TEXT PRIMARY KEY,
  issue_id TEXT NOT NULL REFERENCES issues(id),
  event_id TEXT NOT NULL,
  trace_id TEXT,
  release TEXT,
  received_at TEXT NOT NULL,
  raw_ref TEXT NOT NULL,
  UNIQUE(event_id)
);

CREATE TABLE context_bundles (
  id TEXT PRIMARY KEY,
  project_id TEXT NOT NULL REFERENCES projects(id),
  anchor_type TEXT NOT NULL,
  anchor_id TEXT NOT NULL,
  schema_version TEXT NOT NULL,
  generated_at TEXT NOT NULL,
  redaction_report_ref TEXT NOT NULL,
  bundle_ref TEXT NOT NULL
);

CREATE TABLE agent_sessions (
  id TEXT PRIMARY KEY,
  project_id TEXT NOT NULL REFERENCES projects(id),
  agent_product TEXT NOT NULL,
  status TEXT NOT NULL,
  started_at TEXT NOT NULL,
  ended_at TEXT,
  outcome TEXT
);

CREATE TABLE agent_steps (
  id TEXT PRIMARY KEY,
  session_id TEXT NOT NULL REFERENCES agent_sessions(id),
  seq INTEGER NOT NULL,
  kind TEXT NOT NULL,
  target TEXT,
  result_ref TEXT,
  created_at TEXT NOT NULL,
  UNIQUE(session_id, seq)
);

CREATE TABLE cli_invocations (
  id TEXT PRIMARY KEY,
  project_id TEXT NOT NULL REFERENCES projects(id),
  repo TEXT,
  commit_sha TEXT,
  command TEXT NOT NULL,
  exit_code INTEGER,
  started_at TEXT NOT NULL,
  ended_at TEXT,
  stdout_ref TEXT,
  stderr_ref TEXT
);

CREATE TABLE audit_log (
  id TEXT PRIMARY KEY,
  project_id TEXT NOT NULL REFERENCES projects(id),
  actor_type TEXT NOT NULL,
  actor_id TEXT NOT NULL,
  action TEXT NOT NULL,
  resource_type TEXT NOT NULL,
  resource_id TEXT NOT NULL,
  policy_version TEXT,
  created_at TEXT NOT NULL
);

CREATE INDEX idx_issues_project_fingerprint ON issues(project_id, fingerprint);
CREATE INDEX idx_issue_events_issue_received ON issue_events(issue_id, received_at);
CREATE INDEX idx_context_bundles_anchor ON context_bundles(project_id, anchor_type, anchor_id);
CREATE INDEX idx_agent_steps_session_seq ON agent_steps(session_id, seq);
CREATE INDEX idx_cli_repo_commit_command ON cli_invocations(repo, commit_sha, command);
CREATE INDEX idx_audit_resource_time ON audit_log(resource_type, resource_id, created_at);
```

For Postgres, test online index/migration behavior separately because PostgreSQL
supports `CREATE INDEX CONCURRENTLY`; SQLite-compatible stores do not have the
same operational model.

## Workload Operations

The generator creates deterministic project state with linked issue, bundle,
agent, CLI, and audit records:

| Operation | Mix | What it stresses |
| --- | --- | --- |
| `IssueEventUpsert` | 35% | Hot fingerprint updates, unique event idempotency, issue counters. |
| `BundleCreate` | 10% | Bundle manifest writes and redaction report refs. |
| `AgentStepAppend` | 20% | Ordered append-heavy agent session timelines. |
| `CliInvocationRecord` | 15% | Command metadata writes by repo/commit/command. |
| `AuditAppend` | 15% | Append-only audit log with resource lookup. |
| `HumanReviewUpdate` | 5% | Status/outcome updates racing with reads. |

Run phases:

1. **Load-only:** ingest generated metadata until the tier target is reached.
2. **Mixed read/write:** 70% writes, 30% reads for 30 minutes.
3. **Hot issue contention:** many writers upsert events for the same fingerprint.
4. **Agent timeline burst:** one session receives thousands of ordered steps.
5. **Long read while writing:** stream an audit query while appends continue.
6. **Migration:** add one nullable column, backfill a derived index column, and
   create one new index while workload continues where the candidate supports it.

## Exact Query Set

Measure p50/p95/p99 latency and error rate for each query:

| Query | SQL shape |
| --- | --- |
| `Q1 issue_lookup` | `SELECT * FROM issues WHERE project_id = ? AND fingerprint = ?` |
| `Q2 issue_context_meta` | issue + latest N issue events + latest bundle refs for one issue. |
| `Q3 agent_timeline` | `SELECT * FROM agent_steps WHERE session_id = ? ORDER BY seq` |
| `Q4 cli_by_commit` | CLI invocations by repo, commit SHA, command, and non-zero exit. |
| `Q5 resource_audit` | audit records by resource type/id ordered by time. |
| `Q6 outcome_feedback` | accepted/rejected/reverted fix outcomes by issue and release window. |
| `Q7 policy_version_check` | latest redaction/raw-access policy used for bundle and agent read. |

`Q2`, `Q5`, and `Q7` are the critical Parallax queries because they sit in the
agent context path. Turso fails as default if these are unstable under concurrent
writes even when simple point lookups are fast.

## Crash, Backup, Restore, And Export Tests

Metadata is low volume, but losing it is worse than losing a sampled span. The
prototype must run destructive checks in disposable temp directories/containers:

| Test | Turso check | Postgres check |
| --- | --- | --- |
| Process kill during transaction | Kill benchmark process during multi-row issue/event/bundle transaction; reopen and verify invariants. | Kill client and Postgres container separately; verify committed/uncommitted boundaries. |
| Disk-full simulation | Fill filesystem or mount quota during audit append and backup. | Same with data volume and WAL volume. |
| Backup/restore | Use local file copy where safe plus Turso export/PITR path when using Turso Cloud. Verify row counts and checksums. | Use `pg_dump`/`pg_restore`; verify row counts and checksums. |
| Migration rollback | Fail a migration halfway and confirm old API can still read or cleanly abort. | Same; include `CREATE INDEX CONCURRENTLY` path. |
| Turso-to-Postgres fallback | Export logical rows and import into Postgres adapter; run all Q queries unchanged through trait. | N/A, target side. |

Invariants:

- no orphan `issue_events`, `agent_steps`, `cli_invocations`, or `audit_log`;
- issue `event_count` matches `issue_events`;
- agent step sequence has no duplicates and no missing committed steps;
- every bundle has a redaction report ref;
- every raw-access query leaves an audit row;
- export/import preserves IDs so old evidence bundles remain dereferenceable.

## Numeric Decision Gates

Initial gates for `startup realistic` tier on modest single-node hardware:

| Gate | Required result |
| --- | --- |
| Correctness | Zero invariant violations across crash/restart and migration tests. |
| Backup/restore | Restore produces matching table counts and deterministic checksum report. |
| Hot issue contention | No unacceptable lock storm; p99 issue upsert remains below 250 ms or is explicitly queued. |
| Agent timeline query | p95 below 100 ms for 1,000-step session; p99 below 250 ms. |
| Context metadata query | p95 below 100 ms for issue + events + bundle refs. |
| Audit lookup | p95 below 150 ms by resource id over stress-tier audit table. |
| Fallback export | Turso export/import to Postgres preserves logical schema and passes all Q queries. |
| Operational complexity | Tiny profile runs without mandatory external service. |

Postgres becomes the metadata default if Turso fails correctness, backup/restore,
or fallback export. Turso can remain the tiny default if it fails only stress-tier
throughput but passes correctness and has a clean Postgres migration path.

## Minimum Acceptance Criteria

Turso remains the default only if it passes these gates:

- no observed metadata corruption under crash/restart tests;
- clear backup/restore process for local and tiny deployments;
- p95 metadata query latency stays comfortably below the evidence-bundle query
  budget;
- concurrent writes from ingest, agent sessions, and CLI traces do not create
  unacceptable lock contention;
- schema migrations are deterministic and testable;
- data can be exported or migrated to Postgres without changing Parallax's API;
- beta status risk is documented and acceptable for the chosen release stage.

Postgres becomes the default only if Turso fails correctness, backup/restore,
concurrency, migration, or operational-safety gates.

## Bottom Line

Use Turso as the first metadata-store implementation because it matches the
small, Rust-first, self-hosted Parallax direction. But treat it as a benchmarked
choice, not a belief. The decision is valid only if Turso proves safe enough for
metadata and audit state under the exact agent/CLI/runtime workflows Parallax
needs.
