# Parallax Go / No-Go Verdict

<!-- markdownlint-disable MD013 -->

Research date: 2026-05-25

## Verdict

**GO.**

Build Parallax, but only as the narrow version:

> An open-source, Rust-first, self-hostable execution context engine that accepts
> Sentry-compatible errors, OpenTelemetry telemetry, CLI invocation traces, and
> coding-agent session traces, then stores and serves bounded evidence bundles
> for humans and agents.

Do not build the broad version:

> A generic AI observability dashboard, AI root-cause chatbot, or autonomous SRE
> agent over every production signal.

That broad version is already a feature direction for Sentry, Datadog, Grafana,
New Relic, Dynatrace, Splunk, and other observability platforms. The buildable
Parallax product is the open, self-hosted evidence layer underneath agentic
debugging.

## Gate Answers

| Question | Verdict |
| --- | --- |
| Is the problem real? | **Yes.** The problem is not "no one has dashboards." The problem is that production debugging, CI debugging, CLI execution, and coding-agent work produce fragmented evidence that humans and agents must manually reconstruct. Public product direction from Datadog Bits AI SRE, Sentry Seer, Grafana Assistant, and others validates this pain. |
| Does Parallax solve it? | **Partially, and that is enough.** Parallax can solve context assembly, evidence retention, correlation, issue grouping, and agent-safe bundle generation. It cannot prove all root causes from telemetry alone, and it should never claim omniscient RCA. |
| Are there direct competitors? | **Yes.** Sentry Seer and Datadog Bits AI SRE are direct for production debugging. Grafana Assistant is direct for observability-agent workflows. LangSmith/Langfuse/Phoenix/Braintrust/AgentOps-style systems are adjacent for agent tracing. CI/autofix products are direct for test and pipeline failures. |
| Do competitors leave room? | **Yes, narrowly.** They mostly optimize inside their own observability or LLM-app platform. Parallax can win only if it is simpler to self-host, exposes an open evidence schema, gives CLI/MCP/API access from day one, stores agent and CLI side effects, and produces portable bundles rather than product-bound answers. |
| Is this just a Sentry/Grafana/Datadog feature? | **Generic AI investigation is a feature.** A low-resource, Rust-first, self-hostable context store with Sentry-compatible migration, OTLP-native ingestion, CLI/agent audit traces, and portable evidence bundles is a product wedge. |
| Does the market make sense? | **Yes, with discipline.** AI is making software faster to write and riskier to operate without audit trails. The market is crowded, but the crowding validates the shift from dashboards to evidence-backed investigation. The opportunity is not "better AI"; it is owning the evidence contract agents use. |

## Why This Is A GO

### 1. The Pain Is Already Market-Validated

Datadog documents Bits AI SRE as an investigation loop that forms hypotheses,
queries telemetry, validates evidence, and returns either an evidence-backed
conclusion or an inconclusive result. It uses metrics, APM traces, logs, events,
change tracking, GitHub source code, Watchdog, RUM, network, database, profiler,
and preview third-party integrations.

Sentry documents Seer as an AI debugging agent using issue details, tracing,
logs, profiles, and code context. The Seer Issue Fix API can stop at root cause,
solution, code changes, or opening a pull request.

Grafana Assistant exposes observability workflows through UI, CLI, API, Slack,
Teams, and MCP-related integrations. Its CLI can query telemetry, run
investigations, and connect local projects with a tunnel.

These are not weak signals. The incumbents are building exactly because the
manual debugging loop is painful.

Sources:

