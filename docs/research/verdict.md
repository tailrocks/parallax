# Parallax Go / No-Go Verdict

<!-- markdownlint-disable MD013 -->

Research date: 2026-05-25

## Verdict

**GO.**

Build Parallax, but only as the narrow version:

> An open-source, Rust-first, self-hostable execution context engine that accepts
> fixture-gated Sentry envelope error events, OpenTelemetry telemetry, CLI
> invocation traces, and coding-agent session records from tested capture
> adapters, then stores and serves bounded evidence bundles for humans and
> agents.

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
| Are there direct competitors? | **Yes.** Sentry Seer and Datadog Bits AI SRE are direct for production debugging. Grafana Assistant is direct for observability-agent workflows. LangSmith/Langfuse/Phoenix/Braintrust/AgentOps-style systems are adjacent for agent/LLM execution telemetry. CI/autofix products are direct for test and pipeline failures. |
| Do competitors leave room? | **Yes, narrowly.** They mostly optimize inside their own observability or LLM-app platform. Parallax can win only if it is simpler to self-host, exposes an open evidence schema, gives CLI/HTTP access from day one and read-only MCP only after projection/safety gates, stores agent and CLI side effects, and produces portable bundles rather than product-bound answers. |
| Is this just a Sentry/Grafana/Datadog feature? | **Generic AI investigation is a feature.** A low-resource, Rust-first, self-hostable context store with fixture-gated Sentry envelope error-event migration, conformance-gated OTLP ingestion, adapter-backed CLI/agent audit records, and portable evidence bundles is a product wedge. |
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
- Sentry envelope error-event migration path with SDK fixture gates;
- OpenTelemetry-based correlation with OTLP conformance gates;
- first-class CLI invocation traces;
- coding-agent session records only where tested adapters preserve source,
  projection, and lossiness provenance;
- portable JSON/Markdown evidence bundles;
- read-only CLI/HTTP tools first, with MCP only after the access-surface gate.

If Parallax cannot win on those dimensions, it should not be built.

### 3. The Technical Substrate Exists

The architecture is plausible with current open-source components:

| Layer | Gate decision | Evidence |
| --- | --- | --- |
| Error compatibility | Support the Sentry envelope `event` path, not the whole Sentry product. | Current registry checks still show Rust `sentry`/`sentry-types` `0.48.2`, JS SDKs `10.53.1`, Go `v0.46.2`, and Python `2.60.0`; "Sentry-compatible" remains only a target until those SDK-generated fixtures pass parser, normalization, grouping, redaction, projection, and unsupported-item gates. |
| Telemetry standard | Use OpenTelemetry as the native telemetry protocol. | OTLP `1.10.0` is stable for traces, metrics, and logs, and gives shared `trace_id`, `span_id`, resource, and semantic-convention context. This proves the wire substrate, not agent readiness: public OTLP claims require the conformance ledger, canonical bundle/projection checks, and MCP structured-output validation. |
| Observability store | Start with GreptimeDB as the v0.1 prototype default, benchmark against exact ClickHouse stable/LTS tracks. | GreptimeDB targets metrics, logs, and traces in one observability engine, with native OpenTelemetry support and object-storage-oriented deployment. It reached **v1.0 GA in April 2026** and latest stable checked is `v1.0.2`, so the first build is no longer a bet on an unreleased database. It is still not a proven production winner: trace docs remain experimental, and the storage freshness, bundle-latency, object-cost, and operational-complexity gates keep veto power. |
| Stream | Start with local WAL; add Apache Iggy only when replay/burst separation matters. | Iggy is Rust-native, persistent, append-oriented, and explicitly designed for low-latency message streaming. |
| Metadata | Start with local Turso Database for prototype/tiny metadata; keep Postgres as an active production and scale-out fallback. | Latest non-prerelease checked is `v0.6.1`; `v0.7.0-pre.3` exists but is a prerelease. Turso is Rust-written and SQLite-compatible, but still beta/production-caution in the repository README, so crash, backup/restore, concurrency, migration, and fallback gates are mandatory before any production-default claim. |
| Agent surface | CLI and HTTP first; read-only MCP only after the access-surface safety gate. | Coding agents can call CLIs today, but MCP has become the standard tool discovery/invocation surface and has explicit auth/security requirements. Do not claim first-class agent-native access until MCP projects the same canonical bundle as CLI/API and passes read-only, redaction, output-budget, and audit fixtures. |

Sources:

