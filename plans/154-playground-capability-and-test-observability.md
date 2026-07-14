# Plan 154: Playground full-capability coverage and test observability

> **Executor instructions**: This plan spans the companion
> `parallax-telemetry-playground` repository (baseline `ad6cbfa`, branch
> `main`). Do not create a branch or PR in either repository. This plan was
> collected as an information-only packet on 2026-07-14; no code changed at
> planning time. It owns the operator-directed playground expansion program;
> plan 122 keeps ownership of historical-residual reconciliation and must
> classify any row owned here as "owned by 154", never duplicate it.

## Status

- **Priority**: P1
- **Effort**: XL
- **Risk**: MEDIUM
- **Depends on**: none hard inside this repo; soft 105/111/119 for the
  Parallax-side display follow-ups recorded in W6
- **Category**: cross-repository playground / validation / test observability
- **Planned at**: `cb7c514`, 2026-07-14 (playground baseline `ad6cbfa`)
- **Status**: IN PROGRESS

## Why

The playground is the comparison payload for Parallax against Maple, SigNoz,
OpenObserve, and Sentry over one Rotel fan-out. A full two-repo audit on
2026-07-14 found:

1. Several capabilities are **claimed but broken or absent**, so backend
   comparisons on those axes are currently invalid (worst: W3C baggage is not
   actually propagated; the Java tier emits nothing to Sentry despite README,
   YAML, and source comments saying it does).
2. The operator's stated technology matrix has holes: **no Rust GraphQL
   service exists at all**, Java gRPC is unary-only, and there is no Java gRPC
   client hop.
3. **Test coverage is near zero** (23 inline Rust unit test functions, zero
   Java tests, zero web tests, zero e2e), and **no test run emits telemetry**.
   The operator direction (2026-07-14) is: every playground service must be
   covered by tests, and the tests themselves must be observable — per-test
   spans, failure messages, and stitched app traces visible in Parallax and in
   every comparison backend, for Rust (cargo-nextest), Java (JUnit 5), and the
   frontend (Playwright). A failed Playwright test must surface its error and
   the full distributed trace behind the failing interaction.

## Current state (audit evidence, 2026-07-14)

> Re-audited 2026-07-14 (same day, later session): zero commits in either
> repository since planning (`parallax` `8f24808`, playground `ad6cbfa`);
> **no item below is complete** — all workstreams stand. Spot-checks
> re-confirmed the two highest-risk claims: Java `build.gradle.kts` Sentry
> matches are comments only (no dependency), and checkout only logs
> `tenant`/`tier` (baggage never enters the OTel context).

### Verified working

Cross-language trace spine (browser → Rust axum/tonic → Java agent 2.29 →
Kafka/redpanda → Java → Rust) with W3C tracecontext everywhere including SSR
`<meta traceparent>` and an env-var carrier for the CLI process boundary; gRPC
rpc semconv with streaming, deadline, retry, cancel chaos; Java GraphQL depth
(DataLoader batch vs N+1, partial errors, WS subscription); real Postgres DB
spans + pool metrics (inventory, stable db semconv); typed log events (A29);
tokio/JVM runtime metrics; `parallax.run.id`, canary redaction corpus, cron
suite, execution-stack (A27), evidence-gap scenarios (B21–B23); Rotel
per-signal fan-out with Sentry correctly excluded from metrics.

### Broken or absent versus claims

1. **A10 baggage fake**: checkout only logs `tenant`/`tier`; never sets OTel
   baggage into context; nothing propagates.
2. **Java Sentry not wired**: `deploy/Dockerfile.java` deliberately ships the
   upstream OTel agent; no Sentry dependency exists in any
   `build.gradle.kts`; README line "single Sentry+OTel javaagent", the
`sentry.*` blocks in all three `application.yml` files, and source comments
are dead claims. A15/A16/A17 are impossible on the Java tier as deployed.

### Implementation progress

