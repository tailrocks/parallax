# Plan 036: Fix the playground trace spine — context extraction/injection, ERROR span status, baggage, CORS, resource identity

> **Executor instructions**: This plan targets the **playground repository**
> (`parallax-telemetry-playground`, sibling of the Parallax repo) — its own
> `AGENTS.md`/conventions apply. Follow step by step; run every verification
> command. On any STOP condition, stop and report. When done, update the
> status row for this plan in `plans/README.md` **in the Parallax repo**.
>
> **Drift check (run first)**: in the playground repo,
> `git diff --stat ed1f975..HEAD -- libs/playground-telemetry services deploy web/src`
> If any in-scope file changed, compare the "Current state" excerpts against
> live code; on a mismatch, STOP.

## Status

- **Priority**: P1
- **Effort**: M
- **Risk**: LOW
- **Depends on**: none (coordinate with advisor-plans/034 — see Maintenance)
- **Category**: bug
- **Planned at**: commit `408be17` (Parallax) / `ed1f975` (playground), 2026-07-07

## Why this matters

The playground's whole purpose is to feed Parallax believable multi-service
telemetry, but its Rust tier never extracts incoming W3C trace context and its
HTTP clients never inject it. Result: in the default compose, the pricing gRPC
SERVER span and every HTTP fan-out span (inventory, recommendation,
notifications) are orphan roots — the "distributed trace" only stitches on the
Rust→Java gRPC edge, because the Java agent extracts. Error branches also
never set span status ERROR, so chaos scenarios are invisible to any UI keyed
on span status. Fixing propagation + status + baggage + identity in the shared
lib is the producer-side foundation for Parallax's service map (advisor-plan
031), story (029), evidence gaps (032), and every demo scenario. Without this
plan, most later playground plans demo broken traces.

## Current state

All excerpts verified at playground commit `ed1f975`.

- `libs/playground-telemetry/src/lib.rs` — shared bootstrap for all six Rust
  services (checkout, pricing, inventory, recommendation, orders,
  notifications, plus `cli/`). Registers trace-context propagator **only**,
  and only name+version resource attrs:

  ```rust
  // lib.rs:58-66
  pub fn init(service: &'static str) -> anyhow::Result<Telemetry> {
      global::set_text_map_propagator(TraceContextPropagator::new());

      let resource = Resource::builder()
          .with_attributes([
              KeyValue::new(SERVICE_NAME, service),
              KeyValue::new(SERVICE_VERSION, env!("CARGO_PKG_VERSION")),
          ])
          .build();
  ```

  There is no baggage propagator, no `service.namespace`, no
  `service.instance.id`, no `parallax.run.id` fallback. Sentry `release` is
  hardcoded to the crate version (`lib.rs:115`) — that specific fix belongs to
  plan 042, do not do it here.

- **No server-side extraction anywhere.** `services/pricing/src/main.rs:18-22`:

  ```rust
  #[tracing::instrument(skip(self, request), fields(otel.kind = "server"))]
  async fn quote(
      &self,
      request: Request<QuoteRequest>,
  ) -> Result<Response<QuoteResponse>, Status> {
  ```

  — never reads `request.metadata()`. The axum services register plain
  routers with no extraction layer, e.g. `services/inventory/src/main.rs:51-53`:

  ```rust
  let app = Router::new()
      .route("/reserve", get(reserve))
      .route("/healthz", get(|| async { "ok" }));
  ```

- **Client injection exists only for checkout→pricing gRPC.**
  `services/checkout/src/main.rs:221-246` defines `MetadataInjector` and
  injects into tonic metadata. But the HTTP clients are bare:

  ```rust
  // checkout/src/main.rs:279-292
  #[tracing::instrument(fields(otel.kind = "client"))]
  async fn reserve(sku: &str, quantity: u32) -> anyhow::Result<Value> {
      let base = std::env::var("INVENTORY_URL")...;
      let url = format!("{base}/reserve?sku={sku}&quantity={quantity}");
      Ok(reqwest::get(&url).await?.json::<Value>().await?)
  }
  ```

  (`recommend` at `:286-292` is identical; `quote_stream`'s
  `PricingClient::connect` path at `:253-277` never injects either.)

