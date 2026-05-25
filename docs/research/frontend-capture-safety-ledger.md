# Frontend Capture Safety Ledger

<!-- markdownlint-disable MD013 -->

Research date: 2026-05-25

## Purpose

[Frontend collection and cross-tier correlation](frontend-collection-and-cross-tier-correlation.md)
defines the architecture for browser error capture, breadcrumbs, source maps,
frontend-to-backend trace propagation, and privacy controls. This ledger defines
the result artifacts, row schemas, claim levels, and expiry rules required
before Parallax can say frontend capture is safe, source-mapped, low-overhead, or
usable for frontend-to-backend reconstruction.

The focused
[frontend browser ingest profile recheck](frontend-browser-ingest-profile-recheck.md)
adds the current browser transport boundary: browser OTLP is HTTP-only and must
be proven separately from backend/server gRPC requirements.

The focused
[frontend Replay and source-map privacy recheck](frontend-replay-sourcemap-privacy-recheck.md)
adds the current Replay package-provenance and source-artifact boundary: Replay
and source maps are raw/reference surfaces, not agent-visible defaults.

Current status: **not measured**. The repository has a frontend capture design
and A4 continuation gate, but no dated browser/build/route run artifacts. Until
those results exist, Parallax should describe frontend capture as planned and
designed, not proven.

The central rule:

> No "frontend-to-backend reconstruction", "source-mapped frontend errors",
> "safe browser breadcrumbs", "RUM/replay capture", or "low-overhead browser
> telemetry" claim without dated runs covering browser versions, SDK versions,
> route fixtures, browser ingest posture, source-map identity and artifact
> access, CORS/header propagation, backend continuation, breadcrumbs, privacy
> canaries, export reliability, payload/drop behavior, overhead, replay opt-in
> policy, source-field policy status, canonical bundle hash, projection
> manifest, and agent-visible projections including MCP `structuredContent`.

This ledger is separate from the
[A4 correlation reliability ledger](a4-correlation-reliability-ledger.md): A4
measures whether real anchors link across signals; this ledger measures whether
the browser capture surface itself is safe and configured well enough to create
those anchors.

## Current Source Snapshot

