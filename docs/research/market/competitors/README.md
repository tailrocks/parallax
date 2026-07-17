# Parallax vs Competitors — Unbiased Comparison

> Canonical comparison of Parallax to every relevant observability / debugging /
> investigation product on the market — open and closed source. **No pro-Parallax
> bias.** Where Parallax is behind, behind is written. Where a competitor is
> genuinely better, better is written. A comparison that always favors Parallax is
> a failure state.
>
> This is the overview. Per-product depth lives in
> [`parallax-vs-<product>.md`](.) and the roster lives in
> [`comparison-set.md`](comparison-set.md). The work queue / verification state
> lives in [`PROGRESS.md`](PROGRESS.md).

## How to read this folder

- **[`README.md`](README.md)** (this file) — the wide **feature-presence matrix**: which product has which capability. Kept readable; the matrix is the point.
- **[`parallax-vs-<product>.md`](.)** — one deep left/right comparison per product.
- **[`comparison-set.md`](comparison-set.md)** — the roster, one line per product.
- **[`PROGRESS.md`](PROGRESS.md)** — per-product/cell verification state and the next gap.
- **Legacy market notes** (`../competitive-comparison-matrix.md`, `../observability-feature-matrix.md`, `../closest-to-parallax-ranked.md`, `../<product>-deep-research.md`, …) are **sources**, not the destination. They lead; they do not settle. Where they disagree with current primary sources, the current source wins.

### No-bias rules (enforced on every pass)

1. Default assumption: **the competitor may be better until evidence says otherwise.** Do not start from "Parallax wins."
2. Parallax's own limitations (pre-release, missing/immature signals, GreptimeDB/Turso constraints) are stated plainly, not hidden.
3. "Better" is always scoped to a named axis with evidence — never a vague verdict.
4. Marketing language from any vendor is a lead, not a fact. Confirm against docs, source, changelogs, pricing pages, or measurement.
5. When a claim cannot be proven, mark it **unproven** and say what would prove it. Never present an unproven claim as settled.

### Real-numbers policy

Prefer hard, current, sourced numbers: pricing tiers, ingest throughput, query latency, retention defaults, cardinality limits. When a number exists, cite the source and date. When no public number exists, write that explicitly, give the best-grounded proxy, and note why a direct number is unavailable. Storage/ingest/query performance comparisons are **benchmark-dependent** and marked unproven until measured by the benchmark program — never fabricated to fill a cell.

### Matrix cell legend

- **✅ Yes** — shipped and documented.
- **🟡 Partial** — beta, announced-but-incomplete, gated behind a paid tier, or narrower form.
- **❌ No** — absent.
- **—** — not applicable.
- **🏗 planned** — Parallax-specific: designed but not yet shipped (do not read as "has it").
- **🟡 inherited** — carried from legacy 2026-05/06 market notes, not yet re-verified against current sources. Treat as a hypothesis; see [`PROGRESS.md`](PROGRESS.md).

---

## At-a-glance identity

| | Parallax | Datadog | Sentry | Grafana Cloud | Honeycomb | New Relic | SigNoz | OpenObserve | Coroot | Langfuse |
|---|---|---|---|---|---|---|---|---|---|---|
| **Category** | Execution-context engine | Full-stack SaaS obs+sec | Error-tracking + APM | Managed OSS stack | High-card events obs | Full-stack SaaS | OSS full obs | OSS Rust full obs | OSS eBPF obs+RCA | OSS LLM/agent obs |
| **License** | Apache-2.0 | Closed SaaS (OSS agent) | FSL (→Apache/MIT) | Mixed OSS + Cloud | Closed SaaS | Closed SaaS | MIT-Expat + `ee/` | AGPL-3.0 + EE | Apache-2.0 + EE | MIT + Cloud |
| **Telemetry store** | GreptimeDB | Proprietary closed | ClickHouse+Kafka | Mimir/Loki/Tempo/Py | proprietary | proprietary | ClickHouse | Parquet/DataFusion | ClickHouse+Prom | Postgres/sel. |
| **Self-hostable?** | ✅ (target) | ❌ SaaS only | ✅ heavy (~40 ctnr) | ✅ (OSS bits) | 🟡 limited | ❌ | ✅ (~5 ctnr) | ✅ (1 binary) | ✅ (~5 ctnr) | ✅ |
| **Maturity** | 🏗 pre-release | ✅ incumbent | ✅ incumbent | ✅ incumbent | ✅ mature | ✅ incumbent | ✅ mature | ✅ mature | ✅ mature | ✅ mature |
| Deep-dive | — | [✅](parallax-vs-datadog.md) | [✅](parallax-vs-sentry.md) | [✅](parallax-vs-grafana.md) | [✅](parallax-vs-honeycomb.md) | [✅](parallax-vs-new-relic.md) | [✅](parallax-vs-signoz.md) | [✅](parallax-vs-openobserve.md) | [✅](parallax-vs-coroot.md) | [✅](parallax-vs-langfuse.md) |