- **Error branches never set span ERROR status.** Repo-wide grep for
  `otel.status_code` / `set_status` / `record_exception` in `services/`
  returns nothing. Example failure branch (`checkout/src/main.rs:137-139`):

  ```rust
  if p.fail || flag("PAYMENT_FAILURE") || release_regressed {
      // B1/B12: deliberate failure → error issue + ERROR span status.
      tracing::error!(sku = %p.sku, release_regressed, "payment failure (chaos)");
  ```

  The comment claims ERROR span status; `tracing_opentelemetry` does NOT
  derive span status from event level, so status stays Unset. Same pattern in
  `services/inventory/src/main.rs:34-40` (`fail` → 503),
  `services/orders/src/main.rs` (consumer failure ~`:68`, dead-letter ~`:92`),
  `services/checkout/src/main.rs:182-188` (pricing failure → 502), and
  `cli/src/main.rs` (~`:21`).

- **Browser propagation is blocked server-side.** `web/src/telemetry.ts:36,42`
  registers `W3CTraceContextPropagator` and
  `FetchInstrumentation({ propagateTraceHeaderCorsUrls: apiOrigins })`, so the
  browser SENDS `traceparent` cross-origin — but no Rust service has a CORS
  layer (no `tower-http` dependency in the workspace `Cargo.toml`), so the
  preflight for a cross-origin fetch with `traceparent` fails against
  checkout.

- **Java tier resource identity**: compose sets only `OTEL_SERVICE_NAME` per
  Java service (`deploy/docker-compose.yml:101,116,128`); `service.version`
  and `service.namespace` are absent. `deploy/docker-compose.yml:26` sets
  `OTEL_PROPAGATORS=tracecontext,baggage` — honored by the Java agent only.

- Versions (workspace `Cargo.toml`): `axum 0.8`, `tonic 0.14`,
  `reqwest 0.13` (native-tls — repo TLS rule: **never rustls**),
  `opentelemetry* 0.32`, `tracing-opentelemetry 0.33`.

- Conventions (playground `AGENTS.md`/README): Rust stable, cargo fmt +
  clippy zero warnings, `cargo build` as the gate; latest stable crate
  versions; zero-copy hot-path rule applies to the Parallax repo, not here,
  but keep per-request allocations reasonable.

## Commands you will need

| Purpose | Command (playground repo root) | Expected on success |
|---------|--------------------------------|---------------------|
| Build | `rtk cargo build` | exit 0 |
| Lint | `rtk cargo clippy --all-targets -- -D warnings` | exit 0 |
| Format | `rtk cargo fmt --all` | exit 0, no diff after |
| Full stack (manual verify) | `docker compose -f deploy/docker-compose.yml up --build -d` | services healthy |
| Scenario | `scenarios/a1-checkout.sh` | curl 200s |

## Scope

**In scope** (playground repo only):
- `libs/playground-telemetry/src/lib.rs` and new modules under
  `libs/playground-telemetry/src/`
- `libs/playground-telemetry/Cargo.toml`, workspace `Cargo.toml` (add
  `opentelemetry-http`, `tower-http` with `cors`, `http` if needed)
- `services/checkout/src/main.rs`, `services/pricing/src/main.rs`,
  `services/inventory/src/main.rs`, `services/recommendation/src/main.rs`,
  `services/orders/src/main.rs`, `services/notifications/src/main.rs`,
  `cli/src/main.rs` (status helper + extraction adoption only)
- `deploy/docker-compose.yml`, `deploy/docker-compose.xlang.yml` (Java
  `OTEL_RESOURCE_ATTRIBUTES` additions only)

**Out of scope** (do NOT touch):
- Sentry `release` sourcing from `RELEASE` env and deploy markers — plan 042.
- Sampling configuration (stays 100%/always-on) — plan 054 adds the 10%
  scenario.
