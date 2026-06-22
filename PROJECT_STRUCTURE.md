# Project Structure

This file is the lightweight map of the Parallax repository. Keep it current as
the project evolves.

## Current Stage

Parallax is in research and product-discovery mode. The repository should stay
simple: root-level project rules, a README, and Markdown research notes under
`docs/`.

V1 implementation is underway under `crates/` (authorized 2026-06-12); there
is no release process or CI contract yet.

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
| `docs/guide/` | User-facing V1 docs shipped per scope §2.8: quickstart, CLI reference, agent how-to, conventions, jackin' integration recipe. |
| `docs/research/` | Market, product, and strategy research, grouped by topic. The canonical per-note index is [`docs/research/README.md`](docs/research/README.md). |
| `docs/research/00-vision/` | Why this product: problem/audience/product-shape front door, the north-star autonomous fix loop + impossible triangle, thesis, world-before-Parallax stack, platform direction, AI-native observability synthesis. |
| `docs/research/decisions/` | ADR-style decision records — current truth, conclusion first (go/no-go, strategic coverage, risks, the dated skeptical re-assessment, storage engine, V1 storage adapter vision, native OTLP tables adoption, stack decision, metadata store, agent access surface, fixer boundary). |
| `docs/research/architecture/` | How the pieces fit: implementation concept, overview, evidence-bundle schema, API concept, causal reconstruction, local-first V1, simple UI V2, build roadmap, autonomous fix loop, integration contract, PoC coverage map, V1 build plan, deployment architecture map. |
| `docs/research/capture/` | How each signal is collected and made safe: rust, the operator-stack instrumentation matrix (tracing/OTel crates, gRPC/HTTP/Postgres/ClickHouse/Redis/RabbitMQ/GraphQL recipes), frontend, OTLP-first ingest, future sentry-ingest, agent/CLI tracing, deploy/change context, CI/flaky tests, production-DB evidence, correlation (A4), redaction (A6). |
| `docs/research/storage/` | Telemetry-store evaluation, benchmark plan, freshness/latency and size/object-cost gates, the native-OTLP adoption plan (`native-otel-migration-plan.md`) and the GreptimeDB-team question list (`greptimedb-team-questions.md`), plus `metadata/` and `streaming/` evidence subdirs. |
| `docs/research/storage/greptimedb-vs-clickhouse/` | Deep white-box GreptimeDB vs ClickHouse internals comparison: one-page verdict, run-log, 30+ mechanism notes, the four-build version matrix, and benchmarks. Produced by an indefinite `/goal` or Claude Code `/loop`. |
| `docs/research/validation/` | A1–A7 assumption gates and ledgers (A1 bundle-value subdir, A1 trajectory-source checks, A2 user demand, A3 schema/corpus, A7 scope, self-hosted simplicity, business model, repo intent, profitability analysis). |
| `docs/research/market/` | Market landscape, consolidated competitor watch, alternatives deep analysis, competitive comparison matrix, wider alternatives survey, and focused agent-debugging competitor drift notes. |
| `docs/research/market/maple-deep-research.md` | Deep research on Maple (maple.dev): architecture, features, UX, threat assessment, and Parallax comparison. |
| `docs/research/market/signoz-deep-research.md` | Deep research on SigNoz: MIT-Expat core + proprietary `ee/` + Apache-2.0 MCP, ClickHouse stack, OTel-native (no Sentry path), MCP + agent-skills, "Postmortem Evidence Pack" (no portable schema), threat assessment and Parallax comparison. |
| `docs/research/market/openobserve-deep-research.md` | Deep research on OpenObserve: Rust engine, Parquet-on-object-storage + DataFusion, single-binary local, OTLP-native, AI SRE/RCA + 140+-tool Enterprise MCP, the 10/50/200 GB-day free-tier conflict, threat assessment and Parallax comparison. |
| `docs/research/market/gonzo-deep-research.md` | Deep research on Gonzo (control-theory): MIT Go log-tail TUI with an OTLP **logs-only** receiver and optional local AI; a viewer not a backend/store; MCP belongs to commercial Dstl8, not OSS Gonzo. Category-boundary note + Parallax comparison. |
| `docs/research/market/sentry-deep-research.md` | Deep research on Sentry (the envelope incumbent Parallax interoperates with): answers "does Sentry support OTLP" (beta, HTTP-only traces+logs, no metrics) and "how to run it locally" (~20–40 container self-hosted stack vs the Spotlight dev overlay); FSL license, Relay/Kafka/Snuba/ClickHouse architecture, Seer + official MCP, threat assessment. |
| `docs/research/market/coroot-deep-research.md` | Deep research on Coroot: Apache-2.0 eBPF zero-instrumentation observability + 2-stage AI RCA; OTLP/HTTP + Prometheus, ~5-container local stack, the field's safest MCP (per-user OAuth+RBAC, 18 tools, 1 mutating); refreshes the stale ledger facts (now v1.22.2 / Claude Opus 4.6); threat assessment and Parallax comparison. |
| `docs/research/market/observability-feature-matrix.md` | **Cross-tool feature matrix** — Parallax vs Maple, SigNoz, OpenObserve, Coroot, Sentry, Gonzo, feature-by-feature (ingest/protocols, signals/storage, error workflow, local-run, AI/MCP, evidence/safety/outcomes, platform extras), with ✅/🟡/❌ marks sourced from each standalone deep-dive. Complements the wedge-axis `competitive-comparison-matrix.md`. |
| `docs/research/market/backend-and-data-flow.md` | **Backend + data-flow comparison** — which storage engine each tool uses (ClickHouse / Parquet+DataFusion / GreptimeDB / in-memory), per-tool ingest→process→store→query ASCII schemas, write/read paths, vendor throughput numbers, and what each design is best for. Includes the "GreptimeDB in the wild" validation (TMA1/OpenFuse/Hebo). Backend companion to the feature matrix. |
| `docs/research/market/missed-similar-tools-2026-06.md` | **Discovery sweep** of similar tools not previously tracked, grouped by backend: Tier A GreptimeDB-backed (TMA1, OpenFuse, Hebo), Tier B Parquet/DataFusion/DuckDB (Parseable, Micromegas, Arc, IceGate, smithclay), Tier C AI/MCP/RCA (HolmesGPT, OpenSRE, AgentRx, kagent, Keep), plus backend fact-checks (HyperDX/Uptrace/Dash0=ClickHouse, etc.) and recommended next deep-dives. |
| `docs/research/market/tma1-deep-research.md` | **Deep teardown of TMA1** (`tma1-ai/tma1`) — the closest architectural competitor: Go single binary embedding GreptimeDB as a child process, OTLP reverse-proxy `:14318`, strictly read-only 7-tool MCP serving `get_context_bundle`. Key finding: the "context bundle" is a live, unversioned, **unredacted** session snapshot (not a portable evidence artifact); no metadata store, no Sentry, no error fingerprinting, no redaction, no real outcome loop — dev-machine AI-agent observability, not a production platform. Covers OpenFuse (Langfuse→GreptimeDB fork) and a per-capability Parallax comparison. |
| `docs/research/market/closest-to-parallax-ranked.md` | **Ranked closeness analysis** — every competitor scored against Parallax's actual V1 shape (Rust single-binary, OTLP-native, GreptimeDB+Turso, derived `error_event`, redacted versioned bundles, read-only MCP, fix-outcome loop). Tiers: T1 direct (TMA1 #1 near-mirror, OpenObserve, Maple), T2 strong-overlap (Micromegas, SigNoz, Coroot), T3 evidence/agent-thesis (AgentRx, OpenSRE, HolmesGPT), T4 incumbent-interop (Sentry), T5 references. With what/how/features per tool and the remaining moat. |
| `docs/research/market/open-source-observability-tools-survey.md` | Broad survey of 37 open-source observability tools across three tiers (major platforms, established, emerging). |
| `docs/research/reference/ai-native-debugging-tools.md` | Survey of 13 AI-native debugging, SRE agent, and coding-agent observability tools. |
| `docs/research/capture/sentry-compatible-oss-tools.md` | Survey of 10 Sentry-compatible or Sentry-alternative open-source tools for future adapter/market tracking. |
| `docs/research/reference/` | External technical reviews. Includes `ai-native-debugging-tools.md` (open-source AI debugging agents, SRE agents, coding-agent observability tools). |
| `prompts/` | Reusable research and agent prompts. |
| `bench/` | Local storage-benchmark scaffolding: pinned `compose.yml` for GreptimeDB + ClickHouse smoke runs. Generated datasets/results are gitignored; only compose/scripts are tracked. Consistent with [`docs/research/storage/benchmark-plan.md`](docs/research/storage/benchmark-plan.md). |
| `bench/otlp-fanout/` | OTLP fan-out comparison lab: Rotel hub + competitor backends in Compose, fanning one OTLP stream to OpenObserve/SigNoz/Maple/Sentry and back to host Parallax. Implements [`docs/research/validation/otlp-fanout-comparison-lab.md`](docs/research/validation/otlp-fanout-comparison-lab.md). Vendored clones (`vendor/`) are gitignored. |
| `poc/` | Concept-proving Rust code (operator-approved 2026-06-11). Small, runnable, test-covered proofs of designed mechanisms — not product code, no product claims. First artifact: `poc/evidence-loop/` (OTLP JSON → derived error events → fingerprint → trigger → redacted evidence bundle with canonical hash). Frozen as the concept reference; logic graduates into `crates/` by copy-and-adapt. |
| `crates/` | The V1 product workspace (Rust, edition 2024): `parallax-cli` (the installed `parallax` binary), `parallax-server` (OTLP ingest, API host, workers, engine supervision), `parallax-core` (derivation/fingerprinting/bundles), `parallax-storage` (spool + storage adapters), `parallax-api` (GraphQL schema), `parallax-proto` (OTLP types). Contracts in [`docs/research/architecture/v1-implementation-spec.md`](docs/research/architecture/v1-implementation-spec.md); brief in [`prompts/v1-implementation.md`](prompts/v1-implementation.md). |
| `ui/` | The V1 web UI: TanStack Start SPA (shadcn/ui on Base UI, shadcn charts) served by `parallax serve` from `ui/dist/client` or embedded via the `embed-ui` feature. Talks only to the canonical GraphQL API. |
| `.github/workflows/` | CI, stable release, and preview-release automation. Release workflows build binary archives with Zig/`cargo-zigbuild`; `preview.yml` publishes the rolling `preview` GitHub Release and rewrites the CI-owned `parallax-preview.rb` formula in the per-project `tailrocks/homebrew-parallax` tap. |
| `mise.toml` | Shared tool versions for CI/release automation: cargo-nextest, cargo-zigbuild, Zig, Bun, cosign, and syft. |
| `scripts/` | Operational scripts: `release.sh` (UI build -> `--features embed-ui` release binary -> tarball + sha256). |