- `9ff4254` in the linked playground PR makes the A10 checkout baggage path
  real: `tenant.id` and `user.tier` are attached to the OpenTelemetry parent
  context and injected by the existing composite W3C tracecontext+baggage
  propagator into both HTTP and gRPC downstream carriers. The shared
  propagation fixture round-trips both keys. `4e9602e` additionally stamps
  extracted baggage onto the inventory HTTP and pricing gRPC server spans;
  focused Rust tests and clippy pass. `c8a89a8` adds the generated
  cross-repository `TENANT_ID` convention.
- `950f09c` adds the A10 scenario and a public-boundary Rust integration test.
  The fixture proves the exported HTTP injector/extractor preserves both
  business baggage keys; cargo test, strict clippy, formatting, and scenario
  shell syntax pass locally. The full live-backend rendering assertion remains
  part of the final non-Docker evidence sweep.
- `a21585f` adds the shared Axum middleware and applies it to every Rust HTTP
  service. It emits the standard server span attributes and
  `http.server.request.duration` RED histogram; focused Rust checks pass.
- `dfdd066` moves flagd evaluation into the shared Rust telemetry library and
  connects `cacheLeak`, `poisonMessage`, and `canaryFailure` to their real
  recommendation, orders, and checkout chaos paths, with explicit environment
  fallback and `feature_flag.*` evaluation logs. Focused Rust tests and clippy
  pass.
- `f8c7098` records the default Java Sentry decision in executable form:
  retain the upstream OTel agent as the sole OTLP fan-out path and add Sentry's
  Spring Boot 4-compatible Jakarta starter (`8.46.0`) to catalog, payment, and
  fulfillment. The three services now have DSN, environment, release, trace,
  and profiling configuration; Docker and compose claims match this two-path
  design. Gradle execution remains deferred because its arm64 native platform
  loader currently fails before project configuration in this host.
- `277eb69` completes the ready documentation/runtime drift cleanup: the
  playground now provisions Bun through mise, production `web start` executes
  Bun instead of Node, and README Spring gRPC/Sentry wording matches the live
  dependency model. Reinstalling the foreign web dependency tree with Bun made
  the local production build and TypeScript check pass.
- `fcbfbd9` starts W2 with a real `storefront` workspace service using Juniper
  0.17, `juniper_axum`, and `juniper_graphql_ws`. It exposes GraphQL HTTP,
  both WebSocket subscription protocols, a catalog GraphQL gateway resolver,
  and a pricing gRPC gateway resolver. Resolver spans emit the required
  GraphQL operation/document/field attributes; the compose topology exposes
  the service on port 8094. Cargo check, test, strict clippy, and formatting
  pass locally.
3. **Rust HTTP semconv absent**: manual axum spans carry no
   `http.request.method` / `http.route` / `http.response.status_code` and no
   `http.server.request.duration` histogram. Backend HTTP/service views and
   Parallax's own exemplar card key on exactly these names.
4. **flagd orphans**: `cacheLeak`, `poisonMessage`, `canaryFailure` are
   declared in `flags/flagd.json` but read by no service (chaos runs on query
   params), weakening the A14 flag story.
5. **orders messaging attrs non-semconv**: custom `messaging.delivery.lag_ms`,
   `messaging.orphan`; missing `messaging.system`,
   `messaging.destination.name`, `messaging.operation.type`.
6. **Kafka producer→consumer span link unverified**: comment-only claim;
   depends entirely on agent behavior.
7. **Doc drift**: VERIFICATION.md says OTel 0.30 (workspace is 0.32);
   README/VERIFICATION span counts disagree; spring-grpc version prose wrong;
   web `start` entrypoint mismatch; `MetricsLayer` in the shared lib is dead
   code (no service emits its field prefixes).

### Technology-matrix holes (operator focus list)

| Focus | Status |
|---|---|
| Java Spring Boot | present (3 services, Boot 4.1) |
| GraphQL in Java | present (catalog, deep) |
| GraphQL in Rust | **absent** — no service; design A23/A24 unowned |
| gRPC in Rust | present (server incl. streaming + client) |
| gRPC in Java | server unary only; no `QuoteStream`; no Java gRPC client |

### Test inventory

Rust: 23 inline `#[test]`/`#[tokio::test]` functions (orders 3, pricing 1,
checkout 2, recommendation 2, inventory 2, shared lib 13); no `tests/`
integration directories. Java: zero test sources. Web: zero tests, no
Playwright anywhere. CLI: untested. No JUnit XML, no test telemetry, no CI
test gate in the playground repo.

