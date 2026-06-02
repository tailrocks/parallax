# V1 Storage Adapter Vision

<!-- markdownlint-disable MD013 -->

Decision date: 2026-06-03 · Updated after local-first clarification 2026-06-03

> **Decision — V1 is local-first and adapter-extensible; managed local GreptimeDB is the preferred
> evidence store.** The first implementation should feel like one command on a developer machine:
> Parallax manages a local GreptimeDB standalone process for observability evidence and uses
> Turso/SQLite-like storage for local metadata/grouping state. The product contract still goes through
> a storage adapter so future profiles can target Turso-only fallback, GreptimeDB server/cluster,
> ClickHouse, or another backend without changing the evidence-bundle API.

## What This Means

There are two different "firsts":

- **First local product:** managed local GreptimeDB standalone for telemetry evidence plus
  Turso/SQLite-like metadata, because this avoids rebuilding observability storage.
- **First production storage profile:** GreptimeDB server/cluster, because it is the best current fit
  for high-volume retained observability evidence.
- **Fallback local product:** Turso/SQLite-like only, for ultra-small demos/tests where no GreptimeDB
  sidecar is allowed.

V1 local should store enough bounded telemetry and metadata to answer:

```text
what happened in run_id X?
which errors grouped together?
which spans/logs/metrics led to that failure?
what bundle should I hand to an agent?
```

The product contract remains backend-neutral. Parallax users and agents depend on:

- OpenTelemetry traces, logs, and metrics;
- optional Sentry-compatible error ingest adapter;
- deterministic grouping and correlation;
- evidence graph nodes and edges;
- bounded evidence bundles;
- CLI plus local API access.

They should not depend on Turso table names, GreptimeDB table names, query dialect details, region
layout, object-storage internals, or PromQL-specific implementation behavior. Those belong behind
`StorageAdapter`.

## Local V1 Default

The local profile should optimize for:

- one command;
- no Docker requirement;
- managed GreptimeDB child process or existing GreptimeDB URL;
- Turso/SQLite-like local metadata file;
- short-lived local retention;
- disposable/prunable run history;
- small and medium local app stacks;
- agent access by `run_id`.

GreptimeDB is the preferred local evidence store because it runs as a standalone binary and the
Greptime Homebrew tap supports `brew install greptime`; `greptime standalone start` launches local
HTTP/RPC/MySQL/Postgres ports. Docker is optional.

Turso Database is the leading local metadata candidate because current docs describe an in-process SQL
database written in Rust, compatible with SQLite, with local file and in-memory database examples. It
is still beta, so V1 must keep a fallback path and avoid production durability claims until gates pass.

Plain SQLite or another embedded store can substitute if Turso fails local reliability, migration, or
concurrency checks. Turso-only telemetry storage remains a fallback, not the preferred V1 path.

## GreptimeDB Server Profile

GreptimeDB is still the default production/server focus:

1. **It matches Parallax's high-volume data shape.** Parallax stores observability evidence: errors,
   traces, logs, metrics, and time-windowed context. GreptimeDB docs describe it as a unified
   observability database for metrics, logs, and traces, with SQL and PromQL support.
2. **It reduces server build surface.** GreptimeDB gives observability-oriented features out of the
   box, so Parallax needs less custom storage glue before server bundles work.
3. **It fits the anchored hot path.** Parallax primarily fetches all evidence for one issue, trace,
   fingerprint, run, or narrow window. ClickHouse is stronger for broad analytics, but existing
   research says both engines are interactive for anchored bundle retrieval.
4. **It supports metric evidence cleanly.** Metrics are part of the bundle, not a separate product.
   GreptimeDB's PromQL-compatible path makes Prometheus-style evidence easier to expose.
5. **It aligns with the Rust-first strategy.** GreptimeDB is Rust, so deeper debugging, contribution,
   and long-term operator control are more realistic than with a C++ engine.

This is a server-profile decision, not a claim that GreptimeDB is universally better than ClickHouse.
ClickHouse remains the fallback for analytics-heavy workloads and if cost/cold-read benchmarks overturn
the GreptimeDB assumption.

## Adapter Boundary

The storage layer should expose operations in Parallax terms, not database terms:

```text
start_run(...)
finish_run(...)
write_error_event(...)
write_span_batch(...)
write_log_batch(...)
write_metric_batch(...)
write_deploy_event(...)
fetch_run_window(...)
fetch_issue_window(...)
fetch_trace_evidence(...)
fetch_metric_window(...)
fetch_log_window(...)
build_bundle_inputs(...)
enforce_retention(...)
```

Exact names can change during implementation, but principle should not: callers ask for Parallax
evidence, not database queries. Backend-specific SQL, schemas, indexes, retention behavior, and query
dialects stay inside adapters.

Minimum storage profiles:

| Profile | Role | Status |
| --- | --- | --- |
| `local-greptimedb` | Default local V1 evidence profile using managed GreptimeDB standalone. | Build first for CLI/local runs. |
| `local-metadata` | Turso/SQLite-like metadata/grouping profile. | Build with local GreptimeDB profile. |
| `local-turso-only` | Ultra-small fallback using embedded storage only. | Keep limited; do not make it main observability store. |
| `greptimedb` | Default production/server observability storage. | Same model as local GreptimeDB, scaled up. |
| `clickhouse` | Fallback for raw analytical speed and broad log/trace search. | Keep interface ready; implement when needed or when benchmarks flip. |

## Why Keep It Extensible

Extensibility protects three real futures:

1. **Local-only mode.** Developer runs Parallax fully local, with managed GreptimeDB and no Docker.
   Turso handles grouping/state; GreptimeDB handles logs/traces/metrics.
2. **Storage-result reversibility.** The GreptimeDB-vs-ClickHouse decision is still benchmark-gated. If
   real $/GB, cold-read, or query-mix results flip, Parallax needs a swap path.
3. **Different deployment sizes.** Tiny local, single-node self-hosted, durable server, and scale-out
   deployments may deserve different backends while preserving one bundle contract.

## Non-Negotiables

- V1 local implementation should be managed-GreptimeDB-plus-Turso-shaped.
- V1 server implementation should remain GreptimeDB-shaped.
- API and evidence-bundle contract must be backend-neutral.
- No backend-only feature may become required for bundle correctness unless the adapter contract has a
  portable fallback.
- ClickHouse remains the explicit fallback, not a rejected engine.
- Turso-only storage is a fallback, not the default observability profile.

## Relationship To Existing Decisions

- [Local-first V1 concept](../architecture/local-first-v1.md) explains the one-command, `run_id`-based
  developer loop.
- [Storage engine decision](storage-engine.md) explains why GreptimeDB currently beats ClickHouse for
  Parallax's first production storage focus.
- [Technical implementation concept](../architecture/implementation-concept.md) places storage behind a
  swappable adapter and keeps product metadata separate from high-volume observability evidence.
- [Metadata store decision](metadata-store.md) covers relational metadata; this page narrows V1 local
  storage separately from production metadata.
