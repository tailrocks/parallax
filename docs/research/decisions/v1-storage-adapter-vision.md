# V1 Storage Adapter Vision

<!-- markdownlint-disable MD013 -->

Decision date: 2026-06-03 · Updated after local-first clarification 2026-06-03

> **Current authority (operator, 2026-06-12; native-table refinement,
> 2026-06-18): GreptimeDB + Turso are mandatory.** GreptimeDB uses native raw
> signal tables; Turso owns metadata. ClickHouse, Postgres, SQLite, and Turso-only
> telemetry are not product modes. Capability traits exist for ownership and
> tests, not backend replacement. This is a historical V1 design record, not an
> active plan. Contract cleanup is plan 093 and server work is plan 115.

## What This Means

The historical design distinguished these profiles; current policy retains only
the GreptimeDB + Turso shape:

- **First local product:** managed local GreptimeDB standalone for telemetry
  evidence plus Turso's SQLite-compatible metadata, because this avoids
  rebuilding observability storage.
- **First production storage profile:** GreptimeDB server/cluster, because it is the best current fit
  for high-volume retained observability evidence.

V1 local should store enough bounded telemetry and metadata to answer:

```text
what happened in run_id X?
which errors grouped together?
which spans/logs/metrics led to that failure?
what bundle should I hand to an agent?
```

The product contract remains engine-encapsulated, not engine-swappable. Parallax
users and agents depend on:

- OpenTelemetry traces, logs, and metrics;
- optional Sentry-compatible error ingest adapter;
- deterministic grouping and correlation;
- evidence graph nodes and edges;
- bounded evidence bundles;
- CLI plus local API access.

They should not depend on Turso table names, GreptimeDB table names, query dialect details, region
layout, object-storage internals, or PromQL-specific implementation behavior. Those belong behind
  capability-specific storage boundaries.

## Local V1 Default

The local profile should optimize for:

- one command;
- no Docker requirement;
- managed GreptimeDB child process or existing GreptimeDB URL;
- Turso local metadata file (SQLite-compatible API, not a SQLite fallback);
- short-lived local retention;
- disposable/prunable run history;
- small and medium local app stacks;
- agent access by `run_id`.

GreptimeDB is the preferred local evidence store because it runs as a standalone binary and the
Greptime Homebrew tap supports `brew install greptime`; `greptime standalone start` launches local
HTTP/RPC/MySQL/Postgres ports. Docker is optional.

Turso Database is the mandatory local metadata engine. Its beta status gates
production claims; reliability, migration, or concurrency failures require
fix-forward work.

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

This is not a claim that GreptimeDB is universally faster than ClickHouse.
ClickHouse remains an analytics comparator; cost/cold-read results guide
GreptimeDB or Parallax remediation.

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

Exact names can change under numbered plans, but the principle should not:
callers ask for Parallax evidence, not database queries. GreptimeDB/Turso-specific
details stay inside capability implementations.

Minimum storage profiles:

| Profile | Role | Status |
| --- | --- | --- |
| `local-greptimedb` | Default local V1 evidence profile using managed GreptimeDB standalone. | Build first for CLI/local runs. |
| `local-metadata` | Turso metadata/grouping capability. | Mandatory with local GreptimeDB. |
| `greptimedb` | Default production/server observability storage. | Same model as local GreptimeDB, scaled up. |

## Why Keep It Extensible

Extensibility protects three real futures:

1. **Local-only mode.** Developer runs Parallax fully local, with managed GreptimeDB and no Docker.
   Turso handles grouping/state; GreptimeDB handles logs/traces/metrics.
2. **Deployment growth.** Local and future server profiles preserve one bundle
   contract while using the same mandatory engines.

## Non-Negotiables

- V1 local implementation should be managed-GreptimeDB-plus-Turso-shaped.
- V1 server implementation should remain GreptimeDB-shaped.
- API and evidence-bundle contracts hide engine details but may use approved
  GreptimeDB native extensions.
- Capability boundaries must not imply an alternate product engine.
- ClickHouse and Postgres remain research comparators only.

## Relationship To Existing Decisions

- [Local-first V1 concept](../architecture/local-first-v1.md) explains the one-command, `run_id`-based
  developer loop.
- [Storage engine decision](storage-engine.md) explains why GreptimeDB currently beats ClickHouse for
  Parallax's first production storage focus.
- [Technical implementation concept](../architecture/implementation-concept.md) places storage behind a
  swappable adapter and keeps product metadata separate from high-volume observability evidence.
- [Metadata store decision](metadata-store.md) covers relational metadata; this page narrows V1 local
  storage separately from production metadata.
