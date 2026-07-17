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
- **✅🧪 shipped (pre-release)** — Parallax-specific: landed in code on `main` (verified against `crates/`), but the product is pre-release; "shipped" ≠ proven at scale.
- **🟡 inherited** — carried from legacy 2026-05/06 market notes, not yet re-verified against current sources. Treat as a hypothesis; see [`PROGRESS.md`](PROGRESS.md).

> **Parallax column verified (pass 15, 2026-07-17):** every Parallax cell below
> re-checked against shipped code (`crates/parallax-server`, `parallax-analysis`,
> `parallax-evidence`, `parallax-redaction`, `parallax-mcp-spike`). Corrections
> this pass: **Sentry envelope ingest is shipped** (`sentry_http.rs` router wired
> in `serve.rs`), not planned; **error derivation + test-result derivation are
> shipped** (`parallax-analysis::{derive,fingerprint,test_reporting}`); the
> **bounded redacted bundle exists in code** (`parallax-evidence::bundle` +
> `REDACTION_POLICY_V1`) but remains **A1-unproven**; **MCP is a spike crate
> only** (`parallax-mcp-spike`, separate binary) — not a shipped product surface.

---

## At-a-glance identity

| | Parallax | Datadog | Sentry | Grafana Cloud | Honeycomb | New Relic | SigNoz | OpenObserve | Coroot | Langfuse |
|---|---|---|---|---|---|---|---|---|---|---|
| **Category** | Execution-context engine | Full-stack SaaS obs+sec | Error-tracking + APM | Managed OSS stack | High-card events obs | Full-stack SaaS | OSS full obs | OSS Rust full obs | OSS eBPF obs+RCA | OSS LLM/agent obs |
| **License** | Apache-2.0 | Closed SaaS (OSS agent) | FSL (→Apache/MIT) | Mixed OSS + Cloud | Closed SaaS | Closed SaaS | MIT-Expat + `ee/` | AGPL-3.0 + EE | Apache-2.0 + EE | MIT + Cloud |
| **Telemetry store** | GreptimeDB | Proprietary closed | ClickHouse+Kafka | Mimir/Loki/Tempo/Py | proprietary | proprietary | ClickHouse | Parquet/DataFusion | ClickHouse+Prom | Postgres/sel. |
| **Self-hostable?** | ✅🧪 runs today (pre-release) | ❌ SaaS only | ✅ heavy (~40 ctnr) | ✅ (OSS bits) | 🟡 limited | ❌ | ✅ (~5 ctnr) | ✅ (1 binary) | ✅ (~5 ctnr) | ✅ |
| **Maturity** | 🏗 pre-release | ✅ incumbent | ✅ incumbent | ✅ incumbent | ✅ mature | ✅ incumbent | ✅ mature | ✅ mature | ✅ mature | ✅ mature |
| Deep-dive | — | [✅](parallax-vs-datadog.md) | [✅](parallax-vs-sentry.md) | [✅](parallax-vs-grafana.md) | [✅](parallax-vs-honeycomb.md) | [✅](parallax-vs-new-relic.md) | [✅](parallax-vs-signoz.md) | [✅](parallax-vs-openobserve.md) | [✅](parallax-vs-coroot.md) | [✅](parallax-vs-langfuse.md) |

The identity cells above (category / license / store / self-host / maturity) are **stable structural facts, confirmed by each product's deep-dive (verified 2026-07-17)** — they are *not* aging numbers, so the "🟡 inherited" caveat does **not** apply to them. Drift-sensitive figures (version, stars, pricing) live in each `parallax-vs-<product>.md`, pinned there with a date; **those** are what re-verification targets each pass, not the structural identity row.

## Feature-presence matrix

### Signals ingested

