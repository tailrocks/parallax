# Frontend Replay and Source-Map Privacy Recheck

<!-- markdownlint-disable MD013 -->

Research date: 2026-05-25

## Pass Target

Re-check the most privacy-sensitive part of the frontend direction: whether
Replay, source maps, source content, network details, browser breadcrumbs, and
Sentry package provenance can be treated as ordinary agent-visible evidence.

This pass does not benchmark frontend overhead or run browser fixtures. It
tightens the evidence contract that a later frontend capture run must satisfy.

## Short Verdict

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

## Current Source Snapshot

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

## Required Ledger Tightening

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

## Canary Additions

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

## Updated Claim Boundary

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

## Uncertainty

- Sentry docs and npm package details can drift independently. Runs must persist
  both lockfile versions and docs/source snapshot dates.
- Sentry's internal package layout can change. The manifest field is deliberately
  a provenance field, not a hard-coded dependency requirement.
- This pass did not inspect generated replay payload bytes. The privacy claim
  remains unmeasured until browser fixtures run with seeded canaries.

## Falsification Triggers

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

## Sources

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

## Bottom Line

Frontend capture still belongs in Parallax, but Replay and source maps are
evidence inputs, not agent-visible defaults. The defensible v0 boundary is:
source-map server-side symbolication from private artifacts, Replay disabled or
masked opt-in, explicit network-detail/body policy, and canonical projections
that carry only redacted metadata plus non-dereferenceable raw refs.
