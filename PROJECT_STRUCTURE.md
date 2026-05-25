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
| `docs/research/repo-intent-dependence.md` | Strategic question 13: how much Parallax depends on a context-rich monorepo. Decomposes value into a runtime-evidence floor (no monorepo needed) and an opt-in repo-intent multiplier; degraded mode is the common case the product must serve. |
| `docs/research/business-model-and-economics.md` | Economic/business-model analysis: is the category company-sized, licensing options (Apache-2.0 recommended), and value-capture seams (hosting, the fixer, enterprise ops add-ons) that do not gate the open differentiator. |
| `docs/research/market-landscape.md` | Market research. |
| `docs/research/open-self-hosted-competitor-watch.md` | Focused watchlist for OpenObserve, SigNoz, and Coroot: current primary-source posture, remaining Parallax gaps, and trigger conditions that would reopen the GO verdict. |
| `docs/research/self-hosted-observability-architecture.md` | Architecture research for the Sentry-compatible, OpenTelemetry-native self-hosted observability direction. |
| `docs/research/ci-failure-context-mvp.md` | MVP research for GitHub Actions failure-context bundles. |
| `docs/research/greptimedb-storage-evaluation.md` | Storage-layer evaluation for GreptimeDB, ClickHouse, and observability backends. |
| `docs/research/observability-storage-benchmark-plan.md` | Database-only benchmark plan for observability storage candidates (rationale, axes, decision criteria). |
| `docs/research/storage-benchmark-prototype.md` | Runnable storage-benchmark harness spec: `StorageAdapter` trait, seeded dataset generator, per-candidate DDL, exact evidence-bundle/correlation queries, measurement protocol, and numeric decision gates. Has veto power over the default storage choice. |
| `docs/research/retention-cost-model.md` | Quantified retention cost math (object-store pricing, per-signal compression, 3-tier worked model); finds object-storage retention is ~100× cheaper than ingest-priced SaaS and that egress pricing favors R2/B2 over S3 for a re-read-heavy context engine. |
| `docs/research/metadata-store-benchmark-plan.md` | Turso-first benchmark plan and runnable prototype spec for product metadata, agent session state, CLI invocation state, audit records, crash/restore tests, and Postgres fallback gates. |
| `docs/research/messaging-and-ingestion-layer.md` | Stream and ingest-layer evaluation for Apache Iggy, Redpanda, NATS JetStream, and brokerless startup deployments. |
| `docs/research/causal-reconstruction-and-agent-safety.md` | Evidence-graph, causal reconstruction, agent autonomy, MCP, and production-data safety analysis. |
| `docs/research/ai-native-observability-and-incident-intelligence.md` | Current AI-native observability, incident-intelligence, agent workflow, strategic positioning, and product-wedge synthesis. |
| `docs/research/flaky-test-investigation-and-replay.md` | Flaky-test detection, history, clustering, retry/replay, reproducer, agent-fixability, and wedge analysis. |
| `docs/research/agent-and-cli-execution-tracing.md` | Coding-agent session tracing, CLI application tracing, OpenTelemetry GenAI/MCP/CLI mapping, market room, data model, and unified execution graph analysis. |
| `docs/research/agent-observability-technical-review.md` | Technical review of current agent-observability tools, instrumentation patterns, storage/eval/redaction lessons, and the Parallax-specific audit gap. |
| `docs/research/strategic-verdict-and-research-coverage.md` | Final strategic verdict, prompt coverage map, key decisions, unresolved proof gates, and prototype acceptance criteria. |
| `docs/research/technical-implementation-concept.md` | Opinionated end-to-end blueprint with named component choices, deployment profiles, data flow, and rejected alternatives. |
| `docs/research/build-roadmap-and-validation-sequence.md` | De-risking build sequence: phases ordered to kill the project cheapest-first, with go/no-go gates tied to bear-case assumptions (validate A1 bundle value and A2 users before the storage benchmark). |
| `docs/research/future-platform-direction.md` | Disciplined answer to the Final Goal: the platform/intelligence-layer outcome as an earned, gated, three-stage emergence from the narrow evidence-engine wedge — sound as an evidence engine, flawed only if launched as a day-one platform. |
| `docs/research/evidence-bundle-and-schema.md` | Concrete `v0` spec for the portable evidence bundle and open evidence schema (envelope, node/edge catalog, hypothesis/confidence model, redaction report, versioning) — the named moat artifact. |
| `docs/research/bundle-value-evaluation.md` | Experiment design for the existential claim (kill criterion 3 / assumption A1): does a Parallax bundle beat raw context for agent fix quality? Arms incl. a raw-telemetry-dump control, dataset options, metrics, and the decision gate. |
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
