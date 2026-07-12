# Historical active-plan live evidence

Timestamp: 2026-07-10T07:40:00Z

Repos:

- Parallax: `df81d86e6f0cbaaae9eb34402ea64e1a08dbd756` plus this change set.
- Playground: `830d2c9179dfc5dadd00bc3e2d4d10bcb6f7a9d4` plus this change set.

Environment:

- Parallax `serve`: `http://127.0.0.1:4000`, OTLP gRPC `127.0.0.1:4317`, OTLP HTTP `127.0.0.1:4318`, managed GreptimeDB `127.0.0.1:24000`.
- Playground compose: `deploy/docker-compose.yml` plus `deploy/docker-compose.xlang.yml`.
- All evidence uses GreptimeDB native observability tables: `opentelemetry_traces`, `opentelemetry_logs`, and native metric tables such as `tokio_runtime_blocking_pool_depth` and `messaging_queue_depth`.

## Scenario Run Evidence

The live playground run covered the remaining scenario-backed plan items:

- A1 checkout smoke.
- A7b gRPC stream.
- A13 deploy regression with checkout `v1` and `v2`.
- A14 flag flip.
- A19 long trace.
- A20 batch fan-in.
- A20 compare pair.
- A22 Tokio saturation.
- A25 Postgres reality.
- A28 frontend RUM journey.
- A29 typed events.
- B3b gRPC deadline.
- B17b cron suite.
- B19 JVM GC pressure.
- B20 recommendation container OOM.
- B21 orphan consumer.
- B22 sampling gap.
- B23 uncorrelated log.
- `b-async-chaos`, `b-chaos`, and `b-degradation`.

Compose proof after the run:

```text
telemetry-playground-catalog-1          Up About an hour
telemetry-playground-checkout-1         Up 44 minutes
telemetry-playground-inventory-1        Up About an hour
telemetry-playground-orders-1           Up About an hour
telemetry-playground-payment-1          Up About an hour
telemetry-playground-postgres-1         Up About an hour (healthy)
telemetry-playground-pricing-1          Up About an hour
telemetry-playground-recommendation-1   Up 43 minutes
telemetry-playground-web-1              Up About an hour
```

B20 was rerun on 2026-07-10 after OrbStack recovered:

```text
leak round 32: 8192KiB
curl: (52) Empty reply from server
[000]
telemetry-playground-recommendation-1   Restarting (137) Less than a second ago
```

Final Docker state after restart:

```text
RestartCount=1 OOMKilled=false ExitCode=0 Status=running
```

`OOMKilled=false` is the current restarted process state. The scenario proof is the observed exit `137`, transient `Restarting (137)`, empty HTTP reply, and restart count.

## Native GreptimeDB Snapshots

Overview and catalog:

```graphql
overview(1783665000000000000..1783665800000000000)
```

```text
spanCount=1197 traceCount=317 logCount=1096 errorCount=45 activeServices=8
```

Service catalog included Java and Rust services with SDK identity:

```text
catalog java opentelemetry 1.63.0 playground instances=1
checkout rust opentelemetry 0.32.1 playground instances=8
inventory rust opentelemetry 0.32.1 playground instances=1
orders rust opentelemetry 0.32.1 playground instances=1
payment java opentelemetry 1.63.0 playground instances=1
pricing rust opentelemetry 0.32.1 playground instances=1
recommendation rust opentelemetry 0.32.1 playground instances=2
```

Checkout release windows:

```text
v1 spanCount=644 first=1783665030485708493 last=1783665518258366008
v2 spanCount=20  first=1783665156282126153 last=1783665156394108498
```

Native trace table release proof:

```sql
SELECT service_name, "resource_attributes.service.version", COUNT(*) AS spans
FROM opentelemetry_traces
WHERE service_name = 'checkout'
GROUP BY service_name, "resource_attributes.service.version"
ORDER BY "resource_attributes.service.version";
```

```text
checkout 0.1.0 1960
checkout v1    2090
checkout v2    54
```

Native metric table proof:

```sql
SELECT service_name, MAX(greptime_value)
FROM tokio_runtime_blocking_pool_depth
GROUP BY service_name;
```

```text
checkout 768.0
```

```sql
SELECT service_name, MAX(greptime_value)
FROM messaging_queue_depth
GROUP BY service_name;
```

```text
orders 4.0
```

Postgres span proof from `opentelemetry_traces`:

```text
postgres.query inventory postgresql UPDATE 150
postgres.query inventory postgresql SELECT 26
postgres.pool  inventory postgresql ACQUIRE 6
query orders   checkout  postgresql SELECT 6
```

Typed log event proof from `opentelemetry_logs`:

```text
checkout.completed 130
checkout.failed     19
payment.authorized  76
```

## Trace And Log Resolver Proof

Batch fan-in trace:

```text
traceId=f7d79d393ef8c24ff7484bcf4ef0ce6e
span=consume_batch
typedLinks=8
linkedTraces=8 publish traces from orders
```