### Tooling facts (researched 2026-07-14, versions current then)

- OTel semconv 1.43.0: `test.*` registry is 4 attributes
  (`test.case.name`, `test.case.result.status` pass|fail, `test.suite.name`,
  `test.suite.run.status`) at **Development** stability — centralize as
  constants, expect renames. `cicd.pipeline.*` promoted to **Release
  Candidate** — safer to lean on (`cicd.pipeline.run.id`,
  `cicd.pipeline.task.type=test`, results).
- Playwright: `playwright-opentelemetry` (endformdev, v0.12.1, active) emits a
  root span per test, step spans, injects W3C `traceparent` into browser
  requests so app spans join the test trace; fallback is a custom Reporter
  (~200 LOC; reporter runs in the main process — use `tracer.startSpan` with
  explicit timestamps keyed by `test.id`, never `startActiveSpan`;
  `forceFlush` in `onEnd`) plus a traceparent-injecting fixture or meta-tag
  (Tracetest/autotel pattern). Playwright's own `trace.zip` has no OTel
  converter; attach its path from `result.attachments` as a span attribute.
- cargo-nextest: stable JUnit XML per profile
  (`[profile.ci.junit] path`), per-test time + failure output;
  libtest-json-plus stream is experimental. Each test runs in its own process
  → a per-process `tracing` + OTLP subscriber is contention-free; must
  `force_flush` before exit and use an exporter that works without an ambient
  tokio runtime; parent from `TRACEPARENT` env. `junit2otlp` (Go) exists as a
  converter precedent; `otel-cli exec` wraps a command in a span and exports
  `TRACEPARENT` to children.
- Java: no official Gradle OTel extension; community Gradle plugin
  `com.atkinsondev.opentelemetry-build` v4.6.2 (maintained; Java 21+, Gradle
  8.4+) emits a span per task **and per test** with failure message + stack.
  JUnit 5 span-per-test extensions are all archived; writing one is ~150 LOC
  (`InvocationInterceptor` callbacks). The OTel Java agent on the test JVM
  instruments the code under test (HTTP/JDBC/Kafka during integration tests)
  but creates no per-test spans; Spring Boot test observability is off by
  default — `@AutoConfigureObservability` re-enables it.
- Juniper (`graphql-rust/juniper`): official `juniper_axum` integration;
  subscriptions via `juniper_graphql_ws` (graphql-transport-ws); **no built-in
  tracing/OTel instrumentation** — per-resolver spans are manual, which is
  acceptable because they must emit exactly the `graphql.operation.*` /
  `graphql.field.*` names Parallax's field-tree view parses.

## Scope

In scope (all in the companion repo unless marked Parallax-side):

- W1 correctness fixes; W2 technology-matrix completion (Juniper); W3 test
  coverage for every service; W4 test-run telemetry on all three stacks;
  W5 signal depth + scenario-script completion; W6 Parallax-side follow-up
  triggers (recorded, not implemented here).

Out of scope:

- New comparison backends, brokers, databases, or topology for breadth alone.
- A scored comparison harness (remains manual per operator decision).
- Profiling emission (Parallax cannot ingest profiles; trigger stays in the
  ledger) beyond keeping existing config documented as inert.
- Sentry-envelope ingestion in Parallax (plan 118) and any Parallax product
  contract change from the companion repo.
- async-graphql (operator 2026-07-14: Juniper only for Rust GraphQL).

## Steps

### W1 — Correctness fixes (do first; every later comparison depends on them)

1. Make A10 baggage real: set `tenant.id`/`user.tier` into the OTel context in
   checkout, propagate over HTTP + gRPC, stamp a downstream span attribute in
   pricing and inventory, assert in `scenarios/` and in a Rust integration
   test.
2. Add HTTP semconv to every Rust axum service (`http.request.method`,
   `http.route`, `http.response.status_code`, `url.*` on server spans;
   `error.type` on failures) and emit `http.server.request.duration`
   histograms; pricing keeps `rpc.server.duration`. This lights up RED views
   in every backend and Parallax's exemplar card.
