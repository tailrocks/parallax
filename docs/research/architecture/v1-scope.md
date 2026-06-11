# V1 Scope: Everything Required for the Self-Sufficient Local Machine

<!-- markdownlint-disable MD013 -->

Research date: 2026-06-12. Operator statement #6 recorded 2026-06-12: **V1 solves the
local-development problem, completely and self-sufficiently, on one machine.** The server
profile moves to V2. This note is the exhaustive V1 inventory — what ships, what is deliberately
out, and everything required to make the local experience need nothing beyond the laptop.
Milestone mapping: **V1 = M0 + M1 + M2 of the [build plan](v1-build-plan.md), plus the
packaging/self-sufficiency slice**; M3+ (server/cloud profiles) becomes the V2 line.

> **V1 in one sentence.** A developer installs one tool, runs one command, points any app at it
> with standard OTel env vars, and from that moment every run, panic, log, trace, and metric on
> their machine is captured, grouped, and servable as a bounded evidence bundle their coding
> agent reads through the CLI — with no network, no account, no Docker, no second system to
> operate.

## 1. Acceptance (the dogfood test, unchanged)

The operator connects one of his real Rust services locally with only
`OTEL_EXPORTER_OTLP_ENDPOINT` + the resource-attribute conventions; a real panic appears as a
grouped issue with trace + logs + metric window within ~5 seconds; `parallax issue context <id>`
yields the bundle; his agent fixes the bug from that context alone; cold install → first
evidence in under 15 minutes. Plus the lifecycle-3 loop: when evidence is missing, the bundle's
`missing_evidence` + instrumentation suggestions tell the agent what to add.

## 2. In scope — the V1 inventory

### 2.1 Install and self-sufficiency

| Item | V1 answer |
| --- | --- |
| Install | `brew install tailrocks/tap/parallax` and a static binary download; `cargo install` for Rust users. One binary. |
| GreptimeDB acquisition | Self-sufficient by default: `parallax serve` detects `greptime` on PATH or in `~/.parallax/bin/`; if absent, offers to download the **pinned** release binary (checksum-verified) into `~/.parallax/bin/` — no Docker, no manual setup. Escape hatches: `--greptime-url` (bring your own), `--no-greptime` (Turso-only bounded fallback). |
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
  `parallax.run_id` recognized for run scoping.
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
                                        # OTEL_* env + parallax.run_id, captures
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
| Web UI | V2 ([simple-ui-v2.md](simple-ui-v2.md)) — V1 is CLI/API + Markdown bundles |
| MCP adapter | Gated ([agent-access-surface.md](../decisions/agent-access-surface.md)); CLI is the V1 agent path |
| Sentry envelope ingest | Future adapter ([sentry-ingest.md](../capture/sentry-ingest.md)) |
| Trigger/dispatch machinery, autonomy budgets, fixer rails, outcome ledger | Deferred nice-to-have (statement #5); schemas stay versioned |
| Deploy-event ingestion, GitHub webhooks | V2 with the server profile (local deploys are rare; `vcs.*` resource attrs still captured) |
| Browser/frontend first-class capture | Roadmap ([frontend.md](../capture/frontend.md)); V1 is backend/CLI Rust apps first |
| Multi-user, multi-tenant anything | V2+ |

## 4. Everything required — the build checklist

1. **Crates** (per the [build plan](v1-build-plan.md) layout): `parallax-server`, `parallax-core`
   (graduating the 21 PoC kernels), `parallax-storage` (greptime + turso + memory adapters),
   `parallax-api`, `parallax-proto`, `parallax-cli`.
2. **Engine supervision**: spawn/health/restart of `greptime standalone`, pinned version,
   checksum-verified auto-download, PATH/brew detection. (Top V1 risk — first M1 spike.)
3. **Packaging**: brew tap (`tailrocks/tap`), static release binaries (macOS arm64 first — the
   operator's machine — then Linux x86_64), `cargo install` path.
4. **Fixtures→SDK tests**: integration tests driven by real `tracing` + `opentelemetry-otlp`
   emission (replacing hand-written OTLP JSON), which doubles as the A1 overlay generator.
5. **Docs** from §2.8.
6. **Gate rows V1 must produce** (M5 slice, measured on the local profile): setup <15 min,
   ingest-to-queryable ≤5 s p95, bundle assembly ≤300 ms warm, zero canary leaks from the
   redaction-lite set on its fixtures.

## 5. V1 risks

| Risk | Mitigation |
| --- | --- |
| GreptimeDB child-process UX (download size, startup time, port clashes, upgrade) | M1 spike; `doctor`; pinned versions; `--no-greptime` keeps a degraded path |
| Turso beta under crash/concurrent CLI+server access | rusqlite feature-flag fallback; single-writer discipline through the server process |
| Wrapper-mode env propagation across build tools (cargo test subprocesses) | wrapper sets env + documents per-tool notes; bare mode as fallback |
| Redaction-lite over-claiming | label it pre-A6 everywhere it surfaces; A6 remains the gate before any agent-visible-by-default posture changes |
| Laptop disk pressure | small TTL defaults, bounded spool, visible usage in `doctor`, `prune` |

V1 done = the operator's daily development runs through it. That is the whole definition.
