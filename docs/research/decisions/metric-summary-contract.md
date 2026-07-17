# Metric summary contract

Decision date: 2026-07-17

Status: **APPROVED**

Approval: operator unblock directive dated 2026-07-17, executed through
Plans 105 and 168

Machine-enforced mirror:
[`metric-summary-contract.toml`](metric-summary-contract.toml). Product policy
rejects a missing record/fixture, unknown or missing fields, and any value that
differs from this approved contract.

This record defines the bounded metric-summary semantics shared by the
overview, metric explorer, dashboards, alerts, and CLI. It preserves
GreptimeDB native per-metric tables; it does not authorize another raw metric
table. Invocation-scoped product points remain in the already-approved
`invocation_metric_points` extension described in
[native-otel-tables.md](native-otel-tables.md).

## Window and counting

- Every overview count and trend is evaluated over the caller's explicit,
  inclusive nanosecond range `[from_nanos, to_nanos]`. Missing or reversed
  bounds are errors; there is no lifetime scan or implicit retention window.
- `metric_point_count` counts exported metric samples, not scalar values
  materialized by a query. One finite gauge or sum row counts once. One
  explicit-histogram export counts once, identified by its `_count` row;
  `_bucket` and `_sum` siblings do not add points.
- A sample whose value is `NaN`, positive/negative infinity, or a native stale
  marker does not count and does not participate in aggregation. Empty windows
  return count `0` and an empty series. Unsupported metric kinds are absent,
  never fabricated as zero-valued samples.
- Gauge and sum samples are eligible. Explicit histograms are eligible through
  their native `_bucket`/`_count`/`_sum` family. Exponential histograms remain
  unsupported by the committed native-table contract and are excluded until
  GreptimeDB supports them or the native-table escalation process approves an
  extension.

## Trends and bounds

- Trend buckets are left-closed/right-open `[start, end)`, except the final
  bucket includes the request's `to_nanos` endpoint. Bucket timestamps are the
  bucket starts. Empty buckets are returned as zero so aligned current/previous
  windows remain comparable.
- A requested step is rounded up, never down, to cover the range with at most
  **120 buckets**. The minimum effective step is one second. If no step is
  supplied, use `max(1 second, ceil(window / 60))`, producing at most 60
  buckets before endpoint alignment.
- Counting and aggregation happen in GreptimeDB SQL. Catalog size, result
  rows, and round trips are bounded independently of the number of UI rows;
  per-metric resolver or catalog-row fan-out is forbidden.

## Native metric names

- The canonical query/display name is the native public-table base name.
  Explicit-histogram siblings collapse only when the complete native family is
  present; `_bucket`, `_count`, or `_sum` suffixes on an unrelated scalar
  metric are not stripped.
- User/semantic names may resolve through the existing native normalization
  candidates (for example dots to underscores and native unit/`_total`
  suffixes) only when exactly one physical metric family matches. Zero matches
  means not found. Multiple matches are a typed collision error. Selection by
  candidate order is forbidden because it silently reads the wrong metric.
- The catalog returns canonical names. Permalinks and graduation payloads are
  rewritten to that canonical name after resolution, making reloads stable.
  No lossy reverse transformation from underscores back to dots is attempted.

## Metric-only service discovery

- A service with at least one eligible finite native metric sample inside the
  requested window is an active service even when it has no span or log in the
  window.
- Service identity comes from the native metric table's resource-derived
  service tag using the same canonical `service.name` semantic-convention
  contract as other signals. Empty identities are ignored; label values are
  deduplicated and sorted. Discovery is bounded and batched with the metric
  catalog, never queried once per service or metric row.

## CLI disposition

The V1 CLI promise is retained as:

```text
parallax metrics --invocation <invocation-id>
```

It reads the bounded `invocation_metric_points` product extension through the
canonical API; it never scans native metric tags for `cli.invocation.id` and
never accepts the retired `--run` spelling. The command returns no-data
success for a known invocation with no points and not-found for an unknown
invocation. Machine output uses the same canonical metric names and finite
value rules as GraphQL.

## Compatibility

- Existing GraphQL field names, argument names, scalar types, inclusive range
  convention, and string-encoded counts remain compatible. Stubbed zero/empty
  results gain the semantics above; no field is silently repurposed.
- New catalog/query fields are additive and use this contract as their sole
  read path. Explorer, dashboard, alert, and CLI code must not fork metric SQL.
- This is contract version **1**. Any change to point eligibility, bucket
  boundaries/caps, collision behavior, or CLI disposition requires a new
  approved decision revision plus adapter/API/UI compatibility evidence.

## Required conformance evidence

Implementation is incomplete until fixtures prove: inclusive endpoints;
finite versus non-finite samples; one-count-per-histogram export; empty bucket
filling and the 120-bucket cap; ambiguous native-name rejection; a metric-only
service; MemoryStore/GreptimeDB parity; GraphQL snapshots; and
`--invocation`/retired-`--run` CLI behavior.