3. Java Sentry decision (operator pick recorded here as default = wire it, per
   design doc): switch `Dockerfile.java` to the Sentry OTel agent distribution
   and add the Sentry Gradle deps, or delete every Java Sentry claim + dead
   YAML. No third state.
4. Wire `cacheLeak`, `poisonMessage`, `canaryFailure` flags to their chaos
   paths (recommendation, orders, checkout) so flag flips — not query params —
   drive B6/B8/B12, and `feature_flag.*` evaluations appear on all tiers.
5. Fix orders messaging semconv (`messaging.system`,
   `messaging.destination.name`, `messaging.operation.type`; keep lag/orphan
   as extra attrs). Verify the Kafka producer→consumer link renders in
   Parallax + one other backend; if the agent emits none, add an explicit link
   in fulfillment or record the agent limitation in VERIFICATION.md.
6. Purge doc drift: VERIFICATION versions/counts, README agent claim,
   spring-grpc prose, web `start` note; delete the dead `MetricsLayer` wiring
   or start using its field prefixes — not both.

### W2 — Technology matrix (Juniper)

7. New Rust GraphQL service `services/storefront` (name free) using
   **Juniper + `juniper_axum`**, subscriptions via `juniper_graphql_ws`:
   fronts catalog (GraphQL→GraphQL, A24) and pricing over gRPC
   (GraphQL→gRPC gateway, A23). Manual per-resolver spans emitting
   `graphql.operation.type/name`, `graphql.document`, `graphql.field.name`,
   `graphql.field.path` — the exact names Parallax's field-tree view consumes;
   include one deliberate N+1 resolver and one partial-error field to mirror
   catalog's shapes for cross-language GraphQL comparison.
8. Implement `QuoteStream` in Java payment (server-streaming parity with Rust
   pricing) and add one Java gRPC **client** hop (fulfillment or catalog →
   pricing) so both directions and both roles exist in both languages.
9. Add `scenarios/a23-*.sh` and `scenarios/a24-*.sh` plus run.sh dispatch.

### W3 — Test coverage (every service, no exceptions)

10. Rust per-service integration tests (`tests/` dirs, run by cargo-nextest):
    axum handlers via in-process `tower::ServiceExt` calls plus one
    spawned-server test per service; tonic services over an in-process
    channel; inventory against real Postgres (compose-provided; skip cleanly
    with a diagnostic when `DATABASE_URL` is absent); orders link/poison/lag
    logic; checkout retry/timeout/degradation branches; notifications once W1
    fleshes it out; CLI cron/daemon exit codes and env-carrier propagation.
11. Java: JUnit 5 across all three services — `@GraphQlTest` slices +
    `@SpringBootTest` for catalog (batch vs N+1 vs partial error),
    spring-grpc in-process server tests for payment (unary + new stream),
    fulfillment with embedded/testcontainer Kafka for the publish→consume→
    notify path.
12. Web: Vitest (Bun-run) unit tests for `telemetry.ts`, `traceparent.ts`,
    `rum.ts`; Playwright e2e for the A28 journey (home → checkout → orders,
    `?nopropagate=1` gap, forced RUM error), replacing today's manual-only
    instructions.
13. CI: one workflow in the playground repo running fmt/clippy/nextest, Gradle
    tests, `bun test`, and Playwright headless, all emitting the W4 telemetry.

### W4 — Test-run telemetry (the operator's visibility requirement)

14. Shared semconv constants for `test.*` + `cicd.pipeline.*` in
    `libs/playground-telemetry` (Rust), the Java shared conventions class, and
    `web/src/semconv.ts`; single source list documented in the repo README.
    When plan 119's registry codegen lands, these constants migrate to
    generated output.
    **Identity contract (plan 155 D1 — the payload must emit what the Tests
    surface consumes):** every test root span carries (a) an explicit
    `parallax.test.id` on at least one suite per stack to exercise the
    override path, (b) a code-reference name path that never encodes
    line/column (Playwright's default fullName is line/column-fragile — the
    reporter must emit qualified title paths instead), (c) parameters as
    attributes for at least one parameterized test per stack (variant
    history), (d) configuration attributes (`test.configuration.*`-shaped:
    os/browser/environment) kept out of any identity value, (e) attempt
    ordinals on retries plus one deliberate pass-after-fail retry per stack
    (flaky-pass evidence), (f) one assertion failure AND one harness error
    per stack so failed-vs-broken derivation is testable, and (g)
    `vcs.ref.head.revision` + `service.version` on test resources so
    same-commit flaky detection and version-under-test attribution work.