| Signal | Parallax | Datadog | Sentry | Grafana Cl. | Honeycomb | New Relic | SigNoz | OpenObserve | Coroot | Langfuse |
|---|---|---|---|---|---|---|---|---|---|---|
| Traces | ✅🧪 OTLP gRPC+HTTP | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | 🟡 eBPF partial | ✅ |
| Logs | ✅🧪 OTLP gRPC+HTTP | ✅ | ✅ | ✅ | 🟡 | ✅ | ✅ | ✅ | ✅ | 🟡 |
| Metrics | ✅🧪 OTLP gRPC+HTTP | ✅ | 🟡 | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ |
| Errors/exceptions | ✅🧪 derived `error_event` + fingerprint (spans+Sentry envelopes) | ✅ best-in-class issue workflow | ✅ best-in-class | ✅ | ✅ | ✅ | 🟡 span-events | 🟡 | 🟡 protocol | 🟡 |
| Continuous profiling | ❌ | ✅ | ✅ Vroom | ✅ Pyroscope | ❌ | ✅ | 🟡 | ❌ | ✅ eBPF | ❌ |
| RUM / session replay | ❌ | ✅ | ✅ | ❌ (3rd-party) | ❌ | ✅ | ❌ | ✅ | ❌ | ❌ |
| LLM / agent spans | 🟡🧪 agent-session + Claude Code modules; CLI-invocation program in flight | ✅ (Agent Obs) | 🟡 | ❌ | 🟡 | 🟡 | ✅ | 🟡 | ❌ | ✅ core |
| CI / test results | ✅🧪 test results derived from spans | ✅ (CI/Test Optimization) | 🟡 | ❌ | ❌ | 🟡 | ❌ | ❌ | ❌ | ❌ |

### Ingestion & transport

| Capability | Parallax | Datadog | Sentry | Grafana Cl. | Honeycomb | New Relic | SigNoz | OpenObserve | Coroot | Langfuse |
|---|---|---|---|---|---|---|---|---|---|---|
| OTLP ingest (any form) | ✅🧪 gRPC+HTTP, all 3 signals | ✅ (Agent + Managed OTLP) | 🟡 beta HTTP-only | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | 🟡 |
| OTLP-native storage | ✅🧪 GreptimeDB native tables | ❌ transforms to proprietary | ❌ | ✅ (own stores) | ❌ | ❌ | ❌ transforms to ClickHouse | 🟡 Parquet | ❌ ClickHouse | ❌ |
| Sentry envelope / DSN | ✅🧪 `sentry_http.rs` wired | ❌ | ✅ native | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ |
| eBPF zero-instrumentation | ❌ | ✅ (USM/CNM) | ❌ | ❌ | ❌ | 🟡 | ❌ | ❌ | ✅ | ❌ |

### Storage & cost behavior at scale

| Capability | Parallax | Datadog | Grafana Cl. | SigNoz | OpenObserve | Coroot |
|---|---|---|---|---|---|---|
| Columnar backend | ✅🧪 GreptimeDB | proprietary | ✅ | ✅ ClickHouse | ✅ Parquet | ✅ ClickHouse |
| Object-storage cold tier | 🟡🏗 (GreptimeDB supports object stores; Parallax deployment unproven) | ✅ Flex/Archive | ✅ | ❌ | ✅ native | ❌ |
| Cost model | open/self-host (no per-event tax) | per-host+per-metric+per-GB+per-span (complex) | per-series/GB | self-host or per-GB cloud | self-host or GB-day | self-host or per-node |
| Cost transparency | ✅🧪 (self-hosted compute) | ⚪ benchmark-dependent; widely cited as expensive & unpredictable | ⚪ benchmark-dependent | ⚪ bdd | ⚪ bdd | ⚪ bdd |

Cost/performance cells are **⚪ benchmark-dependent** — not filled until measured. Datadog's "expensive/unpredictable" reputation is widely documented by third parties (see Datadog deep-dive) but a specific Parallax-vs-Datadog number is unmeasured.

### AI-native / agent-context story (Parallax's wedge — fastest-moving axis)

| Capability | Parallax | Datadog | Sentry | SigNoz | OpenObserve | Coroot | Langfuse |
|---|---|---|---|---|---|---|---|
| Context engine for autonomous agents (bounded, redacted bundle) | 🟡🧪 bundle+redaction in code (`parallax-evidence`), **A1-unproven** | ❌ (human dashboard + chat) | ❌ | ❌ | ❌ | ❌ | ❌ |
| Read-only / safe-by-default agent projection | 🏗 MCP spike only (`parallax-mcp-spike`, not a product surface) | ❌ (write-capable management) | 🟡 | ❌ write/delete | ❌ write/delete default | 🟡 1 mutating tool | 🟡 |
| AI root-cause / investigation | 🏗 planned (no shipped AI RCA) | ✅ Bits AI (Investigation) | ✅ Seer autofix | ✅ MCP RCA skill | ✅ AI SRE | ✅ 2-stage ML+LLM | ❌ |
| AI pricing model | (self-hosted compute) | credit-metered ($500/500cr) | paid (Seer) | free (MCP) | Enterprise+BYO-key | Enterprise/Cloud | self-host or cloud |
| LLM/agent trace evals + experiments | 🏗 planned | ✅ (Agent Observability) | ❌ | ❌ | 🟡 | ❌ | ✅ core |

