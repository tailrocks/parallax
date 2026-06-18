# V1 Build Plan: From Concept to a Tool the Operator Uses Daily

<!-- markdownlint-disable MD013 -->

Research date: 2026-06-11. Operator statement #5 recorded 2026-06-11. This is the finalized
technical projection of **what gets built and how**, synthesizing the decided stack
([implementation-concept.md](implementation-concept.md)), the local-first shape
([local-first-v1.md](local-first-v1.md)), the API contract ([api-concept.md](api-concept.md)),
the outward conventions ([integration-contract.md](integration-contract.md)), and the proven
kernels ([poc-evidence-loop-coverage.md](poc-evidence-loop-coverage.md)).

> **Operator priority (statement #5).** Two goals, treated as co-equal top priorities, with the
> first slightly ahead: **Goal 1 — local-machine visibility** (the operator develops Rust tools
> daily and needs runtime visibility every time; he is user #1), and **Goal 2 — the same tool
> deployed on a server** to analyze his existing deployed Rust services. **Autonomous fixing is
> explicitly demoted to a future nice-to-have**: kept in mind (the schemas and contracts stay
> versioned so the loop remains reachable), but no fix-loop components are built until goals 1
> and 2 are achieved. The [north star](../00-vision/north-star-autonomous-fix-loop.md) is the
> ceiling, not the schedule.
>
> **Version boundary (statement #6, 2026-06-12):** **V1 = the self-sufficient local machine** —
> milestones M0–M2 plus the packaging/self-sufficiency slice, exhaustively inventoried in
> [v1-scope.md](v1-scope.md). The server profiles (M3) open **V2**. Goal 2 keeps its co-equal
> importance; it ships as the next version, not the first.
>
> **Statement #7 (same day) pulls the web UI into V1**: the UI slice of M4 (Issues, dashboards
> incl. user-defined, trace lookup, logs, runs — spec in [simple-ui-v2.md](simple-ui-v2.md))
> becomes a V1 milestone **M2.5-UI** between M2 and packaging; M4's detect-lite trigger
> machinery stays V2. Capture targets are the operator's concrete stack per
> [rust-stack-instrumentation.md](../capture/rust-stack-instrumentation.md) (tokio-postgres and
> the official `clickhouse` crate — not sqlx).

## 1. What done looks like (the two dogfood tests)

**Goal 1 acceptance — the local loop.** On the operator's laptop: `parallax serve` starts
everything (managed GreptimeDB standalone + embedded metadata). An existing Rust service is
pointed at it with nothing but standard OTel env vars and the
[required resource attributes](integration-contract.md). The service panics during development;
within ~5 seconds the panic exists as a grouped issue with its trace, surrounding logs, and
metric window; `parallax issue context <id>` returns the bounded bundle; the coding agent reads
it through the CLI and fixes the bug without the operator narrating context. Cold start to first
evidence: under 15 minutes (the [self-hosted simplicity gate](../validation/self-hosted-simplicity.md)).

**Goal 2 acceptance — the server loop.** The same binary runs on a VM with `--profile cloud`.
The operator's deployed Rust services send OTLP to it. From the laptop:
`parallax --context prod issue list` works exactly like kubectl against a cluster. A user
complaint reduces to a pasted trace ID (`parallax --context prod trace inspect <id>`), and the
full cross-service picture comes back (lifecycle 4 in
[problem-audience-product-shape.md](../00-vision/problem-audience-product-shape.md)).

**Explicitly deferred** (parked, schemas shipped, components unbuilt): dispatch/wake webhooks,
autonomy budgets in product, the Reconciler's outcome writing, the Learner, any fixer
integration, MCP (gated anyway), the Sentry envelope adapter.

## 2. Workspace layout

One Cargo workspace at the repository root when implementation starts (the research stage keeps
it under a new top-level `crates/` plus the existing `poc/`):

```text
crates/
  parallax-server/    # bin: the one binary. axum (HTTP API + OTLP/HTTP) + tonic (OTLP/gRPC),
                      # profile handling, managed-GreptimeDB supervisor, workers.
  parallax-core/      # domain logic: error derivation, fingerprinting, grouping, bundle
                      # assembly, bounding, redaction, hypotheses, rollups, triggers.
                      # Direct graduation target for the PoC kernels.
  parallax-storage/   # StorageAdapter trait + adapters: greptime (local standalone + server
                      # profiles), turso metadata (committed, no fallback engine),
                      # in-memory adapter for tests. No engine magic above this crate.
  parallax-api/       # async-graphql schema: Run, Issue, Trace, LogRecord, MetricWindow,
                      # EvidenceBundle; depth/complexity/pagination limits per api-concept.
  parallax-proto/     # OTLP types: opentelemetry-proto with the tonic codegen feature,
                      # version-pinned (M0 decides pin vs vendored prost output).
  parallax-cli/       # bin: clap. kubectl-style contexts (~/.parallax/config.toml: name, URL,
                      # token). Talks ONLY to the API. `parallax` = thin client + `serve`.
ui/                   # later (M4): TanStack Start + shadcn/ui app, separate from the workspace.
poc/evidence-loop/    # frozen as the concept reference; logic graduates by copy-and-adapt,
                      # the PoC itself stays runnable and unchanged.
```

The API-boundary rule is absolute from the first commit: CLI, UI, agents, and tests (except
adapter-level tests) never touch GreptimeDB/Turso directly.

## 3. Milestones

### M0 — Skeleton that receives real telemetry

Workspace scaffold; `parallax serve --profile local` starts axum+tonic; OTLP/gRPC :4317 and
OTLP/HTTP-protobuf :4318 accept traces/logs/metrics and spool to NDJSON (no storage engine yet);
health/version endpoints. Token-less in local profile by design.

*Exit:* one of the operator's real Rust services, configured with only standard
`OTEL_EXPORTER_OTLP_ENDPOINT` env vars, lands telemetry in the spool. *Decisions closed in M0:*
proto pinning strategy; tokio task layout for receivers.

### M1 — Storage and the error model

Managed GreptimeDB standalone supervisor (`--manage-greptime` default, `--greptime-url`
external mode, per [local-first-v1.md](local-first-v1.md)); Turso metadata file (committed,
no fallback engine — operator, 2026-06-12). **Storage (decided 2026-06-18, native-OTLP):** the adapter
**forwards raw OTLP straight to GreptimeDB's native tables** (`opentelemetry_traces`/`opentelemetry_logs`/metric
engine) and **tees** in-process to derive issues + run-scoped metrics into custom extension tables
(`error_events`, `rollups_fingerprint_minute`, `run_metric_points`); anchored reads by
trace_id/fingerprint/time window run over the native tables. See
[native-otel-tables.md](../decisions/native-otel-tables.md). Graduate from the PoC: error derivation
(both exception encodings), fingerprinting, grouping into issue rows (metadata store owns mutable issue
state — grouped errors are OLTP, per the metadata-store decision).

*Exit:* a real panic becomes a grouped issue queryable ~5s after emit (informal freshness check;
the formal gate row comes in M5). *Risk watched:* GreptimeDB process management UX (download/brew
detection, version pinning, crash restart) — first spike of the milestone.

### M2 — The evidence surface (Goal 1 lands here)

GraphQL read API over runs/issues/traces/logs/metric windows; bundle builder graduation
(assembly + token bounding + redaction-lite + ranked hypotheses + `missing_evidence`); the run
model (`parallax.run.id` resource attribute; `run start/list/inspect/bundle`); CLI v1
(`serve`, `run *`, `issue list/context`, `trace inspect`, `--context` plumbing with the local
default); JSON/Markdown/terminal bundle projections sharing one canonical hash.

*Exit:* **Goal-1 dogfood passes** — the operator's agent fixes a real bug in a real local
service from `parallax issue context` output alone. This is also where SDK-generated fixtures
replace hand-written ones in tests, feeding the A1 overlay tooling.

### M3 — The server profiles (Goal 2 lands here)

`--profile server` (own hardware: local-SSD GreptimeDB, Turso-or-Postgres metadata) and
`--profile cloud` (object-storage-backed GreptimeDB, managed-Postgres recommendation) ship as
presets of one server-side family — the full three-angle picture is the
[deployment architecture map](deployment-architecture-map.md). Concretely:
external GreptimeDB URL + object-storage-backed engine config passthrough,
per-project ingest tokens (`x-parallax-project-token`) and API tokens, Postgres as the metadata
option when Turso gates aren't met; deploy-events endpoint (`parallax.deploy.v0`); CLI remote
contexts (`parallax context add prod --url … --token …`); retention TTLs per signal.

*Exit:* **Goal-2 dogfood passes** — operator's deployed services report to a VM instance;
`parallax --context prod issue list` and trace-ID lookup work from the laptop; the
surface-the-trace-ID convention is exercised by at least one of his services.

### M4 — Detect-lite and the human window

Rollups (fingerprint counters per minute, written by the ingest worker); `new_fingerprint` and
`deploy_adjacent_regression` triggers create/escalate issues (**no dispatch** — detection ends
at the issue, per statement #5); frequency-spike flag on issues. UI first screens over the same
API: Issues, Issue detail, Trace waterfall, Logs object view, **Trace lookup** (the lifecycle-4
entry point). Fix-review screen waits for outcome data to exist (deferred with the fixer rails).

*Exit:* the operator stops opening five tools locally; trigger sanity-checked against replayed
local telemetry.

### M5 — Hardening and the first real gate rows

Measured runs replace informal checks: self-hosted simplicity (<15 min fresh machine), freshness
(≤5 s p95) and anchored-bundle latency (Q6 ≤300 ms) against the
[gate definitions](../storage/freshness-and-latency.md); OTLP conformance L1 fixtures (direct
Rust SDK); redaction pipeline v1 (the real detector library replaces the demo regexes — A6 prep,
not A6 pass); raw-ref TTL + evidence pinning for bundle-cited slices.

*Exit:* the first dated ledger rows in the repository come from running software, not design.

**Status 2026-06-12: measured.** The first gate rows are in
[v1-gates-report.md](v1-gates-report.md) — setup/freshness/bundle-latency/canary
all pass with large margins (`m5_gates` gated test reproduces them). The
remaining M5 items (real detector library replacing redaction-lite, raw-ref
TTL/evidence pinning) stay open.

### M6+ — Parked until goals 1+2 are achieved (operator statement #5)

Read-only MCP (after access-surface gates); Sentry envelope adapter (fixture-gated, breadcrumbs
being the named gap it fills); fixer rails — dispatch, outcome write-back, Reconciler, Learner,
autonomy budgets. Nothing in M0–M5 blocks them: the bundle schema, `fix_candidate.v0` draft, and
outcome contracts are already versioned, which is the whole cost of "keeping it in mind."

## 4. Cross-cutting rules

1. **One API, many clients** — enforced from M0.
2. **Engine portability** — all engine-specific SQL/config lives inside `parallax-storage`
   adapters; the four-build benchmark rule owns any performance claim.
3. **Claim discipline** — milestones produce ledger rows only from measured runs (M5 onward);
   building for the operator does not move A1/A2, which remain the *market* gates and proceed in
   parallel (the M2 bundle output is exactly the Arm-C generator A1 needs).
4. **Determinism where it matters** — canonical hashing and projection equivalence carry over
   from the PoC into `parallax-core` tests.
5. **No new surface without its safety note** — MCP, Sentry ingest, and anything write-shaped
   stay behind their existing gates.

## 5. Risks and open decisions

| Risk / decision | Where it bites | Plan |
| --- | --- | --- |
| GreptimeDB supervision UX (install, pin, restart, upgrade) | M1, goal-1 first impression | M1 spike; brew tap + direct download support; pin engine version per release |
| Turso beta behavior under crash/backup | M1/M3 metadata | fix-forward/upstream only — no fallback engine (operator, 2026-06-12); crash/backup gates stay as turso hardening checklist |
| OTLP proto types churn | M0 | pin `opentelemetry-proto`; vendor generated code if the crate's cadence hurts |
| GraphQL cost control | M2 | depth/complexity/pagination limits from api-concept enforced in `parallax-api` middleware from the first resolver |
| Cloud-profile auth UX (token issuance without a UI) | M3 | CLI-issued tokens stored in metadata; UI management later |
| v1.1 GreptimeDB GA lands mid-build | M1–M5 | adapter boundary absorbs it; re-pin + re-run the load-bearing benchmarks per the standing rule |

## 6. Why this order

Local first because the operator uses it the same week M2 lands and every milestone after that
is dogfooded against real daily work — the fastest possible feedback loop on the product's core
claim ("the agent stops asking you for context"). Server second because it is the same binary
with a profile, not a second product — which is precisely what the one-API/kubectl shape was
chosen for. Fixing last because the operator said so, and because everything the fix loop will
someday need (evidence quality, outcome schemas, deterministic bundles) is exactly what goals 1
and 2 harden anyway. The moonshot stays reachable; the tool becomes real first.
