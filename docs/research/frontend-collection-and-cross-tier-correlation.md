# Frontend Collection and Cross-Tier Correlation

<!-- markdownlint-disable MD013 -->

Research date: 2026-05-25

## Purpose

This answers the prompt's confirmed frontend extension: Parallax must collect
from the frontend, not only the backend, and correlate frontend evidence with the
backend and the rest of the microservices architecture, because a large share of
incidents are user-facing and the real cause usually crosses the tier boundary.

It covers the collection method, cross-tier trace propagation (the hard core),
the schema extension, source-map symbolication, and the privacy problem, with an
honest account of what frontend telemetry cannot do.

Scope note (unchanged): the frontend is a telemetry **source** — a JS/TS browser
client SDK. The Parallax engine and infrastructure stay Rust-first and within the
Rust/Go/Zig/C++/C filter. Nothing here adds a JS dependency to the Parallax core.

Version-freshness note: frontend SDK versions move fast. Pin exact SDK versions at
build time; this document fixes the architecture, not specific minor versions.

The companion [frontend capture safety ledger](frontend-capture-safety-ledger.md)
defines the browser/route run artifacts, source-map rows, CORS/propagation
checks, privacy canaries, overhead budgets, replay policy, and claim levels
required before this architecture becomes product wording.

The focused
[frontend browser ingest profile recheck](frontend-browser-ingest-profile-recheck.md)
separates browser telemetry from the backend OTLP transport profile: browser
OTLP is HTTP-only (`http/protobuf` preferred, HTTP/JSON optional), while gRPC is
expected unsupported in browser builds.

The focused
[frontend Replay and source-map privacy recheck](frontend-replay-sourcemap-privacy-recheck.md)
narrows the Replay/source-artifact boundary: current Sentry JS v10 Replay
provenance comes through `@sentry/browser` internal packages, standalone
`@sentry/replay` is stale unless a lockfile includes it, and Replay/source maps
are raw refs rather than agent-visible defaults.

## Current Primary-Source Checks

The frontend direction rests on current official docs, not only vendor blog
examples:

| Area | Current source signal | Parallax implication |
| --- | --- | --- |
| Browser tracing | OpenTelemetry's browser guide uses `@opentelemetry/sdk-trace-web` and browser instrumentations such as document-load; it also warns browser client instrumentation is experimental and mostly unspecified. | Treat browser OTel as viable for traces, but pin SDK versions and keep compatibility tests for target browsers before product wording. |
| Browser export | OpenTelemetry's JS exporter docs say browser apps cannot use OTLP/gRPC and must use OTLP/HTTP JSON or protobuf. They also call out CSP, CORS, and the risk of exposing a collector publicly. | Parallax frontend ingest should expose a narrow OTLP/HTTP-compatible web endpoint or reverse-proxied collector path, never a broad unauthenticated collector. |
| Trace propagation | W3C Trace Context defines `traceparent`/`tracestate` as the standard cross-system trace context, with `traceparent` carrying trace identity in a portable format. | Frontend-to-backend joins should use W3C trace context for OTEL paths. |
| Baggage propagation | OpenTelemetry baggage can carry arbitrary key/value context downstream, and official docs warn that sensitive baggage can reach unintended resources such as third-party APIs. | Session correlation should use only allowlisted, opaque, first-party-scoped baggage keys; raw user/account IDs, emails, tokens, or third-party propagation fail the frontend safety gate. |
| Fetch instrumentation | OpenTelemetry's fetch instrumentation config includes `propagateTraceHeaderCorsUrls`, request hooks, ignored URLs, and custom span attributes. | Propagation must be allowlisted by first-party API domain; do not leak trace headers to arbitrary third parties. |
| Sentry browser tracing | Sentry's JS tracing docs use `tracePropagationTargets` and propagate `sentry-trace` plus `baggage`; they explicitly warn JavaScript apps need those headers in the CORS allowlist. | For Sentry-compatible frontend errors, preserve Sentry trace context and bridge it into the Parallax correlation model. |
| Breadcrumbs | Sentry's browser SDK records automatic breadcrumbs for UI events, XHR/fetch, console calls, and location changes, with hooks for filtering or dropping them. | Breadcrumbs are essential, but Parallax must filter and redact at capture and bundle-build time. |
| Source maps | Sentry's current source-map flow uses artifact bundles and Debug IDs to bind minified JavaScript to source maps without path guessing. | Parallax should adopt a Debug-ID-like source-map identity, keyed to frontend release/build, stored privately in object storage. |
| Browser privacy defaults | Sentry's JavaScript data-collected docs say cookies, logged-in user identity, user IP, client-side request bodies, and response bodies are not sent by default, but HTTP request/response headers, full request URLs, full query strings, referrer URLs, and console logs/breadcrumbs may be collected; Replay masks text/images/user input by default and network bodies are opt-in. | Do not rely on vendor defaults as the Parallax safety boundary; record explicit header, URL/query, referrer, console, body, replay, and user-context capture policies before browser evidence becomes agent-visible. |
| Replay privacy | Sentry Replay defaults to masking DOM text/user input/images and makes network request/response bodies opt-in. | Replay is a useful reference but must be opt-in, masked by default, and outside the tiny tier. |
| Replay package provenance | The current npm snapshot shows `@sentry/browser` `10.53.1` depending on `@sentry-internal/replay` `10.53.1`, while standalone `@sentry/replay` remains `7.116.0`. | Record the actual app lockfile and Replay package source before citing Replay defaults; do not use stale standalone package metadata as current v10 evidence. |
| Browser semantic attributes | OpenTelemetry browser resource semantic conventions are still development-stage for most `browser.*` fields; `user_agent.original` is stable/recommended. | Store browser attributes, but keep the schema additive and versioned. |

