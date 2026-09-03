# Parallax vs Grafana Cloud / LGTM

## Current live verification — 2026-09-04

Grafana LGTM `0.32.0` was run as the current bundled stack. Grafana API health,
Tempo trace search, Loki labels, Prometheus metrics, and browser Explore UI all
returned fresh workload evidence. Grafana remains the strongest generic query,
dashboard, and visualization reference. Exact evidence: [canonical report](../../validation/2026-09-04-parallax-main-competitor-verification.md).

The dated pass notes below preserve historical claims; this section is the
current version authority.

> An unbiased, one-to-one comparison. Research date: **2026-07-17**.
> Sources: [Grafana pricing](https://grafana.com/pricing/), [Grafana Cloud docs](https://grafana.com/docs/grafana-cloud/), the in-repo [Tempo v3 architecture review](../../reference/grafana-tempo-v3-architecture-review.md) (2026-05-29), and 2026 third-party pricing analyses.
>
> **Bottom line up front:** Grafana Cloud / LGTM is the **largest OSS-origin full
> observability stack** and the strongest open-source full-platform competitor.
> On **breadth, the OSS component ecosystem, dashboards (its namesake), scale,
> OTLP-native ingest, and Cloud maturity, Grafana wins decisively over pre-release
> Parallax.** Parallax's architectural deployment target is a single binary, while
> Grafana's self-hosted Mimir+Loki+Tempo+Pyroscope+Grafana stack is distributed; any
> operational advantage remains unmeasured. Other comparison axes are **Apache-2.0 vs AGPLv3**,
> **native error-workflow** (Grafana has no Sentry-grade issue lifecycle), and
> the *unproven* bundle + fix-outcome thesis.

## What each product is

- **Grafana Cloud / LGTM** (Grafana Labs) — a managed observability platform built from the OSS stack: **Grafana** (visualization/dashboards), **Mimir** (metrics, Prometheus-compatible), **Loki** (logs), **Tempo** (traces), **Pyroscope** (continuous profiling), plus **k6** (load testing), **Alloy** (OTel collector), and app-observability/SLO/AI-insights layers. Sold as Grafana Cloud (SaaS, Free/Pro/Advanced/Enterprise) and self-hostable as OSS (AGPLv3) or Grafana Enterprise. OTLP-native ingest. The OSS telemetry components are the de-facto standard for self-hosted observability.
- **Parallax** — open-source (Apache-2.0), Rust-first, self-hostable **execution-context engine**: OTLP-native ingest of traces/logs/metrics + CLI/agent traces, derives owned `error_event`s, fingerprints, correlates into a typed evidence graph, serves bounded/redacted evidence bundles to humans and coding agents. GreptimeDB + Turso. **Pre-release.**

These overlap on **full-stack OTLP telemetry + self-host OSS**. Grafana is the broad platform; Parallax is a narrow context engine. Compare axis-by-axis.

## Current GA versions (pinned 2026-07-17, pass 49)

Latest stable tags via the [github.com/grafana](https://github.com/grafana) releases API:

| Component | Latest GA tag | Released | Note |
| --- | --- | --- | --- |
| Grafana | **v13.1.0** | 2026-07-01 | the **v13** line (not 12.x) |
| Mimir | **mimir-3.1.3** | 2026-07-16 | metrics, Prometheus-compatible |
| Loki | **v3.7.3** | 2026-06-24 | logs |
| Tempo | **v3.0.2** | 2026-06-09 | traces — **v3 GA** (was wrongly listed as v2.10.7 through pass 48) |
| Pyroscope | **v2.1.1** | 2026-07-10 | continuous profiling |

> **Pass 49 correction:** Tempo **v3.0.0** shipped **2026-05-28**; latest patch **v3.0.2** (2026-06-09). Prior deep-dive claimed “v3 not yet a GA tag / GA is 2.10.7” — **stale**. [v3.0.0 highlights](https://github.com/grafana/tempo/releases/tag/v3.0.0): new ingest/write architecture (legacy ingesters removed), **TraceQL metrics GA**, vParquet5, **trace redaction**, span profiling via otelpyroscope, migration tooling. Breaking 2.x→3.0 upgrade. In-repo [architecture review](../../reference/grafana-tempo-v3-architecture-review.md) (2026-05-29) now describes **shipped** architecture, not forward-looking only.

## Signal coverage — Grafana is the full OSS stack

| Signal | Grafana Cloud (shipped) | Parallax (pre-release; ✅🧪=code-shipped) |
| --- | --- | --- |
| Traces / distributed tracing | ✅ Tempo (**GA v3.0.2** — Kafka-log write path, TraceQL metrics GA, vParquet5, trace redaction) | ✅🧪 OTLP traces (shipped, pre-release) |
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

- **Grafana:** object-storage-native columnar across the stack — Tempo **v3 GA** (`vParquet5`, Kafka-log write path, live-store, TraceQL Metrics GA, trace redaction), Mimir (chunks/blocks on object store), Loki (chunks), Pyroscope (profiles on object store). Documented in [Tempo v3.0 release notes](https://grafana.com/docs/tempo/latest/release-notes/v3-0/) + in-repo [architecture review](../../reference/grafana-tempo-v3-architecture-review.md). Battle-tested at hyperscale (Grafana Labs' own Cloud + many large self-host deployments).
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

- **Grafana's AI (pass 48–49):** app-observability insights, anomaly/forecasting, Sift, NL query, plus **Grafana Assistant** (copilot; Pro includes **3 active AI users** / 40M tokens each + 25M service-account tokens; then **$20/active AI user** + **$2/1M tokens** — live [pricing](https://grafana.com/pricing/)). Docs: **Assistant Investigations** public preview **no charge** (billing for Assistant usage started 2026-01-01). Human-dashboard + assistive investigation AI; **not** a portable redacted coding-agent evidence bundle.
- **Parallax's claim:** bounded, redacted, agent-use (safety/value unproven) evidence bundle for coding agents (**code-shipped**, A1 value unproven gate).

**Verdict:** Grafana ships more AI today (Assistant + Investigations preview + insights/forecasting/NL). Parallax's differentiated agent-context claim is **unproven (A1).** Neither serves the exact "safe bounded context for autonomous coding agents" cell — Parallax's thesis.

## Architecture & deployment model

- **Grafana Cloud:** managed SaaS (Free/Pro/Advanced/Enterprise), multi-region. **Self-host OSS:** the full Mimir+Loki+Tempo+Pyroscope+Grafana stack (AGPLv3) — a **distributed, multi-component system** (ingesters, distributors, queriers, compactors, store-gateways per signal) requiring real SRE. Grafana Enterprise adds RBAC/SSO/licensing on the viz layer.
- **Parallax:** single-binary self-host target, local-first, offline/local deployment target (air-gap unverified), Apache-2.0.

**Verdict:** Grafana's self-hosted OSS stack is distributed, while Parallax's single-binary deployment is an architectural target. Comparative self-host operations and simplicity are **unverified**. On **managed SaaS scale/maturity, Grafana Cloud wins.**

## Operational footprint

- **Grafana Cloud:** zero backend ops. **Grafana self-host:** non-trivial — multiple stateful distributed services per signal; documented operational burden (ingester capacity, compaction, object-store config).
- **Parallax:** self-hosted GreptimeDB + Turso + engine; single-binary target lowers burden.

**Verdict:** **Grafana Cloud wins on zero-ops.** Parallax's lower-ops self-hosting is an architectural target; comparative simplicity remains unverified.

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

**Verdict:** on **compliance certifications, Grafana wins decisively.** Both products have self-host/local deployment paths; Parallax's air-gap capability is unverified, so no comparative sovereignty edge is established.

## Openness, licensing & vendor lock-in

- **Grafana:** the telemetry components (Mimir/Loki/Tempo/Pyroscope) are **AGPLv3**; Grafana itself AGPLv3 core + Grafana Enterprise (commercial); Alloy Apache-2.0. **AGPLv3 is materially less permissive than Apache-2.0** (network-use copyleft clause — a real consideration for embedding/modifying). Self-host OSS is fully viable; lock-in is moderate (standard OTel formats in, Grafana dashboards out; dashboards are portable-ish).
- **Parallax:** Apache-2.0, fully open, OTLP-native, portable bundle.

**Verdict:** on **license permissiveness, Parallax (Apache-2.0) edges Grafana (AGPLv3)** — a real, if narrow, difference for users who care about copyleft/network-use terms. On self-host viability and OTLP-native openness, both are strong.

## Extensibility

- **Grafana:** the largest integration ecosystem in OSS observability — 100s of data sources, dashboard plugins, alerting integrations, Grafana Alloy/Agent, Terraform, public API, Alertmanager. Deepest OSS ecosystem.
- **Parallax:** OTel-native, CLI/HTTP/MCP, pipeline/processor, webhooks (planned).

**Verdict:** on **ecosystem breadth, Grafana wins decisively.**

## Pricing & economics — RESOLVED pass 48

Grafana Cloud pricing is **public** ([grafana.com/pricing](https://grafana.com/pricing/), **live 2026-07-17**):

| Plan | Price | Notes |
| --- | --- | --- |
| **Free** | **$0 always** | limited usage; 14-day retention; community support |
| **Pro** | **from $19 / mo + usage** | ⚠️ pass-5 **$195/mo Pro base is STALE/WRONG** |
| **Enterprise** | **starts $25,000 / year** spend commit | premium support, custom retention, Public/Federal/BYOC |

| Signal / product | Pro starts-at (live) | Included w/ $19 platform then PAYG |
| --- | --- | --- |
| **Metrics** | **$6.50 / 1k active series** | 10k series; 13-mo retention |
| **Logs / Traces / Profiles** | **$0.05 Process + $0.40 Write + $0.10 Retain per GB** | 50 GB/mo; 30-day retention |
| **App Observability** | **$0.025 / host-hour** (~$18/host) | 2,232 host-hours |
| **K8s Monitoring** | **$0.01 / host-hour** + **$0.0007 / container-hour** | host+container hours included |
| **Grafana visualization** | **$8 / active user** | 3 users |
| **+ Enterprise plugins** | **$55 / active user** | 3 users |
| **Grafana Assistant (AI)** | **$20 / AI user** (40M tokens) + **$2 / 1M tokens** | 3 AI users |
| **IRM** | **$20 / IRM user** | 3 users |
| **Frontend Obs** | **$0.75 / 1k sessions** | 50k sessions |
| **k6** | **$0.15 / VU-hour** | 500 VU-hours |

**Self-host OSS:** **free (AGPLv3)** — you operate the stack.

**Parallax pricing:** **no public number** (pre-release).

**Honest cost read:** Live Pro base is **much cheaper than our pass-5 $195 figure** — favors Grafana transparency. Process+Write+Retain can sum **above** old ~$0.50/GB flat proxies. Parallax cost edge vs Grafana is **weaker** than vs Datadog and **unmeasured**.

## Where Grafana plainly wins

- Full OSS telemetry stack breadth (metrics+logs+traces+profiles, all shipped).
- Dashboards/visualization (market standard).
- OTLP-native ingest at parity.
- OSS component ecosystem + integrations (deepest in OSS observability).
- Proven-at-scale + operational maturity (Cloud + self-host).
- Cloud compliance (SOC2/ISO27001/HIPAA) + SSO/RBAC.
- AI insights / forecasting / NL query (more shipped than Parallax).

## Where Parallax honestly edges Grafana

- **Deployment shape** — Parallax targets a single binary vs Grafana's distributed Mimir+Loki+Tempo+Pyroscope+Grafana stack. *(Comparative operational benefit is unverified; Parallax pre-release.)*
- **Native error-issue workflow** — Grafana has none; Parallax **ships** derived errors + fingerprint; fix-outcome offline residual plan **123 DONE**. *(Real Grafana gap; Parallax error derivation **shipped**; fix-outcome offline residual plan 123 DONE; live value unproven.)*
- **License permissiveness** — Apache-2.0 vs AGPLv3 (network-use copyleft). *(Narrow but real.)*
- **Bounded, redacted, agent-use (safety/value unproven) evidence bundle + fix-outcome loop** — unoccupied cells. *(Thesis, **unproven** — A1 gate.)*

## Open questions / what measurement would settle

- **A1 gate vs Grafana:** for a team on Grafana Cloud + Sentry, does a Parallax bundle measurably improve coding-agent fix outcomes for incidents? Unproven.
- **Self-host cost/ops parity:** measured single-binary Parallax vs self-hosted Mimir+Loki+Tempo+Pyroscope (deploy complexity, RAM, ops). Benchmark-dependent, unmeasured.
- ~~Grafana latest versions~~ → pass **49**: Grafana **v13.1.0**, Mimir **3.1.3**, Loki **3.7.3**, Tempo **v3.0.2 GA**, Pyroscope **v2.1.1** (GitHub releases API). Prior “Tempo v3 not GA” claim **corrected**.
- ~~Grafana Cloud Pro $195~~ → **RESOLVED pass 48: from $19/mo + usage** on live page.

## Sources (accessed 2026-07-17; pass 49)

- Live [grafana.com/pricing](https://grafana.com/pricing/); [Assistant pricing docs](https://grafana.com/docs/grafana-cloud/machine-learning/assistant/pricing/).
- [Grafana Cloud docs](https://grafana.com/docs/grafana-cloud/).
- [Tempo v3.0.0 release](https://github.com/grafana/tempo/releases/tag/v3.0.0); [v3.0.2](https://github.com/grafana/tempo/releases/tag/v3.0.2); [Tempo v3.0 release notes](https://grafana.com/docs/tempo/latest/release-notes/v3-0/).
- In-repo: [grafana-tempo-v3-architecture-review.md](../../reference/grafana-tempo-v3-architecture-review.md).
- Secondary pricing blogs demoted where they conflict with live $19 Pro base.
- Parallax side: [decisions/storage-engine.md](../../decisions/storage-engine.md), [storage/greptimedb-vs-clickhouse/](../../storage/greptimedb-vs-clickhouse/), [validation/a1-bundle-value/](../../validation/a1-bundle-value/).
