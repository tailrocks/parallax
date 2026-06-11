# Conventions: what to send so Parallax can correlate

Parallax derives everything from standard OTLP — these conventions are what
make the derived views sharp instead of mushy.

## Resource attributes

| Attribute | Required | Why |
| --- | --- | --- |
| `service.name` | yes | Anchor of every per-service view, trace edge, issue scope. |
| `service.version` | recommended | Release linkage; "did the fix ship" checks. |
| `deployment.environment.name` | recommended | Keeps prod and staging issues apart. |
| `vcs.ref.head.revision` | recommended | The deployed commit — stamp at build time. |
| `vcs.repository.url.full` | recommended | Which repo a fixer gets pointed at. |
| `parallax.run_id` | injected | Set by `parallax run start`; never set it by hand. |

`parallax.run_id` is promoted to a real column on spans and logs at ingest, so
run-scoped queries are exact and fast.

## Exception encodings (both accepted, indefinitely)

OTel deprecated span events in favor of log-based events (2026-03-17); fleets
will straddle that transition for years, so Parallax derives errors from
**all** of:

1. Span events named `exception` (`exception.type`, `exception.message`,
   `exception.stacktrace`).
2. Span status `ERROR` without an exception event.
3. Log records at severity ERROR/FATAL.
4. Log records carrying `exception.*` attributes.

Grouping fingerprints normalize volatile tokens (numbers, hex ids, uuids), so
`timeout after 2000ms` and `timeout after 3500ms` are one issue.

## Database wrapper spans

`tokio-postgres` and the `clickhouse` crate get thin wrapper spans (patterns in
[rust-stack-instrumentation.md](../research/capture/rust-stack-instrumentation.md)):

| Attribute | Example |
| --- | --- |
| `db.system.name` | `postgresql`, `clickhouse` |
| `db.operation.name` | `SELECT` |
| `db.query.text` | `SELECT id, total FROM orders WHERE cart_id = $1` |

Use placeholders, never inline values — `db.query.text` reaches bundles and
agents. Span duration is the query duration; that is how slow-query hypotheses
get their numbers.

## Trace ids belong in front of users

Error responses, error pages, and TUI error output should surface the active
trace id (body, header, or a "copy error reference" control). One pasted trace
id later, `parallax trace inspect` reconstructs the whole workflow — that is
the complaint loop.

## Metrics

Any gauge/sum/histogram you send is immediately chartable: it appears in
`metricNames`, can be aggregated (`avg/min/max/sum/rate`, histogram
quantiles), and can be pinned to a custom dashboard in the UI. Name metrics
`domain.thing.unit` (`checkout.payment.duration_ms`) and keep label cardinality
small — every label combination is a stored series.