## Collection Method

Mirror the backend's dual-API decision (OTLP for telemetry, Sentry envelope for
errors) on the frontend, because the browser ecosystem already speaks both.

| Signal | Essential? | How | Notes |
| --- | --- | --- | --- |
| Frontend error/exception | Essential | Sentry browser envelope and/or OTLP log/event | Must be source-mapped (see symbolication). The single highest-value frontend signal. |
| Outbound request spans (fetch/XHR) | Essential | OTel JS fetch/XHR auto-instrumentation emitting OTLP | These carry the `traceparent` that links to the backend. Without them there is no cross-tier join. |
| User-step breadcrumbs | Essential | SDK breadcrumb buffer (clicks, navigation, console, network) | "What previous steps led here." Bounded ring buffer attached to the error. |
| Route / view context | Essential | SPA router hook → span/attribute | Current route, component, and feature flags at error time. |
| Frontend release/build | Essential | build-time injected release + build id | Joinable to, but distinct from, backend release. |
| Web Vitals / RUM | Nice-to-have | `web-vitals` → OTLP metrics | LCP/INP/CLS for latency-class user issues; not needed for error reconstruction. |
| Session replay | Nice-to-have (opt-in) | rrweb-style DOM recording (Sentry Replay is the GA reference; record package provenance from the actual lockfile) | High value for humans, heavy on privacy and bytes; opt-in, masked by default, later tier. |

Recommendation: tiny tier ships error + fetch/XHR spans + breadcrumbs + route +
release + `traceparent` propagation. Web Vitals and replay are later, opt-in.
Browser OTLP export must use OTLP/HTTP JSON or protobuf; gRPC is not a browser
option. Put Parallax or a reverse proxy in front of any collector-compatible
endpoint to enforce origin allowlists, DSN/project auth, request size limits,
rate limits, and redaction.

Treat this as a separate browser ingest profile, not as a weakening of the
backend OTLP baseline. Backend/server OTLP still needs gRPC and HTTP/protobuf;
browser clients must prove HTTP/protobuf or explicitly labeled HTTP/JSON over a
CORS/CSP-safe endpoint.

## Cross-Tier Trace Propagation (The Core)

Cross-origin browser→backend tracing needs **three layers coordinated**, and it
fails silently if any one is missing:

1. **Browser SDK** injects the W3C `traceparent` (and optionally `tracestate`,
   `baggage`) on outgoing fetch/XHR, restricted to first-party API domains via an
   allowlist (OTel JS: `propagateTraceHeaderCorsUrls`).
