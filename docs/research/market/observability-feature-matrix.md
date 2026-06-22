# Observability Feature Matrix — Cross-Tool Comparison

Research date: 2026-06-22

This is the **feature-by-feature cross-tool map**: which capability each platform provides and
which it does not. It complements (does not replace)
[competitive-comparison-matrix.md](competitive-comparison-matrix.md), which rates tools against
**Parallax's 8 wedge dimensions**. This matrix instead compares the **tools against each other**
across the full product surface, so you can see at a glance who has what.

Every cell is sourced from the per-tool standalone deep-dives — follow these for the evidence and
caveats behind each mark:

- [maple-deep-research.md](maple-deep-research.md)
- [signoz-deep-research.md](signoz-deep-research.md)
- [openobserve-deep-research.md](openobserve-deep-research.md)
- [coroot-deep-research.md](coroot-deep-research.md)
- [sentry-deep-research.md](sentry-deep-research.md)
- [gonzo-deep-research.md](gonzo-deep-research.md)

Broader, shallower coverage of 37+ tools lives in
[open-source-observability-tools-survey.md](open-source-observability-tools-survey.md).

## Legend

- **✅ Yes** — shipped and documented.
- **🟡 Partial** — beta, announced-but-incomplete, gated behind a paid tier, or a narrower form.
- **❌ No** — absent / not in category.
- **—** — not applicable.

"Parallax" column = the planned V1 target shape (no product yet), included as the reference design
the others are measured against.

## At-a-Glance Identity

| | Parallax | Maple | SigNoz | OpenObserve | Coroot | Sentry | Gonzo |
|---|---|---|---|---|---|---|---|
| **Category** | Evidence engine | Full obs platform | Full obs platform | Full obs platform | eBPF obs + APM | Error-tracking + APM | Log-tail TUI |
| **Primary language** | Rust | TypeScript (Bun) | Go + TS | Rust (engine) | Go | Python + Rust (Relay) | Go |
| **License** | Apache-2.0 | FSL-1.1 | MIT-Expat core / propr. `ee/` | AGPL-3.0 / commercial EE | Apache-2.0 / commercial EE | FSL (→Apache/MIT @2yr) | MIT |
| **Telemetry store** | GreptimeDB | ClickHouse (Tinybird/chDB) | ClickHouse | Parquet/object store (DataFusion) | ClickHouse + Prometheus | ClickHouse + Kafka | none (no store) |
| **Metadata store** | Turso (libSQL) | SQLite/Turso | SQLite/Postgres | SQLite/Postgres+NATS | not documented | Postgres | — |
| **GitHub stars (Jun 2026)** | — | ~0.4k | ~27.4k | ~19.4k | ~7.8k | incumbent (large) | ~2.7k |
| **Latest version** | pre-release | v0.0.11 | v0.129.0 | v0.90.3 / v0.91-rc | v1.22.2 | self-hosted 26.6.0 | v0.4.2 |
| **Funding** | — | early | YC + ~$6.5M | $10M Series A | Zaitsev-backed | ~$217M, ~$3B val | commercial Dstl8 |

## Ingest & Protocols

| Capability | Parallax | Maple | SigNoz | OpenObserve | Coroot | Sentry | Gonzo |
|---|---|---|---|---|---|---|---|
| **OTLP receiver (native, over the wire)** | ✅ | ✅ | ✅ | ✅ | ✅ | 🟡 beta, HTTP-only | 🟡 logs only |
| OTLP/gRPC | ✅ | ✅ | ✅ (4317) | ✅ (5081) | 🟡 unconfirmed | ❌ | ✅ (4317) |
| OTLP/HTTP | ✅ | ✅ (4318) | ✅ (4318) | ✅ (5080) | ✅ | 🟡 beta | ✅ (4318) |
| OTLP traces | ✅ | ✅ | ✅ | ✅ | ✅ | 🟡 beta | ❌ |
| OTLP logs | ✅ | ✅ | ✅ | ✅ | ✅ | 🟡 beta | ✅ |
| OTLP metrics | ✅ | ✅ | ✅ | ✅ | 🟡 Prom remote-write | ❌ no OTLP metrics | ❌ |
| **Sentry envelope / DSN ingest** | ✅ planned | ❌ | ❌ | ❌ | ❌ | ✅ native | ❌ |
| Prometheus scrape / remote-write | 🟡 | ❌ | ✅ metrics | ✅ PromQL | ✅ primary | ❌ | ❌ |
| eBPF zero-instrumentation capture | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ |
| stdin / file / k8s log tail | ❌ | ❌ | ❌ | ❌ | 🟡 (agent) | ❌ | ✅ |

## Signals & Storage

