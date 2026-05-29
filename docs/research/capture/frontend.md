# Frontend Capture and Cross-Tier Correlation

> Parallax must capture from the browser (JS/TS), not only the backend, and join that frontend evidence to backend spans across the tier boundary, because most incidents are user-facing and the real cause usually crosses that boundary. The decided architecture is a dual-API browser source — Sentry browser envelopes for rich errors/breadcrumbs plus OpenTelemetry JS fetch/XHR spans for cross-tier trace continuity — exporting over a browser-specific HTTP-only ingest profile (`http/protobuf` preferred, HTTP/JSON optional; OTLP/gRPC is an expected-unsupported negative), with W3C `traceparent`/`tracestate`/`baggage` propagation guarded by backend CORS, an additive schema of frontend nodes and cross-tier edges, private Debug-ID-like source maps symbolicated server-side, and a default-deny privacy posture in which Replay and source maps are opt-in raw refs rather than agent-visible defaults. The frontend is a telemetry source only: the Parallax engine and infrastructure stay Rust-first and within the Rust/Go/Zig/C++/C filter, and nothing here adds a JS dependency to the Parallax core. As of the 2026-05-25 package snapshot the pins are `@sentry/browser`/`@sentry/react` `10.53.1` (internal `@sentry-internal/replay`/`@sentry-internal/replay-canvas` `10.53.1`, standalone `@sentry/replay` stale at `7.116.0`), `@opentelemetry/sdk-trace-web` `2.7.1`, and OTel fetch/XHR plus OTLP HTTP exporters `0.218.0`. The current capture-safety status is `not_measured`: the open gate is that no dated browser/build/route run artifacts yet exist, so frontend capture must be described as designed/planned, not proven, until the ledger proves the browser matrix, source-map identity and access controls, browser ingest posture, source-field policy, CORS/propagation and backend continuation, breadcrumbs, privacy canaries, export reliability, overhead, canonical hashes, and projection equivalence (including MCP `structuredContent`).

This note consolidates the following previously-separate research files, each preserved in full below:

- `frontend-collection-and-cross-tier-correlation.md`
- `frontend-capture-safety-ledger.md`
- `frontend-browser-ingest-profile-recheck.md`
- `frontend-replay-sourcemap-privacy-recheck.md`

## Frontend Collection and Cross-Tier Correlation

_Provenance: merged verbatim from `frontend-collection-and-cross-tier-correlation.md` (2026-05-29 restructure)._

<!-- markdownlint-disable MD013 -->

Research date: 2026-05-25

### Purpose

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

### Current Primary-Source Checks

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

### Collection Method

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

### Cross-Tier Trace Propagation (The Core)

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

### Schema Extension

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

### Cross-Tier Reconstruction Query

The "how did we get to this user-facing error" query crosses the boundary: from a
`frontend_error`, take its `trace_id`, fetch the frontend session's preceding
user-steps, and follow `frontend_request_to_span` into the backend spans/logs/
errors on the same trace. This is benchmark query **Q4 `cross_tier`** in
[the storage benchmark prototype](storage-benchmark-prototype.md) — frontend
collection is exactly why that query exists, and why the dataset generator links a
fraction of frontend sessions into backend traces.

### Source-Map Symbolication

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

### Privacy (The Hardest Part)

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

### Honest Limits

- Frontend capture is **best-effort**: adblockers, privacy modes, crashes before
  flush, and offline all drop data. Never assume completeness.
- A frontend error alone rarely proves root cause — the value is the **join** to
  backend evidence; if propagation/CORS is broken, you get half the picture.
- Replay is high-signal for humans but expensive and privacy-heavy; it is not
  required for agent reconstruction and should not be in the tiny tier.
- Web Vitals/RUM answer latency-class questions, not error causality; keep them
  out of the critical error-reconstruction path.

### Reuse vs Incumbents