2. **Backend CORS** must explicitly allow the `traceparent`, `tracestate`, and
   `baggage` request headers. If it does not, the browser strips them from the
   preflight and propagation **fails silently** — no error, just disconnected
   traces. This is the number-one frontend-tracing footgun.
3. **Backend OTel SDK** extracts the incoming context and creates child spans, so
   the browser span and the backend spans share one `trace_id`.

```text
browser fetch(/api/checkout)
  -- traceparent: 00-<trace_id>-<span_id>-01 --> API gateway
       CORS allows traceparent/tracestate/baggage
       backend service extracts context, continues same trace_id
         emits spans/logs/errors under the same trace
Parallax joins: frontend_error.trace_id == backend spans.trace_id
```

Carry a stable opaque session key in `baggage` so all spans of one user session
are groupable even across page loads and multiple backend calls. It must be
first-party scoped, non-PII, allowlisted, and stripped from third-party
propagation; raw user IDs, account IDs, emails, or tokens fail the safety gate.

Hard parts to design around:

- **Silent CORS failure** (above) — Parallax should detect and flag "frontend
  span present, no backend continuation" as a missing-evidence condition, so the
  gap is visible rather than mistaken for "no backend involvement."
- **Sampling coherence.** The browser decides head sampling without knowing
  whether the backend will keep the trace; mismatched sampling breaks joins.
  Prefer consistent/parent-based sampling, or tail sampling at the collector so
  error traces are always kept end-to-end.
- **Third-party and adblock gaps.** Don't propagate `traceparent` to third-party
  domains (privacy + CORS). Ad/tracker blockers drop some beacons; treat frontend
  capture as best-effort, never complete.
- **Clock skew.** Browser clocks are unreliable; order cross-tier events by trace
  topology (span parentage) and server timestamps, not by browser wall-clock.

## Schema Extension

Extends [the evidence bundle schema](evidence-bundle-and-schema.md) additively —
new node types and cross-tier edges, no breaking change.

New node types:

| Node `type` | Key `data` fields |
| --- | --- |
| `frontend_session` | `session_id`, `started_at`, `user_agent`, `device`, `frontend_release`, `route_entry`, consent flags |
| `user_step` (breadcrumb) | `ts`, `kind` (click/nav/console/network), `target`, `route`, redacted detail |
| `frontend_error` | `error_type`, `message`, `stack` (source-mapped frames), `route`, `trace_id`, `span_id`, `frontend_release`, `handled` |
| `route_view` | `route`, `component`, `feature_flags`, `enter_ts`, `exit_ts` |
| `frontend_release` | `version`, `build_id`, `source_map_ref`, `published_at` |

New cross-tier edges:

| Edge `type` | Meaning | Strength |
| --- | --- | --- |
| `frontend_request_to_span` | A frontend fetch/XHR span continues into a backend span (shared `trace_id`). | strong |
| `session_contains_error` | A frontend error occurred within a session. | strong |
| `step_precedes_error` | A breadcrumb/user-step happened before the error (ordered path). | strong |
| `frontend_error_caused_by_backend` | A backend error/span on the same trace is the source of the user-facing error. | medium |
| `frontend_release_regression` | Frontend error fingerprint first appeared at a frontend release. | medium |

This makes a single bundle span the user's frontend session and the backend
lifecycle it triggered.

## Cross-Tier Reconstruction Query

The "how did we get to this user-facing error" query crosses the boundary: from a
`frontend_error`, take its `trace_id`, fetch the frontend session's preceding
user-steps, and follow `frontend_request_to_span` into the backend spans/logs/
errors on the same trace. This is benchmark query **Q4 `cross_tier`** in
[the storage benchmark prototype](storage-benchmark-prototype.md) — frontend
collection is exactly why that query exists, and why the dataset generator links a
fraction of frontend sessions into backend traces.

## Source-Map Symbolication

Frontend stacks are minified and useless raw. Mirror the Rust debuginfo story
([Rust data collection](rust-data-collection-and-instrumentation.md)):

