# Write Path and Ingestion — Side by Side (Freshness, Axis #1)

<!-- markdownlint-disable MD013 -->

Status: pass 9. The top evaluation axis: **ingest → durable → queryable**, and
exactly when written data becomes visible (freshness). Combines the write-path
mechanisms from the internals notes with an empirical freshness/throughput probe
(`local-benchmark-results.md` Run 5).

Pins: GreptimeDB `v1.0.2` (`0ef5451`), ClickHouse `v26.5.1.882-stable` (`5b96a8d8`).

## Write path, step by step

| Step | GreptimeDB | ClickHouse |
| --- | --- | --- |
| Buffer | Row → **WAL** (raft-engine local, or Kafka remote) → **mutable memtable** (`TimeSeries`/`PartitionTree`). | Rows sorted by `ORDER BY` → written as an **immutable part** (file-per-column Wide, or packed Compact). |
| Durability | WAL append (fsync policy configurable). | The part write itself is the durability unit; fsync on part commit. |
| Visibility (freshness) | **On `committed_sequence` bump** — visible as soon as the row is in the mutable memtable; **no flush needed** (MVCC `Version` snapshot includes live memtables). | **On part commit** — visible as soon as the part is atomically added; **no merge needed**. |
| Flush / background | Memtable freezes → Parquet SST when the write-buffer fills; SSTs compacted by time window. | Background **merges** combine small parts into larger ones (and apply the engine-variant row transform). |
| Small-write absorption | LSM memtable absorbs many small writes **in memory**; one SST per flush. | **One part per INSERT** → many small inserts = many small parts. |

## The freshness mechanism — both are visible-on-write

Run 5 (smoke) confirmed the architecture: a single synchronous insert is
**immediately queryable on both** — the row returned on the first query after the
insert ack, on both engines. Neither requires a flush/merge for visibility. The
per-call millisecond figures (CH 288 ms, GT 124 ms for one insert+ack) are
dominated by client/process + HTTP overhead, **not** the freshness mechanism, so
they do not rank the engines. **Freshness is a tie at the mechanism level: both
make data visible synchronously on the write, sub-second, no flush barrier.**

This matches `greptimedb-internals.md` (queryable on memtable insert via
`committed_sequence`) and `clickhouse-internals.md` (queryable on part commit).

## The real write-path difference: small-write absorption

The mechanism that *does* differ and matters for Parallax:

- **ClickHouse writes one part per INSERT.** High-frequency small inserts (e.g.
  per-event telemetry, one row at a time) create many tiny parts, triggering merge
  pressure and eventually the `parts_to_throw_insert` guard ("too many parts"). The
  fix is **batching**: client-side batching, or **async inserts** (server-side
  buffer). Notably, this ClickHouse 26.x image reports **`async_insert = 1` by
  default** (busy timeout 50–200 ms) — so small inserts are auto-batched, becoming
  visible after the ~50–200 ms buffer window rather than instantly. (Default-on
  async insert is a freshness-vs-throughput tradeoff baked in; verify it is the
  server default vs the image profile before relying on it.)
- **GreptimeDB's LSM memtable absorbs small writes natively.** Many small writes
  accumulate in the in-memory memtable and flush together as one SST — **no
  part-per-insert explosion**, no "too many parts" failure mode, no mandatory
  batching layer. For an ingest pattern of frequent small batches (Parallax's
  likely shape: events/spans/logs streaming in), this is a **genuine write-path
  advantage for GreptimeDB** — it removes the client-side batching burden that
  ClickHouse imposes.

→ Axis-1 consequence: **freshness latency is a tie** (both visible-on-write), but
**operational ingest ergonomics favor GreptimeDB** for high-frequency small
writes — ClickHouse needs a batching strategy to stay healthy, GreptimeDB does
not. For bulk/batched ingest both are fine (below).

**Refinement (Run 7, B9 — measured).** The part-per-insert mechanism is real
(ClickHouse logged 300 `NewPart` events for 300 single-row inserts), **but
background merges collapse bounded bursts aggressively** (300 parts → 1 active via
61 merges) and the `parts_to_throw_insert` guard is far off (**3000**). So "too
many parts" is a **sustained-rate** failure (insert rate persistently > merge
throughput), **not** a per-insert problem, and `async_insert` (default on in 26.x)
mitigates it. The GreptimeDB advantage here is therefore **real but narrower than
first stated**: it matters under *sustained* tiny-write rates that outpace
ClickHouse's merges — for bounded/moderate bursts ClickHouse copes. (This walks
back the original strength of this claim; honest correction.)

