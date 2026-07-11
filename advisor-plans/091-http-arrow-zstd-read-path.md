# Plan 091: Adopt HTTP `format=arrow&compression=zstd` for heavy GreptimeDB reads

> **Executor instructions**: Product change under `crates/parallax-storage`
> (and any thin decode helpers). Do not change UI contracts — GraphQL still
> returns the same domain types. Keep JSON for tiny / schema probes. Follow
> steps; STOP if Arrow decode fails on the pinned engine.

## Status

- **Priority**: P2
- **Effort**: S–M
- **Risk**: MEDIUM (decode path + binary response handling; keep JSON fallback)
- **Depends on**: 090 (measurement spike — **DONE**, GO for arrow+zstd)
- **Category**: perf
- **Planned at**: 2026-07-11 (from plan 090 GO)
- **Status**: **DONE** (2026-07-11) — heavy reads via `sql_arrow` /
  `sql_with_schema_arrow` (`format=arrow&compression=zstd`); decode in
  `arrow_sql.rs`; tiny/DDL/schema stay JSON; unit fixtures + storage nextest
  green; no rustls; research note updated.

## Why this matters

Plan 090 measured the live inventory on GreptimeDB 1.1.2 (N=100k, reps=50):

- `logs_search` wall p50 **324 → 258 ms** with Arrow (~20%); zstd payload
  **91 KB → 23 KB**.
- `histogram_buckets` decode p50 **1.04 → 0.05 ms**; zstd **221 KB → 25 KB**.
- `traces_search` wall p50 **95 → 61 ms** (arrow+zstd).
- Uncompressed Arrow can be **larger** than JSON (logs page 711 KB vs 91 KB) —
  always use **zstd**.

Current product path: form POST `/v1/sql` → `serde_json::Value` tree → clone
rows (`greptime.rs` `sql()`).

## Scope

**In scope**:
- `GreptimeStore::sql` (and/or a sibling `sql_arrow`) using
  `format=arrow&compression=zstd` for heavy typed reads.
- Decode via `arrow-ipc` (+ zstd feature) into the same row accessors or a
  zero-copy-friendly intermediate.
- Keep `greptimedb_v1` JSON for: `LIMIT 0` schema probes, single-row counts,
  information_schema, DDL/admin, and any path where the result is expected to
  be tiny.
- Unit/conformance: row-count parity JSON vs Arrow on golden fixtures or live
  conformance (074 suite).
- TLS: no rustls; existing reqwest native-tls stays.

**Out of scope**:
- MySQL/PG wire client (090 NO-GO).
- RANGE syntax rewrites (090 NO-GO).
- Partition-hint product default (090 REVISIT).
- UI changes.

## Steps

### Step 1: Add Arrow decode helper

Small module in `parallax-storage` (or private in `greptime.rs`): bytes →
columnar batches → iterate rows into existing `Vec<Vec<Value>>` **or** a new
internal row view if zero-copy is easy without rewriting all callers.

**Verify**: unit test with a fixture Arrow IPC stream (capture one from 090
harness or greptime).

### Step 2: Dual-format `sql` path

- Heavy methods (`select_spans`, `logs_search`, `traces_search`,
  `histogram_*`, large metric series) call Arrow+zstd.
- Lightweight methods keep JSON.
- Shared error mapping for greptime `error` JSON still applies if the server
  returns JSON on failure.

**Verify**: `cargo nextest` storage + greptime conformance; live parity row
counts for one heavy query.

### Step 3: Dependency + TLS check

Add `arrow` / `arrow-ipc` with zstd; `cargo tree -i rustls` must stay clean in
product crates (arrow-ipc zstd uses the `zstd` crate, not rustls).

**Verify**: tree check in CI or local before commit.

### Step 4: Document

Update the research note or implementation spec with the production default
(arrow+zstd for heavy reads) and point at 090 numbers.

## Done criteria

- [x] Heavy reads use Arrow+zstd; tiny reads stay JSON
- [x] Conformance / tests pass; no rustls in tree
- [x] No UI contract change
- [x] Note in `docs/research/storage/read-transport-and-engine-defaults.md` or
      architecture spec that product adopted the GO

## STOP conditions

- Arrow IPC from pinned Greptime version cannot be decoded by current
  `arrow-ipc` — record versions and keep JSON.
- Conformance row parity fails on native OTLP tables (wider schema than 090
  synthetic) — fix decode or fall back per-query.
