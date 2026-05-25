# A7 Scope Discipline Ledger

<!-- markdownlint-disable MD013 -->

Research date: 2026-05-25

## Purpose

This note turns assumption A7 from
[Risks and the bear case](risks-and-bear-case.md) into an auditable scope
control ledger:

> The component scope is buildable by the team that exists.

A7 is not proven by a roadmap that says "tiny tier first." It is proven only if
every milestone, dependency, interface, and feature addition is counted against a
phase budget, and if post-tiny ambitions are actively deferred until the gates
that justify them are green.

The failure mode this ledger prevents is clear: Parallax can be technically
right and still fail because it tries to ship Sentry-compatible ingest, OTLP,
storage, metadata, redaction, correlation, frontend capture, CLI tracing,
agent-session tracing, MCP, UI, benchmarks, and a fixer loop all at once. A7
exists to keep the first build narrow enough to reach a useful evidence bundle.

## Current Primary-Source Checks

| Source | Current read for A7 |
| --- | --- |
| [Sentry self-hosted docs](https://develop.sentry.dev/self-hosted/) | Sentry self-hosted is explicitly a minimal setup for simple use cases, has no dedicated support, uses Docker/Docker Compose plus scripts, lists 4 CPU cores, 16 GB RAM plus 16 GB swap, and warns that scaling larger installs becomes custom and complex. This is the complexity baseline Parallax must not recreate. |
| [Sentry self-hosted data flow](https://develop.sentry.dev/self-hosted/data-flow/) | Sentry's value comes from many coordinated services. Parallax should borrow the migration protocol, not the full service graph. |
| [SigNoz Docker install docs](https://signoz.io/docs/install/docker/) | Self-hosted SigNoz still requires Docker, 4 GB allocated memory, and open ports for UI and OTLP. Its install path is simpler than Sentry but still observability-suite-shaped. |
| [SigNoz architecture docs](https://signoz.io/docs/architecture/) | SigNoz centers ClickHouse, a SigNoz OTel Collector, and a bundled SigNoz binary that includes frontend, API server, OpAMP, ruler, and alertmanager. That is a warning against accidentally building an alerting/dashboard suite before Parallax proves bundles. |
| [OpenObserve getting-started docs](https://openobserve.ai/docs/getting-started/) | OpenObserve has a credible single-node self-hosted binary/container path. Parallax's tiny tier must be at least this easy for its narrower job. |
| [OpenObserve architecture docs](https://openobserve.ai/docs/architecture/) | OpenObserve HA splits into router, querier, ingester, compactor, and alertmanager, with NATS and object storage. That supports Parallax's tiered design: scale-out is a later topology, not the first build. |
| [Bugsink Docker install docs](https://www.bugsink.com/docs/docker-install/) | Bugsink can start as one Docker container and is Sentry SDK compatible, but persistent Docker deployments need external database care. It pressures the Sentry-compatible migration wedge. |
| [Urgentry homepage/docs](https://urgentry.com/) | Urgentry markets tiny mode, binary install, Sentry SDK DSN migration, session replay, profiling, logs, and benchmark claims. It is source-available rather than open-source, but it raises the bar for "tiny Sentry replacement" simplicity. |
| [Traceway OTLP/AI/replay recheck](traceway-otlp-ai-replay-recheck.md), [embedded mode](https://docs.tracewayapp.com/learn/embedded-mode), and [SQLite mode](https://docs.tracewayapp.com/server/sqlite) | Traceway can run an embedded development server inside a Go process and can also run a single SQLite container with two SQLite files plus blobs under `/data`. This pressures Parallax's local developer experience and tiny-tier persistence story. |
| [Traceway all-in-one container](https://docs.tracewayapp.com/server/all-in-one) and [Docker signatures](https://github.com/tracewayapp/traceway/blob/main/DOCKER_SIGNATURES.md) | Traceway offers one container bundling backend, ClickHouse, and Postgres, plus signed image variants. Even when internals are heavier, competitors can hide setup complexity behind one artifact. Parallax should count both services and hidden bundled subsystems and should not turn image-size/docs claims into measured performance claims. |

## What A7 Measures

A7 measures buildability, not desirability. A feature can be strategically
valuable and still fail A7 for the current phase.

Count these as scope additions:

| Addition type | Examples |
| --- | --- |
| Required service | Broker, collector, cache, object store, database, MCP server, worker pool, browser replay processor. |
| Protocol surface | Sentry envelope subset, OTLP HTTP, OTLP gRPC, Prometheus remote write, MCP tools, webhook ingest. |
| Signal family | Errors, spans, logs, metrics, deploys, issue trackers, CLI invocations, agent sessions, frontend sessions, database evidence. |
| Storage/index choice | New database, new table family, new object-store mode, new full-text/vector index, new retention tier. |
| Runtime language | Python/Go sidecar, JS service, JVM component, shell-out scanner in request path. |
| User-facing surface | CLI command, HTTP endpoint, UI view, MCP tool, exported schema artifact. |
| Autonomy level | Read-only context, hypothesis generation, patch proposal, PR creation, production action. |
| Operational promise | Backup/restore, upgrade, HA, multi-tenancy, SSO, alerting, billing, support tooling. |

The ledger should make scope drift visible before it becomes architecture.

## Ledger Artifacts

Every A7 review should produce or update:

```text
docs/research/scope-discipline-results.md
docs/research/scope-discipline-runs/<run_id>/manifest.json
docs/research/scope-discipline-runs/<run_id>/component-inventory.jsonl
docs/research/scope-discipline-runs/<run_id>/dependency-ledger.jsonl
docs/research/scope-discipline-runs/<run_id>/feature-intake.jsonl
docs/research/scope-discipline-runs/<run_id>/interface-surface.jsonl
docs/research/scope-discipline-runs/<run_id>/phase-gate-ledger.jsonl
docs/research/scope-discipline-runs/<run_id>/deferred-scope-ledger.jsonl
docs/research/scope-discipline-runs/<run_id>/milestone-review.jsonl
```

`scope-discipline-results.md` is the human summary. The JSONL files are the
auditable rows used when the roadmap changes, a new component is proposed, or a
milestone claims Phase 1 readiness.

## Phase Budgets

### Phase 0 Budget

Phase 0 is validation only:

| Budget item | Limit |
| --- | --- |
| Runtime services | 0 Parallax services required. |
| Required build work | Manual bundles, telemetry overlays, redacted result ledgers, interview artifacts. |
| Allowed code | Fixture generators, schema validators, evaluation scripts, local harness helpers. |
| Disallowed | Product UI, production storage work, MCP server, fixer, frontend replay, durable stream, scale-out topology. |

Phase 0 fails A7 if it becomes an excuse to build the product before A1/A2 are
tested.

### Phase 1 Tiny-Tier Budget

The Phase 1 product budget is intentionally narrow:

| Budget item | Limit |
| --- | --- |
| First useful command | `parallax issue context <issue-id>` returns one evidence bundle. |
| Required services | `parallax-server`, one storage process, and embedded/local metadata. Local WAL is a directory, not a service. |
| Service ceiling | Maximum 3 long-running services, matching the [self-hosted simplicity gate](self-hosted-simplicity-gate.md). |
| Required protocols | Scoped Sentry envelope ingest and scoped OTLP ingest. |
| Required surfaces | CLI plus canonical HTTP context API. Minimal UI is optional only if it does not delay the CLI/API proof. |
| Required signals | Error event, stack frames, release/environment, same-trace spans, nearby logs, missing-evidence report, redaction report. Metrics enter only if cheap through the same path. |
| Required safety | Default-deny redaction, read-only evidence API, no production mutation. |
| Required benchmark proof | A1 auto-bundle recheck, the minimum A4 correlation slice needed for the first bundle, A6 redaction fixture pass, and self-hosted simplicity pass. Full A5 stack roll-up remains a Phase 2 claim unless Phase 1 makes a stack-default claim. |
| Explicitly deferred | MCP, fixer PR workflow, frontend session replay, alerting suite, dashboard suite, SSO, billing, HA, Iggy/NATS/Redpanda default, production database query tools. |

Phase 1 fails A7 if it needs a broker, Postgres, ClickHouse, Redis, a Collector,
MCP, or a multi-service Parallax topology to produce the first issue context.

### Phase 2 Budget

Phase 2 can widen only after the tiny bundle works:

| Addition | Admission condition |
| --- | --- |
| Storage benchmark and fallback | A1/A2 have justified engineering spend; A5 ledger records current result. |
| Open schema/conformance | A3 ledger can count real conformance and external review rows. |
| Redaction red-team | A6 ledger can prove seeded fixtures and projections are safe enough. |
| CLI structural tracing | The CLI trace safety ledger proves the requested capture/redaction/overhead level and it improves bundle quality without adding agent autonomy. |
| Agent-session fixture harness | The agent-session tracing ledger reaches `fixture_harness_ready`; no generic product claim or default ingestion is admitted before per-surface rows exist. |
| Frontend error/correlation slice | A4 can measure strong edges and A6 can protect browser PII. |

Phase 2 still does not justify MCP-as-required, fixer autonomy, HA, enterprise
SSO, broad dashboards, or full session replay unless a gate row explicitly says
they unblock the next proof.

### Phase 3+ Budget

Scale-out and breadth are topology changes after the core contract works:

| Addition | Admission condition |
| --- | --- |
| Iggy/NATS/Redpanda | Ingest-log gate proves replay/backpressure need and setup cost. |
| Split workers | One process is bottlenecked by measured load, not by architecture preference. |
| Object storage default | Storage cost gate proves retained-size and reread economics. |
| MCP adapter | CLI/HTTP bundle API is stable, read-only safety is proven, and agent workflows need tool discovery rather than shell commands. |
| Coding-agent session tracing | Per-surface fixture rows pass for the exact tool/version/config and capture surface; a multi-agent claim requires at least one native OTel adapter and one non-OTel structured adapter. |
| Fixer component | A1 bundle value is green, A3 outcome rows exist, and A6 agent-exposure safety is green. |
| HA deployment | There is real adoption or workload pressure that needs it. |

## Row Schemas

### Feature Intake Row

```json
{
  "run_id": "a7-2026-05-25-phase1-scope",
  "feature": "read-only MCP adapter",
  "requested_by": "operator|research|user|competitor_pressure",
  "phase_requested": "phase1",
  "phase_allowed": "phase3",
  "user_job": "agent discovers and calls context tools",
  "gate_dependency": ["agent-access-surface-cli-api-mcp", "a6-redaction-red-team-ledger"],
  "new_services": 1,
  "new_protocols": ["mcp"],
  "new_runtime_languages": [],
  "scope_decision": "defer",
  "defer_until": "CLI/HTTP bundle API stable and read-only MCP safety green",
  "kill_or_delete_trigger": "if CLI satisfies agent workflows without protocol gap"
}
```

### Component Inventory Row

```json
{
  "run_id": "a7-2026-05-25-phase1-scope",
  "phase": "phase1",
  "component": "parallax-server",
  "required": true,
  "runtime_service": true,
  "language": "rust",
  "owns": ["ingest", "normalization", "bundle_api", "cli_projection"],
  "can_be_deferred": false,
  "split_trigger": "measured CPU/RSS or queue latency exceeds Phase 1 budget"
}
```

### Dependency Ledger Row

```json
{
  "run_id": "a7-2026-05-25-phase1-scope",
  "dependency": "opentelemetry-collector",
  "kind": "external_service",
  "required_for_phase1": false,
  "allowed_profile": "collector_compatibility",
  "replacement": "direct OTLP receiver",
  "risk": "turns tiny tier into a collector deployment",
  "decision": "defer_required_dependency"
}
```

### Milestone Review Row

```json
{
  "run_id": "a7-2026-05-25-phase1-scope",
  "milestone": "phase1-tiny-bundle",
  "required_services": 2,
  "required_protocols": ["sentry-envelope-subset", "otlp-http-subset"],
  "required_surfaces": ["cli", "http_context_api"],
  "deferred_features_count": 14,
  "budget_status": "green",
  "scope_exception_count": 0,
  "decision": "continue"
}
```

## Claim Levels

Use these exact claim levels in A7 summaries:

| Level | Meaning |
| --- | --- |
| `not_measured` | Scope has not been counted for the phase. |
| `narrative_only` | Roadmap wording exists but no inventory or budget rows exist. |
| `phase0_scope_green` | Phase 0 remains validation-only. |
| `phase1_tiny_scope_green` | Tiny tier is within service, protocol, dependency, and surface budgets. |
| `scope_budget_warning` | A feature or dependency is close to breaking the current phase budget. |
| `scope_budget_red` | Current phase exceeds service, protocol, dependency, or surface budget. |
| `scope_reset_required` | The project must cut/defer work before continuing the phase honestly. |
| `phase_n_scope_pass` | Later phase scope is within its explicitly approved gate rows. |

## Admission Rules

A feature enters the active build only if all of these are true:

1. It names the phase it belongs to.
2. It names the user job or proof gate it unlocks.
3. It lists new required services, protocols, storage modes, runtime languages,
   user-facing surfaces, and safety promises.
4. It has a delete/defer trigger.
5. It does not violate the current phase budget.

Default decisions:

| Proposed addition | Default A7 decision |
| --- | --- |
| MCP server in Phase 1 | Defer. CLI/HTTP first. |
| Fixer PR workflow in Phase 1 | Defer. Parallax stores and serves evidence; fixer is separate and later. |
| Generic coding-agent tracing in Phase 1 or Phase 2 | Defer product ingestion/claim. Phase 2 may prepare fixture harnesses; Phase 3+ requires per-surface ledger rows. |
| Frontend session replay in Phase 1 | Defer. Browser PII and replay storage are A4/A6/retention problems. |
| Iggy/NATS/Redpanda in tiny tier | Defer unless local WAL fails an ingest-log gate required for the first bundle. |
| Postgres required in tiny tier | Reject unless embedded/local metadata fails and the product claim is narrowed. |
| ClickHouse required in tiny tier | Reject unless A5 proves GreptimeDB cannot satisfy the profile and the simplicity claim is revised. |
| External Collector required in tiny tier | Reject unless OTLP direct receiver is impossible for the scoped subset. |
| Dashboard/alerting suite before CLI bundle | Reject. It competes with Sentry/SigNoz/OpenObserve on their broad product plane. |

## Warning And Kill Triggers

| Trigger | Consequence |
| --- | --- |
| Required Phase 1 service count exceeds 3. | A7 moves to `scope_budget_red`; cut or defer. |
| First useful bundle requires MCP, external Collector, broker, Redis, or Postgres. | Reopen the tiny-tier design. |
| More than one runtime language is required in the request path. | Justify with a gate row or remove it. |
| A milestone spends material effort on Phase 3+ features before Phase 1 gates pass. | Move to `scope_budget_warning` or `scope_reset_required`. |
| A feature lacks a delete/defer trigger. | It cannot enter active scope. |
| The CLI/API bundle contract is not stable, but UI/MCP/fixer work begins. | Stop that work and return to bundle contract. |
| "Agent tracing" enters a milestone without naming capture surface, tool version/config, lossiness, redaction, projection, and audit-value rows. | Move to `scope_budget_warning`; split or defer the milestone. |
| Self-hosted setup exceeds 15 minutes or needs hidden tribal knowledge. | A7 and the self-hosted simplicity gate both fail. |
| Competitor pressure causes feature copying without a Parallax proof gate. | Record as rejected or deferred; do not silently expand scope. |

## Relationship To Other Research

- [Risks and bear case](risks-and-bear-case.md) owns assumption A7. This ledger
  supplies the scope-counting contract.
- [Build roadmap and validation sequence](build-roadmap-and-validation-sequence.md)
  supplies the phase order A7 enforces.
- [Self-hosted simplicity gate](self-hosted-simplicity-gate.md) supplies the
  deployment/service-count budget for the tiny tier, while the
  [Self-hosted simplicity ledger](self-hosted-simplicity-ledger.md) defines the
  clean-VM run rows and claim levels.
- [A5 stack decision ledger](a5-stack-decision-ledger.md) owns stack-default
  claims; A7 owns whether those stack choices keep the product buildable.
- [Agent access surface: CLI, HTTP API, and MCP](agent-access-surface-cli-api-mcp.md)
  is the main source for deferring MCP until the CLI/API contract is proven.
- [Agent session tracing across real tools](agent-session-tracing-real-tools.md)
  and [Agent session tracing ledger](agent-session-tracing-ledger.md) are the
  main sources for treating agent-session tracing as a per-tool,
  per-capture-surface fixture program rather than a single feature.
- [CLI trace safety ledger](cli-trace-safety-ledger.md) owns the admission gate
  for CLI structural capture, redacted excerpts, raw refs, child-process policy,
  and projection safety.
- [Fixer component and outcome loop](fixer-component-and-outcome-loop.md) keeps
  fix PR work outside the Parallax core until the evidence contract and outcome
  loop are ready.
- [Frontend collection and cross-tier correlation](frontend-collection-and-cross-tier-correlation.md)
  is a later expansion unless A4/A6 prove it can be narrow and safe.

## Bottom Line

A7 is the execution counterpart to the technical ledgers. The safe claim is not
"Parallax can build everything in the prompt." The safe claim is:

> Parallax is buildable only if Phase 1 is treated as one narrow evidence-bundle
> product: scoped Sentry/OTLP ingest, deterministic grouping/correlation, local
> WAL, one storage candidate, embedded/local metadata, CLI/API bundle access,
> and redaction. Everything else must earn admission through a later gate row.

If that discipline slips, the bear case wins even if the architecture is right.