| Source | Current check | Parallax implication |
| --- | --- | --- |
| npm package version snapshot ([Sentry browser](https://www.npmjs.com/package/@sentry/browser), [Sentry React](https://www.npmjs.com/package/@sentry/react), [OTel web SDK](https://www.npmjs.com/package/@opentelemetry/sdk-trace-web), [OTel fetch](https://www.npmjs.com/package/@opentelemetry/instrumentation-fetch), [OTel OTLP HTTP](https://www.npmjs.com/package/@opentelemetry/exporter-trace-otlp-http)) | `npm view` on 2026-05-25 reported `@sentry/browser` `10.53.1`, `@sentry/react` `10.53.1`, `@opentelemetry/sdk-trace-web` `2.7.1`, `@opentelemetry/instrumentation-fetch` `0.218.0`, and OTLP HTTP trace exporters `0.218.0`. | Every run must persist the exact package versions from the lockfile or registry snapshot; docs pages can lag package releases. |
| [OpenTelemetry JavaScript browser guide](https://opentelemetry.io/docs/languages/js/getting-started/browser/) | Browser traces use `@opentelemetry/sdk-trace-web` plus browser instrumentations such as document-load; the guide warns browser client instrumentation is experimental and mostly unspecified. | Parallax can use OTel JS for spans, but browser support must be tested per SDK and browser matrix before product wording. |
| [OpenTelemetry JavaScript exporters](https://opentelemetry.io/docs/languages/js/exporters/) | Browser deployments cannot use OTLP/gRPC; they must use OTLP HTTP/JSON or HTTP/protobuf, handle CSP and CORS, and may require a collector reachable from public browsers. | Parallax should use a narrow browser ingest/proxy endpoint with origin, size, rate, auth, path, and redaction controls instead of exposing a broad collector. |
| [Frontend browser ingest profile recheck](frontend-browser-ingest-profile-recheck.md) | Current package recheck found no version drift from the 2026-05-25 ledger snapshot and verified published OTel `0.218.0` fetch/XHR propagation controls plus browser HTTP/JSON and HTTP/protobuf exporter builds. | Add browser-specific transport/CORS fixtures; do not apply backend gRPC-required OTLP wording to browser clients. |
| [Frontend Replay and source-map privacy recheck](frontend-replay-sourcemap-privacy-recheck.md) | `npm view` on 2026-05-25 found current Sentry JS `10.53.1`; `@sentry/browser` pulls `@sentry-internal/replay` and `@sentry-internal/replay-canvas` `10.53.1`, while standalone `@sentry/replay` remains `7.116.0` from 2025-11-25. | Run manifests must record Replay package provenance and cannot cite stale standalone `@sentry/replay` as current v10 behavior unless the app lockfile actually includes it. |
| [OpenTelemetry fetch instrumentation](https://open-telemetry.github.io/opentelemetry-js/modules/_opentelemetry_instrumentation-fetch.html) | Fetch instrumentation exposes config such as `requestHook`, `ignoreUrls`, and propagation-related options. | Propagation and redaction need explicit allowlists and hooks; raw URLs/body-like fields must not leak by default. |
| [W3C Trace Context](https://www.w3.org/TR/trace-context/) | W3C recommends wide deployment of `traceparent`/`tracestate`; `traceparent` carries portable trace identity and tools must propagate it to avoid broken traces. | Frontend-to-backend joins should use W3C trace context when using OTel paths and record propagation failures as missing evidence. |
| [OpenTelemetry baggage](https://opentelemetry.io/docs/concepts/signals/baggage/) | Baggage can propagate arbitrary key/value context, and official docs warn sensitive baggage can reach unintended resources such as third-party APIs. | Session correlation must use allowlisted opaque values; raw user/account IDs, emails, tokens, or third-party baggage propagation fail the browser safety gate. |
| [MDN Access-Control-Allow-Headers](https://developer.mozilla.org/docs/Web/HTTP/Reference/Headers/Access-Control-Allow-Headers) | Browsers rely on the preflight response to decide which non-safelisted request headers can be sent. | `traceparent`, `tracestate`, `baggage`, `sentry-trace`, and project auth headers need explicit CORS tests for cross-origin APIs. |
| [Sentry JavaScript trace propagation](https://docs.sentry.io/platforms/javascript/guides/capacitor/tracing/trace-propagation/) | Sentry browser tracing propagates `sentry-trace` and `baggage`, requires those headers in CORS allowlists, and uses `tracePropagationTargets` to control outgoing propagation. | Sentry-compatible frontend events must bridge Sentry trace context into Parallax rows without spraying trace headers to third parties. |
| [Sentry JavaScript options](https://docs.sentry.io/platforms/javascript/configuration/environments/) | Browser options include release, max breadcrumbs, `beforeBreadcrumb`, `beforeSend`, `beforeSendSpan`, `tracePropagationTargets`, replay sample rates, and transport behavior; same-origin trace propagation is enabled by default in the browser. | Result rows must capture SDK config, sampling, filtering hooks, trace propagation target scope, payload limits, and whether dropped events are reported. |
| [Sentry breadcrumbs](https://docs.sentry.io/platforms/javascript/guides/svelte/enriching-events/breadcrumbs/) | Browser SDKs automatically record clicks, key presses, XHR/fetch requests, console calls, and location changes; `beforeBreadcrumb` can modify or discard breadcrumbs. | Breadcrumbs are high-value but high-risk; run artifacts must prove value-shape capture and redaction before agent exposure. |
| [Sentry artifact bundles and Debug IDs](https://docs.sentry.io/platforms/javascript/guides/cloudflare/sourcemaps/troubleshooting_js/artifact-bundles/) | Artifact bundles bind minified files and source maps by Debug ID instead of relying on paths; retention and release association are explicit. | Parallax should use Debug-ID-like source-map identity and test missing/mismatched/private source maps directly. |
| [Sentry source-map upload warning](https://docs.sentry.io/platforms/javascript/guides/tanstackstart-react/sourcemaps/uploading/esbuild) | Sentry warns generated source maps can expose source and recommends denying public `.js.map` access or deleting maps after upload. | A source-mapped claim fails if the build deploys public source maps or if agents can dereference source-map/source-content refs by default. |
| [Sentry JavaScript data collected](https://docs.sentry.io/platforms/javascript/guides/react/data-management/data-collected) | Sentry documents privacy-relevant defaults: cookies, logged-in user identity, user IP, client-side request bodies, and response bodies are not sent by default; HTTP request/response headers, full request URLs, full query strings, referrer URLs, and console logs/breadcrumbs may be collected; Replay masks text/images/user input by default and network detail bodies are opt-in. The docs page package detail can lag npm (`10.11.0` shown while npm reports `10.53.1` for `@sentry/react`). | Parallax should keep frontend capture metadata-first, make replay/network bodies opt-in, deny or allowlist headers, redact URL/query/referrer/console values before projection, seed PII canaries across every browser surface, and record docs-vs-registry version drift. |
| [MCP tools specification `2025-11-25`](https://modelcontextprotocol.io/specification/2025-11-25/server/tools) and [RFC 8785 JCS](https://www.rfc-editor.org/rfc/rfc8785.html) | MCP tools can return JSON `structuredContent` validated by `outputSchema`; JCS provides deterministic, hashable JSON. | Frontend projection rows must bind browser privacy canary results to `schema_ref`, post-redaction `canonical_hash`, `projection_manifest`, and MCP `structuredContent`, not only JSON/Markdown text renderings. |
| [OpenTelemetry browser resource semconv](https://opentelemetry.io/docs/specs/semconv/resource/browser/) | Browser resource conventions are development-stage except `user_agent.original`, which is stable/recommended; some fields should be unset if client hints are unavailable. | Store semconv version and avoid treating browser attributes as stable product schema. |

## Claim Levels

Use these levels in `claim-ledger.jsonl`:

| Level | Meaning | Allowed wording |
| --- | --- | --- |
| `not_measured` | No current frontend capture safety run exists. | "Frontend capture is designed but not run-proven." |
| `fixture_harness_ready` | Browser, route, API, source-map, breadcrumb, CORS, and privacy-canary fixtures are repeatable. | "Frontend capture fixture harness prepared." |
| `browser_error_capture_pass` | Browser error/event capture works across the tested browser and route matrix with release/build/session context. | "Frontend error capture works for the tested browser and route matrix." |
| `source_map_symbolication_pass` | Minified stack frames resolve through private Debug-ID-like artifacts; missing/mismatched maps are reported; source maps/source content are not public or agent-dereferenceable by default. | "Frontend errors are source-mapped for the tested build pipeline." |
| `trace_propagation_config_pass` | First-party API calls receive only allowed trace headers, backend CORS permits them, and third-party domains do not. | "Browser trace propagation is configured for the tested first-party routes." |
| `frontend_backend_continuation_pass` | Tested frontend requests continue into backend spans and missing continuations are flagged. | "Frontend-to-backend trace continuation works for the tested first-party routes." |
| `breadcrumb_redaction_pass` | Click/navigation/console/network breadcrumbs preserve useful shapes and leak zero seeded canaries in projections. | "Browser breadcrumbs pass seeded redaction tests for the tested routes." |
| `metadata_privacy_pass` | Default capture excludes raw form values, raw DOM text, cookies, auth headers, user identity, request/response bodies, and sensitive query values. | "Default frontend capture is metadata-first for the tested routes." |
| `browser_export_reliability_pass` | Browser export, browser ingest endpoint posture, payload size limits, CSP/CORS, offline/drop behavior, and client outcome reporting are measured. | "Browser telemetry export behavior is measured for the tested routes." |
| `frontend_overhead_pass` | Capture stays within page-load, interaction, memory, payload, and network overhead budgets. | "Frontend capture overhead is within budget for the tested routes." |
| `replay_opt_in_privacy_pass` | Replay is opt-in, masked by default, blocks media/text/input, and network bodies remain disabled unless allowlisted. | "Replay is privacy-gated for the tested opt-in configuration." |
| `projection_pass` | Agent-visible canonical JSON, Markdown, CLI/HTTP output, and MCP `structuredContent` include redaction reports, source-field policy status, missing-evidence flags, schema/hash/projection metadata, and blocked raw refs without leaking seeded canaries, source content, replay content, or raw refs. | "Frontend evidence projections pass redaction and missing-evidence checks." |
| `frontend_tiny_default_ready` | Error, route, release, source-map, ingest, breadcrumb, export, overhead, metadata privacy, source-field, and projection rows pass for the tiny-tier capture set. | "Parallax captures source-mapped frontend errors and safe breadcrumbs for the tested browser matrix." |
| `frontend_cross_tier_claim_ready` | Tiny default is ready and A4 continuation rows pass for real first-party routes. | "Parallax reconstructs frontend-to-backend paths for the tested first-party routes." |
| `replay_claim_ready` | Replay opt-in privacy, overhead, retention, projection, and access controls pass. | "Parallax can attach privacy-gated replay refs for the tested opt-in configuration." |
| `claim_expired` | SDK, browser, build pipeline, source-map identity, CORS/propagation, capture mode, redaction, projection, or route coverage changed, or 90 days passed. | "Frontend capture result expired; rerun required." |
| `claim_failed` | A required gate fails for the advertised level. | No claim for the failed browser/route/surface. |

Initial Parallax level: `not_measured`.

## Result Artifacts

Create these only for real frontend capture runs:

```text
docs/research/frontend-capture-results.md
docs/research/frontend-capture-runs/<run_id>/manifest.json
docs/research/frontend-capture-runs/<run_id>/browser-route-matrix.jsonl
docs/research/frontend-capture-runs/<run_id>/sdk-config-results.jsonl
docs/research/frontend-capture-runs/<run_id>/error-capture-results.jsonl
docs/research/frontend-capture-runs/<run_id>/source-map-results.jsonl
docs/research/frontend-capture-runs/<run_id>/propagation-results.jsonl
docs/research/frontend-capture-runs/<run_id>/cors-results.jsonl
docs/research/frontend-capture-runs/<run_id>/ingest-endpoint-results.jsonl
docs/research/frontend-capture-runs/<run_id>/breadcrumb-results.jsonl
docs/research/frontend-capture-runs/<run_id>/privacy-canary-results.jsonl
docs/research/frontend-capture-runs/<run_id>/source-field-policy-results.jsonl
docs/research/frontend-capture-runs/<run_id>/export-reliability-results.jsonl
docs/research/frontend-capture-runs/<run_id>/overhead-results.jsonl
docs/research/frontend-capture-runs/<run_id>/replay-privacy-results.jsonl
docs/research/frontend-capture-runs/<run_id>/projection-results.jsonl
docs/research/frontend-capture-runs/<run_id>/claim-ledger.jsonl
docs/research/frontend-capture-runs/<run_id>/hashes.sha256
```

Do not commit raw DOM snapshots, replay segments, source maps, user identifiers,
full URLs with query strings, raw console messages, request/response bodies, or
private incident notes. Use synthetic fixtures or redacted hashes unless the
operator explicitly approves a private retained artifact.

## Run Manifest

```json
{
  "run_id": "frontend-capture-YYYYMMDD-N",
  "research_date": "YYYY-MM-DD",
  "parallax_sdk_commit": "<git-sha>",
  "schema_version": "frontend-capture-safety-v0",
  "otel_semconv_version": "1.41.0",
  "opentelemetry_js_versions": {
    "sdk_trace_web": "x.y.z",
    "instrumentation_fetch": "x.y.z",
    "instrumentation_xhr": "x.y.z"
  },
  "sentry_js_version": "x.y.z",
  "sentry_replay_package_source": "@sentry/browser_internal|@sentry/replay_standalone|none|other",
  "package_version_snapshot": {
    "@sentry/browser": "x.y.z",
    "@sentry/react": "x.y.z",
    "@sentry-internal/replay": "x.y.z|not_present",
    "@sentry-internal/replay-canvas": "x.y.z|not_present",
    "@sentry/replay": "x.y.z|not_present",
    "@opentelemetry/sdk-trace-web": "x.y.z",
    "@opentelemetry/instrumentation-fetch": "x.y.z",
    "@opentelemetry/exporter-trace-otlp-http": "x.y.z",
    "@opentelemetry/exporter-trace-otlp-proto": "x.y.z"
  },
  "docs_package_version_snapshot": {
    "sentry_react_docs_package_detail": "x.y.z|unknown"
  },
  "redaction_policy_version": "a6-default-deny-vN",
  "source_field_policy_version": "phase0-source-field-policy-vN",
  "bundle_schema_ref": {
    "uri": "https://parallax.dev/schemas/evidence-bundle/v0.json",
    "hash": "sha256:...",
    "canonicalization": "jcs-rfc8785"
  },
  "canonical_bundle_hash_algorithm": "sha256 over RFC8785 canonical JSON after frontend redaction",
  "mcp_output_schema_required": true,
  "source_map_identity_version": "debug-id-like-vN",
  "source_maps_public_accessible": false,
  "source_content_agent_visible": false,
  "replay_agent_visible_default": false,
  "replay_network_detail_policy": "disabled|allowlist|vendor_default",
  "replay_masking_config_hash": "sha256:...",
  "frontend_ingest_endpoint": {
    "mode": "same_origin_tunnel|signed_project_token|dsn_public_key|reverse_proxy",
    "public_collector_exposed": false,
    "accepted_paths": ["/v1/traces", "/api/parallax/browser-ingest"],
    "allowed_browser_transports": ["http/protobuf", "http/json"],
    "grpc_negative_fixture_required": true,
    "cors_policy_ref": "cors-results.jsonl",
    "csp_policy_ref": "ingest-endpoint-results.jsonl"
  },
  "raw_ref_policy": "metadata_only_by_default",
  "frontend_build_commit": "<git-sha>",
  "browser_matrix": ["chromium", "firefox", "webkit"],
  "route_count": 0,
  "api_domains": ["https://api.example.test"],
  "capture_modes": ["error_metadata", "breadcrumbs", "trace_propagation", "replay_ref_opt_in"],
  "projection_formats": ["bundle_json", "bundle_markdown", "cli_output", "http_api", "mcp_structuredContent"],
  "budgets": {
    "sdk_bundle_gzip_kb": 40,
    "page_load_delta_p95_ms": 50,
    "interaction_delta_p95_ms": 16,
    "memory_delta_p95_mb": 10,
    "payload_gzip_p95_kb": 64
  },
  "notes": []
}
```

## Row Schemas

### Browser Route Matrix Row

```json
{
  "route_id": "checkout-error-001",
  "browser": "chromium|firefox|webkit",
  "browser_version": "unknown",
  "route": "/checkout",
  "app_kind": "spa|ssr|mpa",
  "api_domain": "https://api.example.test",
  "test_case": "handled_error|unhandled_error|api_error|navigation|form_submit|offline_flush|third_party_request",
  "expected_source_map_debug_id": "uuid-or-hash",
  "expected_backend_continuation": true,
  "seeded_canary_count": 0
}
```

### SDK Config Result Row

```json
{
  "route_id": "checkout-error-001",
  "browser": "chromium",
  "sentry_enabled": true,
  "otel_enabled": true,
  "browser_otlp_transport": "http/protobuf|http/json|none",
  "browser_otlp_content_type": "application/x-protobuf|application/json|none",
  "browser_grpc_attempted": false,
  "trace_propagation_targets": ["https://api.example.test"],
  "third_party_propagation_denied": true,
  "before_send_configured": true,
  "before_breadcrumb_configured": true,
  "before_send_span_configured": true,
  "max_breadcrumbs": 50,
  "replay_default": "disabled|on_error|full_session",
  "sentry_replay_package_source": "@sentry/browser_internal|@sentry/replay_standalone|none|other",
  "sentry_internal_replay_version": "x.y.z|not_present",
  "standalone_sentry_replay_version": "x.y.z|not_present",
  "send_default_pii": false,
  "http_headers_capture_policy": "deny_all|allowlist|vendor_default",
  "url_query_capture_policy": "drop|redact|vendor_default",
  "referrer_capture_policy": "drop|redact|vendor_default",
  "console_breadcrumb_policy": "drop|redact|vendor_default",
  "network_bodies_default": "disabled",
  "pass": true
}
```

### Error Capture Result Row

```json
{
  "route_id": "checkout-error-001",
  "browser": "chromium",
  "error_kind": "handled|unhandled|promise_rejection|api_surface_error",
  "event_received": true,
  "session_id_present": true,
  "route_present": true,
  "release_present": true,
  "build_id_present": true,
  "trace_id_present": true,
  "span_id_present": true,
  "raw_user_identity_present": false,
  "raw_request_headers_present": false,
  "raw_query_string_present": false,
  "raw_referrer_present": false,
  "pass": true
}
```

### Source Map Result Row

```json
{
  "route_id": "checkout-error-001",
  "browser": "chromium",
  "minified_frame_count": 4,
  "debug_id_present": true,
  "artifact_uploaded_before_event": true,
  "artifact_private": true,
  "resolved_frame_count": 4,
  "unresolved_frame_count": 0,
  "mismatch_detected": false,
  "missing_map_reported": false,
  "source_map_publicly_served": false,
  "source_content_in_agent_bundle": false,
  "source_context_agent_visible": false,
  "source_map_ref_dereferenced_by_default": false,
  "artifact_access_scope": "ci_upload_only|scoped_operator|public",
  "pass": true
}
```

### Propagation Result Row

```json
{
  "route_id": "checkout-api-001",
  "browser": "chromium",
  "request_url_class": "first_party_api|same_origin|third_party|collector",
  "traceparent_sent": true,
  "tracestate_sent": false,
  "baggage_sent": true,
  "baggage_keys_allowed": true,
  "baggage_values_opaque": true,
  "sentry_trace_sent": true,
  "third_party_trace_headers_sent": false,
  "third_party_baggage_sent": false,
  "backend_trace_id_matched": true,
  "backend_span_child_or_link": true,
  "missing_continuation_flagged": false,
  "pass": true
}
```

### CORS Result Row

```json
{
  "route_id": "checkout-api-001",
  "browser": "chromium",
  "api_domain": "https://api.example.test",
  "preflight_observed": true,
  "allow_origin_exact": true,
  "allow_credentials_safe": true,
  "allow_headers": ["traceparent", "tracestate", "baggage", "sentry-trace"],
  "project_auth_header_allowed": true,
  "wildcard_with_credentials": false,
  "cors_failure_detected": false,
  "pass": true
}
```

### Ingest Endpoint Result Row

```json
{
  "route_id": "checkout-error-001",
  "browser": "chromium",
  "transport": "sentry_envelope|otlp_http_json|otlp_http_protobuf|parallax_browser_ingest",
  "endpoint_kind": "same_origin_tunnel|reverse_proxy|direct_collector|vendor_ingest",
  "public_collector_exposed": false,
  "accepted_paths_exact": true,
  "admin_or_debug_paths_exposed": false,
  "origin_allowed_exact": true,
  "wildcard_origin": false,
  "credentials_with_wildcard_origin": false,
  "browser_bundle_secret_present": false,
  "auth_mode": "dsn_public_key|signed_project_token|same_origin_cookie|none",
  "request_size_limited": true,
  "rate_limited": true,
  "csp_connect_src_exact": true,
  "content_type_limited": true,
  "pass": true
}
```

### Breadcrumb Result Row

```json
{
  "route_id": "checkout-error-001",
  "browser": "chromium",
  "breadcrumb_kind": "click|keypress|navigation|fetch|console",
  "captured_count": 0,
  "bounded_buffer": true,
  "shape_preserved": true,
  "raw_dom_text_present": false,
  "raw_form_value_present": false,
  "raw_console_message_present": false,
  "raw_url_query_present": false,
  "redacted_count": 0,
  "pass": true
}
```

### Privacy Canary Result Row

```json
{
  "route_id": "checkout-error-001",
  "browser": "chromium",
  "surface": "dom_text|form_input|url_query|request_url|referrer_url|console|breadcrumb|network_header|network_body|user_context|replay|source_map|source_content",
  "capture_mode": "metadata|redacted_excerpt|raw_ref|replay_ref_opt_in",
  "seeded_canaries": 20,
  "canonical_json_leaks": 0,
  "json_projection_leaks": 0,
  "markdown_projection_leaks": 0,
  "cli_output_leaks": 0,
  "http_api_leaks": 0,
  "mcp_structured_content_leaks": 0,
  "raw_ref_leaks": 0,
  "scanner_error_count": 0,
  "scanner_error_behavior": "fail_closed|strip_field|block_bundle",
  "pass": true
}
```

### Source Field Policy Result Row

```json
{
  "route_id": "checkout-error-001",
  "browser": "chromium",
  "source_kind": "direct_production_telemetry|synthetic_fixture|benchmark_fixture|corpus_fixture",
  "source_field_policy_status": "pass|fail|not_applicable",
  "source_field_policy_version": "phase0-source-field-policy-vN",
  "source_field_policy_hash": "sha256:...",
  "denied_zone_count": 0,
  "violation_count": 0,
  "not_applicable_reason": "direct telemetry without mixed eval/corpus source rows",
  "pass": true
}
```

### Export Reliability Result Row

```json
{
  "route_id": "checkout-error-001",
  "browser": "chromium",
  "transport": "sentry_envelope|otlp_http_json|otlp_http_protobuf|parallax_browser_ingest",
  "csp_allows_endpoint": true,
  "origin_allowed": true,
  "public_collector_exposed": false,
  "request_size_limited": true,
  "rate_limited": true,
  "event_sent_count": 10,
  "event_received_count": 10,
  "dropped_event_count": 0,
  "client_drop_reported": true,
  "offline_drop_reported": true,
  "pass": true
}
```

### Overhead Result Row

```json
{
  "route_id": "checkout-error-001",
  "browser": "chromium",
  "capture_mode": "error_metadata+breadcrumbs+trace_propagation",
  "baseline_page_load_p95_ms": 900,
  "observed_page_load_p95_ms": 930,
  "page_load_delta_p95_ms": 30,
  "interaction_delta_p95_ms": 6,
  "memory_delta_p95_mb": 4,
  "sdk_bundle_gzip_kb": 32,
  "payload_gzip_p95_kb": 18,
  "request_count_delta_p95": 1,
  "budget_pass": true
}
```

### Replay Privacy Result Row

```json
{
  "route_id": "checkout-error-001",
  "browser": "chromium",
  "replay_enabled": true,
  "sentry_replay_package_source": "@sentry/browser_internal|@sentry/replay_standalone|none|other",
  "sentry_internal_replay_version": "x.y.z|not_present",
  "standalone_sentry_replay_version": "x.y.z|not_present",
  "trigger": "on_error|full_session|manual",
  "mask_all_text": true,
  "mask_all_inputs": true,
  "block_all_media": true,
  "network_bodies_enabled": false,
  "network_detail_allowlist_count": 0,
  "safe_selector_allowlist_count": 0,
  "raw_dom_leak_count": 0,
  "raw_input_leak_count": 0,
  "raw_media_leak_count": 0,
  "raw_network_body_leak_count": 0,
  "replay_segment_agent_visible": false,
  "agent_bundle_dereference_blocked": true,
  "pass": true
}
```

### Projection Result Row

```json
{
  "route_id": "checkout-error-001",
  "browser": "chromium",
  "projection": "bundle_json|bundle_markdown|cli_output|http_api|mcp_structuredContent",
  "schema_ref_hash": "sha256:...",
  "canonical_bundle_hash": "sha256:...",
  "projection_hash": "sha256:...",
  "projection_manifest_hash": "sha256:...",
  "projection_derives_from_canonical": true,
  "redaction_report_present": true,
  "redaction_report_hash": "sha256:...",
  "source_field_policy_status": "pass|fail|not_applicable",
  "source_field_policy_hash": "sha256:...",
  "source_field_policy_violations": 0,
  "mcp_output_schema_valid": null,
  "mcp_structured_content_hash": null,
  "safety_fields_only_in_meta": false,
  "missing_evidence_present": true,
  "raw_ref_count": 0,
  "replay_ref_count": 0,
  "source_map_ref_count": 1,
  "raw_ref_dereferenced": false,
  "replay_ref_dereferenced": false,
  "source_map_ref_dereferenced": false,
  "source_content_visible": false,
  "seeded_canary_leaks": 0,
  "agent_visible_pass": true
}
```

### Claim Ledger Row

```json
{
  "claim_level": "frontend_tiny_default_ready",
  "status": "pass|fail|expired",
  "run_id": "frontend-capture-YYYYMMDD-N",
  "scope": "chromium-firefox-webkit-spa-checkout",
  "passed_route_count": 0,
  "failed_route_count": 0,
  "blocking_failures": [],
  "allowed_wording": "Parallax captures source-mapped frontend errors and safe breadcrumbs for the tested browser matrix.",
  "expires_on": "YYYY-MM-DD"
}
```

## Counting Rules

- A browser error counts as captured only if it includes route, frontend
  release/build, session id or session hash, stack status, redaction policy, and
  source signal provenance.
- A source-mapped claim requires Debug-ID-like identity, private artifact
  storage, a matching uploaded artifact before the event, and explicit
  missing-map reporting for negative fixtures. It fails if source maps are
  publicly served, if source content is included in agent bundles by default, or
  if an agent can dereference source-map refs without scoped operator approval.
- A frontend fixture, benchmark, or corpus-derived run must include
  `source_field_policy_status: pass` with zero violations before projections can
  pass. Direct production telemetry may use `not_applicable` only when no mixed
  eval/corpus source rows are present, matching the bundle schema rule.
- A frontend-to-backend continuation counts only if the frontend request and
  backend span share trace context, or if Sentry trace context is explicitly
  bridged into the same normalized Parallax trace row.
- Third-party requests must not receive trace headers by default. A single
  third-party propagation leak fails `trace_propagation_config_pass`.
- Baggage counts for session correlation only when keys are allowlisted, values
  are opaque/non-PII, and third-party baggage propagation is absent.
- Cross-origin first-party propagation requires CORS rows showing allowed
  tracing and project auth headers. A frontend span without backend continuation
  must become `missing_backend_continuation`, not "backend not involved."
- Breadcrumbs count only when they preserve event shape without raw DOM text,
  form values, raw query strings, or unfiltered console content.
- Default frontend capture must be metadata-first: no cookies, raw user identity,
  auth headers, raw request/response headers, full URLs, query strings, referrer
  URLs, request/response bodies, form values, raw console messages, raw DOM text,
  or replay segments in agent-visible bundles. Vendor SDK defaults are not
  enough; the run must record the explicit capture policy for each surface.
- A run that mentions Sentry Replay must record Replay provenance:
  `@sentry/browser` internal Replay dependency, standalone `@sentry/replay`, or
  another package/source. Standalone `@sentry/replay` cannot be treated as the
  current Sentry JS v10 Replay source unless the app lockfile includes it.
- Replay is never tiny-tier default. It can become claimable only as an opt-in
  raw/ref surface after masking, network-body, retention, overhead, and
  dereference-denial rows pass.
- Source maps and `sourcesContent` are raw source artifacts. They may support
  server-side symbolication, but public source-map access, source content in
  canonical bundles, or agent-default dereference fails the source-map gate.
- Browser export reliability is best-effort. Dropped, blocked, sampled, offline,
  adblocked, and CSP/CORS-failed events must be counted or surfaced as
  missing-evidence; they cannot be hidden by only counting received events.
- Public ingest must be narrow. A broad unauthenticated collector endpoint fails
  the export gate even if telemetry arrives. Browser bundles must not contain
  private ingest secrets; public keys/DSNs must be treated as identifiers, not
  authority to access retained raw evidence.
- Projection pass requires redaction report presence, source-field policy
  status, missing-evidence flags, zero seeded canary leaks, and no raw source-map
  or replay dereference by default.
- Projection pass also requires `schema_ref`, post-redaction `canonical_hash`,
  `projection_manifest`, and `access` fields on the canonical bundle. A frontend
  projection is incomplete if it cannot be tied back to the exact canonical
  bundle hash that A6 scanned.
- CLI, HTTP, and MCP frontend projections must carry the same canonical bundle
  hash for the same authorized request. A mismatch is a projection failure even
  when each output separately scans clean.
- MCP frontend output counts only when bundle-returning tools validate
  `structuredContent` against the evidence-bundle `outputSchema`. Text-only MCP
  JSON/Markdown is a projection, not a schema-safe frontend bundle.
- Safety fields for frontend evidence cannot live only in MCP `_meta`, tool
  annotations, descriptions, or prompt-wrapper metadata. They must be in the
  canonical bundle JSON.
- `frontend_cross_tier_claim_ready` also requires fresh A4 rows for real or
  fault-injected first-party routes; generator-perfect links are not enough.

## Product Wording

Allowed after `not_measured`:

> Frontend capture is designed but not run-proven.

Allowed after `frontend_tiny_default_ready`:

> Parallax captures source-mapped frontend errors and safe breadcrumbs for the
> tested browser matrix.

Allowed after `frontend_cross_tier_claim_ready`:

> Parallax reconstructs frontend-to-backend paths for the tested first-party
> routes.

Allowed after `replay_claim_ready`:

> Parallax can attach privacy-gated replay refs for the tested opt-in
> configuration.

Avoid:

- "complete user session replay" as a default claim;
- "captures every user step";
- "frontend-to-backend reconstruction for all routes";
- "source maps always resolve";
- "safe RUM/replay" without naming the run, masking mode, and browser matrix;
- "safe source maps" without naming the artifact access mode and projection
  dereference result;
- "zero overhead";
- "agent-visible replay" by default;
- "browser telemetry is reliable" without drop/offline/adblock/CSP/CORS rows.

## Refresh Triggers

Mark affected claims `claim_expired` when:

- OpenTelemetry JS, Sentry JS, browser instrumentations, or browser support
  matrix changes;
- npm registry versions and SDK docs package-detail versions diverge or move
  materially enough to change setup/config assumptions;
- OpenTelemetry browser, HTTP, trace, or resource semantic conventions change
  materially;
- source-map upload, Debug-ID-like identity, bundler, minifier, release, or
  frontend build pipeline changes;
- CORS, CSP, tunnel, ingest endpoint, auth header, trace propagation target, or
  backend extraction config changes;
- frontend route structure, router, rendering mode, API domain, or deployment
  topology changes;
- privacy/redaction policy, source-field policy, breadcrumb filtering, replay
  masking, network-body policy, bundle schema, canonicalization, projection
  manifest, MCP output schema, or raw-ref access control changes;
- a new browser, device class, or route class enters product wording;
- 90 days pass since the last run during active development.

## Relationship To Other Research

- [Frontend collection and cross-tier correlation](frontend-collection-and-cross-tier-correlation.md)
  defines the browser capture and cross-tier architecture this ledger measures.
- [Correlation reliability on real telemetry gate](correlation-reliability-real-telemetry-gate.md)
  consumes continuation and missing-evidence rows when deciding whether A4
  frontend-to-backend reconstruction can pass.
- [A4 correlation reliability ledger](a4-correlation-reliability-ledger.md)
  owns real-anchor correlation claim levels; this ledger supplies browser-side
  capture, source-map, CORS, and privacy rows.
- [Redaction pipeline and secret safety](redaction-pipeline-and-secret-safety.md)
  defines the default-deny frontend privacy posture used by this ledger.
- [A6 redaction red-team ledger](a6-redaction-red-team-ledger.md) remains the
  broader redaction veto before frontend evidence becomes agent-visible.
- [A6 synthetic canary fixture corpus](a6-synthetic-canary-fixture-corpus.md)
  defines the seeded frontend canaries for Replay, source maps, URL/query,
  referrer, console, headers, and network-body surfaces.
- [Redaction detector toolchain](redaction-detector-toolchain.md) defines the
  scanner and canary comparators used for browser privacy rows.
- [Frontend Replay and source-map privacy recheck](frontend-replay-sourcemap-privacy-recheck.md)
  defines the Replay package-provenance fields and source-artifact negative
  fixtures required by this ledger.
- [Evidence bundle and open schema specification](evidence-bundle-and-schema.md)
  must expose frontend nodes, redaction reports, source-field policy status,
  source-map status, replay refs, and missing-evidence warnings without leaking
  denied fields.
- [Storage benchmark prototype](storage-benchmark-prototype.md) includes Q4
  `cross_tier`; this ledger defines the capture proof behind that generated
  shape.
- [Technical implementation concept](technical-implementation-concept.md)
  should treat this ledger as the claim boundary for frontend capture.

## Bottom Line

Frontend capture is essential to the Parallax thesis because many failures are
user-facing and cross the browser/backend boundary. It is also the highest-PII
telemetry surface. The safe default is source-mapped errors, route/release
metadata, bounded redacted breadcrumbs, and first-party trace propagation. Replay
and raw browser content stay opt-in refs. Product claims start only after this
ledger proves the browser matrix, source-map identity and access controls,
browser ingest posture, source-field policy status, propagation, privacy, export
reliability, overhead, canonical hashes, and projection equivalence.
