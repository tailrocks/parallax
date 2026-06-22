# Gonzo Deep Research

Research date: 2026-06-22

Gonzo was previously a single shallow entry (#17) in
`open-source-observability-tools-survey.md`. This note promotes it to a standalone
assessment because it does support OTLP — but the headline finding is a **category
boundary**: Gonzo is a live log-tail TUI with an OTLP-logs receiver and optional AI,
**not a telemetry backend/store or an evidence-bundle platform**. It overlaps Parallax
only at the "local-first + OTLP + AI-native" framing and only on logs.

## Sources

- [github.com/control-theory/gonzo](https://github.com/control-theory/gonzo) — README, repo, releases
- GitHub API `api.github.com/repos/control-theory/gonzo` — stars/forks/license/latest tag
- [controltheory.com/gonzo](https://www.controltheory.com/gonzo/) and [controltheory.com](https://www.controltheory.com/) (commercial Dstl8)
- [controltheory.com/use-case/codex-mcp-logs](https://www.controltheory.com/use-case/codex-mcp-logs/) — Dstl8 MCP server (not Gonzo)
- [Show HN: Gonzo](https://news.ycombinator.com/item?id=45018113), [pkg.go.dev/.../gonzo](https://pkg.go.dev/github.com/control-theory/gonzo)

## What Gonzo Is

Gonzo is a **Go terminal UI (TUI) for real-time log analysis** — explicitly "k9s for logs."
It is the free OSS flagship of **ControlTheory**, a runtime-observability / "AI-SDLC
reliability" startup whose **commercial** product is **Dstl8** ("the runtime feedback loop
for the AI SDLC"). Gonzo is the free terminal companion / lead-gen; Dstl8 is the paid
backend (incidents, patterns, anomalies, knowledge graph, MCP server).

- **License:** MIT (more permissive than Parallax's Apache-2.0).
- **Language:** Go (≥1.21), ~71% Go + ~22% React/TS (bundled web dashboard). Repo created 2025-08-18.
- **Category:** **viewer/analyzer, not a backend or store.** README states it "does not store
  historical logs… built for live analysis and active investigation, not long-term retention."
- **Maturity:** ~2,696 stars, ~98 forks, ~19 contributors. Latest **v0.4.2, 2026-05-15**
  (v0.4.1 May 7, v0.4.0 May 6). ~10 months old, actively developed, fast cadence.

## What It Does

- Live, interactive log investigation in a k9s-style 2×2 TUI grid — charts, severity heatmaps,
  regex filtering, column picker, 11+ themes.
- Pattern detection via the **Drain3** algorithm (recurring-issue clustering).
- **Bundled web dashboard "Dstl8 Lite"** (v0.4.0, May 2026) — press `d` in the TUI to open the
  same view as a local web UI, all in one binary.
- **Input sources:** stdin pipe (`cat logs | gonzo`), files + globs with `-f` following, Kubernetes
  pod logs, Docker logs, network OTLP receiver, arbitrary piped commands (Vercel, CloudWatch, Loki,
  Victoria Logs).

## OTLP / OTel Support — the key nuance

- **Genuine OTLP receiver over the wire** — listens on gRPC :4317 + HTTP :4318 (`/v1/logs`), accepts
  exports from the OpenTelemetry Collector or apps directly (`gonzo --otlp-enabled`). Documented
  Collector exporter configs (`otlp/gonzo_grpc`, `otlphttp/gonzo_http`); can run as a sidecar/local
  proxy. This is a **real endpoint, not just an OTLP-JSON line parser** — though it also auto-detects
  and parses OTLP-formatted log lines from stdin/files.
- **Scope: logs only.** No metrics, no traces over OTLP. This is the defining category limit and the
  reason Gonzo is not a full-signal competitor.

## AI / LLM Surface

- **Providers:** OpenAI, Anthropic via **Claude Code CLI** (`--ai-provider=claude-code`), **Ollama**
  (local), **LM Studio** (local), any OpenAI-compatible API. Runtime model switching (`m` key).
- **What the AI does:** opt-in, per-entry — pattern detection, anomaly analysis, root-cause/debugging
  suggestions, summarize a selected entry. **Entirely optional**; core analysis runs with zero AI.
- **Fully local AI** possible via Ollama / LM Studio.
- **MCP — important distinction:** **OSS Gonzo is NOT an MCP server.** It is installable as a **Claude
  Code plugin** (`/plugin marketplace add control-theory/gonzo`) with a guided log-analysis skill.
  The **MCP server belongs to the commercial Dstl8 product**, not OSS Gonzo. Easy to conflate from
  marketing — keep them separate.

## Local-Run Story

- **Single self-contained Go binary** (web dashboard embedded). Install via `go install`,
  `brew install gonzo`, `nix run github:control-theory/gonzo`, prebuilt release binaries
  (darwin/linux amd64+arm64, windows amd64), or `make build`. Fully local-first; cloud only if you
  pick a cloud LLM.

## Strengths and Gaps (vs the Parallax wedge)

**Strengths:**
- Excellent live-triage UX (k9s familiarity), genuine OTLP-logs wire receiver, broad input sources,
  fully local AI option, single-binary install, strong early traction for its age (2.7k stars).

**Gaps / different category:**
- **Not a backend or store** — no persistence, no retention, no query-over-history, no telemetry
  datastore. Parallax's GreptimeDB + Turso storage stack and evidence bundles have no analogue.
- **Logs only** — no metrics, no traces. Parallax's multi-signal zero-copy ingest is out of scope.
- **No evidence-bundle / outcome-loop** concept; AI is per-entry summarization, not an evidence or
  outcome workflow.
- **No native MCP server in OSS** (that is the paid Dstl8 product); Gonzo's AI integration is a
  Claude Code plugin/skill.
- **No Sentry ingestion, no redaction layer.**

## Comparison: Gonzo vs Parallax

| Dimension | Gonzo | Parallax |
|-----------|-------|----------|
| **Category** | Live log-tail TUI / analyzer (no store) | Evidence context engine (store-backed) |
| **Persistence** | None — live only, no retention | GreptimeDB + Turso, bundles persisted |
| **Signals** | Logs only | Errors + OTLP traces/metrics/logs |
| **OTLP** | Receiver, logs only (gRPC/HTTP) | Full OTLP + Sentry envelope |
| **AI surface** | Per-entry summarize/analyze (optional) | Bounded redacted evidence bundles to agents |
| **MCP** | None in OSS (Dstl8 commercial has it) | CLI-first + read-only MCP after safety gates |
| **Evidence bundles / outcome loop** | None | Core thesis |
| **License** | MIT | Apache-2.0 |

## Verdict and Watch Note

Gonzo is a **complementary terminal investigation tool**, not a competitor to a full OTLP
backend / evidence-bundle / outcome-loop platform. It overlaps Parallax only on "local-first +
OTLP + AI-native" framing and only on logs, and it deliberately does not persist or store.

The strategically relevant comparison in ControlTheory's stable is the **commercial Dstl8**
(backend + MCP + knowledge graph + incidents/patterns), not OSS Gonzo. **Watch trigger:** track the
**Gonzo → Dstl8 funnel** — if Dstl8 ships portable evidence artifacts, Sentry ingestion, or an
outcome loop, re-assess Dstl8 (not Gonzo) as the real competitor. Gonzo's own watch trigger is
narrow: if it adds OTLP **traces/metrics** ingestion and persistence, it would cross from viewer into
backend territory.
