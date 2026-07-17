# Parallax vs Coroot

> An unbiased, one-to-one comparison. Research date: **2026-07-17** (pass 59:
> eBPF→app-errors **UNFIRED**; **pass 103** + **pass 144** MCP/AI re-verify —
> still **v1.23.3 / 7,837★**, last push **2026-07-02**). Sources:
> [coroot.com](https://coroot.com/), [MCP overview](https://docs.coroot.com/mcp/overview/),
> [github.com/coroot/coroot](https://github.com/coroot/coroot), legacy
> [coroot-deep-research.md](../coroot-deep-research.md).
>
> **Pass 144:** MCP docs still OAuth 2.0 per-user RBAC; tool table still has one
> clear Community mutator **`resolve_alerts`**; EE **`investigate_anomaly`** still
> 2-stage deterministic+LLM RCA. **No** portable redacted evidence-bundle product
> claim. eBPF→app-level error/stack capture **still not** claimed as full app
> exception lifecycle (watch UNFIRED).
>
> **Bottom line up front:** Coroot is the **nearest eBPF/RCA open-source competitor**
> and ships the **best MCP safety model in the field** (per-user OAuth + RBAC, one
> mutating tool). On **zero-instrumentation eBPF capture, shipped 2-stage AI RCA,
> continuous profiling, and a clean $1/CPU-core/no-ingest-cost model, Coroot is far
> ahead of pre-release Parallax.** The honest crux: Coroot's eBPF spans are
> **deliberately partial/protocol-level — no app-level errors, panics, or stack
> traces** — which is exactly Parallax's whole point (production error events).
> And on Parallax's recurring "read-only safe agent" claim, **Coroot already ships
> the best RBAC-scoped MCP** — Parallax's edge narrows to read-ONLY (Coroot has one
> mutating tool) + a redaction gate + the bounded bundle (unproven, A1).

## What each product is

- **Coroot** — open-source (**Apache-2.0** core / Community; commercial **Enterprise**
  **$1/CPU-core/mo**) **eBPF-based observability + APM** with zero-instrumentation
  capture and AI Root Cause Analysis. Metrics/logs/traces/continuous-profiling/SLO
  alerting + predefined dashboards/inspections. Wedge = adoption friction: deploy
  the eBPF agent, a service map appears with no app code changes. Go ~61%.
  **7,837★, v1.23.3 (2026-07-02) — pass 103: still latest GitHub release; no newer
  tag.** **MCP** ([docs.coroot.com/mcp/overview](https://docs.coroot.com/mcp/overview/),
  pass 103 live): HTTP streamable `/mcp` + **OAuth 2.0**; agent runs with the
  user's **RBAC**. **Community tools** = topology/health/traces/logs/PromQL +
  **one mutate** `resolve_alerts`; **EE tools** `list_anomalies` +
  **`investigate_anomaly`** (deterministic graph RCA then LLM prose — **not** free
  in CE). Tool table ≈ **17 CE + 2 EE**. **AI RCA** also: EE from $1/core/mo **or**
  Coroot Cloud integration for CE (**10 free investigations/mo** per AI docs).
  **eBPF→app-level-errors watch still UNFIRED** (pass 59/103). Coroot Inc.
- **Parallax** — open-source (Apache-2.0), Rust-first, self-hostable **execution-context engine**: OTLP-native ingest of traces/logs/metrics + CLI/agent traces, derives owned `error_event`s, fingerprints, correlates into a typed evidence graph, serves bounded/redacted evidence bundles to humans and coding agents. GreptimeDB + Turso. **Pre-release.**

Both Apache-2.0 OSS, self-hostable, with an agent/MCP surface and AI RCA. The overlap is real but the *signal source* differs fundamentally: Coroot = eBPF (protocol-level, no app code); Parallax = OTLP/Sentry (app-level, including errors).

## Signal coverage — and the critical eBPF-partial-spans gap

| Signal | Coroot (shipped) | Parallax (pre-release; ✅🧪=code-shipped) |
| --- | --- | --- |
| Traces | 🟡 **eBPF partial/protocol-level spans** (HTTP/Postgres/MySQL/Redis/Mongo/Memcached) | ✅🧪 OTLP traces (shipped, pre-release) |
| Logs | ✅ | ✅🧪 OTLP logs (shipped, pre-release) |
| Metrics | ✅ (Prometheus primary) | ✅🧪 OTLP metrics (shipped, pre-release) |
| Continuous profiling | ✅ eBPF | ❌ |
| **App-level errors / exceptions / panics / stack traces** | ❌ **(eBPF spans are partial — no app error chains)** | ✅🧪 derived `error_event` + fingerprint (shipped, pre-release) |
| Service map / dependency graph | ✅ (instant, eBPF) | ❌ (🏗) |
| SLO-based alerting | ✅ | 🟡 (🏗) |
| Sentry envelope / DSN | ❌ | ✅ shipped |

**Verdict:** Coroot's coverage is broad and all shipped, BUT its defining limitation — **eBPF spans are protocol-level, no app-level errors/panics/stacks** (Coroot's own docs say so) — is **exactly Parallax's whole point.** Parallax derives production error events from real services; Coroot deliberately does not. On coverage breadth, **Coroot wins;** on **app-level error semantics, Parallax **ships** error derivation (**shipped**, pre-release) where Coroot has none; fix-outcome offline residual plan 123 DONE; live value **unproven**).

## Ingestion & transport

- **OTLP:** Coroot ingests OTLP/HTTP traces + logs; metrics via Prometheus Remote Write; eBPF auto-instrumentation generates traces. OTLP/gRPC not clearly confirmed (HTTP emphasized). Both eBPF-auto AND OTLP — flexible.
- **eBPF zero-instrumentation:** Coroot's signature — deploy the node-agent DaemonSet, get a service map with no app changes. Parallax has **no eBPF story** (relies on OTel SDKs).
- **Sentry envelope:** Coroot has **none**. Parallax **ships** bounded envelope ingest (plan 118 DONE; multi-SDK ledger unproven).

**Verdict:** on **eBPF zero-instrumentation adoption friction, Coroot wins decisively** (Parallax has none). On OTLP-native + Sentry-envelope, **Parallax ships both** (plan 118 DONE).

## Storage architecture

- **Coroot:** **ClickHouse** (logs/traces/profiles, ~10× compression) + **Prometheus** (metrics; compatible with VictoriaMetrics/Thanos/Mimir). Multi-container stack (node-agent + cluster-agent + server). Not a single binary.
- **Parallax:** GreptimeDB (native OTLP tables) + Turso, single-binary target.

**Verdict:** on **single-binary simplicity, Parallax's target beats Coroot's multi-container stack** (real design edge). On proven-at-scale + operational maturity (ClickHouse+Prometheus is battle-tested), **Coroot wins.** Parallax's GreptimeDB vs Coroot's ClickHouse is benchmark-dependent/unmeasured (ties to the in-repo GreptimeDB-vs-ClickHouse study).

## Query & correlation

- **Coroot:** service-map-centric exploration, inspections, RCA-driven drill (trace→log→container→profile), predefined dashboards. Strong for infra/SRE investigation.
- **Parallax:** evidence-graph correlation + bounded bundle for agents.

**Verdict:** on **SRE service-map investigation, Coroot wins** (purpose-built eBPF). Parallax's evidence bundle is a different axis (bounded agent context), unproven (A1).

## Error tracking & workflow — Parallax's sharpest edge here

- **Coroot:** **no app-level error tracking, no fingerprinting, no issue lifecycle, no Sentry path.** eBPF spans are partial by design. Incidents exist (RCA on alert), not managed error issues.
- **Parallax:** derived `error_event` + deterministic fingerprint (**shipped**) + fix-outcome offline residual (**plan 123 DONE**; live value **unproven**).

**Verdict:** on **production app-error semantics + workflow, Parallax targets the single biggest Coroot gap** — Coroot does not capture app errors/panics/stacks at all. This is the most Parallax-favorable axis in the comparison, but error derivation is **shipped** (pre-release); fix-outcome offline residual plan 123 DONE; product maturity and live value **unproven.**

## AI-native / agent-context story — the safe-agent crux

- **Coroot's AI (shipped, Enterprise/Cloud-gated):** 2-stage RCA (deterministic-ML-then-LLM), OpenAI-compatible/BYO key, "what broke, why, how to fix." **And the key one: the best MCP safety model in the field** — per-user **OAuth + RBAC projection**, ~18 tools, **only `resolve_alerts` mutates.** This is the closest shipped thing to Parallax's "read-only safe agent projection" thesis.
- **Parallax's claim (code-shipped, A1 value unproven):** bounded, redacted, agent-safe evidence bundle served to coding agents (CLI/HTTP first, local-stdio MCP graduated (plan 112 DONE; remote deferred)) — **read-only by design**, redaction as a first-class gate.

**Honest verdict:** On shipped AI (2-stage RCA) **Coroot leads.** On **safe-agent-projection — a recurring Parallax claim — Coroot already ships the best RBAC-scoped MCP in the market.** This is a direct, uncomfortable test of Parallax's wedge: the "safe agent surface" is NOT unoccupied; Coroot owns the best version of it. Parallax's narrowing differentiators: (a) **strictly read-only** (Coroot has one mutating tool), (b) **redaction-before-access as a gate** (Coroot's MCP doesn't redact), (c) **bounded/versioned bundle** (Coroot serves queries, not a bounded artifact), (d) **production-incident evidence** scope. Error/Sentry/MCP/bundle pieces are **code-shipped**; A1 **value** unproven. Write it plainly: Parallax cannot claim "safe agent projection" as unique — Coroot ships the best one today.

## Architecture & deployment

- **Coroot:** multi-container (node-agent DaemonSet + cluster-agent + server), Docker Compose / K8s Helm / Swarm. Apache-2.0 core + Enterprise. Cloud available.
- **Parallax:** single-binary self-host target, local-first, air-gap-capable, Apache-2.0.

**Verdict:** on **single-binary local-first simplicity, Parallax's target beats Coroot's multi-container stack.** On **eBPF zero-instrumentation + K8s-native deploy, Coroot wins.** Different deployment ergonomics.

## Operational footprint

- **Coroot:** eBPF agents + ClickHouse + Prometheus + server — real stack, but the **adoption-friction pitch is "service map in 2 minutes, no code changes."** Operationally heavier than a single binary, but trivial to *start*.
- **Parallax:** single-binary target; lower ops floor (design goal).

**Verdict:** on **time-to-first-signal, Coroot wins** (eBPF, zero code). On **single-binary low-ops, Parallax's target wins** (Coroot is multi-container).

## Scalability & performance

- **Coroot:** proven at scale (7,837 stars, K8s deployments, Peter Zaitsev-backed). ClickHouse+Prometheus battle-tested. Specific numbers vendor (AI benchmark published); not independently measured here.
- **Parallax:** unproven; benchmark-dependent.

**Verdict:** on **proven-at-scale, Coroot wins conclusively.**

## Security

- **Coroot:** SSO/RBAC = **Enterprise-gated**; Community has none. The MCP's OAuth+RBAC is the standout.
- **Parallax:** SSO/RBAC/audit planned; redaction (A6) designed as first-class.

**Verdict:** on **shipped security (esp. MCP RBAC), Coroot wins.** Parallax's redaction-before-agent-access is a narrower, unproven edge; Coroot's RBAC-scoped MCP is the shipped benchmark for "safe agent."

## Privacy & compliance

- **Coroot:** self-host sovereignty; Cloud available. Compliance posture modest.
- **Parallax:** none yet; data ownership via self-host.

**Verdict:** roughly tied on self-host sovereignty. Scoped.

## Openness, licensing & vendor lock-in

- **Coroot:** **Apache-2.0** core (fully open, OSI), Enterprise features commercial. Self-host viable. Standard formats (OTLP/Prometheus in). Low lock-in. **Same Apache-2.0 as Parallax.**
- **Parallax:** Apache-2.0, OTLP-native, portable bundle.

**Verdict:** **tied** — both Apache-2.0, self-hostable, standard formats. No edge either way.

## Pricing & economics — real numbers

| Plan | Price | Notes |
| --- | --- | --- |
| **Community (OSS GitHub)** | **$0 / Apache-2.0** | core features for learning/PoC (not full commercial SKU) |
| **Standard** | **$1 / monitored CPU core / month** | **pass 63 live [coroot.com/pricing](https://coroot.com/pricing):** eBPF + SLO + AI RCA + **SSO/RBAC** + profiling + cost mon; volume discounts; 14-day trial; **no ingest fee** |
| **Premium** | **custom (no public $)** | 24×7/phone, air-gap install support, training, capacity planning |

**Pass-11 naming fix:** commercial self-host at $1/core is **Standard**, not “Enterprise-only”; Premium is the sales-quoted tier. Example on page: 10 nodes × 4 vCPU = 40 cores = **$40/mo**.

**Parallax pricing:** none public yet (pre-release).

**Honest cost read:** Coroot's $1/CPU-core/no-ingest model is genuinely clean and predictable. Whether Parallax self-host is cheaper is benchmark-dependent/unmeasured — but Coroot's pricing simplicity is a real strength.

## Where Coroot plainly wins

- eBPF zero-instrumentation (best adoption friction — service map in minutes, no code).
- Shipped 2-stage AI RCA + continuous profiling.
- **Best MCP RBAC safety model in the field** (OAuth+RBAC, 1 mutating tool) — the shipped benchmark for "safe agent."
- Proven-at-scale, K8s-native, Apache-2.0.
- Clean $1/CPU-core/no-ingest pricing.

## Where Parallax honestly edges Coroot

- **App-level error events / panics / stack traces** — Coroot's eBPF spans are partial/protocol-level by design; Parallax **ships** production error derivation (pre-release). *(Real, sharp Coroot gap; Parallax **shipped**.)*
- **Sentry-envelope compatibility** — Coroot has none. *(Real; Parallax shipped.)*
- **Strictly read-only + redaction-gated agent bundle** — Coroot's MCP has 1 mutating tool and no redaction gate. *(Real narrowing edge; Parallax **code-shipped** MCP + bundle redaction; A1 value **unproven**.)*
- **Single-binary local-first** — Coroot is multi-container. *(Real design edge.)*
- **Fix-outcome loop + bounded/versioned bundle** — Coroot has neither. *(Bundle **code-shipped**; fix-outcome offline residual plan **123 DONE**; live value **unproven**, A1.)*

> **Honest summary:** Coroot ships the best "safe agent" MCP (RBAC) and the best adoption-friction story (eBPF) — two axes adjacent to Parallax's thesis, both already occupied and mature. Parallax's defensible delta is **app-level error semantics** (Coroot's biggest structural gap — eBPF can't see app errors), **Sentry-envelope**, **single-binary**, **strict read-only + redaction**, and the **bounded+outcome bundle** — Sentry envelope shipped; bundle value/outcome unproven (A1). Do not claim "safe agent projection" as uniquely Parallax; Coroot ships the best one.

## Open questions / what measurement would settle

- **A1 gate vs Coroot RCA:** does a Parallax bundle beat Coroot-2-stage-RCA-as-context for coding-agent fix outcomes (esp. for app-error incidents Coroot can't see)? Unproven — and Coroot's blind spot (no app errors) is Parallax's opening.
- **Coroot latest version + MCP tool count** — pin exact latest tag; re-verify the "~18 tools, 1 mutating" MCP safety model from current docs.
- **eBPF → app-errors extension:** if Coroot adds app-level error capture, Parallax's sharpest edge here shrinks. Track.

## Sources (accessed 2026-07-17)

- [coroot.com](https://coroot.com/); [pricing](https://coroot.com/pricing); [enterprise](https://coroot.com/enterprise); [overview](https://coroot.com/overview).
- [docs.coroot.com/ai](https://docs.coroot.com/ai/); [mcp/overview](https://docs.coroot.com/mcp/overview/); [tracing/ebpf-based-tracing](https://docs.coroot.com/tracing/ebpf-based-tracing/).
- [github.com/coroot/coroot](https://github.com/coroot/coroot).
- Legacy internal: [coroot-deep-research.md](../coroot-deep-research.md) (2026-06-22 — components, eBPF-partial-spans, MCP RBAC safety model, v1.22.2).
- Parallax side: [decisions/storage-engine.md](../../decisions/storage-engine.md), [capture/rust.md](../../capture/rust.md), [validation/a1-bundle-value/](../../validation/a1-bundle-value/).