- Any daemon/`enter`/agent code — advisor-plans/034 owns that.
- The web app (`web/src`) — its propagator list is extended in plan 050;
  server-side CORS here is enough to unblock it.
- Anything in the Parallax repo except the plans/README.md status row.

## Git workflow

- Playground repo, `main`, Conventional Commits, `git commit -s`, one
  `Co-authored-by: Claude <noreply@anthropic.com>` trailer. Push when done.

## Steps

### Step 1: Composite propagator + resource identity in the shared lib

In `libs/playground-telemetry/src/lib.rs`:
1. Replace the propagator registration with a composite:

   ```rust
   use opentelemetry::propagation::TextMapCompositePropagator;
   use opentelemetry_sdk::propagation::{BaggagePropagator, TraceContextPropagator};
   global::set_text_map_propagator(TextMapCompositePropagator::new(vec![
       Box::new(TraceContextPropagator::new()),
       Box::new(BaggagePropagator::new()),
   ]));
   ```

   (Exact module paths per opentelemetry 0.32 — check the crate docs if the
   composite lives elsewhere; it may be
   `opentelemetry_sdk::propagation::TextMapCompositePropagator`.)
2. Extend the resource:
   - `service.namespace` = `"playground"`
   - `service.instance.id` = hostname or a UUID generated at init
   - `deployment.environment.name` = `PARALLAX_ENV` env or `"lab"`
   - `parallax.run.id` = value of env `PARALLAX_RUN_ID` **if set** (fallback
     path; `parallax run start` normally injects it via
     `OTEL_RESOURCE_ATTRIBUTES`, which the SDK already merges — do not
     duplicate if both present: only set the KeyValue when the env var exists
     AND `OTEL_RESOURCE_ATTRIBUTES` does not already contain
     `parallax.run.id`).

**Verify**: `rtk cargo build && rtk cargo clippy --all-targets -- -D warnings`
→ exit 0.

### Step 2: Extraction + injection + status helpers in the shared lib

Add a new module `libs/playground-telemetry/src/propagation.rs` (re-export
from `lib.rs`) providing:

1. **Axum extraction middleware** — an `axum::middleware::from_fn`-compatible
   function `extract_context` that reads the incoming `http::HeaderMap` via
   `opentelemetry_http::HeaderExtractor`, calls
   `global::get_text_map_propagator(|p| p.extract(&HeaderExtractor(headers)))`,
   and stores the parent context on the request-handler span:
   the simplest reliable shape is a middleware that creates nothing itself but
   stashes the extracted `Context` in request extensions, plus a helper
   `parent_from_request(...)`; HOWEVER the existing handlers create their span
   via `#[tracing::instrument]`, so the practical pattern is:
   inside each handler (first line), call a helper
   `playground_telemetry::set_parent_from(&headers)` that does
   `tracing::Span::current().set_parent(extracted_cx)` (via
   `tracing_opentelemetry::OpenTelemetrySpanExt`). Add
   `axum::http::HeaderMap` as an extractor argument to each handler. Pick ONE
   of the two shapes and use it consistently; the helper-in-handler shape
   avoids middleware/span ordering pitfalls with `#[tracing::instrument]`.
2. **Tonic server extraction** — `struct MetadataExtractor<'a>(&'a tonic::metadata::MetadataMap)`
   implementing `opentelemetry::propagation::Extractor`, plus helper
   `set_parent_from_grpc(metadata: &MetadataMap)` mirroring (1).
3. **Reqwest injection** — `pub async fn traced_get(url: &str) -> reqwest::Result<reqwest::Response>`
   (or a `inject_headers(&mut HeaderMap)` helper used with a shared
   `reqwest::Client`): inject the current span context via
   `opentelemetry_http::HeaderInjector` into the outgoing headers.
   Move the existing `MetadataInjector` from `checkout/src/main.rs:221-230`
   into this module so gRPC injection is shared too.
