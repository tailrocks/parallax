# Storage Engine — GreptimeDB vs ClickHouse

<!-- markdownlint-disable MD013 -->

Decision date baseline: 2026-05-29. Operator confirmations followed on
2026-06-03, 2026-06-11, and 2026-06-18.

> **Current authority (operator, 2026-06-12; native-table refinement,
> 2026-06-18): GreptimeDB + Turso are mandatory in every product profile.** Raw
> observability signals use GreptimeDB native tables. ClickHouse and Postgres are
> research comparators only, never fallback engines or implementation targets.
> Storage and metadata traits are capability, ownership, and test boundaries;
> they do not promise engine substitution. The in-memory adapter is test/dev
> support only. Contract cleanup is owned by
> completed [Plan 093 validation](../validation/2026-07-12-plan-093-baseline/README.md),
> and any supported server profile is owned by
> [`docs/research/validation/2026-07-plan-115-v2-server-profile/`](../validation/2026-07-plan-115-v2-server-profile/).
>
> **Implementation status (2026-07-17): shipped.** Product telemetry uses
> GreptimeDB native OTLP tables and product metadata uses Turso. No product
> fallback exists; the in-memory adapter is test-only.
>
> The dated selection, fallback, and flip analysis below is preserved as
> benchmark history. It cannot authorize a product backend change.

This is the condensed historical engine verdict. The **full record** — ~170 benchmark runs, a source-level
teardown of both engines, the four-build version matrix, and the per-pass history — lives in
[../storage/greptimedb-vs-clickhouse/](../storage/greptimedb-vs-clickhouse/) (start at
[verdict-which-to-choose.md](../storage/greptimedb-vs-clickhouse/verdict-which-to-choose.md);
history in [run-log.md](../storage/greptimedb-vs-clickhouse/run-log.md); cross-build matrix in
[four-way-version-comparison.md](../storage/greptimedb-vs-clickhouse/four-way-version-comparison.md)).

## Decision questions (DQ1–DQ6), in one table

| # | Question | Answer (mechanism-grounded) |
| --- | --- | --- |
| DQ1 | Where is **GreptimeDB** genuinely better? | Metrics/PromQL-native (GA + default-on); small-write/upsert ingest ergonomics (LSM, no "too many parts"); horizontal scale-out by design (region auto-rebalance, compute/storage separation, no bulk-copy migration); read-time dedup → correct latest-state on a plain query; OTLP schema-drift auto-adds typed columns; retention = whole-SST drop (cheap by default); object-storage-native (fewer objects → wins cold *full* scans); replayable WAL; cardinality-insensitive metric *ingest* (~flat 1k→1M series). |
| DQ2 | Where is **ClickHouse** genuinely better? | Selective log/trace scan + full-text; time-DESC log-tail locality; generic wide-scan/aggregate throughput (decade-tuned C++ vectorized engine, ~2–3× warm metric-agg); per-column codecs; dynamic-attribute JSON path queries (~8× with the required `.:Type` cast); projections (a 2nd physical order); in-DB anchored cross-tier joins; cold *selective* object-store reads (sparse-granule egress); schema-mistake tolerance. The gap **widens with scale** (5M+). |
| DQ3 | Did ClickHouse prove technical workload comparability? | **Yes** — the historical study stored every signal and produced identical bundles, while requiring a PromQL+OTLP compatibility layer, manual sharding (OSS `SharedMergeTree` is Cloud-only), and ingest batching. This is comparator evidence, not product portability authority. |
| DQ4 | Did GreptimeDB cover the same comparison workload? | **Yes** — Q1–Q6 produced identical results while heavy ad-hoc log/trace scans were slower. Parallax's anchored hot path was **not latency-bound** (Q6 composite ≪300 ms on both). |
| DQ5 | Which to choose for Parallax today? | **GreptimeDB** on workload fit (metrics-native, ingest/upsert ergonomics, retention cost, scale-out) + the Rust tiebreak; ClickHouse's wins are real but less central to anchored retrieval. |
| DQ6 | Better long-term *investment*? | **GreptimeDB** — the speed gap is **closable engineering, not a physics wall** (seven of eight advantages are pure engineering; the two heaviest ride the shared **DataFusion** scan and **Parquet-Variant** JSON roadmaps), and it is the **Rust, open-source substrate the operator can contribute to** rather than wait on (C++). |

