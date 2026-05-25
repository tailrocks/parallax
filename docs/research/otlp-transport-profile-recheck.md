# OTLP Transport Profile Recheck

<!-- markdownlint-disable MD013 -->

Research date: 2026-05-25

## Pass Target

Re-check the weakest part of the current OTLP claim boundary: whether "OTLP"
is precise enough when some competitors accept only OTLP/HTTP JSON, while the
official protocol and SDK exporter specs distinguish `grpc`,
`http/protobuf`, and `http/json`.

This pass does not benchmark receiver performance. It tightens the transport,
endpoint-path, and retry semantics that Parallax must prove before using
"OTLP-native" or "Collector-compatible" wording.

## Short Verdict

Parallax's baseline OTLP claim should require both:

- OTLP/gRPC on the standard `4317` path/service shape; and
- OTLP/HTTP with binary protobuf on `4318` and `/v1/traces`, `/v1/metrics`,
  and `/v1/logs`.

OTLP/HTTP JSON is useful and the official Collector receiver supports it, but
it should remain an explicitly labeled optional path for Parallax v0. A JSON-only
receiver is not enough for "OTLP-native" Parallax wording.

Browser/frontend telemetry is the explicit exception to the gRPC baseline. The
[frontend browser ingest profile recheck](frontend-browser-ingest-profile-recheck.md)
keeps browser OTLP HTTP-only because official OpenTelemetry JavaScript docs say
browser apps cannot use OTLP/gRPC. That exception does not weaken the
server/backend OTLP-native claim; it prevents tests from treating expected
browser gRPC failure as a frontend capture defect.

The second important fix is endpoint URL construction. The OTLP exporter spec
does not treat all endpoint environment variables the same way. A generic
`OTEL_EXPORTER_OTLP_ENDPOINT=http://host:4318` lets exporters construct
per-signal `/v1/{signal}` URLs, but a per-signal endpoint such as
`OTEL_EXPORTER_OTLP_TRACES_ENDPOINT=http://host:4318` is used as-is. The fixture
gate must test both, and Parallax docs must tell users to include `/v1/traces`,
`/v1/metrics`, or `/v1/logs` when they configure per-signal HTTP endpoints.

## Current Primary-Source Snapshot