- upload source maps at build time, keyed by `frontend_release` + `build_id` +
  a Debug-ID-like artifact identifier, to Parallax object storage;
- symbolicate frontend errors server-side at ingest/enrich, never ship source maps
  to the browser;
- **never serve source maps from a public URL** — they expose source. Store them
  in Parallax's object storage behind auth, like backend debug info.
- do not expose raw source maps, `sourcesContent`, or source context to agents by
  default; agent-visible bundles should contain symbolicated frames, artifact
  identity, and non-dereferenceable refs unless scoped operator approval exists.

Sentry's current artifact-bundle model is the right reference: bind minified
source and source map by a Debug ID rather than relying only on path matching.
Parallax should copy the idea, not the Sentry product dependency.

## Privacy (The Hardest Part)

The frontend carries far heavier PII than the backend — form values, URLs with
tokens, DOM content, user identity. Privacy must be designed in, not bolted on:

- **Default-deny on values.** Capture event *shapes* (a click on element X,
  navigation to route Y) not raw values; opt-in to capture content.
- **Replay masking by default.** If replay is enabled, mask all text and block
  media by default; unmask only explicitly safe selectors. (Sentry Replay's
  mask-by-default model is the reference.)
- **Replay as ref, not context.** Replay segments are opt-in raw refs. Agents
  should receive metadata, redaction reports, and blocked refs by default, not
  replay content.
- **Network redaction.** Strip auth headers and redact request/response bodies in
  breadcrumbs and spans; allowlist safe fields.
- **Browser metadata redaction.** Strip or redact full request URLs, query
  strings, referrer URLs, raw request/response headers, and raw console messages;
  vendor SDK defaults are not enough because several of these metadata surfaces
  may be collected even when bodies and logged-in identity are not.
- **Baggage allowlist.** Keep only opaque first-party correlation keys in
  propagated baggage; treat raw user/session/account values or third-party
  baggage propagation as a privacy failure, not as successful correlation.
- **Consent and DNT.** Honor consent state and Do-Not-Track; gate replay/RUM on
  consent; record the consent state in the `frontend_session` node.
- **Data minimization + retention.** Shorter retention for replay/session data;
  redaction happens at bundle-build time so agents never see raw PII (the
  `redaction_report` in the bundle schema applies to frontend nodes too).

