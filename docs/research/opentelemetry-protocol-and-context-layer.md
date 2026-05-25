# OpenTelemetry Protocol and Context Layer

<!-- markdownlint-disable MD013 -->

Research date: 2026-05-25

## Purpose

This note answers the OpenTelemetry section of the Parallax research prompt:
architecture, OTLP internals, semantic conventions, collector pipelines,
Rust-native collection, production deployment patterns, and the question of what
Parallax should build above OTEL.

Version freshness rule: this recommendation is based on current public docs and
source material checked on 2026-05-25. Every future benchmark or comparison must
use the latest reasonably available stable/public version of each candidate as
of the benchmark date, and must label older benchmark posts or architecture docs
as historical evidence.

## Short Recommendation

Treat OpenTelemetry as Parallax's native telemetry protocol layer.

Parallax should accept OTLP HTTP/gRPC directly, preserve OpenTelemetry resource
and trace context, and interoperate cleanly with upstream OpenTelemetry
Collectors. It should not make the Collector mandatory for the tiny deployment.
The product value belongs above OTEL: Sentry-compatible error grouping,
stacktrace normalization, release regression analysis, evidence graph building,
and schema-bound, canonical context bundles for humans and coding agents across
CLI, HTTP, and MCP projections.

The product-claim boundary for "OTLP-native" and "Collector-compatible" is the
[OTLP conformance ledger](otlp-conformance-ledger.md). The focused
[OTLP transport profile recheck](otlp-transport-profile-recheck.md) sharpens the
baseline transport claim: Parallax should require `grpc` and `http/protobuf`,
label `http/json` as optional, and test SDK endpoint URL construction before
using broad OTLP wording.
Stored OTLP rows are necessary but not sufficient for Parallax's agent claim:
OTLP-derived evidence becomes agent-ready only after the
[evidence bundle schema](evidence-bundle-and-schema.md), canonical hashes,
projection manifests, redaction reports, and access-surface equivalence gates
pass.

In one sentence:

> OTEL should be the wire format and context substrate; Parallax should be the
> investigation, grouping, and agent-context layer above it.

Current competitor pressure strengthens this boundary. The focused
[Traceway OTLP/AI/replay recheck](traceway-otlp-ai-replay-recheck.md) found an
active MIT project with direct OTLP/HTTP traces, metrics, and logs; OTel
exception-to-issue conversion; trace-linked logs; AI trace promotion from
`gen_ai.*`; native session-replay protocol; and SQLite/all-in-one/embedded
deployment modes. Treat "OTLP-native and no Collector required" as table stakes,
not as the Parallax moat. The moat must be Sentry migration plus canonical,
redacted, citable evidence bundles and action/outcome audit.

## Current Version Snapshot

| Component | Latest checked version | Release date | Notes |
| --- | --- | --- | --- |
| OpenTelemetry spec docs | OTel `1.57.0`, OTLP `1.10.0`, Semantic Conventions `1.41.0` | Checked 2026-05-25 | OTLP is stable for traces, metrics, and logs; profiles remain development-stage. |
| OpenTelemetry Collector core/source | `v0.153.0` | 2026-05-25 | Latest core/source release checked from GitHub release metadata; it has moved ahead of the runnable distribution. |
| OpenTelemetry Collector distribution | `v0.152.1` | 2026-05-20 | Latest official distribution/binary release checked from GitHub release metadata. Treat this separately from core/source for conformance. |
| OpenTelemetry Collector Contrib | `v0.152.0` | 2026-05-11 | Contrib carries its own distribution/version line with core Collector plus a wider component set; track separately from latest core/source. |
| OpenTelemetry Rust | `opentelemetry-0.32.0` | 2026-05-09 | Rust traces, metrics, and logs are all listed as beta by the official Rust docs. |
| Rotel | `v0.2.2` | 2026-05-04 | Rust-native OTEL collector alternative; promising, but early. |
| MCP specification | `2025-11-25` | Checked 2026-05-25 | MCP tool outputs can carry schema-validated `structuredContent`; text/Markdown should remain projections of canonical bundle JSON. |
| JSON canonicalization | RFC 8785 JCS | June 2020 | Use deterministic JSON hashes for bundle/projection equality instead of renderer-specific output. |

Sources:

- [OpenTelemetry overview](https://opentelemetry.io/docs/specs/otel/overview/)
- [OTLP specification](https://opentelemetry.io/docs/specs/otlp/)
- [OpenTelemetry Protocol Exporter spec](https://opentelemetry.io/docs/specs/otel/protocol/exporter/)
- [OpenTelemetry resource semantic conventions](https://opentelemetry.io/docs/specs/semconv/resource/)
- [OpenTelemetry Rust docs](https://opentelemetry.io/docs/languages/rust/)
- [OpenTelemetry Collector core v0.153.0 release](https://github.com/open-telemetry/opentelemetry-collector/releases/tag/v0.153.0)
- [OpenTelemetry Collector distribution v0.152.1 release](https://github.com/open-telemetry/opentelemetry-collector-releases/releases/tag/v0.152.1)
- [OpenTelemetry Collector Contrib v0.152.0 release](https://github.com/open-telemetry/opentelemetry-collector-contrib/releases/tag/v0.152.0)
- [Rotel docs](https://rotel.dev/)
- [Rotel v0.2.2 release](https://github.com/rotel-dev/rotel/releases/tag/v0.2.2)
- [OpenTelemetry Rust 0.32.0 release](https://github.com/open-telemetry/opentelemetry-rust/releases/tag/opentelemetry-0.32.0)
- [MCP tools specification 2025-11-25](https://modelcontextprotocol.io/specification/2025-11-25/server/tools)
- [RFC 8785 JSON Canonicalization Scheme](https://www.rfc-editor.org/rfc/rfc8785.html)
- [Evidence bundle and open schema](evidence-bundle-and-schema.md)
- [OTLP conformance ledger](otlp-conformance-ledger.md)

## Is OTEL Becoming The Universal Protocol Layer?

Yes, for observability transport and instrumentation.

OpenTelemetry's architecture defines independent signals for traces, metrics,
logs, baggage, resources, and context propagation. OTLP then standardizes the
encoding, transport, and delivery mechanism between telemetry sources,
collectors, and backends. OTLP is stable for traces, metrics, and logs, and it
supports both gRPC and HTTP transports using protobuf payloads.

That makes OTEL the obvious default for Parallax ingestion:

- SDKs already know how to emit OTLP.
- Collectors already know how to receive, transform, batch, retry, and export
  OTLP.
- Backends increasingly accept OTLP directly.
- `trace_id`, `span_id`, resource attributes, and semantic conventions provide
  the shared keys needed for cross-signal joins.

But OTEL is not the universal investigation layer. It does not define Sentry
issue grouping, release regression semantics, stacktrace fingerprinting,
symbolication policy, product issue workflows, source-code context bundles,
agent-safe evidence boundaries, or root-cause confidence scoring. Those are the
Parallax opportunity.

## What OTEL Gives Parallax

| Capability | OTEL source | Parallax use |
| --- | --- | --- |
| Trace context | `trace_id`, `span_id`, parent/child spans, links, baggage | Join errors, logs, spans, and metrics around one failing request or workflow. |
| Resource identity | `service.name`, `service.version`, deployment, host, process, container, Kubernetes attributes | Attach failures to service, release, runtime, deploy, and infrastructure context. |
| Logs | OTLP log records and non-OTLP trace-context field guidance | Correlate structured logs with traces and error events without custom log format lock-in. |
| Metrics | OTLP metrics data model and Prometheus compatibility paths through collectors/backends | Add saturation, error-rate, latency, and deploy-impact evidence to error bundles. |
| Collector pipelines | receivers, processors, exporters, connectors, service pipelines | Let production users reuse existing collection, sampling, redaction, batching, retry, and fan-out infrastructure. |
| Deployment patterns | agent, gateway, sidecar, DaemonSet, load-balanced collector tiers | Fit from laptop/tiny deployments to Kubernetes and regional collector tiers. |
| Backpressure expectations | OTLP retryable errors, retry-after, exponential backoff, partial-success response model | Make Parallax's OTLP endpoints predictable under overload and invalid payloads. |

## What OTEL Does Not Give Parallax

OTEL data is necessary evidence, not a finished debugging product.

Parallax still needs its own logic for:

- Sentry-compatible envelope ingestion and event normalization.
- Grouping and fingerprinting error events into stable issues.
- Rust stacktrace normalization, symbolication, panic location extraction, and
  build/release enrichment.
- Release regression detection and "first seen after deploy" analysis.
- Evidence graph edges such as `error_observed_in_span`, `log_near_error`,
  `metric_anomaly_near_release`, and `deploy_precedes_regression`.
- Bounded context bundles for coding agents, with redaction and least-privilege
  query scopes.
- Canonical evidence bundles, projection manifests, MCP output schemas, or
  proof that CLI/HTTP/MCP surfaces preserve the same agent-visible context.
- Human issue workflow: assignment, status, notes, suppressions, regressions,
  and audit history.

The strategic answer is to avoid competing with OTEL at the protocol layer and
compete where OTEL intentionally stops.

## Target Ingestion Architecture

Tiny single-node:

```text
App / service
  -> Sentry envelope endpoint
  -> OTLP HTTP/gRPC endpoint
  -> parallax-server
       - auth and DSN validation
       - redaction and size limits
       - OTLP decode and normalization
       - local WAL / outbox
       - grouping and evidence graph writer
  -> GreptimeDB
  -> Turso metadata
```

Production with existing collection:

```text
App / service
  -> OpenTelemetry SDK
  -> OpenTelemetry Collector or Rotel
       - batch
       - memory limits
       - resource enrichment
       - sampling / filtering
       - redaction / transform
       - retries and queues
  -> Parallax OTLP endpoint
  -> Iggy or local WAL
  -> normalizer / groupers / evidence workers
  -> GreptimeDB
  -> Turso metadata
  -> context API / MCP
```

Large deployment:

```text
Apps and collectors
  -> regional load balancer
  -> gateway collector tier
  -> optional trace-aware collector tier for tail sampling
  -> parallax-ingest x N
  -> clustered durable stream (NATS/Redpanda; Iggy is single-node today)
  -> processing workers x N
  -> GreptimeDB distributed or ClickHouse fallback
  -> Turso or Postgres metadata fallback
  -> API / MCP / UI
```

The key design constraint: the Collector is supported, not required. A small
team should be able to point an SDK directly at Parallax. A larger team should
be able to keep its Collector topology and add Parallax as another OTLP
destination.

## OTLP Endpoint Requirements

Parallax's OTLP receiver should implement:

- OTLP/gRPC on `4317`.
- OTLP/HTTP on `4318`.
- HTTP paths `/v1/traces`, `/v1/metrics`, and `/v1/logs`.
- Required transports: `grpc` and `http/protobuf`.
- JSON protobuf as an explicitly labeled optional compatibility path; a
  JSON-only receiver is not enough for Parallax's OTLP-native claim.
- SDK endpoint URL construction fixtures, because generic
  `OTEL_EXPORTER_OTLP_ENDPOINT` and per-signal endpoint variables build paths
  differently.
- Gzip request handling.
- Partial-success responses where accepted and rejected records differ.
- `HTTP 400` for permanently bad payloads so clients do not retry invalid data.
- `HTTP 429` or `HTTP 503` with `Retry-After` where overload is recoverable.
- Bounded request body size before decode, matching the security lesson from
  recent Collector request-size fixes.
- Idempotency keys or event IDs at Parallax normalization boundaries, because
  OTLP delivery reliability is scoped to one client/server hop, not end-to-end
  across a multi-hop pipeline.

The initial implementation should be strict enough to protect the service and
compatible enough to receive from official SDKs and collectors.

## Semantic Context To Preserve

Parallax should preserve OTEL fields as first-class join keys rather than hide
them inside JSON blobs.

Minimum normalized fields:

| Field | Why it matters |
| --- | --- |
| `trace_id` | Primary join across spans, logs, and error events. |
| `span_id` / `parent_span_id` | Places the error inside the request or job flow. |
| `service.name` | Service identity and issue ownership. |
| `service.version` | Release regression analysis. |
| `deployment.environment.name` or equivalent deployment context | Separates production, staging, preview, and local evidence. |
| `telemetry.sdk.language`, `telemetry.sdk.name`, `telemetry.sdk.version` | SDK behavior, compatibility, and migration debugging. |
| Host/container/k8s/process attributes | Runtime placement and noisy-neighbor correlation. |
| HTTP/RPC/database/messaging semantic attributes | Root-cause hints and query pivots. |
| Exception/error semantic attributes | Bridge OTEL exceptions to Sentry-style events. |

For non-OTLP logs, Parallax should follow OTEL's stable guidance to read
`trace_id`, `span_id`, and `trace_flags` from top-level structured fields when
available.

## Collector Position

The OpenTelemetry Collector is the production integration layer, not Parallax's
core product.

Use it when a deployment needs:

- multiple receivers beyond OTLP;
- centralized resource enrichment;
- filter, transform, sampling, or redaction policy before Parallax;
- queues, retries, batching, and fan-out;
- agent, sidecar, DaemonSet, gateway, or regional collection patterns;
- trace-aware routing for tail sampling or span-metrics connectors.

Do not require it when:

- a small team wants the simplest self-hosted path;
- the app can emit OTLP directly;
- Parallax's own ingest gateway already handles auth, size limits, redaction,
  WAL append, and normalization.

The production default should be compatible with existing Collector configs:

```yaml
receivers:
  otlp:
    protocols:
      grpc:
        endpoint: 0.0.0.0:4317
      http:
        endpoint: 0.0.0.0:4318

processors:
  memory_limiter:
  batch:

exporters:
  otlp/parallax:
    endpoint: parallax-ingest:4317

service:
  pipelines:
    traces:
      receivers: [otlp]
      processors: [memory_limiter, batch]
      exporters: [otlp/parallax]
    metrics:
      receivers: [otlp]
      processors: [memory_limiter, batch]
      exporters: [otlp/parallax]
    logs:
      receivers: [otlp]
      processors: [memory_limiter, batch]
      exporters: [otlp/parallax]
```

## Rotel Position

Rotel is worth tracking because it aligns with Parallax's Rust-first thesis:
Rust implementation, low overhead positioning, Lambda/serverless form factors,
OTLP gRPC/HTTP/JSON receivers, Kafka receiver, ClickHouse/Kafka/Datadog/AWS
exporters, and Python/Rust processor SDKs.

It should not replace the official OpenTelemetry Collector in the default
compatibility story yet:

- It is early (`v0.2.2` on 2026-05-04).
- The official Collector is the ecosystem reference point for configs,
  components, operator knowledge, and vendor integrations.
- Parallax's first milestone needs interoperability more than collector
  differentiation.

Recommended posture:

1. Support official OTLP so either Collector or Rotel can sit upstream.
2. Benchmark Rotel against the latest official Collector for Parallax-shaped
   workloads: cold start, memory, CPU, ingest latency, retry behavior, and
   redaction/processor overhead.
3. Consider Rotel as an optional embedded/sidecar choice for tiny, Lambda, and
   agent-sandbox deployments if it proves materially simpler or cheaper.

## Scaling Patterns

The Collector docs support three practical scaling levels for Parallax users:

| Pattern | Fit | Parallax guidance |
| --- | --- | --- |
| Direct OTLP to Parallax | Small teams, local dev, single-node self-hosting | Default quickstart. Keep this path simple. |
| Agent/sidecar/DaemonSet Collector | Node-local collection, Kubernetes, host metrics, resource enrichment | Supported production path. Useful for pre-processing and local buffering. |
| Gateway Collector tier | Central policy, regional ingress, load balancing, tail sampling | Recommended for larger installs, especially where traces must be routed by trace ID or service. |

Collector scaling is mostly horizontal for stateless components, but stateful
processors such as tail sampling need careful routing because all spans for a
trace must reach the same decision point. Parallax should document this rather
than hide it.

## Product Opportunities Above OTEL

The strongest Parallax opportunities are:

1. **Issue intelligence.** Turn raw errors and spans into stable issues with
   grouping, regression detection, ownership, and release context.
2. **Evidence graph.** Convert OTEL records, Sentry events, deploys, commits,
   CI runs, and user actions into explicit typed edges with confidence.
3. **Agent-ready bundles.** Serve bounded, redacted context that coding agents
   can consume without broad production-data access.
4. **Rust-first ergonomics.** Make panic/error capture, `tracing` span fields,
   `anyhow`/`thiserror` chains, backtraces, debug IDs, and source links feel
   native.
5. **Self-hosted simplicity.** Keep the single-node deployment smaller than
   Sentry plus a full observability stack.
6. **Compatibility bridges.** Accept Sentry envelopes and OTLP side by side so
   teams can migrate gradually.

## Implementation Gates

Before claiming OTEL-native support, Parallax should pass these tests:

- Receive traces, logs, and metrics from official OpenTelemetry SDKs over
  OTLP/HTTP and OTLP/gRPC.
- Receive the same signals through the latest OpenTelemetry Collector and
  latest Collector Contrib distribution.
- Preserve `trace_id`, `span_id`, `service.name`, `service.version`, deployment,
  host/container/k8s, and exception attributes in queryable columns.
- Join an error event to same-trace logs and spans within a bounded time window.
- Return correct retry/backpressure behavior under invalid payload, overload,
  and temporary storage failure.
- Verify that Collector and direct-SDK ingestion produce equivalent normalized
  evidence rows.
- Verify that OTLP-derived rows assemble into schema-valid evidence bundles with
  canonical bundle hashes, evidence-edge hashes, projection manifests, and
  equivalent CLI/HTTP/MCP outputs before using them in agent-facing claims.
- Benchmark latest Rotel versus latest official Collector for the Parallax
  quickstart and Lambda/serverless cases before recommending it.

The detailed fixture matrix and pass/fail semantics for these claims are
specified in
[OTLP receiver conformance and Collector equivalence](otlp-receiver-conformance-and-collector-equivalence.md).
The access-surface projection and MCP structured-output requirements are
specified in
[Agent access surface: CLI, HTTP API, and MCP](agent-access-surface-cli-api-mcp.md)
and enforced through the OTLP conformance ledger for OTLP-derived evidence.

## Bottom Line

OpenTelemetry is becoming the universal observability protocol layer. Parallax
should embrace that rather than compete with it.

The durable product wedge is the layer above OTEL: taking traces, logs, metrics,
Sentry-style errors, deploys, commits, and CI evidence, then producing a
schema-valid, canonical, redacted debugging context that a human or coding agent
can act on. Without that bundle/projection proof, Parallax has only an ingest
claim, not an AI-native context-engine claim.
