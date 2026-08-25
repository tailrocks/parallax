# Parallax public introduction and readiness review

**Review date:** 2026-08-25  
**Scope:** `parallax` and `parallax-telemetry-playground`  
**Decision:** publish as a transparent pre-release local observability/context project; do not present it as a production replacement for Sentry, Grafana, or Kibana yet.

## Executive summary

Parallax has a credible, demonstrable core:

- OTLP traces, logs, and metrics over gRPC and HTTP.
- Bounded Sentry-envelope ingestion and deterministic issue derivation.
- Trace/log/metric correlation, service/dependency views, SQL, GraphQL, CLI, and UI.
- Bounded evidence bundles for humans and coding agents.
- Local read-only MCP access for issue context and agent-session context.

The playground is a strong workload and verification harness. It generates realistic Rust, Java, browser, gRPC, GraphQL, Kafka, database, retry, chaos, RUM, and error traffic. It is not an observability backend or console; Parallax supplies the storage, query, issue, and visualization surfaces.

Current public-readiness verdict: **good for a documented local demo and early adopters; not ready for an unqualified public-production claim.**

## Evidence map

| Audience question | Current proof | Evidence |
| --- | --- | --- |
| How are logs, traces, and metrics collected? | OTLP/gRPC and OTLP/HTTP endpoints feed a durable spool/queue/worker path; signals are normalized before storage. Basic RED, histogram, and exemplar paths exist, but exponential histograms and summaries are dropped. | `crates/parallax-server/src/{otlp_grpc,otlp_http}.rs`; `crates/parallax-ingest/src/{logs,traces,metrics}.rs`; `crates/parallax-ingest/src/tests.rs` |
| How are they displayed? | GraphQL and UI expose issues, traces, logs, metrics, services, dashboards, investigations, and SQL; CLI exposes live logs, traces, metrics, and bundles. Known UI discrepancies remain in ecosystem/invocation routes, exemplar links, clock-skew, and MCP parity. | `crates/parallax-api/src/lib.rs`; `ui/src/routes/`; `parallax-telemetry-playground/docs/coverage-matrix.md` |
| How do we see cross-service communication? | Trace lookup, invocation grouping, trace/log joins, service maps, dependency edges, and trace-linked errors are implemented. | `crates/parallax-storage/src/adapter/traits.rs`; `crates/parallax-greptime/src/greptime/invocation_store.rs`; `crates/parallax-api/src/lib.rs` |
| How do we inspect errors? | Exception spans and ERROR/FATAL logs derive deterministic issues; bounded Sentry envelopes are also accepted. Occurrences, trends, evidence gaps, anchored traces/logs, and bundles are queryable. Cross-language same-error grouping is not established. | `crates/parallax-server/src/worker.rs`; `crates/parallax-analysis/src/{derive,fingerprint}.rs`; `crates/parallax-api/src/lib.rs`; `parallax-telemetry-playground/VERIFICATION.md` |
| Does the demo exercise real distributed behavior? | Browser → Rust → Rust/Java services, GraphQL, gRPC, Kafka, reverse HTTP, baggage, retries, failures, and broken propagation scenarios exist. | `parallax-telemetry-playground/README.md`; `TOUR.md`; `docs/coverage-matrix.md` |

## Recommended public story

Use this positioning:

> Parallax is a local-first, self-hosted OTLP execution-context engine with bounded Sentry-envelope ingest. It turns errors, traces, logs, metrics, runs, and agent sessions into bounded, redacted evidence for developers and coding agents.

Use the playground as:

> A polyglot telemetry lab that produces repeatable distributed-system failures and lets users inspect the same evidence in Parallax.

Avoid these claims until measured and hardened:

- “Sentry replacement” or full multi-SDK/grouping parity.
- “Grafana replacement.”
- “Kibana alternative” without narrowing it to correlated structured incident logs.
- “Cheaper at scale.” Current footprint evidence is laptop-local only.
- “Production benchmark.” The playground is a scenario/coverage harness, not scale proof.
- “Autonomous fixer” or “proven superior AI-agent context.” Bundle value remains an open validation gate.

## Capability walkthrough for a future public repository

Introduce features in this order. Each chapter should include one command, one screenshot or live query, one architecture diagram, and one limitation.

### 1. Start locally

Run `parallax serve`, connect an instrumented app with standard OpenTelemetry, and show the local UI/API ports. State clearly that V1 is local-only and has no auth.

### 2. Collect the three signals

Use the playground to emit traces, structured logs, and metrics. Show the OTLP endpoints, normalization, durable spool, GreptimeDB native tables, and loss/health counters. State that unsupported exponential histograms and summaries are counted and dropped.

### 3. Follow one distributed request

