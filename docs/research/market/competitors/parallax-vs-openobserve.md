# Parallax vs OpenObserve

## Current live verification — 2026-09-04

OpenObserve GA `v0.92.2` was the current stable comparison artifact; `v1.0.0-rc2`
was excluded as a release candidate. Fresh Rotel fan-out delivered current
playground traces/logs/metrics, and the OpenObserve search path returned fresh
service data. OpenObserve remains ahead on shipped Rust single-binary
observability, general query maturity, and broader platform surface. Parallax's
live edge is its narrower derived-error/evidence/MCP workflow; bundle value is
unproven. Exact evidence: [canonical report](../../validation/2026-09-04-parallax-main-competitor-verification.md).

The dated pass notes below preserve historical claims; this section is the
current version authority.

> An unbiased, one-to-one comparison. Research date: **2026-07-17** (pass 48
> Cloud pricing; **pass 94** EE gates; **pass 102** pin; **pass 119** pricing;
> **pass 155** + **pass 182** + **pass 206** pin + MCP primary docs). Still
> **v0.91.2** latest stable; **20,197★** (pass **206**; was 20,196); AGPL-3.0;
> **v0.92.0-rc2** still RC. Sources: [pricing](https://openobserve.ai/pricing/),
> [MCP marketing](https://openobserve.ai/mcp-server/),
> **[MCP docs](https://openobserve.ai/docs/integration/ai/mcp/)**, GitHub.
>
> **Pass 206 MCP re-fetch:** still **`O2_AI_ENABLED`** + **`CreateAlert` /
> `DeleteAlert`** (and peer mutators). Marketing “read-only tool execution”
> still **contradicted** by docs. Parallax free RO stdio MCP distinct (A1 unproven).
>
> **Bottom line up front:** OpenObserve is the **nearest open-source competitor
> on Parallax's own architectural axes** — Rust engine, single binary, self-host,
> OTLP-native, object-storage-Parquet. **On those shared axes, OpenObserve is
> shipping and mature; Parallax is pre-release.** Parallax edges: Sentry-envelope,
> derived errors + outcome loop, **Apache-2.0 vs AGPL**, free read-only MCP vs
> **EE mutating MCP**, unproven bounded redacted bundle (A1).

## What each product is

- **OpenObserve** ("O2") — open-source (**AGPL-3.0** core, relicensed from Apache-2.0 in Nov 2023), cloud-native observability platform unifying logs, metrics, traces, frontend/RUM (+ session replay), data pipelines, and LLM observability in a **single Rust binary**. Positioned as a Datadog/Splunk/Elasticsearch alternative; headline "140× lower storage cost" from Parquet-on-object-storage. Ships an **AI SRE** + **broad MCP server** (**Enterprise-only** per docs; **mutating** tools including delete/create alerts, streams, dashboards, users — pass **155**). OpenObserve Inc.; Series A **$10M** (~2026-04-29). **Latest stable: v0.91.2** (2026-07-17); **20,196★**; **v0.92.0-rc2** prerelease only.
- **Parallax** — open-source (Apache-2.0), Rust-first, self-hostable **execution-context engine**: OTLP-native ingest of traces/logs/metrics + CLI/agent traces, derives owned `error_event`s, fingerprints, correlates into a typed evidence graph, serves bounded/redacted evidence bundles to humans and coding agents. GreptimeDB + Turso. **Pre-release.**

These overlap maximally on architecture — both Rust, single-binary, self-host, OTLP-native, object-storage-columnar. OpenObserve is the closest competitor on Parallax's own bet. The honest differentiator is product *intent*: OpenObserve is a general full-platform Datadog-alternative; Parallax is a narrow agent-context engine for production incidents.

## Signal coverage

| Signal | OpenObserve (shipped) | Parallax (pre-release; ✅🧪=code-shipped) |
| --- | --- | --- |
| Traces | ✅ | ✅🧪 OTLP traces (shipped, pre-release) |
| Logs | ✅ (strength — full-text, tantivy index) | ✅🧪 OTLP logs (shipped, pre-release) |
| Metrics | ✅ | ✅🧪 OTLP metrics (shipped, pre-release) |
| Frontend / RUM + session replay | ✅ | ❌ |
| Data pipelines / VRL transform | ✅ | 🟡 (🏗) |
| LLM observability | ✅ | ✅ (🏗) |
| Errors / exceptions | 🟡 (queryable; no Sentry-grade issue lifecycle) | ✅🧪 derived `error_event` + fingerprint (shipped, pre-release) |
| Dashboards / alerts | ✅ mature | 🟡 minimal (🏗) |

**Verdict:** OpenObserve's coverage is broader and all shipped. On coverage, **OpenObserve wins decisively.** No native Sentry-grade error-issue lifecycle (same gap as Grafana/Honeycomb) — a cell Parallax's design targets.

## Ingestion & transport — a real overlap, OpenObserve ahead

- **OTLP:** OpenObserve is genuinely **OTLP-native** (logs/metrics/traces over OTLP/gRPC + HTTP). Same native stance Parallax designs for. **Parity on the OTLP-native claim** — but OpenObserve ships it, Parallax does not yet.
- **Protocols:** OTLP, Prometheus remote-write, Fluent/Vector, many log shippers, Sentry (partial). Broad.
- **Parallax:** OTLP gateway + shipped Sentry-envelope adapter.

**Verdict:** on OTLP-native ingest, **tied in design; OpenObserve wins in that it ships today.** On protocol breadth, **OpenObserve wins.**

## Storage architecture — near-identical physics, different engine

- **OpenObserve:** object-storage-native **Parquet** (schema-on-read), WAL+memtable→Parquet→object store; **tantivy inverted index** (enabled by default) + bloom filters (added 2024, correcting older "no inverted index" notes); **Apache DataFusion** query over Parquet. Single-node SQLite or HA Postgres+NATS. Runs on **≥512 MB RAM**. "140× lower storage cost" claim (vendor, benchmark-dependent).
- **Parallax:** **GreptimeDB** (native OTLP tables) + Turso metadata. Same physics (columnar-on-object-store) and Parallax also evaluated DataFusion-class engines — but chose GreptimeDB's native OTLP model.

**Verdict:** on the architectural bet (Rust + columnar-on-object-store + self-host), **these are the same physics** — and OpenObserve has shipped it at scale (20k stars, large deployments) while Parallax's GreptimeDB-native variant is **unproven.** Parallax's GreptimeDB-vs-Parquet/DataFusion edge (if any) is **benchmark-dependent and unmeasured.** On proven-at-scale, **OpenObserve wins.**

## Query & correlation

- **OpenObserve:** SQL/DataFusion over Parquet, full-text (tantivy), dashboards, log/metric/trace exploration, cross-signal pivoting in the UI. Mature general-purpose query.
- **Parallax:** evidence-graph correlation + bounded bundle for agents. Different goal.

**Verdict:** on **general cross-signal query/exploration, OpenObserve wins** (mature, shipped). Parallax's evidence-bundle abstraction is a different axis (bounded agent context), unproven (A1).

## Error tracking & workflow

- **OpenObserve:** errors are queryable logs/span-events + alerting; **no native issue lifecycle** (no resolve/regress/assign/ownership like Sentry). Has an Incidents feature.
- **Parallax:** derived `error_event` + deterministic fingerprint (**shipped**) + fix-outcome offline residual (**plan 123 DONE**; live value **unproven**).

**Verdict:** on **error-issue workflow, Parallax ships error derivation + fingerprint** (pre-release) where OpenObserve is thin; fix-outcome offline residual plan 123 DONE, live value **unproven.** Scoped.

## AI-native / agent-context story — the key axis (and a real OpenObserve strength + weakness)

- **OpenObserve's AI (shipped, Enterprise-gated):**
  - **AI SRE** — background service for intelligent workflows, automated investigation, "always-available SRE." Enterprise + **BYO-LLM-key**.
  - **MCP server (Enterprise only)** — live docs [openobserve.ai/docs/…/mcp](https://openobserve.ai/docs/integration/ai/mcp/) (pass **56**): explicitly **Enterprise edition**; requires `O2_AI_ENABLED=true`. Tool surface includes **search** (`SearchSQL`, PromQL, traces) **and large write/destructive sets** (⚠️ legend): Create/Delete **alerts**, **dashboards**, **streams**, **pipelines**, **users/roles**, **KV**, ingest `LogsIngestionJson`, etc. Categories documented: Alerts 28, Dashboards 20, Search 17, Auth, Pipelines, Streams, Traces, Users, … — **far beyond a read-only projection; no public “read-only MCP mode” product flag**. Docs recommend dedicated scoped user + confirm tool calls (security section).
  - **AI Assistant** (Enterprise).
  - **All AI/MCP features are Enterprise-gated** (AGPL core does not include them); **Sensitive Data Redaction (SDR) is also Enterprise** (pass 60 re-confirm + [oss-agent-surface-gating-2026-07-17.md](../oss-agent-surface-gating-2026-07-17.md) — README lists SDR as EE; free AGPL core has no portable redacted evidence-bundle export).
- **Parallax's claim (code-shipped, A1 value unproven):** bounded, redacted, agent-use (safety/value unproven) evidence bundle served to coding agents (CLI/HTTP first, local-stdio MCP graduated (plan 112 DONE; remote deferred)) — **read-only by design**, redaction as a first-class pre-exposure gate.

**Honest verdict:** OpenObserve ships far more AI today (AI SRE + EE MCP with **dozens of mutating tools** + assistant) than Parallax. On shipped AI, **OpenObserve leads.** OpenObserve's AI is **Enterprise-gated + BYO-key + write-capable by design** — not a free read-only agent surface. That cell (free, read-only, redacted, bounded) stays unoccupied in O2 — Parallax designs it, value **unproven (A1)**. **No-bias:** O2's EE MCP is a mature, broad agent ops surface; Parallax cannot claim “MCP for obs” as unique — only the read-only/redacted/portable bundle posture.

## Architecture & deployment model — near-mirror, OpenObserve shipped

- **OpenObserve:** **single Rust binary** (SQLite + local/object store) or horizontally-scaled stateless services (Postgres + object store + NATS). Self-host OSS (AGPL, free, no caps) or OpenObserve Cloud (SaaS, usage-based). Multi-region Super Cluster = Enterprise.
- **Parallax:** single-binary self-host target, local-first, offline/local deployment target (air-gap unverified), Apache-2.0. GreptimeDB + Turso.

**Verdict:** on the single-binary-Rust-self-host bet, **these are the same design — and OpenObserve has shipped it**, while Parallax is pre-release. **OpenObserve is ahead on Parallax's own architectural claim.** Parallax's real differentiators here are **Apache-2.0 vs AGPL-3.0** (a license-permissiveness edge) and the **GreptimeDB-native** storage choice (unproven advantage). Honestly: the "Rust single-binary self-host OTLP-native" wedge is **no longer unique to Parallax** — OpenObserve owns it, shipped.

## Operational footprint

- **OpenObserve:** single binary, **≥512 MB RAM** floor, disk or object store. Mature, low-footprint. Cloud = zero ops.
- **Parallax:** self-hosted GreptimeDB + Turso + engine; single-binary target.

**Verdict:** on **operational simplicity/footprint, OpenObserve wins** — it ships the single-binary-low-RAM story Parallax targets, today. Parallax's target is parity, unproven.

## Scalability & performance

- **OpenObserve:** proven at scale (20k stars, large deployments, Series A). "140× lower storage cost" is a vendor claim, **benchmark-dependent.** tantivy+bloom+DataFusion query is real.
- **Parallax:** unproven at production scale.

**Verdict:** on **proven-at-scale, OpenObserve wins conclusively.** The GreptimeDB-vs-Parquet/DataFusion cost/perf question is **benchmark-dependent and unmeasured** — and directly relevant to the in-repo [GreptimeDB-vs-ClickHouse](../../storage/greptimedb-vs-clickhouse/) study (Parquet/DataFusion is the OpenObserve side of that comparison).

## Security

- **OpenObserve:** SSO/RBAC/audit = **Enterprise-gated** (free under self-host Enterprise ≤50 GB/day; paid beyond). Not in the AGPL OSS core.
- **Parallax:** SSO/RBAC/audit planned; redaction (A6) designed as first-class (not paywalled).

**Verdict:** on **shipped security, OpenObserve wins** (it exists, even if gated) — Parallax's is planned. But Parallax's **redaction-as-free-first-class-gate** is a real philosophical contrast to OpenObserve's **Enterprise-gated Sensitive Data Redaction.** Scoped.

## Privacy & compliance

- **OpenObserve:** data residency, self-host sovereignty; compliance posture via Cloud/Enterprise. Self-host = your posture.
- **Parallax:** none yet; data ownership via self-host.

**Verdict:** on **self-host sovereignty, tied** (both self-hostable). On compliance certifications, OpenObserve Cloud has more. Scoped.

## Openness, licensing & vendor lock-in — a real Parallax edge

- **OpenObserve:** **AGPL-3.0** core (network-use copyleft — a real consideration for embedding/modifying/distributing); Enterprise features under a separate commercial license. Self-host OSS free with no caps. OTLP-native (standard formats). Moderate lock-in (Parquet/standard in; OpenObserve dashboards/pipelines out).
- **Parallax:** **Apache-2.0**, fully open, OTLP-native, portable bundle. No copyleft network-use clause.

**Verdict:** on **license permissiveness, Parallax (Apache-2.0) edges OpenObserve (AGPL-3.0)** — a real, if narrow, difference for users who care about AGPL's network-use terms. This is one of Parallax's few *real* edges vs OpenObserve. (OpenObserve's AGPL move from Apache in 2023 was deliberately to protect commercial positioning — a trade-off.)

## Extensibility

- **OpenObserve:** broad ingestion protocols, pipelines (VRL), functions, integrations, 140+-tool MCP, alerting, dashboards, API. Deep.
- **Parallax:** OTel-native, CLI/HTTP/MCP, pipeline/processor, webhooks (planned).

**Verdict:** on **ecosystem/integration breadth, OpenObserve wins decisively.**

## Pricing & economics — real numbers

OpenObserve pricing is **public** ([openobserve.ai/pricing](https://openobserve.ai/pricing/), **pass 48 + pass 63 + pass 94 + pass 119 re-confirm**):

| Plan | Price | Notes |
| --- | --- | --- |
| **OSS / Community (AGPL)** | **$0, no caps** | self-host core features |
| **Self-Hosted Enterprise** | **free ≤50 GB/day**; paid/contact beyond | SSO/RBAC/audit/redaction/QoS (FAQ, twice on page) |
| **Cloud Professional (PAYG)** | **$0.50 / GB ingest** (+ annual ~30% discount claim) + **$0.01 / GB query** | metrics ret. **15 mo**; non-metrics **30 days** (+$0.02/GB per extra 30d); unlimited users; 14-day free trial; **AI preview free with 20 credits** (Incident/AI SRE/Assistant) per pricing FAQ |
| **Cloud Enterprise** | custom | AI SRE / Incident Mgmt / AI Assistant, pipelines, redaction, BYOC, SLAs |

**AI (FAQ):** AI SRE Agent + AI Assistant **free during preview** (20 credits). ⚠️ pass-9 “Cloud fully usage-based with no rates” was underspecified — **ingest/query unit prices are public**. **Pass 119:** unit rates and 50 GB/day EE free allowance **unchanged** vs pass 94.

**Parallax pricing:** **no public number** (pre-release).

**Honest cost read:** OpenObserve's self-host economics are very strong (free, unlimited, 140×-storage-cost claim). Whether Parallax self-host is cheaper at a workload is **benchmark-dependent and unmeasured.** On cost-transparency-for-self-host, OpenObserve is a tough benchmark — Parallax cannot assume a cost edge here without measurement.

## Where OpenObserve plainly wins

- **Parallax's own architectural axes** — Rust, single binary, self-host, OTLP-native, Parquet-on-object-store: **all shipped and proven at scale.** This is the key honest finding.
- Full signal breadth (logs strength + RUM/replay + pipelines + LLM obs).
- Shipped AI (AI SRE + 140-tool MCP + assistant) — more than Parallax.
- Proven-at-scale + operational maturity (≥512 MB RAM).
- Deep ecosystem/protocols.
- Strong self-host economics (free unlimited + free Enterprise ≤50 GB/day).

## Where Parallax honestly edges OpenObserve

- **License permissiveness** — Apache-2.0 vs AGPL-3.0 (network-use copyleft). *(Real, narrow.)*
- **Read-only, safe-by-default agent projection** — OpenObserve's MCP is Enterprise-gated + write/destructive; Parallax designs read-only+redacted. *(Real design contrast; Parallax local-stdio MCP + bundle redaction **code-shipped**; A1 value unproven.)*
- **Redaction as a free, first-class pre-exposure gate** — OpenObserve's Sensitive Data Redaction is Enterprise-gated. *(Real philosophical edge; Parallax A6 planned.)*
- **Sentry-envelope compatibility** — OpenObserve has no Sentry-envelope path; Parallax ships envelope ingest to absorb Sentry's 30+ SDKs. *(Real; Parallax shipped; plan 118 DONE.)*
- **Production error-issue workflow + fix-outcome loop** — OpenObserve has none. *(Real gap; Parallax error derivation **shipped**; fix-outcome offline residual plan 123 DONE; live value unproven.)*
- **GreptimeDB native-OTLP storage** — a different storage bet (vs Parquet/DataFusion). *(Design choice; advantage unproven.)*
- **Bounded, redacted, agent-use (safety/value unproven) evidence bundle** — the differentiated thesis. *(Thesis, **unproven** — A1 gate.)*

> **The honest summary:** OpenObserve has *already shipped* the Rust-single-binary-self-host-OTLP-native architecture Parallax is building. Parallax's wedge against OpenObserve is **not** the architecture — it is (a) Apache-vs-AGPL, (b) read-only-safe + free-redaction agent posture vs Enterprise-gated-write-capable MCP, (c) Sentry-envelope compat, (d) production-error derivation (**shipped**) + outcome offline residual (plan 123 DONE), and (e) the bounded agent bundle (**code-shipped**, A1 value **unproven**). This is the sharpest "competitor may be better on Parallax's own axes" case in the set.

## Open questions / what measurement would settle

- **A1 gate vs OpenObserve AI SRE:** does a Parallax bounded bundle beat OpenObserve-AI-SRE-as-context for coding-agent fix outcomes? Unproven — and OpenObserve's shipped AI SRE is the direct competitor.
- **Storage cost/perf:** measured GreptimeDB (Parallax) vs Parquet/DataFusion (OpenObserve) — ingest, query, cost-per-byte. Benchmark-dependent, unmeasured; ties to the GreptimeDB-vs-ClickHouse study.
- ~~MCP-safety / read-only mode~~ → **pass 56:** docs show **no public read-only MCP product mode**; EE MCP is write-heavy (DeleteStream, CreateAlert, …). Track if O2 later ships a restricted tool profile.
- **AGPL → Apache risk:** unlikely, but track OpenObserve's license posture.

## Sources (accessed 2026-07-17)

- [openobserve.ai](https://openobserve.ai/); [pricing](https://openobserve.ai/pricing/) (**pass 94:** Self-Hosted EE free ≤**50 GB/day**; AI SRE/SDR Enterprise); [AI SRE](https://openobserve.ai/ai-sre/); [MCP server](https://openobserve.ai/mcp-server/); [downloads (v0.91.2)](https://openobserve.ai/downloads); [GitHub releases](https://github.com/openobserve/openobserve/releases) — **v0.91.2 (2026-07-17), 20,196★** (pass 102).
- [github.com/openobserve/openobserve](https://github.com/openobserve/openobserve) (AGPL-3.0).
- Legacy internal: [openobserve-deep-research.md](../openobserve-deep-research.md) (2026-06-22 — sources, architecture, Series A, tantivy correction).
- Parallax side: [decisions/storage-engine.md](../../decisions/storage-engine.md), [storage/greptimedb-vs-clickhouse/](../../storage/greptimedb-vs-clickhouse/), [validation/a1-bundle-value/](../../validation/a1-bundle-value/).
