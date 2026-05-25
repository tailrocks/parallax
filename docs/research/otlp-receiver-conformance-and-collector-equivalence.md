# OTLP Receiver Conformance and Collector Equivalence

<!-- markdownlint-disable MD013 -->

Research date: 2026-05-25

## Purpose

[OpenTelemetry protocol and context layer](opentelemetry-protocol-and-context-layer.md)
sets the direction: Parallax should accept OTLP directly and support upstream
Collectors without making a Collector mandatory for the tiny tier. This note
turns that into a proof gate.

The product claim is not "we have an endpoint on port 4318." The claim is:

> A supported OTLP payload sent directly to Parallax, through the official
> OpenTelemetry Collector, or through Rotel produces equivalent normalized
> evidence rows and bundle edges, modulo explicit pipeline transformations.

If this gate fails, Parallax can still ingest OTLP experimentally, but it should
not call the path OTLP-native.

## Current Primary-Source Checks

| Source | What matters for Parallax |
| --- | --- |
| [OTLP specification 1.10.0](https://opentelemetry.io/docs/specs/otlp/) | OTLP defines encoding, transport, delivery, partial-success behavior, retryable errors, and HTTP/gRPC paths. Partial success is not a retry signal; retryable overload should use the documented retry behavior. |
| [OpenTelemetry proto v1.10.0](https://github.com/open-telemetry/opentelemetry-proto/releases/tag/v1.10.0) | Trace, log, and metric export responses carry per-signal `partial_success` fields with rejected span/log/data-point counts and human-readable error messages. |
| [OpenTelemetry logs data model](https://opentelemetry.io/docs/specs/otel/logs/data-model/) | Logs carry timestamp/observed timestamp, severity, body, attributes, resource/scope context, and optional trace/span correlation. If `SpanId` is present, `TraceId` should be present too. |
| [OpenTelemetry metrics data model](https://opentelemetry.io/docs/specs/otel/metrics/data-model/) | Metric stream identity includes resource attributes, instrumentation scope, metric name, data point type, unit, temporality, and monotonicity. Attribute sets identify individual streams. Parallax must not flatten this into ambiguous rows. |
| [OpenTelemetry trace API](https://opentelemetry.io/docs/specs/otel/trace/api/) | Spans, links, events, status, attributes, span context, trace ID, and span ID are the core lifecycle evidence for Parallax correlation. |
| [OpenTelemetry Collector configuration](https://opentelemetry.io/docs/collector/configuration/) | Collector configs are receiver/processor/exporter/connectors plus service pipelines. Defining a receiver does not enable it until a pipeline references it. Standard OTLP examples use gRPC `4317` and HTTP `4318`. |
| [OpenTelemetry Collector v0.152.1](https://github.com/open-telemetry/opentelemetry-collector/releases/tag/v0.152.1) | Latest official Collector release checked for this pass. It is the compatibility reference for production OTLP pipelines. |
| [OpenTelemetry Collector Contrib v0.152.0](https://github.com/open-telemetry/opentelemetry-collector-contrib/releases/tag/v0.152.0) | Contrib is the realistic production distribution for broader processors/receivers/exporters. Parallax should verify both core and contrib where pipeline components differ. |
| [OpenTelemetry Rust 0.32.0](https://github.com/open-telemetry/opentelemetry-rust/releases/tag/opentelemetry-0.32.0) | Latest Rust release checked. Rust SDK fixtures are the first direct-SDK path because Parallax is Rust-first. |
| [Rotel v0.2.2](https://github.com/rotel-dev/rotel/releases/tag/v0.2.2) and [Rotel README](https://github.com/rotel-dev/rotel) | Rotel supports metrics/logs/traces, OTLP gRPC, OTLP HTTP/protobuf, OTLP HTTP/JSON, OTLP export, batching, retries, and resource attributes, with default receiver paths on `4317`/`4318`. It is promising but early and must be a smoke/eval path, not the compatibility baseline. |
| [GreptimeDB OTLP docs](https://docs.greptime.com/user-guide/ingest-data/for-observability/opentelemetry/) | GreptimeDB can consume OTLP/HTTP, but its metric mapping can rename metrics/labels and discard some resource/scope attributes by default in Prometheus-compatible mode. Parallax must own normalization before storage or configure storage ingestion deliberately. |

## Compatibility Levels

| Level | Meaning | Product wording |
| --- | --- | --- |
| L0 endpoint | OTLP/gRPC `4317`, OTLP/HTTP `4318`, `/v1/traces`, `/v1/logs`, `/v1/metrics`, binary protobuf, gzip, size limits, and stable error responses work. | "OTLP endpoint." |
| L1 direct Rust SDK | Current OpenTelemetry Rust traces, logs, and metrics reach Parallax directly and normalize into queryable rows. | "Rust OTLP ingestion." |
| L2 signal semantics | Trace, log, and metric fixtures preserve resource, scope, trace/span IDs, status, links/events, log bodies, metric temporality, histograms, and attributes. | "OTLP-native telemetry ingestion." |
| L3 Collector equivalence | Same fixtures through official Collector and Collector Contrib produce equivalent normalized rows, except declared processor-added/removed fields. | "Collector-compatible OTLP ingestion." |
| L4 Rotel equivalence | Same fixtures through Rotel produce equivalent normalized rows for supported receiver/exporter modes. | "Rotel-compatible OTLP ingestion." |
| L5 production pipeline | Redaction, batching, retries, partial success, overload, idempotency, and storage-failure behavior are proven under mixed signal load. | "Production-ready OTLP ingestion." |

The v0 target is **L1 + L2 + L3** for a narrow signal subset. L4 is a smoke gate
because Rotel is still pre-1.0. L5 is required before broad public claims.

## Fixture Generation Strategy

Fixtures should be generated from real SDKs and collectors, not handwritten
protobuf blobs:

```text
fixture app / sdk version / signal scenario
  -> direct OTLP/gRPC to Parallax
  -> direct OTLP/HTTP protobuf to Parallax
  -> OpenTelemetry Collector -> Parallax
  -> Collector Contrib -> Parallax
  -> Rotel -> Parallax
  -> raw payload hash + normalized row snapshots
  -> bundle edge snapshots
```

Each fixture directory should record:

- SDK/exporter name and version;
- runtime version;
- transport (`grpc`, `http/protobuf`, optional `http/json`);
- intermediary (`none`, `collector-core`, `collector-contrib`, `rotel`);
- Collector/Rotel config;
- raw request hash before and after intermediary;
- expected accepted and rejected counts;
- normalized row snapshot;
- evidence-edge snapshot;
- redaction report snapshot when attributes/log bodies include canaries.

## Fixture Matrix

| Fixture | Must prove |
| --- | --- |
| `trace_basic_tree` | Root/child spans preserve `trace_id`, `span_id`, parent ID, timestamps, duration, kind, status, attributes, resource, and scope. |
| `trace_links_events` | Span links and events are stored as evidence, not silently dropped. |
| `trace_exception_attrs` | Exception semantic attributes can support error correlation without replacing Sentry error events. |
| `log_correlated` | Log record with `trace_id` and `span_id` joins to the matching span. |
| `log_uncorrelated` | Log record without trace context is accepted but never promoted to a strong edge by time proximity alone. |
| `log_complex_body` | String, map, array, bytes, and numeric bodies either normalize safely or receive explicit unsupported-field outcomes. |
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
| `json_http_optional` | OTLP/HTTP JSON works only if explicitly supported; otherwise rejected with clear non-retryable behavior. |
| `malformed_payload` | Invalid protobuf/JSON fails before expensive processing and does not poison subsequent requests. |
| `partial_reject` | Oversized or policy-rejected records return correct partial-success counts and messages. |
| `storage_unavailable` | Temporary storage/WAL outage returns retryable overload behavior without accepting data that is not durable. |
| `duplicate_delivery` | Retried batches do not duplicate normalized rows or issue/event edges. |

## Normalization Equivalence Rules

Equivalence should be set-based, not byte-for-byte:

- OTLP intermediaries may batch, split, or reorder telemetry.
- Collector processors may intentionally add, remove, or transform attributes.
- Resource and scope identities must remain explicit.
- Trace/log correlation keys must not be lost.
- Metric stream identity must include data type, unit, temporality, and
  monotonicity.
- Raw payload refs should record both the received Parallax payload and, when
  available, the original SDK payload before an intermediary changed it.

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

## Collector And Rotel Test Configs

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

## Endpoint Behavior Requirements

Parallax's OTLP receiver should expose and test:

| Behavior | Requirement |
| --- | --- |
| Ports | gRPC `4317`; HTTP `4318`. |
| Paths | `/v1/traces`, `/v1/logs`, `/v1/metrics`; optional base-path alias only if documented. |
| Content types | `application/x-protobuf` required; JSON protobuf optional and explicitly labeled. |
| Compression | gzip for HTTP and gRPC where SDKs/collectors use it. |
| Payload limits | Reject oversized requests before full decode. |
| Partial success | Accepted records are durable; rejected counts are accurate; sender should not retry accepted records. |
| Retryable errors | Temporary overload/storage failure returns retryable status plus `Retry-After` where possible. |
| Non-retryable errors | Malformed payload, auth failure, unsupported content type, and permanently invalid records return non-retryable status. |
| Durability | Do not acknowledge accepted telemetry before local WAL/outbox durability in the tiny tier. |
| Audit | Record source (`direct`, `collector-core`, `collector-contrib`, `rotel`) and config hash when known. |

## Pass / Fail Gate

Pass only when:

- current OpenTelemetry Rust fixtures pass over OTLP/gRPC and OTLP/HTTP
  protobuf;
- the same fixture set passes through official Collector `v0.152.1`;
- the same fixture set passes through Collector Contrib `v0.152.0` for the
  configured components Parallax recommends;
- Rotel `v0.2.2` smoke fixtures pass for the supported subset or any
  differences are documented and not product-blocking;
- direct and Collector paths produce equivalent normalized rows and evidence
  edges;
- `trace_id`, `span_id`, `service.name`, `service.version`, deployment
  environment, resource attrs, scope attrs, and metric temporality are not lost;
- redaction canaries in attributes, log bodies, and resource fields are removed
  from agent-visible JSON/Markdown;
- retry, partial success, and duplicate delivery behavior is deterministic.

Fail or narrow the claim if:

- Parallax requires a Collector for the tiny tier;
- Collector forwarding changes supported fields without the diff being
  explicitly caused by configured processors;
- metrics lose temporality/monotonicity or merge incompatible streams;
- logs with trace context fail to join spans;
- GreptimeDB's OTLP mapping drops evidence-critical fields and Parallax has no
  normalization layer to compensate;
- partial-success responses cause SDK/Collector retry loops or duplicate rows;
- unsupported payloads are silently accepted and then disappear.

## Product Implication

The honest first release wording should be:

> OTLP-native for a tested subset of current Rust SDK traces, logs, and metrics,
> with direct SDK and official Collector equivalence.

Avoid:

- "full OpenTelemetry backend";
- "Collector replacement";
- "supports all OTLP data";
- "Rotel-based ingestion" as a default claim before Rotel-specific equivalence
  and overhead tests pass.

This keeps the tiny tier simple while making the production Collector path real.

## Relationship To Other Research

- [OpenTelemetry protocol and context layer](opentelemetry-protocol-and-context-layer.md)
  makes the protocol decision; this note defines the conformance proof.
- [Self-hosted simplicity gate](self-hosted-simplicity-gate.md) requires OTLP
  ingestion without an external Collector in the tiny tier.
- [Storage freshness and bundle latency gate](storage-freshness-and-bundle-latency-gate.md)
  measures whether accepted OTLP becomes queryable fast enough.
- [Redaction pipeline and secret safety](redaction-pipeline-and-secret-safety.md)
  has veto power over OTLP attributes and log bodies before agent exposure.
- [Evidence bundle and open schema](evidence-bundle-and-schema.md) consumes the
  normalized rows and evidence edges this gate protects.

## Bottom Line

OTLP is the right protocol substrate, but OTLP-native is a conformance claim.
Parallax should earn it with direct-SDK, Collector, Collector Contrib, and Rotel
fixtures that prove equivalent normalized evidence. Otherwise it risks building
another "accepts protobuf" endpoint that loses the exact fields agents need.