15. Run-level parent: every test session runs under
    `parallax run start -- <runner>` so the whole session is a Parallax Run
    with `parallax.run.id`; export `TRACEPARENT` into child processes so all
    per-test spans join one session trace (CI job id recorded as
    `cicd.pipeline.run.id`).
16. Rust: (a) nextest CI profile writes JUnit XML; a small converter
    (`cli` subcommand `playground test-report <junit.xml>`) emits one OTLP
    span per test with `test.*` attrs, failure message + stack as span status
    and events; (b) opt-in per-test in-process telemetry helper in the shared
    lib (per-process subscriber, simple/blocking exporter, `force_flush`
    guard, parent from `TRACEPARENT`) for integration tests whose internal
    service-to-service calls must be visible as child spans.
    Mechanism refinements (research doc §5): identity/attempt attrs come
    from nextest env (`NEXTEST_BINARY_ID`, `NEXTEST_TEST_NAME`,
    `NEXTEST_ATTEMPT`, `NEXTEST_ATTEMPT_ID`); use `SimpleSpanProcessor` or
    explicit provider shutdown — libtest exits via `process::exit` on
    failure, so Drop-based flushing loses exactly the failing tests; the
    JUnit XML converter is also the gap-fill for SIGKILL/timeout-killed
    tests. When plan 155 D9 ships product adapters, the playground migrates
    to them and keeps only scenario-specific glue.
17. Java: adopt `com.atkinsondev.opentelemetry-build` for per-task/per-test
    spans (failure message + stack on the span); attach the OTel Java agent to
    the test JVM (`test { jvmArgs }`) + `@AutoConfigureObservability` so
    integration-test HTTP/gRPC/Kafka/JDBC client spans of the code under test
    nest beneath; add the thin JUnit 5 extension only if the plugin's
    attribute names cannot be mapped to the shared `test.*` constants.
    Known plugin losses to record in VERIFICATION.md (research doc §5.1):
    displayName-only identity, stacks truncated to 5 frames, no attempt
    counter, non-semconv attribute names, degraded config-cache mode;
    Gradle `mergeReruns=true` JUnit XML is the authoritative flaky/rerun
    record. Forward `TRACEPARENT`/`PARALLAX_RUN_ID` explicitly via the
    `Test` task environment (Gradle daemon does not inherit the shell).
18. Playwright: evaluate `playwright-opentelemetry` (endformdev) first; if its
    span model or Bun compatibility fails, write the custom Reporter
    (explicit-timestamp spans keyed by `test.id`, step spans, `forceFlush` in
    `onEnd`) + a traceparent-injecting fixture so browser requests — and
    therefore the entire backend trace — join the test span's trace. On
    failure: span status ERROR, error message + stack as attributes/events,
    Playwright `trace.zip` path attached as an artifact attribute.
19. Acceptance demo (this is the point of W4): one failed Playwright test and
    one failed Rust integration test, each visible in Parallax (as a Run with
    session trace, failing test span, error, and stitched app spans) and
    inspected side-by-side in Maple/SigNoz/OpenObserve/Sentry; record what
    each backend can and cannot show about test telemetry in
    VERIFICATION.md's comparison rubric (test-observability becomes a rubric
    dimension alongside run-scoping and redaction).

### W5 — Signal depth and scenario completion

20. ExponentialHistogram conformance probe: enable
    `base2_exponential_bucket_histogram` on one Java service's agent; record
    per-backend behavior (Parallax is expected to drop it — that feeds the
    existing trigger-ledger row, not a product change).
21. Give catalog a real Postgres (Spring JDBC) so agent-emitted Java `db.*`
    spans exist alongside Rust's hand-rolled stable names — a live old-vs-new
    db semconv rendering comparison.
