# OpenTelemetry Protocol and Context Layer

> OpenTelemetry should be Parallax's native telemetry protocol layer and context substrate, while the durable product value lives above OTEL: Sentry-compatible error grouping, stacktrace normalization, release regression analysis, an evidence graph, and schema-bound canonical context bundles for humans and coding agents across CLI, HTTP, and MCP projections. The baseline transport claim is decided: Parallax must require `grpc` and `http/protobuf`, label `http/json` as explicitly optional (a JSON-only receiver is not enough for "OTLP-native" wording), test SDK endpoint URL construction, and back any "Collector-compatible" claim with a runnable Collector distribution fixture rather than a core/source release note. "OTLP-native" and "Collector-compatible" are conformance claims, not endpoint facts: they require direct-SDK, official Collector, Collector Contrib, and Rotel fixtures that produce equivalent normalized rows, canonical bundle and evidence-edge hashes, projection manifests, and MCP `structuredContent`/`outputSchema` validation. The current ledger status is **not measured** — no direct-SDK, Collector, Collector Contrib, or Rotel fixture results exist yet — so Parallax should describe OTLP support as a target or design direction, with the v0 gate being L1 + L2 + L3 (direct Rust SDK, signal semantics preserved, Collector equivalence) for a narrow signal subset. The open gates remain: running the dated fixture matrix to advance past `not_measured`, resolving the stable Collector distribution `v0.153.0` axis (core/source is `v0.153.0` while the runnable distribution `/releases/latest` still resolves to `v0.152.1`), and proving OTLP-derived evidence assembles into agent-ready bundles before any agent-facing claim.

This note consolidates the following previously-separate research files, each preserved in full below:

- `opentelemetry-protocol-and-context-layer.md`
- `otlp-transport-profile-recheck.md`
- `otlp-receiver-conformance-and-collector-equivalence.md`
- `otlp-conformance-ledger.md`

## OpenTelemetry Protocol and Context Layer (strategy)

_Provenance: merged verbatim from `opentelemetry-protocol-and-context-layer.md` (2026-05-29 restructure)._

<!-- markdownlint-disable MD013 -->

Research date: 2026-05-25

### Purpose

This note answers the OpenTelemetry section of the Parallax research prompt:
architecture, OTLP internals, semantic conventions, collector pipelines,
Rust-native collection, production deployment patterns, and the question of what
Parallax should build above OTEL.

Version freshness rule: this recommendation is based on current public docs and
source material checked on 2026-05-25. Every future benchmark or comparison must
use the latest reasonably available stable/public version of each candidate as
of the benchmark date, and must label older benchmark posts or architecture docs
as historical evidence.

### Short Recommendation

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

### Current Version Snapshot

| Component | Latest checked version | Release date | Notes |
| --- | --- | --- | --- |
| OpenTelemetry spec docs | OTel `1.57.0`, OTLP `1.10.0`, Semantic Conventions `1.41.0` | Checked 2026-05-25 | OTLP is stable for traces, metrics, and logs; profiles remain development-stage. |
| OpenTelemetry Collector core/source | `v0.153.0` | 2026-05-25 | Latest core/source release checked from GitHub release page, `/releases/latest` redirect, and tag ref; it has moved ahead of the runnable distribution. |
| OpenTelemetry Collector distribution | `v0.152.1` | 2026-05-20 | Latest official distribution/binary release checked from `/releases/latest` redirect and tag ref. The matching stable `v0.153.0` collector-releases page still returned 404 while nightly `v0.153.0` tags were visible. Treat this separately from core/source for conformance. |
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

### Is OTEL Becoming The Universal Protocol Layer?

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

### What OTEL Gives Parallax

| Capability | OTEL source | Parallax use |
| --- | --- | --- |
| Trace context | `trace_id`, `span_id`, parent/child spans, links, baggage | Join errors, logs, spans, and metrics around one failing request or workflow. |
| Resource identity | `service.name`, `service.version`, deployment, host, process, container, Kubernetes attributes | Attach failures to service, release, runtime, deploy, and infrastructure context. |
| Logs | OTLP log records and non-OTLP trace-context field guidance | Correlate structured logs with traces and error events without custom log format lock-in. |
| Metrics | OTLP metrics data model and Prometheus compatibility paths through collectors/backends | Add saturation, error-rate, latency, and deploy-impact evidence to error bundles. |
| Collector pipelines | receivers, processors, exporters, connectors, service pipelines | Let production users reuse existing collection, sampling, redaction, batching, retry, and fan-out infrastructure. |
| Deployment patterns | agent, gateway, sidecar, DaemonSet, load-balanced collector tiers | Fit from laptop/tiny deployments to Kubernetes and regional collector tiers. |
| Backpressure expectations | OTLP retryable errors, retry-after, exponential backoff, partial-success response model | Make Parallax's OTLP endpoints predictable under overload and invalid payloads. |

### What OTEL Does Not Give Parallax

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

### Target Ingestion Architecture

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

### OTLP Endpoint Requirements

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

### Semantic Context To Preserve

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

### Collector Position

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

### Rotel Position

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

### Scaling Patterns

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

### Product Opportunities Above OTEL

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

### Implementation Gates

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

### Bottom Line

OpenTelemetry is becoming the universal observability protocol layer. Parallax
should embrace that rather than compete with it.

The durable product wedge is the layer above OTEL: taking traces, logs, metrics,
Sentry-style errors, deploys, commits, and CI evidence, then producing a
schema-valid, canonical, redacted debugging context that a human or coding agent
can act on. Without that bundle/projection proof, Parallax has only an ingest
claim, not an AI-native context-engine claim.

## OTLP Transport Profile Recheck

_Provenance: merged verbatim from `otlp-transport-profile-recheck.md` (2026-05-29 restructure)._

_(Shared note — see the OpenTelemetry Protocol and Context Layer (strategy) section above.)_

### Pass Target

Re-check the weakest part of the current OTLP claim boundary: whether "OTLP"
is precise enough when some competitors accept only OTLP/HTTP JSON, while the
official protocol and SDK exporter specs distinguish `grpc`,
`http/protobuf`, and `http/json`.

This pass does not benchmark receiver performance. It tightens the transport,
endpoint-path, and retry semantics that Parallax must prove before using
"OTLP-native" or "Collector-compatible" wording.

### Short Verdict

Parallax's baseline OTLP claim should require both:

- OTLP/gRPC on the standard `4317` path/service shape; and
- OTLP/HTTP with binary protobuf on `4318` and `/v1/traces`, `/v1/metrics`,
  and `/v1/logs`.

OTLP/HTTP JSON is useful and the official Collector receiver supports it, but
it should remain an explicitly labeled optional path for Parallax v0. A JSON-only
receiver is not enough for "OTLP-native" Parallax wording.

Browser/frontend telemetry is the explicit exception to the gRPC baseline. The
[frontend browser ingest profile recheck](frontend-browser-ingest-profile-recheck.md)
keeps browser OTLP HTTP-only because official OpenTelemetry JavaScript docs say
browser apps cannot use OTLP/gRPC. That exception does not weaken the
server/backend OTLP-native claim; it prevents tests from treating expected
browser gRPC failure as a frontend capture defect.

The second important fix is endpoint URL construction. The OTLP exporter spec
does not treat all endpoint environment variables the same way. A generic
`OTEL_EXPORTER_OTLP_ENDPOINT=http://host:4318` lets exporters construct
per-signal `/v1/{signal}` URLs, but a per-signal endpoint such as
`OTEL_EXPORTER_OTLP_TRACES_ENDPOINT=http://host:4318` is used as-is. The fixture
gate must test both, and Parallax docs must tell users to include `/v1/traces`,
`/v1/metrics`, or `/v1/logs` when they configure per-signal HTTP endpoints.

### Current Primary-Source Snapshot

