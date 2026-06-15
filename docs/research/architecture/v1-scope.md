# V1 Scope: Everything Required for the Self-Sufficient Local Machine

<!-- markdownlint-disable MD013 -->

Research date: 2026-06-12. Operator statement #6 recorded 2026-06-12: **V1 solves the
local-development problem, completely and self-sufficiently, on one machine.** The server
profile moves to V2. This note is the exhaustive V1 inventory — what ships, what is deliberately
out, and everything required to make the local experience need nothing beyond the laptop.
**Revised same day for statement #7: the web UI is V1 scope** (Sentry-grade issues, service +
**custom** dashboards, trace lookup, full interactivity — specified in
[simple-ui-v2.md](simple-ui-v2.md)), and the capture targets are the operator's concrete stack —
Ratatui TUI, bollard Docker CLI, tonic gRPC microservices, axum HTTP, Juniper GraphQL +
DataLoaders, **tokio-postgres** and the official **`clickhouse`** crate, Redis, lapin/RabbitMQ —
with the verified emission matrix in
[capture/rust-stack-instrumentation.md](../capture/rust-stack-instrumentation.md).
Milestone mapping: **V1 = M0 + M1 + M2 + the UI milestone, plus the packaging/self-sufficiency
slice**; M3+ (server/cloud profiles) becomes the V2 line.

> **V1 in one sentence.** A developer installs one tool, runs one command, points any app at it
> with standard OTel env vars, and from that moment every run, panic, log, trace, and metric on
> their machine is captured, grouped, and servable as a bounded evidence bundle their coding
> agent reads through the CLI — with no network, no account, no Docker, no second system to
> operate.

## 1. Acceptance (the dogfood test, unchanged)

The operator connects one of his real Rust services locally with OTLP/gRPC
endpoint/protocol env vars plus the resource-attribute conventions; a real
panic appears as a grouped issue with trace + logs + metric window within ~5
seconds; `parallax issue context <id>` yields the bundle; his agent fixes the
bug from that context alone; cold install → first evidence in under 15
minutes. Plus the lifecycle-3 loop: when evidence is missing, the bundle's
`missing_evidence` + instrumentation suggestions tell the agent what to add.

Statement #7 adds the stack-shaped scenarios V1 must pass on the operator's real systems:

1. **Cross-service gRPC trace**: a request crossing two tonic microservices (and an axum edge)
   arrives as one trace; the UI waterfall shows both services; `parallax trace inspect
   <trace_id>` returns it.
2. **Database visibility**: tokio-postgres and `clickhouse` wrapper spans show query text,
   operation, and duration in the waterfall span detail.
3. **GraphQL visibility**: Juniper operation + resolver spans (and a DataLoader batch span)
   answer "how each part was generated and how long".
4. **The visual→agent handoff**: the operator sees an error trend in the UI, clicks into the
   window, opens one event's trace, copies the trace/run ID, and hands it to the agent — which
   pulls the full picture via the CLI.
5. **Custom dashboard**: a metric the operator's app emits is composed into a saved custom
   dashboard page and renders historically.
6. **TUI run**: a `parallax run start -- jackin …` session captures the run with OTLP-only
   logging (no stdout corruption).

## 2. In scope — the V1 inventory

### 2.1 Install and self-sufficiency

| Item | V1 answer |
| --- | --- |
| Install | `brew install tailrocks/tap/parallax` and a static binary download; `cargo install` for Rust users. One binary. |
| GreptimeDB acquisition | Self-sufficient by default: `parallax serve` detects `greptime` on PATH or in `~/.parallax/bin/`; if absent, offers to download the **pinned** release binary (checksum-verified) into `~/.parallax/bin/` — no Docker, no manual setup. Escape hatch: `--greptime-url` (bring your own GreptimeDB). **No fallback engines** (operator, 2026-06-12): GreptimeDB + Turso are the mandatory stack; `storage.mode = "none"` (in-memory) exists for tests/dev harnesses only and is not a supported product mode. |
| Offline | Everything works with zero network after install (the only network feature is the optional engine download). No phone-home, no telemetry-about-telemetry, no account. |
| Data layout | `~/.parallax/`: `bin/` (managed engine), `greptime-data/`, `meta.db` (Turso), `spool/` (ingest WAL), `config.toml`. One directory to back up or delete. |
| Uninstall | `parallax uninstall --purge` removes the data dir; the binary removal is the package manager's job. Nothing else was installed. |
| Diagnostics | `parallax doctor`: engine child health, port conflicts, data-dir permissions, version pins, spool backlog — the self-sufficiency tool when something is off. |

