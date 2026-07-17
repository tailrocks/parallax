# Plan 164 — live-engine GraphQL assertions (helper agent, 2026-07-17)

Read-only assertions executed with `curl` against the running QA stack
(`parallax serve` at `127.0.0.1:4000`, managed GreptimeDB on 24000-24003,
playground corpus loaded) at repo head `7302672`. Every count below is a
real GreptimeDB answer, not a memory-adapter result.

## Traces

| Assertion | Query | Result |
|---|---|---|
| Structured filter narrows | `tracesPage(attributeFilters: [{key: "http.request.method", op: "=", value: "POST"}], limit: 1) { total }` | `total: "45"` |
| SQL ground truth matches exactly | `sql("SELECT COUNT(DISTINCT \"trace_id\") FROM opentelemetry_traces WHERE CAST(\"span_attributes.http.request.method\" AS STRING) = 'POST'")` | `[45]` |
| Facet count agrees with page total | `traceFacets` → `http.request.method` | `GET 97, POST 45, OPTIONS 5` |
| Injection-shaped value stays one literal | `tracesPage(attributeFilters: [{key: "http.request.method", op: "=", value: "x' OR 1=1--"}])` | `total: "0"`, no error |
| Duration stats answer over the filtered set | `traceDurationStats(attributeFilters: […POST…]) { p50Ms p95Ms }` | `p50Ms: 2.853233, p95Ms: 15016.652902` |
| All four bounded dimensions answer | `traceFacets { dimension }` | service (20 values), status (`UNSET 5216, ERROR 635`), http.request.method, error.type (`shapes::RecurringFailure 400, poison_message 15, …`) |

## Logs

| Assertion | Query | Result |
|---|---|---|
| Numeric ordering filter narrows | `logs(attributeFilters: [{key: "severity_number", op: ">=", value: "17"}], limit: 500) { severityNum }` | 500 rows (limit cap), min severity 17 |
| Histogram tracks the same filter | `logCountSeries(…same filter…, stepSeconds: 86400)` | buckets `1 + 8233 = 8234` |
| Facets agree with the filtered series | `logFacets` → severity | `ERROR 4234 + FATAL 4000 = 8234` — exact match with the series total across three independent code paths (facet GROUP BY, filtered series, filter compiler) |
| Injection-shaped CONTAINS | `logs(attributeFilters: [{key: "body", op: "CONTAINS", value: "x' OR 1=1--"}])` | `[]`, no error |
| Empty dimensions stay empty, not wrong | `logFacets` → http.request.method / error.type | `[]` (corpus logs carry no such attributes) |

## Verdict

Live-engine narrowing, facet counting, duration stats, histogram/filter
consistency, and both injection proofs hold on the real engine. Remaining
for the primary executor: the `f-attrs` 70/20/10 scenario cross-check (the
scenario's spans were not in the loaded corpus window at assertion time),
executing the ignore-gated `m9_attribute_filters_greptime` suite once the
QA stack frees ports 24000-24003, the facet-window cap decision, and the
browser-walk evidence for the wired routes.
