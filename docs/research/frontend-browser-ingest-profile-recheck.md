# Frontend Browser Ingest Profile Recheck

<!-- markdownlint-disable MD013 -->

Research date: 2026-05-25

## Pass Target

Re-check the weakest frontend claim boundary: whether Parallax can reuse the
backend OTLP transport profile for browser telemetry. The answer is no. Browser
capture is still OTLP/Sentry compatible, but it needs its own ingest profile
because browser clients cannot use OTLP/gRPC, are constrained by CSP/CORS, and
run with public credentials and high-PII surfaces.

This pass does not benchmark browser overhead or export reliability. It tightens
the source-checked transport, CORS, source-map, replay, and agent-visible safety
requirements that the frontend ledger must prove later.

## Short Verdict

Keep the existing frontend architecture: Sentry browser envelopes for rich
frontend errors/breadcrumbs plus OpenTelemetry JS fetch/XHR spans for cross-tier
trace continuity.

Narrow the transport claim:

- Backend/server OTLP baseline remains `grpc` plus `http/protobuf`.
- Browser OTLP baseline is HTTP only: `http/json` or `http/protobuf` over a
  browser-specific endpoint or reverse proxy.
- A browser failing the `grpc` fixture is expected, not a frontend capture
  failure.
- A public generic Collector endpoint is the wrong default. Parallax needs a
  browser ingest boundary that enforces origin allowlists, path allowlists,
  public-project auth/DSN semantics, request size/rate limits, CSP/CORS,
  redaction, and drop/outcome reporting.

The current package recheck did not find version drift from the ledger snapshot:
`@sentry/browser` and `@sentry/react` are still `10.53.1`; OTel web SDK remains
`2.7.1`; OTel fetch/XHR and OTLP HTTP exporters remain `0.218.0`. A follow-up
[frontend Replay and source-map privacy recheck](frontend-replay-sourcemap-privacy-recheck.md)
adds one important package nuance: `@sentry/browser` `10.53.1` contains internal
Replay packages at `10.53.1`, while standalone `@sentry/replay` is still
`7.116.0`; record which path an actual app uses.

## Current Primary-Source Snapshot