### 2.2 Ingest (the front door)

- OTLP/gRPC `:4317` and OTLP/HTTP-protobuf `:4318` — traces, logs, metrics
  ([conformance profile](../capture/otlp.md)). Loopback bind by default; `--bind` to open it to
  a LAN (no auth in V1 — local profile is single-user by definition).
- Both exception encodings accepted (span `exception` events and exception-as-log records).
- Resource-attribute conventions per the [integration contract](integration-contract.md);
  `parallax.run.id` recognized for run scoping.
- No Sentry endpoint, no browser-specific handling (a browser SDK posting OTLP/HTTP may work,
  but CORS/CSP support is explicitly best-effort in V1).

### 2.3 Processing (what V1 computes)

- Error-event derivation → deterministic fingerprinting → **issues** in the metadata store
  (first/last seen, count, status open/resolved).
- Correlation by `trace_id`/time window; span topology (`span_child_of`); CLI invocations from
  `process.command_line` spans.
- Rollups: per-(fingerprint, minute) counters — powers frequency display and keeps queries off
  the raw firehose. (Trigger *machinery* — dispatch, budgets — is **not** in V1; an issue simply
  exists and is listed. `new_fingerprint` marking comes free from grouping.)
- Bundle assembly on demand: bounded (10K-token default), redaction-lite (the detector set,
  honestly labeled pre-A6), ranked evidence-cited hypotheses, `missing_evidence`,
  instrumentation suggestions, canonical hash. JSON, Markdown, and terminal projections.

### 2.4 The run model (the local UX centerpiece)

```bash
parallax run start -- cargo test        # wrapper mode: assigns run_id, injects
                                        # OTLP/gRPC env + parallax.run.id, captures
                                        # exit code as a cli_invocation, ends the run
parallax run start                      # bare mode: prints the exports to source
parallax run list
parallax run inspect <run_id>           # everything the run produced
parallax run bundle <run_id>            # the run-anchored bundle
```

Wrapper mode is the centerpiece: one prefix turns any command — a test run, a TUI session, a
service under development — into a bounded, inspectable evidence unit (the Jackin lifecycle).

### 2.5 The CLI surface (V1 commands, complete list)

```text
parallax serve                # the server (local profile only in V1)
parallax doctor               # diagnostics
parallax prune                # apply TTL / free space now
parallax run start|list|inspect|bundle
parallax issue list [--run]   # grouped errors, counts, first/last seen
parallax issue context <id>   # the bounded bundle (json|markdown|term)
parallax issue resolve <id>
parallax trace inspect <trace_id>
parallax logs --trace <id> | --run <id> [--grep]
parallax metrics --run <id> [--name]
parallax uninstall --purge
```

The CLI talks only to the local API (GraphQL/HTTP on `:4000`); `--context` plumbing exists but
V1 ships with the implicit `local` context only.

### 2.5a The web UI (statement #7 — V1 scope)