## Async insert — ClickHouse's server-side buffer (mechanism, pass 56)

ClickHouse's answer to small-write absorption is a **server-side buffer**, the
`AsynchronousInsertQueue` (`src/Interpreters/AsynchronousInsertQueue.cpp`). Small
inserts accumulate per (query, settings) key and **flush to one part** when any
trigger fires (live defaults, Run 33): `async_insert_max_data_size` = **10 MiB**,
`async_insert_max_query_number` = **450**, or the **adaptive busy timeout**
`async_insert_busy_timeout_min_ms`=50 / `max_ms`=200 (`async_insert_use_adaptive_busy_timeout`
tunes it by load). `async_insert=1` default-on in 26.x.

The buffer **trades freshness and/or durability for throughput**:

- **Until the buffer flushes, the data is neither queryable nor a durable part.** The
  window is bounded by the busy timeout (≤200 ms).
- **`wait_for_async_insert`=1 (default):** the client *blocks* until the flush — so on
  ack the data is visible + on-disk, but the insert call **absorbs the ≤200 ms buffer
  latency**.
- **`wait_for_async_insert`=0:** fast ack, but a ≤200 ms window where the data is
  **invisible and not durable** (lost on crash — ties `wal-and-durability.md`).

Either way there is a ~50–200 ms cost *somewhere* — blocking on ack (wait=1) or an
invisibility/loss window (wait=0). (The window is too small to catch cleanly across
separate `docker exec` calls, each ~50–100 ms; a single async insert had already
flushed by query time — the *mechanism* and triggers are source+settings-confirmed.)

**GreptimeDB has no async-insert buffer — and needs none.** Its **LSM memtable is the
buffer**: small writes accumulate in memory and flush to one SST when the write buffer
fills, *exactly* the part-explosion fix — but the memtable is **queryable immediately**
(`committed_sequence`, visible-on-write, Run 5; re-confirmed Run 33: a single insert
→ `count=1` instantly, no window) **and durable immediately** (WAL-first,
`wal-and-durability.md`). So GreptimeDB gets the small-write-absorption benefit with
**neither** the freshness window **nor** the durability tradeoff that ClickHouse's
async buffer imposes — the absorption is native to the storage engine, not a bolt-on
queue in front of a part-per-insert engine.

→ Sharpens the axis-1 write-path point: ClickHouse *can* absorb small writes (async
insert), but the mechanism is a buffer that costs a ≤200 ms freshness/durability/
latency window; GreptimeDB's LSM gives the same absorption for free, visible+durable on
write. For Parallax's streaming small-write ingest this is a clean GreptimeDB
ergonomics + freshness edge (mechanism-grounded, modest in absolute ms).

## Bulk ingest throughput (Run 5, smoke)

| Engine | 1M spans load | ~rows/s | Measurement |
| --- | --- | --- | --- |
| ClickHouse | 0.575 s (`INSERT … FROM INFILE FORMAT CSV`) | ~1.74M | client wall (`--time`) |
| GreptimeDB | 0.895 s (`COPY … FORMAT CSV`) | ~1.12M | server `execution_time_ms` |

ClickHouse ~1.55× faster on this bulk CSV load, but the measurement bases differ
(client wall vs server time) and it is single-file, non-concurrent, smoke scale.
**Both exceed 1M rows/s single-node** — neither is an ingest bottleneck for a
Tier-1 Parallax deployment. The throughput ranking is *inconclusive* pending a
matched-protocol, concurrent ingest+query run.

## Ingest protocols (capability)

| | GreptimeDB | ClickHouse |
| --- | --- | --- |
| Native protocols | OTLP (traces/logs/metrics), Prometheus remote-write, InfluxDB line, MySQL/PG wire, gRPC, HTTP SQL — all GA. | Native TCP, HTTP, many input formats; OTLP via an exporter/Collector. **Correction (pass 44):** Prom remote-write is **no longer absent** — the **experimental `TimeSeries` engine** (off by default) accepts it. So this is GA-native (GreptimeDB) vs experimental (ClickHouse), not present-vs-absent. See `promql-and-metrics-query.md`. |
| Parallax fit | OTLP + Prom remote-write **native** → telemetry lands with no translation. | **OTLP** needs an OTel-Collector + ClickHouse-exporter pipeline; **Prom** remote-write lands in the experimental `TimeSeries` engine. |