This is harder than backend redaction and is a real adoption gate: a self-hosted,
data-owned posture is itself part of the answer (frontend PII never leaves the
team's infrastructure), which is a Parallax advantage over SaaS RUM.

## Honest Limits

- Frontend capture is **best-effort**: adblockers, privacy modes, crashes before
  flush, and offline all drop data. Never assume completeness.
- A frontend error alone rarely proves root cause — the value is the **join** to
  backend evidence; if propagation/CORS is broken, you get half the picture.
- Replay is high-signal for humans but expensive and privacy-heavy; it is not
  required for agent reconstruction and should not be in the tiny tier.
- Web Vitals/RUM answer latency-class questions, not error causality; keep them
  out of the critical error-reconstruction path.

## Reuse vs Incumbents

- **Sentry browser SDK + rrweb-based Replay** is the strongest reference for
  error capture, breadcrumbs, and masked replay; the envelope format is one
  Parallax already accepts on the backend, so the frontend error path is the same
  ingestion contract.
- **OpenTelemetry JS** (fetch/XHR instrumentation + W3C propagators) is the
  reference for the cross-tier span path. Use it for traces; use the Sentry
  envelope for rich errors. This is the same dual-standard stance as the backend.

## MVP Scope (Tiny Tier)

Ship: frontend error (source-mapped) + fetch/XHR spans with `traceparent`
propagation + bounded breadcrumbs + route/release context, joined to backend by
`trace_id`. Defer: session replay, Web Vitals dashboards, full RUM. Prove the
cross-tier join on one frontend↔backend path before broadening. The real-data
pass/fail threshold for that claim lives in
[Correlation reliability on real telemetry gate](correlation-reliability-real-telemetry-gate.md),
with row-level proof captured by the
[A4 correlation reliability ledger](a4-correlation-reliability-ledger.md).

## Relationship To Other Research

- [Evidence bundle and open schema specification](evidence-bundle-and-schema.md)
  — the schema this extends.
- [Storage benchmark prototype](storage-benchmark-prototype.md) — the cross-tier
  query (Q4) and the frontend-linked dataset.
- [Rust data collection and instrumentation](rust-data-collection-and-instrumentation.md)
  — the backend capture analog and the symbolication pattern.
- [OpenTelemetry protocol and context layer](opentelemetry-protocol-and-context-layer.md)
  — OTLP and propagation foundation.
- [Causal reconstruction and agent safety](causal-reconstruction-and-agent-safety.md)
  — how cross-tier edges and missing-evidence flags feed safe agent reasoning.
- [Correlation reliability on real telemetry gate](correlation-reliability-real-telemetry-gate.md)
  — the A4 gate for real frontend-backend continuation and missing-evidence
  reporting.
- [A4 correlation reliability ledger](a4-correlation-reliability-ledger.md)
  — the run artifact schema that proves frontend continuation rates came from
  real anchors, not generator-perfect traces.
- [Frontend capture safety ledger](frontend-capture-safety-ledger.md) — the
  browser-side result contract for source maps, CORS, breadcrumbs, privacy,
  export reliability, overhead, replay refs, and projection safety.
- [Frontend browser ingest profile recheck](frontend-browser-ingest-profile-recheck.md)
  — current browser Sentry/OTel package and transport-profile recheck; separates
  browser HTTP-only ingest from backend OTLP gRPC/protobuf requirements.
- [Frontend Replay and source-map privacy recheck](frontend-replay-sourcemap-privacy-recheck.md)
  — current Replay package provenance and source-artifact raw-ref boundary.

## Sources

Primary sources:

- [OpenTelemetry JavaScript browser getting started](https://opentelemetry.io/docs/languages/js/getting-started/browser/)
- [OpenTelemetry JavaScript exporters](https://opentelemetry.io/docs/languages/js/exporters/)
- [OpenTelemetry fetch instrumentation config](https://open-telemetry.github.io/opentelemetry-js/interfaces/_opentelemetry_instrumentation-fetch.FetchInstrumentationConfig.html)
- [OpenTelemetry browser resource semantic conventions](https://opentelemetry.io/docs/specs/semconv/resource/browser/)
- [Frontend browser ingest profile recheck](frontend-browser-ingest-profile-recheck.md)
- [Frontend Replay and source-map privacy recheck](frontend-replay-sourcemap-privacy-recheck.md)
- [W3C Trace Context](https://www.w3.org/TR/trace-context/)
- [Sentry JavaScript trace propagation](https://docs.sentry.io/platforms/javascript/guides/capacitor/tracing/trace-propagation/)
- [Sentry JavaScript trace propagation targets](https://docs.sentry.io/platforms/javascript/configuration/environments/#tracepropagationtargets)
- [Sentry JavaScript breadcrumbs](https://docs.sentry.io/platforms/javascript/guides/svelte/enriching-events/breadcrumbs/)
- [Sentry source-map artifact bundles and Debug IDs](https://docs.sentry.io/platforms/javascript/guides/cloudflare/sourcemaps/troubleshooting_js/artifact-bundles/)
- [Sentry JavaScript data collected and Replay privacy defaults](https://docs.sentry.io/platforms/javascript/guides/react/data-management/data-collected)
- [Sentry Session Replay privacy](https://docs.sentry.io/platforms/javascript/session-replay/privacy/)
- [Sentry Session Replay configuration](https://docs.sentry.io/platforms/javascript/session-replay/configuration/)
- [npm `@sentry/browser`](https://www.npmjs.com/package/@sentry/browser)
- [npm `@sentry/replay`](https://www.npmjs.com/package/@sentry/replay)

Secondary implementation references:

- [Propagating OTel context from browser to backend (Tracetest)](https://tracetest.io/blog/propagating-the-opentelemetry-context-from-the-browser-to-the-backend)
- [OpenTelemetry JS trace context propagation (Uptrace)](https://uptrace.dev/get/opentelemetry-js/propagation)
