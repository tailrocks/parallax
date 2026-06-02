# Parallax Research

This directory is the research record behind Parallax. It is organized so a reader can reach
**what Parallax is, which storage engine, and why** in a few minutes, then drill into evidence.

> **Parallax is an open-source, Rust-first, self-hosted execution-context engine.** It ingests
> OpenTelemetry traces/logs/metrics plus CLI and coding-agent execution traces, derives
> Parallax-owned error events from exception spans and ERROR/FATAL logs, groups errors
> deterministically, correlates signals into a typed evidence graph, and serves **bounded,
> redacted, schema-valid evidence bundles** to humans and coding agents over a CLI/HTTP API first,
> and a read-only MCP adapter after safety gates. Parallax is the **context engine, not the fixer**
> — a separate coding agent consumes the bundle and proposes the fix.

## Current answers (the short version)

| Question | Answer | Where |
| --- | --- | --- |
| Is it worth building? | **GO**, for the *narrow* evidence/context engine (not a generic RCA chatbot or autonomous SRE). | [decisions/go-no-go.md](decisions/go-no-go.md) |
| Which storage engine? | **Current lean: GreptimeDB** (cost + Rust + self-hosted), **not yet settled**. Both engines stay behind one `StorageAdapter`; ClickHouse is the fallback and wins raw analytical speed. | [decisions/storage-engine.md](decisions/storage-engine.md) |
| What is the V1 storage design? | **Local-first, adapter-extensible.** V1 local runs use managed GreptimeDB standalone for observability evidence plus Turso/SQLite-like metadata; GreptimeDB remains the production/server profile; bundle/API contract stays backend-neutral. | [decisions/v1-storage-adapter-vision.md](decisions/v1-storage-adapter-vision.md), [architecture/local-first-v1.md](architecture/local-first-v1.md) |
| Why GreptimeDB lean? | Hot path is *anchored* evidence-bundle retrieval (all signals for one `trace_id`/`fingerprint`) — both engines are interactive (≪300 ms) there, so ClickHouse's scan-speed lead is off the hot path; the decision turns on cost + Rust, where GreptimeDB leads. | [decisions/storage-engine.md](decisions/storage-engine.md) |
| What's still open before the engine is settled? | Sized $/GB cost on a server tier, cold-read latency from object storage, the self-host-vs-managed-cloud call, and a re-test on GreptimeDB v1.1 GA. | [decisions/storage-engine.md](decisions/storage-engine.md) |
| How is it built? | Three deployment tiers, one event/bundle contract; ingest → normalize → group → correlate → evidence-graph → CLI/HTTP/MCP. | [architecture/implementation-concept.md](architecture/implementation-concept.md) |
| What still needs research? | Ranked, cheapest-to-kill-first: A1 (bundle beats raw) and monetization are the two gates the GO rests on. | [research-agenda.md](research-agenda.md) |

## Map

### `00-vision/` — why this product, in plain terms
- [thesis.md](00-vision/thesis.md) — the original thesis.
- [world-before-parallax.md](00-vision/world-before-parallax.md) — the pre-Parallax stack (Sentry + traces/logs/metrics/UI), why it exists, and what Parallax tries to collapse.
- [platform-direction.md](00-vision/platform-direction.md) — the platform/intelligence-layer outcome as an earned, gated emergence from the narrow wedge.
- [ai-native-observability.md](00-vision/ai-native-observability.md) — AI-native observability, incident-intelligence, and product-wedge synthesis.