| Capability | Parallax | Maple | SigNoz | OpenObserve | Coroot | Sentry | Gonzo |
|---|---|---|---|---|---|---|---|
| Traces / distributed tracing | ✅ | ✅ | ✅ | ✅ | 🟡 partial eBPF spans | ✅ | ❌ |
| Logs | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ live only |
| Metrics / dashboards | 🟡 | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ |
| Continuous profiling | ❌ | ❌ | 🟡 | ❌ | ✅ eBPF | ✅ (Vroom) | ❌ |
| Session replay (RUM) | ❌ | ✅ | ❌ | ✅ | ❌ | ✅ | ❌ |
| Persistent store / retention | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ no retention |
| Object-storage-native (cheap tier) | 🟡 | 🟡 | ❌ | ✅ Parquet/S3 | ❌ | ❌ | — |

## Error Tracking & Workflow

| Capability | Parallax | Maple | SigNoz | OpenObserve | Coroot | Sentry | Gonzo |
|---|---|---|---|---|---|---|---|
| Error/exception capture | ✅ | ✅ | ✅ span-events | ✅ RUM/OTLP | 🟡 protocol-level only | ✅ best-in-class | 🟡 log parse |
| Deterministic grouping / fingerprinting | ✅ | 🟡 type/message | ❌ | ❌ | ❌ | ✅ | ❌ |
| Issue lifecycle (resolve/regress/ignore/assign) | 🟡 | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ |
| Ownership / triage routing | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ |
| 30+ language SDK fleet | 🟡 (via Sentry compat) | 🟡 OTLP SDKs | 🟡 OTLP SDKs | 🟡 OTLP SDKs | — (eBPF) | ✅ | — |

## Local-Run / Deployment

| Capability | Parallax | Maple | SigNoz | OpenObserve | Coroot | Sentry | Gonzo |
|---|---|---|---|---|---|---|---|
| **True single binary, no Docker** | ✅ | ✅ (Bun+chDB) | ❌ | ✅ (Rust) | ❌ | ❌ | ✅ (Go) |
| One-command local run | ✅ | ✅ | 🟡 ~5 containers | ✅ | 🟡 ~5 containers | 🟡 ~20–40 containers | ✅ |
| Disk-only / no external store | ✅ | ✅ | ❌ | ✅ default | ❌ | ❌ | ✅ |
| Approx. local RAM floor | low | low | ≥4 GB | <1 GB cited | heavier (CH+Prom) | 16–32 GB | tiny |
| Air-gapped / fully offline | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Self-host free tier | ✅ | ✅ | ✅ | 🟡 10/50 GB-day EE key | ✅ (AI gated) | ✅ (heavy ops) | ✅ |

## AI / Agent / MCP Surface

| Capability | Parallax | Maple | SigNoz | OpenObserve | Coroot | Sentry | Gonzo |
|---|---|---|---|---|---|---|---|
| **Official MCP server** | ✅ planned | ✅ 10+ tools | ✅ ~38 tools | ✅ 140+ tools | ✅ 18 tools | ✅ ~20 tools | ❌ (Dstl8 only) |
| Self-hostable MCP | ✅ | ✅ | ✅ | 🟡 Enterprise | ✅ | ✅ stdio | — |
| **MCP read-only / safe by default** | ✅ | 🟡 read-oriented | ❌ has write/delete | ❌ write/delete default | 🟡 1 mutating tool | 🟡 | — |
| Per-user RBAC-scoped agent | 🟡 | ❌ | 🟡 | 🟡 RBAC | ✅ OAuth+RBAC | 🟡 | — |
| AI root-cause / investigation | ✅ | 🟡 chat | ✅ skill (RCA) | ✅ AI SRE | ✅ 2-stage ML+LLM | ✅ Seer autofix | 🟡 per-entry |
| Autofix → opens PR | ✅ planned | 🟡 demo | ❌ | ❌ | ❌ | ✅ | ❌ |
| AI in free/OSS tier | ✅ | ✅ | ✅ MCP free | ❌ Enterprise+BYO-key | 🟡 10 free/mo cloud | 🟡 Seer paid | ✅ local LLM |
| Local LLM (Ollama/LM Studio) | 🟡 | ❌ | 🟡 BYO | 🟡 self-hosted | 🟡 OpenAI-compat | ❌ | ✅ |
| Coding-agent clients (Claude Code/Cursor/…) | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | 🟡 plugin |

## Evidence, Safety & Outcomes (the Parallax thesis dimensions)

