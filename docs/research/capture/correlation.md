# A4 — Correlation Reliability on Real Telemetry

> A4 is the bear-case assumption that deterministic cross-signal correlation is reliable in real, messy telemetry, and it remains an open validation gate: it can pass only through a row-level audit of real (`real_pilot`) telemetry, never on aggregate rates or generator-perfect data. The gate defines what to measure — strong-edge prevalence, trace/log/release/deploy coverage, trace-context validity and scope consistency, frontend-to-backend continuation, async links, baggage privacy, projection equivalence, a manual false-strong-edge audit, and complete missing-evidence reporting — with first-pass pass targets for the Rust backend/tiny-tier wedge (e.g. backend `trace_context_rate` >= 80%, `error_in_span_rate` >= 70%, `false_strong_edge_rate` <= 5%, `missing_evidence_report_rate` = 100%). The ledger defines the proof artifact required before any aggregate can count: a per-run manifest, per-anchor rows, manual audit rows, instrumentation repair rows, claim levels, and freshness/rerun rules. If strong edges are common and the same safe canonical bundle survives CLI/API/MCP projection, Parallax can claim evidence-backed reconstruction for the target wedge; if strong edges are rare or projections diverge, the product must become honest best-effort context plus tooling that helps users fix instrumentation and access-surface gaps. A failed A4 gate is itself useful — it tells Parallax where instrumentation, setup diagnostics, and honest product wording must change.

This note consolidates the following previously-separate research files, each preserved in full below:

- `correlation-reliability-real-telemetry-gate.md`
- `a4-correlation-reliability-ledger.md`

## Correlation Reliability on Real Telemetry Gate

_Provenance: merged verbatim from `correlation-reliability-real-telemetry-gate.md` (2026-05-29 restructure)._

<!-- markdownlint-disable MD013 -->

Research date: 2026-05-25

### Purpose

This operationalizes bear-case assumption A4:

> Deterministic cross-signal correlation is reliable in real, messy telemetry.

The synthetic benchmark can generate perfect `trace_id`, `span_id`, frontend
continuation, release markers, and error links. Real systems do not. They have
missing trace context, inconsistent sampling, broken browser CORS propagation,
async queue boundaries without span links, logs without trace fields, clock skew,
deployment metadata gaps, and manually emitted errors that never entered an
active span.

This gate measures whether Parallax's evidence graph has enough strong edges in
real telemetry to justify the product claim. If it fails, Parallax can still be a
useful error/context store, but it must stop promising lifecycle reconstruction.
The [A4 correlation reliability ledger](a4-correlation-reliability-ledger.md)
defines the row-level result artifact required before any aggregate result can
count as proof.

### Source Posture

Current primary references support correlation as a standard mechanism, but also
show why it must be measured:

- W3C Trace Context defines the interoperable `traceparent` and `tracestate`
  headers; `traceparent` carries trace identity and sampling flags across system
  boundaries
  ([W3C Trace Context](https://www.w3.org/TR/trace-context/),
  [W3C Trace Context Level 2](https://www.w3.org/TR/trace-context-2/)).
- OpenTelemetry's trace API treats a span context as valid only when both
  `TraceId` and `SpanId` are non-zero, and its propagator API requires W3C trace
  context propagators to parse and validate `traceparent`/`tracestate`
  ([OpenTelemetry trace API](https://opentelemetry.io/docs/specs/otel/trace/api),
  [OpenTelemetry propagators API](https://opentelemetry.io/docs/specs/otel/context/api-propagators/)).
- OpenTelemetry logs explicitly support correlation by time, execution/trace
  context, and resource context; direct trace/log joins depend on `TraceId` and
  `SpanId` being present in log records
  ([OpenTelemetry logs](https://opentelemetry.io/docs/specs/otel/logs/),
  [Trace context in non-OTLP logs](https://opentelemetry.io/docs/specs/otel/compatibility/logging_trace_context/)).
- OpenTelemetry sampling guidance says head sampling cannot select traces based
  on later error or latency outcomes; error or slow-trace retention often needs
  tail sampling in the Collector
  ([OpenTelemetry sampling](https://opentelemetry.io/docs/concepts/sampling/),
  [OpenTelemetry trace SDK](https://opentelemetry.io/docs/specs/otel/trace/sdk/)).
- OpenTelemetry's agent-to-gateway deployment docs warn that tail sampling only
  makes accurate decisions when all spans for a trace reach the same Collector
  instance; trace-ID routing is possible but advanced
  ([OpenTelemetry agent-to-gateway deployment](https://opentelemetry.io/docs/collector/deploy/other/agent-to-gateway/)).
- Sentry SDK tracing propagates `sentry-trace` and `baggage`, and can optionally
  propagate W3C `traceparent` for OpenTelemetry interoperability when configured
  and allowed by `tracePropagationTargets`
  ([Sentry trace propagation SDK spec](https://develop.sentry.dev/sdk/foundations/trace-propagation/)).
- OpenTelemetry baggage can propagate arbitrary key/value data and its docs warn
  that sensitive baggage can be shared with unintended resources such as
  third-party APIs
  ([OpenTelemetry baggage](https://opentelemetry.io/docs/concepts/signals/baggage/)).

Internal sources:

- [Causal reconstruction and agent safety](causal-reconstruction-and-agent-safety.md)
  says time correlation is weak and deterministic edges must be labeled.
- [Frontend collection and cross-tier correlation](frontend-collection-and-cross-tier-correlation.md)
  identifies CORS, sampling coherence, and browser gaps as the cross-tier failure
  modes.
- [Frontend capture safety ledger](frontend-capture-safety-ledger.md) defines
  the browser/route, source-map, CORS, privacy, export, canonical hash,
  projection-manifest, and MCP structured-output rows that must be fresh before
  frontend anchors can support product wording.
- [Evidence bundle and open schema](evidence-bundle-and-schema.md) already
  distinguishes strong, medium, weak, and inferred edges, and requires
  `schema_ref`, `canonical_hash`, `projection_manifest`, and access fields for
  agent-visible CLI/API/MCP bundles.

### What To Measure

Run this against real or pilot telemetry, not the generator. Start with
operator-owned services, then design partners from the
[user interview gate](user-interview-and-deployment-intent-gate.md).

Measure by anchor class:

| Anchor class | Minimum sample |
| --- | --- |
| Backend error events | 100 events or all available recent events. |
| Frontend errors | 50 events from instrumented first-party routes. |
| CI/test failures | 50 failures with logs and commit metadata. |
| CLI invocations | 50 failed or nonzero-exit commands. |
| Agent sessions | 25 sessions with commands/tests/outcomes. |

For each anchor, build the bundle using only evidence actually present in the
telemetry and metadata stores. Do not backfill synthetic links for the audit.
For any agent-visible claim, also emit the canonical bundle JSON plus projection
hashes for file, CLI, HTTP, and MCP output paths; otherwise A4 has measured
correlation rows but has not proven safe agent context.

### Core Metrics

| Metric | Definition |
| --- | --- |
| `trace_context_rate` | Fraction of anchors with usable `trace_id`; for span-scoped anchors, also `span_id`. |
| `trace_context_validity_rate` | Fraction of anchors with present context whose `trace_id`/`span_id`, `trace_flags`, and `tracestate` parse as valid according to the source protocol. Invalid IDs are missing evidence, not usable joins. |
| `same_trace_bundle_rate` | Fraction of anchors where Q1 trace-context query returns spans/logs/errors beyond the anchor itself. |
| `log_trace_join_rate` | Fraction of relevant log excerpts carrying `trace_id` or `span_id`. |
| `error_in_span_rate` | Fraction of error events that can be placed inside a span by `trace_id` + `span_id` or equivalent Sentry trace context. |
| `frontend_backend_continuation_rate` | Fraction of first-party frontend errors/requests whose trace continues into backend spans. |
| `async_link_rate` | Fraction of queue/background workflow anchors with span links or explicit message/job IDs. |
| `trace_scope_consistency_rate` | Fraction of strong same-trace joins whose project, environment, service/resource, and tenant scope match the anchor or have an explicit cross-scope edge. |
| `sampling_state_explained_rate` | Fraction of anchors with missing span bodies where `TraceFlags`, SDK sampler, collector policy, or tail-sampling route explains whether the trace was sampled, dropped, or fragmented. |
| `baggage_privacy_pass_rate` | Fraction of anchors carrying baggage/session context where only allowlisted opaque values are present and no raw user, account, token, email, or third-party target value is propagated. |
| `release_context_rate` | Fraction of anchors attachable to release/version/commit metadata. |
| `deploy_context_rate` | Fraction attachable to a deploy marker or rollout window. |
| `release_commit_rate` | Fraction of release markers with exact commit SHA or source revision. |
| `deploy_success_status_rate` | Fraction of deploy markers with terminal success/failure/error status. |
| `compare_base_rate` | Fraction of release/deploy windows with predecessor base available for code-change comparison. |
| `strong_edge_count_p50` | Median count of deterministic strong edges per bundle. |
| `weak_only_bundle_rate` | Fraction of bundles with no strong or medium edges. |
| `false_strong_edge_rate` | Manual-audit rate where a strong edge is structurally present but semantically wrong because of instrumentation bugs. |
| `missing_evidence_report_rate` | Fraction of bundles that correctly list every expected missing evidence category. |
| `projection_equivalence_rate` | Fraction of agent-visible bundles whose canonical JSON, CLI output, HTTP API result, and MCP `structuredContent` carry the same post-redaction canonical bundle hash and complete safety fields. |

Expected missing-evidence categories:

- `missing_trace_id`
- `missing_span_id`
- `missing_log_trace_context`
- `missing_backend_continuation`
- `missing_release`
- `missing_release_commit`
- `missing_deploy`
- `missing_deploy_status`
- `missing_deploy_environment`
- `missing_predecessor_release`
- `missing_compare_base`
- `pr_file_list_truncated`
- `missing_issue_tracker_link`
- `missing_source_map`
- `missing_async_link`
- `sampled_out_trace`
- `invalid_trace_context`
- `unparsed_tracestate`
- `trace_scope_mismatch`
- `missing_sampling_policy`
- `tail_sampling_route_unverified`
- `unsafe_baggage`
- `duplicate_span_identity`
- `clock_skew_suspected`
- `redaction_removed_required_field`
- `missing_canonical_bundle_hash`
- `projection_hash_mismatch`
- `mcp_structured_content_missing`
- `mcp_output_schema_invalid`

### Manual Audit

Automatically computed edge rates are not enough. Manually review at least 20
bundles per pilot and classify:

| Question | Why |
| --- | --- |
| Does every strong edge follow from a deterministic key, not time proximity? | Prevents laundering weak correlation into strong evidence. |
| Does the trace path actually include the failing operation? | Catches wrong propagation, duplicate spans, or unrelated spans in the same trace. |
| Are the trace context fields valid and scoped to the same project/environment/resource boundary? | Prevents invalid IDs or cross-tenant/resource collisions from becoming strong edges. |
| Are sampling gaps explained by `TraceFlags`, SDK sampler, collector policy, or tail-sampling route evidence? | Prevents sampled-out or fragmented traces from being mistaken for complete lifecycle evidence. |
| Does propagated baggage contain only allowlisted opaque values and stay on first-party paths? | Prevents user/session correlation from becoming a PII leak or third-party propagation bug. |
| Are medium release/deploy edges contradicted by first-seen history? | Prevents false regression claims. |
| Are missing-data gaps explicit? | Agents must know when evidence is absent. |
| Would the bundle lead a human toward the same next investigation step? | Tests practical usefulness before A1 agent evals. |

Record reviewer, date, bundle id, verdict, false-positive edges, false-negative
missing-data flags, and recommended instrumentation fixes.

### Pass Targets

Initial targets for the Rust backend/tiny-tier wedge:

| Gate | Target |
| --- | --- |
| Backend `trace_context_rate` | >= 80 percent for instrumented Rust services. |
| Backend `trace_context_validity_rate` | >= 99 percent for anchors with present trace context. |
| Backend `error_in_span_rate` | >= 70 percent for instrumented Rust services. |
| `same_trace_bundle_rate` | >= 70 percent for backend error anchors. |
| `trace_scope_consistency_rate` | 100 percent for strong edges exposed to agents. |
| `sampling_state_explained_rate` | 100 percent for anchors with missing expected span bodies. |
| `release_context_rate` | >= 90 percent for production error anchors. |
| `deploy_context_rate` | >= 70 percent where deploy markers are configured. |
| `strong_edge_count_p50` | >= 2 strong edges per backend error bundle. |
| `weak_only_bundle_rate` | <= 20 percent for the target MVP anchor classes. |
| `false_strong_edge_rate` | <= 5 percent in manual audit. |
| `missing_evidence_report_rate` | 100 percent for expected categories. |
| `projection_equivalence_rate` | 100 percent for agent-visible bundles. |

Separate frontend/cross-tier target:

| Gate | Target |
| --- | --- |
| `frontend_backend_continuation_rate` | >= 60 percent for instrumented first-party API calls after CORS/header configuration. |
| Silent propagation failure detection | 100 percent of frontend spans with no backend continuation are flagged as missing evidence, not treated as no backend involvement. |
| `baggage_privacy_pass_rate` | 100 percent for frontend/session baggage before cross-tier evidence is agent-visible. |

Separate async target:

| Gate | Target |
| --- | --- |
| `async_link_rate` | >= 50 percent for instrumented queue/background workflows before claiming async lifecycle reconstruction. |

These are first-pass targets. If real users cluster in lower-instrumentation
environments, lower the product claim before lowering the safety bar.

### Failure Consequences

| Failure | Product consequence |
| --- | --- |
| Backend strong-edge gates fail | Narrow MVP to Sentry-compatible grouping + release context + best-effort trace/log links. |
| Frontend continuation fails | Keep frontend capture as session/error evidence, but do not claim frontend-to-backend reconstruction. Ship a CORS/propagation diagnostic first. |
| Async links fail | Treat queues/background jobs as separate lifecycle anchors unless explicit message IDs or span links are present. |
| Trace context validity or scope consistency fails | Exclude affected edges from strong-edge counts and block agent-visible bundles until the propagator/resource mapping is fixed. |
| Sampling state is unexplained | Mark spans/logs as missing evidence and avoid "complete trace" language for the affected anchor class. |
| Baggage privacy fails | Disable baggage-derived session joins and treat the run as a redaction/privacy failure, not an A4 pass. |
| False strong edges exceed target | Downgrade the edge class or fix instrumentation before agent exposure. |
| Missing evidence is not reported | Block agent-visible bundles; absence reporting is a safety invariant. |
| Projection equivalence fails | Keep the correlation result as internal measurement only; do not expose the bundle through CLI/API/MCP or agent prompts until canonical hashes and safety fields match. |
| Release/deploy context missing | Do not rank release-regression hypotheses above medium/weak. |

The goal is not to force a pass. A failed A4 gate is useful: it tells Parallax
where instrumentation, setup diagnostics, and honest product wording must change.

### Instrumentation Fixes To Try Before Declaring Failure

- Ensure Sentry events carry trace context and map to OTLP `trace_id`/`span_id`.
- Reject invalid or all-zero trace/span IDs before edge construction, and record
  the source propagator that parsed `traceparent`, `tracestate`, `sentry-trace`,
  or non-OTLP log fields.
- Use parent-based sampling for end-to-end trace consistency, or collector tail
  sampling that keeps error/slow traces.
- Route all spans of one trace to the same tail-sampling collector when using a
  gateway topology.
- Record sampler and collector policy evidence when a trace is missing expected
  spans; do not infer completeness from a sampled flag alone.
- Add `trace_id`/`span_id` injection into structured logs and non-OTLP log
  formats.
- Add browser `traceparent` propagation allowlists and backend CORS headers for
  `traceparent`, `tracestate`, and `baggage`.
- Allowlist baggage keys and make session/account values opaque, scoped, and
  non-PII before using baggage for frontend or cross-service joins.
- Add explicit message IDs or span links at queue boundaries.
- Emit release/version and deploy markers from CI/CD.
- Add a `parallax doctor correlation` command that checks SDK propagation,
  browser CORS headers, sampler configuration, log context, and deploy metadata.

### Result Record

Store aggregate results in a Markdown note and machine-readable JSON. The row
schema, run manifest, manual audit rows, instrumentation repair rows, claim
levels, and refresh rules are defined in the
[A4 correlation reliability ledger](a4-correlation-reliability-ledger.md).

```json
{
  "audit_date": "2026-05-25",
  "project": "proj_checkout",
  "anchor_class": "backend_error",
  "sample_size": 100,
  "trace_context_rate": 0.84,
  "same_trace_bundle_rate": 0.76,
  "log_trace_join_rate": 0.68,
  "error_in_span_rate": 0.72,
  "release_context_rate": 0.96,
  "deploy_context_rate": 0.74,
  "strong_edge_count_p50": 2,
  "weak_only_bundle_rate": 0.14,
  "false_strong_edge_rate": 0.03,
  "missing_evidence_report_rate": 1.0,
  "projection_equivalence_rate": 1.0,
  "verdict": "pass_backend_mvp"
}
```

Recommended future doc:

```text
docs/research/correlation-reliability-results.md
```

That results note should link to the per-run ledger artifacts and separate real
telemetry from synthetic/fault-injected telemetry. Do not use generator-perfect
data to pass A4.

### Relationship To Other Research

- [Causal reconstruction and agent safety](causal-reconstruction-and-agent-safety.md)
  defines the edge-strength model this gate measures.
- [Frontend collection and cross-tier correlation](frontend-collection-and-cross-tier-correlation.md)
  defines the browser-to-backend propagation path and its failure modes.
- [Frontend capture safety ledger](frontend-capture-safety-ledger.md) defines
  the browser-side source-map, CORS, privacy, export, overhead, canonical hash,
  and projection rows consumed by frontend correlation claims.
- [Evidence bundle and open schema](evidence-bundle-and-schema.md) defines the
  edge and missing-evidence fields that must be audited.
- [A4 correlation reliability ledger](a4-correlation-reliability-ledger.md)
  defines the public run artifacts, row schemas, claim levels, and refresh rules.
- [Bundle-value evaluation](bundle-value-evaluation.md) depends on this gate: an
  agent eval over unrealistically perfect links would overstate the product.
- [User interview and deployment intent gate](user-interview-and-deployment-intent-gate.md)
  can recruit teams with real incidents for the audit.
- [OpenTelemetry protocol and context layer](opentelemetry-protocol-and-context-layer.md)
  defines the base trace/log/resource fields.
- [Deploy, change, and issue-tracker context](deploy-change-and-issue-context.md)
  defines the release/deploy/code-change/work-item edge semantics measured here.

### Bottom Line

A4 is true only if real telemetry contains enough deterministic edges. Measure
that before marketing lifecycle reconstruction. If strong edges are common,
and the same safe canonical bundle survives CLI/API/MCP projection, Parallax can
claim evidence-backed reconstruction for the target wedge. If strong edges are
rare or projections diverge, the product must become honest best-effort context
plus tooling that helps users fix instrumentation and access-surface gaps.

## A4 Correlation Reliability Ledger

_Provenance: merged verbatim from `a4-correlation-reliability-ledger.md` (2026-05-29 restructure)._

<!-- markdownlint-disable MD013 -->

Research date: 2026-05-25

### Purpose

The [correlation reliability gate](correlation-reliability-real-telemetry-gate.md)
defines the metrics for bear-case assumption A4:

> Deterministic cross-signal correlation is reliable in real, messy telemetry.

This note defines the result ledger needed to make that gate auditable. A4 must
not pass on aggregate rates alone. Every run needs per-anchor rows, manual audit
rows, instrumentation repair records, and a claim-freshness state so future
research can tell exactly what was measured and what product wording is allowed.

### Current Primary-Source Checks

Primary sources support trace/log correlation, but they also show why Parallax
has to keep the evidence granular:

- W3C Trace Context defines the `traceparent` and `tracestate` propagation model.
  When a valid `traceparent` is received, downstream systems update the
  `parent-id` and propagate the trace; invalid trace fields can force a new
  trace and discard `tracestate`
  ([W3C Trace Context](https://www.w3.org/TR/trace-context/)).
- OpenTelemetry's trace API says a valid span context requires non-zero
  `TraceId` and `SpanId`, and the propagator API requires W3C trace context
  propagators to parse and validate `traceparent`/`tracestate`
  ([OpenTelemetry trace API](https://opentelemetry.io/docs/specs/otel/trace/api),
  [OpenTelemetry propagators API](https://opentelemetry.io/docs/specs/otel/context/api-propagators/)).
- OpenTelemetry's model treats `TraceId`, `SpanId`, and `TraceFlags` as explicit
  log-record fields, with `SpanId` implying `TraceId`; the non-OTLP compatibility
  spec maps those fields to `trace_id`, `span_id`, and `trace_flags`
  ([OpenTelemetry logs data model](https://opentelemetry.io/docs/specs/otel/logs/data-model/),
  [Trace context in non-OTLP logs](https://opentelemetry.io/docs/specs/otel/compatibility/logging_trace_context/)).
- OpenTelemetry sampling guidance separates head sampling from tail sampling:
  head sampling cannot know later error/latency outcomes, while tail sampling can
  use completed-trace criteria but is more operationally complex
  ([OpenTelemetry sampling](https://opentelemetry.io/docs/concepts/sampling/)).
- OpenTelemetry Collector deployment guidance says tail sampling is accurate
  only when all spans for a trace reach the same Collector instance; trace-ID
  routing can support this but needs careful testing
  ([OpenTelemetry agent-to-gateway](https://opentelemetry.io/docs/collector/deploy/other/agent-to-gateway/)).
- OpenTelemetry span links are the standard way to represent causal relations
  across asynchronous or batch work when parent/child trace topology is not
  enough
  ([OpenTelemetry overview](https://opentelemetry.io/docs/specs/otel/overview/)).
- Sentry's SDK trace propagation spec requires `sentry-trace` and `baggage`
  propagation, gates outgoing headers by `tracePropagationTargets`, and can send
  W3C `traceparent` for OpenTelemetry interoperability only when configured
  ([Sentry trace propagation](https://develop.sentry.dev/sdk/foundations/trace-propagation/)).
- OpenTelemetry baggage can propagate arbitrary key/value context and its docs
  warn that sensitive baggage can reach unintended downstream resources such as
  third-party APIs
  ([OpenTelemetry baggage](https://opentelemetry.io/docs/concepts/signals/baggage/)).
- MCP `2025-11-25` separates human-readable text content from JSON
  `structuredContent` and lets tools advertise an `outputSchema`; RFC 8785/JCS
  provides deterministic, hashable JSON
  ([MCP tools specification](https://modelcontextprotocol.io/specification/2025-11-25/server/tools),
  [RFC 8785](https://www.rfc-editor.org/rfc/rfc8785.html)).

Internal sources:

- [Correlation reliability on real telemetry gate](correlation-reliability-real-telemetry-gate.md)
  defines the metrics and pass/fail thresholds.
- [Frontend collection and cross-tier correlation](frontend-collection-and-cross-tier-correlation.md)
  defines browser-to-backend propagation and CORS failure modes.
- [Frontend capture safety ledger](frontend-capture-safety-ledger.md) supplies
  browser-side source-map, CORS, privacy, export, overhead, and projection rows
  consumed by frontend correlation claims.
- [Deploy, change, and issue-tracker context](deploy-change-and-issue-context.md)
  defines release, deploy, code-change, and work-item edge strengths.
- [Evidence bundle and open schema](evidence-bundle-and-schema.md) defines
  strong, medium, weak, inferred, and missing-evidence fields.

### Why A Ledger Is Required

Aggregate rates hide the failure modes that matter most for Parallax:

- one service may have strong trace/log joins while another silently drops
  `traceparent`;
- invalid, all-zero, or unparsable context fields can look like deterministic
  joins if the run only checks that a string exists;
- sampling can make an error appear trace-linked while the useful span body is
  missing;
- frontend traces can exist without backend continuation because CORS stripped
  propagation headers;
- baggage can make cross-tier/session joins easier while also propagating raw
  user, tenant, account, or token-like values beyond the intended boundary;
- a deploy edge can be present but point to the wrong release window;
- a strong edge can be structurally valid but semantically wrong because an SDK
  or collector duplicated, rewrote, or mixed context.

The ledger is therefore the proof artifact. The A4 summary can say "pass" only
when the underlying rows show that the result came from real telemetry and that
manual audit did not find false strong edges above the threshold.

### Artifact Set

Use this public, redacted layout once real runs begin:

```text
docs/research/correlation-reliability-results.md
docs/research/correlation-reliability-runs/<run_id>/manifest.json
docs/research/correlation-reliability-runs/<run_id>/anchor-ledger.jsonl
docs/research/correlation-reliability-runs/<run_id>/manual-audit.jsonl
docs/research/correlation-reliability-runs/<run_id>/projection-audit.jsonl
docs/research/correlation-reliability-runs/<run_id>/instrumentation-repairs.jsonl
docs/research/correlation-reliability-runs/<run_id>/hashes.sha256
```

The Markdown summary is the human-readable verdict. The JSON/JSONL files are the
audit trail. Do not commit raw stack traces, log bodies, customer identifiers,
full trace IDs, source-map content, replay data, or private incident notes. Use
stable redacted IDs and salted or keyed hashes when a deterministic join needs to
be auditable without exposing the original value.

### Run Manifest

Each run gets exactly one manifest:

```json
{
  "schema_version": "a4-correlation-ledger-v1",
  "run_id": "a4-2026-05-25-operator-api",
  "research_date": "2026-05-25",
  "dataset_kind": "real_pilot",
  "project_label": "operator-api",
  "environment": "production",
  "sample_window_start": "2026-05-18T00:00:00Z",
  "sample_window_end": "2026-05-25T00:00:00Z",
  "anchor_classes": ["backend_error", "frontend_error"],
  "telemetry_sources": ["sentry_envelope", "otlp_traces", "otlp_logs", "deploy_markers"],
  "sdk_versions": {
    "sentry_rust": "x.y.z",
    "opentelemetry_rust": "x.y.z",
    "opentelemetry_js": "x.y.z"
  },
  "propagators": ["tracecontext", "baggage", "sentry_trace"],
  "trace_context_validator_version": "a4-trace-context-v1",
  "project_scope_fields": ["project_id", "environment", "service.name"],
  "collector_topology": "agent_to_single_gateway_tail_sampling",
  "collector_instance_count": 1,
  "tail_sampling_trace_routing_verified": true,
  "sampling_policy": "tail_sample_errors_and_10pct_other",
  "log_trace_context_format": "otel_log_fields",
  "baggage_policy": "first_party_opaque_allowlist_v1",
  "bundle_schema_ref": {
    "uri": "https://parallax.dev/schemas/evidence-bundle/v0.json",
    "hash": "sha256:...",
    "canonicalization": "jcs-rfc8785"
  },
  "projection_surfaces_required": ["bundle_json", "bundle_markdown", "cli_output", "http_api", "mcp_structuredContent"],
  "mcp_output_schema_required": true,
  "frontend_capture_claim_level": "not_measured|projection_pass|frontend_tiny_default_ready",
  "release_source": "ci_release_marker",
  "deploy_source": "github_actions_environment_deploy",
  "redaction_policy": "parallax-default-deny-v0",
  "reviewer": "redacted"
}
```

`dataset_kind` must be one of:

- `real_pilot` - production or staging telemetry from an operator or design
  partner system.
- `synthetic_fault` - fault-injected telemetry in a real instrumented service.
- `synthetic_generator` - generated rows from the benchmark harness.

Only `real_pilot` can support product claims about real-world correlation.

### Per-Anchor Ledger Row

Every sampled anchor gets one row in `anchor-ledger.jsonl`:

```json
{
  "schema_version": "a4-anchor-v1",
  "run_id": "a4-2026-05-25-operator-api",
  "anchor_id": "backend_error_000123",
  "anchor_class": "backend_error",
  "anchor_time": "2026-05-24T12:43:10Z",
  "telemetry_provenance": ["sentry_envelope", "otlp_trace", "otlp_log"],
  "trace_id_present": true,
  "trace_id_hash": "sha256:...",
  "span_id_present": true,
  "span_id_hash": "sha256:...",
  "trace_context_valid": true,
  "trace_flags_sampled": true,
  "trace_flags_raw": "01",
  "tracestate_valid": true,
  "remote_parent_observed": true,
  "resource_scope_consistent": true,
  "sampling_policy_observed": true,
  "sampled_out_explained": null,
  "same_trace_span_count": 9,
  "same_trace_log_count": 14,
  "duplicate_span_identity_count": 0,
  "error_in_span": true,
  "frontend_backend_continuation": null,
  "async_link_observed": false,
  "baggage_keys_seen": ["parallax.session"],
  "baggage_policy_pass": true,
  "release_context_present": true,
  "release_commit_present": true,
  "deploy_context_present": true,
  "deploy_success_status_present": true,
  "compare_base_present": true,
  "strong_edge_count": 4,
  "medium_edge_count": 2,
  "weak_edge_count": 1,
  "weak_only_bundle": false,
  "missing_evidence": ["missing_async_link"],
  "bundle_artifact_hash": "sha256:...",
  "bundle_schema_ref_hash": "sha256:...",
  "canonical_bundle_hash": "sha256:...",
  "projection_manifest_hash": "sha256:...",
  "projection_surfaces_checked": ["bundle_json", "bundle_markdown", "cli_output", "http_api", "mcp_structuredContent"],
  "projection_equivalence_pass": true,
  "mcp_structured_content_valid": true,
  "safety_fields_only_in_meta": false,
  "frontend_capture_claim_level": "not_applicable|projection_pass|frontend_tiny_default_ready",
  "frontend_projection_pass": null,
  "redaction_report_hash": "sha256:..."
}
```

Use `null` when a field does not apply to the anchor class. Use `false` when it
applies and was expected but absent.

### Counting Rules

- Do not compute a pass if the minimum sample size from the A4 gate is not met
  for that anchor class.
- Count a strong edge only when the join key is deterministic: exact
  `trace_id`/`span_id`, span parentage, span link, message/job ID, release ID,
  commit SHA, deploy marker ID, CI run ID, or issue/work-item ID.
- A deterministic trace key is not enough by itself: `trace_context_valid` must
  be true, `resource_scope_consistent` must be true, and the join must stay
  inside the run's `project_scope_fields` unless an explicit cross-scope edge is
  present.
- Invalid, all-zero, unparsable, or redaction-damaged trace context cannot count
  toward `trace_context_rate`; record `invalid_trace_context`,
  `unparsed_tracestate`, or `redaction_removed_required_field` instead.
- Do not count time-window proximity as a strong edge. At most it is weak, and
  only if the bundle marks it as such.
- Count `log_trace_join_rate` only from logs that carry OTel `TraceId`/`SpanId`
  fields or the non-OTLP `trace_id`/`span_id` equivalents.
- Count `frontend_backend_continuation_rate` only when the frontend request and
  backend span share trace context, or when Sentry trace context is explicitly
  bridged into the same normalized trace row.
- Count `async_link_rate` only when there is an OTel span link or an explicit
  message/job/workflow ID. Queue timing alone does not qualify.
- If an event has trace context but useful span data is unavailable because of
  sampling, include `sampled_out_trace` in `missing_evidence`; if the SDK
  sampler, collector policy, or tail-sampling route is not recorded, also include
  `missing_sampling_policy` or `tail_sampling_route_unverified`.
- Baggage-derived session or tenant joins count only when `baggage_policy_pass`
  is true and the observed keys are allowlisted opaque values. Raw user IDs,
  emails, account names, tokens, or third-party propagation failures make the run
  a privacy/redaction failure for that anchor class.
- Duplicate `(trace_id, span_id, service/resource scope)` rows must be deduped or
  counted as `duplicate_span_identity`; do not let duplicates inflate
  `same_trace_span_count` or `strong_edge_count`.
- Every expected missing-evidence category from the A4 gate must be represented
  when absent. Absence of the absence report blocks agent-visible bundles.
- Agent-visible A4 bundles require `schema_ref`, post-redaction
  `canonical_hash`, `projection_manifest`, `access`, `redaction_report`, and
  `missing_evidence` in the canonical JSON. `bundle_artifact_hash` alone is not
  enough because it can describe an unsafe or stale rendering.
- CLI, HTTP, and MCP projections must carry the same canonical bundle hash for
  the same authorized anchor request. A projection hash mismatch, missing
  projection row, or unscanned output path blocks the anchor from agent-visible
  A4 counts.
- MCP bundle output counts only when `structuredContent` validates against the
  evidence-bundle `outputSchema`; text-only JSON/Markdown can be a demo
  projection but not A4 proof.
- Safety fields such as `missing_evidence`, `redaction_report`,
  `source_field_policy`, and cited edge strengths must live in canonical JSON,
  not only in MCP `_meta`, descriptions, annotations, or prompt-wrapper
  metadata.
- Frontend anchors can count for `frontend_cross_tier_pass` only when the
  browser-side run has fresh frontend capture rows for trace propagation,
  metadata privacy, source-field policy, and projection safety. A trace
  continuation row without those browser safety rows is a correlation data
  point, not product wording.

### Manual Audit Row

Manual review rows go in `manual-audit.jsonl`:

```json
{
  "schema_version": "a4-manual-audit-v1",
  "run_id": "a4-2026-05-25-operator-api",
  "audit_id": "audit_000017",
  "anchor_id": "backend_error_000123",
  "reviewer": "redacted",
  "review_date": "2026-05-25",
  "strong_edges_checked": 4,
  "false_strong_edges": 0,
  "false_medium_edges": 1,
  "missing_evidence_false_negative": false,
  "trace_path_includes_failure": true,
  "trace_context_validated": true,
  "resource_scope_checked": true,
  "sampling_gap_explained": true,
  "baggage_privacy_checked": true,
  "canonical_projection_checked": true,
  "mcp_structured_content_checked": true,
  "frontend_capture_rows_checked": true,
  "bundle_next_step_useful": true,
  "verdict": "pass",
  "notes": "Medium deploy edge downgraded because first-seen history contradicted the rollout window."
}
```

At least 20 bundles per pilot need manual review. If fewer than 20 anchors exist,
review all of them and mark the aggregate claim as undersized.

### Instrumentation Repair Row

When a run fails because instrumentation is incomplete, record the fix loop in
`instrumentation-repairs.jsonl`:

```json
{
  "schema_version": "a4-instrumentation-repair-v1",
  "run_id": "a4-2026-05-25-operator-api",
  "repair_id": "repair_000004",
  "category": "browser_cors_trace_headers",
  "problem": "Frontend spans emitted traceparent but backend CORS did not allow traceparent/tracestate/baggage.",
  "before_metric": {
    "frontend_backend_continuation_rate": 0.12
  },
  "fix": "Allow traceparent, tracestate, and baggage on first-party API CORS preflight.",
  "after_metric": {
    "frontend_backend_continuation_rate": 0.68
  },
  "status": "verified"
}
```

Allowed `category` values:

- `browser_cors_trace_headers`
- `trace_propagation_targets`
- `trace_context_validation`
- `resource_scope_mapping`
- `collector_trace_id_routing`
- `sampling_policy`
- `tail_sampling_route_verification`
- `log_trace_context_injection`
- `baggage_allowlist`
- `deploy_marker_ingest`
- `release_commit_mapping`
- `async_span_links`
- `source_map_identity`
- `clock_skew`
- `redaction_removed_required_field`

### Derived Summary

The public Markdown summary should include:

- run manifest summary;
- per-anchor-class sample sizes;
- aggregate A4 metrics from the gate;
- trace context validity, scope consistency, sampling-explanation, duplicate-span,
  and baggage privacy rates;
- manual audit false-edge rates;
- missing-evidence report coverage;
- repair records attempted before final verdict;
- claim level and expiry date;
- links to the JSON/JSONL run artifacts.

### Claim Levels

Use these claim levels in `correlation-reliability-results.md`:

| Level | Meaning | Product wording allowed |
| --- | --- | --- |
| `not_measured` | No current A4 run exists. | "Correlation is planned." |
| `synthetic_only` | Only generator or fault-injected runs exist. | "Correlation model has been tested on fixtures." |
| `backend_mvp_measured` | Real backend anchors were measured, but one or more pass targets failed or sample size was too small. | "Best-effort backend context with explicit missing evidence." |
| `backend_mvp_pass` | Real backend error anchors pass the A4 backend targets. | "Evidence-backed backend request reconstruction for instrumented services." |
| `frontend_cross_tier_pass` | Backend targets pass, real frontend anchors pass continuation and missing-evidence targets, and the corresponding frontend capture safety rows are fresh for trace propagation, metadata privacy, source-field policy, and projection safety. | "Evidence-backed frontend-to-backend reconstruction for configured first-party routes." |
| `async_pass` | Queue/background anchors pass the async target. | "Evidence-backed async workflow reconstruction for instrumented queues." |
| `mixed_anchor_pass` | Backend, frontend, CI, CLI, and agent-session anchors pass their target slices. | "Evidence-backed execution context across configured engineering workflows." |
| `claim_expired` | A previous pass is older than the freshness window or a rerun trigger occurred. | "Previously measured; rerun required." |

Do not promote product language above the current claim level. The result can
advance incrementally; A4 does not have to pass every anchor class before the
backend MVP is useful.

### Freshness And Rerun Triggers

Rerun A4 when any of these change:

- SDK major/minor version used for Sentry, OpenTelemetry, or browser capture;
- propagator configuration, trace-context validator, baggage allowlist, or
  project/resource scope mapping;
- collector topology, sampling policy, tail-sampling route verification, or
  trace-ID routing;
- frontend CORS/header propagation configuration;
- log format or log trace-context injection method;
- release/deploy marker source;
- schema version for evidence bundle edges or missing-evidence categories;
- evidence-bundle canonicalization, projection renderer, MCP output schema, or
  access-surface projection behavior;
- frontend capture safety claim level or projection safety result;
- redaction policy that can remove join keys;
- a new anchor class enters the product claim;
- 90 days pass after a public product claim based on the previous run.

Until rerun, downgrade the affected claim to `claim_expired` or to the highest
unaffected narrower level.

### Relationship To Other Research

- [Risks and bear case](risks-and-bear-case.md) treats A4 as one of the
  load-bearing technical assumptions.
- [Build roadmap and validation sequence](build-roadmap-and-validation-sequence.md)
  puts A4 in Phase 1-2 after the market and bundle-value killers.
- [Strategic verdict and research coverage](strategic-verdict-and-research-coverage.md)
  lists A4 as a still-unproven technical proof gate.
- [A1 eval result ledger and model refresh](a1-eval-result-ledger-and-model-refresh.md)
  is the equivalent proof artifact for bundle-value claims.
- [A2 interview evidence ledger](a2-interview-evidence-ledger.md) and
  [A3 schema adoption and corpus ledger](a3-schema-adoption-corpus-ledger.md)
  follow the same pattern: future claims need committed evidence rows, not only
  narrative confidence.

### Bottom Line

A4 can pass only through a row-level audit of real telemetry. The ledger keeps
Parallax honest: strong edges must be deterministic, missing evidence must be
explicit, trace context must be valid and scoped, sampling gaps must be
explained, baggage-derived joins must be privacy-safe, repairs must be recorded,
agent-visible projections must preserve the same canonical bundle hash, and
product claims must expire when the instrumentation or projection surface
changes.