- **Sentry browser SDK + rrweb-based Replay** is the strongest reference for
  error capture, breadcrumbs, and masked replay; the envelope format is one
  Parallax already accepts on the backend, so the frontend error path is the same
  ingestion contract.
- **OpenTelemetry JS** (fetch/XHR instrumentation + W3C propagators) is the
  reference for the cross-tier span path. Use it for traces; use the Sentry
  envelope for rich errors. This is the same dual-standard stance as the backend.

### MVP Scope (Tiny Tier)

Ship: frontend error (source-mapped) + fetch/XHR spans with `traceparent`
propagation + bounded breadcrumbs + route/release context, joined to backend by
`trace_id`. Defer: session replay, Web Vitals dashboards, full RUM. Prove the
cross-tier join on one frontend↔backend path before broadening. The real-data
pass/fail threshold for that claim lives in
[Correlation reliability on real telemetry gate](correlation-reliability-real-telemetry-gate.md),
with row-level proof captured by the
[A4 correlation reliability ledger](a4-correlation-reliability-ledger.md).

### Relationship To Other Research

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

### Sources

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

## Frontend Capture Safety Ledger

_Provenance: merged verbatim from `frontend-capture-safety-ledger.md` (2026-05-29 restructure)._

_(Shared note — see the Frontend Collection and Cross-Tier Correlation section above.)_

### Purpose

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

### Current Source Snapshot

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

### Claim Levels

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

### Result Artifacts

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

### Run Manifest

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

### Row Schemas

#### Browser Route Matrix Row

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

#### SDK Config Result Row

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

#### Error Capture Result Row

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

#### Source Map Result Row

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

#### Propagation Result Row

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

#### CORS Result Row

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

#### Ingest Endpoint Result Row

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

#### Breadcrumb Result Row

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

#### Privacy Canary Result Row

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

#### Source Field Policy Result Row

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

#### Export Reliability Result Row

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

#### Overhead Result Row

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

#### Replay Privacy Result Row

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

#### Projection Result Row

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

#### Claim Ledger Row

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

### Counting Rules

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

### Product Wording

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

### Refresh Triggers

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

### Relationship To Other Research

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

### Bottom Line

Frontend capture is essential to the Parallax thesis because many failures are
user-facing and cross the browser/backend boundary. It is also the highest-PII
telemetry surface. The safe default is source-mapped errors, route/release
metadata, bounded redacted breadcrumbs, and first-party trace propagation. Replay
and raw browser content stay opt-in refs. Product claims start only after this
ledger proves the browser matrix, source-map identity and access controls,
browser ingest posture, source-field policy status, propagation, privacy, export
reliability, overhead, canonical hashes, and projection equivalence.

## Frontend Browser Ingest Profile Recheck

_Provenance: merged verbatim from `frontend-browser-ingest-profile-recheck.md` (2026-05-29 restructure)._

_(Shared note — see the Frontend Collection and Cross-Tier Correlation section above.)_

### Pass Target

Re-check the weakest frontend claim boundary: whether Parallax can reuse the
backend OTLP transport profile for browser telemetry. The answer is no. Browser
capture is still OTLP/Sentry compatible, but it needs its own ingest profile
because browser clients cannot use OTLP/gRPC, are constrained by CSP/CORS, and
run with public credentials and high-PII surfaces.

This pass does not benchmark browser overhead or export reliability. It tightens
the source-checked transport, CORS, source-map, replay, and agent-visible safety
requirements that the frontend ledger must prove later.

### Short Verdict

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

### Current Primary-Source Snapshot

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

### Required Browser Ingest Profile

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

### Fixture Additions

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

### Uncertainty

- The published OTel package tarballs prove the current type/runtime surface for
  `0.218.0`, but OTel browser packages are still a fast-moving surface. Fixture
  runs expire when package versions, target browsers, bundlers, or CSP/CORS
  policy changes.
- The Sentry docs are authoritative for current configuration guidance, but
  exact defaults can differ by SDK/framework integration. Parallax must record
  the lockfile and generated SDK config used in each run.
