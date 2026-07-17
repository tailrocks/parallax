# Releases Lane Design

> **Status (2026-07-17): historical spike/design record.** Current GraphQL and UI
> provide service, issue, trace, dashboard, investigation, and alert surfaces;
> this note does not imply a separate shipped Releases route. Any remaining
> release-lane work requires current plan ownership.

Plan 041 spike output. Current target: ship service release windows and a
latest-release badge without changing the ingest hot path.

## Release Timeline Query

Greptime's native traces table materializes resource attributes as widened
columns. The release window query works against live A13 playground data with
quoted dotted column names and `timestamp` cast to bigint nanoseconds:

```sql
SELECT
  service_name,
  "resource_attributes.service.version" AS version,
  MIN(CAST("timestamp" AS BIGINT)) AS first_seen_nanos,
  MAX(CAST("timestamp" AS BIGINT)) AS last_seen_nanos,
  COUNT(*) AS span_count
FROM opentelemetry_traces
WHERE "resource_attributes.service.version" IS NOT NULL
GROUP BY service_name, "resource_attributes.service.version"
ORDER BY service_name, first_seen_nanos
LIMIT 20;
```

Live result on 2026-07-08 included `checkout` windows for `0.1.0`, `v1`, and
`v2`; the `v2` window covered the A13 error phase. Greptime requires quotes
around both `timestamp` and `resource_attributes.service.version`. The memory
adapter should read `span.resource["service.version"]` from `SpanRow`.

## Issue And Release Linkage Options

Chosen for this plan: **option A, read-time trace back-join**.

Option A: issue -> `last_trace_id` -> `spans_by_trace` -> first span resource
`service.version`.

- Cost: one bounded trace read on issue detail, already the pattern used by the
  UI to get `traceRunId` from correlated spans.
- Benefit: no ingest-path change, no schema migration, no extra derived row
  state, and graceful fallback when the trace aged out.
- Limit: only latest evidence trace release is available; issue lists and aged
  out traces cannot show affected releases.

Option B: persist `service_version` on `ErrorEventRow` at derivation.

- Cost: model, storage, Greptime table, metadata, and API changes; possible
  migration/default handling for existing events.
- Benefit: durable issue-release linkage even after trace TTL, and later
  `Issue.affectedReleases` can be computed without trace joins.
- Hot path: `derive_from_traces` already receives each `ResourceSpans` and can
  lookup `service.version` beside `service.name` without cloning telemetry. This
  respects zero-copy if implemented as a borrowed attr lookup.
- Decision: defer until the product needs durable affected-release lists or
  regression lifecycle automation. It is the structural follow-up, not needed
  for this plan's release strip and latest-release badge.

Option C: Turso `release_first_seen` rollup.

- Cost: new metadata table plus write path during ingest or worker derivation.
- Benefit: fast release catalog independent of Greptime scans.
- Limit: it still does not attach individual error events to releases; it solves
  release discovery, not issue linkage. Defer until deploy webhook ingest lands.

## Regression Semantics

Full "regressed" means: issue was resolved while latest known release was X,
then a new error event appears in a later release Y for the same service and
fingerprint.

With option A only, Parallax can compute:

- latest evidence release for an issue, from its `last_trace_id`;
- service release windows over a time range;
- whether latest issue activity falls inside a release window.

It cannot prove "reappeared after resolution in a newer release" because issue
metadata has status but no status-history timestamp or resolved release. The UI
therefore ships a neutral `release <version>` badge, not a regression verdict.
Regression automation needs durable event release attribution plus resolution
history.

## UI Shape

Service detail gets a slim release strip below the header and above the
summary cards. Each version segment spans `(last_seen - first_seen)` within the
selected time window. Segment label is the version; tooltip/title includes
first seen, last seen, and span count. Empty release data renders nothing.

Issue detail gets a compact `release <version>` badge in the existing header
badge row when the correlated latest trace still exists and has
`service.version` in its resource JSON. Missing trace or missing attr renders
no badge.