Specified in [simple-ui-v2.md](simple-ui-v2.md); served by `parallax serve` as an embedded
static build (one binary, one URL, **no auth in V1**). The V1 pages: **Issues** (Sentry-grade
list + detail: trend sparkline, events count, age/first/last seen, tags, stack trace, message,
context, SDK/dependency info); **Dashboards** — predefined service overview (historical CPU/
memory, HTTP/gRPC rate + latency percentiles, error rate) **plus user-defined dashboards built
from any metric the apps send** (metric picker → aggregation → chart type → saved grid;
statement #7b); **Traces** — lookup by pasted `trace_id` and by `run_id`, cross-service
waterfall with span attributes (`db.query.text`, resolver/DataLoader spans); **Logs** (object
view); **Runs**. Interactivity rule: every chart is a filter, every entity links to its
neighbors (error chart → window → events → trace → span → logs). Stack rules: TanStack Start +
shadcn/ui **on Base UI**, default theme as-is, shadcn charts (Recharts v3) and blocks reused
wholesale.

### 2.6 The API (V1 surface)

GraphQL read API (runs, issues, traces, logs, metric windows, bundles) with depth/complexity/
pagination limits per [api-concept.md](api-concept.md); OTLP write; health/version. This is the
same API agents may hit directly and the V2 UI/server build on — nothing V1-specific to throw
away.

### 2.7 Retention (local defaults)

Telemetry TTL 7 days (per-signal configurable), issues and rollups kept until resolved+30d,
spool bounded; `parallax prune` for immediate reclaim. Disk is the laptop's — defaults stay
small and visible (`parallax doctor` reports usage).

### 2.8 Documentation shipped with V1

Quickstart (install → serve → connect a Rust app → first bundle, mirroring the integration
contract's Rust snippet); the CLI reference; the agent how-to ("point your coding agent at
`parallax issue context`"); the conventions page (resource attributes, run_id, exception
encodings).

## 3. Explicitly out of V1 (and where each went)

| Out | Where it lives |
| --- | --- |
| Server + cloud profiles, tokens/auth, remote contexts | V2 — build plan M3 ([deployment map](deployment-architecture-map.md) angles B/C) |
| ~~Web UI~~ | **Moved into V1** by statement #7 (§2.5a); only the fix-review screen stays deferred with the fixer rails |
| MCP adapter | Gated ([agent-access-surface.md](../decisions/agent-access-surface.md)); CLI is the V1 agent path |
| Sentry envelope ingest | Future adapter ([sentry-ingest.md](../capture/sentry-ingest.md)) |
| Trigger/dispatch machinery, autonomy budgets, fixer rails, outcome ledger | Deferred nice-to-have (statement #5); schemas stay versioned |
| Deploy-event ingestion, GitHub webhooks | V2 with the server profile (local deploys are rare; `vcs.*` resource attrs still captured) |
| Browser/frontend first-class capture | Roadmap ([frontend.md](../capture/frontend.md)); V1 is backend/CLI Rust apps first |
| Multi-user, multi-tenant anything | V2+ |

## 4. Everything required — the build checklist

1. **Crates** (per the [build plan](v1-build-plan.md) layout): `parallax-server`, `parallax-core`
   (graduating the 21 PoC kernels), `parallax-storage` (greptime + turso + memory adapters),
   `parallax-api`, `parallax-proto`, `parallax-cli` — plus the `ui/` TanStack Start app embedded
   into the server build.
0. **The concrete contracts** — [v1-implementation-spec.md](v1-implementation-spec.md):
   storage DDL (GreptimeDB + Turso), the OTLP→column mapping, the GraphQL SDL, ports (managed
   GreptimeDB child shifted to 24000–24003 to avoid the :4000 collision), `config.toml` keys,
   pinned dependencies, the supervision contract, workspace conventions. The implementing
   agent's entry point is [prompts/v1-implementation.md](../../../prompts/v1-implementation.md).
1a. **Capture documentation for the operator's stack** — the
   [instrumentation matrix](../capture/rust-stack-instrumentation.md): one shared telemetry-init
   pattern; middleware picks for tonic/axum; the manual-wrapper recipes for tokio-postgres,
   `clickhouse` (with its 0.15+ trace-context propagation feature), redis, lapin, Juniper
   resolvers and DataLoaders; TUI OTLP-only logging; the version-lockstep table maintained per
   release.
2. **Engine supervision**: spawn/health/restart of `greptime standalone`, pinned version,
   checksum-verified auto-download, PATH/brew detection. (Top V1 risk — first M1 spike.)
3. **Packaging**: brew tap (`tailrocks/tap`), static release binaries (macOS arm64 first — the
   operator's machine — then Linux x86_64), `cargo install` path.
4. **Fixtures→SDK tests**: integration tests driven by real `tracing` + `opentelemetry-otlp`
   emission (replacing hand-written OTLP JSON), which doubles as the A1 overlay generator.
5. **Docs** from §2.8.
6. **Gate rows V1 must produce** (M5 slice, measured on the local profile): setup <15 min,
   ingest-to-queryable ≤5 s p95, bundle assembly ≤300 ms warm, zero canary leaks from the
   redaction-lite set on its fixtures. **Measured 2026-06-12 — all four pass**; numbers,
   reproduction commands, and caveats in the [gates report](v1-gates-report.md).

## 5. V1 risks

| Risk | Mitigation |
| --- | --- |
| GreptimeDB child-process UX (download size, startup time, port clashes, upgrade) | M1 spike; `doctor`; pinned versions; supervision is fix-forward — no fallback engine (operator, 2026-06-12) |
| Turso beta under crash/concurrent CLI+server access | single-writer discipline through the server process; turso pitfalls regression-tested (open-statement write loss); fix-forward/upstream — no fallback engine (operator, 2026-06-12) |
| Wrapper-mode env propagation across build tools (cargo test subprocesses) | wrapper sets env + documents per-tool notes; bare mode as fallback |
| Redaction-lite over-claiming | label it pre-A6 everywhere it surfaces; A6 remains the gate before any agent-visible-by-default posture changes |
| Laptop disk pressure | small TTL defaults, bounded spool, visible usage in `doctor`, `prune` |

V1 done = the operator's daily development runs through it. That is the whole definition.
