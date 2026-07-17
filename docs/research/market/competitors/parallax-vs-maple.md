# Parallax vs Maple

> An unbiased, one-to-one comparison. Research date: **2026-07-17** (pass 59 +
> pass 112; **pass 139** re-pin). Tinybird-decoupling watch **still UNFIRED** —
> latest tag still **v0.0.12** (2026-06-18) / **1,532★**; push **2026-07-17**
> (UI/traces perf commits only). README still documents **Tinybird** cloud path
> (`TINYBIRD_*` env) + **embedded ClickHouse** local — not a GreptimeDB/self-owned
> store decoupling. Sources: [maple.dev](https://maple.dev/),
> [github.com/MapleTechLabs/maple](https://github.com/MapleTechLabs/maple), legacy
> [maple-deep-research.md](../maple-deep-research.md).
>
> **Bottom line up front:** Maple is the **local-experience benchmark** Parallax
> wants to match — a single Bun binary with embedded chDB (ClickHouse) local mode,
> OTLP-native, **and the same Turso/libSQL metadata choice Parallax made.** On
> **shipped local-mode polish, full-stack OTLP platform, dashboards, and
> error/session-replay coverage, Maple is ahead of pre-release Parallax.** Parallax's
> honest edges are Rust vs TS/Bun, **self-hosted GreptimeDB vs Maple's Tinybird-
> vendor-coupled ClickHouse** (Maple's hosted fast-path depends on a vendor),
> **Apache-2.0 vs FSL-1.1**, and the *code-shipped bundle (A1 value unproven)* + *offline fix-outcome residual (plan 123 DONE)* thesis.

## What each product is

- **Maple** (`MapleTechLabs/maple`, renamed from `Makisuo/maple`) — open-source, **OTLP-native full-stack observability platform** (distributed tracing, logs, metrics/dashboards, error tracking, service catalog + dependency map, K8s monitoring, browser session replay, AI/MCP agent surface). Positioned as a New Relic/Datadog/Dash0 alternative. **FSL-1.1** (source-available, self-hostable, competitive-use restrictions — less open than Apache). **TypeScript ~95%** (Effect framework) on **Bun**; Rust only for the local-mode chDB bridge. Storage: **ClickHouse** — **Tinybird hosted** (cloud) / **chDB embedded** (local). **Metadata: SQLite via libSQL (Turso)** — *the same metadata choice as Parallax*. **1,532 stars, v0.0.12** (2026-06-18; GitHub API 2026-07-17 — up from ~0.4k in the June legacy note; pre-1.0, fast-moving).
- **Parallax** — open-source (Apache-2.0), Rust-first, self-hostable **execution-context engine**: OTLP-native ingest of traces/logs/metrics + CLI/agent traces, derives owned `error_event`s, fingerprints, correlates into a typed evidence graph, serves bounded/redacted evidence bundles to humans and coding agents. GreptimeDB + Turso. **Pre-release.**

Both single-binary-local-first, OTLP-native, Turso-metadata. The closest pair on *local-UX intent* and the *metadata choice*. Differ on language (TS/Bun vs Rust), storage (Tinybird/chDB ClickHouse vs GreptimeDB), license (FSL vs Apache), and product intent (full platform vs evidence engine).

## Signal coverage

| Signal | Maple (shipped) | Parallax (pre-release; ✅🧪=code-shipped) |
| --- | --- | --- |
| Traces | ✅ (waterfall/flamegraph/flow, trace-log correlation) | ✅🧪 OTLP traces (shipped, pre-release) |
| Logs | ✅ (full-text, severity, streaming) | ✅🧪 OTLP logs (shipped, pre-release) |
| Metrics / dashboards | ✅ (20+ chart types, drag-drop builder) | 🟡 minimal (🏗) |
| Errors / exceptions | ✅ (smart grouping by type/message, trend, spam filtering) | ✅🧪 derived `error_event` + fingerprint (shipped, pre-release) |
| Session replay (browser) | ✅ | ❌ |
| Service catalog / dependency map | ✅ (latency, Apdex, dep edges, commit SHA) | ❌ (🏗) |
| K8s monitoring | ✅ (Helm) | ❌ |
| LLM / agent spans | 🟡 (explore) | ✅ (🏗) |
| Sentry envelope / DSN | ❌ | ✅ shipped |

**Verdict:** Maple's coverage is broader and all shipped — it is a full platform. Parallax is narrower (evidence engine). On coverage, **Maple wins decisively.** Maple gap vs Parallax thesis: **no Sentry-envelope path, no fix-outcome loop** (Parallax ships envelope + offline outcome residual plan 123 DONE).

## Ingestion & transport

- **OTLP:** Maple is genuinely **OTLP-native** — standard OTel SDKs emit OTLP traces/logs/metrics to `apps/ingest` → OTel Collector. Local mode: OTLP/HTTP on `:4318`. Same native stance as Parallax.
- **Sentry envelope:** Maple has **none**. Parallax **ships** bounded envelope ingest (plan 118 DONE; multi-SDK ledger unproven).
- **Local mode capture:** single Bun binary + libchdb; standard OTel SDKs target `localhost:4318`.

**Verdict:** on OTLP-native ingest, **tied in design; Maple ships it.** On Sentry-envelope, **Parallax ships bounded envelope ingest** (plan 118 DONE).

## Storage architecture — same metadata, different telemetry engine + a vendor coupling

- **Maple:** **ClickHouse** everywhere — **Tinybird (managed ClickHouse)** for the hosted cloud fast-path, **chDB (embedded ClickHouse)** for local mode (`~/.maple/data`). **Metadata: SQLite via libSQL (Turso)** — same as Parallax. ⚠️ **The hosted fast-path is coupled to Tinybird** (a vendor) — Maple's cloud query engine is a managed service, not self-hosted ClickHouse. **Tinybird-decoupling watch UNFIRED pass 59 (2026-07-17):** recent commits = Clerk/rerender perf (service-detail charts) — **not** storage-vendor decoupling; Tinybird still in codebase; hosted fast-path still Tinybird-coupled.
- **Parallax:** **GreptimeDB** (native OTLP tables) + **Turso** metadata — both self-hosted-native, no vendor coupling for the fast path.

**Verdict:** on the **Turso metadata choice, identical** (mutual validation). On the **telemetry engine**, a real split: Maple = ClickHouse (Tinybird-hosted for cloud / chDB for local); Parallax = GreptimeDB (self-hosted). **Parallax's edge: no vendor coupling on the fast path** (Maple's hosted performance depends on Tinybird). GreptimeDB-vs-ClickHouse cost/perf is benchmark-dependent/unmeasured (ties to the in-repo study). On proven-at-scale, **ClickHouse (Maple) is more battle-tested** than GreptimeDB.

## Query & correlation

- **Maple:** trace-log correlation, dependency-map drill, dashboard-driven exploration, MCP query tools. Mature general-purpose.
- **Parallax:** evidence-graph correlation + bounded bundle for agents.

**Verdict:** on **general cross-signal query/dashboard exploration, Maple wins** (full platform). Parallax's bundle is a different axis (bounded agent context), unproven (A1).

## Error tracking & workflow

- **Maple:** **smart grouping by type/message**, trend detection, trace linking, spam/env filtering — a real error-tracking surface (more than Grafana/Honeycomb/Coroot offer natively), though not Sentry-grade lifecycle.
- **Parallax:** derived `error_event` + deterministic fingerprint (**shipped**) + fix-outcome offline residual (**plan 123 DONE**; live value **unproven**).

**Verdict:** on **shipped error grouping, Maple is ahead of pre-release Parallax.** On the **fix-outcome loop**, Parallax targets an unoccupied cell (offline residual plan **123 DONE**; live value **unproven**). Scoped.

## AI-native / agent-context story

- **Maple's AI/MCP:** MCP server in `apps/api`, **10+ read-oriented tools**; explore-level AI. A read-leaning agent surface, but not a bounded/redacted/versioned bundle.
- **Parallax's claim:** bounded, redacted, agent-safe evidence bundle for coding agents (**code-shipped**, A1 value unproven).

**Verdict:** Maple ships more agent surface today (10+ MCP tools) than Parallax. On **read-oriented** MCP, Maple is closer to Parallax's safety intent than SigNoz/OpenObserve (which are write-capable) — but Maple's is not bounded/redacted/versioned. Parallax's differentiated bundle is **unproven (A1).**

## Architecture & deployment — the local-UX benchmark

- **Maple:** **single Bun binary** for local mode (`maple` + `libchdb`, OTLP :4318, dashboard at `local.maple.dev` or `--offline`). Polished "Operator Terminal" design. Hosted = Cloudflare Workers + D1. K8s Helm. **This local-mode UX is the benchmark Parallax wants to match.**
- **Parallax:** single-binary Rust target, local-first, air-gap-capable, Apache-2.0.

**Verdict:** on **shipped single-binary local-mode polish, Maple wins** (it is the local-experience benchmark). Parallax's single-binary target is parity-by-design, pre-release. On **language/substrate, Parallax (Rust) vs Maple (TS/Bun)** — Parallax's stated preference; not a decisive product difference.

## Operational footprint

- **Maple:** local mode = one binary + chDB (tiny). Hosted = managed. Self-host K8s = Helm chart. Low friction to start locally.
- **Parallax:** single-binary target; GreptimeDB + Turso.

**Verdict:** on **local-mode simplicity, Maple wins** (shipped, polished). Parallax's target is parity, unproven.

## Scalability & performance

- **Maple:** small (~1.5k stars, pre-1.0); ClickHouse (Tinybird) is proven, but Maple's own harness is early. Hosted performance tied to Tinybird.
- **Parallax:** unproven; benchmark-dependent.

**Verdict:** neither is proven at large self-hosted scale (both early). ClickHouse-vs-GreptimeDB is the measurable question (benchmark-dependent). **Roughly tied on maturity (both pre-scale), with Maple shipping more today.**

## Security

- **Maple:** hosted has auth/org/ingest-keys; self-host posture modest (pre-1.0). SSO/RBAC maturity TBD.
- **Parallax:** SSO/RBAC/audit planned; redaction (A6) designed.

**Verdict:** both early on enterprise security. Scoped.

## Openness, licensing & vendor lock-in — a real Parallax edge

- **Maple:** **FSL-1.1** (Functional Source License) — source-available, self-hostable, but with **competitive-use restrictions** and a 2-year convert-to-Apache/MIT clause. **Less permissive than Apache-2.0.** Plus the **Tinybird vendor coupling** on the hosted fast-path (moderate lock-in for cloud).
- **Parallax:** **Apache-2.0**, fully open, OTLP-native, portable bundle, no vendor fast-path coupling.

**Verdict:** on **license permissiveness + no-vendor-coupling, Parallax (Apache-2.0 + self-hosted GreptimeDB) edges Maple (FSL + Tinybird-hosted fast-path).** A real Parallax edge — both the license and the storage-vendor-independence.

## Pricing & economics — real numbers

| Plan | Price | Notes |
| --- | --- | --- |
| **Self-host (FSL)** | $0 | self-host, FSL-1.1 |
| **Startup** | **~$29–$39/mo** | **300 GB total data included**, then **$0.25/GB** (annual vs monthly variance — confirm on [maple.dev/pricing](https://maple.dev/pricing/)) |

Sources: [maple.dev/pricing](https://maple.dev/pricing/). Maple markets large savings vs New Relic (~$351/mo cited).

**Parallax pricing:** none public yet (pre-release).

**Honest cost read:** Maple's $0.25/GB-over-300GB is competitive. Whether Parallax self-host is cheaper is benchmark-dependent/unmeasured. Maple's hosted path adds Tinybird as a cost layer (vendor margin).

## Where Maple plainly wins

- **Shipped single-binary local-mode polish** — the local-experience benchmark (chDB embedded, polished "Operator Terminal").
- Full-stack OTLP platform (traces/logs/metrics/errors/replay/catalog/K8s) — all shipped.
- Error tracking (smart grouping) — more than most platforms.
- Same Turso metadata choice (mutual validation).
- 10+ read-oriented MCP tools.

## Where Parallax honestly edges Maple

- **License permissiveness** — Apache-2.0 vs FSL-1.1 (competitive-use restrictions). *(Real.)*
- **No vendor coupling on the fast path** — self-hosted GreptimeDB vs Maple's Tinybird-hosted ClickHouse. *(Real; Maple's hosted perf depends on a vendor.)*
- **Rust vs TS/Bun** — Parallax's stated substrate. *(Minor.)*
- **Sentry-envelope compatibility** — Maple has none. *(Real; Parallax shipped.)*
- **Fix-outcome loop + bounded/versioned/redacted bundle** — Maple has neither. *(Thesis, unproven, A1.)*

> **Honest summary:** Maple is the local-UX benchmark and ships a polished single-binary-local OTLP platform with the same Turso metadata Parallax chose. Parallax's defensible delta is **Apache-vs-FSL**, **no-Tinybird-vendor-coupling** (self-hosted GreptimeDB), **Sentry-envelope** (shipped), and the **bounded+outcome bundle** — residual unproven (A1). Borrow Maple's local-mode polish; do not assume GreptimeDB beats ClickHouse without measurement.

## Open questions / what measurement would settle

- ~~Maple latest version~~ → **pinned v0.0.12 (2026-06-18), 1,532★** (GitHub API 2026-07-17); repo **renamed `Makisuo/maple` → `MapleTechLabs/maple`**.
- **A1 gate vs Maple:** if a team has Maple for local-mode OTLP obs, does Parallax's bounded bundle add measurable value for production-incident agent fixes? Unproven.
- **GreptimeDB-vs-ClickHouse/chDB** — measured cost/perf at parity (local + hosted). Benchmark-dependent, unmeasured.

## Sources (accessed 2026-07-17)

- [maple.dev](https://maple.dev/); [pricing](https://maple.dev/pricing/); [docs](https://maple.dev/docs).
- [github.com/MapleTechLabs/maple](https://github.com/MapleTechLabs/maple) — **v0.0.12 (2026-06-18), 1,532★** (GitHub API 2026-07-17; renamed from `Makisuo/maple`).
- Legacy internal: [maple-deep-research.md](../maple-deep-research.md) (2026-05-31 — data-flow, tech choices, feature inventory).
- Parallax side: [decisions/metadata-store.md](../../decisions/metadata-store.md) (the shared Turso choice), [decisions/storage-engine.md](../../decisions/storage-engine.md), [validation/a1-bundle-value/](../../validation/a1-bundle-value/).