All identity cells except Datadog are **🟡 inherited** from legacy market notes (2026-05/06) — verify on each product's deep-dive pass.

## Feature-presence matrix

### Signals ingested

| Signal | Parallax | Datadog | Sentry | Grafana Cl. | Honeycomb | New Relic | SigNoz | OpenObserve | Coroot | Langfuse |
|---|---|---|---|---|---|---|---|---|---|---|
| Traces | ✅🏗 | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | 🟡 eBPF partial | ✅ |
| Logs | ✅🏗 | ✅ | ✅ | ✅ | 🟡 | ✅ | ✅ | ✅ | ✅ | 🟡 |
| Metrics | ✅🏗 | ✅ | 🟡 | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ |
| Errors/exceptions | ✅🏗 | ✅ best-in-class issue workflow | ✅ best-in-class | ✅ | ✅ | ✅ | 🟡 span-events | 🟡 | 🟡 protocol | 🟡 |
| Continuous profiling | ❌ | ✅ | ✅ Vroom | ✅ Pyroscope | ❌ | ✅ | 🟡 | ❌ | ✅ eBPF | ❌ |
| RUM / session replay | ❌ | ✅ | ✅ | ❌ (3rd-party) | ❌ | ✅ | ❌ | ✅ | ❌ | ❌ |
| LLM / agent spans | ✅🏗 | ✅ (Agent Obs) | 🟡 | ❌ | 🟡 | 🟡 | ✅ | 🟡 | ❌ | ✅ core |
| CI / test results | ✅🏗 | ✅ (CI/Test Optimization) | 🟡 | ❌ | ❌ | 🟡 | ❌ | ❌ | ❌ | ❌ |

### Ingestion & transport

| Capability | Parallax | Datadog | Sentry | Grafana Cl. | Honeycomb | New Relic | SigNoz | OpenObserve | Coroot | Langfuse |
|---|---|---|---|---|---|---|---|---|---|---|
| OTLP ingest (any form) | ✅🏗 | ✅ (Agent + Managed OTLP) | 🟡 beta HTTP-only | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | 🟡 |
| OTLP-native storage | ✅🏗 (GreptimeDB native tables) | ❌ transforms to proprietary | ❌ | ✅ (own stores) | ❌ | ❌ | ❌ transforms to ClickHouse | 🟡 Parquet | ❌ ClickHouse | ❌ |
| Sentry envelope / DSN | ✅🏗 planned | ❌ | ✅ native | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ |
| eBPF zero-instrumentation | ❌ | ✅ (USM/CNM) | ❌ | ❌ | ❌ | 🟡 | ❌ | ❌ | ✅ | ❌ |

### Storage & cost behavior at scale

| Capability | Parallax | Datadog | Grafana Cl. | SigNoz | OpenObserve | Coroot |
|---|---|---|---|---|---|---|
| Columnar backend | ✅🏗 GreptimeDB | proprietary | ✅ | ✅ ClickHouse | ✅ Parquet | ✅ ClickHouse |
| Object-storage cold tier | 🟡🏗 | ✅ Flex/Archive | ✅ | ❌ | ✅ native | ❌ |
| Cost model | open/self-host (no per-event tax) | per-host+per-metric+per-GB+per-span (complex) | per-series/GB | self-host or per-GB cloud | self-host or GB-day | self-host or per-node |
| Cost transparency | ✅🏗 (self-hosted compute) | ⚪ benchmark-dependent; widely cited as expensive & unpredictable | ⚪ benchmark-dependent | ⚪ bdd | ⚪ bdd | ⚪ bdd |

Cost/performance cells are **⚪ benchmark-dependent** — not filled until measured. Datadog's "expensive/unpredictable" reputation is widely documented by third parties (see Datadog deep-dive) but a specific Parallax-vs-Datadog number is unmeasured.

### AI-native / agent-context story (Parallax's wedge — fastest-moving axis)

