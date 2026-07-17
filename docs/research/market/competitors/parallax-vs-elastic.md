# Parallax vs Elastic Observability

> An unbiased, one-to-one comparison. Research date: **2026-07-17**.
> Sources: [Elastic Observability](https://www.elastic.co/observability/) + [OpenTelemetry/EDOT](https://www.elastic.co/observability/opentelemetry) + [pricing](https://www.elastic.co/pricing), [cubeapm 2026 review](https://cubeapm.com/blog/elastic-observability-pricing-and-review/).
>
> **Bottom line up front:** Elastic Observability is a major **OSS-origin (ELv2)
> incumbent** unifying logs/metrics/traces/APM/profiling/synthetics with **security
> (SIEM)** — a search-engine-grade log/analytics backend (the Elasticsearch origin)
> now OTLP-native via **EDOT**. On **log search/analytics depth, ES|QL, the
> unified observability+security story, self-host, and scale, Elastic is far ahead
> of pre-release Parallax.** Parallax's honest edges are **Apache-2.0 vs ELv2**
> (a real license difference — ELv2's managed-service restriction),
> **purpose-built-telemetry-native vs search-engine-as-backend**, **single-binary
> Rust vs a heavy distributed ES cluster**, Sentry-envelope, and the *unproven*
> bundle + fix-outcome thesis (A1 gate).

## What each product is

- **Elastic Observability** (Elastic N.V.) — the observability stack on **Elasticsearch/Kibana**: logs, metrics, traces/APM, continuous profiling, synthetics, RUM, + **SIEM/security** (unified search + observability + security). Query via **ES|QL/KQL**. **OTLP-native** ingest via **EDOT** (Elastic Distributions of OpenTelemetry) + auto-instrumentation; data stored in the **native OTel schema** (no translation). Sold as **Elastic Cloud Hosted** and **Elastic Cloud Serverless** + **self-host**. **License: Elastic License v2 (ELv2)** — source-available, self-hostable, **but with a managed-service restriction** (you cannot offer Elastic as a hosted service to others). Current major line **8.x / 9.x** (Elasticsearch 9 line; **pin exact latest**). Search-engine-origin backend.
- **Parallax** — open-source (Apache-2.0), Rust-first, self-hostable **execution-context engine**: OTLP-native ingest of traces/logs/metrics + CLI/agent traces, derives owned `error_event`s, fingerprints, correlates into a typed evidence graph, serves bounded/redacted evidence bundles to humans and coding agents. GreptimeDB + Turso. **Pre-release.**

Both OSS-origin, self-hostable, OTLP-native, with a columnar/search backend. The core difference: **Elastic is a search engine (Lucene) repurposed for observability**; **Parallax is a purpose-built telemetry-native engine on GreptimeDB**. Different optimization center.

## Signal coverage

| Signal | Elastic (shipped) | Parallax (planned) |
| --- | --- | --- |
| Logs (search-grade) | ✅ **(the origin strength — full-text, analytics)** | ✅ OTLP logs (🏗) |
| Metrics | ✅ | ✅ OTLP metrics (🏗) |
| Traces / APM | ✅ (EDOT/OTLP) | ✅ OTLP traces (🏗) |
| Continuous profiling | ✅ | ❌ |
| Synthetics | ✅ | ❌ |
| SIEM / security | ✅ (unified obs + sec) | ❌ |
| Errors / exceptions | 🟡 (queryable; no Sentry-grade issue lifecycle) | ✅ derived `error_event` + fingerprint (🏗) |
| ES|QL / KQL query | ✅ | ❌ (SQL via GreptimeDB) |
| Sentry envelope / DSN | ❌ | ✅ shipped |

**Verdict:** Elastic's coverage is broad and all shipped, with a distinctive **search-grade log analytics + unified obs/security** strength. On coverage, **Elastic wins decisively.** No Sentry-envelope path, no fix-outcome loop (same gap).

## Ingestion & transport

- **OTLP:** Elastic is genuinely **OTLP-native** via **EDOT** (Elastic's OpenTelemetry Collector distribution) + language SDK auto-instrumentation; data lands in the **native OTel schema** (no attribute translation). This is a real, current OTLP-native stance — parity with Parallax's design.
- **Beats/Elastic Agent:** legacy + current collectors alongside EDOT.
- **Sentry envelope:** none.

**Verdict:** on OTLP-native ingest, **tied in design; Elastic ships it.** On Sentry-envelope, **Parallax ships bounded envelope ingest** (plan 118 DONE).

## Storage architecture — the central contrast

- **Elastic:** **Apache Lucene** (inverted-index search engine) + columnar/doc-values; the search-grade backend. Powerful full-text/analytics, but **historically heavier and more operationally complex** than purpose-built telemetry columnar stores (shards, indices, JVM heap tuning). Self-host = a distributed cluster. Object-storage cold tiers exist.
- **Parallax:** **GreptimeDB** (purpose-built time-series/telemetry columnar, native OTLP tables) + Turso — designed for telemetry ingest/query, not general search.

**Verdict:** on **search-grade log analytics + full-text**, **Elastic wins** (Lucene is unmatched for search). On **telemetry-optimized ingest/query + operational simplicity**, **Parallax's purpose-built bet targets an edge** (but is **unproven** vs Elastic's mature stack). GreptimeDB-vs-Elasticsearch cost/perf for *telemetry* workloads is benchmark-dependent/unmeasured. On proven-at-scale, **Elastic wins conclusively.**

## Query & correlation

- **Elastic:** ES|QL/KQL + Discover + dashboards; cross-signal correlation; unified obs+sec investigation (a real strength — one backend for logs, traces, and security signals).
**Parallax:** evidence-graph correlation + bounded bundle for agents.

**Verdict:** on **general cross-signal + obs/sec investigation, Elastic wins** (mature, unified). Parallax's evidence-bundle is a different axis (bounded agent context), unproven (A1).

## Error tracking & workflow

- **Elastic:** errors are queryable logs/span-events; **no native Sentry-grade issue lifecycle.**
- **Parallax:** derived `error_event` + fingerprint + (planned) fix-outcome loop.

**Verdict:** on **error-issue workflow, Parallax targets a real Elastic gap** — but planned/unproven.

## AI-native / agent-context story

- **Elastic's AI:** Elastic AI Assistant (observability + security), ES|QL-from-natural-language, anomaly detection, the **Elastic AI Error Assistant**. A human-assistive AI across obs+sec; not a bounded agent-context projection.
- **Parallax's claim:** bounded, redacted, agent-safe evidence bundle for coding agents (planned, A1).

**Honest verdict:** Elastic ships more AI today (assistant, NL→ES|QL, anomaly, error assistant, plus the LLM-obs features) than Parallax. On shipped AI, **Elastic leads.** Parallax's bounded-agent-context claim is **unproven (A1).**

## Architecture & deployment

- **Elastic:** **self-host** (ELv2, distributed ES cluster — operationally heavy: shards, replicas, JVM) or **Elastic Cloud Hosted** / **Serverless** (managed). K8s/Helm + on-prem.
- **Parallax:** single-binary self-host target, local-first, air-gap-capable, Apache-2.0.

**Verdict:** on **single-binary local-first + low-ops, Parallax's target beats Elastic's distributed cluster** (real operational edge — operating production Elasticsearch is famously heavy). On **managed SaaS + Serverless, Elastic wins.** On **self-host viability, both** (but Elastic's is heavy).

## Operational footprint

- **Elastic:** self-host = real cluster ops (JVM, shards, indices, capacity planning); Cloud/Serverless = zero ops.
- **Parallax:** single-binary target.

**Verdict:** on **self-host operator burden, Parallax's target is far lower** (Elastic is one of the heavier self-host stacks). On **SaaS zero-ops, Elastic wins.** This is a genuine Parallax-favorable axis (self-host simplicity) — but Parallax is pre-release.

## Scalability & performance

- **Elastic:** proven at hyperscale (one of the most-deployed search/log backends; massive production deployments). Specific numbers vendor; not independently measured.
- **Parallax:** unproven; benchmark-dependent.

**Verdict:** on **proven-at-scale, Elastic wins conclusively.**

## Security

- **Elastic:** SSO/SAML, RBAC, encryption, audit; **+ unified SIEM/security** (a major strength — Elastic Security is a real product). Mature.
- **Parallax:** SSO/RBAC/audit planned; redaction (A6) designed.

**Verdict:** on **shipped security (esp. the SIEM unification), Elastic wins decisively.**

## Privacy & compliance

- **Elastic:** SOC2/ISO27001/HIPAA/FedRAMP, data residency; self-host sovereignty.
- **Parallax:** none yet; data ownership via self-host.

**Verdict:** on **compliance, Elastic wins decisively.** On self-host sovereignty, both.

## Openness, licensing & vendor lock-in — a real Parallax edge (narrow)

- **Elastic:** **ELv2** (Elastic License v2) — source-available, self-hostable, **but you cannot offer it as a managed service** (the key restriction). Not OSI-open. Moderate lock-in (ES query + indices; the open OTel schema in helps).
- **Parallax:** **Apache-2.0**, fully open (OSI), no managed-service restriction, OTLP-native, portable bundle.

**Verdict:** on **license permissiveness, Parallax (Apache-2.0) edges Elastic (ELv2)** — ELv2's managed-service restriction is a real (if narrow) difference for users who care about OSI-openness or offering a hosted service. Both are self-hostable OSS-origin, so the gap is smaller than vs Datadog/LangSmith (closed). Honest: a narrow edge, not decisive.

## Pricing & economics — model + verify-flag

Elastic Observability pricing is **public** ([elastic.co/pricing](https://www.elastic.co/pricing)), **usage-based** across **Hosted** and **Serverless** plans (resolved pass 36 against the live page + [cubeapm 2026](https://cubeapm.com/blog/elastic-observability-pricing-and-review/) + [Serverless billing-dimensions docs](https://www.elastic.co/docs/deploy-manage/cloud-organization/billing/elastic-observability-billing-dimensions)):

| Component | Rate | Notes |
| --- | --- | --- |
| **Hosted Standard ingest** | **~$0.09 / GB** | e.g. 3,000 GB ≈ $270 |
| **Hosted Standard retention** | **~$0.019 / GB** | e.g. 3,000 GB ≈ $57 |
| **Serverless** | **GB-ingested + retention** (consumption-metered) | per official billing-dimensions docs |
| **APM add-on** | **~$31 / host / mo** | (~$15/host/mo infra-only) |
| **Small deployments** | **~$1,500–$8,000 / mo** | Standard/Gold; Platinum/Enterprise $10K+/mo |

Self-host ELv2 = **$0 software cost** (you operate the cluster). Hosted Standard tier starts ~$99 + usage. **Confirm exact live rates on [elastic.co/pricing](https://www.elastic.co/pricing)** (Elastic uses a calculator; the above are the documented 2026 components).

**Parallax pricing:** none public yet (pre-release).

**Honest cost read:** Elastic's cost reputation is mixed — powerful but the per-GB + cluster-resource model can be expensive at high log volume (a documented concern; the "search-engine-as-backend" overhead). Whether Parallax self-host is cheaper is benchmark-dependent/unmeasured, but Parallax's purpose-built-telemetry bet targets exactly this cost inefficiency. **Unproven.**

## Where Elastic plainly wins

- **Search-grade log analytics** (Lucene — the origin strength; full-text, analytics, ES|QL).
- **Unified observability + security (SIEM)** — one backend for both.
- OTLP-native (EDOT, native OTel schema) at parity.
- Proven-at-scale + mature + broad compliance (SOC2/ISO/HIPAA/FedRAMP).
- AI assistant + ES|QL + anomaly + error assistant.
- Self-host OSS (ELv2) viability.

## Where Parallax honestly edges Elastic

- **Self-host operational simplicity** — single-binary Rust vs Elastic's heavy distributed JVM cluster. *(Real, significant ops edge; Parallax pre-release.)*
- **Purpose-built telemetry-native storage** — GreptimeDB vs search-engine-as-backend (targets Elastic's cost/overhead at telemetry workloads). *(Design bet; unproven.)*
- **License permissiveness** — Apache-2.0 (OSI, no managed-service restriction) vs ELv2. *(Narrow but real.)*
- **Sentry-envelope compatibility** — Elastic has none. *(Real; Parallax shipped.)*
- **Fix-outcome loop + bounded/versioned/redacted bundle** — Elastic has neither. *(Thesis, unproven, A1.)*

> **Honest summary:** Elastic is a hyperscale, search-grade, obs+security-unified incumbent — far ahead of pre-release Parallax on breadth/scale/search/security/compliance. Parallax's defensible delta is **self-host operational simplicity** (Elastic is famously heavy to operate), **purpose-built-telemetry storage cost** (vs search-engine overhead — unproven), **Apache-vs-ELv2**, **Sentry-envelope**, and the **bounded+outcome bundle** (A1). The GreptimeDB-vs-Elasticsearch telemetry-cost question is a real, measurable opening.

## Open questions / what measurement would settle

- **A1 gate vs Elastic AI Error Assistant:** does a Parallax bundle beat Elastic-AI-assistant-as-context for coding-agent fix outcomes? Unproven.
- **GreptimeDB-vs-Elasticsearch telemetry cost/perf** — measured ingest/query/cost at a representative telemetry workload. Benchmark-dependent, unmeasured; a real opening given Elastic's search-engine overhead.
- **Elastic exact latest version + pricing rates** — **RESOLVED pass 36**: current major line = **Elasticsearch 9.x** (the v9 line; v8.x legacy); Hosted Standard ~$0.09/GB ingest + ~$0.019/GB retain; Serverless = GB + retention; APM ~$31/host. Still confirm exact live calculator rates for a specific workload.

## Sources (accessed 2026-07-17)

- [Elastic Observability](https://www.elastic.co/observability/); [OpenTelemetry/EDOT](https://www.elastic.co/observability/opentelemetry); [pricing](https://www.elastic.co/pricing).
- [cubeapm Elastic pricing & review 2026](https://cubeapm.com/blog/elastic-observability-pricing-and-review/).
- Parallax side: [decisions/storage-engine.md](../../decisions/storage-engine.md), [storage/greptimedb-vs-clickhouse/](../../storage/greptimedb-vs-clickhouse/), [validation/a1-bundle-value/](../../validation/a1-bundle-value/).
