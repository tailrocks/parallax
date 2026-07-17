# Parallax vs HyperDX (ClickStack)

> One-to-one comparison. **No pro-Parallax bias.** Where HyperDX is ahead, ahead
> is written. Where Parallax's edge is only *planned* or *unproven*, that is
> stated, not hidden.
>
> Research date: **2026-07-17** (**pass 44 pricing RESOLVED** against live
> [hyperdx.io/pricing](https://www.hyperdx.io/pricing) + ClickStack Managed beta).
> Promoted from `watch` to deep-dive pass 25.

## TL;DR verdict (scoped per axis)

- **HyperDX is now strategically elevated:** ClickHouse Inc. bundles the HyperDX
  UI as **"ClickStack"** — ClickHouse's *official* observability stack. That
  makes HyperDX the **"just use ClickHouse directly"** answer to observability,
  directly adjacent to Parallax's GreptimeDB-vs-ClickHouse storage bet.
- **Full-stack breadth (logs/traces/metrics/errors + session replay), OTLP +
  multi-protocol ingest, ClickHouse-proven store, MIT OSS, self-host, and
  ClickHouse-backing: HyperDX wins, plainly** over pre-release Parallax —
  especially **session replay**, which Parallax does not have.
- **Parallax's differentiated edges are all unproven (A1 gate):** production
  error-event derivation + Sentry-envelope ingest, a bounded redacted evidence
  bundle, and a fix-outcome loop. HyperDX is a general full-stack platform, not
  an evidence/agent-context engine.

## HyperDX — what it is (verified 2026-07-17)

Open-source **full-stack observability platform** (`hyperdxio/hyperdx`)
unifying **session replays, logs, metrics, traces, and errors** — powered by
**ClickHouse** (storage/search) and **OpenTelemetry** (collection). Positioned
as "Datadog without the price tag." Now also distributed by ClickHouse Inc. as
**ClickStack** (HyperDX UI + ClickHouse).

| | HyperDX | Source |
|---|---|---|
| **Repo** | `hyperdxio/hyperdx`, **9,680 stars** (GitHub API, 2026-07-17) | [github.com/hyperdxio/hyperdx](https://github.com/hyperdxio/hyperdx) |
| **License** | **MIT** (repo LICENSE — the comparison-set's "Apache-2.0" was imprecise; the core is MIT, *more* permissive) | GitHub API |
| **Signals** | session replay + logs + metrics + traces + errors (full-stack incl. RUM/replay) | repo + [hyperdx.io](https://www.hyperdx.io/) |
| **Storage** | **ClickHouse** (Lucene-style log search); the HyperDX UI is also ClickHouse Inc.'s **ClickStack** | [clickhouse.com/clickstack](https://clickhouse.com/clickstack) |
| **Ingest** | OTLP-native **+ multi-protocol**: OTel, syslog, Elasticsearch, Loki, Datadog formats | ClickStack docs |
| **Latest release** | `@hyperdx/app@2.30.1` (2026-07-13); monorepo active (push 2026-07-17). ClickStack **active in 2026** (Managed ClickStack beta **2026-02-04**) — pass-25 “July 2025 last feature blog” is **stale** | GitHub + [Managed ClickStack](https://clickhouse.com/blog/introducing-managed-clickstack-beta) |
| **Self-host** | ✅ Docker (MIT); **or** HyperDX Cloud; **or** Managed ClickStack on ClickHouse Cloud | repo + hyperdx.io + clickhouse.com |
| **Correlation** | auto-correlates logs ↔ traces ↔ metrics ↔ infra in one view | product |
| **Pricing** | **RESOLVED pass 44** — Free 3GB / Starter **$20 + $0.40/GB** / Enterprise custom | [hyperdx.io/pricing](https://www.hyperdx.io/pricing) |

## Pricing & economics — RESOLVED pass 44

### HyperDX Cloud (live [hyperdx.io/pricing](https://www.hyperdx.io/pricing), 2026-07-17)

| Plan | Price | Included | Overage | Retention / seats |
| --- | --- | --- | --- | --- |
| **Free** | **$0/mo** | **3 GB/mo** | n/a | 3-day; 1 user |
| **Starter** | **$20/mo** | **50 GB/mo** | **$0.40 / GB** logs+traces; **$0.40 per 100 metrics (1 DPM)** | 30-day; **unlimited users**, $0/seat, $0/host |
| **Enterprise** | custom | custom | volume discounts | custom retention; SAML SSO |

- GB = **uncompressed** data sent (spans, sessions, logs).
- Metric unit = **DPM**; higher resolution multiplies bill.
- Vendor **~10× vs Datadog** claim on example 10 TB / 30-day workload — marketing, not independently measured.

### Managed ClickStack (ClickHouse Cloud)

[Managed ClickStack beta (2026-02-04)](https://clickhouse.com/blog/introducing-managed-clickstack-beta): priced on **infrastructure (compute+storage), not events**. Vendor claims full-fidelity OTel retention economics **< ~$0.03/GB/mo**. **No single public rate card** independent of ClickHouse Cloud SKUs.

**Parallax pricing:** **no public number** (pre-release). Self-host = no per-GB tax by design.

**Honest cost read:** HyperDX Cloud is **public, simple, cheap** ($0.40/GB, $20 entry) — strong cost transparency. Head-to-head vs Parallax self-host **unmeasured**. Managed ClickStack is the strongest commercial “just use ClickHouse” counter to GreptimeDB.

## Axis-by-axis comparison

### Signal coverage

| Signal | HyperDX (shipped) | Parallax (pre-release; ✅🧪=code-shipped) | Who |
|---|---|---|---|
| Logs | ✅ Lucene search | ✅🧪 OTLP logs (shipped, pre-release) | **HyperDX** (search maturity) |
| Traces | ✅ OTLP | ✅🧪 OTLP traces (shipped, pre-release) | **HyperDX** (maturity) |
| Metrics | ✅ | ✅🧪 OTLP metrics (shipped, pre-release) | **HyperDX** |
| Errors / exceptions | ✅ (errors tab) | ✅🧪 derived `error_event` + fingerprint (shipped, pre-release) | **HyperDX** (maturity) |
| **Session replay / RUM** | ✅ **core** | ❌ | **HyperDX** (Parallax has none) |
| Profiling | ❌ | ❌ | tie (neither) |
| LLM / agent spans | ❌ | 🟡 planned | **Parallax** (planned) |
| Multi-protocol ingest | ✅ (OTLP/syslog/ES/Loki/DD) | ✅ OTLP + shipped Sentry-envelope | **HyperDX** (breadth); Parallax (Sentry lane) |

**Verdict:** HyperDX is a **broad full-stack platform incl. session replay**,
all shipped. On breadth + replay, **HyperDX wins decisively** (Parallax has no
replay/RUM). Parallax's signal *model* differs (derived errors, Sentry) but is
narrower/unshipped-at-parity.

### Ingestion & transport

- **HyperDX: OTLP-native + multi-protocol** (OTel, syslog, Elasticsearch, Loki,
  Datadog) — broad ingest compatibility. **Parallax: OTLP-native + shipped
  Sentry-envelope ingest** (`sentry_envelope.rs`).
> HyperDX wins on ingest *breadth* (5 protocols); Parallax's distinctive ingest
> edge is the **Sentry-envelope** lane (HyperDX reads the Datadog *format*, not
> Sentry envelopes).

### Storage architecture — the crux (adjacent to Parallax's bet)

- **HyperDX = ClickHouse** (the proven, battle-tested telemetry store; Lucene
  full-text). Now **ClickHouse Inc.'s official observability UI (ClickStack)** —
  so choosing HyperDX = choosing ClickHouse end-to-end.
- **Parallax = GreptimeDB** (native OTLP tables) — the deliberate
  ClickHouse-alternative bet.

> **This is the storage bet made concrete.** HyperDX/ClickStack = "ClickHouse is
> enough, run it directly." Parallax = "GreptimeDB's native-OTLP path is better
> for OTLP-native + local-first." ClickHouse is **far more proven**; GreptimeDB
> is **younger**. GreptimeDB-vs-ClickHouse cost/perf is **benchmark-dependent,
> unmeasured** (ties to the in-repo study). On proven scale, **ClickHouse
> (HyperDX) wins;** on the OTLP-native/local-first engine fit, Parallax's bet is
> unmeasured.

### Query & correlation

HyperDX: unified single-view auto-correlation of logs↔traces↔metrics↔infra +
Lucene log search. Mature cross-signal exploration. Parallax: evidence-graph +
bounded bundle (**unproven**, A1). **HyperDX wins** on shipped cross-signal UX.

### Error tracking & workflow

HyperDX: errors tab (queryable, grouping-ish) — a real but not Sentry-grade issue
lifecycle. Parallax: derived `error_event` + fingerprint + (planned) outcome
loop. Roughly comparable on error-as-data; Parallax's outcome loop is the
unproven differentiator.

### Dashboards & visualization

HyperDX: full-stack UI (replay viewer, log search, trace/metric views). Parallax:
minimal V1. **HyperDX wins** within its full-stack domain.

### AI-native / agent-context story (Parallax's wedge — be most honest)

- **HyperDX:** a **human full-stack monitoring platform** (logs/traces/replay for
  engineers). No bounded/redacted agent-context projection, no fix-outcome loop,
  no AI autofix→PR surfaced.
- **Parallax's claim:** bounded, redacted, agent-safe evidence bundle for coding
  agents.

> **Honest verdict:** HyperDX is **not** an agent-context engine. The two barely
> overlap on the AI axis. HyperDX does, however, occupy the **full-stack OTLP +
  ClickHouse + Apache-class OSS + self-host** ground adjacent to Parallax's
> telemetry layer — and it does so with **ClickHouse Inc. backing (ClickStack)**,
> which is the strongest "why not just ClickHouse?" counter to Parallax's
> GreptimeDB choice.

### Architecture & deployment

HyperDX: MIT self-host (Docker: HyperDX UI + ClickHouse) **or** HyperDX Cloud.
Parallax: single-binary self-host, Apache-2.0 (GreptimeDB + Turso). Both open +
self-hostable. **HyperDX shipped/mature;** Parallax's single-binary local-first
is a (design) simplicity edge.

### Scalability & performance

HyperDX: ClickHouse-backed → proven at large scale. Specific numbers vendor; not
independently measured. Parallax: **benchmark-dependent, unproven.** On proven
scale, **HyperDX (ClickHouse) wins.**

### Security & compliance

HyperDX Cloud: standard SaaS security; self-host = your posture. Verify current
compliance certs (no SOC2/HIPAA confirmed this pass). Parallax: SSO/RBAC/audit
planned; redaction (A6) designed. Roughly even on paper; both immature vs
Datadog/Sentry.

### Openness, licensing & lock-in

- **HyperDX: MIT** (core) — genuinely open, self-hostable, OTLP + multi-protocol
  in (low ingest lock-in). Backend = ClickHouse (open). **Comparable-to-slightly-
  more-open than Parallax's Apache-2.0** (both permissive; MIT lacks the patent
  grant).
- **Parallax: Apache-2.0**, OTLP-native.

> **Verdict:** on openness, **roughly tied** (both permissive OSS, OTLP-native,
> self-hostable). No lock-in advantage either way.

### Pricing & economics

See **Pricing & economics — RESOLVED pass 44** above. HyperDX Cloud is public
($0.40/GB); MIT self-host free. Parallax: **no public number.** Direct TCO
**benchmark-dependent, unmeasured.**

## Where HyperDX plainly wins (no bias)

1. **Full-stack breadth incl. session replay/RUM** (Parallax has none).
2. **ClickHouse-proven store** + **ClickHouse Inc. backing (ClickStack)** — the
   strongest "just use ClickHouse" counter to Parallax's GreptimeDB bet.
3. **Multi-protocol ingest** (OTLP/syslog/ES/Loki/Datadog).
4. **Lucene log search + unified cross-signal correlation** (shipped).
5. **MIT OSS** self-host + Cloud; 9.7k★ community.
6. **Shipped/mature today;** Parallax pre-release.

## Where Parallax honestly edges HyperDX

1. **Production error events + fix-outcome loop** — HyperDX has neither as a
   managed artifact. *(Thesis, **unproven** — A1 gate.)*
2. **Bounded, redacted, agent-safe evidence bundle** — HyperDX is a human
   dashboard, not an agent-context projection. *(Thesis, **unproven** — A1 gate.)*
3. **Sentry-envelope ingest lane** — HyperDX reads the Datadog format, not Sentry
   envelopes (Parallax ships `sentry_envelope.rs`).
4. **Single-binary local-first** — HyperDX self-host is a UI+ClickHouse stack.
   *(Design edge.)*
5. **GreptimeDB native-OTLP-table engine choice** — a deliberate bet vs
   ClickHouse; **unproven** head-to-head (HyperDX/ClickHouse is more proven).

## Watch triggers — re-evaluate HyperDX if it:

- Adds **AI autofix→PR** or a **bounded agent-context artifact**.
- Adds **LLM/agent observability** (none today).
- Adds a **fix-outcome loop** or **error-issue lifecycle** (Sentry-grade).
- **ClickStack cadence stalls** (last feature blog July 2025 — confirm alive).

## Sources (checked 2026-07-17)

- [github.com/hyperdxio/hyperdx](https://github.com/hyperdxio/hyperdx) — **9,680★**, **MIT**, full-stack + replay + ClickHouse + OTel.
- [hyperdx.io](https://www.hyperdx.io/) — product + pricing.
- [clickhouse.com/clickstack](https://clickhouse.com/clickstack) — HyperDX UI = ClickHouse's official observability stack; [What's new July 2025](https://clickhouse.com/blog/whats-new-clickstack-july-2025).
- [Horovits — ClickStack analysis](https://horovits.medium.com/clickstack-clickhouses-new-observability-stack-unveiled-73f129a179a3).
- Parallax side: [decisions/storage-engine.md](../../decisions/storage-engine.md), [storage/greptimedb-vs-clickhouse/](../../storage/greptimedb-vs-clickhouse/), [validation/a1-bundle-value/](../../validation/a1-bundle-value/).