- [Datadog Bits AI SRE investigation docs](https://docs.datadoghq.com/bits_ai/bits_ai_sre/investigate_issues/)
- [Sentry Seer docs](https://docs.sentry.io/product/ai-in-sentry/seer/)
- [Sentry Seer Issue Fix API](https://docs.sentry.io/api/seer/start-seer-issue-fix/)
- [Grafana Assistant CLI docs](https://grafana.com/docs/grafana-cloud/machine-learning/assistant/guides/cli/)
- [Grafana Assistant MCP servers docs](https://grafana.com/docs/grafana-cloud/machine-learning/assistant/mcp/)

### 2. The Existing Products Also Prove The Trap

The trap is building "AI root cause analysis" as a headline. That is no longer
a differentiated product.

The incumbent pattern is:

```text
telemetry + topology + changes + source context
  -> hypothesis loop
  -> evidence-backed conclusion or inconclusive result
  -> action, ticket, PR, or recommendation
```

That pattern is now table stakes. Parallax should not compete with the broad
suite. It should compete on the evidence substrate:

- open schema;
- self-hosted and low-resource operation;
- Rust-first capture quality;
- Sentry-compatible migration path;
- OpenTelemetry-native correlation;
- first-class CLI invocation traces;
- first-class coding-agent session traces;
- portable JSON/Markdown evidence bundles;
- read-only CLI/MCP/API tools with tight scope.

If Parallax cannot win on those dimensions, it should not be built.

### 3. The Technical Substrate Exists

The architecture is plausible with current open-source components:

| Layer | Gate decision | Evidence |
| --- | --- | --- |
| Error compatibility | Support the Sentry envelope event path, not the whole Sentry product. | Sentry envelopes are the modern SDK ingestion format, and Relay is a useful Rust reference without copying its Kafka/Snuba architecture. |
| Telemetry standard | Use OpenTelemetry as the native telemetry protocol. | OTLP is stable for traces, metrics, and logs, and gives shared `trace_id`, `span_id`, resource, and semantic-convention context. |
| Observability store | Start with GreptimeDB, benchmark against ClickHouse. | GreptimeDB publicly targets metrics, logs, and traces in one observability engine, with native OpenTelemetry support and object-storage-oriented deployment. |
| Stream | Start with local WAL; add Apache Iggy only when replay/burst separation matters. | Iggy is Rust-native, persistent, append-oriented, and explicitly designed for low-latency message streaming. |
| Metadata | Start with Turso, keep Postgres as fallback. | Turso is Rust-written and SQLite-compatible, but still beta, so benchmark and backup gates are mandatory. |
| Agent surface | CLI first, MCP required for first-class agent UX, HTTP API underneath. | Coding agents can call CLIs today, but MCP has become the standard tool discovery/invocation surface and has explicit auth/security requirements. |

Sources:

- [Sentry envelopes](https://develop.sentry.dev/sdk/foundations/envelopes/)
- [Sentry Relay repository](https://github.com/getsentry/relay)
- [OpenTelemetry OTLP specification](https://opentelemetry.io/docs/specs/otlp/)
- [OpenTelemetry MCP semantic conventions](https://opentelemetry.io/docs/specs/semconv/gen-ai/mcp/)
- [GreptimeDB docs](https://docs.greptime.com/)
- [Apache Iggy docs](https://iggy.apache.org/docs/)
- [Turso Database repository](https://github.com/tursodatabase/turso)
- [MCP authorization specification](https://modelcontextprotocol.io/specification/2025-06-18/basic/authorization)
- [MCP security best practices](https://modelcontextprotocol.io/docs/tutorials/security/security_best_practices)

## What Parallax Actually Solves

Parallax should solve these concrete jobs:

1. Preserve runtime evidence cheaply enough that teams do not fear diagnostic
   cost spikes.
2. Group errors deterministically before AI touches them.
3. Join Sentry-style errors with OTLP traces, logs, metrics, releases, deploys,
   CI runs, CLI invocations, and agent sessions.
4. Build an evidence graph with typed edge strengths, not a loose text blob.
5. Serve bounded context bundles through CLI, HTTP API, and MCP.
6. Record what an agent saw, queried, changed, tested, proposed, and shipped.
7. Say "inconclusive" when evidence is missing instead of inventing certainty.

This is a real product surface. It is also much smaller than an observability
suite.

## What Parallax Does Not Solve

Parallax does not solve:

- every production root cause;
- missing instrumentation;
- sampled-away spans;
- unstructured logs with no trace context;
- cross-service causality without topology or span links;
- business-rule failures not represented in telemetry;
- safe autonomous production mutation;
- trust in a generated patch without tests, evidence, and human-review policy.

The right claim is:

> Parallax reconstructs the best available evidence-backed lifecycle and ranks
> hypotheses. It does not prove every root cause.

That honesty is a strength, not a limitation.

## Direct Competitor Read

| Competitor | What they prove | Where they fall short for Parallax's goal |
| --- | --- | --- |
| Sentry Seer | Production error AI debugging and PR generation are real workflows. | Platform-bound to Sentry context; not positioned as a small open evidence store for Sentry-compatible migration plus OTLP plus CLI/agent audit. |
| Datadog Bits AI SRE | Hypothesis-driven, evidence-backed investigations are the enterprise direction. | Datadog data gravity is strong, but the product is not an open, self-hosted Rust context engine or portable evidence-bundle standard. |
| Grafana Assistant | Agent access through CLI/API/MCP-like surfaces is becoming normal. | Grafana's center is the LGTM/Grafana ecosystem, not Sentry-compatible issue grouping or coding-agent action audit. |
| Dynatrace / New Relic / Splunk | Topology-aware RCA is enterprise table stakes. | Enterprise suite gravity, not open small-team self-hosting or agent-readable bundle portability. |
| LangSmith / Langfuse / Phoenix / Braintrust / AgentOps / similar | Agent and LLM traces are important. | They usually observe LLM app execution, not the full chain from production error to deploy, CLI side effect, coding-agent patch, CI validation, and outcome. |
| CI autofix and flaky-test tools | Failure bundles and PR automation are valuable. | They usually start at CI/test evidence, not production Sentry/OTLP context plus runtime evidence graph. |

## Market Verdict

The market is crowded, but not closed.

It is closed for:

- generic AI RCA;
- generic dashboard assistant;
- "Sentry plus AI";
- "Datadog but open source";
- LLM log summarization;
- flaky-test detection alone.

It is open enough for:

- open-source evidence bundle format;
- low-resource self-hosted deployment;
- Sentry-compatible error migration;
- OTLP-native correlation;
- Rust-first capture and stacktrace quality;
- CLI and agent-session observability;
- safe MCP/API/CLI context retrieval;
- accepted-fix feedback loop.

## Phase 2 Gate

Because the verdict is **GO**, proceed to the implementation blueprint.

The blueprint must keep the boundary strict:

```text
Parallax stores and serves evidence
  -> CLI / HTTP API / MCP expose bounded context
  -> separate fixer component pulls Parallax + repository context
  -> coding agent proposes or opens a PR
```

Parallax itself must not become the fixer. It is the context engine.

## Kill Criteria

Reverse this GO if prototype evidence shows any of the following:

1. Sentry-compatible event ingestion cannot work without recreating Relay,
   Kafka, Snuba, and the operational burden Parallax exists to avoid.
2. GreptimeDB or the fallback store cannot answer evidence-bundle queries with
   acceptable freshness, latency, and storage cost.
3. Agent bundles do not improve diagnosis or patch quality over raw Sentry/CI
   context in controlled tests.
4. CLI and agent-session tracing produces too much sensitive data to redact
   safely.
5. MCP/API access cannot be made least-privilege, auditable, and read-only by
   default.
6. The first deployment is not meaningfully simpler than self-hosted Sentry.

Until those kill criteria trigger, the correct decision is **GO**.