- [Sentry envelopes](https://develop.sentry.dev/sdk/foundations/envelopes/)
- [Sentry Relay repository](https://github.com/getsentry/relay)
- [OpenTelemetry OTLP specification](https://opentelemetry.io/docs/specs/otlp/)
- [OpenTelemetry MCP semantic conventions](https://opentelemetry.io/docs/specs/semconv/gen-ai/mcp/)
- [GreptimeDB docs](https://docs.greptime.com/)
- [GreptimeDB v1.0.2 release](https://github.com/GreptimeTeam/greptimedb/releases/tag/v1.0.2)
- [GreptimeDB trace read/write docs](https://docs.greptime.com/user-guide/traces/read-write/)
- [Apache Iggy docs](https://iggy.apache.org/docs/)
- [Turso Database repository](https://github.com/tursodatabase/turso)
- [Turso Database v0.6.1 release](https://github.com/tursodatabase/turso/releases/tag/v0.6.1)
- [Turso Database v0.7.0-pre.3 release](https://github.com/tursodatabase/turso/releases/tag/v0.7.0-pre.3)
- [MCP specification 2025-11-25](https://modelcontextprotocol.io/specification/2025-11-25)
- [MCP authorization specification](https://modelcontextprotocol.io/specification/2025-11-25/basic/authorization)
- [MCP security best practices](https://modelcontextprotocol.io/docs/tutorials/security/security_best_practices)

## What Parallax Actually Solves

Parallax should solve these concrete jobs:

1. Preserve runtime evidence cheaply enough that teams do not fear diagnostic
   cost spikes.
2. Group errors deterministically before AI touches them.
3. Join Sentry-style errors with OTLP traces, logs, metrics, releases, deploys,
   CI runs, CLI invocations, and agent sessions.
4. Build an evidence graph with typed edge strengths, not a loose text blob.
5. Serve bounded context bundles through CLI and HTTP first, then MCP after
   projection-equivalence and safety gates pass.
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
| Sentry Seer | Production error AI debugging and PR generation are real workflows. | GA but closed-source, SaaS-only, and confirmed **not available to self-hosted Sentry** (2026-05). The dominant error tracker paywalls its AI away from exactly the self-hosting, data-ownership audience Parallax targets. This is the single clearest opening. |
| Sentry MCP | Coding-agent MCP access over Sentry data is now a first-party Sentry surface, including remote service, Claude Code plugin/subagent path, and stdio transport for self-hosted Sentry. | The current checked release is `sentry-mcp` `0.35.0`; its README calls stdio a work-in-progress path, the documented self-hosted token scopes include write scopes, AI-powered search tools require OpenAI or Anthropic provider configuration, and self-hosted instances may need unsupported Seer skills disabled. This is not hosted Seer parity and makes MCP table stakes, not a moat. |
| Datadog Bits AI SRE / Dev Agent | Hypothesis-driven investigations and flaky-test autofix are the enterprise direction. | Closed, expensive, SaaS-only, and tied to Datadog data gravity. Dev Agent (flaky-test autofix) is still public Preview. Not an open, self-hosted Rust context engine or portable evidence-bundle standard. |
| Grafana Assistant | Agent access through CLI/API/MCP surfaces is now normal. | Now on-prem and free for OSS Grafana (Apr 2026) but **still requires a Grafana Cloud account for the LLM connection** — not air-gapped — and is dashboard/assistant-first, not portable evidence bundles. LGTM-shaped, not evidence-engine-shaped. |
| OpenObserve "Observability 3.0" (late Apr 2026) | An open, Rust, single-binary, object-storage observability store *with* an AI SRE agent + MCP is now real and self-hostable. | The closest thing to a wedge-killer on storage/runtime fit, saved by three current gaps: AI SRE/MCP require Enterprise edition/license while public pages conflict on the free Self-Hosted Enterprise allowance, the MCP surface is broad and write-capable rather than a bounded read-only evidence bundle, and checked ingestion docs show OTLP rather than a Sentry-envelope path. |
| SigNoz agent-native (May 2026) | Open, self-hostable MCP server + trace-ID RCA shipping in OSS validates the agent-native direction loudly. | Go + ClickHouse (fails the runtime filter and carries the heavy store Parallax escapes), a query/management interface rather than a checked deterministic evidence graph / portable bundle, an unproven landing-page claim around an "open investigation format," and **no Sentry envelope error-event ingest path**. |
| Dynatrace / New Relic / Splunk | Topology-aware RCA is enterprise table stakes. | Enterprise suite gravity, not open small-team self-hosting or agent-readable bundle portability. |
| LangSmith / Langfuse / Phoenix / Braintrust / AgentOps / similar | Agent and LLM traces are important. | They usually observe LLM app execution, not the full chain from production error to deploy, CLI side effect, coding-agent patch, CI validation, and outcome. |
| CI autofix and flaky-test tools | Failure bundles and PR automation are valuable. | They usually start at CI/test evidence, not production Sentry/OTLP context plus runtime evidence graph. |

## Competitive Window (2026-05 update)

This is the finding that moves the posture from "comfortable GO" to "GO, move
now." Between the earlier market pass and 2026-05-25, agent-native observability
went from emerging to table stakes, and two open, non-incumbent projects moved
toward Parallax's exact space: OpenObserve shipped an AI SRE agent + MCP on a
Rust, object-storage, AGPL-self-hostable base, and SigNoz shipped an open,
self-hostable agent-native MCP server.
Sentry's first-party MCP server adds pressure from the incumbent side too: even
self-hosted Sentry users can expose Sentry data to coding agents through a
work-in-progress stdio path, although the checked tool/scopes/provider shape is
not Parallax's bounded read-only evidence-bundle contract.

Neither closes the wedge today — OpenObserve's AI SRE/MCP surfaces require
Enterprise edition/license, its source-conflicted Self-Hosted Enterprise
allowance weakens a simple paywall claim, and checked docs still show no Sentry
ingest; SigNoz is Go/ClickHouse with no Sentry ingest and no checked
evidence-graph/bundle abstraction behind its "open investigation format" claim.
But both could close their gap inside 6–12 months. The consequence: **the moat
cannot be any single feature.** It must be the assets that compound with usage
and are hard to copy from a standing start —

1. the failure/fixer-outcome corpus, if outcome rows prove more than PR
   creation;
2. the open evidence schema and portable bundle format as a standard others
   build on;
3. runtime-plus-repo-intent linkage;
4. Rust-first capture quality.

The strategic instruction that follows: ship the narrow tiny tier fast, get the
schema and bundle format adopted, and start accumulating the corpus before the
category fully commoditizes. If an open competitor ships the full combination
(open + self-hosted + Rust-light + Sentry-compatible + evidence bundles) before
Parallax has adoption and a corpus, revisit this verdict — that is the live path
to NO-GO.

Current source checks for this competitive-window claim:

- [OpenObserve pricing](https://openobserve.ai/pricing/)
- [OpenObserve homepage](https://openobserve.ai/)
- [OpenObserve SRE Agent setup](https://openobserve.ai/docs/administration/deployment/sre-agent-setup-guide/)
- [OpenObserve MCP docs](https://openobserve.ai/docs/integration/ai/mcp/)
- [OpenObserve OTLP ingestion docs](https://openobserve.ai/docs/ingestion/logs/otlp/)
- [Self-hosted Sentry docs](https://develop.sentry.dev/self-hosted/)
- [Sentry MCP service](https://mcp.sentry.dev/)
- [Sentry MCP repository](https://github.com/getsentry/sentry-mcp)
- [Sentry MCP 0.35.0 release](https://github.com/getsentry/sentry-mcp/releases/tag/0.35.0)
- [SigNoz agent-native observability](https://signoz.io/agent-native-observability/)
- [SigNoz MCP server](https://signoz.io/docs/ai/signoz-mcp-server/)

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
- Sentry envelope error-event migration after SDK fixture gates pass;
- OTLP-backed correlation only after conformance and projection gates pass;
- Rust-first capture and stacktrace quality;
- CLI and adapter-proven agent-session observability;
- safe CLI/HTTP context retrieval first, with MCP only after the access-surface
  gate;
- fixer outcome feedback loop after review, merge/revert, and recurrence rows
  exist.

## Phase 2 Gate

Because the verdict is **GO**, proceed to the implementation blueprint.

The blueprint must keep the boundary strict:

```text
Parallax stores and serves evidence
  -> CLI / HTTP API expose bounded context first
  -> read-only MCP projects the same context after safety gates
  -> separate fixer component pulls Parallax + repository context
  -> coding agent proposes or opens a PR
  -> fixer writes outcome rows back; PR creation is not proof of fix
```

Parallax itself must not become the fixer. It is the context engine.

## Kill Criteria

Reverse this GO if prototype evidence shows any of the following:

1. Sentry envelope event ingestion cannot work without recreating Relay,
   Kafka, Snuba, and the operational burden Parallax exists to avoid.
2. GreptimeDB or the fallback store cannot answer evidence-bundle queries with
   acceptable freshness, latency, and storage cost.
3. Agent bundles do not improve diagnosis or patch quality over raw Sentry/CI
   context in controlled tests. The experiment that decides this is designed in
   [Bundle-value evaluation](bundle-value-evaluation.md) — note its raw-telemetry-dump
   control: the bundle must beat a raw dump, not just repo-only context.
4. CLI and agent-session capture produces too much sensitive data to redact
   safely, or tested adapters cannot preserve source/projection/lossiness
   provenance.
5. MCP/API access cannot be made least-privilege, auditable, and read-only by
   default.
6. The first deployment fails the
   [self-hosted simplicity gate](self-hosted-simplicity-gate.md), cannot pass
   the [self-hosted simplicity ledger](self-hosted-simplicity-ledger.md), and is
   not meaningfully simpler than self-hosted Sentry.

Until those kill criteria trigger, the correct decision is **GO**.

For the maintained adversarial counterweight to this verdict — the steelmanned
NO-GO case, the load-bearing-assumption register, and a full risk matrix — see
[Risks and the bear case](risks-and-bear-case.md). The bear case argues the real
danger is distribution and monetization, not feasibility, and names the market
assumptions (bundle value, real users, schema adoption) to validate before the
comfortable engineering work.
