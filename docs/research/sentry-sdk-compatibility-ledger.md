# Sentry SDK Compatibility Ledger

<!-- markdownlint-disable MD013 -->

Research date: 2026-05-25

## Purpose

This ledger turns "Sentry-compatible" from broad product language into an
auditable claim. It consumes the fixture gate in
[Sentry SDK fixture compatibility gate](sentry-sdk-fixture-compatibility.md) and
the ingestion design in
[Sentry-compatible ingestion](sentry-compatible-ingestion.md).

Current status: **not measured**. The repository has a compatibility strategy and
fixture design, but it does not yet have SDK-generated fixture results. Until
those results exist, Parallax should describe Sentry compatibility as a target,
not as a proven product property.

The central rule:

> No public "Sentry SDK compatible" claim without a dated SDK/version matrix,
> raw fixture hashes, parser results, normalization snapshots, grouping results,
> redaction results, source-field policy status, `schema_ref`, canonical bundle
> hashes, projection manifests, CLI/HTTP/MCP projection-equivalence rows, MCP
> `structuredContent`/`outputSchema` validation, and explicit unsupported-item
> outcomes.

## Current Source Snapshot

| Source | Current check | Why it matters |
| --- | --- | --- |
| Package registry snapshot ([crates.io](https://crates.io/crates/sentry), [npm browser](https://www.npmjs.com/package/@sentry/browser), [npm node](https://www.npmjs.com/package/@sentry/node), [npm React](https://www.npmjs.com/package/@sentry/react), [Go proxy](https://proxy.golang.org/github.com/getsentry/sentry-go/@latest), [PyPI](https://pypi.org/project/sentry-sdk/)) | Registry checks re-run on 2026-05-25 found Rust `sentry` `0.48.2`, Rust `sentry-types` `0.48.2`, JavaScript `@sentry/browser`, `@sentry/node`, and `@sentry/react` `10.53.1`, Go `github.com/getsentry/sentry-go` `v0.46.2`, and Python `sentry-sdk` `2.60.0`. Exact registry timestamps are recorded in the freshness pass below. | A compatibility claim must pin the exact SDK version and package source, because Sentry SDKs and item models move quickly. |
| [sentry Rust crate 0.48.2](https://docs.rs/sentry/latest/sentry/) | Docs.rs currently resolves `sentry` to `0.48.2`; the crate integrates Rust panics, contexts, backtraces, `anyhow`, `tracing`, OpenTelemetry, transports, and protocol/types. | This is the first SDK fixture target because Parallax is Rust-first. |
| [Sentry envelope struct](https://docs.rs/sentry/latest/sentry/struct.Envelope.html) | Sentry describes the envelope as the ingestion data format; it can contain related items such as events and attachments, plus independent items such as sessions. | The compatibility surface is not only JSON events; item policy is part of the claim. |
| [sentry-types envelope parser](https://docs.rs/sentry-types/latest/src/sentry_types/protocol/envelope.rs.html) | Current envelope headers include `event_id`, `dsn`, `sdk`, `sent_at`, and trace/dynamic-sampling context; item headers include `type`, optional `length`, `content_type`, filename, and attachment type. Current item variants include `event`, `session`, `sessions`, `transaction`, `attachment`, `check_in`, `log`, and `trace_metric`, and the Rust enum is non-exhaustive. | Parser fixtures must cover length/no-length payloads, current container items, and unsupported/future items without poisoning supported event ingestion. |
| [sentry-python envelope source](https://getsentry.github.io/sentry-python/_modules/sentry_sdk/envelope.html) | The Python SDK source documents Sentry envelope constraints and says each envelope may contain at most one `event` or `transaction`, not both. | A second SDK confirms that compatibility claims must respect SDK-side envelope rules, not only Parallax parser behavior. |
| [sentry-tracing 0.48.2](https://docs.rs/sentry-tracing/latest/sentry_tracing/) | The tracing integration can map `tracing` events to Sentry events, breadcrumbs, logs, and spans; by default, high-severity events become error events and ordinary events become breadcrumbs/spans. | Rust fixtures must cover `tracing::error!`, structured fields, tags, breadcrumbs, and span/trace fields. |
| [Sentry issue grouping](https://docs.sentry.io/concepts/data-management/event-grouping/) | Sentry considers fingerprint first, then stack trace, exception, and message; stacktrace grouping depends on in-app frame material and grouping algorithm versions. | Parallax should prove deterministic Parallax grouping, not claim exact Sentry grouping parity. |
| [Sentry fingerprint rules](https://docs.sentry.io/concepts/data-management/event-grouping/fingerprint-rules/) | Fingerprint rules can override default grouping or refine it with `{{ default }}`. | Parallax must preserve client-provided fingerprints and record whether the grouping source was client, Parallax default, or fallback. |
| [Sentry trace propagation](https://develop.sentry.dev/sdk/foundations/trace-propagation/) | Sentry trace propagation uses `sentry-trace` and `baggage`; SDKs can optionally emit W3C `traceparent` for OpenTelemetry interop. | Compatibility is only valuable for Parallax if Sentry error context can join to OTLP traces/logs. |
| [Sentry Relay repository](https://github.com/getsentry/relay) | Relay remains the closest Rust ingestion reference, but it is a gateway/processing system rather than a tiny Parallax dependency. | Use Relay as a reference oracle where useful, not as the operational architecture. |
| [MCP 2025-11-25 tools spec](https://modelcontextprotocol.io/specification/2025-11-25/server/tools), [MCP base protocol](https://modelcontextprotocol.io/specification/2025-11-25/basic), and [RFC 8785 JCS](https://www.rfc-editor.org/rfc/rfc8785.html) | MCP tools can return `structuredContent` validated by `outputSchema`; MCP reserves `_meta` for protocol metadata; JCS provides a deterministic JSON representation suitable for hashes. | Sentry fixture outputs cannot be considered agent-ready unless bundle projections are structured, schema-valid, and hash-equivalent across access surfaces. |
| [Bugsink Sentry SDK compatibility](https://www.bugsink.com/sentry-sdk-compatible/), [Bugsink simplicity recheck](bugsink-sentry-compatible-simplicity-recheck.md), and [Urgentry](https://urgentry.com/) | Lightweight competitors publicly use DSN-change or drop-in Sentry replacement language. Bugsink's current docs explicitly frame migration as keeping the SDK and updating the DSN. | Parallax needs more precise evidence and wording; simple compatibility language is already crowded. |

## 2026-05-25 Freshness Pass

Pass target: falsify whether the SDK matrix had gone stale before Parallax has
real fixture results.

Result: **no package-version drift found** from the current ledger snapshot, and
the Sentry Rust item model remains non-exhaustive with `log` and `trace_metric`
container items. This does **not** advance Parallax beyond `not_measured`.
Registry freshness proves only that the source snapshot is current; it does not
prove parser, normalization, grouping, redaction, idempotency, trace-correlation,
canonical-bundle, projection-equivalence, MCP structured-output, or
unsupported-item behavior.

| Package/model | Primary source checked | Current version or model | Registry/source timestamp | Implication |
| --- | --- | --- | --- | --- |
| Rust `sentry` | [crates.io API](https://crates.io/api/v1/crates/sentry) | `0.48.2` | crate updated `2026-05-11T09:10:36.368466Z` | No SDK-version drift from the existing Rust fixture target. |
| Rust `sentry-types` | [crates.io API](https://crates.io/api/v1/crates/sentry-types) and [docs.rs envelope source](https://docs.rs/sentry-types/latest/src/sentry_types/protocol/envelope.rs.html) | `0.48.2`; envelope/item enums are non-exhaustive; modeled items still include `event`, `session`, `sessions`, `transaction`, `attachment`, `check_in`, `log`, and `trace_metric`. | crate updated `2026-05-11T09:08:58.215870Z` | Fixture matrix still needs `log`, `trace_metric`, and unknown future-item outcomes before any multi-SDK smoke claim. |
| JavaScript `@sentry/browser` | [npm registry](https://registry.npmjs.org/@sentry%2fbrowser) | `10.53.1` | package modified `2026-05-12T17:07:50.879Z`; version published `2026-05-12T17:07:50.769Z` | Browser smoke remains a later L4 target, not v0 proof. |
| JavaScript `@sentry/node` | [npm registry](https://registry.npmjs.org/@sentry%2fnode) | `10.53.1` | package modified `2026-05-12T17:07:53.853Z`; version published `2026-05-12T17:07:53.694Z` | Node remains optional smoke only if migration demand needs it. |
| JavaScript `@sentry/react` | [npm registry](https://registry.npmjs.org/@sentry%2freact) | `10.53.1` | package modified `2026-05-12T17:08:04.434Z`; version published `2026-05-12T17:08:04.309Z` | React belongs in the source snapshot because frontend capture is a roadmap branch, but it does not change the Rust-first v0 target. |
| Go `sentry-go` | [Go module proxy](https://proxy.golang.org/github.com/getsentry/sentry-go/@latest) | `v0.46.2` | module timestamp `2026-05-04T09:47:49Z`; tag hash `1d2598e7580f52f201f06ce6b5d819c11a977f4c` | Go smoke remains a later L4 target after Rust L1/L2 pass. |
| Python `sentry-sdk` | [PyPI JSON](https://pypi.org/pypi/sentry-sdk/json) | `2.60.0` | upload `2026-05-13T13:34:52.516271Z` | Python remains smoke/adoption research, not part of the first compatibility claim. |

Freshness conclusion: keep the version matrix, keep the item-drift fixture
requirements, and keep public wording at **planned Sentry-compatible ingestion**
until real SDK-generated envelopes pass the ledger.

## Claim Levels

| Level | Meaning | Allowed wording |
| --- | --- | --- |
| `not_measured` | No current SDK-generated fixture run exists. | "Sentry-compatible ingestion is planned." |
| `parser_only` | Envelope parser accepts syntactically valid envelopes and rejects malformed envelopes deterministically, but no SDK matrix has passed. | "Envelope parser prototype." |
| `rust_error_event_compatible` | Current Rust SDK panic, captured error, message, and tracing event fixtures normalize into Parallax error rows. | "Compatible with current Sentry Rust SDK error-event envelopes." |
| `rust_trace_link_compatible` | Rust SDK fixtures carrying Sentry trace context join to matching OTLP trace/log rows. | "Sentry Rust errors link to OpenTelemetry trace context." |
| `rust_grouping_stable` | Rust fixtures plus rebuild/debuginfo variants produce stable versioned Parallax fingerprints. | "Deterministic Parallax grouping for Rust Sentry error events." |
| `multi_sdk_error_smoke` | At least Rust plus two non-Rust SDKs parse and normalize core error fields for dated versions. | "Sentry SDK-compatible error ingestion for the tested SDK matrix." |
| `sentry_sdk_compatible_error_ingest` | Multi-SDK error-event matrix passes parser, normalization, grouping, redaction, source-field, canonical bundle, projection-equivalence, MCP structured-output, idempotency, unsupported-item, and trace-context gates. | "Sentry SDK-compatible error ingestion" with matrix link. |
| `drop_in_sentry_replacement_not_supported` | Sessions, replay, profiles, release health, attachments, exact grouping parity, and Sentry API/UI parity are not supported. | Required caveat for MVP. |
| `claim_expired` | A supported SDK, envelope item model, grouping algorithm, redaction/source-field/projection policy, or 90-day timer changed after the last pass. | "Compatibility result expired; rerun required." |
| `claim_failed` | A fixture run failed any required gate for the advertised level. | No compatibility claim for the affected SDK/version/path. |

Initial Parallax level: `not_measured`.

## Result Artifacts

Compatibility runs should be durable, source-linked, and diffable:

```text
docs/research/sentry-compatibility-results.md
docs/research/sentry-compatibility-runs/<run_id>/manifest.json
docs/research/sentry-compatibility-runs/<run_id>/raw-envelopes/<fixture_id>.envelope
docs/research/sentry-compatibility-runs/<run_id>/parser-results.jsonl
docs/research/sentry-compatibility-runs/<run_id>/normalization-results.jsonl
docs/research/sentry-compatibility-runs/<run_id>/grouping-results.jsonl
docs/research/sentry-compatibility-runs/<run_id>/redaction-results.jsonl
docs/research/sentry-compatibility-runs/<run_id>/source-field-policy-results.jsonl
docs/research/sentry-compatibility-runs/<run_id>/trace-correlation-results.jsonl
docs/research/sentry-compatibility-runs/<run_id>/unsupported-item-results.jsonl
docs/research/sentry-compatibility-runs/<run_id>/projection-results.jsonl
docs/research/sentry-compatibility-runs/<run_id>/bundle-projection-results.jsonl
docs/research/sentry-compatibility-runs/<run_id>/mcp-projection-results.jsonl
docs/research/sentry-compatibility-runs/<run_id>/sdk-matrix.jsonl
docs/research/sentry-compatibility-runs/<run_id>/claim-ledger.jsonl
docs/research/sentry-compatibility-runs/<run_id>/hashes.sha256
```

Do not create these result directories for hypothetical data. Add them only when
a real fixture run exists.

## Run Manifest

Each `manifest.json` should include:

```json
{
  "run_id": "sentry-compat-YYYYMMDD-N",
  "research_date": "YYYY-MM-DD",
  "fixture_generator_commit": "<git-sha>",
  "parallax_parser_commit": "<git-sha>",
  "parallax_grouping_version": "rust-stack-v1",
  "bundle_schema_ref": {
    "uri": "schema://parallax/evidence-bundle/v0",
    "hash": "sha256:<hex>",
    "canonicalization": "jcs-rfc8785"
  },
  "canonical_bundle_hash_required": true,
  "redaction_policy_version": "a6-default-deny-vN",
  "source_field_policy_version": "phase0-source-field-policy-vN",
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
    "sentry_rust": "0.48.2",
    "sentry_tracing": "0.48.2",
    "sentry_types": "0.48.2",
    "@sentry/browser": "10.53.1",
    "@sentry/node": "10.53.1",
    "@sentry/react": "10.53.1",
    "sentry-go": "v0.46.2",
    "sentry-sdk-python": "2.60.0"
  },
  "envelope_item_model_snapshot": ["event", "session", "sessions", "transaction", "attachment", "check_in", "log", "trace_metric", "unknown_future_item"],
  "endpoint": "POST /api/<project_id>/envelope/",
  "unsupported_item_policy": "explicit_outcome",
  "size_limits": {},
  "idempotency_policy": "project_id+event_id",
  "raw_envelope_visibility": "fixture_only_not_agent_visible",
  "raw_envelope_access_policy": "deny_dereference_by_default",
  "projection_formats": ["bundle_json", "bundle_markdown", "cli_output", "http_api", "mcp_structuredContent"],
  "fixture_app_hashes": [],
  "notes": []
}
```

The manifest must separate SDK version, language/runtime version, Parallax
parser version, grouping algorithm version, and redaction policy version. A pass
with one grouping/redaction version does not automatically carry over to another.

## Row Schemas

### SDK Matrix Row

```json
{
  "sdk_name": "sentry-rust",
  "sdk_version": "0.48.2",
  "language": "rust",
  "runtime": "rustc <version>",
  "package_source": "crates.io|npm|go-proxy|pypi|git",
  "registry_checked_at": "YYYY-MM-DD",
  "features": ["panic", "backtrace", "contexts", "tracing"],
  "fixture_id": "rust_panic_default",
  "scenario": "panic with default integrations",
  "envelope_hash": "sha256:<hex>",
  "target_level": "rust_error_event_compatible"
}
```

### Parser Result Row

```json
{
  "fixture_id": "rust_panic_default",
  "accepted": true,
  "response_status": 200,
  "envelope_header_fields": ["event_id", "sdk", "sent_at", "trace"],
  "item_count": 1,
  "item_types": ["event"],
  "unknown_items": [],
  "length_mode": "with_length|without_length",
  "malformed_behavior": null,
  "supported_items_processed": ["event"],
  "unsupported_outcomes": [],
  "raw_envelope_ref": "raw-envelopes/rust_panic_default.envelope",
  "raw_envelope_agent_visible": false
}
```

### Normalization Result Row

```json
{
  "fixture_id": "rust_panic_default",
  "event_id": "<uuid>",
  "platform": "rust",
  "level": "fatal|error|info",
  "release": "example@1.2.3",
  "environment": "fixture",
  "exception_preserved": true,
  "stack_frame_count": 12,
  "in_app_frame_count": 3,
  "request_redacted": true,
  "breadcrumbs_preserved": true,
  "trace_context_preserved": true,
  "debug_meta_preserved": true,
  "fingerprint_preserved": true,
  "unsupported_item_refs_excluded": true,
  "intentional_drops": []
}
```

### Grouping Result Row

```json
{
  "fixture_id": "rust_explicit_fingerprint",
  "fingerprint_source": "client|parallax_stack|parallax_message_fallback",
  "client_fingerprint_preserved": true,
  "parallax_fingerprint": "sha256:<hex>",
  "grouping_algorithm_version": "rust-stack-v1",
  "stable_across_rebuild_variants": true,
  "notes": []
}
```

### Trace Correlation Row

```json
{
  "fixture_id": "rust_trace_context",
  "sentry_trace_id": "<32-hex>",
  "sentry_span_id": "<16-hex>",
  "otlp_trace_id": "<32-hex>",
  "otlp_span_id": "<16-hex>",
  "matched": true,
  "edge_strength": "strong",
  "missing_evidence": []
}
```

### Redaction Result Row

```json
{
  "fixture_id": "rust_request_context",
  "seeded_canaries": 12,
  "leaked_canaries": 0,
  "agent_visible_leaks": 0,
  "useful_context_preserved": true,
  "redaction_policy_version": "a6-default-deny-vN",
  "redaction_report_hash": "sha256:<hex>"
}
```

### Source Field Policy Result Row

```json
{
  "fixture_id": "rust_request_context",
  "source_kind": "synthetic_fixture",
  "source_field_policy_status": "pass|fail",
  "source_field_policy_version": "phase0-source-field-policy-vN",
  "source_field_policy_hash": "sha256:<hex>",
  "denied_zone_count": 0,
  "violation_count": 0
}
```

### Unsupported Item Result Row

```json
{
  "fixture_id": "rust_log_container",
  "item_type": "log|trace_metric|attachment|session|profile|replay_recording|unknown_future_item",
  "item_present": true,
  "event_item_present": true,
  "event_processing_poisoned": false,
  "outcome": "accepted_event_only|metadata_only|rejected_with_outcome|unsupported_ref_only",
  "retry_safe": true,
  "agent_visible_payload": false,
  "notes": []
}
```

### Projection Result Row

```json
{
  "fixture_id": "rust_request_context",
  "projection_surface": "bundle_json|bundle_markdown|cli_output|http_api|mcp_structuredContent",
  "schema_ref_hash": "sha256:<hex>",
  "canonical_bundle_hash": "sha256:<hex>",
  "projection_manifest_hash": "sha256:<hex>",
  "projection_equivalence_hash": "sha256:<hex>",
  "projection_equivalence_pass": true,
  "mcp_output_schema_valid": "true|false|not_used",
  "mcp_structured_content_hash": "sha256:<hex>|not_used",
  "redaction_report_present": true,
  "source_field_policy_status": "pass|fail",
  "missing_evidence_present": true,
  "raw_envelope_ref_count": 1,
  "raw_envelope_ref_dereferenced": false,
  "safety_fields_only_in_meta": false,
  "unsupported_payload_visible": false,
  "seeded_canary_leaks": 0,
  "agent_visible_pass": true
}
```

### Claim Ledger Row

```json
{
  "run_id": "sentry-compat-YYYYMMDD-N",
  "claim_level": "rust_error_event_compatible",
  "claim_status": "pass|fail|expired",
  "sdk_matrix": ["sentry-rust@0.48.2"],
  "canonical_bundle_hashes": ["sha256:<hex>"],
  "projection_manifest_hashes": ["sha256:<hex>"],
  "mcp_output_schema_valid": "true|false|not_used",
  "product_wording": "Compatible with current Sentry Rust SDK error-event envelopes.",
  "required_caveats": ["not a drop-in Sentry replacement"],
  "expires_at": "YYYY-MM-DD"
}
```

## Counting Rules

- No broad "Sentry-compatible" claim without a dated SDK/language/version
  matrix.
- No agent-visible Sentry compatibility claim without `schema_ref`,
  `canonical_bundle_hash`, `projection_manifest_hash`, and access-policy rows
  for each fixture bundle.
- CLI, HTTP, MCP, Markdown, and persisted JSON projections must carry equivalent
  canonical bundle and projection manifest hashes for the same fixture scope.
- MCP projection rows count only when `structuredContent` validates against the
  declared `outputSchema`; text-only MCP output is not an agent-ready Sentry
  compatibility result.
- Safety-critical fields do not count if they appear only in MCP `_meta`, text
  descriptions, or free-form PR/diagnosis prose.
- No "supports Sentry SDKs" wording until at least Rust L1/L2 and
  multi-SDK error smoke pass.
- No "drop-in Sentry replacement" wording unless sessions, replay, profiles,
  attachments, release health, grouping semantics, and relevant Sentry API/UI
  behavior are deliberately supported and tested. MVP should explicitly say this
  is not supported.
- Preserve `event_id` and enforce project-scoped idempotency.
- Preserve client `fingerprint` and record whether Parallax used it.
- Preserve `contexts.trace` and propagated trace identifiers well enough to join
  to OTLP rows.
- Unsupported envelope items must create explicit outcomes; silent poison-pill
  behavior is a failed compatibility result.
- Current item drift must be represented in fixtures. `log` and `trace_metric`
  container items, plus an unknown future item, must be covered before any
  multi-SDK smoke claim.
- Exact Sentry grouping parity is not claimed. The testable claim is stable,
  versioned Parallax grouping.
- Raw envelopes used for tests must be synthetic or explicitly safe fixtures, not
  production customer data.
- Redaction and source-field policy must pass before any fixture output is
  promoted into an agent-visible bundle.
- A6 redaction rows must be current before any Sentry fixture projection is
  considered safe for agent use.
- A4 correlation rows must be current before trace-link compatibility is used
  as evidence for cross-system reconstruction.
- Raw envelopes and unsupported-item payloads may be retained for parser audit,
  but agent-visible projections must expose only refs, redaction/source-field
  reports, missing-evidence flags, and normalized safe fields.

## Refresh Triggers

Rerun the matrix and mark affected claims `claim_expired` when any of these
change:

- supported Sentry SDK release;
- `sentry-types` or Relay envelope/item model change;
- JavaScript, Go, Python, or Rust package registry version for any advertised
  SDK changes;
- common SDK item type appears in the supported path;
- Parallax parser version changes;
- Parallax grouping algorithm changes;
- Parallax redaction policy changes;
- Parallax source-field policy or projection schema changes;
- Parallax bundle schema, canonicalization method, projection surface list, MCP
  output schema, or access policy changes;
- unsupported-item policy changes;
- 90 days pass since the last run.

## Product Wording

Allowed after `rust_error_event_compatible`:

> Compatible with current Sentry Rust SDK error-event envelopes for panics,
> captured errors, messages, breadcrumbs, tags, release/environment, and trace
> context. Not a drop-in Sentry replacement.

Allowed after `sentry_sdk_compatible_error_ingest`:

> Sentry SDK-compatible error ingestion for the tested SDK matrix.

Always link the matrix, unsupported-item outcomes, and projection safety rows.
When MCP is part of the advertised access surface, also link the
`structuredContent` validation row and canonical projection hash.
Avoid unqualified claims such as:

- "Drop-in Sentry replacement";
- "same grouping as Sentry";
- "supports Sentry SDKs" without tested versions;
- "supports Sentry" when only `event` items are accepted.

## Relationship To Other Research

- [Sentry SDK fixture compatibility gate](sentry-sdk-fixture-compatibility.md) -
  defines the fixture scenarios this ledger turns into result rows.
- [Sentry-compatible ingestion](sentry-compatible-ingestion.md) - defines the
  ingest boundary and unsupported item policy.
- [Rust stacktrace grouping and symbolication](rust-stacktrace-grouping-and-symbolication.md)
  and [Rust stacktrace grouping ledger](rust-stacktrace-grouping-ledger.md)
  - control the Rust grouping-stability subclaim and the result rows needed for
  `rust_grouping_stable`.
- [A6 redaction red-team ledger](a6-redaction-red-team-ledger.md) - controls
  whether fixture output may enter agent-visible bundles.
- [Evidence bundle and open schema specification](evidence-bundle-and-schema.md)
  - defines the source-field policy and redaction report fields that
  compatibility projections must carry.
- [OTLP receiver conformance and Collector equivalence](otlp-receiver-conformance-and-collector-equivalence.md)
  - pairs with the trace-correlation rows for mixed Sentry/OTLP evidence.
- [Lightweight Sentry-compatible competitor watch](lightweight-sentry-compatible-competitor-watch.md)
  - explains why compatibility must be precise and evidence-backed.

## Bottom Line

Parallax can use Sentry SDK compatibility as a migration wedge only if it is
measured like a protocol contract. The first honest target is narrow:

> current Sentry Rust SDK error envelopes parsed, normalized, grouped,
> redacted, deduplicated, and linked to OTLP trace evidence.

Everything broader waits for dated fixture results.
