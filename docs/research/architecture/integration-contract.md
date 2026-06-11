# Integration Contract: How Applications and Tools Attach to Parallax (Concept)

<!-- markdownlint-disable MD013 -->

Research date: 2026-06-11. Status: **design, not measured.** This note pins down the outward-facing
contract that other documents assume but none owned: what an application, a CI job, a deploy
system, a browser, a coding agent, and a fixer each do, concretely, to participate.

> **Principle: no proprietary SDK.** An application integrates with Parallax by emitting standard
> OpenTelemetry with a small set of required resource attributes. If Parallax disappeared, nothing
> in the application would need to change. Everything Parallax-specific lives server-side
> (derivation, grouping, bundling) or in optional tooling (CLI, CI action) — never as a lock-in
> client library. Sentry-protocol ingest remains a future migration adapter
> ([capture/sentry-ingest.md](../capture/sentry-ingest.md)), not the integration path.

## 1. Applications (services, CLIs): standard OTel + conventions

Transport: OTLP gRPC (4317) and HTTP/protobuf (4318), per the
[OTLP conformance profile](../capture/otlp.md). Collector optional (tiny tier sends direct).

Required resource attributes — the correlation contract:

| Attribute | Why Parallax needs it | Source |
| --- | --- | --- |
| `service.name` | Anchor of every per-service view and edge | OTel semconv (stable) |
| `service.version` | Release linkage; pairs with deploy events | OTel semconv |
| `deployment.environment.name` | Issue scoping (prod vs staging); recurrence windows are per-environment | OTel semconv |
| `vcs.ref.head.revision` | The deployed commit — the strong edge to code changes and fix validation | OTel semconv (VCS), stamped at build time |
| `vcs.repository.url.full` | Which repo a fixer should be pointed at (mono-repo: one value everywhere) | OTel semconv (VCS) |

