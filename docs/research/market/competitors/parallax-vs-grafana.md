# Parallax vs Grafana Cloud / LGTM

> An unbiased, one-to-one comparison. Research date: **2026-07-17**.
> Sources: [Grafana pricing](https://grafana.com/pricing/), [Grafana Cloud docs](https://grafana.com/docs/grafana-cloud/), the in-repo [Tempo v3 architecture review](../../reference/grafana-tempo-v3-architecture-review.md) (2026-05-29), and 2026 third-party pricing analyses.
>
> **Bottom line up front:** Grafana Cloud / LGTM is the **largest OSS-origin full
> observability stack** and the strongest open-source full-platform competitor.
> On **breadth, the OSS component ecosystem, dashboards (its namesake), scale,
> OTLP-native ingest, and Cloud maturity, Grafana wins decisively over pre-release
> Parallax.** Parallax's honest edges are **self-host simplicity** (Grafana's
> self-hosted Mimir+Loki+Tempo+Pyroscope+Grafana stack is a heavy distributed
> system vs Parallax's single-binary target), **Apache-2.0 vs AGPLv3**,
> **native error-workflow** (Grafana has no Sentry-grade issue lifecycle), and
> the *unproven* bundle + fix-outcome thesis.

## What each product is

- **Grafana Cloud / LGTM** (Grafana Labs) — a managed observability platform built from the OSS stack: **Grafana** (visualization/dashboards), **Mimir** (metrics, Prometheus-compatible), **Loki** (logs), **Tempo** (traces), **Pyroscope** (continuous profiling), plus **k6** (load testing), **Alloy** (OTel collector), and app-observability/SLO/AI-insights layers. Sold as Grafana Cloud (SaaS, Free/Pro/Advanced/Enterprise) and self-hostable as OSS (AGPLv3) or Grafana Enterprise. OTLP-native ingest. The OSS telemetry components are the de-facto standard for self-hosted observability.
- **Parallax** — open-source (Apache-2.0), Rust-first, self-hostable **execution-context engine**: OTLP-native ingest of traces/logs/metrics + CLI/agent traces, derives owned `error_event`s, fingerprints, correlates into a typed evidence graph, serves bounded/redacted evidence bundles to humans and coding agents. GreptimeDB + Turso. **Pre-release.**

These overlap on **full-stack OTLP telemetry + self-host OSS**. Grafana is the broad platform; Parallax is a narrow context engine. Compare axis-by-axis.

## Current GA versions (pinned 2026-07-17)

Latest stable tags via the [github.com/grafana](https://github.com/grafana) releases API:

| Component | Latest GA tag | Released | Note |
| --- | --- | --- | --- |
| Grafana | **v13.1.0** | 2026-07-01 | the **v13** line (not 12.x) |
| Mimir | **mimir-3.1.3** | 2026-07-16 | metrics, Prometheus-compatible |
| Loki | **v3.7.3** | 2026-06-24 | logs |
| Tempo | **v2.10.7** | 2026-06-12 | traces — **v3 (Kafka-log/vParquet5) is reviewed in-repo but NOT yet a GA tag** |
| Pyroscope | **v2.1.1** | 2026-07-10 | continuous profiling |

> ⚠️ The signal/storage sections below reference **Tempo v3** features from the in-repo [architecture review](../../reference/grafana-tempo-v3-architecture-review.md) (2026-05-29). v3 is the **reviewed/next line, not the current GA release** (2.10.7 is). Do not read "Tempo v3" as the shipped product — it is forward-looking architecture.

## Signal coverage — Grafana is the full OSS stack

| Signal | Grafana Cloud (shipped) | Parallax (pre-release; ✅🧪=code-shipped) |
| --- | --- | --- |
| Traces / distributed tracing | ✅ Tempo (**GA v2.10.7**; v3 Kafka/vParquet5 path reviewed in-repo, not yet a GA tag) | ✅🧪 OTLP traces (shipped, pre-release) |
| Logs | ✅ Loki | ✅🧪 OTLP logs (shipped, pre-release) |
| Metrics | ✅ Mimir (Prometheus-compatible) | ✅🧪 OTLP metrics (shipped, pre-release) |
| Continuous profiling | ✅ Pyroscope | ❌ |
| Errors / exceptions | 🟡 (queryable span-events; **no native issue lifecycle**) | ✅🧪 derived `error_event` + fingerprint (shipped, pre-release) |
| Dashboards | ✅ Grafana — the market standard | ✅ minimal V1 (🏗) |
| Alerts / SLOs | ✅ mature (Alertmanager, on-call, SLOs) | 🟡 minimal (🏗) |
| Synthetics / RUM | ✅ Synthetics + Frontend (RUM) | ❌ |
| LLM / agent spans | 🟡 (app-observability + LLM tooling emerging) | ✅ (🏗) |

**Verdict:** Grafana's signal coverage is comprehensive and all shipped. Parallax is narrower. On coverage, **Grafana wins decisively.** The one structural Grafana gap: **no native error-issue workflow** (errors are queryable, not managed work items) — a cell where Sentry and Parallax's design differ from Grafana.

## Ingestion & transport — both OTLP-native

- **OTLP:** Grafana Cloud is genuinely **OTLP-native** — OTLP endpoints for traces/logs/metrics (Tempo/Loki/Mimir ingest OTLP via the OTel protocol / Alloy collector). Mimir accepts OTLP metrics natively (not just Prometheus remote-write). This is a real parity point: **both Parallax and Grafana are OTLP-native**; unlike Sentry (no OTLP metrics) or Datadog (OTLP→proprietary transform).
- **Collectors:** Alloy (Grafana's OTel collector, Apache-2.0) + standard OTel Collector. Parallax ingests directly (OTLP gateway).

**Verdict:** on OTLP-native ingest, **roughly tied** (both truly native). On collector/agent maturity, **Grafana wins** (Alloy is mature, shipped at scale).

## Storage architecture — same physics, different engines

- **Grafana:** object-storage-native columnar across the stack — Tempo (`vParquet5`), Mimir (chunks/blocks on object store), Loki (chunks), Pyroscope (profiles on object store). The [in-repo Tempo v3 review](../../reference/grafana-tempo-v3-architecture-review.md) documents Tempo 3.0's Kafka-log write path, live-store, TraceQL Metrics GA, and retroactive redaction as a block-rewrite job. Battle-tested at hyperscale (Grafana Labs' own Cloud + many large self-host deployments).
- **Parallax:** GreptimeDB (native OTLP tables) + Turso, single-binary. Same physics (columnar on object store) as Tempo/Mimir, different engine. Parallax's storage perf vs the Grafana stack is **benchmark-dependent and unproven.**

**Verdict:** on proven-at-scale + operational maturity, **Grafana wins conclusively.** Parallax's GreptimeDB-native design is newer and unproven. (The in-repo review explicitly treats Tempo as an *architectural reference to borrow from* — Kafka-WAL, Parquet, redaction-as-job — not a product Parallax must beat.)

## Query & correlation

- **Grafana:** best-in-class cross-signal correlation *via dashboards* (panel→drill, exemplars, trace-to-logs, Loki/Tempo/Mimir interlinks) + Grafana's exploration. Strong for human investigation across signals.
- **Parallax:** evidence-graph correlation + bounded bundle for agents. The *agent-actionable* abstraction is differentiated but **unproven (A1).**

**Verdict:** on **human cross-signal investigation via dashboards, Grafana wins decisively** (it defines the category). Parallax's bundle is a different axis (agent context), unproven.

## Dashboards & visualization — Grafana's moat

- **Grafana:** the namesake — the most widely adopted visualization/dashboard layer in observability. Templated dashboards, alert panels, Canvas, Geomap, and an enormous library of community dashboards. **Grafana v13.1.0** (2026-07-01) — the v13 line.
- **Parallax:** V1 UI = Sentry-grade issues + predefined/user dashboards (TanStack/shadcn, React Flow for graph viz). Intentionally minimal.

**Verdict:** **Grafana wins decisively.** This is Grafana's defining strength; Parallax does not compete on visualization breadth.

## Error tracking & workflow — a Grafana gap

- **Grafana:** errors are queryable span-events/logs — **no native issue lifecycle** (no resolve/regress/ignore/assign, no ownership rules, no suspect-commits). Grafana pairs with Sentry or relies on the user.
- **Parallax:** derives `error_event` + deterministic fingerprint (**shipped**) + fix-outcome offline residual (**plan 123 DONE**; live value **unproven**).

**Verdict:** on **error-issue workflow, Parallax's design targets a real Grafana gap** (Grafana has none; Sentry owns this). Parallax **ships** error derivation + fingerprint; fix-outcome offline residual plan 123 DONE; live value **unproven.** An honest Parallax-favorable axis, scoped + gated.

## AI-native / agent-context story

- **Grafana's AI:** app-observability insights, anomaly detection/forecasting, Sift (root-cause exploration), Grafana LLM features (query/natural-language), and emerging LLM/agent tooling. A human-dashboard + assistive AI; **not a bounded, read-only, agent-context projection.**
- **Parallax's claim:** bounded, redacted, agent-safe evidence bundle for coding agents (**code-shipped**, A1 value unproven gate).

**Verdict:** Grafana ships more AI today (insights, forecasting, NL query). Parallax's differentiated agent-context claim is **unproven (A1).** Neither serves the exact "safe bounded context for autonomous coding agents" cell — Parallax's thesis.

## Architecture & deployment model

- **Grafana Cloud:** managed SaaS (Free/Pro/Advanced/Enterprise), multi-region. **Self-host OSS:** the full Mimir+Loki+Tempo+Pyroscope+Grafana stack (AGPLv3) — a **distributed, multi-component system** (ingesters, distributors, queriers, compactors, store-gateways per signal) requiring real SRE. Grafana Enterprise adds RBAC/SSO/licensing on the viz layer.
- **Parallax:** single-binary self-host target, local-first, air-gap-capable, Apache-2.0.

**Verdict:** on **self-host simplicity, Parallax's single-binary target beats Grafana's distributed OSS stack** (operating Mimir+Loki+Tempo+Pyroscope+Grafana in prod is heavy — this is a real Parallax wedge against self-hosted Grafana). On **managed SaaS scale/maturity, Grafana Cloud wins.**

## Operational footprint

- **Grafana Cloud:** zero backend ops. **Grafana self-host:** non-trivial — multiple stateful distributed services per signal; documented operational burden (ingester capacity, compaction, object-store config).
- **Parallax:** self-hosted GreptimeDB + Turso + engine; single-binary target lowers burden.

**Verdict:** **Grafana Cloud wins on zero-ops; Parallax's target wins on self-host simplicity** vs Grafana's distributed stack. Scoped.

## Scalability & performance

- **Grafana:** proven at hyperscale (Cloud + large self-host). Tempo/Mimir/Loki are designed for very large volume. Specific numbers vendor/marketing; not independently measured here.
- **Parallax:** unproven at production scale; **benchmark-dependent.**

**Verdict:** on **proven-at-scale, Grafana wins conclusively.** Parallax cannot yet make a measured scale claim. (Flagged for the benchmark program — this is a 4-build, GreptimeDB-vs-ClickHouse-adjacent comparison.)

## Security

- **Grafana Cloud:** SSO/SAML, RBAC, fine-grained access, audit; Grafana Enterprise adds advanced RBAC. Mature.
- **Parallax:** SSO/RBAC/audit planned, not shipped; redaction (A6) designed.

**Verdict:** on **shipped security posture, Grafana wins decisively.**

## Privacy & compliance

- **Grafana Cloud:** SOC 2, ISO 27001, GDPR, HIPAA-eligible; data residency. Mature.
- **Parallax:** none yet (pre-release). Data ownership via self-host.

**Verdict:** on **compliance certifications, Grafana wins decisively.** On **data sovereignty (self-host, air-gap), Parallax wins by design** — though Grafana self-host OSS also satisfies sovereignty, so this edge is weaker vs Grafana than vs Datadog.

## Openness, licensing & vendor lock-in

- **Grafana:** the telemetry components (Mimir/Loki/Tempo/Pyroscope) are **AGPLv3**; Grafana itself AGPLv3 core + Grafana Enterprise (commercial); Alloy Apache-2.0. **AGPLv3 is materially less permissive than Apache-2.0** (network-use copyleft clause — a real consideration for embedding/modifying). Self-host OSS is fully viable; lock-in is moderate (standard OTel formats in, Grafana dashboards out; dashboards are portable-ish).
- **Parallax:** Apache-2.0, fully open, OTLP-native, portable bundle.

**Verdict:** on **license permissiveness, Parallax (Apache-2.0) edges Grafana (AGPLv3)** — a real, if narrow, difference for users who care about copyleft/network-use terms. On self-host viability and OTLP-native openness, both are strong.

## Extensibility

- **Grafana:** the largest integration ecosystem in OSS observability — 100s of data sources, dashboard plugins, alerting integrations, Grafana Alloy/Agent, Terraform, public API, Alertmanager. Deepest OSS ecosystem.
- **Parallax:** OTel-native, CLI/HTTP/MCP, pipeline/processor, webhooks (planned).

**Verdict:** on **ecosystem breadth, Grafana wins decisively.**

## Pricing & economics — real numbers

Grafana Cloud pricing is **public** ([grafana.com/pricing](https://grafana.com/pricing/), accessed 2026-07-17):

| Plan | Price | Notes |
| --- | --- | --- |
| **Free** | $0 | generous free tiers (metrics/logs/traces/profiles/users) |
| **Pro** | **$195/mo** base (self-serve) | included usage + usage-based overage |
| **Advanced / Cloud Enterprise** | custom | scale, security, support |

Billable units (Pro overage, per 2026 sources): **metrics ~$6.50 / 1,000 active series** (10k free); **logs/traces/profiles ~$0.45–0.50 / GB**; **active visualization users ~$8 each**. **Application Observability** host-hour model (~**$0.04 / host-hour** for pre-2026-02-13 customers, with included series/trace credits). Sources disagree slightly on exact GB/series rates ([cloudzero](https://cloudzero.com/blog/grafana-cloud-pricing/), [monitoringcost](https://monitoringcost.com/grafana-cloud-pricing), [cubeapm](https://cubeapm.com/blog/grafana-cloud-pricing-and-review/)) — list-vs-contract; **confirm exact current rate on grafana.com before quoting as precise.**

**Self-host OSS:** **free (AGPLv3)** — unlimited, you operate the stack.

**Parallax pricing:** none public yet (pre-release). Stated shape: Apache-2.0 open core + gated enterprise-ops + managed cloud + outcome-priced fixer.

**Honest cost read:** Grafana Cloud's free tier + cheap-ish usage pricing is competitive, and self-host OSS is free. Whether Parallax self-host is cheaper at a given workload is **benchmark-dependent and unmeasured.** Grafana's cost reputation is generally better than Datadog's, so Parallax's cost edge is weaker vs Grafana than vs Datadog.

## Where Grafana plainly wins

- Full OSS telemetry stack breadth (metrics+logs+traces+profiles, all shipped).
- Dashboards/visualization (market standard).
- OTLP-native ingest at parity.
- OSS component ecosystem + integrations (deepest in OSS observability).
- Proven-at-scale + operational maturity (Cloud + self-host).
- Cloud compliance (SOC2/ISO27001/HIPAA) + SSO/RBAC.
- AI insights / forecasting / NL query (more shipped than Parallax).

## Where Parallax honestly edges Grafana

- **Self-host simplicity** — single-binary vs Grafana's distributed Mimir+Loki+Tempo+Pyroscope+Grafana stack. *(Real operational wedge; Parallax pre-release.)*
- **Native error-issue workflow** — Grafana has none; Parallax plans derived errors + fingerprint + outcome loop. *(Real Grafana gap; Parallax error derivation **shipped**; fix-outcome offline residual plan 123 DONE; live value unproven.)*
- **License permissiveness** — Apache-2.0 vs AGPLv3 (network-use copyleft). *(Narrow but real.)*
- **Bounded, redacted, agent-safe evidence bundle + fix-outcome loop** — unoccupied cells. *(Thesis, **unproven** — A1 gate.)*

## Open questions / what measurement would settle

- **A1 gate vs Grafana:** for a team on Grafana Cloud + Sentry, does a Parallax bundle measurably improve coding-agent fix outcomes for incidents? Unproven.
- **Self-host cost/ops parity:** measured single-binary Parallax vs self-hosted Mimir+Loki+Tempo+Pyroscope (deploy complexity, RAM, ops). Benchmark-dependent, unmeasured.
- ~~Grafana latest versions~~ → **pinned 2026-07-17 pass 5b**: Grafana v13.1.0, Mimir mimir-3.1.3, Loki v3.7.3, Tempo v2.10.7, Pyroscope v2.1.1 (see the "Current GA versions" table). **Tempo v3 is reviewed/in-development, not a GA tag** — corrected two prior claims ("Grafana 12.x"→13.1.0; "Tempo v3.x shipped"→GA is 2.10.7).

## Sources (accessed 2026-07-17)

- [Grafana pricing](https://grafana.com/pricing/); [Grafana Cloud docs](https://grafana.com/docs/grafana-cloud/); [app-observability pricing](https://grafana.com/docs/grafana-cloud/monitor-applications/application-observability/pricing/).
- In-repo: [grafana-tempo-v3-architecture-review.md](../../reference/grafana-tempo-v3-architecture-review.md) (Tempo 3.0 cut, 2026-05-29).
- 2026 pricing analyses: [cloudzero](https://cloudzero.com/blog/grafana-cloud-pricing/), [monitoringcost](https://monitoringcost.com/grafana-cloud-pricing), [cubeapm](https://cubeapm.com/blog/grafana-cloud-pricing-and-review/).
- Parallax side: [decisions/storage-engine.md](../../decisions/storage-engine.md), [storage/greptimedb-vs-clickhouse/](../../storage/greptimedb-vs-clickhouse/), [validation/a1-bundle-value/](../../validation/a1-bundle-value/).