## Research Record

The research is grouped by topic under `docs/research/`. For the full,
conclusion-first per-note index, see [`docs/research/README.md`](docs/research/README.md).
The decisions front door is [`docs/research/decisions/`](docs/research/decisions/): each
settled question has exactly one ADR (conclusion first, linking to its evidence), and the
still-open proof gates are enumerated in
[`decisions/strategic-coverage.md`](docs/research/decisions/strategic-coverage.md) and the
relevant `validation/` notes. The prioritized, decision-moving research backlog (what to research
and compare next, ranked) is [`docs/research/research-agenda.md`](docs/research/research-agenda.md).

| Area | Where |
| --- | --- |
| Vision and thesis | [`00-vision/`](docs/research/00-vision/) |
| Decisions (ADRs) | [`decisions/`](docs/research/decisions/) |
| Architecture | [`architecture/`](docs/research/architecture/) |
| Signal capture and safety | [`capture/`](docs/research/capture/) |
| Storage and the engine sub-study | [`storage/`](docs/research/storage/) |
| Assumption validation (A1–A7) | [`validation/`](docs/research/validation/) |
| Market and competitors | [`market/`](docs/research/market/) |
| External reference | [`reference/`](docs/research/reference/) |

## Prompts

| Path | Purpose |
| --- | --- |
| `prompts/README.md` | How to use the prompts in this folder (one-off, `/goal`, `/loop`). |
| `prompts/AGENTS.md` | Authoring rule for this folder: prompt files stay goal-only and runnable; run-mechanics and how-to-use live in `prompts/README.md`. |
| `prompts/CLAUDE.md` | Thin pointer to `prompts/AGENTS.md`. |
| `prompts/deep-research-parallax.md` | Deep research brief for validating the AI-native debugging/investigation direction. |
| `prompts/greptimedb-vs-clickhouse-internals.md` | Never-ending `/goal` or Claude Code `/loop` brief for the under-the-hood GreptimeDB vs ClickHouse comparison; writes to `docs/research/storage/greptimedb-vs-clickhouse/`. |
| `prompts/v1-implementation.md` | `/goal` brief implementing Parallax V1 end-to-end from the authoritative docs, milestone by milestone, with the v1-scope acceptance scenarios as the stop condition. |

## Update Rule

When adding a new top-level file, directory, or durable research area, update
this file in the same commit.