| Source | Current signal checked 2026-05-25 | Parallax implication |
| --- | --- | --- |
| npm registry for `@sentry/browser` and `@sentry/react` | `npm view` reports `10.53.1`, modified 2026-05-12. | Keep pinning exact Sentry JS package versions in every browser run; docs pages can lag package releases. |
| [Frontend Replay and source-map privacy recheck](frontend-replay-sourcemap-privacy-recheck.md) | `@sentry/browser` `10.53.1` depends on `@sentry-internal/replay` `10.53.1`; standalone `@sentry/replay` is `7.116.0`, modified 2025-11-25. | Replay package provenance is a manifest field, not an assumption. Do not use stale standalone package metadata for current v10 Replay behavior unless the lockfile includes it. |
| npm registry for `@opentelemetry/sdk-trace-web` | `npm view` reports `2.7.1`, modified 2026-05-01. | Browser trace fixtures should record OTel JS core/web SDK separately from instrumentation package versions. |
| npm registry for `@opentelemetry/instrumentation-fetch`, `@opentelemetry/instrumentation-xml-http-request`, and OTLP HTTP trace exporters | `npm view` reports `0.218.0` for fetch, XHR, HTTP/JSON, HTTP/protobuf, and gRPC exporter packages; HTTP exporter packages modified 2026-05-13. | Run manifests need the full package matrix; exporter and instrumentation packages are on the experimental `0.x` line even when the web SDK is `2.x`. |
| [OpenTelemetry JavaScript browser guide](https://opentelemetry.io/docs/languages/js/getting-started/browser/) | The official browser guide uses `@opentelemetry/sdk-trace-web` and document-load instrumentation for browser spans. | OTel JS remains the right reference path for browser spans, but browser support must be fixture-proven per package/browser matrix. |
| [OpenTelemetry JavaScript exporters](https://opentelemetry.io/docs/languages/js/exporters/) | Browser usage explicitly cannot use OTLP/gRPC; it is limited to OTLP HTTP/JSON or HTTP/protobuf and must handle CSP, CORS, and public collector exposure risk. | The browser profile must be separate from the backend OTLP profile and must prefer a narrow Parallax/proxy ingest endpoint. |
| Published `@opentelemetry/instrumentation-fetch@0.218.0` tarball and [Fetch config docs](https://open-telemetry.github.io/opentelemetry-js/interfaces/_opentelemetry_instrumentation-fetch.FetchInstrumentationConfig.html) | `FetchInstrumentationConfig` still contains `propagateTraceHeaderCorsUrls`, `ignoreUrls`, `requestHook`, `applyCustomAttributesOnSpan`, `measureRequestSize`, and `semconvStabilityOptIn`. Runtime code skips header injection when CORS propagation policy denies a URL. | Existing frontend docs are directionally correct. Pin package types in fixtures instead of trusting only examples or overview docs. |
| Published `@opentelemetry/instrumentation-xml-http-request@0.218.0` tarball and [XHR config docs](https://open-telemetry.github.io/opentelemetry-js/interfaces/_opentelemetry_instrumentation-xml-http-request.XMLHttpRequestInstrumentationConfig.html) | XHR config also has `propagateTraceHeaderCorsUrls`, `ignoreUrls`, `applyCustomAttributesOnSpan`, `measureRequestSize`, and `semconvStabilityOptIn`; runtime code uses the same header-propagation gate. | Fetch and XHR propagation must be tested separately; SPAs may use either surface. |
| Published `@opentelemetry/exporter-trace-otlp-http@0.218.0` tarball | Browser build maps to a browser platform exporter and sets `Content-Type: application/json` with default `v1/traces`. | HTTP/JSON is a real browser path but should be labeled distinctly from backend `http/protobuf`. |
| Published `@opentelemetry/exporter-trace-otlp-proto@0.218.0` tarball | Browser build maps to a browser platform exporter and sets `Content-Type: application/x-protobuf` with default `v1/traces`. | Browser `http/protobuf` should be the preferred OTel path when the ingest endpoint supports it. |
| [Sentry JavaScript automatic instrumentation](https://docs.sentry.io/platforms/javascript/tracing/instrumentation/automatic-instrumentation/) and [Sentry JavaScript options](https://docs.sentry.io/platforms/javascript/configuration/options/#trace-propagation-targets) | `tracePropagationTargets` controls outgoing `sentry-trace` and `baggage`; Sentry warns backend CORS must allow those headers. Current docs show default targets such as localhost and same-origin paths. | Sentry trace context must be bridged to Parallax rows, and first-party/third-party propagation needs explicit allowlist tests. |
| [MDN Access-Control-Allow-Headers](https://developer.mozilla.org/docs/Web/HTTP/Reference/Headers/Access-Control-Allow-Headers) | Browsers use preflight responses to decide whether non-safelisted request headers can be sent. | `traceparent`, `tracestate`, `baggage`, `sentry-trace`, `content-type`, and project auth headers require CORS fixtures. |
| [Sentry JavaScript data collected](https://docs.sentry.io/platforms/javascript/guides/react/data-management/data-collected/) | Sentry documents privacy-relevant defaults and surfaces such as headers, URLs/query strings, referrers, console logs/breadcrumbs, and Replay masking defaults. | Browser metadata must be default-deny in Parallax projections regardless of vendor defaults. |
| [Sentry source-map upload guidance](https://docs.sentry.io/platforms/javascript/guides/tanstackstart-react/sourcemaps/uploading/esbuild/) and artifact-bundle docs | Sentry recommends upload flows and warns that source maps can expose source when publicly served. | Source maps are raw source artifacts, not agent-visible evidence; Parallax should store private Debug-ID-like source-map refs and test public-map negatives. |

## Required Browser Ingest Profile

| Surface | Required v0 behavior | Claim boundary |
| --- | --- | --- |
| Sentry browser envelope | Accept frontend error events, breadcrumbs, release/build/session context, and Sentry trace context. Unsupported side items follow the Sentry envelope item policy. | "Frontend Sentry-compatible error ingest" only after browser SDK fixtures pass. |
| OTel browser traces | Accept OTLP/HTTP `application/x-protobuf` and optionally OTLP/HTTP JSON for browser spans. | "Browser OTLP" must name HTTP/protobuf or HTTP/JSON; never imply browser gRPC. |
| OTLP/gRPC | Explicit negative fixture for browser builds. | Expected unsupported path; not a frontend failure. |
| Browser endpoint | Same-origin tunnel, signed/public project token, DSN-like project key, or reverse proxy in front of a collector-compatible path. | No broad unauthenticated public Collector claim. |
| CORS | Explicit preflight allowlist for trace/context/export headers and content type. | Missing CORS is a missing-evidence condition, not proof that no backend continuation happened. |
| CSP | `connect-src` must include the browser ingest endpoint. | Export reliability claim fails if CSP blocks telemetry. |
| Source maps | Private Debug-ID-like artifact identity, no public `.map` files, no source contents in agent bundles by default. | Source-mapped claim requires private artifact tests and public-map negative tests. |
| Replay | Opt-in, masked by default, reference-only in bundles unless separately approved; manifest records the actual Replay package source. | Replay is outside the tiny tier and outside agent-visible defaults. |

## Fixture Additions

Add these rows to the frontend capture ledger before any product wording:

| Fixture | Expected result |
| --- | --- |
| `browser_otlp_http_protobuf_export` | Browser exporter sends `/v1/traces` with `Content-Type: application/x-protobuf`; Parallax accepts or returns a clear non-retryable config error. |
| `browser_otlp_http_json_export` | Browser exporter sends `/v1/traces` with `Content-Type: application/json`; result is either supported and labeled or rejected without retry ambiguity. |
| `browser_otlp_grpc_negative` | Browser build cannot use OTLP/gRPC; docs and tests classify this as expected unsupported. |
| `browser_ingest_public_collector_denied` | Direct broad public Collector exposure fails the safety gate unless fronted by the approved proxy/auth/rate/redaction layer. |
| `cors_trace_headers_allowed` | First-party API preflight permits `traceparent`, `tracestate`, `baggage`, `sentry-trace`, and needed project/export headers. |
| `cors_trace_headers_missing` | Missing CORS headers produce an explicit missing-continuation/missing-evidence flag. |
| `third_party_propagation_denied` | Fetch/XHR to third-party domains receives no trace or baggage headers. |
| `otel_fetch_and_xhr_propagation_separate` | Fetch and XHR instrumentation are validated independently because package types and runtime patches are separate. |
| `sentry_trace_context_bridge` | Sentry `sentry-trace`/`baggage` context is normalized into Parallax frontend rows and joinable to OTel trace identifiers where configured. |
| `source_map_public_negative` | Public `.js.map` or source-content access fails the source-map safety gate. |
| `replay_masked_opt_in` | Replay remains disabled by default; opt-in replay refs are masked and blocked from agent projections unless explicitly permitted. |

## Uncertainty

- The published OTel package tarballs prove the current type/runtime surface for
  `0.218.0`, but OTel browser packages are still a fast-moving surface. Fixture
  runs expire when package versions, target browsers, bundlers, or CSP/CORS
  policy changes.
- The Sentry docs are authoritative for current configuration guidance, but
  exact defaults can differ by SDK/framework integration. Parallax must record
  the lockfile and generated SDK config used in each run.
- Browser telemetry remains best-effort: blockers, privacy modes, offline
  shutdown, CSP, and failed preflights can drop data.

## Falsification Triggers

Reopen this note if:

- OTel JS browser exporters add a supported browser gRPC path;
- OTel fetch/XHR instrumentation removes or renames propagation allowlist
  controls;
- Sentry changes browser trace propagation defaults or header names;
- CORS/CSP browser behavior changes for telemetry export;
- real frontend fixtures show browser HTTP/protobuf is unreliable enough that
  HTTP/JSON must become the default;
- a competitor proves safer browser-to-backend reconstruction with fewer public
  ingest and privacy risks.

## Sources

- [OpenTelemetry JavaScript browser getting started](https://opentelemetry.io/docs/languages/js/getting-started/browser/)
- [OpenTelemetry JavaScript exporters](https://opentelemetry.io/docs/languages/js/exporters/)
- [OpenTelemetry fetch instrumentation docs](https://open-telemetry.github.io/opentelemetry-js/modules/_opentelemetry_instrumentation-fetch.html)
- [OpenTelemetry fetch instrumentation config](https://open-telemetry.github.io/opentelemetry-js/interfaces/_opentelemetry_instrumentation-fetch.FetchInstrumentationConfig.html)
- [OpenTelemetry XHR instrumentation config](https://open-telemetry.github.io/opentelemetry-js/interfaces/_opentelemetry_instrumentation-xml-http-request.XMLHttpRequestInstrumentationConfig.html)
- [OpenTelemetry Protocol Exporter spec](https://opentelemetry.io/docs/specs/otel/protocol/exporter/)
- [Frontend Replay and source-map privacy recheck](frontend-replay-sourcemap-privacy-recheck.md)
- [npm `@sentry/browser`](https://www.npmjs.com/package/@sentry/browser)
- [npm `@sentry/react`](https://www.npmjs.com/package/@sentry/react)
- [npm `@opentelemetry/sdk-trace-web`](https://www.npmjs.com/package/@opentelemetry/sdk-trace-web)
- [npm `@opentelemetry/instrumentation-fetch`](https://www.npmjs.com/package/@opentelemetry/instrumentation-fetch)
- [npm `@opentelemetry/instrumentation-xml-http-request`](https://www.npmjs.com/package/@opentelemetry/instrumentation-xml-http-request)
- [npm `@opentelemetry/exporter-trace-otlp-http`](https://www.npmjs.com/package/@opentelemetry/exporter-trace-otlp-http)
- [npm `@opentelemetry/exporter-trace-otlp-proto`](https://www.npmjs.com/package/@opentelemetry/exporter-trace-otlp-proto)
- [Sentry JavaScript automatic instrumentation](https://docs.sentry.io/platforms/javascript/tracing/instrumentation/automatic-instrumentation/)
- [Sentry JavaScript `tracePropagationTargets`](https://docs.sentry.io/platforms/javascript/configuration/options/#trace-propagation-targets)
- [Sentry React data collected](https://docs.sentry.io/platforms/javascript/guides/react/data-management/data-collected/)
- [Sentry source-map upload guidance](https://docs.sentry.io/platforms/javascript/guides/tanstackstart-react/sourcemaps/uploading/esbuild/)
- [Sentry artifact bundles and Debug IDs](https://docs.sentry.io/platforms/javascript/guides/cloudflare/sourcemaps/troubleshooting_js/artifact-bundles/)
- [MDN `Access-Control-Allow-Headers`](https://developer.mozilla.org/docs/Web/HTTP/Reference/Headers/Access-Control-Allow-Headers)

## Bottom Line

Browser capture still serves Parallax's goal: it supplies the missing
user-facing side of the runtime lifecycle and links it to backend evidence.
The claim must be narrower than "same OTLP as the backend": browser telemetry is
HTTP-only, CORS/CSP-bound, public-ingest-facing, source-map-sensitive, and
privacy-heavy. That makes a separate browser ingest profile and ledger fixtures
mandatory before Parallax calls frontend reconstruction proven.