| Capability | Parallax | Maple | SigNoz | OpenObserve | Coroot | Sentry | Gonzo |
|---|---|---|---|---|---|---|---|
| **Portable, versioned evidence-bundle schema** | ✅ | ❌ | ❌ (LLM markdown) | ❌ (auto-report) | ❌ (RCA on incident) | ❌ | ❌ |
| Redaction / PII scrub before agent access | ✅ A6 | ❌ | ❌ | 🟡 VRL (EE) | ❌ | 🟡 server scrub | ❌ |
| **Fix-outcome loop (accepted/rejected/reverted)** | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ |
| CLI / agent-session / CI capture | ✅ | ❌ | ❌ | ❌ | ❌ | 🟡 cron/uptime | ❌ |
| Deploy / change-context capture | ✅ | 🟡 commit SHA | 🟡 | 🟡 | ✅ deploy tracking | ✅ releases | ❌ |
| "Evidence/investigation" framing in product | ✅ | ❌ | 🟡 "open investigation format" | 🟡 "evidence chain" | 🟡 RCA on incident | 🟡 Seer RCA | ❌ |

## Platform Extras

| Capability | Parallax | Maple | SigNoz | OpenObserve | Coroot | Sentry | Gonzo |
|---|---|---|---|---|---|---|---|
| Service map / dependency graph | ❌ | ✅ | ✅ | 🟡 | ✅ | ✅ | ❌ |
| SLO tracking | ❌ | 🟡 | ✅ | ✅ | ✅ | ❌ | ❌ |
| Alerting / notifications | 🟡 | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ |
| Cron / uptime monitoring | ❌ | ❌ | 🟡 | ✅ | ❌ | ✅ | ❌ |
| Cost monitoring | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ |
| Kubernetes-native views | ❌ | ✅ Helm | ✅ | ✅ | ✅ DaemonSet | 🟡 | 🟡 |
| Data pipelines / transform | 🟡 | ❌ | 🟡 | ✅ VRL | ❌ | 🟡 | ❌ |
| LLM/AI-app observability | 🟡 | 🟡 explore | ✅ | ✅ | ❌ | 🟡 | ❌ |

## What This Map Shows

### Everyone is OTLP-native except the edges

Maple, SigNoz, OpenObserve, and Coroot are all genuine OTLP receivers (all three signals, gRPC+HTTP
for most). **Sentry's OTLP is a beta HTTP-only bolt-on (traces+logs, no metrics)** and **Gonzo's is
logs-only**. Only **Sentry** ingests the Sentry envelope — and **no OSS competitor ingests both
envelope and OTLP**, which is Parallax's first wedge.

### The single-binary local-first club is small

**Maple, OpenObserve, and Gonzo** ship a true single binary. **SigNoz (~5 containers, ≥4 GB),
Coroot (~5 containers), and Sentry (~20–40 containers, 16–32 GB)** require a Docker stack. Local-first
friction is a real axis of differentiation, and OpenObserve is the only single-binary tool that is also
a full-signal Rust backend.

### Error-workflow is Sentry's moat — and almost no one else's

Only **Sentry** has mature issue grouping + lifecycle (resolve/regress/ignore/assign) + a 30+ SDK fleet.
The OTLP-native platforms treat errors as queryable span-events, not managed work items. This is exactly
why Parallax plans **envelope compatibility** (absorb Sentry's SDKs) rather than competing on grouping UX.

### MCP is now table stakes — read-only safety is not

All six platforms ship an MCP surface (Gonzo's is the commercial Dstl8, not OSS). But **SigNoz and
OpenObserve expose write/destructive tools by default**; **Coroot has the best safety model** (per-user
OAuth + RBAC, 1 mutating tool). Parallax's strictly read-only, redacted, bounded projection is still
unoccupied ground — and AI is **gated/metered** on OpenObserve (Enterprise+BYO-key), Coroot
(Enterprise/Cloud), and Sentry (Seer paid), while free on SigNoz's MCP and Gonzo (local LLM).

### The thesis dimensions are empty across the board

**No tool** — open or incumbent — ships a portable versioned evidence-bundle schema, a fix-outcome loop,
or CLI/agent/CI session capture. The closest pressure is framing: SigNoz "open investigation format",
OpenObserve "evidence chain", Coroot RCA-on-incident, Sentry Seer — all **in-product workflows or
generated prose, not exportable validated artifacts**. This is the consistent finding across all six
deep-dives and the core of Parallax's remaining differentiation.

## How to Use This Doc

- **Picking a tool to run locally and study:** OpenObserve (Rust single-binary, closest architecture),
  Maple (best local UX, OTLP-only), Coroot (instant eBPF service map + safest MCP), SigNoz (most agent/MCP
  mature). Sentry only via the heavy self-hosted stack or the Spotlight dev overlay. Gonzo for live log
  triage, not as a backend.
- **Tracking competitive drift:** watch the **Evidence/Safety/Outcomes** table — the first OSS tool to
  flip any of those cells from ❌/🟡 to ✅ is the one that pressures the Parallax wedge. Per-tool watch
  triggers live in each deep-dive's Threat Assessment.
- **Keep it current:** when a deep-dive is refreshed, update the matching column here in the same change.
