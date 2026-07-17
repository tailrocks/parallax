# Parallax vs HolmesGPT

> An unbiased, one-to-one comparison. Research date: **2026-07-17** (**pass 41**
> version pin; **pass 95** Operator-mode; **pass 142** + **pass 177** re-pin +
> GitHub MCP write surface). Sources:
> [HolmesGPT/holmesgpt (GitHub)](https://github.com/HolmesGPT/holmesgpt)
> (**v0.36.0**, 2026-07-13; **2,874★** pass **177**; Apache-2.0; last push
> 2026-07-16), [holmesgpt.dev](https://holmesgpt.dev/), operator + GitHub MCP docs.
>
> **Bottom line up front:** HolmesGPT is a **CNCF Sandbox, Apache-2.0 AI SRE agent** that
> **investigates alerts by querying your existing telemetry stack** (Prometheus/Loki/Tempo/K8s)
> — it has **no own store**; it is the AI-investigation *layer*, not a backend. It is
> **strategically central to Parallax**: Parallax's own framing is *"the context engine,
> not the fixer — a separate agent consumes the bundle."* **HolmesGPT is that separate
> agent, shipping today.** On shipped AI-SRE investigation (alert→RCA→runbook, MCP-extensible,
> CNCF-native), **HolmesGPT is far ahead of pre-release Parallax.** They are **mostly
> complementary** (HolmesGPT queries telemetry; Parallax could be a telemetry+bundle source),
> but the **A1 crux is sharp: does a Parallax bounded bundle beat raw-telemetry-querying-via-HolmesGPT
> for agent fix outcomes?** Unproven.

## What each product is

- **HolmesGPT** (`HolmesGPT/holmesgpt`, by Robusta.dev) — an **open-source (Apache-2.0) AI SRE agent**, **CNCF Sandbox** project. Latest **v0.36.0** (2026-07-13), **2,874★**, active (pushed 2026-07-16 — **pass 177:** pin **unchanged** vs 142). **Investigates alerts/tickets** (Alertmanager/Prometheus/Jira/PagerDuty), pulls evidence from **K8s/cloud/DBs/VMs** for RCA (often <30s claim), runs **Markdown runbooks**, groups alerts into incidents (via Robusta). **MCP toolset support**. CNCF stack: Prometheus/Alertmanager, OpenTelemetry, Grafana Mimir/Loki/Tempo, Kubernetes. Commercial **Robusta** optional. **No own telemetry store** — queries yours.
  - **Pass 95 — Operator mode (README lead feature):** background 24/7 health checks; can message Slack with a fix; with **GitHub integration** can **open PRs** to fix findings ([operator docs](https://holmesgpt.dev/operator/)). This is **infra/SRE closed-loop pressure**, not a portable redacted multi-signal **evidence-bundle schema** and not Parallax's **outcome ledger** product. Still **no store**.
  - **Pass 142 — pin + write nuance:** release still **0.36.0** / **2,873★**. Operator docs still **alpha**, recommend GitHub MCP to **open PRs** (not market “auto-merge as product”). GitHub MCP docs: write permissions **optional**; `pull_requests` toolset (~10 tools) includes **PR ops, reviews, comments, merging** when write-enabled. So Holmes **can** merge if token+tools allow — unlike Datadog Bits (“never auto-merges”). **Does not** close the north-star gap of **app-code auto-merge + open portable outcome/recurrence corpus**; still no own store / redacted evidence-bundle contract.
- **Parallax** — open-source (Apache-2.0), Rust-first, self-hostable **execution-context engine**: OTLP-native ingest of traces/logs/metrics + CLI/agent traces, derives owned `error_event`s, fingerprints, correlates into a typed evidence graph, serves bounded/redacted evidence bundles to humans and coding agents. GreptimeDB + Turso. **Pre-release.**

Both Apache-2.0, agent-facing, in the "AI investigation" space. **HolmesGPT is the investigation agent (no store); Parallax is the telemetry+context-engine (own store).** Parallax's own framing ("context engine, not the fixer") literally describes the HolmesGPT relationship — HolmesGPT is the fixer/investigator Parallax feeds.

## Signal coverage — HolmesGPT investigates; Parallax stores+derives

| Signal | HolmesGPT (shipped) | Parallax (planned/shipped) |
| --- | --- | --- |
| AI alert investigation (alert→RCA) | ✅ **(the core — <30s RCA)** | 🟡 (🏗) |
| Runbook execution | ✅ | ❌ |
| MCP toolset (extensible data sources) | ✅ | ✅🧪 local-stdio MCP shipped (2 tools; remote deferred) |
| Incident grouping (Robusta) | ✅ | ❌ |
| **Own telemetry store** | ❌ (queries yours) | ✅🧪 GreptimeDB (shipped, pre-release) |
| Error derivation / fingerprinting | ❌ (reads your signals) | ✅ derived `error_event` (🧪 shipped) |
| Evidence bundle / bounded agent context | ❌ | 🟡🧪 code (A1 unproven) |
| Sentry envelope / DSN | ❌ | ✅ shipped |

**Verdict:** **different layers.** HolmesGPT excels at *investigating* telemetry you already have; Parallax excels (in design) at *owning, deriving, and bounding* telemetry into agent-safe evidence. HolmesGPT **has no store, no error-derivation, no bounded bundle** — exactly Parallax's layers.

## Ingestion & transport — the layer relationship

- **HolmesGPT:** queries your existing CNCF stack (Prometheus/Loki/Tempo/K8s/Alertmanager) + MCP toolsets. It is an **AI consumer/investigator of telemetry**, not a telemetry backend.
- **Parallax:** OTLP ingest gateway (producer/owner) + shipped Sentry-envelope adapter.

**Verdict:** HolmesGPT is an **investigation consumer; Parallax is a telemetry source.** They are **complementary** — HolmesGPT could query Parallax's store/bundles. On AI-investigation capability, **HolmesGPT is ahead of Parallax** (shipped, CNCF). On telemetry-ownership + error-derivation, **Parallax targets layers HolmesGPT doesn't occupy.**

## Storage / Query / Error / Workflow — HolmesGPT doesn't own these

HolmesGPT **has no storage, no query engine, no error-derivation, no issue lifecycle** — it reads your existing stack and writes analysis back (Slack/source). All the own-the-data layers are Parallax's domain (in design).

**Verdict:** no head-to-head on own-the-data axes — different layers.

## AI-native / agent-context story — the central crux

- **HolmesGPT:** a **shipped AI SRE agent** — fetch an alert, pull evidence across the stack, produce an RCA, run a runbook, write it back. **CNCF-native, MCP-extensible, Apache-2.0.** This is the **canonical "AI investigation agent that consumes telemetry"** — exactly the role Parallax's framing assigns to "a separate agent."
- **Parallax's claim:** a **bounded, redacted, agent-safe evidence bundle** served to coding agents — a *context engine* that produces a safe, validated dossier, not an open-ended investigator.

**Honest verdict (the crux):** HolmesGPT **is the shipped realization of "an agent that investigates telemetry"** — and it's Apache-2.0, CNCF-backed, mature. **Parallax's entire thesis rests on the bet that a bounded/redacted/validated bundle beats raw-telemetry-querying-via-HolmesGPT for coding-agent fix outcomes** — and that bet is **unproven (A1 gate).** Written plainly: if a team already has HolmesGPT investigating their stack, the value-add of Parallax must be *measured*, not assumed. The honest framing from Parallax's own legacy research holds: **HolmesGPT is "the AI investigation layer Parallax must feed, not beat."** Parallax's delta is owning the telemetry + error-derivation + bounded/redacted safety — unproven to beat HolmesGPT-over-raw-telemetry.

## Architecture & deployment

- **HolmesGPT:** **Apache-2.0 OSS** (self-host), deployable via Robusta; CNCF Sandbox. No backend to run (uses yours).
- **Parallax:** single-binary self-host target, local-first, air-gap-capable, Apache-2.0, Rust + GreptimeDB + Turso.

**Verdict:** both Apache-2.0 + self-hostable. HolmesGPT is lighter (no backend); Parallax owns the backend. **Complementary deployment** — HolmesGPT can query Parallax.

## Scalability / Security / compliance

- **HolmesGPT:** scales with your stack (it's stateless investigation); CNCF community; security/compliance = your stack's. Robusta (commercial) adds platform features.
- **Parallax:** unproven at scale; SSO/RBAC/audit planned; redaction (A6) designed.

**Verdict:** **different concerns** — HolmesGPT inherits your stack's posture; Parallax brings its own.

## Openness, licensing & vendor lock-in

- **HolmesGPT:** **Apache-2.0** (OSI-open, same as Parallax), CNCF Sandbox. Zero lock-in (queries standard CNCF stack). Robusta (commercial) is optional.
- **Parallax:** Apache-2.0, fully open, OTLP-native, portable bundle.

**Verdict:** **tied on openness** — both Apache-2.0, CNCF-adjacent, standard-formats. No edge either way. Strong mutual openness.

## Pricing & economics

- **HolmesGPT:** **Apache-2.0 OSS free** (self-host; bring your LLM key). **Robusta** (commercial platform: incident grouping, memory, multi-cluster) — paid; **confirm tiers on [robusta.dev](https://robusta.dev/).**
- **Parallax pricing:** none public yet (pre-release); stated shape = Apache open core + managed cloud + outcome-priced fixer.

**Honest cost read:** HolmesGPT-core is free (Apache + BYO-key). Not a cost contest with Parallax (different layer). A stack could use Parallax (own+derive+bundle) as a richer source for HolmesGPT.

## Where HolmesGPT plainly wins

- **Shipped AI SRE investigation** (alert→RCA <30s, runbooks, MCP-extensible) — the canonical "agent that investigates telemetry."
- **CNCF-native** (Prometheus/Loki/Tempo/K8s/Alertmanager/OTel) + **Apache-2.0** + **CNCF Sandbox** backing.
- Incident grouping (Robusta) + write-back to Slack/source.

## Where Parallax honestly edges HolmesGPT

- **Owns the telemetry** (store + error-derivation + fingerprint) — HolmesGPT has no store; it queries yours. *(Real layer difference.)*
- **Bounded, redacted, agent-safe evidence bundle** — HolmesGPT investigates open-endedly over raw telemetry; Parallax produces a validated dossier. *(Thesis, unproven A1 — the crux.)*
- **Fix-outcome loop (Parallax-shaped)** — HolmesGPT has **no portable outcome ledger / recurrence verdict product** over app errors. **Pass 95 nuance:** Operator mode + GitHub PR is a **real SRE automation path** (detect→investigate→propose PR); do **not** pretend HolmesGPT is fix-loop-free. Difference = **layer + artifact**: HolmesGPT automates over **your existing stack APIs**, not a versioned redacted multi-signal **bundle + outcome record**. Parallax offline residual plan **123 DONE**; live value **unproven**.
- **Sentry-envelope compatibility** — HolmesGPT has a **Sentry MCP toolset** (query issues as a consumer); Parallax **ingests** Sentry envelopes as a store path. Different sides of Sentry.

> **Honest summary:** HolmesGPT is **strategically central** to Parallax — it is the **shipped, CNCF, Apache-2.0 realization of "an AI agent that investigates telemetry,"** which is exactly the role Parallax's "context engine, not the fixer" framing assigns to a *separate* agent. On shipped AI-investigation, HolmesGPT is far ahead. **Operator mode (pass 95)** adds continuous detection + optional PR open — **A1 pressure rises** (raw-context agents get better automation). The two are **mostly complementary** (HolmesGPT queries; Parallax owns+derives+bundles) — HolmesGPT could query Parallax as a richer source. **The A1 crux is sharp and must be stated plainly: Parallax's value over HolmesGPT-over-raw-telemetry is unproven.** Do not assume Parallax beats HolmesGPT; it must be measured.

## Watch triggers / open questions

- **A1 gate vs HolmesGPT (the crux):** does a Parallax bounded bundle beat HolmesGPT-investigating-raw-telemetry for coding-agent fix outcomes? **Unproven — the central validation question.** Braintrust-class eval tooling could measure it.
- **Operator mode + app-code auto-merge:** if HolmesGPT (or peers) ship **default auto-merge of application-code PRs** from production failures with portable outcome records, north-star closed-loop claim tightens (today README emphasizes open PR / Slack fix messaging, not unattended app merge).
- **HolmesGPT → Parallax integration** — could HolmesGPT query Parallax's store/bundles as a richer, pre-redacted source? (Likely yes via MCP/OTLP.) Worth a PoC — and it may be the *natural* deployment (Parallax feeds HolmesGPT).
- **Robusta commercial trajectory** — track whether Robusta adds owned-telemetry/bundle features (would narrow the layer distinction).

## Sources (accessed 2026-07-17)

- [HolmesGPT/holmesgpt (GitHub)](https://github.com/HolmesGPT/holmesgpt) — **v0.36.0** (2026-07-13), **2,873★**, Apache-2.0, last push 2026-07-16 (**pass 95:** pin reconfirmed).
- README **Operator mode** lead feature + [holmesgpt.dev/operator](https://holmesgpt.dev/operator/); [holmesgpt.dev](https://holmesgpt.dev/); [CNCF project page](https://www.cncf.io/projects/holmesgpt/).
- [CNCF blog: Auto-diagnosing K8s alerts with HolmesGPT (Apr 2026)](https://www.cncf.io/blog/2026/04/21/auto-diagnosing-kubernetes-alerts-with-holmesgpt-and-cncf-tools/); [Robusta.dev](https://robusta.dev/).
- Parallax side: [00-vision/north-star-autonomous-fix-loop.md](../../00-vision/north-star-autonomous-fix-loop.md), [reference/agent-observability-review.md](../../reference/agent-observability-review.md), [validation/a1-bundle-value/](../../validation/a1-bundle-value/).
- Sibling (different-layer peers): [parallax-vs-causely.md](parallax-vs-causely.md) (causal-MCP layer), [parallax-vs-odigos.md](parallax-vs-odigos.md) (instrumentation), [parallax-vs-mezmo.md](parallax-vs-mezmo.md) (pipeline).
