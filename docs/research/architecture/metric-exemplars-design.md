# Metric Exemplars Design

Status: **shipped design record** (re-verified 2026-07-17 against source). The
historical plan 033 implementation normalizes number/histogram exemplars,
persists them in the `metric_exemplars` extension table, and exposes bounded
API reads. Invocation-scoped metric points live in `invocation_metric_points`
(legacy `run_metric_points` is dropped at bootstrap). The design rationale
below describes the adopted shape, not an unfinished implementation plan.

## Storage Shape

Exemplars should not be added only to `invocation_metric_points`.

`invocation_metric_points` is useful for the invocation-scoped subset because
it already stores Parallax-owned point rows outside the Greptime metric engine,
and an invocation/run key on exemplar rows preserves run filtering when
present. It is not enough for the general case because most metric points do
not carry `cli.invocation.id` / legacy `parallax.run.id`; keeping exemplars
there would silently drop non-invocation metric to trace links.

The adopted Parallax extension table, `metric_exemplars`, follows the same
bootstrap and batched insert pattern as `invocation_metric_points`:

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