4. **Error status helper** — `pub fn mark_span_error(err_type: &str)`:
   sets `otel.status_code = ERROR` on the current span. With
   `tracing-opentelemetry`, setting the field works only if declared in the
   `#[instrument]` fields — so implement via
   `OpenTelemetrySpanExt`: `tracing::Span::current().set_status(opentelemetry::trace::Status::error(...))`
   and also `set_attribute("error.type", err_type.to_string())`.

Dependencies: add `opentelemetry-http = "0.32"` (HeaderExtractor/Injector;
match the workspace otel minor), `http`, and for Step 4 `tower-http = { version = "0.6", features = ["cors"] }`
— all to the workspace `Cargo.toml` with `workspace = true` references. Keep
reqwest on `native-tls` (repo TLS rule: never enable a rustls feature).

**Verify**: `rtk cargo build && rtk cargo clippy --all-targets -- -D warnings`
→ exit 0.

### Step 3: Adopt the helpers in all six services + CLI

- `services/pricing/src/main.rs`: in `quote` and `quote_stream`, first line
  `playground_telemetry::set_parent_from_grpc(request.metadata());`.
- `services/inventory/src/main.rs`, `services/recommendation/src/main.rs`,
  `services/notifications/src/main.rs`, `services/checkout/src/main.rs`
  (its axum handlers `checkout`, `quote_stream`): add `headers: HeaderMap`
  extractor argument and call `set_parent_from(&headers)` first.
- `services/checkout/src/main.rs`: replace both `reqwest::get(&url)` calls
  (`reserve` `:283`, `recommend` `:291`) with the traced client helper;
  switch the local `MetadataInjector` uses to the shared one.
- `services/orders/src/main.rs`: consumer/producer paths — call
  `mark_span_error` on the failure/dead-letter branches.
- Error status adoption at every deliberate-failure branch listed in Current
  state: checkout payment-failure (`:137-155`) and pricing-failure (`:182-188`),
  inventory `fail` branch (`:34-40`), orders failure/dead-letter, cli non-zero
  exit path. Keep the existing `tracing::error!` lines — add
  `mark_span_error("...")` beside them with a stable `error.type` string
  (e.g. `"payment_failure"`, `"out_of_stock"`, `"pricing_unavailable"`,
  `"poison_message"`, `"nonzero_exit"`).

**Verify**: `rtk cargo build && rtk cargo clippy --all-targets -- -D warnings && rtk cargo fmt --all`
→ exit 0, no fmt diff (`git diff --stat` shows only intended files).

### Step 4: CORS for the browser-reachable service

Add `tower_http::cors::CorsLayer` to **checkout**'s router (the only service
the web app calls — verify by grepping `web/src` for the checkout URL/port
8088 before wiring more):

```rust
let cors = CorsLayer::new()
    .allow_origin(tower_http::cors::AllowOrigin::mirror_request()) // lab-only stack; if a WEB_ORIGIN env exists, prefer exact origin
    .allow_methods([http::Method::GET, http::Method::POST])
    .allow_headers([
        http::header::CONTENT_TYPE,
        http::HeaderName::from_static("traceparent"),
        http::HeaderName::from_static("tracestate"),
        http::HeaderName::from_static("baggage"),
    ]);
let app = Router::new().route(...).layer(cors);
```

Prefer an exact `WEB_ORIGIN` env-configured origin over mirror if simple;
document the chosen default in a code comment. This is a local lab stack —
permissive CORS here is acceptable and intentional; note it in the comment.

**Verify**: `rtk cargo build` → exit 0. Manual (if stack is run):
`curl -i -X OPTIONS http://localhost:8088/checkout -H "Origin: http://localhost:5173" -H "Access-Control-Request-Method: GET" -H "Access-Control-Request-Headers: traceparent"`
→ 2xx with `access-control-allow-headers` containing `traceparent`.

### Step 5: Java resource identity in compose

