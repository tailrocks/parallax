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
| Which storage engine? | **GreptimeDB only (decided 2026-06-18).** V1 adopts GreptimeDB's **native OTLP tables**; ClickHouse is deferred — not a V1 fallback or design constraint (revisit only on a concrete benefit). The engine comparison is retained as historical evidence. | [decisions/native-otel-tables.md](decisions/native-otel-tables.md), [decisions/storage-engine.md](decisions/storage-engine.md) |
| What is the V1 storage design? | **Native-first on GreptimeDB.** The proxy forwards raw OTLP straight to GreptimeDB's native tables and tees in-process to derive issues into a few custom extension tables (`error_events`, `rollups_fingerprint_minute`, `run_metric_points`); Turso holds metadata. Greenfield, no migration. | [decisions/native-otel-tables.md](decisions/native-otel-tables.md), [storage/native-otel-migration-plan.md](storage/native-otel-migration-plan.md) |
| Why GreptimeDB? | Anchored evidence-bundle retrieval is interactive on it (≪300 ms); the team optimizes around the native OTLP model and it's the Rust, self-hosted substrate Parallax can build on. | [decisions/storage-engine.md](decisions/storage-engine.md) |
| What's still open on storage? | Vendor confirmations for the native-table customizations (custom columns/indexes vs schema auto-widening, traces OTLP GA, etc.). | [storage/greptimedb-team-questions.md](storage/greptimedb-team-questions.md) |
| How is it built? | Three deployment tiers, one event/bundle contract; ingest → normalize → group → correlate → evidence-graph → CLI/HTTP/MCP. | [architecture/implementation-concept.md](architecture/implementation-concept.md) |
| What still needs research? | Ranked, cheapest-to-kill-first: A1 (bundle beats raw) and monetization are the two gates the GO rests on. | [research-agenda.md](research-agenda.md) |

## Map

