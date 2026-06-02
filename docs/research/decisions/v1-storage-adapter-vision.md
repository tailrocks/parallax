# V1 Storage Adapter Vision

<!-- markdownlint-disable MD013 -->

Decision date: 2026-06-03

> **Decision — V1 is GreptimeDB-first and adapter-extensible.** The first implementation should be
> designed around GreptimeDB as the default observability storage engine because it gives Parallax the
> fastest path to a useful evidence engine: unified metrics/logs/traces, OpenTelemetry-oriented ingest,
> SQL and PromQL, and cloud-native storage/scale assumptions that fit retained runtime evidence. The
> product contract still goes through a storage adapter so future profiles can target ClickHouse,
> local-only storage, Turso/SQLite-like embedded storage, or another backend without changing the
> evidence-bundle API.

## What This Means

V1 should not pretend storage is neutral in the implementation details. The first schema, query
patterns, ingest buffering, and retention workflow should be optimized for GreptimeDB because that is
the shortest path to shipping automatic evidence bundles.

But V1 also must not bake GreptimeDB into the Parallax product contract. Parallax users and agents
should depend on:

- Sentry-compatible error ingest;
- OpenTelemetry traces, logs, and metrics;
- deterministic grouping and correlation;
- evidence graph nodes and edges;
- bounded evidence bundles;
- CLI/HTTP context API.

They should not depend on GreptimeDB table names, query dialect details, region layout, object-storage
internals, or PromQL-specific implementation behavior. Those belong behind `StorageAdapter`.

## Why GreptimeDB First

GreptimeDB is the default V1 focus for practical product speed:

1. **It matches Parallax's data shape.** Parallax stores observability evidence: errors, traces, logs,
   metrics, and time-windowed context. Current GreptimeDB docs describe it as a unified observability
   database for metrics, logs, and traces, with SQL and PromQL support.
2. **It reduces build surface.** GreptimeDB already gives Parallax many observability-oriented features
   out of the box. That lowers the amount of custom compatibility glue needed before V1 can assemble
   useful bundles.
3. **It fits the anchored hot path.** Parallax primarily fetches all evidence for one issue, trace,
   fingerprint, or narrow time window. ClickHouse is stronger for broad analytics, but existing research
   says both engines are interactive for anchored bundle retrieval.
4. **It supports metric evidence cleanly.** Metrics are part of the bundle, not a separate product.
   GreptimeDB's PromQL-compatible path makes Prometheus-style evidence easier to expose.
5. **It aligns with the Rust-first strategy.** GreptimeDB is Rust, so deeper debugging, contribution,
   and long-term operator control are more realistic than with a C++ engine.

This is a shipping decision, not a claim that GreptimeDB is universally better than ClickHouse.
ClickHouse remains the fallback for analytics-heavy workloads and if cost/cold-read benchmarks overturn
the GreptimeDB assumption.

## Adapter Boundary

The storage layer should expose operations in Parallax terms, not database terms:

```text
write_error_event(...)
write_span_batch(...)
write_log_batch(...)
write_metric_batch(...)
write_deploy_event(...)
fetch_issue_window(...)
fetch_trace_evidence(...)
fetch_metric_window(...)
fetch_log_window(...)
build_bundle_inputs(...)
enforce_retention(...)
```

The exact names can change during implementation, but the principle should not: callers ask for
Parallax evidence, not GreptimeDB queries. GreptimeDB-specific SQL, PromQL, schemas, indexes, and
retention behavior stay inside the GreptimeDB adapter.

Minimum V1 profiles:

| Profile | Role | Status |
| --- | --- | --- |
| `greptimedb` | Default V1 observability storage. | Build first. |
| `clickhouse` | Fallback for raw analytical speed and broad log/trace search. | Keep interface ready; implement when needed or when benchmarks flip. |
| `local` / `turso` | Future local-first or single-user profile using embedded SQLite-compatible storage. | Design for later; do not make it block V1. |

## Why Keep It Extensible

Extensibility is not architecture ceremony here. It protects three real futures:

1. **Local-only mode.** A developer may want Parallax fully local, with no GreptimeDB container. A
   future embedded profile could store enough evidence for small projects, demos, tests, or personal
   debugging.
2. **Storage-result reversibility.** The GreptimeDB-vs-ClickHouse decision is still benchmark-gated.
   If real $/GB, cold-read, or query-mix results flip, Parallax needs a swap path.
3. **Different deployment sizes.** Tiny local, single-node self-hosted, and durable server deployments
   may deserve different backends while preserving one bundle contract.

Turso Database is a plausible future local profile because its current repository describes an
in-process SQL database written in Rust, compatible with SQLite, and usable against local database
files from Rust. It is still beta, so it should be treated as a future gated option, not the V1
observability store.

## Non-Negotiables

- V1 implementation can be GreptimeDB-shaped.
- V1 API and evidence-bundle contract must be backend-neutral.
- No GreptimeDB-only feature may become required for bundle correctness unless the adapter contract has
  a portable fallback.
- ClickHouse remains the explicit fallback, not a rejected engine.
- Local/Turso-like storage remains a future profile, not a reason to weaken the GreptimeDB-first V1.

## Relationship To Existing Decisions

- [Storage engine decision](storage-engine.md) explains why GreptimeDB currently beats ClickHouse for
  Parallax's first storage focus.
- [Technical implementation concept](../architecture/implementation-concept.md) places storage behind a
  swappable adapter and keeps product metadata separate from high-volume observability evidence.
- [Metadata store decision](metadata-store.md) covers Turso-first relational metadata; this page only
  says a Turso/SQLite-like local evidence profile should remain possible later.
