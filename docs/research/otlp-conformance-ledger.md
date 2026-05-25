# OTLP Conformance Ledger

<!-- markdownlint-disable MD013 -->

Research date: 2026-05-25

## Purpose

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

## Current Source Snapshot

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
| [Collector core v0.153.0](https://github.com/open-telemetry/opentelemetry-collector/releases/tag/v0.153.0) | Current checked core/source release is `v0.153.0`, published on 2026-05-25. It stabilizes several feature gates, including pdata/proto encoding and ref-counting behavior, and fixes a Snappy memory-corruption issue in gRPC config. Its release body points to collector-releases `v0.153.0` for binaries, but that stable release tag returned 404 in the follow-up check. | Core/source drift can affect payload handling, pdata semantics, compression, or config interpretation. Fixture runs must record Collector source/core separately from the runnable distribution and must not infer a runnable binary from core release notes alone. |
| [Collector distribution v0.152.1](https://github.com/open-telemetry/opentelemetry-collector-releases/releases/tag/v0.152.1) | The official distribution/binary releases API still reported `v0.152.1` on 2026-05-25 while core/source had moved to `v0.153.0`; the stable `v0.153.0` release tag and HTML page returned 404, while the tag list showed only `v0.153.0-nightly.*` tags. The `v0.152.1` line had request-body/decompression fixes and a `pcommon.Value.AsString` behavior change: map/slice values no longer HTML-escape `<`, `>`, and `&`. | Compatibility claims should name the tested binary distribution, release resolution source, and the core/source line. Conformance must preserve typed AnyValue/log-body semantics and not rely on string-rendered equality for redaction or row identity. Nightly distribution tests are development evidence, not stable Collector-equivalence proof. |
| [Collector Contrib v0.152.0](https://github.com/open-telemetry/opentelemetry-collector-contrib/releases/tag/v0.152.0) | Current checked Contrib release is `v0.152.0`, released on 2026-05-11, with broad processor/receiver/exporter changes. | Contrib is the realistic production distribution for many deployments; it needs its own fixture row. |
| [OpenTelemetry Rust 0.32.0](https://docs.rs/crate/opentelemetry/latest) and [opentelemetry-otlp 0.32.0 changelog](https://docs.rs/crate/opentelemetry-otlp/latest/source/CHANGELOG.md) | Docs.rs resolves `opentelemetry` to `0.32.0`; `opentelemetry-otlp` 0.32.0 adds per-signal protocol env vars and OTLP partial-success handling. | Rust fixtures should cover per-signal protocol settings and server partial-success responses. |
| [Rotel v0.2.2](https://github.com/rotel-dev/rotel/releases/tag/v0.2.2) and [Rotel README](https://github.com/rotel-dev/rotel) | Rotel is a Rust OpenTelemetry collector alternative with default OTLP gRPC `4317`, HTTP `4318`, `/v1/traces`, `/v1/metrics`, `/v1/logs`, gzip export, retries/timeouts, and multiple exporters. | Rotel is a useful smoke/eval path, but it is still pre-1.0 and should not replace official Collector equivalence. |
| [GreptimeDB OTLP docs](https://docs.greptime.com/user-guide/ingest-data/for-observability/opentelemetry/) | GreptimeDB supports OTLP/HTTP, but metric ingestion can rename metric/label names, keep only selected resource attributes by default, discard scope attributes by default, and currently does not support ExponentialHistogram. | Parallax must own evidence semantics before storage or prove storage mapping preserves all evidence-critical fields. |
| [MCP tools 2025-11-25](https://modelcontextprotocol.io/specification/2025-11-25/server/tools) and [MCP base protocol 2025-11-25](https://modelcontextprotocol.io/specification/2025-11-25/basic/index) | Tool results can carry JSON `structuredContent`; tools can declare `outputSchema`; MCP uses JSON Schema 2020-12 by default; `_meta` is reserved protocol metadata. | OTLP-derived evidence served to agents must be canonical structured JSON, not text-only output, and safety-critical fields cannot be hidden only in `_meta`. |
| [RFC 8785 JSON Canonicalization Scheme](https://www.rfc-editor.org/rfc/rfc8785.html) | JCS defines deterministic JSON serialization for repeatable hashing/signing. | Bundle and projection equivalence should use canonical JSON hashes, not renderer-specific byte output. |

## Claim Levels

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

## Result Artifacts

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

## Run Manifest

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
    "collector_distribution": "0.152.1",
    "collector_distribution_resolution": "releases_latest=v0.152.1; stable_v0.153.0_release_tag_404; v0.153.0-nightly_tags_visible",
    "collector_contrib": "0.152.0",
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

## Row Schemas

### Fixture Matrix Row

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
  "config_hash": "sha256:<hex>",
  "request_hash": "sha256:<hex>",
  "target_level": "direct_rust_traces"
}
```

### Endpoint Result Row

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

### Normalization Result Row

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

### Equivalence Result Row

```json
{
  "fixture_id": "collector_batch_reorder",
  "direct_request_hash": "sha256:<hex>",
  "intermediary": "collector-core",
  "intermediary_version": "0.152.1",
  "intermediary_source_version": "0.153.0",
  "intermediary_distribution_version": "0.152.1",
  "config_hash": "sha256:<hex>",
  "equivalent": true,
  "allowed_differences": ["batch_reorder"],
  "unexpected_differences": [],
  "normalized_row_count_direct": 10,
  "normalized_row_count_intermediary": 10
}
```

### Metric Stream Result Row

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

### AnyValue Rendering Result Row

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

### Partial Success Result Row

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

### Retry/Overload Result Row

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

### Redaction Result Row

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

### Bundle Projection Result Row

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

### MCP Structured Output Result Row

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

### Claim Ledger Row

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

## Counting Rules

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

## Refresh Triggers

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

## Product Wording

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

## Relationship To Other Research

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

## Bottom Line

OTLP is the right telemetry substrate, but "OTLP-native" is a conformance claim.
The first honest target is:

> current OpenTelemetry Rust traces, logs, and metrics ingested directly and
> through the official Collector with equivalent normalized evidence rows,
> evidence edges, canonical bundles, and agent-facing projections.

Everything broader waits for dated fixture results.
