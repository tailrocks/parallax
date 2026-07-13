# Metric Exemplars Design

Status: **shipped design record**. The historical plan 033 implementation now
normalizes number/histogram exemplars, persists them in the
`metric_exemplars` extension table, and exposes bounded API reads. The remaining
schema correction is active only in
[Plan 092 closure](https://github.com/tailrocks/parallax/commit/953409b):
remove high-cardinality trace/span identifiers from the primary key and migrate
existing data. The design rationale below describes the adopted shape, not an
unfinished implementation plan.

## Storage Shape

Exemplars should not be added only to `run_metric_points`.

`run_metric_points` is useful for the run-scoped subset because it already
stores Parallax-owned point rows outside the Greptime metric engine, and a
`run_id` column on exemplar rows preserves run filtering when present. It is
not enough for the general case because most metric points do not carry
`parallax.run.id`; keeping exemplars there would silently drop non-run metric
to trace links.

The adopted Parallax extension table, `metric_exemplars`, follows the same
bootstrap and batched insert pattern as `run_metric_points`:

- `ts` as `TIMESTAMP(9)` time index
- `service`, `name`, `value`
- `trace_id`, `span_id`
- optional `run_id`
- `attributes` JSON for exemplar filtered attributes

Greptime native metric-engine tables still receive the raw OTLP batch for
normal metric reads. The exemplar table is the correlation sidecar queried by
`metricExemplars`.

## Ingest

The pinned `opentelemetry-proto 0.32.0` types expose the required fields:

- `NumberDataPoint.exemplars: Vec<Exemplar>`
- `HistogramDataPoint.exemplars: Vec<Exemplar>`
- `Exemplar.trace_id: Vec<u8>`
- `Exemplar.span_id: Vec<u8>`
- `Exemplar.time_unix_nano: u64`
- `Exemplar.filtered_attributes: Vec<KeyValue>`
- `Exemplar.value` oneof with double or int values

Normalization reads exemplars while it already iterates each metric data
point. It borrows the request, converts only each exemplar row into the storage
shape, and never clones the telemetry batch. Existing unavoidable per-row
allocations stay local: service/name strings, hex trace/span ids, and JSON
attributes.

## Producer Coverage

At implementation time the Rust playground lacked metric exemplars because the
Rust SDK did not emit them. Historical plan 033 therefore used
JVM/Micrometer-style exemplar data and synthetic OTLP fixtures instead of
changing the producer. The shipped UI renders the explicit no-exemplar fallback
"No trace exemplar attached; showing traces near this timestamp," with route
tests pinning that behavior.

## Query Cost

`metricExemplars(name, fromNanos, toNanos, service?, limit?)` should read only
the extension table. The query is bounded by metric name and an inclusive time
range on the table time index, with optional service filtering and a small
server-side limit. This keeps the expensive native metric tables out of the
correlation path and avoids scanning raw OTLP payloads.
