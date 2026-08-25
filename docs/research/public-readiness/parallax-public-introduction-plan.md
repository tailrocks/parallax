# Parallax public introduction and readiness review

**Review date:** 2026-08-25  
**Scope:** `parallax` and `parallax-telemetry-playground`  
**Decision:** publish as a transparent pre-release local observability/context project; do not present it as a production replacement for Sentry, Grafana, or Kibana yet.

## Executive summary

Parallax has a credible, demonstrable core:

- OTLP traces, logs, and metrics over gRPC and HTTP.
- Sentry-envelope ingestion and derived issue grouping.
- Trace/log/metric correlation, service/dependency views, SQL, GraphQL, CLI, and UI.
- Bounded evidence bundles for humans and coding agents.
- Local read-only MCP access for issue context and agent-session context.

The playground is a strong workload and verification harness. It generates realistic Rust, Java, browser, gRPC, GraphQL, Kafka, database, retry, chaos, RUM, and error traffic. It is not an observability backend or console; Parallax supplies the storage, query, issue, and visualization surfaces.

Current public-readiness verdict: **good for a documented local demo and early adopters; not ready for an unqualified public-production claim.**

## Evidence map

| Audience question | Current proof | Evidence |
| --- | --- | --- |
| How are logs, traces, and metrics collected? | OTLP/gRPC and OTLP/HTTP endpoints feed a durable spool/queue/worker path; signals are normalized before storage. | `crates/parallax-server/src/{otlp_grpc,otlp_http}.rs`; `crates/parallax-ingest/src/{logs,traces,metrics}.rs` |
| How are they displayed? | GraphQL and UI expose issues, traces, logs, metrics, services, dashboards, investigations, and SQL; CLI exposes live logs, traces, metrics, and bundles. | `crates/parallax-api/src/lib.rs`; `ui/src/routes/`; `docs/guide/quickstart.md` |
| How do we see cross-service communication? | Trace lookup, invocation grouping, trace/log joins, service maps, dependency edges, and trace-linked errors are implemented. | `crates/parallax-storage/src/adapter/traits.rs`; `crates/parallax-greptime/src/greptime/invocation_store.rs`; `crates/parallax-api/src/lib.rs` |
| How do we inspect errors? | Exception spans and ERROR/FATAL logs derive deterministic issues; occurrences, trends, evidence gaps, anchored traces/logs, and bundles are queryable. | `crates/parallax-server/src/worker.rs`; `crates/parallax-analysis/src/{derive,fingerprint}.rs`; `crates/parallax-api/src/lib.rs` |
| Does the demo exercise real distributed behavior? | Browser → Rust → Rust/Java services, GraphQL, gRPC, Kafka, reverse HTTP, baggage, retries, failures, and broken propagation scenarios exist. | `parallax-telemetry-playground/README.md`; `TOUR.md`; `docs/coverage-matrix.md` |

## Recommended public story

Use this positioning:

> Parallax is a local-first, self-hosted OTLP and Sentry-compatible execution-context engine. It turns errors, traces, logs, metrics, runs, and agent sessions into bounded, redacted evidence for developers and coding agents.

Use the playground as:

> A polyglot telemetry lab that produces repeatable distributed-system failures and lets users inspect the same evidence in Parallax.

Avoid these claims until measured and hardened:

- “Sentry replacement.”
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

Use the playground to emit traces, structured logs, and metrics. Show the OTLP endpoints, normalization, durable spool, GreptimeDB native tables, and loss/health counters.

### 3. Follow one distributed request

Run scenario `a1`. Show browser/checkout/pricing/inventory/recommendation in one trace, then show correlated logs and metric windows. Next demonstrate `a3` or `a4` for producer/consumer and reverse-hop correlation.

### 4. Inspect an error as context

Run `a31` or the documented payment failure. Show issue grouping, occurrence history, linked trace, logs, metrics, evidence gaps, and the bounded `parallax issue context` bundle.

### 5. Compare propagation quality

Run `a28` for stitched versus intentionally broken browser/backend propagation. Explain what trace IDs, invocation IDs, baggage, span links, and orphan logs mean.

### 6. Give the context to an AI agent

Use the CLI bundle or local read-only MCP tools. Demonstrate that Parallax supplies context and boundaries; a separate agent proposes changes. Keep A1 fix-quality results explicitly marked unproven.

### 7. Show the honest competitor boundary

Use a small comparison table: Parallax offers local correlation and evidence bundles; Sentry is stronger in error lifecycle/ecosystem; Grafana is stronger in dashboards, alerting, scale, and telemetry ecosystem; Kibana/Elastic is stronger in search-grade logs and security operations.

## Playground scenario index

| Story | Scenario(s) | Demonstrates |
| --- | --- | --- |
| Basic waterfall | `a1` | HTTP → gRPC/HTTP fan-out and trace waterfall |
| Browser propagation | `a28` | Frontend/backend stitching and broken propagation |
| Async topology | `a3`, `a4`, `a8` | Span links, Kafka, Java → Rust reverse hop |
| GraphQL behavior | `a6` | DataLoader batching versus N+1 |
| Context propagation | `a10` | W3C baggage across services |
| Error investigation | `a31`, `c8`, `b23` | Application error, Sentry event, orphan log |
| Metrics correlation | `a2`, `a5` | RED metrics, exemplars, metric windows |

The playground README and `TOUR.md` remain the scenario source of truth. Screenshots in `artifacts/ui/` are illustrative until backed by repeatable live assertions.

## Public-readiness blockers

### High priority

1. **Unauthenticated Parallax ingest.** OTLP HTTP/gRPC routes bypass API bearer auth. A non-loopback bind can permit telemetry injection, resource exhaustion, and privacy leakage. Until auth/TLS exists, enforce loopback/trusted-network binding and label it prominently.
2. **Unlimited buffered telemetry bodies.** Playground `/v1/traces` and `/v1/logs` read request bodies without a strict bound. Add limits, timeouts, and rate limiting before any shared-network demo.
3. **Silent telemetry loss in the playground proxy.** The trace proxy can return `202` while dropping data when its upstream is unavailable. Return an explicit failure or provide bounded queueing plus visible loss metrics.

### Medium priority

- Playground compose exposes unauthenticated APIs and uses the default database password `playground`; bind local services to loopback and require generated credentials.
- Browser tracing, replay-on-error, feedback, and console capture need documented masking/consent defaults.
- Pin Java base images and verify downloaded agents with checksums.
- Document `protoc` as a prerequisite or install it through the prescribed toolchain; direct playground test commands currently fail when it is absent.
- Metrics support drops exponential histograms and summaries; expose loss clearly or implement them.
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
7. Docs distinguish shipped, partial, planned, and unproven capabilities.
8. A small competitor matrix has source-linked claims and no replacement overclaims.

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
