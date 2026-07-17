# Parallax vs Uptrace

> An unbiased, one-to-one comparison. Research date: **2026-07-17**.
> Sources: [uptrace.dev](https://uptrace.dev/) + [editions](https://uptrace.dev/editions) + [pricing-update April 2026](https://uptrace.dev/blog/pricing-update-april-2026), [github.com/uptrace/uptrace](https://github.com/uptrace/uptrace), [OneUptime setup guide](https://oneuptime.com/blog/post/2026-02-06-setup-uptrace-opentelemetry-backend/view).
>
> **Bottom line up front:** Uptrace is an **open-source (AGPL), OTLP-native,
> tracing-first APM** on ClickHouse+Postgres — same family as SigNoz/OpenObserve/HyperDX
> but **tracing-centric**, Go-based (Bun-author lineage), and notably cheap (60–90%-
> cheaper-than-legacy claim). On **tracing-first APM, ClickHouse+Postgres maturity,
> AGPL self-host-free, and cost, Uptrace is ahead of pre-release Parallax.** Parallax's
> honest edges are **Apache-2.0 vs AGPL**, **GreptimeDB-native** (vs ClickHouse),
> **Sentry-envelope**, and the *unproven* bounded agent bundle (A1).

## What each product is

- **Uptrace** — an **open-source (AGPL), OTLP-native APM** built on **ClickHouse** (traces/metrics/logs; v2.0 uses ClickHouse's JSON type for ~10× speed) + **PostgreSQL** (metadata: users/projects/dashboards/metric-defs). **Tracing-first** (distributed traces + flame graphs + span analysis), with metrics + logs. Go-based (maintained by the author of the Bun framework / go-pg). Self-host (Docker Compose: Uptrace+ClickHouse+Postgres) or Cloud. **AGPL** Community free unlimited; paid self-host/on-prem editions (Starter $39 / Team $199 / Business $499 per mo); Cloud ~$0.05–0.10/GB. Positions as a cheap Datadog/New-Relic alternative (60–90% cheaper claim).
- **Parallax** — open-source (Apache-2.0), Rust-first, self-hostable **execution-context engine**: OTLP-native ingest of traces/logs/metrics + CLI/agent traces, derives owned `error_event`s, fingerprints, correlates into a typed evidence graph, serves bounded/redacted evidence bundles to humans and coding agents. GreptimeDB + Turso. **Pre-release.**

Both OSS, OTLP-native, ClickHouse-adjacent, self-hostable. Uptrace is a tracing-first APM (Bun-author lineage); Parallax is a GreptimeDB-native agent-context engine.

## Signal coverage

| Signal | Uptrace (shipped) | Parallax (planned/shipped) |
| --- | --- | --- |
| Traces / distributed tracing | ✅ **(the core — flame graphs, span analysis)** | ✅🧪 OTLP traces (shipped, pre-release) |
| Metrics | ✅ | ✅🧪 OTLP metrics (shipped, pre-release) |
| Logs | ✅ | ✅🧪 OTLP logs (shipped, pre-release) |
| Errors / exceptions | 🟡 (queryable; no Sentry-grade lifecycle) | ✅ derived `error_event` + fingerprint (🧪 shipped) |
| Dashboards / alerting | ✅ | 🟡 minimal (🏗) |
| Sentry envelope / DSN | ❌ | ✅ shipped |

**Verdict:** Uptrace's coverage is solid and shipped, tracing-first. On coverage, **Uptrace wins.** Parallax ships Sentry-envelope (Uptrace none).

## Ingestion & transport

- **OTLP/OTel:** Uptrace is **OTLP-native** (traces/metrics/logs via OTel into ClickHouse). Same native stance as Parallax.
- **SDKs:** multi-language via OTel instrumentations.
- **Sentry envelope:** none.

**Verdict:** on OTLP-native ingest, **tied in design; Uptrace ships it.** On Sentry-envelope, **Parallax wins** (shipped; Uptrace none).

## Storage architecture

- **Uptrace:** **ClickHouse** (telemetry; v2.0 JSON-type ~10× speed) + **PostgreSQL** (metadata). Docker-Compose-simple.
- **Parallax:** **GreptimeDB** (native OTLP tables) + **Turso**, single-binary.

**Verdict:** on **ClickHouse-for-APM maturity + the Postgres-metadata split, Uptrace wins** (shipped). Parallax's **GreptimeDB-native + Turso** is a different (unproven) engine choice. GreptimeDB-vs-ClickHouse-APM is benchmark-dependent/unmeasured.

## Query & correlation

- **Uptrace:** trace-centric (flame graphs, span analysis) + metrics + logs + dashboards; mature tracing-first APM query.
- **Parallax:** evidence-graph correlation + bounded bundle for agents.

**Verdict:** on **tracing-first APM query, Uptrace wins** (mature). Parallax's bundle is a different axis (bounded agent context), unproven (A1).

## Error tracking & workflow

- **Uptrace:** errors are queryable; **no native Sentry-grade error-issue lifecycle.**
- **Parallax:** derived `error_event` + fingerprint (**shipped**) + fix-outcome offline residual (**plan 123 DONE**; live value **unproven**).

**Verdict:** on **error-issue workflow, Parallax ships error derivation + fingerprint** (pre-release); fix-outcome offline residual plan 123 DONE, live value **unproven**.

## AI-native / agent-context story

- **Uptrace:** no significant AI/agent surface (traditional APM). Not an agent-context engine.
- **Parallax's claim:** bounded, redacted, agent-safe evidence bundle for coding agents (**code-shipped**, A1 value unproven gate).

**Honest verdict:** Uptrace has no AI/agent story. Parallax's differentiated agent-context claim is **unproven (A1)** — Uptrace doesn't occupy that cell.

## Architecture & deployment

- **Uptrace:** **self-host OSS (AGPL)** free unlimited (Docker Compose / K8s / on-prem) or Cloud + paid editions. Go.
- **Parallax:** single-binary self-host target, local-first, air-gap-capable, Apache-2.0, Rust.

**Verdict:** both OSS + self-hostable. **Uptrace ships; Parallax pre-release.** Parallax's **Rust single-binary + Apache** vs Uptrace's **Go + AGPL + multi-container**.

## Scalability / Security / compliance

- **Uptrace:** proven for small-to-medium APM scale; SSO/RBAC on paid editions; self-host = your posture. Compliance posture modest.
- **Parallax:** unproven at scale; SSO/RBAC/audit planned; redaction (A6) designed.

**Verdict:** on **shipped maturity, Uptrace wins** (modest scale).

## Openness, licensing & vendor lock-in

- **Uptrace:** **AGPL** (Community free unlimited self-host; network-use copyleft — same consideration as OpenObserve). Paid editions for commercial self-host support. Standard OTLP in. Moderate lock-in.
- **Parallax:** **Apache-2.0**, fully open (OSI, no copyleft network-use clause), OTLP-native, portable bundle.

**Verdict:** on **license permissiveness, Parallax (Apache-2.0) edges Uptrace (AGPL)** — a real (if narrow) difference for users who care about AGPL's network-use terms. Both self-hostable.

## Pricing & economics — real numbers

| Edition | Price | Capacity |
| --- | --- | --- |
| **Community (AGPL self-host)** | **$0** | unlimited |
| **Starter (self-host/on-prem)** | **$39/mo** | ≤50 GB/day |
| **Team** | **$199/mo** | ≤200 GB/day |
| **Business** | **$499/mo** | higher |
| **Cloud** | ~**$0.05–0.10/GB** ingested | + per-timeseries metrics |

**⚠️ April 10 2026: prices rose up to 15% (Hetzner infra costs); metrics shifted to per-timeseries.** Sources: [pricing-update April 2026](https://uptrace.dev/blog/pricing-update-april-2026), [editions](https://uptrace.dev/editions). **60–90%-cheaper-than-legacy claim.** No per-host/per-seat (APM included in ingest).

**Parallax pricing:** none public yet (pre-release); self-host = no per-event tax by design.

**Honest cost read:** Uptrace is genuinely cheap (AGPL free unlimited self-host + $0.05–0.10/GB cloud + 60–90%-cheaper claim). Whether Parallax self-host is cheaper is benchmark-dependent/unmeasured — Uptrace is a strong cost-positioned OSS APM.

## Where Uptrace plainly wins

- **Tracing-first APM** (flame graphs, span analysis — the core, mature).
- ClickHouse+Postgres stack + OTLP-native + AGPL-self-host-free.
- Cheap (60–90%-cheaper claim; no per-seat) + deployment flexibility (Docker/K8s/on-prem/cloud).
- Bun-author lineage (Go expertise).

## Where Parallax honestly edges Uptrace

- **License permissiveness** — Apache-2.0 vs AGPL (network-use copyleft). *(Real, narrow.)*
- **Engine choice (GreptimeDB-native telemetry)** — vs ClickHouse (unproven advantage).
- **Sentry-envelope compatibility** — Uptrace has none; Parallax ships it. *(Real.)*
- **Production error events + fix-outcome loop** — Uptrace has neither. *(Real: error events **shipped**; fix-outcome offline residual plan 123 DONE; live value **unproven**.)*
- **Bounded, redacted, agent-safe evidence bundle** — Uptrace has none. *(Thesis, unproven, A1.)*

> **Honest summary:** Uptrace is a solid **OSS tracing-first APM** (AGPL, ClickHouse+Postgres, OTLP-native, cheap, Bun-author lineage) — same family as SigNoz/OpenObserve/HyperDX but tracing-centric. Ahead of pre-release Parallax on tracing-APM maturity, ClickHouse+Postgres, AGPL-self-host-free, cost. Parallax's defensible delta is **Apache-vs-AGPL**, **GreptimeDB-native** (vs ClickHouse, unproven), **Sentry-envelope**, **prod-error + outcome loop**, and the **bounded+outcome bundle** (A1 unproven). Lower strategic priority than SigNoz/OpenObserve (more niche), but completes the OSS-ClickHouse-OTLP-platform coverage.

## Open questions / what measurement would settle

- **A1 gate:** does a Parallax bundle add value beyond Uptrace's tracing-APM for coding-agent incident fixes? Unproven.
- **GreptimeDB-vs-ClickHouse APM** — measured cost/perf (Uptrace's v2.0 JSON-type 10× claim is a ClickHouse strength to beat).

## Sources (accessed 2026-07-17)

- [uptrace.dev](https://uptrace.dev/); [editions](https://uptrace.dev/editions); [pricing-update April 2026](https://uptrace.dev/blog/pricing-update-april-2026).
- [github.com/uptrace/uptrace](https://github.com/uptrace/uptrace); [OneUptime OTel-backend setup](https://oneuptime.com/blog/post/2026-02-06-setup-uptrace-opentelemetry-backend/view).
- Parallax side: [decisions/storage-engine.md](../../decisions/storage-engine.md), [storage/greptimedb-vs-clickhouse/](../../storage/greptimedb-vs-clickhouse/), [validation/a1-bundle-value/](../../validation/a1-bundle-value/).
- Sibling (same family): [parallax-vs-signoz.md](parallax-vs-signoz.md), [parallax-vs-openobserve.md](parallax-vs-openobserve.md), [parallax-vs-hyperdx.md](parallax-vs-hyperdx.md).
