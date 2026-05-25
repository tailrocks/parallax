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
| `docs/research/greptimedb-vs-clickhouse/` | Deep under-the-hood GreptimeDB vs ClickHouse internals comparison, produced by an indefinite research loop. |
| `prompts/` | Reusable research and agent prompts. |
| `bench/` | Local storage-benchmark scaffolding: pinned `compose.yml` for GreptimeDB + ClickHouse smoke runs. Generated datasets/results are gitignored; only compose/scripts are tracked. Consistent with `docs/research/storage-benchmark-prototype.md`. |

## Research Documents

| Path | Purpose |
| --- | --- |
| `docs/research/project-thesis.md` | Original thesis. |
| `docs/research/verdict.md` | Phase 1 GO / NO-GO gate for whether Parallax is worth building. |
| `docs/research/risks-and-bear-case.md` | Adversarial counterweight to the verdict: steelmanned NO-GO case, load-bearing-assumption register, risk matrix, and NO-GO/strengthen triggers. |
| `docs/research/repo-intent-dependence.md` | Strategic question 13: how much Parallax depends on a context-rich monorepo. Decomposes value into a runtime-evidence floor (no monorepo needed) and an opt-in repo-intent multiplier; degraded mode is the common case the product must serve. |
| `docs/research/business-model-and-economics.md` | Economic/business-model analysis: is the category company-sized, licensing options (Apache-2.0 recommended), and value-capture seams (hosting, the fixer, enterprise ops add-ons) that do not gate the open differentiator. |
| `docs/research/user-interview-and-deployment-intent-gate.md` | A2 validation runbook: target interview slices, past-behavior question bank, scoring rubric, deployment/data/budget commitment tests, and pass/continue/kill criteria for proving demand beyond the operator. |
| `docs/research/a2-interview-evidence-ledger.md` | A2 evidence-ledger contract: redacted result schema, artifact boundary, evidence classes, commitment ladder, and bias controls for making deployment-intent interviews auditable without committing raw private notes. |
| `docs/research/schema-adoption-and-corpus-moat-gate.md` | A3 validation gate: canonical schema/conformance artifacts, adoption-clock thresholds, failure/fix corpus events, compatibility policy, and moat-claim limits. |
| `docs/research/a3-schema-adoption-corpus-ledger.md` | A3 schema-adoption and corpus ledger: public event schemas, counting rules, claim levels, and refresh cadence for reviews, integrations, conformance runs, compatibility decisions, and outcome rows. |
| `docs/research/self-hosted-simplicity-gate.md` | Operational proof gate for kill criterion 6: measure Phase 1 tiny-tier setup against current self-hosted Sentry, SigNoz, OpenObserve, GreptimeDB, and Turso/libSQL baselines. |
| `docs/research/self-hosted-deployment-baseline-inventory.md` | Source-linked version and deployment-shape manifest for the self-hosted simplicity benchmark: Sentry, SigNoz, OpenObserve, Bugsink, Rustrak, Traceway, GoSnag, and Urgentry. |
| `docs/research/market-landscape.md` | Market research. |
| `docs/research/open-self-hosted-competitor-watch.md` | Focused watchlist for OpenObserve, SigNoz, and Coroot: current primary-source posture, remaining Parallax gaps, and trigger conditions that would reopen the GO verdict. |
| `docs/research/lightweight-sentry-compatible-competitor-watch.md` | Focused watchlist for Bugsink, Rustrak, Traceway, GoSnag, and Urgentry: lightweight Sentry-compatible or OTLP-native self-hosted challengers that pressure Parallax's migration and simplicity claims. |
| `docs/research/agentic-observability-competitor-drift-ledger.md` | Market-drift ledger for agentic observability competitors: current source snapshot, drift levels, trigger-hit rows, refresh triggers, and public wording boundaries. |
| `docs/research/self-hosted-observability-architecture.md` | Architecture research for the Sentry-compatible, OpenTelemetry-native self-hosted observability direction. |
| `docs/research/ci-failure-context-mvp.md` | MVP research for GitHub Actions failure-context bundles. |
| `docs/research/greptimedb-storage-evaluation.md` | Storage-layer evaluation for GreptimeDB, ClickHouse, and observability backends. |
| `docs/research/observability-storage-benchmark-plan.md` | Database-only benchmark plan for observability storage candidates (rationale, axes, decision criteria). |
| `docs/research/storage-benchmark-prototype.md` | Runnable storage-benchmark harness spec: `StorageAdapter` trait, seeded dataset generator, per-candidate DDL, exact evidence-bundle/correlation queries, measurement protocol, and numeric decision gates. Has veto power over the default storage choice. |
| `docs/research/storage-freshness-and-bundle-latency-gate.md` | Proof gate for mixed-load ingest-to-queryable freshness and Q6 evidence-bundle latency under concurrent ingest; specifies timing definitions, per-signal probes, pass targets, and storage-default consequences. |
| `docs/research/storage-size-and-object-cost-gate.md` | Proof gate for retained size, per-signal compression, object-store request/egress cost, cache dependency, and provider cost projection across GreptimeDB and ClickHouse. |
| `docs/research/greptimedb-vs-clickhouse/` | White-box internals comparison of GreptimeDB and ClickHouse: source-level teardown of write/read paths, indexing, compaction, compression, execution, and the distributed model, mapped to per-signal speed/cost/scaling and a build decision. Produced by an indefinite `/loop`. |
| `docs/research/retention-cost-model.md` | Quantified retention cost math (object-store pricing, per-signal compression, 3-tier worked model); finds object-storage retention is ~100× cheaper than ingest-priced SaaS and that egress pricing favors R2/B2 over S3 for a re-read-heavy context engine. |
| `docs/research/metadata-store-benchmark-plan.md` | Turso-first benchmark plan and runnable prototype spec for product metadata, agent session state, CLI invocation state, audit records, crash/restore tests, and Postgres fallback gates. |
| `docs/research/turso-metadata-production-readiness.md` | Turso metadata production-readiness gate: current Turso source posture, local-vs-cloud distinction, MVCC conflict/CDC/sync constraints, backup/restore requirements, and Postgres fallback triggers. |
| `docs/research/messaging-and-ingestion-layer.md` | Stream and ingest-layer evaluation for Apache Iggy, Redpanda, NATS JetStream, and brokerless startup deployments. |
| `docs/research/ingest-log-replay-and-backpressure-gate.md` | Proof gate for the append-only ingest log: local WAL versus Iggy/NATS/Redpanda replay, backpressure, durability modes, fault tests, and pass/fail criteria. |
| `docs/research/a5-stack-decision-ledger.md` | A5 stack-decision result contract: roll up storage speed/cost, metadata, ingest-log, setup, and integration gates into explicit stack claim levels and fallback triggers. |
| `docs/research/causal-reconstruction-and-agent-safety.md` | Evidence-graph, causal reconstruction, agent autonomy, MCP, and production-data safety analysis. |
| `docs/research/redaction-pipeline-and-secret-safety.md` | A6 redaction-trust plan: source-specific minimization, default-deny ingest policy, detector/output passes, redaction report fields, and red-team gate before agent exposure. |
| `docs/research/rust-stacktrace-grouping-and-symbolication.md` | Rust grouping proof gate: debuginfo policy, symbolication status, conservative frame normalization, `rust-stack-v1` fingerprinting, and fixture matrix for grouping stability across rebuild/debug-info variants. |
| `docs/research/ai-native-observability-and-incident-intelligence.md` | Current AI-native observability, incident-intelligence, agent workflow, strategic positioning, and product-wedge synthesis. |
| `docs/research/flaky-test-investigation-and-replay.md` | Flaky-test detection, history, clustering, retry/replay, reproducer, agent-fixability, and wedge analysis. |
| `docs/research/agent-and-cli-execution-tracing.md` | Coding-agent session tracing, CLI application tracing, OpenTelemetry GenAI/MCP/CLI mapping, market room, data model, and unified execution graph analysis. |
| `docs/research/agent-cli-otel-semconv-mapping.md` | Contract for mapping OpenTelemetry GenAI, MCP, CLI, process, and CI/CD semantic conventions into stable Parallax agent/session/CLI rows, with versioning, content-capture levels, deduplication, trace propagation, and lossiness gates. |
| `docs/research/agent-session-tracing-real-tools.md` | Proof gate for agent-session tracing across Codex, Claude Code, Amp, and OpenCode: adapter surfaces, normalized schema, redaction defaults, value-eval matrix, and pass/fail criteria. |
| `docs/research/cli-trace-overhead-and-redaction.md` | Proof gate for default-on CLI tracing: structural capture policy, args/env/config/stdout/stderr redaction, canary fixtures, overhead budgets, and failure criteria. |
| `docs/research/agent-access-surface-cli-api-mcp.md` | Focused decision on the agent access surface: canonical HTTP API, day-one CLI, read-only MCP adapter, security rules, implementation order, and MCP shipping gate. |
| `docs/research/fixer-component-and-outcome-loop.md` | Boundary and contract for the separate fixer component: evidence-bundle request, patch/PR autonomy levels, outcome records, gates, and why PR creation is commodity while outcome feedback is moat-bearing. |
| `docs/research/production-database-evidence-access.md` | Safety gate for treating production databases as evidence sources: telemetry-first DB context, read-only query templates, least privilege, RLS/views, redaction, audit, and no generic SQL tools. |
| `docs/research/agent-observability-technical-review.md` | Technical review of current agent-observability tools, instrumentation patterns, storage/eval/redaction lessons, and the Parallax-specific audit gap. |
| `docs/research/strategic-verdict-and-research-coverage.md` | Final strategic verdict, prompt coverage map, key decisions, unresolved proof gates, and prototype acceptance criteria. |
| `docs/research/technical-implementation-concept.md` | Opinionated end-to-end blueprint with named component choices, deployment profiles, data flow, and rejected alternatives. |
| `docs/research/build-roadmap-and-validation-sequence.md` | De-risking build sequence: phases ordered to kill the project cheapest-first, with go/no-go gates tied to bear-case assumptions (validate A1 bundle value and A2 users before the storage benchmark). |
| `docs/research/future-platform-direction.md` | Disciplined answer to the Final Goal: the platform/intelligence-layer outcome as an earned, gated, three-stage emergence from the narrow evidence-engine wedge — sound as an evidence engine, flawed only if launched as a day-one platform. |
| `docs/research/evidence-bundle-and-schema.md` | Concrete `v0` spec for the portable evidence bundle and open evidence schema (envelope, node/edge catalog, hypothesis/confidence model, redaction report, versioning) — the named moat artifact. |
| `docs/research/bundle-value-evaluation.md` | Experiment design for the existential claim (kill criterion 3 / assumption A1): does a Parallax bundle beat raw context for agent fix quality? Arms incl. a raw-telemetry-dump control, dataset options, metrics, and the decision gate. |
| `docs/research/bundle-value-seed-corpus.md` | Seed-corpus selection note for the A1 bundle-value eval: current executable SWE-style task sources, task eligibility gates, telemetry overlay requirements, and manifest shape before running Phase 0. |
| `docs/research/phase0-telemetry-overlay-contract.md` | Deterministic telemetry-overlay artifact contract for the A1 Phase 0 eval: provenance labels, no-cheat rules, normalized rows, raw-vs-bundle evidence parity, redaction, and pass/fail gates. |
| `docs/research/a1-eval-result-ledger-and-model-refresh.md` | A1 result-ledger and refresh policy: public run manifests, model snapshots, contamination tiers, per-arm result rows, claim levels, and expiry/rerun triggers for bundle-value claims. |
| `docs/research/bundle-value-phase0-runbook.md` | Concrete first-pass A1 runbook: task mix, arms, artifact contract, agent-run protocol, scoring, analysis, and continue/kill thresholds for testing bundles against raw telemetry dumps. |
| `docs/research/sentry-compatible-ingestion.md` | Focused Sentry envelope, Relay, grouping, fingerprinting, stacktrace normalization, and event pipeline analysis for Parallax ingestion. |
| `docs/research/sentry-sdk-fixture-compatibility.md` | Fixture-driven Sentry compatibility gate: current SDK-generated envelopes, Rust-first fixture matrix, parser oracle strategy, normalization snapshots, and honest product wording. |
| `docs/research/sentry-sdk-compatibility-ledger.md` | Result-ledger contract for Sentry SDK compatibility claims: claim levels, fixture-run artifacts, row schemas, expiry triggers, and allowed product wording. |
| `docs/research/opentelemetry-protocol-and-context-layer.md` | Focused OpenTelemetry protocol, OTLP endpoint, Collector, Rotel, semantic context, scaling, and above-OTEL product opportunity analysis. |
| `docs/research/otlp-receiver-conformance-and-collector-equivalence.md` | Fixture and conformance gate for OTLP-native claims: direct Rust SDK, official Collector, Collector Contrib, and Rotel equivalence over normalized Parallax evidence rows. |
| `docs/research/otlp-conformance-ledger.md` | Result-ledger contract for OTLP-native and Collector-compatible claims: protocol/version matrix, run artifacts, row schemas, expiry triggers, and allowed product wording. |
| `docs/research/rust-data-collection-and-instrumentation.md` | Rust-first data-collection decision (SDK/OTLP vs eBPF) and error-capture data model. |
| `docs/research/frontend-collection-and-cross-tier-correlation.md` | Frontend (browser JS/TS) collection method, cross-tier frontend↔backend trace propagation, schema extension (frontend nodes/edges), source-map symbolication, and the frontend privacy problem. |
| `docs/research/correlation-reliability-real-telemetry-gate.md` | A4 validation gate for real telemetry: strong-edge prevalence, trace/log/release/deploy coverage, frontend continuation, async links, false-strong-edge audit, and missing-evidence reporting. |
| `docs/research/a4-correlation-reliability-ledger.md` | A4 result-ledger contract: run manifests, per-anchor rows, manual audit rows, instrumentation repairs, claim levels, and freshness rules for proving real-telemetry correlation claims. |
| `docs/research/deploy-change-and-issue-context.md` | Contract for release/deploy/code-change/work-item evidence: GitHub/Sentry/Linear/Jira sources, normalized nodes, edge-strength rules, missing evidence, privacy defaults, and proof gates. |
| `docs/research/redaction-detector-toolchain.md` | A6 detector/toolchain decision: use a Rust default-deny runtime redaction engine, with Gitleaks, TruffleHog, detect-secrets, Presidio, and GitHub patterns as offline validators and red-team comparators. |
| `docs/research/a6-redaction-red-team-ledger.md` | A6 result-ledger contract: red-team run manifests, seeded surface fixture rows, scanner comparisons, projection audits, usefulness audits, repair rows, claim levels, and freshness rules. |
| `docs/research/a7-scope-discipline-ledger.md` | A7 result-ledger contract: phase budgets, component/dependency/feature rows, admission rules, warning triggers, and scope claim levels for keeping the tiny tier buildable. |

## Prompts

| Path | Purpose |
| --- | --- |
| `prompts/README.md` | How to use the prompts in this folder (one-off, `/goal`, `/loop`). |
| `prompts/deep-research-parallax.md` | Deep research brief for validating the AI-native debugging/investigation direction. |
| `prompts/greptimedb-vs-clickhouse-internals.md` | Never-ending `/loop` brief for the under-the-hood GreptimeDB vs ClickHouse comparison; writes to `docs/research/greptimedb-vs-clickhouse/`. |

## Update Rule

When adding a new top-level file, directory, or durable research area, update
this file in the same commit.