**OTLP re-verified at ClickHouse 26.5 (pass 46) — claim HOLDS, no drift.** Unlike
PromQL/Prom-remote-write (which ClickHouse *did* add, pass 44), there is **no native
OTLP receiver** in ClickHouse 26.5: no `otlp`/`otel` table function or function
(`system.table_functions`/`system.functions` empty), and no OTLP HTTP handler in
`src/Server`. OTLP ingest still requires the OTel Collector + ClickHouse exporter (or a
bundled collector like ClickStack/HyperDX). GreptimeDB has a **native, GA OTLP receiver
for all three signals** — `src/servers/src/http/otlp.rs` handles `metrics`/`traces`/
`logs` via the official `opentelemetry_proto` + OTel-Arrow; `/v1/otlp/v1/{metrics,
traces}` returned **HTTP 400** live (endpoint exists, rejects a bad payload — not 404).
**Nuance:** ClickHouse's 26.x observability-protocol investment went to **Prometheus**
(TimeSeries + remote-write + PromQL), **not OTLP** — so for Parallax's OTLP-centric
telemetry the native-ingest advantage stays decisively with GreptimeDB.

**Confirmed live (pass 33) — schema-on-write native ingest.** A native InfluxDB
line-protocol write (`POST /v1/influxdb/write`, plain text, HTTP 204) **auto-created
the table** with the correct mapping, no DDL: tags → `PRIMARY KEY`, field →
`DOUBLE`, an auto `greptime_timestamp` `TIME INDEX`, and
`merge_mode='last_non_null'` (upsert-last). Queryable immediately. This is the
capability ClickHouse lacks twice over — it has **no** native InfluxDB/OTLP
ingest endpoint (needs a collector; Prom remote-write is the one exception, via the
experimental `TimeSeries` engine — pass 44) **and** requires the table to be created
first (no schema-on-write). **Nuance found:** GreptimeDB's **OTLP metrics endpoint is
protobuf-only** — JSON is explicitly rejected (`src/servers/src/http/otlp.rs:80`,
`UnsupportedJsonContentType`); in practice an OTel Collector/SDK emits protobuf, so
this is fine, but you cannot hand-POST OTLP JSON. So "native OTLP" = the binary
protocol a collector/SDK speaks, not a JSON shortcut.

## Freshness/write-path verdict (axis #1)

| Question | Answer | Confidence |
| --- | --- | --- |
| Ingest-to-queryable latency | **Tie** — both visible-on-write, sub-second, no flush barrier. | smoke + arch |
| Small high-frequency writes | **GreptimeDB** — LSM memtable absorbs them; ClickHouse needs batching/async-insert to avoid part explosion. | arch (well-known CH failure mode) |
| Bulk ingest throughput | ClickHouse ~1.55× here, but both >1M rows/s; inconclusive at smoke. | smoke |
| Native OTLP / Prom ingest | **GreptimeDB** — native; ClickHouse needs a collector. | arch |

**Net:** freshness latency does not separate the engines (both fresh-on-write).
The write-path advantages that *do* exist favor **GreptimeDB** for Parallax's
likely ingest shape (streaming small batches of OTLP/Prom telemetry): native
protocols + memtable absorption of small writes without a batching layer. This is
the first axis where the architecture leans GreptimeDB on grounds other than
metrics — and it is axis #1.

## What still needs measuring

- **Concurrent ingest+query freshness** (the harness protocol: stamp `t_emit`,
  poll every 50 ms until visible, p50/p95/p99 under mixed load) — the real axis-1
  number; not yet run.
- **ClickHouse part-explosion threshold** empirically (small-insert rate until
  "too many parts") vs GreptimeDB steady-state under the same rate.
- Confirm whether `async_insert=1` is the genuine ClickHouse 26.x server default.

## Source / evidence

- GreptimeDB write path: `src/mito2/src/{worker.rs,region_write_ctx.rs,flush.rs}`, WAL `src/log-store/src/lib.rs`; visibility via `committed_sequence` (`src/mito2/src/lib.rs`).
- ClickHouse write path: part creation in `src/Storages/MergeTree/`; async insert `src/Interpreters/AsynchronousInsertQueue.cpp`; `parts_to_throw_insert` in `MergeTreeSettings`.
- Empirical: `local-benchmark-results.md` Run 5.