### `decisions/` — current truth, one decision per file (ADR-style, conclusion first)
- [go-no-go.md](decisions/go-no-go.md) — the GO / NO-GO gate for whether Parallax is worth building.
- [strategic-coverage.md](decisions/strategic-coverage.md) — strategic verdict, prompt-coverage map, key decisions, open proof gates.
- [risks-and-bear-case.md](decisions/risks-and-bear-case.md) — steelmanned NO-GO case, load-bearing assumptions, NO-GO/strengthen triggers.
- [skeptical-reassessment-2026-05.md](decisions/skeptical-reassessment-2026-05.md) — dated whole-concept stress-test: what still makes sense, what must be built, what benefit actually competes (A1 elevated to #1; monetization structural).
- [storage-engine.md](decisions/storage-engine.md) — GreptimeDB vs ClickHouse: the one-page current verdict (full record in [storage/greptimedb-vs-clickhouse/](storage/greptimedb-vs-clickhouse/)).
- [v1-storage-adapter-vision.md](decisions/v1-storage-adapter-vision.md) — V1 storage implementation stance: managed local GreptimeDB + Turso metadata, GreptimeDB production profile, backend-neutral adapter contract.
- [stack-decision.md](decisions/stack-decision.md) — A5 stack-decision: rolls storage/metadata/ingest/setup gates into stack claim levels and fallback triggers.
- [metadata-store.md](decisions/metadata-store.md) — relational metadata store: Turso-first, Postgres fallback (evidence in [storage/metadata/](storage/metadata/)).
- [agent-access-surface.md](decisions/agent-access-surface.md) — canonical HTTP API, day-one CLI, read-only MCP after safety gates.
- [fixer-boundary.md](decisions/fixer-boundary.md) — the separate fixer component, outcome loop, and why PR creation is commodity while outcome feedback is the moat.

### `architecture/` — how the pieces fit
- [implementation-concept.md](architecture/implementation-concept.md) — opinionated end-to-end blueprint with named component choices, deployment profiles, data flow, rejected alternatives.
- [overview.md](architecture/overview.md) — the OpenTelemetry-native self-hosted architecture, with Sentry compatibility as a future adapter.
- [evidence-bundle-schema.md](architecture/evidence-bundle-schema.md) — the `v0` portable evidence-bundle and open schema (the named moat artifact).
- [api-concept.md](architecture/api-concept.md) — GraphQL-first query/exploration API, OTLP-first ingest, future Sentry adapter, and strict API boundary.
- [causal-reconstruction.md](architecture/causal-reconstruction.md) — evidence-graph, causal reconstruction, and agent-safety analysis.
- [agent-trust-boundary-and-prompt-injection.md](architecture/agent-trust-boundary-and-prompt-injection.md) — prompt injection via attacker-controlled telemetry (inject-*in*, vs A6 redaction's leak-*out*): the threat and the trust-boundary design constraints it forces.
- [agent-context-integration.md](architecture/agent-context-integration.md) — how real coding agents ingest context (MCP structuredContent + token-budget caps → bounded bundle) and how to link to repo intent (reference, don't invent; the unsolved evidence→intent edge).
- [local-first-v1.md](architecture/local-first-v1.md) — one-command local `run_id` evidence server for agent-assisted development, with managed GreptimeDB evidence and Turso metadata.
- [simple-ui-v2.md](architecture/simple-ui-v2.md) — V2 local investigation UI using TanStack Start + shadcn/ui, covering Sentry-style issues plus Grafana/Kibana/Tempo/Prometheus-style inspection.
- [build-roadmap.md](architecture/build-roadmap.md) — de-risking build sequence with go/no-go gates tied to bear-case assumptions.

### `capture/` — how each signal is collected and made safe
- [rust.md](capture/rust.md) — Rust data collection, capture fidelity, and stacktrace grouping/symbolication.
- [frontend.md](capture/frontend.md) — browser collection, cross-tier correlation, source maps, and the frontend privacy problem.
- [sentry-ingest.md](capture/sentry-ingest.md) — future Sentry envelope/Relay/grouping ingest, envelope-item policy, and SDK fixture compatibility.
- [otlp.md](capture/otlp.md) — OpenTelemetry protocol/Collector context layer, transport profile, and receiver conformance.
- [agent-cli-tracing.md](capture/agent-cli-tracing.md) — coding-agent and CLI execution tracing, OTel semconv mapping, and trace overhead/redaction.
- [deploy-change-context.md](capture/deploy-change-context.md) — release/deploy/code-change/work-item evidence ("what changed?").
- [ci-and-flaky-tests.md](capture/ci-and-flaky-tests.md) — CI failure-context bundles and flaky-test detection/replay.
- [production-db-evidence.md](capture/production-db-evidence.md) — safety gate for treating production databases as evidence sources.
- [correlation.md](capture/correlation.md) — A4: correlation reliability on real telemetry.
- [redaction.md](capture/redaction.md) — A6: redaction pipeline, detector toolchain, canary corpus, and red-team gate.

### `storage/` — the telemetry store and its evidence
- [evaluation.md](storage/evaluation.md) — storage-layer evaluation across GreptimeDB, ClickHouse, and observability backends.
- [benchmark-plan.md](storage/benchmark-plan.md) — the database benchmark plan, runnable prototype spec, and artifact interpretation.
- [freshness-and-latency.md](storage/freshness-and-latency.md) — ingest-to-queryable freshness and evidence-bundle latency gate.
- [size-and-object-cost.md](storage/size-and-object-cost.md) — retained size, per-signal compression, object-store cost, and the retention cost model.
- [metadata/](storage/metadata/) — Turso-first metadata benchmark plan and production-readiness evidence.
- [streaming/](storage/streaming/) — stream/ingest-layer evaluation and the ingest-log replay/backpressure gate.
- [greptimedb-vs-clickhouse/](storage/greptimedb-vs-clickhouse/) — the deep white-box engine sub-study (verdict + 30+ mechanism notes + benchmarks).

### `validation/` — the A1–A7 assumption gates and their ledgers
- [a1-bundle-value/](validation/a1-bundle-value/) — A1: does a Parallax bundle beat raw context for agent fix quality? (eval design, seed corpus, Phase-0 runbook, ledgers, AgentRx trajectory-IR source check).
- [a2-user-demand.md](validation/a2-user-demand.md) — A2: user-interview and deployment-intent gate + evidence ledger.
- [a3-schema-corpus.md](validation/a3-schema-corpus.md) — A3: schema-adoption and corpus-moat gate + ledger.
- [a7-scope.md](validation/a7-scope.md) — A7: scope-discipline ledger keeping the tiny tier buildable.
- [self-hosted-simplicity.md](validation/self-hosted-simplicity.md) — operational proof that the tiny tier is simpler than self-hosted Sentry (gate + baseline inventory + ledger).
- [business-model.md](validation/business-model.md) — business-model/economics analysis + validation ledger.
- [monetization-and-paying-segment.md](validation/monetization-and-paying-segment.md) — the paying buyer (hard-boundary/air-gap/sovereign self-hoster) sized, and the monetization shape (Apache-2.0 open core + gated enterprise-ops + managed cloud + outcome-priced fixer).
- [repo-intent.md](validation/repo-intent.md) — how much Parallax depends on a context-rich repo (runtime floor vs intent multiplier) + ledger.

> A4 lives in [capture/correlation.md](capture/correlation.md), A5 in [decisions/stack-decision.md](decisions/stack-decision.md), A6 in [capture/redaction.md](capture/redaction.md).

### `market/` — landscape and competitor watch
- [landscape.md](market/landscape.md) — market research.
- [competitor-watch.md](market/competitor-watch.md) — consolidated watch: OpenObserve, SigNoz, Coroot, Bugsink, Rustrak, Traceway, GoSnag, Urgentry, Sentry/Seer/MCP, the MCP-power boundary, and the drift ledger.
- [alternatives-deep-analysis.md](market/alternatives-deep-analysis.md) — balanced skeptical analysis: 60+ alternatives surveyed, arguments FOR and AGAINST Parallax, empirical gates, and kill criteria.
- [competitive-comparison-matrix.md](market/competitive-comparison-matrix.md) — quick-reference matrix of all competitors against Parallax's 8 wedge dimensions.
- [agent-debugging-competitor-drift-2026-06-02.md](market/agent-debugging-competitor-drift-2026-06-02.md) — focused recheck of Syncause, AgentRx, Notrix Trax, AgentReplay, and OpenTelemetry MCP/replay/crash semantic-convention drift.

### `reference/`
- [agent-observability-review.md](reference/agent-observability-review.md) — technical review of current agent-observability tools and the Parallax-specific gap.
- [grafana-tempo-v3-architecture-review.md](reference/grafana-tempo-v3-architecture-review.md) — Tempo v3.0.0 (Kafka-log write path, vParquet5, TraceQL Metrics GA, retroactive redaction) reviewed as an architectural reference: three borrows, not a wedge competitor.

## Conventions

- **Decisions** lead with the current answer; long re-verification history lives in clearly-labeled
  evidence/changelog sections, not the front door.
- Storage engine choice stays **behind a `StorageAdapter`** — no engine magic in the schema or bundle
  contract.
- Treat every note as a theory until current primary-source evidence supports it; mark
  benchmark-dependent claims as unproven until measured.