In `deploy/docker-compose.yml` (and mirror in `docker-compose.xlang.yml`
where Java services appear): for each Java service (catalog, payment,
fulfillment), append to its environment:

```yaml
OTEL_RESOURCE_ATTRIBUTES: "service.version=0.1.0,service.namespace=playground,deployment.environment.name=${PARALLAX_ENV:-lab}"
```

(If an `OTEL_RESOURCE_ATTRIBUTES` already exists for a service, merge keys
rather than duplicating the variable.)

**Verify**: `docker compose -f deploy/docker-compose.yml config` → exit 0 and
the rendered env contains the merged attributes for all three Java services.

### Step 6: End-to-end trace check (manual, records the win)

With a local `parallax serve` running (Parallax repo: `cargo run -p
parallax-cli -- serve` or the documented invocation — see its README) and the
playground compose pointed at it (`OTEL_EXPORTER_OTLP_ENDPOINT` override, or
the lab as-is), run `scenarios/a1-checkout.sh` and, in the Parallax UI trace
detail for one checkout request, confirm: checkout SERVER span is the root;
pricing/inventory/recommendation spans are **children in the same trace**
(not separate one-span traces); a `?fail=1` request shows ERROR status on the
checkout span. Record the observed trace id in the commit message body or
the PR/commit notes.

## Test plan

- Playground has minimal test infra; the gates are build + clippy + the
  manual Step 6 check. Add one unit test in
  `libs/playground-telemetry/src/propagation.rs` for the extractor round-trip:
  inject a context into a `HeaderMap` with the injector helper, extract it
  with the extractor helper, assert the span context (trace id) survives.
  Run with `rtk cargo test -p playground-telemetry` (or `cargo nextest run`
  if the repo has nextest configured — check; plain `cargo test` acceptable
  here).

## Done criteria

ALL must hold (playground repo):

- [ ] `rtk cargo build` exit 0; `rtk cargo clippy --all-targets -- -D warnings` exit 0
- [ ] `rtk grep -rn "reqwest::get" services/` → no matches (all HTTP client
      calls go through the traced helper)
- [ ] `rtk grep -rln "set_parent_from" services/ | wc -l` ≥ 4 (all axum
      services + checkout) and pricing uses `set_parent_from_grpc`
- [ ] `rtk grep -rn "mark_span_error" services/ cli/ | wc -l` ≥ 6
- [ ] Propagation round-trip unit test passes
- [ ] `docker compose -f deploy/docker-compose.yml config` exit 0 with Java
      resource attrs present
- [ ] Step 6 manual check recorded (same-trace children + ERROR status)
- [ ] Status row updated in Parallax repo `plans/README.md`

## STOP conditions

- opentelemetry 0.32 API surface differs from the shapes above (e.g.
  `set_status` unavailable on `OpenTelemetrySpanExt`) — report the actual API
  and the closest working shape before writing all six services.
- `opentelemetry-http` at the workspace's otel minor doesn't exist / pulls a
  conflicting otel version — STOP; do not hand-roll header maps silently
  (report the version matrix instead).
- Adding `HeaderMap` extractors changes any handler's route signature in a
  way axum rejects — report rather than restructuring handlers.
- advisor-plans/034 already landed its composite-propagator change (check
  `git log` for it) — skip Step 1's propagator part and reuse its helper;
  report the merge.

## Maintenance notes

- **Overlap with advisor-plans/034**: 034 (execution-stack) also plans a
  composite propagator + env-context extraction in this same lib. Whichever
  lands second must reuse, not duplicate. This plan deliberately does NOT add
  034's `TRACEPARENT`-env extraction helper.
- Plans 042/047/048/049/050/054 (playground scenario families) all assume
  this plan's helpers exist — land this first.
- Reviewer: check every handler got extraction (easy to miss one), that
  `error.type` strings are stable/low-cardinality, and that no rustls feature
  crept in via `tower-http`/`opentelemetry-http`.
- Deferred: per-message stream events and per-attempt retry spans (plan 049);
  browser-side baggage propagator registration (plan 050).