## Why the first production focus is GreptimeDB

Parallax is not choosing the fastest analytical database in the abstract. It is choosing the first
production/server storage substrate for an execution-context product whose critical user action is:

```text
issue / event / trace / fingerprint
  -> fetch related errors, spans, logs, metric windows, deploys, CLI runs, agent actions
  -> assemble one bounded evidence bundle
```

That workload makes GreptimeDB the better first focus for five reasons:

1. **The hot path is anchored, not broad.** ClickHouse's strongest advantage is broad analytical scan,
   log search, dynamic-attribute filtering, and mature SQL throughput. Parallax's first hot path is
   anchored bundle assembly by `trace_id`, `fingerprint`, issue, or narrow time window. Existing local
   benchmark runs show both engines interactive on that path, so ClickHouse's speed lead does not
   decide the server profile.
2. **GreptimeDB matches the observability shape.** Current GreptimeDB docs position it as a unified
   observability database for metrics, logs, and traces, with SQL and PromQL support. That is closer
   to Parallax's retained evidence model than a general analytical warehouse.
3. **Metrics are evidence, not a side quest.** Parallax bundles need metric windows and anomalies beside
   traces/logs/errors. GreptimeDB's PromQL-compatible path makes Prometheus-style metric evidence
   easier to preserve without a separate query layer.
4. **Retention economics matter more than peak scan speed.** Parallax's self-hosted promise depends on
   keeping enough history that bundles remain useful without turning diagnostic data into a cost spike.
   GreptimeDB's cloud-native, disaggregated compute/storage and object-storage-oriented design are the
   reason it gets first focus. This is still a measured claim, not a settled fact, until the sized
   $/GB and cold-read gates close.
5. **Rust is a strategic tie-breaker.** GreptimeDB is the Rust engine the operator can inspect and
   contribute to. ClickHouse is stronger and more mature in many analytical paths, but it is a C++
   substrate. When the hot path is fast enough on both, operator-contributable Rust matters.

This focus should not be misread as "ClickHouse is worse." ClickHouse remains a
useful analytical comparator for heavy ad-hoc analytics and broad log search;
contrary measurements expose GreptimeDB risks or upstream work, not a product
engine switch.

## Why the lean is GreptimeDB even though ClickHouse is faster

Two lenses once reached opposite defaults; the resolved query mix breaks the tie toward GreptimeDB:

- **Fit + long-term-investment lens → GreptimeDB.** Rust (operator-contributable), object-store-native
  cost, metrics/PromQL-native, scale-out by design; its speed deficits are closable on shared roadmaps.
- **Parallax-as-proxy lens → once leaned ClickHouse.** Because Parallax itself owns OTLP
  ingest/routing/conversion (operator architecture decision, 2026-05-25), GreptimeDB's native-ingest
  edge is neutralized, leaving retrieval speed + build-on-top ecosystem (SigNoz/Uptrace/HyperDX/
  ClickStack) — both ClickHouse wins.
- **The resolver — query mix is RESOLVED (operator 2026-05-29): anchored-bundle-retrieval-dominant.**
  The hot path fetches all signals for one `trace_id`/`fingerprint`/issue to assemble a bundle, not
  broad ad-hoc analytics. On that path **both engines are interactive at every tested scale**, so
  ClickHouse's raw-speed lead is **not decisive for Parallax**. The decision therefore turns on
  **cost + Rust**, where GreptimeDB leads — not on analytical-scan speed, where ClickHouse leads.

## Operator re-affirmation 2026-06-11 (with reality checks)