22. Scripts + run.sh dispatch for every scriptless scenario: A2 (exemplar
    verification against each backend), A5/A28 (Playwright-driven), A8, B2,
    B5, B6, B10, B13, B15 (Playwright rage-click), B16 (k6 dispatch entry).
23. New cross-language error-grouping scenario: identical `PaymentError` from
    Rust checkout and Java payment; record grouping behavior per backend
    (Sentry groups, OTLP-only backends mostly do not, Parallax fingerprint in
    between).

### W6 — Parallax-side follow-ups (recorded triggers only; no implementation here)

- **Test reporting surface is now plan 155** (operator-directed 2026-07-14):
  a dedicated `/tests` page with identity registry, attempt chains,
  failed-vs-broken taxonomy, flaky state machine, and shared-fingerprint
  fusion with production issues. Evidence base:
  `docs/research/market/test-reporting-ecosystem.md`. This plan's W4 payload
  is plan 155's live verification input; the identity contract in step 14 is
  the shared cross-repo contract.
- Playground already emits other data Parallax cannot display: web-vitals/RUM
  spans, baggage, `feature_flag.*`. Candidate future plans: RUM/web-vitals
  view, baggage surfacing, feature-flag view. Metric stubs stay plan 105;
  exp-histogram ingest stays a ledger trigger; Sentry envelope stays plan
  118.

## Test Plan

- Every W1/W2 change lands with the corresponding W3 test; nextest, Gradle
  test, `bun test`, and Playwright suites all green locally and in the
  playground CI workflow.
- Propagation assertions: baggage attribute visible on pricing + inventory
  spans in a captured OTLP fixture; Juniper gateway trace shows
  browser→storefront→catalog and storefront→pricing gRPC in one trace.
- Test-telemetry assertions: a deliberately failing test in each stack
  produces a span with `test.case.result.status=fail`, failure message, and a
  child/linked app trace, verified via Parallax GraphQL queries (`runs`,
  `traces_by_run`) — not screenshots.
- Fan-out check: one scenario sweep with all five backends up; per-backend
  rendering notes updated in VERIFICATION.md.

## Done Criteria

- [ ] All seven W1 defects fixed or (Java Sentry) explicitly decided and
      consistent across code + docs.
- [ ] Juniper storefront service exists with per-resolver GraphQL semconv
      spans; A23/A24 scripts run; Java `QuoteStream` + Java gRPC client hop
      exist.
- [ ] Every service (9 app services + cli + web) has passing tests wired into
      a playground CI workflow.
- [ ] Test runs on all three stacks emit OTel spans with the shared `test.*` /
      `cicd.*` constants, parented under a `parallax run` session; the W4 §19
      acceptance demo is recorded in VERIFICATION.md.
- [ ] A failed Playwright test shows its error and the full stitched app trace
      in Parallax; per-backend test-telemetry rendering notes exist for the
      other four backends.
- [ ] Every scenario ID in the catalog has a `run.sh` dispatch entry or an
      explicit "requires live host X" note; no scriptless verified claims
      remain.
- [ ] VERIFICATION.md and README claims match code (versions, counts, agent,
      entrypoints); dead config/flags/MetricsLayer removed or wired.

## STOP Conditions

- The operator rejects the default Java Sentry decision in step 3 — stop that
  step and record the alternative; everything else proceeds.
- Juniper cannot serve the required subscription/gateway shape on current
  stable releases — stop W2 step 7 and record the exact upstream limitation;
  do not substitute async-graphql without operator approval.
- `playwright-opentelemetry` and a custom reporter both prove incompatible
  with the Bun-only rule — stop step 18 and record the conflict for an
  operator tooling decision.
- Any change would require a new backend, broker, or product fallback.
- Cross-repository wire behavior (semconv names, run.id contract) would drift
  from Parallax consumers without a compatibility decision.

## Remove When

All done criteria are checked, the acceptance evidence lives in the playground
VERIFICATION.md (and, where durable, `docs/research/validation/`), and plan 122's
disposition table confirms no row here remains open; delete this file and its
index row in the same commit.
