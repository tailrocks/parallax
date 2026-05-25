# Correlation Reliability on Real Telemetry Gate

<!-- markdownlint-disable MD013 -->

Research date: 2026-05-25

## Purpose

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

## Source Posture

Current primary references support correlation as a standard mechanism, but also
show why it must be measured:

- W3C Trace Context defines the interoperable `traceparent` and `tracestate`
  headers; `traceparent` carries trace identity and sampling flags across system
  boundaries
  ([W3C Trace Context](https://www.w3.org/TR/trace-context/),
  [W3C Trace Context Level 2](https://www.w3.org/TR/trace-context-2/)).
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

Internal sources:

- [Causal reconstruction and agent safety](causal-reconstruction-and-agent-safety.md)
  says time correlation is weak and deterministic edges must be labeled.
- [Frontend collection and cross-tier correlation](frontend-collection-and-cross-tier-correlation.md)
  identifies CORS, sampling coherence, and browser gaps as the cross-tier failure
  modes.
- [Evidence bundle and open schema](evidence-bundle-and-schema.md) already
  distinguishes strong, medium, weak, and inferred edges.

## What To Measure

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

## Core Metrics

| Metric | Definition |
| --- | --- |
| `trace_context_rate` | Fraction of anchors with usable `trace_id`; for span-scoped anchors, also `span_id`. |
| `same_trace_bundle_rate` | Fraction of anchors where Q1 trace-context query returns spans/logs/errors beyond the anchor itself. |
| `log_trace_join_rate` | Fraction of relevant log excerpts carrying `trace_id` or `span_id`. |
| `error_in_span_rate` | Fraction of error events that can be placed inside a span by `trace_id` + `span_id` or equivalent Sentry trace context. |
| `frontend_backend_continuation_rate` | Fraction of first-party frontend errors/requests whose trace continues into backend spans. |
| `async_link_rate` | Fraction of queue/background workflow anchors with span links or explicit message/job IDs. |
| `release_context_rate` | Fraction of anchors attachable to release/version/commit metadata. |
| `deploy_context_rate` | Fraction attachable to a deploy marker or rollout window. |
| `release_commit_rate` | Fraction of release markers with exact commit SHA or source revision. |
| `deploy_success_status_rate` | Fraction of deploy markers with terminal success/failure/error status. |
| `compare_base_rate` | Fraction of release/deploy windows with predecessor base available for code-change comparison. |
| `strong_edge_count_p50` | Median count of deterministic strong edges per bundle. |
| `weak_only_bundle_rate` | Fraction of bundles with no strong or medium edges. |
| `false_strong_edge_rate` | Manual-audit rate where a strong edge is structurally present but semantically wrong because of instrumentation bugs. |
| `missing_evidence_report_rate` | Fraction of bundles that correctly list every expected missing evidence category. |

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
- `clock_skew_suspected`
- `redaction_removed_required_field`

## Manual Audit

Automatically computed edge rates are not enough. Manually review at least 20
bundles per pilot and classify:

| Question | Why |
| --- | --- |
| Does every strong edge follow from a deterministic key, not time proximity? | Prevents laundering weak correlation into strong evidence. |
| Does the trace path actually include the failing operation? | Catches wrong propagation, duplicate spans, or unrelated spans in the same trace. |
| Are medium release/deploy edges contradicted by first-seen history? | Prevents false regression claims. |
| Are missing-data gaps explicit? | Agents must know when evidence is absent. |
| Would the bundle lead a human toward the same next investigation step? | Tests practical usefulness before A1 agent evals. |

Record reviewer, date, bundle id, verdict, false-positive edges, false-negative
missing-data flags, and recommended instrumentation fixes.

## Pass Targets

Initial targets for the Rust backend/tiny-tier wedge:

| Gate | Target |
| --- | --- |
| Backend `trace_context_rate` | >= 80 percent for instrumented Rust services. |
| Backend `error_in_span_rate` | >= 70 percent for instrumented Rust services. |
| `same_trace_bundle_rate` | >= 70 percent for backend error anchors. |
| `release_context_rate` | >= 90 percent for production error anchors. |
| `deploy_context_rate` | >= 70 percent where deploy markers are configured. |
| `strong_edge_count_p50` | >= 2 strong edges per backend error bundle. |
| `weak_only_bundle_rate` | <= 20 percent for the target MVP anchor classes. |
| `false_strong_edge_rate` | <= 5 percent in manual audit. |
| `missing_evidence_report_rate` | 100 percent for expected categories. |

Separate frontend/cross-tier target:

| Gate | Target |
| --- | --- |
| `frontend_backend_continuation_rate` | >= 60 percent for instrumented first-party API calls after CORS/header configuration. |
| Silent propagation failure detection | 100 percent of frontend spans with no backend continuation are flagged as missing evidence, not treated as no backend involvement. |

Separate async target:

| Gate | Target |
| --- | --- |
| `async_link_rate` | >= 50 percent for instrumented queue/background workflows before claiming async lifecycle reconstruction. |

These are first-pass targets. If real users cluster in lower-instrumentation
environments, lower the product claim before lowering the safety bar.

## Failure Consequences

| Failure | Product consequence |
| --- | --- |
| Backend strong-edge gates fail | Narrow MVP to Sentry-compatible grouping + release context + best-effort trace/log links. |
| Frontend continuation fails | Keep frontend capture as session/error evidence, but do not claim frontend-to-backend reconstruction. Ship a CORS/propagation diagnostic first. |
| Async links fail | Treat queues/background jobs as separate lifecycle anchors unless explicit message IDs or span links are present. |
| False strong edges exceed target | Downgrade the edge class or fix instrumentation before agent exposure. |
| Missing evidence is not reported | Block agent-visible bundles; absence reporting is a safety invariant. |
| Release/deploy context missing | Do not rank release-regression hypotheses above medium/weak. |

The goal is not to force a pass. A failed A4 gate is useful: it tells Parallax
where instrumentation, setup diagnostics, and honest product wording must change.

## Instrumentation Fixes To Try Before Declaring Failure

- Ensure Sentry events carry trace context and map to OTLP `trace_id`/`span_id`.
- Use parent-based sampling for end-to-end trace consistency, or collector tail
  sampling that keeps error/slow traces.
- Route all spans of one trace to the same tail-sampling collector when using a
  gateway topology.
- Add `trace_id`/`span_id` injection into structured logs and non-OTLP log
  formats.
- Add browser `traceparent` propagation allowlists and backend CORS headers for
  `traceparent`, `tracestate`, and `baggage`.
- Add explicit message IDs or span links at queue boundaries.
- Emit release/version and deploy markers from CI/CD.
- Add a `parallax doctor correlation` command that checks SDK propagation,
  browser CORS headers, sampler configuration, log context, and deploy metadata.

## Result Record

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

## Relationship To Other Research

- [Causal reconstruction and agent safety](causal-reconstruction-and-agent-safety.md)
  defines the edge-strength model this gate measures.
- [Frontend collection and cross-tier correlation](frontend-collection-and-cross-tier-correlation.md)
  defines the browser-to-backend propagation path and its failure modes.
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

## Bottom Line

A4 is true only if real telemetry contains enough deterministic edges. Measure
that before marketing lifecycle reconstruction. If strong edges are common,
Parallax can claim evidence-backed reconstruction for the target wedge. If they
are rare, the product must become honest best-effort context plus tooling that
helps users fix instrumentation gaps.