gRPC stream trace events:

```text
traceId=e313ec59afc7fb4f0cee8178a1014836
namePrefix=rpc.message
events=12
skippedSpans=0
truncated=false
services=pricing, checkout
```

Trace compare:

```text
left=7d3cd5d878f37e167235070582bddb4e
right=dcbb2e369aa2e531423719d7a8798615
criticalPath.totalGatedNs=114925042
compare added reserve/postgres.query siblings
compare removed recommendation spans
compare changed checkout/resolve_bool_value/reserve/pricing spans
```

Logs context:

```text
logsAround(anchor=1783665355675684288, service=checkout)
slow render observed
checkout compare variant
checkout.completed
checkout ok
```

Field explorer:

```text
fieldStats(resource.service.name, service=checkout)
rowCount=664 nonNullCount=664 coverage=1.0 topValue=checkout:664
```

Invalid field keys are rejected safely by the allowlist; `authorization` returned `invalid field key`.

## Browser Proof

Playwright final sweep against the served UI:

```text
service detail: ok
clock skew: ok
investigation restore: ok
```

Checked routes:

- `/services/checkout?range=custom&from="1783665000000000000"&to="1783665800000000000"` rendered checkout detail, releases `v1`/`v2`, runtime metrics, and `tokio.runtime`.
- `/traces/3248c7e686b957e37c4170343d1b8171?view=lanes&range=custom&from="1783665000000000000"&to="1783665800000000000"` rendered `Clock skew suspected` and included `skewed-op`.
- `/investigations/case_plan_052_live?...` restored `Plan 052 Live Proof`, pins, and exact notes value `Plan 052 save restore proof`.

Saved view lifecycle:

```graphql
mutation { savedViewDelete(id: "view_plan_057_logs") }
```

```text
savedViewDelete=true
savedViews(page:"/logs")=[]
```

## Fixes Landed Before Retirement

- GreptimeDB native `span_links` can use `trace_id`/`span_id` as stored by `opentelemetry_traces`; GraphQL `typedLinks` and `linkedTraces` resolve batch fan-in links.
- GreptimeDB native `span_events` can use `time` strings such as `2026-07-10 06:31:40.754723148+0000`; GraphQL `traceEvents` parses gRPC stream events.
- `traces_by_ids` reads native `opentelemetry_traces` rows and summarizes in Rust, avoiding fragile SQL joins while preserving caller order.
- `/services/$service` now renders through the parent route `Outlet`.
- Trace skew detection keeps normal same-service drift ignored, but flags extreme same-service or rootless backdated spans.

## Plan Retirement Map

- 036: trace-spine smoke and dependency note satisfied by compose run, trace IDs, and native trace-table proof.
- 040: large waterfall/log proof satisfied by large trace browser route, logs route proof, route tests, and build.
- 041: releases/deploy lane satisfied by checkout `v1`/`v2` release GraphQL, native trace table proof, service/issue browser proof.
- 042: release/env/flag/catalog proof satisfied by A13/A14 and catalog/release snapshots.
- 043: service catalog satisfied by catalog GraphQL and `/services/checkout` browser route.
- 044: runtime dashboards and metric discovery satisfied by native metric tables, metricNames, and runtime route proof.
- 045: runtime scenarios satisfied by A22, B19, B20, native runtime/JVM metric evidence, and Docker restart proof.
- 046: field explorer satisfied by `fieldKeys`, `fieldStats`, invalid-key rejection, and traces field drawer proof.
- 048: Postgres reality satisfied by A25 and native `db.*` span proof.
- 049: messaging/gRPC semantics satisfied by typed links, linked traces, queue-depth metric, and `rpc.message` events.
- 050: frontend RUM journey satisfied by A28 browser journey and native route/vital telemetry from the live run.
- 051: critical path and compare satisfied by real trace IDs, `traceCriticalPath`, and `traceCompare` proof with sibling ordinals.
- 052: investigations satisfied by save/restore browser proof and invalid-state GraphQL rejection.
- 053: design/a11y sweep satisfied by final browser sweep over dense routes plus UI tests/build.
- 054: quality scenarios and tour satisfied by B17b/B21/B22/B23/chaos/degradation scenario evidence.
- 056: typed events/logs satisfied by native `opentelemetry_logs` event-name counts.
- 057: logs context and saved views satisfied by `logsAround`, route tests, save/list/delete proof.
- 061: trace view modes satisfied by errors mode, service lanes, minimap, deep links, and skew route proof.
- 063: trace-shape scenarios satisfied by long trace, compare pair, and backdated skew trace proof.
- 064: command center satisfied by overview counts, dashboard brush route tests, and linked drilldown proof.

Historical result at the timestamp above: every then-active plan file had
removal evidence. This packet is completion evidence, not a current plan index;
current unfinished work is authoritative only in `plans/`.
