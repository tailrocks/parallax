# Project Structure

This file is the lightweight map of the Parallax repository. Keep it current as
the project evolves.

## Current Stage

Parallax is in research and product-discovery mode. The repository should stay
simple: root-level project rules, a README, and Markdown research notes under
`docs/`.

There is no docs UI, application source tree, package manager, release process,
or CI contract yet.

## Root Files

| Path | Purpose |
| --- | --- |
| `README.md` | Short repository entry point with links to current research. |
| `AGENTS.md` | Canonical AI-agent instructions for this repository. |
| `CLAUDE.md` | Claude Code linker that points to `AGENTS.md`. |
| `BRANCHING.md` | Current `main`-first workflow and pull-request policy. |
| `COMMITS.md` | Commit message and AI-agent attribution conventions. |
| `PROJECT_STRUCTURE.md` | This repository map. |
| `.gitignore` | Local files that should not be committed. |

## Directories

| Path | Purpose |
| --- | --- |
| `docs/` | Documentation and research notes. No generated docs UI yet. |
| `docs/research/` | Market, product, and strategy research. |
| `prompts/` | Reusable research and agent prompts. |

## Research Documents

| Path | Purpose |
| --- | --- |
| `docs/research/project-thesis.md` | Original thesis. |
| `docs/research/verdict.md` | Phase 1 GO / NO-GO gate for whether Parallax is worth building. |
| `docs/research/risks-and-bear-case.md` | Adversarial counterweight to the verdict: steelmanned NO-GO case, load-bearing-assumption register, risk matrix, and NO-GO/strengthen triggers. |
| `docs/research/business-model-and-economics.md` | Economic/business-model analysis: is the category company-sized, licensing options (Apache-2.0 recommended), and value-capture seams (hosting, the fixer, enterprise ops add-ons) that do not gate the open differentiator. |
| `docs/research/market-landscape.md` | Market research. |
| `docs/research/self-hosted-observability-architecture.md` | Architecture research for the Sentry-compatible, OpenTelemetry-native self-hosted observability direction. |
| `docs/research/ci-failure-context-mvp.md` | MVP research for GitHub Actions failure-context bundles. |
| `docs/research/greptimedb-storage-evaluation.md` | Storage-layer evaluation for GreptimeDB, ClickHouse, and observability backends. |
| `docs/research/observability-storage-benchmark-plan.md` | Database-only benchmark plan for observability storage candidates (rationale, axes, decision criteria). |
| `docs/research/storage-benchmark-prototype.md` | Runnable storage-benchmark harness spec: `StorageAdapter` trait, seeded dataset generator, per-candidate DDL, exact evidence-bundle/correlation queries, measurement protocol, and numeric decision gates. Has veto power over the default storage choice. |
| `docs/research/metadata-store-benchmark-plan.md` | Turso-first benchmark plan and runnable prototype spec for product metadata, agent session state, CLI invocation state, audit records, crash/restore tests, and Postgres fallback gates. |
| `docs/research/messaging-and-ingestion-layer.md` | Stream and ingest-layer evaluation for Apache Iggy, Redpanda, NATS JetStream, and brokerless startup deployments. |
| `docs/research/causal-reconstruction-and-agent-safety.md` | Evidence-graph, causal reconstruction, agent autonomy, MCP, and production-data safety analysis. |
| `docs/research/ai-native-observability-and-incident-intelligence.md` | Current AI-native observability, incident-intelligence, agent workflow, strategic positioning, and product-wedge synthesis. |
| `docs/research/flaky-test-investigation-and-replay.md` | Flaky-test detection, history, clustering, retry/replay, reproducer, agent-fixability, and wedge analysis. |
| `docs/research/agent-and-cli-execution-tracing.md` | Coding-agent session tracing, CLI application tracing, OpenTelemetry GenAI/MCP/CLI mapping, market room, data model, and unified execution graph analysis. |
| `docs/research/agent-observability-technical-review.md` | Technical review of current agent-observability tools, instrumentation patterns, storage/eval/redaction lessons, and the Parallax-specific audit gap. |
| `docs/research/strategic-verdict-and-research-coverage.md` | Final strategic verdict, prompt coverage map, key decisions, unresolved proof gates, and prototype acceptance criteria. |
| `docs/research/technical-implementation-concept.md` | Opinionated end-to-end blueprint with named component choices, deployment profiles, data flow, and rejected alternatives. |
| `docs/research/evidence-bundle-and-schema.md` | Concrete `v0` spec for the portable evidence bundle and open evidence schema (envelope, node/edge catalog, hypothesis/confidence model, redaction report, versioning) — the named moat artifact. |
| `docs/research/sentry-compatible-ingestion.md` | Focused Sentry envelope, Relay, grouping, fingerprinting, stacktrace normalization, and event pipeline analysis for Parallax ingestion. |
| `docs/research/opentelemetry-protocol-and-context-layer.md` | Focused OpenTelemetry protocol, OTLP endpoint, Collector, Rotel, semantic context, scaling, and above-OTEL product opportunity analysis. |
| `docs/research/rust-data-collection-and-instrumentation.md` | Rust-first data-collection decision (SDK/OTLP vs eBPF) and error-capture data model. |
| `docs/research/frontend-collection-and-cross-tier-correlation.md` | Frontend (browser JS/TS) collection method, cross-tier frontend↔backend trace propagation, schema extension (frontend nodes/edges), source-map symbolication, and the frontend privacy problem. |

## Prompts

| Path | Purpose |
| --- | --- |
| `prompts/README.md` | How to use the prompts in this folder (one-off, `/goal`, `/loop`). |
| `prompts/deep-research-parallax.md` | Deep research brief for validating the AI-native debugging/investigation direction. |

## Update Rule

When adding a new top-level file, directory, or durable research area, update
this file in the same commit.