> **Honest read of this column:** on every *shipped* AI axis, Datadog, Sentry, SigNoz, Coroot, and Langfuse are ahead of pre-release Parallax. Parallax's only differentiated AI claim is the *bounded, redacted, agent-safe bundle* as a typed artifact — which is **unproven** (the A1 gate: does a bundle beat raw context for agent fix quality?). Do not read "✅🏗 planned" as parity with a shipped competitor feature.

### Architecture & deployment

| Capability | Parallax | Datadog | Sentry | SigNoz | OpenObserve | Coroot |
|---|---|---|---|---|---|---|
| Single binary, no Docker | 🟡🧪 `parallax-server` binary supervises GreptimeDB + embeds Turso | ❌ | ❌ | ❌ | ✅ | ❌ |
| Self-host free tier | ✅🧪 Apache-2.0, pre-release | ❌ (SaaS only) | ✅ heavy | ✅ | 🟡 10/50 GB-day EE | ✅ (AI gated) |
| Air-gapped / offline | ✅🧪 self-host, no phone-home | ❌ | ✅ | ✅ | ✅ | ✅ |
| Multi-tenancy / SSO-RBAC | 🏗 planned | ✅ best-in-class | ✅ | 🟡 | 🟡 | 🟡 |

### Security & compliance

| Capability | Parallax | Datadog | Sentry | SigNoz | OpenObserve |
|---|---|---|---|---|---|
| SSO/SAML/OIDC | 🏗 planned | ✅ (Enterprise) | ✅ | 🟡 | 🟡 |
| Fine-grained RBAC | 🏗 planned | ✅ | ✅ | 🟡 | 🟡 |
| PII scrub / redaction | 🟡🧪 bundle-path redaction shipped (`REDACTION_POLICY_V1`); ingest-time scrub = A6 gate | ✅ Sensitive Data Scanner | 🟡 server scrub | ❌ | 🟡 VRL (EE) |
| Compliance (SOC2/HIPAA/PCI) | ❌ not yet | ✅ SOC2/HIPAA/PCI | ✅ | ❌ self-attest | ❌ |
| Data ownership / lock-in cost | ✅ low (OSS self-host) | ❌ high (proprietary, SaaS) | 🟡 medium | ✅ low | ✅ low |

> Datadog's compliance posture is genuinely best-in-class here; Parallax has none of it yet. This is an axis where the incumbent plainly wins.

## What the matrix shows (no-bias read)

