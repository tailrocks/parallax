# Exponential histograms (ingest-converted)

## Problem

OTel SDKs increasingly export `ExponentialHistogram` (notably .NET, Java, and
Prometheus-native bridges). Parallax dropped every such datapoint at ingest
(`dropped_unsupported`), and the native GreptimeDB metric engine has no exp
type — so exp instruments were lost on both stores, silently except for a
health counter. GOAL §4 names exponential histograms explicitly.

## Design

Convert at ingest; never merge with native explicit tables.

1. **Normalize** (`parallax-ingest/src/exp_histogram.rs`): each exp point
   becomes one `HistogramRow` with exact implicit bounds (`base^index`,
   `base = 2^(2^-scale)`), ordered negative → zero → positive. Overflow
   clamps to finite `±MAX`, non-advancing bounds merge mass into the previous
   bucket, degenerate scales refuse conversion (counted, not stored).
   Exemplars survive with trace/span linkage.
2. **Strip** (`strip_exp_histograms`, applied in the worker before the native
   OTLP forward): the originals never reach the engine, so engines that
   accept exp histograms cannot double-store and engines that reject them
   cannot poison the batch. Gauge/sum siblings in the same batch are
   unaffected.
3. **Durable**:
   - Memory store files converted rows with explicit histograms — one query
     path serves both encodings.
   - GreptimeDB persists them to the `exp_histograms` extension table (one
     row per export, bucket grids as JSON), created and TTL-reconciled by
     bootstrap like the other extension tables.
4. **Query** (Greptime): histogram quantile(s)/avg/count fall back to the
   extension table only when no native `_bucket`/`_sum`/`_count` table exists
   for the metric — native explicit wins on mixed encodings. Interpolation
   uses the shared `parallax_storage::adapter_math::explicit_bucket_quantile`
   both stores call, so memory and Greptime agree bucket-for-bucket.
   Attribute filters apply client-side (extension attributes are one JSON
   column, not tag columns).
5. **Discovery**: catalog, metric names, labels, label values, and signal
   counts union the extension table; exp-only instruments are first-class.

`Summary` remains dropped (no lossless representation) and still counts in
`dropped_unsupported`. Counter/histogram rate math assumes cumulative
temporality (pre-existing gap, unchanged by this slice).

## Verification

- `cargo test -p parallax-ingest` (conversion, ordering, merge, exemplars,
  strip, degenerate scale)
- `cargo test -p parallax-storage` (shared interpolation)
- `cargo test -p parallax-greptime` (DDL, decode, series builders, SQL arms)
- `cargo test -p parallax-api metric_query_serves_converted_exp_histograms`
  (quantile + catalog over a converted row)
- `cargo test -p parallax-server --test m12_exp_histogram_greptime -- --ignored`
  (live GreptimeDB 1.1.2 end-to-end incl. MemoryStore parity; requires free
  managed ports)