Run scenario `a1` for browser → checkout → pricing/inventory/recommendation fan-out, then show correlated logs and metric windows. Use `a23` for Java gRPC and `a3`, `a8`, and `a4` for producer/consumer, Kafka, and reverse-hop correlation.

### 4. Inspect an error as context

Run `a31` or the documented payment failure. Show issue grouping, occurrence history, linked trace, logs, metrics, evidence gaps, and the bounded `parallax issue context` bundle.

### 5. Compare propagation quality

Run `a28` for stitched versus intentionally broken browser/backend propagation. Use `a10` for baggage, `a3/a4/a8` for span links, and `b23` for orphan logs.

### 6. Give the context to an AI agent

Use the CLI bundle or local read-only MCP tools for issue and agent-session context. Remote MCP is not shipped and MCP parity has a known discrepancy. Demonstrate that Parallax supplies context and boundaries; a separate agent proposes changes. Keep A1 fix-quality results explicitly marked unproven.

### 7. Show the honest competitor boundary

Use a source-linked comparison table. The agent-context space is not empty: Sentry, Grafana, OpenObserve, Coroot, SigNoz, and others already expose investigation or agent surfaces. Parallax’s differentiation is a product hypothesis, not a proven moat.

| Tool | Stronger today | Parallax’s narrower angle |
| --- | --- | --- |
| Sentry | Issue lifecycle, SDK/ecosystem, Seer/fix workflow | Local evidence projection plus bounded Sentry-envelope ingest |
| Grafana | Dashboards, alerting ecosystem, maturity, and scale | Simpler local self-hosting and native issue derivation |
| Elastic/Kibana | Full-text/search-grade logs, ES|QL, security/SIEM | Telemetry-native local incident context |
| OpenObserve, Maple, SigNoz, Coroot, Traceway, TMA1 | Direct open/self-hosted/agent-native pressure | Must be compared directly; do not imply unique local correlation or MCP |

Canonical comparison sources: `docs/research/market/competitors/README.md`, `docs/research/market/landscape.md`, and `docs/research/market/competitors/comparison-set.md`.

## Playground scenario index

| Story | Scenario(s) | Demonstrates |
| --- | --- | --- |
| Basic waterfall | `a1` | HTTP → gRPC/HTTP fan-out and trace waterfall |
| Browser propagation | `a28` | Frontend/backend stitching and broken propagation |
| Async topology | `a3`, `a4`, `a8` | Span links, Kafka, Java → Rust reverse hop |
| GraphQL behavior | `a6` | DataLoader batching versus N+1 |
| Context propagation | `a10` | W3C baggage across services |
| Error investigation | `a31`, `c8`, `b23` | Application error, Sentry event, orphan log |
| Metrics correlation | `a1`, `a2` | RED metrics, exemplars, metric windows |

The playground README and `TOUR.md` remain the scenario source of truth. Many screenshots have dated live assertions; known discrepancies remain explicitly listed in `docs/coverage-matrix.md`.

## Public-readiness blockers

### High priority

1. **OTLP ingest does not validate the configured bearer token.** Non-loopback startup is guarded by token-required configuration validation, but OTLP routes bypass bearer middleware once bound. This can permit telemetry injection, resource exhaustion, and privacy leakage. Until ingest auth/TLS exists, enforce loopback/trusted-network binding and label it prominently.
2. **Unlimited buffered bodies in the playground web proxy.** Playground `/v1/traces` and `/v1/logs` buffer request bodies without a strict bound. Parallax’s own OTLP HTTP path has a 16 MiB default limit. Add limits, timeouts, and rate limiting before any shared-network demo.
3. **Silent telemetry loss in the playground proxy.** The trace proxy can return `202` while dropping data when its upstream is unavailable. Return an explicit failure or provide bounded queueing plus visible loss metrics.

### Medium priority

- Playground compose exposes unauthenticated APIs and uses the default database password `playground`; bind local services to loopback and require generated credentials.
- Browser tracing, replay-on-error, feedback, and console capture need documented masking/consent defaults.
- Pin Java base images and verify downloaded agents with checksums.
- Document that playground commands require `mise install` or otherwise available `protoc`; the prescribed toolchain already declares it.
- Metrics support drops exponential histograms and summaries; retain the documented loss counters or implement the formats.
- Sentry side-item support and multi-SDK compatibility are bounded, not full parity.
- Fix stale documentation, including the MCP statement in `docs/guide/agent-howto.md` and the playground catalog-ID count.
- Complete repeatable live collector + Parallax UI assertions for exemplars, Sentry grouping, and clock-skew behavior.

## Release gates

Do not call the project “public-ready” until all gates have evidence in CI or a reproducible script:

1. Fresh-machine quickstart completes with pinned prerequisites.
2. Playground `a1`, `a28`, and `a31` produce expected traces, logs, metrics, and issues in Parallax.
3. A cross-service trace assertion proves at least one Rust → Java → broker → Rust path.
4. An error assertion proves grouping, occurrence, linked telemetry, and bundle generation.
5. Ingest body limits, timeouts, bind safety, and proxy-loss behavior are tested.
6. No secret appears in checked-in fixtures, screenshots, bundles, or logs; browser capture policy is documented.
7. Docs distinguish shipped, partial, planned, and unproven capabilities, including known UI/MCP discrepancies.
8. A source-linked competitor matrix covers Sentry, Grafana, Elastic/Kibana, and direct open/self-hosted competitors without implying Parallax uniquely owns local correlation or MCP.

## Verification snapshot

Delegated bounded checks on 2026-08-25 produced this evidence:

| Check | Result | Meaning |
| --- | --- | --- |
| Playground scenario-dispatch checker | **PASS** — 89 dispatches | Scenario inventory is structurally valid. |
| Playground web unit tests | **PASS** — 5 files, 11 tests | Producer/UI logic has a green focused test set. |
| Playground TypeScript typecheck | **PASS** | Strict web typecheck is executable. |
| Playground web E2E | **PASS** — 5 passed, 2 expected W4 skips | Checkout, propagation break, rage click, orders, and RUM journeys have browser coverage; missing Rotel warnings are expected. |
| Live `a1`, `a28`, `a31` | **BLOCKED** — services unavailable on `:8088`/`:5173` | No live Parallax/playground stack was started during the bounded audit. |
| Playground Rust workspace tests | **BLOCKED** — `protoc` absent | `mise install` is required before this gate. |
| Parallax OTLP ingest normalization | **PASS** — 29/29 `parallax-ingest` tests | Signal normalization and ingest contracts are covered. |
| Parallax error derivation | **PASS** — 7/7 targeted tests | Exception/log derivation and fingerprint behavior are covered. |
| Parallax server validation/deduplication | **PASS** — 9/9 targeted tests | OTLP boundaries, gzip input, trace IDs, exemplars, and shared occurrence behavior are covered. |
| Parallax API query surfaces | **PASS** — 7/7 targeted tests | Issue, trace, log, metric, and schema query paths are covered. |
| Parallax evidence bundles | **PASS** — 31/31 targeted tests | Bundle bounds/stability and incident projections are covered. |
| Parallax local serve + seed smoke | **PASS** — managed GreptimeDB, `/health`, `/version`, `/ingest/loss`, GraphQL health, seed trace, CLI trace/log output | Local collection/query path booted and cleaned up; this is not public-network proof. |
| Playground live `a1` | **PARTIAL** — HTTP responses returned `1999`, `3998`, `5997`, `9995`; Parallax persistence failed because managed GreptimeDB archive extraction was truncated | Workload path passes; persisted Parallax UI/query evidence remains unproven. |
| Parallax end-to-end UI smoke | **BLOCKED** — GreptimeDB archive failure during live playground run | Repeat after repairing/redownloading the managed engine; retain the failed run as a known setup gate. |

Therefore the document proves source-level capability, focused web checks, and a Parallax local serve/seed smoke. It does not yet prove a fresh live playground → Parallax persistence → UI/query session because the managed GreptimeDB download failed during the bounded `a1` run.

The playground is a sibling repository. Paths written as `parallax-telemetry-playground/...` in this document refer to `/Users/donbeave/Projects/tailrocks/parallax-project/parallax-telemetry-playground/...`, not a nested directory in this repository.

## Suggested repository shape

If a separate GitHub-facing education repository is created, keep it as an audience layer, not a second implementation source:

```text
parallax-public/
  README.md                 # five-minute promise and honest limits
  01-quickstart.md          # install and first signal
  02-signals.md             # logs, traces, metrics
  03-distributed-trace.md   # playground a1/a28
  04-errors.md              # a31 and evidence bundle
  05-ai-agent-context.md    # CLI/MCP, context engine boundary
  06-competitor-boundary.md # Sentry/Grafana/Kibana comparison
  scenarios/                # thin wrappers or links; no copied service logic
  evidence/                 # dated run manifests and screenshots
```

Keep implementation and authoritative capability status in the two source repositories. The public repository should link to exact commits, scenario IDs, and evidence artifacts so examples cannot silently drift.

## Delegated review record

This review used four independent read-only subagent passes:

- core ingestion/storage/API/error capability inventory;
- playground signal-generation, topology, and UI walkthrough;
- security/reliability/public-exposure review;
- competitor-positioning and AI-agent claim audit.

The findings were consolidated here against current files. This document is a readiness assessment, not proof that every runtime gate above has passed.