1. **On breadth, maturity, scale, enterprise readiness, and shipped AI features, the incumbents (Datadog especially) are far ahead of pre-release Parallax.** That is the plain reality; hiding it would defeat the purpose.
2. **Parallax's shipped-in-code (pre-release) surface is real but unproven:** OTLP ingest of all three signals into GreptimeDB native tables, Sentry-envelope ingest, derived error events + fingerprints, span-derived test results, and a bounded redacted evidence bundle (A1-unvalidated). **Planned-only:** MCP agent surface (spike crate), AI root-cause, evals, SSO/RBAC, fix-outcome loop. The defensible axes today are openness/self-hostability/cost transparency/data ownership; the bundle/outcome edge is gated behind A1.
3. **No product — open or closed — ships all of:** OTLP-native + (future) Sentry-envelope ingest + a portable, versioned, redacted evidence bundle + a read-only safe agent projection + a fix-outcome loop, from a telemetry store. That combination is Parallax's thesis, but "no one ships it" is not evidence it is valuable — that is exactly the A1 gate.
4. **The cells most likely to be wrong are the Parallax column (self-assessed, bias-prone) and the AI-native column (fast-moving).** Both are flagged for re-verification first on every pass.
5. **Parallax's agent-context thesis faces shipped pressure from *three* layers at once** — and must beat (or complement) all of them, not assume superiority:
   - **Telemetry-native-store layer:** [TMA1](parallax-vs-tma1.md) ships embedded-GreptimeDB + read-only MCP for coding agents (Parallax's own substrate, narrower scope).
   - **Reasoning/BYO-telemetry layer:** [Causely](parallax-vs-causely.md) ships causal-MCP grounding over *your* telemetry (no rip-and-replace) — the strongest shipped "agent-context layer."
   - **The fixer-agent itself:** [HolmesGPT](parallax-vs-holmesgpt.md) (CNCF, Apache) *is* the "separate agent that investigates telemetry" Parallax's "context engine, not the fixer" framing names — it could query Parallax as a richer source.
   The **A1 gate is the single crux against all three**: does a Parallax bounded/redacted bundle beat TMA1's live bundle, Causely's causal-MCP-over-your-telemetry, and HolmesGPT-investigating-raw-telemetry for coding-agent fix outcomes? **Unproven.** Honest framing: Parallax may be *complementary* to these (own+derive+bundle feeds them) rather than superior — and that is a defensible position only if the bundle measurably helps (A1).

## Full deep-dive roster (30 products + layers)

The wide matrix above is a readable 10-column slice; the authoritative roster
lives in [`comparison-set.md`](comparison-set.md). Every product with a
`parallax-vs-<product>.md` deep-dive:

- **Closed incumbents** — [Datadog](parallax-vs-datadog.md), [Sentry](parallax-vs-sentry.md), [Grafana Cloud](parallax-vs-grafana.md), [Honeycomb](parallax-vs-honeycomb.md), [New Relic](parallax-vs-new-relic.md), [Dynatrace](parallax-vs-dynatrace.md), [Splunk Obs](parallax-vs-splunk.md), [Elastic](parallax-vs-elastic.md), [Sumo Logic](parallax-vs-sumo.md), [Chronosphere](parallax-vs-chronosphere.md), [Observe](parallax-vs-observe.md), [Axiom](parallax-vs-axiom.md).
- **OSS / self-host platforms** — [SigNoz](parallax-vs-signoz.md), [OpenObserve](parallax-vs-openobserve.md), [Coroot](parallax-vs-coroot.md), [Highlight.io](parallax-vs-highlight.md), [Uptrace](parallax-vs-uptrace.md), [HyperDX](parallax-vs-hyperdx.md) *(ClickHouse Inc.'s ClickStack)*, [Odigos](parallax-vs-odigos.md), [Maple](parallax-vs-maple.md), **[TMA1](parallax-vs-tma1.md)** *(nearest architectural mirror)*.
- **AI / LLM-agent observability** — [Langfuse](parallax-vs-langfuse.md), [LangSmith](parallax-vs-langsmith.md), [Arize Phoenix](parallax-vs-arize-phoenix.md), [PostHog](parallax-vs-posthog.md), [Helicone](parallax-vs-helicone.md), [Braintrust](parallax-vs-braintrust.md).
- **Different-layer (causal / investigation / pipeline)** — **[Causely](parallax-vs-causely.md)** *(clearest shipped "agent-context layer" — BYO-telemetry causal MCP)*, [HolmesGPT](parallax-vs-holmesgpt.md) *(CNCF AI SRE)*, [Mezmo](parallax-vs-mezmo.md) *(pipeline)*.

> **Strategically central — read first:** [Causely](parallax-vs-causely.md) and
> [TMA1](parallax-vs-tma1.md) are the two clearest "context-layer-for-production-
> agents" bets alongside Parallax, from *opposite* layers (reasoning-layer-BYO-
> telemetry vs telemetry-native-store); [HyperDX](parallax-vs-hyperdx.md) is the
> ClickHouse Inc. "just use ClickHouse" counter to Parallax's GreptimeDB bet;
> [Sentry](parallax-vs-sentry.md) is the interop target. Watch status, open gaps,
> and the bias audit per product live in [`PROGRESS.md`](PROGRESS.md).

## Sources

Every deep-dive (all **30** products + layers, verified **2026-07-17**) carries its own dated primary-source list. The matrix above is backed by those deep-dives — **no cell relies on un-reverified legacy 2026-05/06 notes**; the legacy market notes are sources/leads only, with superseded-pointers into this folder. Drift-sensitive figures (version, stars, pricing) are pinned *per deep-dive* with a date; **those** are what re-verification targets each pass (products release; numbers age). See [`PROGRESS.md`](PROGRESS.md) for the per-product verification state, open questions, and the next-gap queue.
