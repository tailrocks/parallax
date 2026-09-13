# Investigation-surface primary-source restamp — 2026-09-13

Cited competitor UX for grouping, log neighborhood, waterfall/critical path,
metric spike → exact trace, and cross-signal hops. Not a bake-off; no
time-to-understand measurements. Status: **partial** (docs, not live SaaS
tenants). Does not grow the freeze P0 list.

## Summary

Sentry is the only 2026 surface among these products whose issue object
explains grouping, merge/unmerge, and regression/escalation as first-class
statuses. Datadog is the fastest documented log-neighborhood path: it rewrites
the query to adjacent lines and can assemble a full request from a shared ID
even when those lines miss the original filter. Grafana Tempo is the only
waterfall with a one-click critical-path isolate; Honeycomb has the tightest
in-trace search/error/minigraph loop. Stored spike-to-exact-trace IDs live in
Grafana exemplars and Sentry metric samples; ClickStack is the clearest
cross-signal hop because logs, traces, metrics, and sessions share one search
UI without stitching timestamps or correlation IDs.

## Error grouping and regression

Sentry’s Issue Details page is the only current surface that shows, per event,
which fingerprint inputs (including stack-trace contribution) produced the
group, plus Similar/Merged Issues controls and an activity feed of assignments,
regressions, and escalations. [S1] Grouping is a versioned cascade—custom
fingerprint, then stack trace, exception, then message—with an AI embedding
fallback that only merges new unmatched hashes; those merges are visible and
unmerge blocks regrouping. [S2] Resolve-in-release reopens as Regressed;
archived issues become Escalating when volume exceeds a per-issue forecast from
the previous week, and both statuses have dedicated Issues tabs. [S3]

Datadog groups by service, error type, message, and the topmost meaningful
stack frame (a custom `error.fingerprint` is used as-is and is service-scoped).
A RESOLVED issue that recurs on a newer version (or with no versions) moves to
FOR REVIEW with a Regression tag rather than duplicating the issue. [S4]
Grafana Cloud Frontend Observability only gained four-layer cascading
fingerprints on 2026-04-21—first-seen dates reset because hashes were new—and
its docs still describe inspection and filtering, not resolve/regress/escalate
states. [S5] SigNoz, Honeycomb, and HyperDX/ClickStack cluster by message/type
or Drain3-style patterns; they do not document a persistent fingerprint-debug
issue object with resolve-in-release regression or volume escalation. [S6]

## Log surrounding context

Datadog’s View in context rewrites the explorer query to the lines immediately
before and after the selected event—even when those lines fail the current
filter—using host, service, file, and container identity. [S9] Transactions
then assemble a full request or user session from a shared high-cardinality id
(requestId, orderId), including logs that never matched the original query, so
the surrounding request is one click rather than a rebuilt search. [S10]

Grafana Explore / Logs Drilldown Show context is grep -C on the same stream,
then widen by dropping label filters, stretching the time window, or opening
the context query in split view; Live tail streams new lines in Explore. [S11]
Honeycomb’s path is Query Builder plus Explore Events: any attribute can be
filtered or grouped regardless of cardinality, a graph click opens a
timestamp-ordered event list with Δ Time between adjacent lines, and a present
`trace.trace_id` jumps to the waterfall. [S12]

## Trace waterfall and critical path

Among current official waterfall docs, Grafana Tempo is the only product with a
first-class Critical path control: a pill that highlights the longest sequence
of dependent tasks that sets the trace’s minimum duration, and turning Show all
spans off hides everything else. [S13] Honeycomb’s waterfall is the tightest
in-trace loop—span search with match count and arrows, error-count hops, a
peer-outlier heatmap minigraph, and a condensed six-level summary so long or
error spans are found without scrolling the full tree—but it does not document
critical-path highlighting. [S14]

SigNoz synchronizes flame graph and waterfall (100k+ / 10k+ spans), Highlight
Errors hops that dim non-errors, in-trace query/filter pills, and a
span-percentile comparison to similar spans over the past hour—not a
critical-path overlay. [S15] Datadog’s APM Trace View has structured span
search, an Error checkbox that intersects matches, prev/next hopping, and
focus-rescale; documented critical path is CI Visibility, not APM, and outlier
analysis sits on Trace Explorer rather than an in-waterfall minigraph. [S16]
HyperDX/ClickStack’s documented slow-trace path is Event Deltas on a duration
heatmap (box outliers, auto-compare attributes), not waterfall isolate; the
waterfall is a scrollable service tree where a bottom error propagates up the
call chain. [S17] Sentry’s Trace View is a combined errors/logs/profiles
waterfall with Previous/Next traces beside the search bar; official docs do not
specify critical-path isolate, error-count hops, or a peer minigraph. [S18]

Parallax already ships `traceCriticalPath` (keep). Best *documented* isolate
control is Tempo’s pill, not Datadog APM.

## Metric spike to exact trace

Grafana’s hop is a stored exemplar: click the diamond/star on a Prometheus
time-series panel and the linked Tempo trace is one click away. Trace-to-logs
is a separate configured “Logs for this span” link (or copy service/time into
Loki); missing tag mapping hides the link with no error, and tail sampling can
404 the referenced trace. [S20] Sentry stores `trace_id` and `span_id` on each
metric event; the documented flow is Aggregates spike → Samples tab → click a
sample into the waterfall with related spans, logs, and errors. Metrics are
typically 100% sampled while traces are not, so some samples have no attached
trace. [S19]