- Browser telemetry remains best-effort: blockers, privacy modes, offline
  shutdown, CSP, and failed preflights can drop data.

### Falsification Triggers

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

### Sources

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

### Bottom Line

Browser capture still serves Parallax's goal: it supplies the missing
user-facing side of the runtime lifecycle and links it to backend evidence.
The claim must be narrower than "same OTLP as the backend": browser telemetry is
HTTP-only, CORS/CSP-bound, public-ingest-facing, source-map-sensitive, and
privacy-heavy. That makes a separate browser ingest profile and ledger fixtures
mandatory before Parallax calls frontend reconstruction proven.

## Frontend Replay and Source-Map Privacy Recheck

_Provenance: merged verbatim from `frontend-replay-sourcemap-privacy-recheck.md` (2026-05-29 restructure)._

_(Shared note — see the Frontend Collection and Cross-Tier Correlation section above.)_

### Pass Target

Re-check the most privacy-sensitive part of the frontend direction: whether
Replay, source maps, source content, network details, browser breadcrumbs, and
Sentry package provenance can be treated as ordinary agent-visible evidence.

This pass does not benchmark frontend overhead or run browser fixtures. It
tightens the evidence contract that a later frontend capture run must satisfy.

### Short Verdict

Keep the frontend architecture, but narrow the claim boundary:

- Tiny tier remains source-mapped frontend errors, route/release context,
  bounded redacted breadcrumbs, and first-party trace propagation.
- Replay is useful, but it is not a default Parallax signal. It is an opt-in raw
  reference surface, masked by default, with no agent dereference unless a later
  access-control and projection run explicitly allows it.
- Source maps and `sourcesContent` are raw source artifacts. They can power
  server-side symbolication, but they are not agent-visible evidence by default.
- Package provenance matters. Current Sentry JS v10 Replay is pulled through
  `@sentry/browser` internal dependencies; the public standalone
  `@sentry/replay` package is stale on npm and should not be used as the
  current-version source of truth unless a lockfile actually includes it.

The existing `not_measured` frontend status remains correct. Product wording
must not imply "safe Replay", "agent-visible Replay", or "safe source maps"
until run artifacts prove masking, public-map denial, source-content denial,
retention, projection hashes, and raw-ref access controls.

### Current Source Snapshot