The operator restated the lean with more conviction: GreptimeDB as the engine to invest in because
it is Rust, already built around the columnar/object-storage concepts ClickHouse proves, and —
since AI agents contribute best to Rust codebases — whatever it still misses versus ClickHouse can
be added over time ("absorb the gap upstream") until one engine serves everything in one place.
This strengthens DQ6 (long-term investment) and is consistent with the
[parity roadmap](../storage/greptimedb-vs-clickhouse/greptimedb-parity-roadmap.md). It does **not**
close the cost/cold-read finalizer gates below, and three reality checks bound the
"AI-extends-it-upstream" strategy (checked 2026-06-11):

1. **Upstream is gated.** GreptimeDB requires a CLA and has an explicit AI-assisted-PR policy:
   authors must understand changes end-to-end, "AI dump" PRs may be closed unreviewed, and review
   capacity is stated as "very limited" ([CONTRIBUTING.md](https://github.com/GreptimeTeam/greptimedb/blob/main/CONTRIBUTING.md)).
   Development is overwhelmingly core-team (~136 contributors, external PRs a small share). The
   strategy is therefore *high-quality AI-assisted contributions with human ownership* — with a
   fork as the hedge, which Apache-2.0 permits.
2. **OSS/Enterprise split matters for the loop.** Triggers/alerting, RBAC, audit logging, and
   read replicas are Enterprise-only ([enterprise docs](https://docs.greptime.com/enterprise/overview/)).
   Parallax must own detection/dispatch in its own workers regardless of engine — which the
   adapter boundary already requires.
3. **Release cadence (historical June claim, superseded).** The 2026-06-11 note said v1.1 had not
   GA'd and nightlies stalled at `v1.1.0-nightly-20260525`. **That pin is obsolete as of
   2026-07-17 (pass 52 / pass 48 re-pin):** stable is **`v1.1.3`** (2026-07-17), with prior
   `v1.1.0` / `v1.1.1` / `v1.1.2` on the same line; nightly is **`v1.2.0-nightly-20260706`**.
   The **v1.1-GA retest trigger has fired for version currency**; **sized cost / cold-read /
   Parallax-shaped benchmark re-runs on v1.1.3 remain unmeasured** (benchmark agent owns them).

## Historical finalizer questions

1. **Sized cost numbers on a real server tier** — $/GB retained, per-signal compression, and
   **multi-replica object-storage cost** (GreptimeDB 1× shared S3 vs OSS ClickHouse N× replica
   copies). The operator's #1 priority and the least-measured axis. Evidence:
   [../storage/size-and-object-cost.md](../storage/size-and-object-cost.md).
2. **Cold-read latency at GB–TB from object storage** — the one regime that could still surprise an
   anchored workload. Evidence: [../storage/freshness-and-latency.md](../storage/freshness-and-latency.md).
3. **Self-hosted vs managed cloud** — strictly self-hosted at scale favors GreptimeDB's 1× object copy
   + compute/storage separation; if ClickHouse Cloud (`SharedMergeTree`) is acceptable, that erases
   GreptimeDB's cost-economics edge.
4. **Re-test on GreptimeDB v1.1.x stable (now shipping)** — version pin is **`v1.1.3`** (2026-07-17).
   Re-run load-bearing speed/cost benchmarks against this line (and current ClickHouse feature
   stable); prior matrices pinned to `v1.0.2` / early nightlies are **not** current evidence.

## Historical flip analysis (superseded)

The 2026-05 selection study would have flipped toward ClickHouse if sized costs
were equal, managed service use was acceptable, and the workload became
analytics-dominated. That counterfactual remains useful for interpreting
benchmark risk. Current policy does not permit it to change the product stack;
such evidence instead narrows claims or creates GreptimeDB/Parallax fix-forward
work in `plans/`.

## Standing maintenance

- Keep capability-specific storage boundaries for ownership and testability;
  they do not imply a ClickHouse implementation.
- Query mix is **resolved** (anchored-retrieval-dominant); the remaining finalizers are the sized cost
  numbers and the self-host-vs-managed-cloud call, not another query-shape model.
- Re-pin versions and re-verify load-bearing claims on each new stable release.
- **Version re-pin 2026-07-17 (pass 52; GitHub releases API — no performance claims):**
  - GreptimeDB latest stable: **[`v1.1.3`](https://github.com/GreptimeTeam/greptimedb/releases/tag/v1.1.3)**
    (2026-07-17). Nightly: **`v1.2.0-nightly-20260706`**. Supersedes 2026-06-03/06-11 pins of
    `v1.0.2` / `v1.1.0-nightly-20260525`.
  - ClickHouse feature-line re-pin **2026-07-17 pass 60:** latest non-LTS feature tag observed
    **`v26.6.1.1193-stable`** (2026-06-25); newest non-LTS patch date also shows
    `v26.5.5.8-stable` (2026-07-01). Four-way rule = **feature line not LTS** → prefer **26.6.x**.
    No performance claim.
  - Product stance unchanged: GreptimeDB + Turso mandatory; ClickHouse comparator only.
- **Historical re-verify 2026-06-03 (official docs + GitHub releases — version pins superseded):**
  - GreptimeDB docs still describe a unified observability database for metrics, logs, and traces with
    SQL and PromQL support, OpenTelemetry ingestion paths for metrics/logs/traces, and a distributed
    architecture with region-based sharding and disaggregated compute/storage.
  - ClickHouse docs still describe ClickHouse as a highly efficient observability storage engine with
    strong compression and fast query response, but also state that using it as observability storage
    requires a UI and collection framework; current OTLP usage flows through an OpenTelemetry Collector
    exporter into ClickHouse tables.
  - ~~GitHub releases GA `v1.0.2`~~ → see 2026-07-17 pin above.

## Source anchors checked on 2026-06-03

- [GreptimeDB introduction](https://docs.greptime.com/) — unified observability database for metrics,
  logs, and traces; SQL/PromQL positioning.
- [GreptimeDB observability ingest overview](https://docs.greptime.com/user-guide/overview) —
  observability scenario support for metrics, logs, and traces via OpenTelemetry-related tooling.
- [GreptimeDB HTTP / PromQL protocol](https://docs.greptime.com/user-guide/protocols/http) —
  PromQL-compatible query surface.
- [GreptimeDB FAQ](https://docs.greptime.com/faq-and-others/faq) — distributed system, region-based
  sharding, unified metrics/logs/traces model, SQL + PromQL, cloud-native architecture with
  disaggregated compute and storage.
- [GreptimeDB releases](https://github.com/GreptimeTeam/greptimedb/releases) — current GA/pre-release
  pin.
- [ClickHouse observability introduction](https://clickhouse.com/docs/use-cases/observability/build-your-own/introduction) —
  efficient observability storage, fast query response, compression, and need for UI/collection
  framework.
- [ClickHouse OpenTelemetry integration](https://clickhouse.com/docs/use-cases/observability/build-your-own/integrating-opentelemetry) —
  OTLP receiver/exporter path through OpenTelemetry Collector into ClickHouse.
- [ClickHouse observability schema design](https://clickhouse.com/docs/use-cases/observability/build-your-own/schema-design) —
  materialized columns and table design for logs.
- [ClickHouse releases](https://github.com/ClickHouse/ClickHouse/releases) — current stable feature/LTS
  release pins.

## Related records

- V1 implementation stance: [v1-storage-adapter-vision.md](v1-storage-adapter-vision.md).
- Stack roll-up that gates this becoming a stack default: [stack-decision.md](stack-decision.md) (A5).
- Relational metadata store (separate from the columnar engine): [metadata-store.md](metadata-store.md).
- Parity/closability analysis and alternatives survey:
  [../storage/greptimedb-vs-clickhouse/greptimedb-parity-roadmap.md](../storage/greptimedb-vs-clickhouse/greptimedb-parity-roadmap.md),
  [../storage/greptimedb-vs-clickhouse/platform-fit-and-alternatives.md](../storage/greptimedb-vs-clickhouse/platform-fit-and-alternatives.md).