Error semantics Parallax derives from (no fourth signal, per
[api-concept.md](api-concept.md)): span status `ERROR` + `exception` span events, ERROR/FATAL log
records, and `exception.*` attributes on log records. Note OTel's 2026-03-17 deprecation of Span
Events in favor of log-based events (`OTEL_SEMCONV_EXCEPTION_SIGNAL_OPT_IN` transition,
[announcement](https://opentelemetry.io/blog/2026/deprecating-span-events/)): Parallax must
accept **both** exception encodings indefinitely — fleets will straddle the transition for years.

Reference Rust setup (entirely standard crates; this is documentation, not an SDK):

```rust
// Cargo.toml: tracing, tracing-subscriber, tracing-opentelemetry,
//             opentelemetry, opentelemetry-otlp, opentelemetry-sdk
use opentelemetry::KeyValue;
use opentelemetry_otlp::WithExportConfig;

fn init_telemetry() -> anyhow::Result<()> {
    let resource = opentelemetry_sdk::Resource::builder()
        .with_attributes([
            KeyValue::new("service.name", env!("CARGO_PKG_NAME")),
            KeyValue::new("service.version", env!("CARGO_PKG_VERSION")),
            KeyValue::new("deployment.environment.name", std::env::var("APP_ENV")?),
            // Stamped at build time, e.g. via vergen or a build script:
            KeyValue::new("vcs.ref.head.revision", env!("BUILD_GIT_SHA")),
            KeyValue::new("vcs.repository.url.full", env!("BUILD_GIT_REMOTE")),
        ])
        .build();

    let tracer_provider = opentelemetry_sdk::trace::SdkTracerProvider::builder()
        .with_batch_exporter(
            opentelemetry_otlp::SpanExporter::builder()
                .with_tonic()
                // OTEL_EXPORTER_OTLP_ENDPOINT=https://parallax.internal:4317
                .with_endpoint(std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT")?)
                .build()?,
        )
        .with_resource(resource)
        .build();

    // Same pattern for the OTLP log exporter; panics reach logs through a
    // panic hook + tracing-log/log-error bridging (see capture/rust.md).
    opentelemetry::global::set_tracer_provider(tracer_provider);
    Ok(())
}
```

Auth: per-project ingest token, sent as OTLP metadata/header
(`x-parallax-project-token: <token>`); DSN-style single connection string later for one-line
config. Multi-tenancy boundaries are an open design item tracked in
[api-concept.md](api-concept.md).

**Surface the trace ID to end users.** The convention that powers the rung-2 complaint loop
([lifecycle 4](../00-vision/problem-audience-product-shape.md)): error responses and error pages
expose the current trace ID — printed on the page, in a "copy error reference" control, or in an
error toast — so a user who hits a bug can hand support one token instead of a reproduction
essay. Server-side this is trivial (the active span's trace ID into the error response body or a
`traceresponse`-style header); browser-side the W3C `traceparent` the frontend already propagates
is the same identifier. One pasted trace ID later, the agent reconstructs the entire user
workflow from Parallax.

## 2. Deploy systems: the deploy event

Two equivalent paths; both normalize to the same `deploy`/`release` nodes of
[deploy-change-context.md](../capture/deploy-change-context.md):

1. **Native endpoint** (any deploy tool, one curl):

```json
POST /v1/deploys
{
  "schema": "parallax.deploy.v0",
  "project_id": "proj_checkout",
  "release": "1.42.0",
  "vcs_sha": "abc123...",
  "environment": "production",
  "status": "succeeded",
  "started_at": "2026-06-11T10:00:00Z",
  "finished_at": "2026-06-11T10:03:12Z",
  "actor": { "type": "ci", "ref": "github-actions/deploy.yml#812" },
  "links": { "workflow_run": "https://github.com/tailrocks/checkout/actions/runs/812" }
}
```

1. **GitHub webhook ingest** (zero deploy-tool changes): Parallax subscribes to `deployment`,
   `deployment_status`, `workflow_run`, `check_run`, `pull_request`, and `pull_request_review`
   events — the same feed the Reconciler
   ([autonomous-fix-loop.md](autonomous-fix-loop.md) §Stage 5) consumes for fix validation.

The deploy event is what turns "errors after 10:03" into a `deploy_adjacent_regression` trigger
with a strong edge — it is the single highest-leverage integration after OTLP itself.

## 3. CI: the action step

CI participates twice: as a telemetry source (test failures, flaky detection — future scope per
[ci-and-flaky-tests.md](../capture/ci-and-flaky-tests.md)) and as a deploy-event emitter (above).
Minimal v0 (concept):

```yaml
# .github/workflows/deploy.yml
- name: Notify Parallax of deploy
  run: |
    curl -fsS -X POST "$PARALLAX_URL/v1/deploys" \
      -H "x-parallax-project-token: ${{ secrets.PARALLAX_TOKEN }}" \
      -d "{\"schema\":\"parallax.deploy.v0\",\"project_id\":\"proj_checkout\",
           \"release\":\"$VERSION\",\"vcs_sha\":\"$GITHUB_SHA\",
           \"environment\":\"production\",\"status\":\"succeeded\"}"
```

A first-party `parallax-deploy-action` wrapping this is Phase-3 polish, not architecture.

## 4. Browsers: OTLP over HTTP + trace propagation

Per [capture/frontend.md](../capture/frontend.md): browser OTel SDK emitting OTLP HTTP/protobuf,
W3C `traceparent` propagated on outbound requests so a user-facing error joins the backend spans
it caused. Privacy gates (source maps, breadcrumbs, replay) stay opt-in raw-reference surfaces.
No Parallax-specific browser SDK; same no-lock-in principle.

## 5. Coding agents: read-only context surfaces

Per [agent-access-surface.md](../decisions/agent-access-surface.md), one canonical contract,
three transports — CLI first, HTTP canonical, MCP after safety gates. Loop addition: the
**dispatch subscription** ([autonomous-fix-loop.md](autonomous-fix-loop.md) §Stage 3) — webhook,
work-item, or pull. v0 read surface (names indicative):

```text
parallax issue list --dispatchable        # what needs attention
parallax issue context iss_91             # the bounded bundle (JSON/Markdown)
parallax run inspect run_17               # local-first dev loop
MCP: issue_search, issue_context, run_inspect   (read-only, structuredContent)
```

## 6. Fixers: the write-back

The only write surface a fixer ever gets: append-only outcome records
(`POST /v1/outcomes`, schema exactly per the [fixer boundary](../decisions/fixer-boundary.md)
outcome contract). No other mutation exists on the agent path. GitHub webhooks let the
Reconciler fill CI/review/merge/recurrence rows even when a fixer forgets to report.

## 7. Compatibility posture

| Direction | Posture |
| --- | --- |
| In: OTLP | Primary, conformance-gated ([capture/otlp.md](../capture/otlp.md)) |
| In: Sentry envelopes | Future adapter, fixture-gated, never required |
| In: deploy/VCS webhooks | GitHub first, provider-pluggable |
| Out: bundles | Versioned open schema, canonical hash, projection-equivalent everywhere |
| Out: dispatch events | Versioned (`parallax.fix_candidate.v0`), transport-agnostic |

Everything in this note is concept-stage: endpoint names, payloads, and the action are
illustrative until the V1 API freezes. The invariants that are *not* illustrative: no proprietary
app SDK, required resource attributes as the correlation contract, both exception encodings
accepted, deploy events as first-class evidence, read-only agent surfaces, append-only fixer
write-back.
