# Metadata Store Benchmark Plan

<!-- markdownlint-disable MD013 -->

Research date: 2026-05-25

## Purpose

Parallax needs a separate benchmark for low-volume product metadata. This is not
the GreptimeDB/ClickHouse observability-storage benchmark. The metadata store
holds control-plane and audit state:

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