| Capability | Parallax | Datadog | Sentry | SigNoz | OpenObserve | Coroot | Langfuse |
|---|---|---|---|---|---|---|---|
| Context engine for autonomous agents (bounded, redacted bundle) | ✅🏗 | ❌ (human dashboard + chat) | ❌ | ❌ | ❌ | ❌ | ❌ |
| Read-only / safe-by-default agent projection | ✅🏗 | ❌ (write-capable management) | 🟡 | ❌ write/delete | ❌ write/delete default | 🟡 1 mutating tool | 🟡 |
| AI root-cause / investigation | ✅🏗 | ✅ Bits AI (Investigation) | ✅ Seer autofix | ✅ MCP RCA skill | ✅ AI SRE | ✅ 2-stage ML+LLM | ❌ |
| AI pricing model | (self-hosted compute) | credit-metered ($500/500cr) | paid (Seer) | free (MCP) | Enterprise+BYO-key | Enterprise/Cloud | self-host or cloud |
| LLM/agent trace evals + experiments | ✅🏗 planned | ✅ (Agent Observability) | ❌ | ❌ | 🟡 | ❌ | ✅ core |

> **Honest read of this column:** on every *shipped* AI axis, Datadog, Sentry, SigNoz, Coroot, and Langfuse are ahead of pre-release Parallax. Parallax's only differentiated AI claim is the *bounded, redacted, agent-safe bundle* as a typed artifact — which is **unproven** (the A1 gate: does a bundle beat raw context for agent fix quality?). Do not read "✅🏗 planned" as parity with a shipped competitor feature.

### Architecture & deployment

| Capability | Parallax | Datadog | Sentry | SigNoz | OpenObserve | Coroot |
|---|---|---|---|---|---|---|
| Single binary, no Docker | ✅🏗 | ❌ | ❌ | ❌ | ✅ | ❌ |
| Self-host free tier | ✅🏗 | ❌ (SaaS only) | ✅ heavy | ✅ | 🟡 10/50 GB-day EE | ✅ (AI gated) |
| Air-gapped / offline | ✅🏗 | ❌ | ✅ | ✅ | ✅ | ✅ |
| Multi-tenancy / SSO-RBAC | 🏗 planned | ✅ best-in-class | ✅ | 🟡 | 🟡 | 🟡 |

### Security & compliance

| Capability | Parallax | Datadog | Sentry | SigNoz | OpenObserve |
|---|---|---|---|---|---|
| SSO/SAML/OIDC | 🏗 planned | ✅ (Enterprise) | ✅ | 🟡 | 🟡 |
| Fine-grained RBAC | 🏗 planned | ✅ | ✅ | 🟡 | 🟡 |
| PII scrub / redaction | ✅🏗 (A6 gate) | ✅ Sensitive Data Scanner | 🟡 server scrub | ❌ | 🟡 VRL (EE) |
| Compliance (SOC2/HIPAA/PCI) | ❌🏗 not yet | ✅ SOC2/HIPAA/PCI | ✅ | ❌ self-attest | ❌ |
| Data ownership / lock-in cost | ✅ low (OSS self-host) | ❌ high (proprietary, SaaS) | 🟡 medium | ✅ low | ✅ low |

> Datadog's compliance posture is genuinely best-in-class here; Parallax has none of it yet. This is an axis where the incumbent plainly wins.

## What the matrix shows (no-bias read)

1. **On breadth, maturity, scale, enterprise readiness, and shipped AI features, the incumbents (Datadog especially) are far ahead of pre-release Parallax.** That is the plain reality; hiding it would defeat the purpose.
2. **Parallax's real, defensible axes — all still partly *planned*, not shipped — are:** open-source/self-hostability, single-binary local-first simplicity, cost transparency and data ownership (no proprietary lock-in), and the *unproven* bounded-redacted-bundle + fix-outcome thesis. Only the openness/cost/ownership axes are real today; the bundle/outcome edge is gated behind A1.
3. **No product — open or closed — ships all of:** OTLP-native + (future) Sentry-envelope ingest + a portable, versioned, redacted evidence bundle + a read-only safe agent projection + a fix-outcome loop, from a telemetry store. That combination is Parallax's thesis, but "no one ships it" is not evidence it is valuable — that is exactly the A1 gate.
4. **The cells most likely to be wrong are the Parallax column (self-assessed, bias-prone) and the AI-native column (fast-moving).** Both are flagged for re-verification first on every pass.

## Sources

Every deep-dive carries its own dated source list. The matrix above inherits from the legacy market notes (dated 2026-05/06) and the verified Datadog deep-dive (2026-07-17). Re-verify before trusting any 🟡 cell; see [`PROGRESS.md`](PROGRESS.md) for the per-cell queue.
