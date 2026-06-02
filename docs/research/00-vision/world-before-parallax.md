# World Before Parallax

<!-- markdownlint-disable MD013 -->

Research date: 2026-06-03

## Current World

Before Parallax, self-hosted production debugging normally means a stack, not one system:

| Need | Common choice | Why |
| --- | --- | --- |
| Errors, grouped issues, releases, tags, stack traces | Sentry | De facto best product for application errors and issue workflow. |
| Traces and spans | Tempo, Jaeger, Zipkin, or similar | Dedicated trace storage/retrieval. |
| Logs | Elasticsearch/Kibana, Loki/Grafana, OpenSearch, or similar | Dedicated log object/search store. |
| Metrics | Prometheus | De facto metrics and PromQL standard. |
| Cross-signal UI | Grafana, Kibana, Sentry UI, tool-specific UIs | Users need one place to inspect/query, but each backend has its own model. |
| Routing | OpenTelemetry Collector / Grafana Alloy | Fan out signals to each backend. |

This stack works. It is also operationally heavy: multiple storage engines, multiple query models,
multiple UIs, multiple upgrade paths, and multiple teams/projects behind them.

## Why Sentry Still Matters

Sentry is not the thing to dismiss. It is the strongest part of the current world for:

- grouped errors and issue identity;
- stack traces and source context;
- releases, tags, environments, and fingerprints;
- breadcrumbs and nearby execution context;
- developer workflow around "what broke and where."

For Rust projects, `tracing` plus Sentry's Rust `tracing` integration can send errors, breadcrumbs,
structured logs, spans, and application metrics into Sentry. That makes Sentry a good single product
for application debugging.

But Sentry is not the same as a full telemetry warehouse. If the question becomes "show all logs,
metrics, traces, and system behavior across time with flexible exploration and long retention,"
Sentry alone is not the natural backend. That is why teams add Tempo/Jaeger, Loki/Elasticsearch, and
Prometheus.

## Why Logs Often Want Elasticsearch/Kibana

Loki is a good log backend, especially when logs are treated as streams with labels and Grafana is the
main UI. It is efficient and integrates well with Tempo and Prometheus.

Elasticsearch/Kibana gives a different experience: logs are JSON-like documents/objects. Each event can
be searched, filtered, expanded, and displayed by fields. Kibana Discover is natural for object-style
log exploration: open one log record, inspect fields, filter by any field, search text, and build views.

That distinction matters for Parallax:

- Loki-style logs are cheap and stream-shaped.
- Elasticsearch/Kibana-style logs are object/search-shaped and often better for human log exploration.
- Parallax should learn from the Kibana object-inspection experience, not only the Grafana log-stream
  experience.

## Why Prometheus Still Matters

Prometheus remains the default mental model for metrics:

- time series;
- labels;
- PromQL;
- alerting;
- broad ecosystem support.

Any replacement or simplification has to respect that model. If Parallax stores metric evidence in
GreptimeDB, it still needs PromQL-compatible thinking because operators already understand metrics that
way.

## Why Grafana Exists In The Stack

Grafana is the glue UI: query and visualize metrics, logs, traces, and profiles across data sources.
Tempo, Loki, and Prometheus make most sense together because Grafana can link between them.

But Grafana does not replace Sentry's error-grouping workflow. A team still usually needs both:

```text
Sentry: grouped issue -> stack/release/error workflow
Grafana stack: metrics/logs/traces exploration
```

That split is the pain. Debugging crosses both worlds.

## Where Parallax Starts

Parallax starts from this observation:

> Teams do not want five systems to answer one failure question.

The first Parallax idea is a smaller, Rust-first, self-hosted evidence engine:

- CLI-first first, UI later;
- easy to run on one self-hosted server;
- low resource use;
- one coherent product model;
- Sentry-style error grouping;
- OpenTelemetry-native traces/logs/metrics;
- evidence bundles for humans and agents;
- optional scale components only when needed.

## Why GreptimeDB Fits The First Storage Bet

GreptimeDB is attractive because it can collapse much of the Tempo/Loki/Prometheus storage problem into
one Rust observability database:

- metrics, logs, and traces in one engine;
- OpenTelemetry-oriented ingest;
- SQL and PromQL;
- object-storage / cloud-native direction;
- Rust implementation.

So V1 can focus on:

```text
Parallax Rust app
  -> GreptimeDB for high-volume evidence
  -> metadata DB for grouping/project/state
  -> evidence bundle API / CLI / UI
```

This is not "GreptimeDB forever." It is "GreptimeDB first because it gives many needed capabilities
out of the box."

## Modular Growth Path

Parallax should start tiny and add pieces only when scale requires them:

| Tier | Storage / processing | Use case |
| --- | --- | --- |
| Local | embedded/local storage, future Turso/SQLite-like profile | CLI, demos, personal debugging, tests. |
| Single server | GreptimeDB + metadata DB | first serious self-hosted setup. |
| Durable server | GreptimeDB object storage + Postgres + workers | retained production evidence. |
| Scale-out | GreptimeDB distributed + Postgres + Apache Iggy or another stream | parallel processing, replay, backpressure. |

Grouping, issue state, projects, users, policies, and audit rows need relational state. Postgres is the
safe production answer; Turso/SQLite-like storage is plausible for local mode because it is Rust and
SQLite-compatible, but it remains future-gated.

Apache Iggy or another stream should not be mandatory at the start. It appears when Parallax needs
replay, parallel processors, and backpressure isolation.

## Product Vision

Parallax should feel like one self-contained system instead of a zoo:

- one install path;
- one CLI;
- one evidence API;
- one UI;
- one bundle model;
- Rust-first internals where possible;
- scale knobs added by enabling components, not by redesigning the product.

The UI should preserve the best ideas from existing tools:

- Sentry's grouped issue workflow;
- Kibana's object-style log inspection;
- Grafana's cross-signal linking;
- Prometheus-style metric queries;
- Tempo-style trace waterfall.

But the product center should be different:

> not dashboards first, evidence bundle first.

That is the core differentiation.

## Source Anchors

- [Sentry Rust tracing docs](https://docs.sentry.io/platforms/rust/guides/tracing/)
- [Sentry Rust logs docs](https://docs.sentry.io/platforms/rust/common/logs/)
- [Sentry Rust metrics docs](https://docs.sentry.io/platforms/rust/common/metrics/)
- [Grafana Tempo docs](https://grafana.com/docs/tempo/)
- [Grafana Loki docs](https://grafana.com/docs/loki/)
- [Prometheus OpenTelemetry guide](https://prometheus.io/docs/guides/opentelemetry/)
- [Elasticsearch documents and indices](https://www.elastic.co/guide/en/elasticsearch/reference/current/documents-indices.html)
- [Elasticsearch search overview](https://www.elastic.co/guide/en/elasticsearch/reference/current/search-your-data.html)
