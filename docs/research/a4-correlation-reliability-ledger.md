# A4 Correlation Reliability Ledger

<!-- markdownlint-disable MD013 -->

Research date: 2026-05-25

## Purpose

The [correlation reliability gate](correlation-reliability-real-telemetry-gate.md)
defines the metrics for bear-case assumption A4:

> Deterministic cross-signal correlation is reliable in real, messy telemetry.

This note defines the result ledger needed to make that gate auditable. A4 must
not pass on aggregate rates alone. Every run needs per-anchor rows, manual audit
rows, instrumentation repair records, and a claim-freshness state so future
research can tell exactly what was measured and what product wording is allowed.

## Current Primary-Source Checks

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

## Why A Ledger Is Required

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

## Artifact Set

Use this public, redacted layout once real runs begin:

```text
docs/research/correlation-reliability-results.md
docs/research/correlation-reliability-runs/<run_id>/manifest.json
docs/research/correlation-reliability-runs/<run_id>/anchor-ledger.jsonl
docs/research/correlation-reliability-runs/<run_id>/manual-audit.jsonl
docs/research/correlation-reliability-runs/<run_id>/instrumentation-repairs.jsonl
docs/research/correlation-reliability-runs/<run_id>/hashes.sha256
```

The Markdown summary is the human-readable verdict. The JSON/JSONL files are the
audit trail. Do not commit raw stack traces, log bodies, customer identifiers,
full trace IDs, source-map content, replay data, or private incident notes. Use
stable redacted IDs and salted or keyed hashes when a deterministic join needs to
be auditable without exposing the original value.

## Run Manifest

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

## Per-Anchor Ledger Row

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
  "redaction_report_hash": "sha256:..."
}
```

Use `null` when a field does not apply to the anchor class. Use `false` when it
applies and was expected but absent.

## Counting Rules

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

## Manual Audit Row

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
  "bundle_next_step_useful": true,
  "verdict": "pass",
  "notes": "Medium deploy edge downgraded because first-seen history contradicted the rollout window."
}
```

At least 20 bundles per pilot need manual review. If fewer than 20 anchors exist,
review all of them and mark the aggregate claim as undersized.

## Instrumentation Repair Row

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

## Derived Summary

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

## Claim Levels

Use these claim levels in `correlation-reliability-results.md`:

| Level | Meaning | Product wording allowed |
| --- | --- | --- |
| `not_measured` | No current A4 run exists. | "Correlation is planned." |
| `synthetic_only` | Only generator or fault-injected runs exist. | "Correlation model has been tested on fixtures." |
| `backend_mvp_measured` | Real backend anchors were measured, but one or more pass targets failed or sample size was too small. | "Best-effort backend context with explicit missing evidence." |
| `backend_mvp_pass` | Real backend error anchors pass the A4 backend targets. | "Evidence-backed backend request reconstruction for instrumented services." |
| `frontend_cross_tier_pass` | Backend targets pass and real frontend anchors pass continuation and missing-evidence targets. | "Evidence-backed frontend-to-backend reconstruction for configured first-party routes." |
| `async_pass` | Queue/background anchors pass the async target. | "Evidence-backed async workflow reconstruction for instrumented queues." |
| `mixed_anchor_pass` | Backend, frontend, CI, CLI, and agent-session anchors pass their target slices. | "Evidence-backed execution context across configured engineering workflows." |
| `claim_expired` | A previous pass is older than the freshness window or a rerun trigger occurred. | "Previously measured; rerun required." |

Do not promote product language above the current claim level. The result can
advance incrementally; A4 does not have to pass every anchor class before the
backend MVP is useful.

## Freshness And Rerun Triggers

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
- redaction policy that can remove join keys;
- a new anchor class enters the product claim;
- 90 days pass after a public product claim based on the previous run.

Until rerun, downgrade the affected claim to `claim_expired` or to the highest
unaffected narrower level.

## Relationship To Other Research

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

## Bottom Line

A4 can pass only through a row-level audit of real telemetry. The ledger keeps
Parallax honest: strong edges must be deterministic, missing evidence must be
explicit, trace context must be valid and scoped, sampling gaps must be
explained, baggage-derived joins must be privacy-safe, repairs must be recorded,
and product claims must expire when the instrumentation surface changes.