| Source checked 2026-05-25 | Current signal | Parallax implication |
| --- | --- | --- |
| npm registry for [`@sentry/browser`](https://www.npmjs.com/package/@sentry/browser) | `npm view @sentry/browser@latest version time.modified dependencies --json` reports `10.53.1`, modified `2026-05-12T17:07:50.879Z`, with `@sentry-internal/replay` and `@sentry-internal/replay-canvas` pinned to `10.53.1`. | Browser runs must record `@sentry/browser` and internal replay dependency versions. For Sentry JS v10, Replay provenance is not proven by looking only at standalone `@sentry/replay`. |
| npm registry for [`@sentry/react`](https://www.npmjs.com/package/@sentry/react) | `npm view @sentry/react@latest version time.modified dependencies peerDependencies --json` reports `10.53.1`, modified `2026-05-12T17:08:04.434Z`, depending on `@sentry/browser` `10.53.1`. | React frontend fixtures should record both framework integration and browser package versions. |
| npm registry for [`@sentry/replay`](https://www.npmjs.com/package/@sentry/replay) | `npm view @sentry/replay@latest version time.modified description --json` reports `7.116.0`, modified `2025-11-25T14:44:52.058Z`. | Treat `@sentry/replay` as a legacy/stale standalone package unless a real app lockfile includes it. Do not cite it as the current v10 Replay implementation path. |
| npm registry for OTel browser packages | `@opentelemetry/sdk-trace-web` remains `2.7.1`; fetch and XHR instrumentations remain `0.218.0`. | No version drift from the browser ingest recheck; this pass only changes Replay/source-map privacy bookkeeping. |
| [Sentry React data collected](https://docs.sentry.io/platforms/javascript/guides/react/data-management/data-collected/) | Sentry documents browser surfaces that may include headers, full URLs/query strings, referrers, console breadcrumbs, Replay metadata, and opt-in network request/response bodies, while some PII and body content are off by default. | Parallax must record explicit capture policy per surface and seed canaries across metadata, not only obvious bodies and form fields. |
| [Sentry Session Replay privacy](https://docs.sentry.io/platforms/javascript/session-replay/privacy/) and [configuration](https://docs.sentry.io/platforms/javascript/session-replay/configuration/) | Replay has masking/blocking controls and network-detail options; network bodies become a separate allowlist decision. | Replay can be claimable only after masking, selector allowlist, network body, retention, overhead, and projection rows pass. |
| [Sentry source-map upload guidance](https://docs.sentry.io/platforms/javascript/guides/tanstackstart-react/sourcemaps/uploading/esbuild/) | Sentry warns source maps can expose source and recommends denying `.js.map` access or deleting maps after upload. | A source-map claim fails if public map access works or if source content reaches agent-visible bundles by default. |
| [Sentry artifact bundles and Debug IDs](https://docs.sentry.io/platforms/javascript/guides/cloudflare/sourcemaps/troubleshooting_js/artifact-bundles/) | Debug IDs bind minified files and source maps without path-only matching. | Parallax should keep Debug-ID-like identity, private storage, and negative fixtures for missing/mismatched/public maps. |
| [A6 synthetic canary fixture corpus](a6-synthetic-canary-fixture-corpus.md) | The current corpus covers generic frontend expansion but does not yet name Replay/source-map specific canaries. | Add frontend canary classes for Replay segments, source-map refs, `sourcesContent`, URL/query/referrer, headers, console, DOM text, and network bodies. |

### Required Ledger Tightening

Add these fields to every future frontend capture run manifest:

```json
{
  "sentry_replay_package_source": "@sentry/browser_internal|@sentry/replay_standalone|none|other",
  "sentry_internal_replay_version": "x.y.z|not_present",
  "sentry_internal_replay_canvas_version": "x.y.z|not_present",
  "standalone_sentry_replay_version": "x.y.z|not_present",
  "replay_agent_visible_default": false,
  "replay_network_detail_policy": "disabled|allowlist|vendor_default",
  "replay_masking_config_hash": "sha256:...",
  "source_maps_public_accessible": false,
  "source_content_agent_visible": false
}
```

Add these row-level checks before `replay_claim_ready` or
`source_map_symbolication_pass` can pass:

| Check | Failure mode it catches |
| --- | --- |
| `replay_package_provenance_recorded` | A run claims current Replay behavior while only recording `@sentry/react` or stale `@sentry/replay` package data. |
| `replay_segment_agent_deref_denied` | Agent-visible bundle, CLI, HTTP, or MCP output can open raw replay content by default. |
| `replay_network_body_allowlist_empty_by_default` | Request/response bodies leak because Replay network detail capture was enabled without a scoped allowlist. |
| `source_map_public_negative` | Deployed `.js.map` or equivalent source-map URL is publicly accessible. |
| `sources_content_agent_visible_negative` | Raw source, `sourcesContent`, or source context appears in canonical bundles or projections. |
| `source_map_ref_access_scoped` | A source-map ref is dereferenceable without scoped operator approval or a server-side symbolication path. |

### Canary Additions

The A6 corpus should include frontend-specific canaries with public-safe raw
values or private hashes:

| Fixture ID | Surface | Required assertion |
| --- | --- | --- |
| `frontend_replay_dom_text_001` | Replay DOM recording | Masked replay projection and metadata contain no seeded DOM text. |
| `frontend_replay_input_001` | Replay form/input | Input value is blocked or masked before any projection. |
| `frontend_replay_network_body_001` | Replay network detail | Request/response body canary is absent unless an explicit allowlist fixture expects it. |
| `frontend_sourcemap_sources_content_001` | Source map artifact | `sourcesContent` is never present in agent-visible evidence. |
| `frontend_sourcemap_public_negative_001` | Deployed build | Public source-map fetch fails or is blocked for tested build assets. |
| `frontend_url_referrer_console_001` | Browser metadata | Query string, referrer, and console canaries are redacted before canonical JSON, Markdown, CLI, HTTP, and MCP outputs. |

### Updated Claim Boundary

Keep these claims:

- Frontend capture serves Parallax's core goal because user-facing failures often
  require browser-to-backend reconstruction.
- Sentry JS and OTel JS remain the right ecosystem references for browser error
  envelopes, breadcrumbs, Replay reference behavior, and trace propagation.
- Source maps are still required for useful frontend stack traces.

Narrow these claims:

- "Sentry Replay defaults" should be cited from Sentry JS docs and the actual
  app lockfile, not from the standalone `@sentry/replay` npm package by default.
- "Source-mapped frontend errors" means server-side symbolication from private
  artifacts, not public source-map access and not source content in bundles.
- "Replay refs" means masked, access-controlled references after a dedicated
  replay privacy run, not raw event replay in agent context.

### Uncertainty

- Sentry docs and npm package details can drift independently. Runs must persist
  both lockfile versions and docs/source snapshot dates.
- Sentry's internal package layout can change. The manifest field is deliberately
  a provenance field, not a hard-coded dependency requirement.
- This pass did not inspect generated replay payload bytes. The privacy claim
  remains unmeasured until browser fixtures run with seeded canaries.

### Falsification Triggers

Reopen this note if:

- `@sentry/replay` becomes the current v10+ recommended Replay package again;
- Sentry removes internal Replay dependencies from `@sentry/browser` or changes
  default masking/network-detail behavior;
- source-map tooling changes make public maps unnecessary and private maps
  harder to manage;
- a frontend fixture shows Parallax can safely expose useful Replay-derived
  summaries without raw replay refs;
- an A6 run finds Replay, source maps, console, URL/query, or referrer canaries
  leaking after the current policy.

### Sources

- [npm `@sentry/browser`](https://www.npmjs.com/package/@sentry/browser)
- [npm `@sentry/react`](https://www.npmjs.com/package/@sentry/react)
- [npm `@sentry/replay`](https://www.npmjs.com/package/@sentry/replay)
- [npm `@opentelemetry/sdk-trace-web`](https://www.npmjs.com/package/@opentelemetry/sdk-trace-web)
- [npm `@opentelemetry/instrumentation-fetch`](https://www.npmjs.com/package/@opentelemetry/instrumentation-fetch)
- [npm `@opentelemetry/instrumentation-xml-http-request`](https://www.npmjs.com/package/@opentelemetry/instrumentation-xml-http-request)
- [Sentry React data collected](https://docs.sentry.io/platforms/javascript/guides/react/data-management/data-collected/)
- [Sentry Session Replay privacy](https://docs.sentry.io/platforms/javascript/session-replay/privacy/)
- [Sentry Session Replay configuration](https://docs.sentry.io/platforms/javascript/session-replay/configuration/)
- [Sentry source-map upload guidance](https://docs.sentry.io/platforms/javascript/guides/tanstackstart-react/sourcemaps/uploading/esbuild/)
- [Sentry artifact bundles and Debug IDs](https://docs.sentry.io/platforms/javascript/guides/cloudflare/sourcemaps/troubleshooting_js/artifact-bundles/)
- [A6 synthetic canary fixture corpus](a6-synthetic-canary-fixture-corpus.md)

### Bottom Line

Frontend capture still belongs in Parallax, but Replay and source maps are
evidence inputs, not agent-visible defaults. The defensible v0 boundary is:
source-map server-side symbolication from private artifacts, Replay disabled or
masked opt-in, explicit network-detail/body policy, and canonical projections
that carry only redacted metadata plus non-dereferenceable raw refs.