Honeycomb’s heatmap cell opens an arbitrary matching span for that time and
value—not a stored metric-exemplar Trace ID—and metrics live in a separate
time-series dataset correlated by time window and shared filter fields. [S21]
SigNoz dashboard hops preserve filters, time range, and group-by into the
contributing record set; a specific trace opens only when the clicked datapoint
carries a `trace_id`. [S22] Datadog’s resource page opens a traces list already
filtered by environment, service, operation, and resource—not an exemplar ID on
the metric point. OpenTelemetry trace↔metrics correlation is
`host.name`/`container.id` on the Infrastructure tab, and logs may carry a
`trace_id` for a trace that was not retained. [S23] ClickStack Event Deltas
(trace heatmaps only) let you drag-select a latency region and open matching
traces with filters carried over; the demo path is log → Trace tab and trace →
Infrastructure metrics, not a click on a metric exemplar. [S24]

Parallax ships `metricExemplars` plus **Traces around peak** (time-window
pivot). Stored exemplar diamonds remain Grafana’s stronger chart-native hop.

## Cross-signal navigation

ClickStack documents a single HyperDX search UI that correlates logs, traces,
metrics, and sessions without switching tools or stitching
timestamps/correlation IDs, with Lucene-style property search, live tail, and
dashboards over high-cardinality events. [S7] Same-session/same-request context
is assembled by clicking a session network request or error into a Trace tab
(associated logs, spans, database queries) and the reverse path from a
Search-view trace to a Session replay tab. [S8]

## Sources

- [S1] [Issue Details](https://docs.sentry.io/product/issues/issue-details/)
- [S2] [Issue Grouping](https://docs.sentry.io/concepts/data-management/event-grouping/)
- [S3] [Issue Status](https://docs.sentry.io/product/issues/states-triage/)
- [S4] [Error Grouping](https://docs.datadoghq.com/error_tracking/error_grouping/)
- [S5] [Error Fingerprinting in Frontend Observability](https://grafana.com/whats-new/2026-04-21-error-fingerprinting-in-frontend-observability/)
- [S6] [Errors and Exceptions](https://signoz.io/docs/userguide/exceptions/)
- [S7] [ClickStack](https://clickhouse.com/docs/clickstack/overview)
- [S8] [Session replay](https://clickhouse.com/docs/clickstack/features/session-replay)
- [S9] [Log Side Panel](https://docs.datadoghq.com/logs/explorer/side_panel/)
- [S10] [Grouping Logs Into Transactions](https://docs.datadoghq.com/logs/explorer/analytics/transactions/)
- [S11] [Logs in Explore](https://grafana.com/docs/grafana/next/visualizations/explore/logs-integration/)
- [S12] [Explore Events](https://docs.honeycomb.io/investigate/analyze/explore-events)
- [S13] [Span filters (Tempo)](https://grafana.com/docs/grafana/next/datasources/tempo/span-filters/)
- [S14] [Explore Traces](https://docs.honeycomb.io/investigate/analyze/explore-traces)
- [S15] [Trace Details](https://signoz.io/docs/userguide/span-details/)
- [S16] [Trace View](https://docs.datadoghq.com/tracing/trace_explorer/trace_view/?tab=waterfall)
- [S17] [Remote demo dataset](https://clickhouse.com/docs/clickstack/example-datasets/remote-demo-data)
- [S18] [Sentry Trace View](https://docs.sentry.io/concepts/key-terms/tracing/trace-view/)
- [S19] [Application Metrics](https://docs.sentry.io/product/metrics/)
- [S20] [Navigate between signals](https://grafana.com/docs/grafana-cloud/learn-and-build/telemetry-signals/use-signals-together/navigation-between-signals/)
- [S21] [Visualize Events Over Time](https://docs.honeycomb.io/investigate/analyze/visualize-events)
- [S22] [Interactivity in dashboards](https://signoz.io/docs/dashboards/interactivity/)
- [S23] [Resource Page](https://docs.datadoghq.com/tracing/services/resource_page/)
- [S24] [Event deltas with ClickStack](https://clickhouse.com/docs/clickstack/features/event-deltas)

## Coverage and uncertainty

No inspected 2026 source measures developer time-to-understand across these
products. Rankings above are from documented surfaces, not a timed bake-off.
Datadog issue-panel grouping rationale may exist in UI beyond Explorer docs.
Honeycomb Errors is documented as a Frontend Launchpad Enterprise add-on; no
inspected Honeycomb page defines backend exception fingerprinting. SigNoz Logs
Explorer (updated 2026-08-24) documents hover Show in Context (±10, load-more)
and was not independently timed. Sentry logs are trace-connected by default;
surrounding-lines independent of the trace is not documented the way
Grafana/SigNoz/Datadog document it. Honeycomb waterfall caps the viewer at
32,000 spans. Sentry metrics samples can lack a trace because metrics are
typically 100% sampled and traces are not.