### `00-vision/` — why this product, in plain terms
- [problem-audience-product-shape.md](00-vision/problem-audience-product-shape.md) — the front door: what problem Parallax solves, who it is for (audience ladder, local-dev first), and the product shape (best of three worlds — OTel/Sentry/Grafana concepts, agent-first; CLI + API + UI as clients of one canonical API, kubectl-style remote CLI).
- [north-star-autonomous-fix-loop.md](00-vision/north-star-autonomous-fix-loop.md) — the named moonshot (operator, 2026-06-11): the autonomous fix loop, earned autonomy via outcome-fed budgets, and the impossible triangle (performance + cost + complete evidence).
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
- [native-otel-tables.md](decisions/native-otel-tables.md) — adopt GreptimeDB native OTLP tables (traces/logs/metrics), customize by `ALTER`, keep derived signals custom; what blocks native today + the path-A/B write fork; portability at the adapter API, not the physical table.
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
- [autonomous-fix-loop.md](architecture/autonomous-fix-loop.md) — the closed loop as one concept: Detect (trigger taxonomy) → Context → Dispatch (wake contract + autonomy budget) → Fix → Validate (the Reconciler: CI/merge/deploy/recurrence linkage) → Learn (outcomes adjust evidence selection and budgets), plus the cost architecture (tiering, pre-aggregation, evidence pinning).
- [integration-contract.md](architecture/integration-contract.md) — how other apps attach: no proprietary SDK, required OTel resource attributes, both exception encodings, deploy events, CI step, browser path, read-only agent surfaces, append-only fixer write-back.
- [poc-evidence-loop-coverage.md](architecture/poc-evidence-loop-coverage.md) — claim-discipline map for `poc/evidence-loop`: each of the 20 executable kernels → its design doc → the governing gate that still reads `not_measured`, what the PoC deliberately does not prove, and the graduation path. Rule: kernels move understanding; dated result rows move claim levels.
- [v1-build-plan.md](architecture/v1-build-plan.md) — the finalized buildable projection (operator statement #5): goals 1+2 (local visibility slightly first, then the server profile), workspace/crate layout, milestones M0–M6 with dogfood exit criteria, fixer explicitly parked at the schema level.
- [deployment-architecture-map.md](architecture/deployment-architecture-map.md) — the three-angle architecture map (local laptop / own server / cloud + object storage): per-angle diagrams, setup flows, and exactly what GreptimeDB (telemetry evidence engine; managed child → local SSD → object-storage backend) vs Turso/Postgres (mutable product state: issues, runs, tokens, policies) holds in each.
- [v1-scope.md](architecture/v1-scope.md) — the V1 definition (operator statement #6, revised for #7): the self-sufficient local machine, exhaustively — install/engine auto-download, ingest, processing, the `parallax run start -- <cmd>` wrapper centerpiece, the web UI, the complete CLI list, retention defaults, docs, the out-of-scope table, the build checklist, stack acceptance scenarios, and V1 risks. V1 done = the operator's daily development runs through it.
- [v1-implementation-spec.md](architecture/v1-implementation-spec.md) — the concrete contracts that make V1 implementable by an agent: workspace conventions, pinned dependency set, the :4000 port-collision fix (managed GreptimeDB child on 24000–24003), `config.toml` keys, GreptimeDB + Turso DDL, OTLP→column mapping, the GraphQL SDL, UI page→query map, CLI output contract, and the engine supervision contract. Contract changes land here first, then in code. Runner: [prompts/v1-implementation.md](../../prompts/v1-implementation.md).
- [local-first-v1.md](architecture/local-first-v1.md) — one-command local `run_id` evidence server for agent-assisted development, with managed GreptimeDB evidence and Turso metadata.
- [simple-ui-v2.md](architecture/simple-ui-v2.md) — the **V1 UI specification** (statement #7 pulled the UI into V1; filename historical): Sentry-grade issues list/detail, predefined + user-defined dashboards, trace lookup by trace_id/run_id, full chart→window→event→trace interactivity; TanStack Start + shadcn/ui on Base UI, default theme as-is, shadcn charts/blocks reused wholesale; no auth in V1.
- [build-roadmap.md](architecture/build-roadmap.md) — de-risking build sequence with go/no-go gates tied to bear-case assumptions.

### `capture/` — how each signal is collected and made safe
- [rust.md](capture/rust.md) — Rust data collection, capture fidelity, and stacktrace grouping/symbolication.
- [rust-stack-instrumentation.md](capture/rust-stack-instrumentation.md) — the verified emission matrix for the operator's stack (statement #7): tracing/OTel 0.32 core init, tonic/axum middlewares, manual-wrapper recipes for tokio-postgres + the official `clickhouse` crate (0.15+ trace-context propagation), redis/fred, lapin, Juniper resolvers + DataLoaders, metrics feeds (HTTP histograms, process CPU/mem, tokio runtime), Ratatui/bollard notes, and the version-lockstep hazard table.
- [frontend.md](capture/frontend.md) — browser collection, cross-tier correlation, source maps, and the frontend privacy problem.
- [sentry-ingest.md](capture/sentry-ingest.md) — future Sentry envelope/Relay/grouping ingest, envelope-item policy, and SDK fixture compatibility.
- [otlp.md](capture/otlp.md) — OpenTelemetry protocol/Collector context layer, transport profile, and receiver conformance.
- [agent-cli-tracing.md](capture/agent-cli-tracing.md) — coding-agent and CLI execution tracing, OTel semconv mapping, and trace overhead/redaction.
- [deploy-change-context.md](capture/deploy-change-context.md) — release/deploy/code-change/work-item evidence ("what changed?").
- [ci-and-flaky-tests.md](capture/ci-and-flaky-tests.md) — CI failure-context bundles and flaky-test detection/replay.
- [production-db-evidence.md](capture/production-db-evidence.md) — safety gate for treating production databases as evidence sources.
- [correlation.md](capture/correlation.md) — A4: correlation reliability on real telemetry.
- [run-id-standardization.md](capture/run-id-standardization.md) — **standing page**: no OTel standard for a CLI run id exists and `session.id` is only an interop bridge; the migration ladder (jackin.run_id → parallax.run.id → future standard), our draft upstream proposal (generalize sessions or `cli.run.id`), and the tracked semconv threads (#2883 et al.).
- [redaction.md](capture/redaction.md) — A6: redaction pipeline, detector toolchain, canary corpus, and red-team gate.

### `storage/` — the telemetry store and its evidence
- [native-otel-migration-plan.md](storage/native-otel-migration-plan.md) — living plan to adopt GreptimeDB native OTLP tables: decisions (Q1–Q6), grouping division-of-labor, file-by-file implementation roadmap, open questions.
- [greptimedb-team-questions.md](storage/greptimedb-team-questions.md) — detailed questions for the GreptimeDB team backing the native-OTLP adoption (review on next sync).
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
- [jackin-first-integration.md](validation/jackin-first-integration.md) — the first real CLI on Parallax: jackin' run telemetry over OTLP (design, verified end-to-end evidence, gaps found, operator verification recipe).

> A4 lives in [capture/correlation.md](capture/correlation.md), A5 in [decisions/stack-decision.md](decisions/stack-decision.md), A6 in [capture/redaction.md](capture/redaction.md).

### `market/` — landscape and competitor watch
- [landscape.md](market/landscape.md) — market research.
- [competitor-watch.md](market/competitor-watch.md) — consolidated watch: OpenObserve, SigNoz, Coroot, Bugsink, Rustrak, Traceway, GoSnag, Urgentry, Sentry/Seer/MCP, the MCP-power boundary, and the drift ledger.
- [alternatives-deep-analysis.md](market/alternatives-deep-analysis.md) — balanced skeptical analysis: 60+ alternatives surveyed, arguments FOR and AGAINST Parallax, empirical gates, and kill criteria.
- [competitive-comparison-matrix.md](market/competitive-comparison-matrix.md) — quick-reference matrix of all competitors against Parallax's 8 wedge dimensions.
- [agent-debugging-competitor-drift-2026-06-02.md](market/agent-debugging-competitor-drift-2026-06-02.md) — focused recheck of Syncause, AgentRx, Notrix Trax, AgentReplay, and OpenTelemetry MCP/replay/crash semantic-convention drift.
- [maple-deep-research.md](market/maple-deep-research.md) — standalone deep-dive: Maple (maple.dev), FSL-1.1, OTLP-only, single-binary local mode, 10+ MCP tools.
- [signoz-deep-research.md](market/signoz-deep-research.md) — standalone deep-dive: SigNoz, MIT-Expat core + Apache-2.0 MCP, ClickHouse, OTel-native (no Sentry path, no portable evidence schema), ~5-container local stack.
- [openobserve-deep-research.md](market/openobserve-deep-research.md) — standalone deep-dive: OpenObserve, Rust + Parquet/DataFusion, single-binary local, OTLP-native, Enterprise-gated AI SRE/RCA + 140+-tool MCP, 10/50/200 GB-day free-tier conflict resolved.
- [gonzo-deep-research.md](market/gonzo-deep-research.md) — standalone deep-dive: Gonzo (control-theory), MIT Go log-tail TUI, OTLP **logs-only** receiver, optional local AI — a viewer not a backend; MCP is the commercial Dstl8, not OSS Gonzo.
- [sentry-deep-research.md](market/sentry-deep-research.md) — standalone deep-dive: Sentry, the envelope incumbent Parallax interoperates with; OTLP support = beta HTTP-only traces+logs (no metrics); local run = ~20–40-container self-hosted stack vs Spotlight dev overlay; FSL, Relay/Kafka/Snuba, Seer + official MCP.
- [coroot-deep-research.md](market/coroot-deep-research.md) — standalone deep-dive: Coroot, Apache-2.0 eBPF zero-instrumentation obs + 2-stage AI RCA; OTLP/HTTP + Prometheus, ~5-container local, safest MCP (OAuth+RBAC, 18 tools); refreshes stale ledger facts (v1.22.2 / Claude Opus 4.6).
- [observability-feature-matrix.md](market/observability-feature-matrix.md) — **cross-tool feature matrix**: Parallax vs Maple, SigNoz, OpenObserve, Coroot, Sentry, Gonzo, feature-by-feature (ingest, signals, error workflow, local-run, AI/MCP, evidence/safety/outcomes, extras). Complements the wedge-axis comparison matrix.
- [backend-and-data-flow.md](market/backend-and-data-flow.md) — **backend + data-flow comparison**: storage engine per tool (ClickHouse / Parquet+DataFusion / GreptimeDB / in-memory), per-tool ingest→store→query ASCII schemas, write/read paths, vendor throughput, designed-for-what, and the "GreptimeDB in the wild" validation (TMA1/OpenFuse/Hebo).
- [missed-similar-tools-2026-06.md](market/missed-similar-tools-2026-06.md) — **discovery sweep** of untracked similar tools by backend: GreptimeDB-backed (TMA1/OpenFuse/Hebo), Parquet/DataFusion/DuckDB cohort (Parseable/Micromegas/Arc/IceGate), AI/MCP/RCA cohort (HolmesGPT/OpenSRE/AgentRx/kagent/Keep), backend fact-checks, and recommended next deep-dives.
- [closest-to-parallax-ranked.md](market/closest-to-parallax-ranked.md) — **ranked closeness analysis**: every competitor scored against Parallax's V1 shape, tiered T1 direct (TMA1 #1, OpenObserve, Maple) → T5 references, with what/how/features each implements and the remaining moat (portable redacted bundle + fix-outcome loop).

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