| Source | Current signal | Parallax implication |
| --- | --- | --- |
| [OpenTelemetry specs page](https://opentelemetry.io/docs/specs/) | The current docs list OpenTelemetry Specification `1.57.0`, OTLP Specification `1.10.0`, and semantic conventions `1.41.0`. | Keep the existing version matrix. No version drift from the current ledger snapshot. |
| [OTLP specification 1.10.0](https://opentelemetry.io/docs/specs/otlp/) | OTLP is stable for traces, metrics, and logs, development for profiles. It defines OTLP/gRPC and OTLP/HTTP, HTTP binary protobuf and JSON protobuf encodings, gzip, default HTTP port `4318`, partial success, bad-data handling, retryable status codes, and throttling behavior. | A receiver must prove protocol behavior, not just route existence. Profiles stay out of v0. |
| [OpenTelemetry Protocol Exporter spec](https://opentelemetry.io/docs/specs/otel/protocol/exporter/) | Exporter protocol values are `grpc`, `http/protobuf`, and `http/json`. SDKs should support both `grpc` and `http/protobuf`, must support at least one, and may support `http/json`. The default protocol should be `http/protobuf` unless an SDK has compatibility reasons to keep `grpc`. | Parallax must test `grpc` and `http/protobuf` as the baseline and label `http/json` separately. |
| [Exporter endpoint URL rules](https://opentelemetry.io/docs/specs/otel/protocol/exporter/#endpoint-urls-for-otlphttp) | Generic OTLP HTTP endpoint variables construct per-signal paths, while per-signal endpoint variables are used as-is. | Add endpoint-construction fixtures so a quickstart does not silently send traces to `/` and blame the receiver. |
| [Collector OTLP receiver README](https://github.com/open-telemetry/opentelemetry-collector/blob/main/receiver/otlpreceiver/README.md) | The receiver is stable for traces, metrics, and logs, alpha for profiles, supports gRPC or HTTP, defaults to `localhost:4317` and `localhost:4318`, and can receive HTTP/JSON with configurable per-signal paths. | Collector equivalence should include both standard ports and path behavior. JSON support is a useful comparison point, not a v0 product requirement by itself. |
| [Collector configuration docs](https://opentelemetry.io/docs/collector/configuration/) | Defining a receiver does not enable it; the receiver must be referenced by a service pipeline. The example OTLP receiver uses gRPC `4317` and HTTP `4318`. | Fixture manifests need config hashes and must distinguish configured components from enabled pipelines. |
| [OpenTelemetry proto v1.10.0](https://github.com/open-telemetry/opentelemetry-proto/releases/tag/v1.10.0) | Latest proto release checked by the GitHub API remains `v1.10.0`, published 2026-03-09. | Parser/schema fixtures should keep pinning proto separately from SDK and Collector versions. |
| [Collector core v0.153.0](https://github.com/open-telemetry/opentelemetry-collector/releases/tag/v0.153.0) | Latest core/source release checked by the GitHub API remains `v0.153.0`, published 2026-05-25. Its release body points readers to collector-releases `v0.153.0` for binaries, but the release-tag endpoint and GitHub HTML URL for that stable distribution tag still returned 404 in this follow-up check. | Core/source drift is still a separate axis from runnable distribution binaries. Do not treat a core release-note link as proof that the matching distribution binary exists. |
| [Collector distribution v0.152.1](https://github.com/open-telemetry/opentelemetry-collector-releases/releases/tag/v0.152.1) | The collector-releases API still returned `v0.152.1` as the latest visible binary distribution on 2026-05-25, while core/source is `v0.153.0`. A `v0.153.0` stable distribution release tag was not visible through the release-tag endpoint or HTML page; the repository tag list showed `v0.153.0-nightly.*` tags only. | Do not claim current Collector equivalence from core/source alone. The manifest must pin the actual runnable binary and record whether it used a stable distribution release or a nightly tag. |
| [Collector Contrib v0.152.0](https://github.com/open-telemetry/opentelemetry-collector-contrib/releases/tag/v0.152.0) | Latest Contrib release checked remains `v0.152.0`, published 2026-05-11. | Contrib remains its own compatibility axis. |
| [opentelemetry crate](https://crates.io/crates/opentelemetry) and [opentelemetry-otlp crate](https://crates.io/crates/opentelemetry-otlp) | crates.io still reports `0.32.0` for both, updated 2026-05-08. | Rust fixtures remain the first direct SDK path; no Rust crate drift found. |
| [Rotel v0.2.2](https://github.com/rotel-dev/rotel/releases/tag/v0.2.2) and [Rotel README](https://github.com/rotel-dev/rotel) | Latest Rotel release remains `v0.2.2`. README says the receiver supports gRPC, HTTP/protobuf, and HTTP/JSON; default receiver ports are `4317` and `4318`. | Rotel smoke should test the same transport profile, but Rotel remains pre-1.0 and not the baseline. |
| [Traceway recheck](traceway-otlp-ai-replay-recheck.md) | Current Traceway evidence shows OTLP/HTTP routes for traces, metrics, and logs that accept protobuf or JSON under `/api/otel`. | Direct OTLP/HTTP is table stakes, but Traceway's base path makes standard-path and endpoint-construction tests important. |
| [Urgentry recheck](urgentry-sentry-tiny-benchmark-recheck.md) | Current Urgentry source-level recheck found OTLP HTTP/JSON routes and explicit protobuf rejection in checked source. | Treat JSON-only OTLP as a compatibility caveat, not as equivalent to Parallax's desired OTLP-native claim. |

## Required Transport Profile

| Transport path | v0 status | Claim impact |
| --- | --- | --- |
| `grpc` | Required. | Needed for Collector and many SDK deployments. |
| `http/protobuf` | Required. | Official exporter default direction and easiest direct HTTP path. |
| `http/json` | Optional, explicitly labeled. | Useful for curl/debug and parity with Traceway/Collector behavior, but not sufficient alone. |
| Browser OTLP | Separate frontend profile. | Browser builds use HTTP/protobuf or HTTP/JSON only; gRPC negative fixtures are expected. |
| Profiles | Out of scope. | OTLP profiles are development-stage in the checked spec/docs. |
| Custom base paths | Optional alias only. | Standard `/v1/{signal}` paths must work first; aliases must not replace them. |

## 2026-05-25 Collector Release-Axis Follow-Up

This follow-up keeps the existing version matrix but tightens how Parallax reads
Collector releases:

- `open-telemetry/opentelemetry-collector` `v0.153.0` exists and is the latest
  checked core/source release.
- The core release body points to
  `open-telemetry/opentelemetry-collector-releases/releases/tag/v0.153.0` for
  images and binaries.
- The collector-releases API and HTML page still returned 404 for that stable
  tag, while `/releases/latest` returned `v0.152.1`.
- The collector-releases tag list showed `v0.153.0-nightly.*` tags, not a stable
  `v0.153.0` release tag.

Implication: the OTLP conformance manifest needs a distinct
`collector_distribution_resolution` field. A nightly Collector binary can inform
development, but it cannot satisfy the stable Collector-equivalence claim unless
the run is explicitly labeled nightly and rerun on the stable distribution.

## Endpoint URL Construction Fixtures

Add these fixtures to the OTLP gate:

| Fixture | Configuration | Expected result |
| --- | --- | --- |
| `endpoint_generic_http_appends_paths` | `OTEL_EXPORTER_OTLP_ENDPOINT=http://parallax:4318`, protocol `http/protobuf`. | SDK sends `/v1/traces`, `/v1/metrics`, and `/v1/logs`; Parallax accepts the standard paths. |
| `endpoint_signal_http_explicit_paths` | `OTEL_EXPORTER_OTLP_TRACES_ENDPOINT=http://parallax:4318/v1/traces` and equivalent per-signal metric/log endpoints. | Parallax accepts because paths are explicit. |
| `endpoint_signal_http_missing_path` | `OTEL_EXPORTER_OTLP_TRACES_ENDPOINT=http://parallax:4318`, protocol `http/protobuf`. | Expected to hit `/`; Parallax may reject with non-retryable 404/400, and docs should explain the config error. |
| `protocol_grpc_required` | `OTEL_EXPORTER_OTLP_PROTOCOL=grpc` against `4317`. | Required pass for baseline. |
| `protocol_http_protobuf_required` | `OTEL_EXPORTER_OTLP_PROTOCOL=http/protobuf` against `4318`. | Required pass for baseline. |
| `protocol_http_json_optional` | `OTEL_EXPORTER_OTLP_PROTOCOL=http/json` against `4318`. | Either pass as an explicitly supported optional path or fail with a clear non-retryable unsupported-content result. |

## Response Semantics

Transport errors must match the OTLP retry model:

- malformed protobuf or invalid permanent data: non-retryable `400`;
- unsupported content type or unsupported optional JSON path: non-retryable
  status with a developer-facing message;
- overload, rate limit, or temporary storage/WAL failure: retryable status such
  as `429` or `503`, with `Retry-After` where possible;
- partial success: accepted records are durable, rejected counts are accurate,
  and clients should not retry the accepted records;
- no acknowledgment before local WAL/outbox durability in the tiny tier.

## Product Wording

Allowed before fixture results:

> Planned OTLP/gRPC and OTLP/HTTP protobuf ingestion.

Allowed after required transport fixtures and direct Rust signal fixtures pass:

> Rust OTLP ingestion over gRPC and HTTP/protobuf for the tested SDK/exporter
> versions.

Allowed only if JSON fixture passes:

> OTLP/HTTP JSON is supported for the tested subset.

Avoid:

- "OTLP-native" for a JSON-only endpoint;
- "Collector-compatible" without a runnable Collector distribution fixture;
- "supports OTLP" without naming `grpc`, `http/protobuf`, and optional
  `http/json` status;
- custom base-path examples that omit standard `/v1/{signal}` behavior.

## Falsification Triggers

Reopen this note if:

- OTLP spec, exporter spec, proto, Collector receiver, or Rust exporter behavior
  changes;
- a current Rust SDK defaults away from the expected transport or endpoint URL
  construction behavior;
- official Collector distribution `v0.153.0` or later appears and changes
  OTLP receiver behavior;
- real SDK fixtures show JSON is necessary for a target migration path;
- rejecting JSON causes retry loops or data-loss behavior that differs from the
  documented non-retryable model;
- a competitor proves standard gRPC plus HTTP/protobuf plus JSON plus evidence
  bundle parity with less operational complexity.

## Sources

- [OpenTelemetry specs page](https://opentelemetry.io/docs/specs/)
- [OTLP specification 1.10.0](https://opentelemetry.io/docs/specs/otlp/)
- [OpenTelemetry Protocol Exporter spec](https://opentelemetry.io/docs/specs/otel/protocol/exporter/)
- [OpenTelemetry Collector OTLP receiver README](https://github.com/open-telemetry/opentelemetry-collector/blob/main/receiver/otlpreceiver/README.md)
- [OpenTelemetry Collector configuration docs](https://opentelemetry.io/docs/collector/configuration/)
- [OpenTelemetry proto v1.10.0](https://github.com/open-telemetry/opentelemetry-proto/releases/tag/v1.10.0)
- [OpenTelemetry Collector core v0.153.0](https://github.com/open-telemetry/opentelemetry-collector/releases/tag/v0.153.0)
- [OpenTelemetry Collector distribution v0.152.1](https://github.com/open-telemetry/opentelemetry-collector-releases/releases/tag/v0.152.1)
- [OpenTelemetry Collector Contrib v0.152.0](https://github.com/open-telemetry/opentelemetry-collector-contrib/releases/tag/v0.152.0)
- [opentelemetry crate](https://crates.io/crates/opentelemetry)
- [opentelemetry-otlp crate](https://crates.io/crates/opentelemetry-otlp)
- [Rotel v0.2.2 release](https://github.com/rotel-dev/rotel/releases/tag/v0.2.2)
- [Rotel README](https://github.com/rotel-dev/rotel)
- [Traceway OTLP/AI/replay recheck](traceway-otlp-ai-replay-recheck.md)
- [Urgentry Sentry/Tiny/benchmark recheck](urgentry-sentry-tiny-benchmark-recheck.md)
- [Frontend browser ingest profile recheck](frontend-browser-ingest-profile-recheck.md)

## Bottom Line

"OTLP-compatible" is too vague for Parallax. The baseline should mean
gRPC plus HTTP/protobuf, standard paths, endpoint URL construction fixtures,
retry-safe failure behavior, and Collector distribution evidence. HTTP/JSON can
be useful, but a JSON-only path is a caveat, not the Parallax compatibility
target.