| Source | Current signal | Parallax implication |
| --- | --- | --- |
| [OpenTelemetry specs page](https://opentelemetry.io/docs/specs/) | The current docs list OpenTelemetry Specification `1.57.0`, OTLP Specification `1.10.0`, and semantic conventions `1.41.0`. | Keep the existing version matrix. No version drift from the current ledger snapshot. |
| [OTLP specification 1.10.0](https://opentelemetry.io/docs/specs/otlp/) | OTLP is stable for traces, metrics, and logs, development for profiles. It defines OTLP/gRPC and OTLP/HTTP, HTTP binary protobuf and JSON protobuf encodings, gzip, default HTTP port `4318`, partial success, bad-data handling, retryable status codes, and throttling behavior. | A receiver must prove protocol behavior, not just route existence. Profiles stay out of v0. |
| [OpenTelemetry Protocol Exporter spec](https://opentelemetry.io/docs/specs/otel/protocol/exporter/) | Exporter protocol values are `grpc`, `http/protobuf`, and `http/json`. SDKs should support both `grpc` and `http/protobuf`, must support at least one, and may support `http/json`. The default protocol should be `http/protobuf` unless an SDK has compatibility reasons to keep `grpc`. | Parallax must test `grpc` and `http/protobuf` as the baseline and label `http/json` separately. |
| [Exporter endpoint URL rules](https://opentelemetry.io/docs/specs/otel/protocol/exporter/#endpoint-urls-for-otlphttp) | Generic OTLP HTTP endpoint variables construct per-signal paths, while per-signal endpoint variables are used as-is. | Add endpoint-construction fixtures so a quickstart does not silently send traces to `/` and blame the receiver. |
| [Collector OTLP receiver README](https://github.com/open-telemetry/opentelemetry-collector/blob/main/receiver/otlpreceiver/README.md) | The receiver is stable for traces, metrics, and logs, alpha for profiles, supports gRPC or HTTP, defaults to `localhost:4317` and `localhost:4318`, and can receive HTTP/JSON with configurable per-signal paths. | Collector equivalence should include both standard ports and path behavior. JSON support is a useful comparison point, not a v0 product requirement by itself. |
| [Collector configuration docs](https://opentelemetry.io/docs/collector/configuration/) | Defining a receiver does not enable it; the receiver must be referenced by a service pipeline. The example OTLP receiver uses gRPC `4317` and HTTP `4318`. | Fixture manifests need config hashes and must distinguish configured components from enabled pipelines. |
| [OpenTelemetry proto v1.10.0](https://github.com/open-telemetry/opentelemetry-proto/releases/tag/v1.10.0) | Latest proto release checked by release page and tag ref remains `v1.10.0`; `git ls-remote` shows tag `ca839c51f706f5d53bfb46f06c3e90c3af3a52c6`. | Parser/schema fixtures should keep pinning proto separately from SDK and Collector versions. |
| [Collector core v0.153.0](https://github.com/open-telemetry/opentelemetry-collector/releases/tag/v0.153.0) | Latest core/source release checked by release page, `/releases/latest` redirect, and tag ref remains `v0.153.0`, published 2026-05-25. `git ls-remote` shows tag `c013d5846b82e502d373d6e8424236612b85ed1c`. Its release body points readers to collector-releases `v0.153.0` for binaries, but the release-tag endpoint and GitHub HTML URL for that stable distribution tag still returned 404 in this follow-up check. | Core/source drift is still a separate axis from runnable distribution binaries. Do not treat a core release-note link as proof that the matching distribution binary exists. |
| [Collector distribution v0.152.1](https://github.com/open-telemetry/opentelemetry-collector-releases/releases/tag/v0.152.1) | The collector-releases `/releases/latest` redirect still resolved to `v0.152.1` as the latest visible binary distribution on 2026-05-25, while core/source is `v0.153.0`; `git ls-remote` shows tag `853ddb2dbfab1a0cd6b43b808334fbb4cfc6160d`. A `v0.153.0` stable distribution release tag was not visible through the release-tag endpoint or HTML page; the repository tag list showed `v0.153.0-nightly.*` tags only. Unauthenticated GitHub API calls returned 403 in this pass, so API-only latest checks are not sufficient. | Do not claim current Collector equivalence from core/source alone. The manifest must pin the actual runnable binary and record release-resolution method, effective URL, HTTP status, tag ref, API availability, and whether it used a stable distribution release or a nightly tag. |
| [Collector Contrib v0.152.0](https://github.com/open-telemetry/opentelemetry-collector-contrib/releases/tag/v0.152.0) | Latest Contrib release checked remains `v0.152.0`, published 2026-05-11. | Contrib remains its own compatibility axis. |
| [opentelemetry crate](https://crates.io/crates/opentelemetry) and [opentelemetry-otlp crate](https://crates.io/crates/opentelemetry-otlp) | crates.io still reports `0.32.0` for both, updated 2026-05-08. | Rust fixtures remain the first direct SDK path; no Rust crate drift found. |
| [Rotel v0.2.2](https://github.com/rotel-dev/rotel/releases/tag/v0.2.2) and [Rotel README](https://github.com/rotel-dev/rotel) | Latest Rotel release remains `v0.2.2`. README says the receiver supports gRPC, HTTP/protobuf, and HTTP/JSON; default receiver ports are `4317` and `4318`. | Rotel smoke should test the same transport profile, but Rotel remains pre-1.0 and not the baseline. |
| [Traceway recheck](traceway-otlp-ai-replay-recheck.md) | Current Traceway evidence shows OTLP/HTTP routes for traces, metrics, and logs that accept protobuf or JSON under `/api/otel`. | Direct OTLP/HTTP is table stakes, but Traceway's base path makes standard-path and endpoint-construction tests important. |
| [Urgentry recheck](urgentry-sentry-tiny-benchmark-recheck.md) | Current Urgentry source-level recheck found OTLP HTTP/JSON routes and explicit protobuf rejection in checked source. | Treat JSON-only OTLP as a compatibility caveat, not as equivalent to Parallax's desired OTLP-native claim. |

### Required Transport Profile

| Transport path | v0 status | Claim impact |
| --- | --- | --- |
| `grpc` | Required. | Needed for Collector and many SDK deployments. |
| `http/protobuf` | Required. | Official exporter default direction and easiest direct HTTP path. |
| `http/json` | Optional, explicitly labeled. | Useful for curl/debug and parity with Traceway/Collector behavior, but not sufficient alone. |
| Browser OTLP | Separate frontend profile. | Browser builds use HTTP/protobuf or HTTP/JSON only; gRPC negative fixtures are expected. |
| Profiles | Out of scope. | OTLP profiles are development-stage in the checked spec/docs. |
| Custom base paths | Optional alias only. | Standard `/v1/{signal}` paths must work first; aliases must not replace them. |

### 2026-05-25 Collector Release-Axis Follow-Up

This follow-up keeps the existing version matrix but tightens how Parallax reads
Collector releases:

- `open-telemetry/opentelemetry-collector` `v0.153.0` exists and is the latest
  checked core/source release by release-page redirect and tag ref.
- The core release body points to
  `open-telemetry/opentelemetry-collector-releases/releases/tag/v0.153.0` for
  images and binaries.
- The collector-releases stable `v0.153.0` HTML page still returned 404, while
  `/releases/latest` redirected to `v0.152.1`.
- The collector-releases tag list showed `v0.153.0-nightly.*` tags, not a stable
  `v0.153.0` release tag.
- Unauthenticated GitHub API calls returned 403 during this pass, so the
  durable evidence relies on release redirects, HTML HTTP status, crates.io API,
  and `git ls-remote` tag refs.

Implication: the OTLP conformance manifest needs a distinct
`collector_distribution_resolution` field plus resolution method, effective URL,
HTTP status, tag ref, and API availability. A nightly Collector binary can
inform development, but it cannot satisfy the stable Collector-equivalence claim
unless the run is explicitly labeled nightly and rerun on the stable
distribution.

### Endpoint URL Construction Fixtures

Add these fixtures to the OTLP gate:

| Fixture | Configuration | Expected result |
| --- | --- | --- |
| `endpoint_generic_http_appends_paths` | `OTEL_EXPORTER_OTLP_ENDPOINT=http://parallax:4318`, protocol `http/protobuf`. | SDK sends `/v1/traces`, `/v1/metrics`, and `/v1/logs`; Parallax accepts the standard paths. |
| `endpoint_signal_http_explicit_paths` | `OTEL_EXPORTER_OTLP_TRACES_ENDPOINT=http://parallax:4318/v1/traces` and equivalent per-signal metric/log endpoints. | Parallax accepts because paths are explicit. |
| `endpoint_signal_http_missing_path` | `OTEL_EXPORTER_OTLP_TRACES_ENDPOINT=http://parallax:4318`, protocol `http/protobuf`. | Expected to hit `/`; Parallax may reject with non-retryable 404/400, and docs should explain the config error. |
| `protocol_grpc_required` | `OTEL_EXPORTER_OTLP_PROTOCOL=grpc` against `4317`. | Required pass for baseline. |
| `protocol_http_protobuf_required` | `OTEL_EXPORTER_OTLP_PROTOCOL=http/protobuf` against `4318`. | Required pass for baseline. |
| `protocol_http_json_optional` | `OTEL_EXPORTER_OTLP_PROTOCOL=http/json` against `4318`. | Either pass as an explicitly supported optional path or fail with a clear non-retryable unsupported-content result. |

### Response Semantics

Transport errors must match the OTLP retry model:

- malformed protobuf or invalid permanent data: non-retryable `400`;
- unsupported content type or unsupported optional JSON path: non-retryable
  status with a developer-facing message;
- overload, rate limit, or temporary storage/WAL failure: retryable status such
  as `429` or `503`, with `Retry-After` where possible;
- partial success: accepted records are durable, rejected counts are accurate,
  and clients should not retry the accepted records;
- no acknowledgment before local WAL/outbox durability in the tiny tier.

### Product Wording

Allowed before fixture results:

> Planned OTLP/gRPC and OTLP/HTTP protobuf ingestion.

Allowed after required transport fixtures and direct Rust signal fixtures pass:

> Rust OTLP ingestion over gRPC and HTTP/protobuf for the tested SDK/exporter
> versions.

Allowed only if JSON fixture passes:

> OTLP/HTTP JSON is supported for the tested subset.

Avoid:

- "OTLP-native" for a JSON-only endpoint;
- "Collector-compatible" without a runnable Collector distribution fixture;
- "supports OTLP" without naming `grpc`, `http/protobuf`, and optional
  `http/json` status;
- custom base-path examples that omit standard `/v1/{signal}` behavior.

### Falsification Triggers

Reopen this note if:

- OTLP spec, exporter spec, proto, Collector receiver, or Rust exporter behavior
  changes;
- a current Rust SDK defaults away from the expected transport or endpoint URL
  construction behavior;
- official Collector distribution `v0.153.0` or later appears and changes
  OTLP receiver behavior;
- real SDK fixtures show JSON is necessary for a target migration path;
- rejecting JSON causes retry loops or data-loss behavior that differs from the
  documented non-retryable model;
- a competitor proves standard gRPC plus HTTP/protobuf plus JSON plus evidence
  bundle parity with less operational complexity.

### Sources

- [OpenTelemetry specs page](https://opentelemetry.io/docs/specs/)
- [OTLP specification 1.10.0](https://opentelemetry.io/docs/specs/otlp/)
- [OpenTelemetry Protocol Exporter spec](https://opentelemetry.io/docs/specs/otel/protocol/exporter/)
- [OpenTelemetry Collector OTLP receiver README](https://github.com/open-telemetry/opentelemetry-collector/blob/main/receiver/otlpreceiver/README.md)
- [OpenTelemetry Collector configuration docs](https://opentelemetry.io/docs/collector/configuration/)
- [OpenTelemetry proto v1.10.0](https://github.com/open-telemetry/opentelemetry-proto/releases/tag/v1.10.0)
- [OpenTelemetry Collector core v0.153.0](https://github.com/open-telemetry/opentelemetry-collector/releases/tag/v0.153.0)
- [OpenTelemetry Collector distribution v0.152.1](https://github.com/open-telemetry/opentelemetry-collector-releases/releases/tag/v0.152.1)
- [OpenTelemetry Collector Contrib v0.152.0](https://github.com/open-telemetry/opentelemetry-collector-contrib/releases/tag/v0.152.0)
- [opentelemetry crate](https://crates.io/crates/opentelemetry)
- [opentelemetry-otlp crate](https://crates.io/crates/opentelemetry-otlp)
- [Rotel v0.2.2 release](https://github.com/rotel-dev/rotel/releases/tag/v0.2.2)
- [Rotel README](https://github.com/rotel-dev/rotel)
- [Traceway OTLP/AI/replay recheck](traceway-otlp-ai-replay-recheck.md)
- [Urgentry Sentry/Tiny/benchmark recheck](urgentry-sentry-tiny-benchmark-recheck.md)
- [Frontend browser ingest profile recheck](frontend-browser-ingest-profile-recheck.md)

### Bottom Line

"OTLP-compatible" is too vague for Parallax. The baseline should mean
gRPC plus HTTP/protobuf, standard paths, endpoint URL construction fixtures,
retry-safe failure behavior, and Collector distribution evidence. HTTP/JSON can
be useful, but a JSON-only path is a caveat, not the Parallax compatibility
target.

## OTLP Receiver Conformance and Collector Equivalence

_Provenance: merged verbatim from `otlp-receiver-conformance-and-collector-equivalence.md` (2026-05-29 restructure)._

_(Shared note — see the OpenTelemetry Protocol and Context Layer (strategy) section above.)_

### Purpose

[OpenTelemetry protocol and context layer](opentelemetry-protocol-and-context-layer.md)
sets the direction: Parallax should accept OTLP directly and support upstream
Collectors without making a Collector mandatory for the tiny tier. This note
turns that into a proof gate.

The product claim is not "we have an endpoint on port 4318." The claim is:

> A supported OTLP payload sent directly to Parallax, through the official
> OpenTelemetry Collector, or through Rotel produces equivalent normalized
> evidence rows, bundle edges, canonical bundles, and agent-facing projections,
> modulo explicit pipeline transformations.

If this gate fails, Parallax can still ingest OTLP experimentally, but it should
not call the path OTLP-native.

Results and product-claim status should be published through the
[OTLP conformance ledger](otlp-conformance-ledger.md), not inferred from this
fixture design alone.
The focused [OTLP transport profile recheck](otlp-transport-profile-recheck.md)
defines the required transport baseline and endpoint URL construction fixtures.

### Current Primary-Source Checks

| Source | What matters for Parallax |
| --- | --- |
| [OTLP specification 1.10.0](https://opentelemetry.io/docs/specs/otlp/) | OTLP defines encoding, transport, delivery, partial-success behavior, retryable errors, and HTTP/gRPC paths. Partial success is not a retry signal; retryable overload should use the documented retry behavior. |
| [OpenTelemetry Protocol Exporter spec](https://opentelemetry.io/docs/specs/otel/protocol/exporter/) | Exporter protocols are `grpc`, `http/protobuf`, and `http/json`; SDKs should support both `grpc` and `http/protobuf`, may support `http/json`, and build HTTP paths differently for generic versus per-signal endpoint variables. |
| [OTLP transport profile recheck](otlp-transport-profile-recheck.md) | Current primary-source pass confirmed no version drift, but tightened Parallax's claim boundary: require `grpc` and `http/protobuf`, label `http/json` optional, and test endpoint URL construction. |
| [OpenTelemetry proto v1.10.0](https://github.com/open-telemetry/opentelemetry-proto/releases/tag/v1.10.0) | Trace, log, and metric export responses carry per-signal `partial_success` fields with rejected span/log/data-point counts and human-readable error messages. |
| [OpenTelemetry logs data model](https://opentelemetry.io/docs/specs/otel/logs/data-model/) | Logs carry timestamp/observed timestamp, severity, body, attributes, resource/scope context, and optional trace/span correlation. If `SpanId` is present, `TraceId` should be present too. |
| [OpenTelemetry metrics data model](https://opentelemetry.io/docs/specs/otel/metrics/data-model/) | Metric stream identity includes resource attributes, instrumentation scope, metric name, data point type, unit, temporality, and monotonicity. Attribute sets identify individual streams. Parallax must not flatten this into ambiguous rows. |
| [OpenTelemetry trace API](https://opentelemetry.io/docs/specs/otel/trace/api/) | Spans, links, events, status, attributes, span context, trace ID, and span ID are the core lifecycle evidence for Parallax correlation. |
| [OpenTelemetry Collector configuration](https://opentelemetry.io/docs/collector/configuration/) | Collector configs are receiver/processor/exporter/connectors plus service pipelines. Defining a receiver does not enable it until a pipeline references it. Standard OTLP examples use gRPC `4317` and HTTP `4318`. |
| [OpenTelemetry Collector core v0.153.0](https://github.com/open-telemetry/opentelemetry-collector/releases/tag/v0.153.0) | Latest core/source release checked on 2026-05-25. The release page and `/releases/latest` redirect resolve to `v0.153.0`, and `git ls-remote` shows tag `c013d5846b82e502d373d6e8424236612b85ed1c`. It stabilizes several feature gates including pdata/proto encoding and ref-counting behavior, and fixes a Snappy memory-corruption issue in gRPC config. Its release body points to collector-releases `v0.153.0` for binaries, but the stable collector-releases tag returned 404 in the follow-up check. | Treat Collector source/core version as a separate compatibility axis from the runnable distribution. Payload, pdata, compression, and config behavior changes should trigger fixture reruns, but a core release-note link is not enough to declare a runnable distribution current. |
| [OpenTelemetry Collector distribution v0.152.1](https://github.com/open-telemetry/opentelemetry-collector-releases/releases/tag/v0.152.1) | Latest official distribution/binary release resolved by `/releases/latest` redirect on 2026-05-25, even though core/source has moved to `v0.153.0`; `git ls-remote` shows tag `853ddb2dbfab1a0cd6b43b808334fbb4cfc6160d`. The stable `v0.153.0` distribution tag and HTML page returned 404; the repo tag list showed `v0.153.0-nightly.*` tags only. Unauthenticated GitHub API calls returned 403 in this pass, so future runs should not rely on API-only latest resolution. | A conformance manifest must record the exact runnable Collector binary distribution, resolution source, tag ref, HTTP status, API availability, and stable-versus-nightly status separately from the core/source release; do not claim current Collector equivalence from only one axis. |
| [OpenTelemetry Collector Contrib v0.152.0](https://github.com/open-telemetry/opentelemetry-collector-contrib/releases/tag/v0.152.0) | Contrib is the realistic production distribution for broader processors/receivers/exporters. Parallax should verify both core and contrib where pipeline components differ. |
| [OpenTelemetry Rust 0.32.0](https://github.com/open-telemetry/opentelemetry-rust/releases/tag/opentelemetry-0.32.0) | Latest Rust release checked. Rust SDK fixtures are the first direct-SDK path because Parallax is Rust-first. |
| [Rotel v0.2.2](https://github.com/rotel-dev/rotel/releases/tag/v0.2.2) and [Rotel README](https://github.com/rotel-dev/rotel) | Rotel supports metrics/logs/traces, OTLP gRPC, OTLP HTTP/protobuf, OTLP HTTP/JSON, OTLP export, batching, retries, and resource attributes, with default receiver paths on `4317`/`4318`. It is promising but early and must be a smoke/eval path, not the compatibility baseline. |
| [GreptimeDB OTLP docs](https://docs.greptime.com/user-guide/ingest-data/for-observability/opentelemetry/) | GreptimeDB can consume OTLP/HTTP, but its metric mapping can rename metrics/labels and discard some resource/scope attributes by default in Prometheus-compatible mode. Parallax must own normalization before storage or configure storage ingestion deliberately. |
| [MCP tools 2025-11-25](https://modelcontextprotocol.io/specification/2025-11-25/server/tools) and [MCP base protocol 2025-11-25](https://modelcontextprotocol.io/specification/2025-11-25/basic/index) | Tool results can include schema-validated `structuredContent`; MCP uses JSON Schema 2020-12 by default; `_meta` is reserved metadata. OTLP evidence served through MCP must be canonical structured JSON, not text-only output, and safety-critical fields cannot be hidden only in `_meta`. |
| [RFC 8785 JSON Canonicalization Scheme](https://www.rfc-editor.org/rfc/rfc8785.html) | JCS provides deterministic JSON serialization for repeatable hashing. Bundle and projection equality should use canonical JSON hashes rather than renderer-specific text. |

### Compatibility Levels

| Level | Meaning | Product wording |
| --- | --- | --- |
| L0 endpoint | OTLP/gRPC `4317`, OTLP/HTTP `4318`, `/v1/traces`, `/v1/logs`, `/v1/metrics`, `grpc`, `http/protobuf`, gzip, size limits, endpoint URL construction behavior, and stable error responses work. HTTP/JSON is optional and labeled. | "OTLP endpoint." |
| L1 direct Rust SDK | Current OpenTelemetry Rust traces, logs, and metrics reach Parallax directly and normalize into queryable rows. | "Rust OTLP ingestion." |
| L2 signal semantics | Trace, log, and metric fixtures preserve resource, scope, trace/span IDs, status, links/events, log bodies, metric temporality, histograms, and attributes. | "OTLP-native telemetry ingestion." |
| L3 Collector equivalence | Same fixtures through official Collector and Collector Contrib produce equivalent normalized rows, except declared processor-added/removed fields. | "Collector-compatible OTLP ingestion." |
| L4 Rotel equivalence | Same fixtures through Rotel produce equivalent normalized rows for supported receiver/exporter modes. | "Rotel-compatible OTLP ingestion." |
| L5 production pipeline | Redaction, batching, retries, partial success, overload, idempotency, and storage-failure behavior are proven under mixed signal load. | "Production-ready OTLP ingestion." |

The v0 target is **L1 + L2 + L3** for a narrow signal subset. L4 is a smoke gate
because Rotel is still pre-1.0. L5 is required before broad public claims.

### Fixture Generation Strategy

Fixtures should be generated from real SDKs and collectors, not handwritten
protobuf blobs:

```text
fixture app / sdk version / signal scenario
  -> direct OTLP/gRPC to Parallax
  -> direct OTLP/HTTP protobuf to Parallax
  -> optional direct OTLP/HTTP JSON to Parallax
  -> OpenTelemetry Collector -> Parallax
  -> Collector Contrib -> Parallax
  -> Rotel -> Parallax
  -> raw payload hash + normalized row snapshots
  -> bundle edge snapshots
  -> canonical evidence bundle snapshot
  -> CLI/API/MCP projection-equivalence snapshots
```

Each fixture directory should record:

- SDK/exporter name and version;
- runtime version;
- transport (`grpc`, `http/protobuf`, optional `http/json`);
- intermediary (`none`, `collector-core`, `collector-contrib`, `rotel`);
- intermediary source/core version and runnable binary/distribution version when
  they differ;
- intermediary release resolution method, effective release URL, HTTP status,
  tag ref, and API availability;
- Collector/Rotel config;
- raw request hash before and after intermediary;
- expected accepted and rejected counts;
- normalized row snapshot;
- evidence-edge snapshot;
- schema ref and schema hash for the bundle produced from the fixture;
- canonical bundle hash and evidence-edge hashes;
- projection manifest hash for JSON, Markdown, CLI, HTTP API, and MCP surfaces;
- MCP output-schema hash and structured-content validation result when the
  projection surface is MCP;
- redaction report snapshot when attributes/log bodies include canaries.

### Fixture Matrix

| Fixture | Must prove |
| --- | --- |
| `trace_basic_tree` | Root/child spans preserve `trace_id`, `span_id`, parent ID, timestamps, duration, kind, status, attributes, resource, and scope. |
| `trace_links_events` | Span links and events are stored as evidence, not silently dropped. |
| `trace_exception_attrs` | Exception semantic attributes can support error correlation without replacing Sentry error events. |
| `log_correlated` | Log record with `trace_id` and `span_id` joins to the matching span. |
| `log_uncorrelated` | Log record without trace context is accepted but never promoted to a strong edge by time proximity alone. |
| `log_complex_body` | String, map, array, bytes, and numeric bodies either normalize safely or receive explicit unsupported-field outcomes. Include `<`, `>`, and `&` inside map/list values to detect Collector `AsString` rendering drift and prove redaction sees the typed value. |
| `metric_gauge` | Gauge points preserve resource, scope, name, unit, attributes, value, and timestamp. |
| `metric_sum_delta_cumulative` | Sum points preserve temporality and monotonicity; delta and cumulative series are not merged. |
| `metric_histogram` | Explicit bucket histograms preserve count, sum, bucket boundaries/counts, min/max when present, and exemplars. |
| `metric_exponential_histogram` | Either supported explicitly or rejected/ref-stored with partial success; no silent conversion. |
| `resource_collision` | `service.name`, `service.version`, deployment environment, host/container/process attrs, and custom attrs remain separable. |
| `multi_resource_batch` | One OTLP request containing multiple resource groups normalizes to independent resource identities. |
| `collector_batch_reorder` | Collector batching/reordering does not change row identity or bundle edges. |
| `collector_resource_enrichment` | Resource processor additions are recorded as pipeline-added evidence, not mistaken for SDK-origin fields. |
| `rotel_forward` | Rotel forwarding preserves supported signal fields for the same fixture subset. |
| `gzip_http` | Compressed OTLP/HTTP requests work and obey the same body-size and decode limits. |
| `endpoint_generic_http_appends_paths` | Generic `OTEL_EXPORTER_OTLP_ENDPOINT=http://host:4318` constructs `/v1/traces`, `/v1/metrics`, and `/v1/logs` for HTTP exporters. |
| `endpoint_signal_http_explicit_paths` | Per-signal endpoint variables work when the `/v1/{signal}` path is explicit. |
| `endpoint_signal_http_missing_path` | Per-signal endpoint variables without a path hit `/` and receive a clear non-retryable config error, not ambiguous data loss. |
| `json_http_optional` | OTLP/HTTP JSON works only if explicitly supported; otherwise rejected with clear non-retryable behavior and product wording says JSON is unsupported. |
| `malformed_payload` | Invalid protobuf/JSON fails before expensive processing and does not poison subsequent requests. |
| `partial_reject` | Oversized or policy-rejected records return correct partial-success counts and messages. |
| `storage_unavailable` | Temporary storage/WAL outage returns retryable overload behavior without accepting data that is not durable. |
| `duplicate_delivery` | Retried batches do not duplicate normalized rows or issue/event edges. |

### Normalization Equivalence Rules

Equivalence should be set-based, not byte-for-byte:

- OTLP intermediaries may batch, split, or reorder telemetry.
- Collector processors may intentionally add, remove, or transform attributes.
- Resource and scope identities must remain explicit.
- Trace/log correlation keys must not be lost.
- Metric stream identity must include data type, unit, temporality, and
  monotonicity.
- Nested `AnyValue` values must remain typed through normalization. Equivalence
  should not be based on a string renderer for map/list bodies because Collector
  rendering behavior can change without changing the underlying telemetry
  value.
- Raw payload refs should record both the received Parallax payload and, when
  available, the original SDK payload before an intermediary changed it.
- Canonical bundle hashes and evidence-edge hashes must match across direct,
  Collector, Collector Contrib, and Rotel paths for the same semantic fixture,
  except for declared processor transformations.
- CLI JSON, HTTP API JSON, MCP `structuredContent`, Markdown, and stored bundle
  JSON are projections of the same canonical bundle. A renderer-only match does
  not prove equivalence.
- MCP projections must validate `structuredContent` against the declared
  `outputSchema`. Text-only MCP output cannot prove agent-ready OTLP evidence.
- Redaction status, source-field policy status, and missing-evidence reports
  must be present in canonical structured content, not only in MCP `_meta`.

The normalized row identity should be derived from signal semantics:

| Signal | Candidate identity |
| --- | --- |
| Span | `project_id + trace_id + span_id + start_time_unix_nano + span_name` |
| Log | `project_id + observed_time + trace_id/span_id if present + body_hash + attrs_hash` |
| Metric stream | `project_id + resource_identity + scope_identity + metric_name + data_type + unit + temporality + monotonicity + attrs_hash` |

Do not let GreptimeDB's default OTLP mapping decide Parallax semantics. The
GreptimeDB docs show useful native OTLP ingestion, but also show metric/label
renaming and default resource/scope attribute behavior. Parallax should either:

1. normalize OTLP in `parallax-ingest` before writing storage rows; or
2. send storage headers/config that preserve every field required for evidence
   bundles and verify the resulting rows with this gate.

The first option is safer for v0 because it keeps the open evidence schema
independent of storage-specific OTLP mapping.

### Collector And Rotel Test Configs

The official Collector equivalence fixture should start with the smallest
pipeline:

```yaml
receivers:
  otlp:
    protocols:
      grpc:
        endpoint: 127.0.0.1:4317
      http:
        endpoint: 127.0.0.1:4318

processors:
  batch:

exporters:
  otlp/parallax:
    endpoint: 127.0.0.1:14317
    tls:
      insecure: true

service:
  pipelines:
    traces:
      receivers: [otlp]
      processors: [batch]
      exporters: [otlp/parallax]
    metrics:
      receivers: [otlp]
      processors: [batch]
      exporters: [otlp/parallax]
    logs:
      receivers: [otlp]
      processors: [batch]
      exporters: [otlp/parallax]
```

Add `memory_limiter`, resource enrichment, transform/redaction, and retry queue
fixtures only after the base equivalence test passes.

Rotel should be tested as an alternate upstream OTLP hop with the same fixture
subset:

```sh
rotel start \
  --otlp-grpc-endpoint 127.0.0.1:4317 \
  --otlp-http-endpoint 127.0.0.1:4318 \
  --exporter otlp \
  --otlp-exporter-endpoint 127.0.0.1:14317 \
  --otlp-exporter-protocol grpc
```

If Rotel differs, document whether the difference is a Rotel bug, a Parallax
parser bug, or an unsupported Rotel mode. Do not block v0 on Rotel unless the
official Collector path also fails.

### Endpoint Behavior Requirements

Parallax's OTLP receiver should expose and test:

| Behavior | Requirement |
| --- | --- |
| Ports | gRPC `4317`; HTTP `4318`. |
| Paths | `/v1/traces`, `/v1/logs`, `/v1/metrics`; optional base-path alias only if documented. |
| Transport profile | `grpc` and `http/protobuf` required; `http/json` optional and explicitly labeled. |
| Content types | `application/x-protobuf` required for HTTP/protobuf; `application/json` only if the optional JSON path is supported. |
| Endpoint URL construction | Generic and per-signal OTLP endpoint variables are tested because per-signal HTTP endpoints are used as-is. |
| Compression | gzip for HTTP and gRPC where SDKs/collectors use it. |
| Payload limits | Reject oversized requests before full decode. |
| Partial success | Accepted records are durable; rejected counts are accurate; sender should not retry accepted records. |
| Retryable errors | Temporary overload/storage failure returns retryable status plus `Retry-After` where possible. |
| Non-retryable errors | Malformed payload, auth failure, unsupported content type, and permanently invalid records return non-retryable status. |
| Durability | Do not acknowledge accepted telemetry before local WAL/outbox durability in the tiny tier. |
| Audit | Record source (`direct`, `collector-core`, `collector-contrib`, `rotel`) and config hash when known. |

### Pass / Fail Gate

Pass only when:

- current OpenTelemetry Rust fixtures pass over OTLP/gRPC and OTLP/HTTP
  protobuf;
- endpoint URL construction fixtures pass for generic endpoints, explicit
  per-signal paths, and documented missing-path failures;
- OTLP/HTTP JSON either passes as an explicitly supported optional path or
  fails with a clear non-retryable unsupported-content result;
- the same fixture set passes through the latest checked official Collector
  distribution, with the Collector core/source version recorded separately when
  it has moved ahead of the runnable distribution;
- the same fixture set passes through Collector Contrib `v0.152.0` for the
  configured components Parallax recommends;
- Rotel `v0.2.2` smoke fixtures pass for the supported subset or any
  differences are documented and not product-blocking;
- direct and Collector paths produce equivalent normalized rows and evidence
  edges;
- direct, Collector, Collector Contrib, and Rotel paths produce matching
  canonical bundle and evidence-edge hashes for supported, untransformed fields;
- CLI, HTTP API, Markdown, and MCP projections validate against the projection
  manifest and match the canonical bundle hash for the same fixture;
- MCP tool output declares an output schema and returns schema-valid
  `structuredContent`;
- `trace_id`, `span_id`, `service.name`, `service.version`, deployment
  environment, resource attrs, scope attrs, and metric temporality are not lost;
- nested log bodies and attributes preserve typed `AnyValue` semantics, and
  redaction canaries inside map/list bodies are detected before any text
  rendering;
- redaction canaries in attributes, log bodies, and resource fields are removed
  from agent-visible JSON/Markdown;
- raw OTLP request refs and Collector payload refs remain referenced but are not
  dereferenced into agent-visible projections by default;
- retry, partial success, and duplicate delivery behavior is deterministic.

Fail or narrow the claim if:

- Parallax requires a Collector for the tiny tier;
- Collector forwarding changes supported fields without the diff being
  explicitly caused by configured processors;
- metrics lose temporality/monotonicity or merge incompatible streams;
- logs with trace context fail to join spans;
- GreptimeDB's OTLP mapping drops evidence-critical fields and Parallax has no
  normalization layer to compensate;
- canonical bundle hashes or projection-equivalence hashes diverge for
  undeclared reasons;
- MCP returns only text or hides safety-critical fields only in `_meta`;
- raw payload bytes become visible to agent surfaces without an explicit
  read-sensitive approval path;
- partial-success responses cause SDK/Collector retry loops or duplicate rows;
- unsupported payloads are silently accepted and then disappear.

### Product Implication

The honest first release wording should be:

> OTLP-native for a tested subset of current Rust SDK traces, logs, and metrics,
> with direct SDK and official Collector equivalence, canonical bundle
> projection, and MCP structured-output validation.

Avoid:

- "full OpenTelemetry backend";
- "Collector replacement";
- "supports all OTLP data";
- "Rotel-based ingestion" as a default claim before Rotel-specific equivalence
  and overhead tests pass.

This keeps the tiny tier simple while making the production Collector path real.

### Relationship To Other Research

- [OpenTelemetry protocol and context layer](opentelemetry-protocol-and-context-layer.md)
  makes the protocol decision; this note defines the conformance proof.
- [OTLP conformance ledger](otlp-conformance-ledger.md) turns conformance runs
  into claim levels, row schemas, expiry triggers, and allowed product wording.
- [Self-hosted simplicity gate](self-hosted-simplicity-gate.md) requires OTLP
  ingestion without an external Collector in the tiny tier.
- [Storage freshness and bundle latency gate](storage-freshness-and-bundle-latency-gate.md)
  measures whether accepted OTLP becomes queryable fast enough.
- [Redaction pipeline and secret safety](redaction-pipeline-and-secret-safety.md)
  has veto power over OTLP attributes and log bodies before agent exposure.
- [Evidence bundle and open schema](evidence-bundle-and-schema.md) consumes the
  normalized rows and evidence edges this gate protects.

### Bottom Line

OTLP is the right protocol substrate, but OTLP-native is a conformance claim.
Parallax should earn it with direct-SDK, Collector, Collector Contrib, and Rotel
fixtures that prove equivalent normalized evidence, canonical bundles, and
agent-facing projections. Otherwise it risks building another "accepts protobuf"
endpoint that loses the exact fields agents need after ingest.

## OTLP Conformance Ledger

_Provenance: merged verbatim from `otlp-conformance-ledger.md` (2026-05-29 restructure)._

_(Shared note — see the OpenTelemetry Protocol and Context Layer (strategy) section above.)_

### Purpose

This ledger turns "OTLP-native" and "Collector-compatible" into auditable
claims. It consumes the proof gate in
[OTLP receiver conformance and Collector equivalence](otlp-receiver-conformance-and-collector-equivalence.md)
and the protocol decision in
[OpenTelemetry protocol and context layer](opentelemetry-protocol-and-context-layer.md).

Current status: **not measured**. The repository has an OTLP strategy and
fixture design, but it does not yet have direct-SDK, Collector, Collector
Contrib, or Rotel fixture results. Until those results exist, Parallax should
describe OTLP support as a target or design direction, not as a proven product
property.

The central rule:

> No public "OTLP-native" claim without a dated protocol/version matrix,
> direct-SDK results, Collector equivalence results, normalized row snapshots,
> canonical bundle and evidence-edge hashes, projection manifests,
> CLI/HTTP/MCP projection-equivalence results, MCP `structuredContent` /
> `outputSchema` validation, partial-success/retry behavior, and explicit
> unsupported-field outcomes.

### Current Source Snapshot

| Source | Current check | Why it matters |
| --- | --- | --- |
| [OpenTelemetry specs page](https://opentelemetry.io/docs/specs/) | The docs currently list OpenTelemetry Specification `1.57.0`, OTLP Specification `1.10.0`, and semantic conventions `1.41.0`. | Result runs must pin which spec and semantic-convention versions the claim was tested against. |
| [OTLP specification 1.10.0](https://opentelemetry.io/docs/specs/otlp/) | OTLP is stable for traces, metrics, and logs, development-stage for profiles; it defines gRPC and HTTP transports, protobuf payloads, gzip support, partial success, bad-data behavior, retryable status codes, and interoperability rules. | The receiver gate must test protocol behavior, not only decode success. |
| [OpenTelemetry Protocol Exporter spec](https://opentelemetry.io/docs/specs/otel/protocol/exporter/) | Exporter protocol values are `grpc`, `http/protobuf`, and `http/json`. SDKs should support both `grpc` and `http/protobuf`, may support `http/json`, and construct HTTP paths differently for generic versus per-signal endpoint variables. | The ledger must include transport-profile and endpoint URL construction rows, not just payload-decoding rows. |
| [OTLP transport profile recheck](otlp-transport-profile-recheck.md) | Current recheck found no source-version drift, but tightened the claim boundary around required `grpc` and `http/protobuf`, optional `http/json`, endpoint URL construction fixtures, and JSON-only competitor caveats. | A JSON-only endpoint cannot advance Parallax beyond an experimental/partial OTLP claim. |
| [OpenTelemetry proto v1.10.0](https://github.com/open-telemetry/opentelemetry-proto/releases/tag/v1.10.0) | The protobuf release is the schema baseline for export request/response messages. | Parallax parser and fixture generators must pin proto versions separately from SDK versions. |
| [OpenTelemetry logs data model](https://opentelemetry.io/docs/specs/otel/logs/data-model/) | Logs carry timestamp, observed timestamp, trace/span context, severity, body, resource, instrumentation scope, and attributes. If `SpanId` is present, `TraceId` should also be present. | Log rows must preserve trace joins and structured bodies instead of flattening logs into lossy text. |
| [OpenTelemetry metrics data model](https://opentelemetry.io/docs/specs/otel/metrics/data-model/) | The metrics model is stable and explicitly preserves metric semantics across transformations, including temporality and stream identity. | Parallax must not merge incompatible streams or drop temporality/monotonicity. |
| [OpenTelemetry trace API](https://opentelemetry.io/docs/specs/otel/trace/api/) | Spans carry parent/child relations, span kind, attributes, links, events, status, start/end timestamps, trace ID, and span ID. | These fields are the core lifecycle evidence for bundles. |
| [Collector configuration](https://opentelemetry.io/docs/collector/configuration/) | Collector configs define receivers, processors, exporters, connectors, extensions, and service pipelines. Configuring a component does not enable it until a pipeline references it. OTLP defaults use `4317` and `4318`. | Equivalence results must include config hashes and declared processor transforms. |
| [Collector core v0.153.0](https://github.com/open-telemetry/opentelemetry-collector/releases/tag/v0.153.0) | Current checked core/source release is `v0.153.0`, published on 2026-05-25. The GitHub release page and `/releases/latest` redirect resolve to `v0.153.0`, and `git ls-remote` shows tag `c013d5846b82e502d373d6e8424236612b85ed1c`. Its release body points to collector-releases `v0.153.0` for binaries, but that stable release tag still returned 404 in the follow-up check. | Core/source drift can affect payload handling, pdata semantics, compression, or config interpretation. Fixture runs must record Collector source/core separately from the runnable distribution and must not infer a runnable binary from core release notes alone. |
| [Collector distribution v0.152.1](https://github.com/open-telemetry/opentelemetry-collector-releases/releases/tag/v0.152.1) | The official distribution/binary `/releases/latest` redirect still resolved to `v0.152.1` on 2026-05-25 while core/source had moved to `v0.153.0`; `git ls-remote` shows tag `853ddb2dbfab1a0cd6b43b808334fbb4cfc6160d`. The stable `v0.153.0` release page returned 404, while the tag list showed only `v0.153.0-nightly.*` tags. Unauthenticated GitHub API calls returned 403 during this pass, so the source snapshot used release redirects, HTML status, and tag refs instead of API-only latest resolution. The `v0.152.1` line had request-body/decompression fixes and a `pcommon.Value.AsString` behavior change: map/slice values no longer HTML-escape `<`, `>`, and `&`. | Compatibility claims should name the tested binary distribution, release resolution source, and the core/source line. Conformance must preserve typed AnyValue/log-body semantics and not rely on string-rendered equality for redaction or row identity. Nightly distribution tests are development evidence, not stable Collector-equivalence proof. |
| [Collector Contrib v0.152.0](https://github.com/open-telemetry/opentelemetry-collector-contrib/releases/tag/v0.152.0) | Current checked Contrib release is `v0.152.0`, released on 2026-05-11, with broad processor/receiver/exporter changes. | Contrib is the realistic production distribution for many deployments; it needs its own fixture row. |
| [OpenTelemetry Rust 0.32.0](https://docs.rs/crate/opentelemetry/latest) and [opentelemetry-otlp 0.32.0 changelog](https://docs.rs/crate/opentelemetry-otlp/latest/source/CHANGELOG.md) | Docs.rs resolves `opentelemetry` to `0.32.0`; `opentelemetry-otlp` 0.32.0 adds per-signal protocol env vars and OTLP partial-success handling. | Rust fixtures should cover per-signal protocol settings and server partial-success responses. |
| [Rotel v0.2.2](https://github.com/rotel-dev/rotel/releases/tag/v0.2.2) and [Rotel README](https://github.com/rotel-dev/rotel) | Rotel is a Rust OpenTelemetry collector alternative with default OTLP gRPC `4317`, HTTP `4318`, `/v1/traces`, `/v1/metrics`, `/v1/logs`, gzip export, retries/timeouts, and multiple exporters. | Rotel is a useful smoke/eval path, but it is still pre-1.0 and should not replace official Collector equivalence. |
| [GreptimeDB OTLP docs](https://docs.greptime.com/user-guide/ingest-data/for-observability/opentelemetry/) | GreptimeDB supports OTLP/HTTP, but metric ingestion can rename metric/label names, keep only selected resource attributes by default, discard scope attributes by default, and currently does not support ExponentialHistogram. | Parallax must own evidence semantics before storage or prove storage mapping preserves all evidence-critical fields. |
| [MCP tools 2025-11-25](https://modelcontextprotocol.io/specification/2025-11-25/server/tools) and [MCP base protocol 2025-11-25](https://modelcontextprotocol.io/specification/2025-11-25/basic/index) | Tool results can carry JSON `structuredContent`; tools can declare `outputSchema`; MCP uses JSON Schema 2020-12 by default; `_meta` is reserved protocol metadata. | OTLP-derived evidence served to agents must be canonical structured JSON, not text-only output, and safety-critical fields cannot be hidden only in `_meta`. |
| [RFC 8785 JSON Canonicalization Scheme](https://www.rfc-editor.org/rfc/rfc8785.html) | JCS defines deterministic JSON serialization for repeatable hashing/signing. | Bundle and projection equivalence should use canonical JSON hashes, not renderer-specific byte output. |

### Claim Levels

| Level | Meaning | Allowed wording |
| --- | --- | --- |
| `not_measured` | No current fixture run exists. | "OTLP-native ingestion is planned." |
| `endpoint_smoke` | OTLP/gRPC and OTLP/HTTP protobuf endpoints accept simple valid payloads, endpoint URL construction fixtures behave as documented, and malformed/unsupported payloads fail deterministically. HTTP/JSON is optional and labeled. | "OTLP endpoint prototype." |
| `direct_rust_traces` | Current OpenTelemetry Rust trace fixtures reach Parallax directly over gRPC and HTTP/protobuf and normalize into span rows. | "Rust OTLP trace ingestion." |
| `direct_rust_three_signal` | Current OpenTelemetry Rust traces, logs, and metrics reach Parallax directly and normalize into queryable rows. | "Rust OTLP traces, logs, and metrics ingestion." |
| `otel_semantics_preserved` | Direct fixtures preserve resource, scope, trace/span IDs, span links/events/status, log bodies/severity, metric stream identity, temporality, histograms, attributes, canonical evidence-edge hashes, canonical bundle hashes, and projection manifests. | "OTLP telemetry semantics preserved for the tested subset." |
| `collector_core_equivalent` | Official Collector distribution forwarding produces equivalent normalized rows and bundle edges for the tested subset, with the Collector source/core version recorded separately, except declared processor changes. | "Collector-compatible OTLP ingestion." |
| `collector_contrib_equivalent` | Collector Contrib forwarding produces equivalent normalized rows for recommended Contrib processors/components. | "Collector Contrib-compatible for the tested pipeline." |
| `rotel_smoke_equivalent` | Rotel forwarding preserves the tested subset or documented differences are non-blocking. | "Rotel-compatible smoke tested." |
| `production_otlp_ingest` | Redaction, batching, retries, partial success, duplicate delivery, overload, WAL durability, storage-failure behavior, canonical projection equivalence, and MCP structured-output validation pass under mixed load. | "Production-ready OTLP ingestion for the tested subset." |
| `claim_expired` | A spec/proto/SDK/Collector/Rotel/storage mapping/redaction/parser version changed or the freshness window elapsed. | "OTLP result expired; rerun required." |
| `claim_failed` | A fixture run failed any required gate for the advertised level. | No claim for the affected signal/path/version. |

Initial Parallax level: `not_measured`.

### Result Artifacts

Conformance runs should be durable and diffable:

```text
docs/research/otlp-conformance-results.md
docs/research/otlp-conformance-runs/<run_id>/manifest.json
docs/research/otlp-conformance-runs/<run_id>/raw-requests/<fixture_id>.<transport>.pb
docs/research/otlp-conformance-runs/<run_id>/collector-configs/otelcol.yaml
docs/research/otlp-conformance-runs/<run_id>/collector-configs/otelcol-contrib.yaml
docs/research/otlp-conformance-runs/<run_id>/collector-configs/rotel.args
docs/research/otlp-conformance-runs/<run_id>/endpoint-results.jsonl
docs/research/otlp-conformance-runs/<run_id>/normalization-results.jsonl
docs/research/otlp-conformance-runs/<run_id>/equivalence-results.jsonl
docs/research/otlp-conformance-runs/<run_id>/metric-stream-results.jsonl
docs/research/otlp-conformance-runs/<run_id>/anyvalue-rendering-results.jsonl
docs/research/otlp-conformance-runs/<run_id>/partial-success-results.jsonl
docs/research/otlp-conformance-runs/<run_id>/retry-overload-results.jsonl
docs/research/otlp-conformance-runs/<run_id>/redaction-results.jsonl
docs/research/otlp-conformance-runs/<run_id>/bundle-projection-results.jsonl
docs/research/otlp-conformance-runs/<run_id>/mcp-structured-output-results.jsonl
docs/research/otlp-conformance-runs/<run_id>/claim-ledger.jsonl
docs/research/otlp-conformance-runs/<run_id>/hashes.sha256
```

Do not create these result directories for hypothetical data. Add them only when
a real fixture run exists.

### Run Manifest

Each `manifest.json` should include:

```json
{
  "run_id": "otlp-conformance-YYYYMMDD-N",
  "research_date": "YYYY-MM-DD",
  "fixture_generator_commit": "<git-sha>",
  "parallax_parser_commit": "<git-sha>",
  "parallax_normalizer_version": "otlp-normalizer-vN",
  "redaction_policy_version": "a6-default-deny-vN",
  "bundle_schema_ref": {
    "uri": "schema://parallax/evidence-bundle/v0",
    "hash": "sha256:<hex>",
    "canonicalization": "jcs-rfc8785"
  },
  "canonical_bundle_hash_required": true,
  "projection_manifest_required": true,
  "projection_surfaces_required": [
    "bundle_json",
    "bundle_markdown",
    "cli_output",
    "http_api",
    "mcp_structuredContent"
  ],
  "mcp_output_schema_required": true,
  "source_snapshot": {
    "otel_spec": "1.57.0",
    "otlp_spec": "1.10.0",
    "semantic_conventions": "1.41.0",
    "opentelemetry_proto": "1.10.0",
    "opentelemetry_rust": "0.32.0",
    "opentelemetry_otlp": "0.32.0",
    "collector_core_source": "0.153.0",
    "collector_core_resolution": "releases_latest_redirect=v0.153.0; tag_ref=c013d5846b82e502d373d6e8424236612b85ed1c; github_api_status=403_unauthenticated",
    "collector_distribution": "0.152.1",
    "collector_distribution_resolution": "releases_latest_redirect=v0.152.1; tag_ref=853ddb2dbfab1a0cd6b43b808334fbb4cfc6160d; stable_v0.153.0_release_tag_404; v0.153.0-nightly_tags_visible; github_api_status=403_unauthenticated",
    "collector_contrib": "0.152.0",
    "collector_contrib_resolution": "releases_latest_redirect=v0.152.0; tag_ref=fd4d5188f6bc33e6c90329ff7caa1df62c6c543d",
    "rotel": "0.2.2"
  },
  "transports": ["grpc", "http/protobuf"],
  "optional_transports": ["http/json"],
  "endpoint_url_construction_required": true,
  "intermediaries": ["none", "collector-core", "collector-contrib", "rotel"],
  "intermediary_version_axes": {
    "collector_core": ["source_release", "distribution_binary"],
    "collector_contrib": ["contrib_distribution", "embedded_core"],
    "rotel": ["release"]
  },
  "storage_mapping": "parallax-owned-normalization",
  "size_limits": {},
  "durability_policy": "ack_after_wal",
  "notes": []
}
```

The manifest must separate protocol versions, SDK/exporter versions,
intermediary versions, Parallax parser/normalizer versions, redaction version,
and storage mapping. A pass in one combination does not carry over to another.

### Row Schemas

#### Fixture Matrix Row

```json
{
  "fixture_id": "trace_basic_tree",
  "signal": "traces",
  "sdk_name": "opentelemetry-rust",
  "sdk_version": "0.32.0",
  "exporter": "opentelemetry-otlp",
  "exporter_version": "0.32.0",
  "runtime": "rustc <version>",
  "transport": "grpc|http/protobuf",
  "intermediary": "none|collector-core|collector-contrib|rotel",
  "intermediary_source_version": "0.153.0|null",
  "intermediary_distribution_version": "0.152.1|null",
  "intermediary_resolution_method": "release_redirect|release_page|git_ls_remote|api|manual",
  "intermediary_release_url_effective": "https://github.com/open-telemetry/opentelemetry-collector-releases/releases/tag/v0.152.1|null",
  "intermediary_release_http_status": 200,
  "intermediary_tag_ref": "853ddb2dbfab1a0cd6b43b808334fbb4cfc6160d|null",
  "config_hash": "sha256:<hex>",
  "request_hash": "sha256:<hex>",
  "target_level": "direct_rust_traces"
}
```

#### Endpoint Result Row

```json
{
  "fixture_id": "gzip_http",
  "transport": "http/protobuf",
  "path": "/v1/traces",
  "endpoint_env_mode": "generic|per_signal_explicit_path|per_signal_missing_path",
  "content_type": "application/x-protobuf",
  "compression": "gzip",
  "accepted": true,
  "status_code": 200,
  "retryable": false,
  "partial_success": false,
  "request_size_bytes": 1234
}
```

#### Normalization Result Row

```json
{
  "fixture_id": "trace_links_events",
  "signal": "traces",
  "resource_identity_preserved": true,
  "scope_identity_preserved": true,
  "trace_id_preserved": true,
  "span_id_preserved": true,
  "span_links_preserved": true,
  "span_events_preserved": true,
  "status_preserved": true,
  "attribute_count": 42,
  "evidence_edge_hash": "sha256:<hex>",
  "canonical_bundle_hash": "sha256:<hex>",
  "projection_manifest_hash": "sha256:<hex>",
  "intentional_drops": []
}
```

#### Equivalence Result Row

```json
{
  "fixture_id": "collector_batch_reorder",
  "direct_request_hash": "sha256:<hex>",
  "intermediary": "collector-core",
  "intermediary_version": "0.152.1",
  "intermediary_source_version": "0.153.0",
  "intermediary_distribution_version": "0.152.1",
  "intermediary_resolution_method": "release_redirect+git_ls_remote",
  "intermediary_release_http_status": 200,
  "intermediary_tag_ref": "853ddb2dbfab1a0cd6b43b808334fbb4cfc6160d",
  "config_hash": "sha256:<hex>",
  "equivalent": true,
  "allowed_differences": ["batch_reorder"],
  "unexpected_differences": [],
  "normalized_row_count_direct": 10,
  "normalized_row_count_intermediary": 10
}
```

#### Metric Stream Result Row

```json
{
  "fixture_id": "metric_sum_delta_cumulative",
  "metric_name": "queue.length",
  "data_type": "sum",
  "unit": "{item}",
  "temporality": "delta|cumulative",
  "monotonic": true,
  "resource_identity_preserved": true,
  "scope_identity_preserved": true,
  "attribute_set_hash": "sha256:<hex>",
  "merged_with_incompatible_stream": false
}
```

#### AnyValue Rendering Result Row

```json
{
  "fixture_id": "log_complex_body",
  "signal": "logs",
  "field": "body.map.message",
  "raw_value_hash": "sha256:<hex>",
  "typed_anyvalue_preserved": true,
  "collector_as_string_rendering": "contains <tag> & value",
  "parallax_rendering_policy": "typed-json-vN",
  "html_escape_delta_allowed": false,
  "redaction_input_matches_typed_value": true,
  "equivalence_basis": "typed_anyvalue",
  "status": "pass"
}
```

#### Partial Success Result Row

```json
{
  "fixture_id": "partial_reject",
  "signal": "logs",
  "accepted_records": 8,
  "rejected_records": 2,
  "partial_success": true,
  "error_message_present": true,
  "client_should_retry": false,
  "durable_accepted_before_response": true
}
```

#### Retry/Overload Result Row

```json
{
  "fixture_id": "storage_unavailable",
  "failure_mode": "wal_full|storage_down|overload|bad_data",
  "status_code": 503,
  "retryable": true,
  "retry_after_present": true,
  "accepted_records": 0,
  "durable_accepted_before_response": true
}
```

#### Redaction Result Row

```json
{
  "fixture_id": "log_complex_body",
  "surface": "resource|scope|attributes|body",
  "seeded_canaries": 12,
  "leaked_canaries": 0,
  "agent_visible_leaks": 0,
  "useful_context_preserved": true,
  "redaction_policy_version": "a6-default-deny-vN"
}
```

#### Bundle Projection Result Row

```json
{
  "fixture_id": "trace_links_events",
  "signal": "traces",
  "schema_ref_hash": "sha256:<hex>",
  "canonical_bundle_hash": "sha256:<hex>",
  "evidence_edge_hashes": ["sha256:<hex>"],
  "projection_surface": "bundle_json|bundle_markdown|cli_output|http_api|mcp_structuredContent",
  "projection_manifest_hash": "sha256:<hex>",
  "projection_equivalence_hash": "sha256:<hex>",
  "projection_equivalence_pass": true,
  "raw_payload_ref_dereferenced": false,
  "agent_visible_pass": true
}
```

#### MCP Structured Output Result Row

```json
{
  "fixture_id": "log_complex_body",
  "tool_name": "parallax_issue_context",
  "mcp_spec": "2025-11-25",
  "output_schema_hash": "sha256:<hex>",
  "structured_content_hash": "sha256:<hex>",
  "structured_content_valid": true,
  "text_content_is_projection": true,
  "safety_fields_only_in_meta": false,
  "agent_visible_pass": true
}
```

#### Claim Ledger Row

```json
{
  "run_id": "otlp-conformance-YYYYMMDD-N",
  "claim_level": "collector_core_equivalent",
  "claim_status": "pass|fail|expired",
  "version_matrix": {
    "otlp_spec": "1.10.0",
    "opentelemetry_rust": "0.32.0",
    "collector_core_source": "0.153.0",
    "collector_distribution": "0.152.1"
  },
  "bundle_schema_ref_hash": "sha256:<hex>",
  "canonical_bundle_hash": "sha256:<hex>",
  "projection_manifest_hash": "sha256:<hex>",
  "mcp_output_schema_valid": true,
  "product_wording": "Collector-compatible OTLP ingestion for the tested subset.",
  "required_caveats": ["profiles not supported", "HTTP/JSON optional"],
  "expires_at": "YYYY-MM-DD"
}
```

### Counting Rules

- No "OTLP-native" claim without a dated protocol, proto, SDK, Collector, and
  normalizer matrix.
- Baseline transport support means both `grpc` and `http/protobuf` pass for the
  tested signals. `http/json` is optional and must be named separately.
- A JSON-only endpoint is not enough for an OTLP-native or Collector-compatible
  Parallax claim.
- Generic and per-signal OTLP endpoint environment variable behavior must be
  tested because per-signal HTTP endpoints are used as-is by the exporter spec.
- No agent-visible OTLP evidence claim without schema refs, canonical bundle
  hashes, evidence-edge hashes, projection manifests, and access-surface result
  rows for the advertised surfaces.
- CLI JSON, HTTP API JSON, MCP `structuredContent`, Markdown, and persisted
  bundle JSON must agree through canonical hashes for the same fixture. Markdown
  is a projection, not the source of truth.
- MCP counts only when the tool declares an `outputSchema` and returns
  schema-valid `structuredContent`. Text-only MCP output is not agent-ready
  OTLP evidence.
- Safety-critical fields, redaction status, source-field policy status, and
  missing-evidence reports cannot live only in MCP `_meta`; they must be present
  in the canonical bundle or the projection fails.
- Raw OTLP requests and Collector payload refs are denied by default for
  agent-visible projections. A projection result must prove it did not
  dereference raw payload refs.
- The Collector matrix must separate core/source release, runnable distribution
  binary, Contrib distribution, embedded core lineage when known, and config
  hash. A pass on one axis does not imply a pass on another.
- No "Collector-compatible" claim until the official Collector distribution
  equivalence run passes for the advertised signal subset, with Collector
  core/source lineage recorded.
- No "Collector replacement" wording. Parallax supports direct OTLP and
  Collector paths; it does not replace Collector processors, receivers, routing,
  or deployment patterns.
- Profiles remain out of scope until OTLP profile status and Parallax storage
  support are deliberately added.
- HTTP/protobuf and gRPC are required. HTTP/JSON is optional and must be labeled
  explicitly.
- Partial success is not a retry signal; accepted records must be durable before
  a partial-success response.
- Bad data should produce non-retryable behavior; overload and temporary
  downstream failure should produce retryable behavior.
- Metric temporality, monotonicity, unit, data type, resource identity, scope
  identity, and attribute set are part of metric identity.
- Map, array, bytes, and nested OTLP `AnyValue` fields must be compared and
  redacted as typed values. A Collector string-rendering change is not an
  allowed semantic difference unless the run proves the typed value and
  redaction input are unchanged.
- Collector processor changes must be declared by config hash and listed as
  allowed differences. Undeclared field loss is a failed equivalence result.
- GreptimeDB OTLP mapping cannot define Parallax evidence semantics unless the
  conformance run proves that every evidence-critical field survives the mapping.
- Redaction must pass for resource attributes, scope attributes, log bodies, and
  signal attributes before any output is exposed to agents.
- A4 correlation and A6 redaction gates must be current before OTLP trace/log
  joins can count as cross-system reconstruction evidence for agents.

### Refresh Triggers

Rerun the matrix and mark affected claims `claim_expired` when any of these
change:

- OpenTelemetry spec, OTLP spec, proto, or semantic-convention version changes;
- supported OpenTelemetry Rust or `opentelemetry-otlp` release changes;
- official Collector core/source release, Collector distribution/binary release,
  embedded Contrib core lineage, or Collector Contrib release changes;
- Rotel release changes for any Rotel-related claim;
- Parallax parser, normalizer, evidence schema, storage mapping, or WAL
  durability policy changes;
- canonicalization method, bundle schema, projection manifest format, MCP output
  schema, access policy, or advertised projection surfaces change;
- GreptimeDB OTLP mapping changes for fields Parallax depends on;
- redaction policy changes;
- 90 days pass since the last run.

### Product Wording

Allowed after `direct_rust_three_signal`:

> Rust OTLP traces, logs, and metrics ingestion for the tested OpenTelemetry
> Rust SDK/exporter versions.

Allowed after `collector_core_equivalent`:

> Collector-compatible OTLP ingestion for the tested traces/logs/metrics subset.

Allowed after `production_otlp_ingest`:

> Production-ready OTLP ingestion for the tested subset, including direct SDK
> and Collector paths, partial success, retries, overload behavior, duplicate
> delivery, redaction, WAL durability, canonical projection equivalence, and MCP
> structured-output validation.

Avoid:

- "full OpenTelemetry backend";
- "supports all OTLP data";
- "OTLP-native" for JSON-only ingest;
- "Collector replacement";
- "OTLP-native" without a run matrix;
- "Rotel-compatible" beyond the exact Rotel fixture subset.

### Relationship To Other Research

- [OTLP receiver conformance and Collector equivalence](otlp-receiver-conformance-and-collector-equivalence.md)
  defines the fixture scenarios this ledger turns into result rows.
- [OpenTelemetry protocol and context layer](opentelemetry-protocol-and-context-layer.md)
  makes the protocol and Collector/direct-ingest decision.
- [Storage freshness and bundle latency gate](storage-freshness-and-bundle-latency-gate.md)
  measures whether accepted OTLP becomes queryable fast enough.
- [Storage size and object cost gate](storage-size-and-object-cost-gate.md)
  checks retained size and cost for the normalized signal rows.
- [A5 stack decision ledger](a5-stack-decision-ledger.md) should treat this
  ledger as the OTLP integration input for any stack-default claim.
- [A6 redaction red-team ledger](a6-redaction-red-team-ledger.md) controls
  agent-visible safety for OTLP attributes and log bodies.
- [Evidence bundle and open schema](evidence-bundle-and-schema.md) consumes the
  normalized rows and edges protected here.

### Bottom Line

OTLP is the right telemetry substrate, but "OTLP-native" is a conformance claim.
The first honest target is:

> current OpenTelemetry Rust traces, logs, and metrics ingested directly and
> through the official Collector with equivalent normalized evidence rows,
> evidence edges, canonical bundles, and agent-facing projections.

Everything broader waits for dated fixture results.
