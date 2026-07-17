# Platform Fit — Re-deciding the Store Through the Parallax-Proxy Lens (+ alternatives, + the metadata/error-grouping question)

<!-- markdownlint-disable MD013 -->

Status: pass (Run 153) — strategic re-examination requested by the operator. Question, in the
operator's frame: *given Parallax is the first layer (a proxy that owns OTLP, routing, and conversion),
is ClickHouse actually the better choice than GreptimeDB? Is there a third system that beats both? And
where do Sentry-style grouped errors and other metadata live — do we need Postgres?* Be skeptical;
compare on **practical fit for the vision**, not raw speed.

> **Current authority (operator, 2026-06-12): GreptimeDB + Turso are mandatory.**
> ClickHouse and Postgres are research comparators only. The default/fallback/flip
> conclusions in this dated Run 153 analysis are superseded and retained solely
> to explain how the benchmark was interpreted. Capability boundaries do not
> promise engine substitution. Plan 093 is closed; plan 115 owns
> supported server work.

This note reframes the whole comparison. It does not replace the mechanism notes; it changes the
**weighting** of what they found.

## The linchpin: Parallax-as-proxy neutralizes GreptimeDB's headline advantage

The operator's architecture decision: **Parallax is always the first layer.** Customers send telemetry
to Parallax; Parallax implements OTLP itself, decides where/how to route, converts formats, and only
then writes to a backend. "If there is no API in ClickHouse that supports that format out of the box,
it's OK — Parallax designs that API."

That single decision **invalidates GreptimeDB's marquee selling point.** GreptimeDB's own 2026 pitch
(and every vendor comparison) leads with: *native OTLP receiver, native PromQL, native Jaeger Query
API, schema-on-write — so you don't need "buffering middleware, ingestion workers, transformation
pipelines" in front of the database.* That is the entire ingest-ergonomics story we spent Runs
150–152 grounding (metric-engine multiplexer, log pipeline, OTLP→span table + Jaeger read).

**But Parallax *is* that ingestion tier, by design and on purpose** — the operator explicitly wants
Parallax to own buffering/routing/conversion. So "the DB speaks OTLP/Jaeger/Prom natively" stops being
a differentiator: Parallax speaks those protocols and translates to whatever the backend wants. The
proof is the market itself — **SigNoz, Uptrace, OpenObserve, and ClickHouse's own ClickStack/HyperDX
all put a platform layer (collector → buffer → transform → UI) in front of ClickHouse** and treat it
as a dumb-but-fast store. Parallax-on-ClickHouse would be doing exactly what those proven products do.

→ **Verdict reversal pressure:** a large share of the GreptimeDB column in our matrices is
ingest-ergonomics (native protocols, schema-on-write, Jaeger-out, pipeline ETL). Behind the proxy,
that share is **worth ~nothing to Parallax.** The honest re-scoring must set it aside and judge on what
remains.

## What survives the reframe (the axes that still matter)

Strip ingest ergonomics. What Parallax cannot paper over in its proxy layer:

| Axis | Still matters? | Who wins (measured/grounded) |
| --- | --- | --- |
| **Retrieval speed** (the AI-debugging queries) | **Yes — central** | **ClickHouse.** Faster on scans, aggregation, log search, joins (our Runs across the record; ~2× warm agg, ~7–14× unindexed scan, ~13× in-DB join). Anchored `trace_id` lookup is a tie-ish (both ≪ 300 ms). |
| **Storage cost / compression** | Yes | **~tie** at matched effort (our compression-and-cost runs); GreptimeDB's "50% of ClickHouse / 50× cheaper" are **vendor claims vs Loki/ES, not vs tuned ClickHouse** — our own data shows ~parity. Both can sit on object storage. |
| **Object-store economics at scale** | Yes | **GreptimeDB** (1× shared S3 per region; OSS ClickHouse `ReplicatedMergeTree` = N× copies, `SharedMergeTree` is Cloud-only). Real edge *if* self-hosted HA at scale. |
| **High-cardinality metrics** | Partly (Parallax can pre-model/pre-agg) | **GreptimeDB** out-of-the-box (metric-engine dict, no `LowCardinality` 8192 cliff). But Parallax controls schema, so it can model low-card keys + pre-aggregate on ClickHouse. |
| **Horizontal scale / topology change** | Yes (startups→big trajectory) | **GreptimeDB** (region auto-rebalance, repartition) vs ClickHouse OSS manual resharding. But ClickHouse is *proven* at massive obs scale (SigNoz/ClickStack fleets); Cloud `SharedMergeTree` closes it commercially. |
| **Ecosystem / "what we can build on top"** | **Yes — central** | **ClickHouse, decisively.** De-facto obs backend; richest SQL, integrations, MVs, JIT; battle-tested by SigNoz/Uptrace/HyperDX/ClickStack. The largest surface to "develop on top of." |
| **Strict-durability ingest cost** | Minor (Parallax batches) | GreptimeDB (WAL append-fsync ≪ CH whole-part fsync), but Parallax's proxy can batch/buffer, softening it. |

The two **central** axes for "a store Parallax builds a debugging product on" are **retrieval speed**
and **build-on-top ecosystem** — and ClickHouse wins both. GreptimeDB's wins (object-store economics,
cardinality, auto-rebalance) are real but **niche or proxy-softenable**.

## Why GreptimeDB is slower, and why it's cheaper — two SEPARATE things (not one trade-off)

A common mental model is: *"GreptimeDB is slower because it stores files on remote object storage,
and that's the price of the cheaper storage."* The cost intuition is right; the **causation is wrong**,
and getting it straight matters for the decision.

1. **Why GreptimeDB is slower on heavy queries: the execution engine, NOT where the bytes live.**
   ClickHouse is a decade-tuned C++ vectorized engine (65k-row blocks, SIMD, LLVM-JIT, adaptive hash
   aggregation). GreptimeDB runs DataFusion over Arrow — competitive but younger codegen. We measured
   the gap **on local disk, warm cache** (the bench is not even reading from S3): ClickHouse still wins
   aggregation ~2× and unindexed scan ~7–14×. So the slowness is the **query engine**, not the storage
   location. It is also **closable** — DataFusion improves release over release (`greptimedb-parity-roadmap.md`).
   Cold reads from S3 *can* add latency, but that hits **both** engines (ClickHouse on S3 too) and is
   mitigated by local caching; the persistent warm gap is engine-driven.

2. **Why GreptimeDB can be cheaper: object-store-native architecture — independent of the speed gap.**
   - **Compute/storage separation:** scale cheap S3 storage independently of expensive compute; you're
     not forced into big, always-on, local-disk-attached instances.
   - **1× vs N× copies:** GreptimeDB keeps **one shared S3 copy** per region (HA via metadata/leadership);
     OSS ClickHouse `ReplicatedMergeTree` keeps a **full copy per replica** (N× S3) unless you use Cloud
     `SharedMergeTree` (proprietary). At HA scale this is a real storage-bill difference.
   - **Elastic, near-stateless compute:** datanodes hold little durable local state (WAL+SSTs are in
     Kafka/S3), so compute can scale down when idle and up under load — **fewer always-on servers
     (EC2)**.

3. **Both can use object storage.** It is *not* "object storage = GreptimeDB, local = ClickHouse."
   ClickHouse has an S3 disk tier and Cloud `SharedMergeTree`. The difference is GreptimeDB does it
   **natively and at 1× cost in OSS**, while ClickHouse OSS bolts S3 on (N× copies) and the elastic,
   separated version is Cloud-only.

4. **They are separate levers — you do not "pay for speed with cost."** GreptimeDB happens to be both
   (a) a bit slower on heavy analytics (engine, closable) and (b) cheaper to operate at scale
   (architecture). A fast *and* object-store-native engine is entirely possible; the two are not
   causally linked.

**So — which system for what:**

- **ClickHouse = the analytics/retrieval engine.** Built to scan and aggregate huge data fast; wins
  when query speed and the "build on top" surface dominate. Object storage is available but OSS HA is
  N× cost and elasticity needs Cloud.
- **GreptimeDB = the observability-native, object-store-native store.** Timestamp-first layout, native
  cardinality handling, and — the edge the proxy does **not** neutralize — **cheap, elastic,
  object-store economics: fewer always-on servers, pay mostly for inexpensive storage.** Slightly
  slower on heavy ad-hoc analytics (engine), but the cost/scaling architecture is its genuine win.

**The operator's cost intuition is correct and is GreptimeDB's strongest *surviving* argument under the
proxy lens.** "Fewer compute servers, more cheap storage" = compute/storage separation + 1× S3
economics — exactly one of the three things the proxy does not neutralize. If self-hosted
cost-at-scale is the priority, this is GreptimeDB's best case. Just don't attribute the *speed* gap to
the storage model — it's the engine, and it's closable.

## Historical skeptical re-score (superseded)

> **Superseded as the standing lean (2026-05-29; operator re-affirmed 2026-06-11).** This
> section's conclusion held while the query mix was unresolved. The operator then resolved the
> mix as **anchored-bundle-retrieval-dominant**, which moves ClickHouse's retrieval-speed lead
> off the hot path and turns the decision on cost + Rust — where GreptimeDB leads. The current
> product engine is **GreptimeDB**; ClickHouse is comparator-only; see
> [decisions/storage-engine.md](../../decisions/storage-engine.md). The analysis below remains
> valid as the answer to a different question ("what if the mix were analytics-dominated?") and
> as the documented flip condition.

The historical re-score found that once ingest ergonomics were removed from the
scorecard, the case for GreptimeDB narrowed and ClickHouse looked like the
pragmatic default. This was analysis, not current product authority.

- **Choose ClickHouse if** the dominant value is fast retrieval over heterogeneous telemetry + the
  widest "build on top" surface + a proven unified-obs backend you don't have to de-risk. This fits
  Parallax's AI-debugging query mix (scan/aggregate/correlate evidence), and Parallax's proxy supplies
  the OTLP/PromQL/Jaeger APIs GreptimeDB would otherwise give for free. **This is the lower-risk,
  higher-ceiling default.**
- **Choose GreptimeDB only if** one of its non-neutralized edges is decisive: (a) PromQL-native,
  cardinality-insensitive **metrics** are a first-class product surface and Parallax does *not* want to
  pre-model/pre-aggregate them; (b) **self-hosted HA storage economics** at large scale (1× vs N× S3)
  dominate the cost model; (c) **zero-ops horizontal auto-rebalance** from tiny→huge is a hard
  requirement and ClickHouse Cloud is off the table.

The earlier verdict leaned GreptimeDB largely on ingest-nativeness + scaling design. **The proxy
removes the first pillar.** What's left (scaling design, object-store economics, metrics cardinality)
is a narrower, more conditional case. So: **the honest reconsideration tilts toward ClickHouse as
Parallax's default store, with GreptimeDB reserved for the metrics-heavy / self-hosted-economics /
auto-scale-mandatory bet.** This is a genuine shift from the prior "GreptimeDB on fit" lean — driven
by the operator's proxy architecture, not by raw speed alone.

## Are there better alternatives than GT/CH? (skeptical survey, language filter applies)

Within the hard language filter (Rust/Go/Zig/C++/C; no JVM/Python/Ruby), candidates for a **single
store Parallax embeds as a backend**:

| System | Lang | Unified m/l/t? | Verdict for Parallax-as-backend |
| --- | --- | --- | --- |
| **ClickHouse** | C++ | Yes (proven: SigNoz/Uptrace/HyperDX/ClickStack) | Historical comparator winner on retrieval/ecosystem; not a Parallax product target. |
| **GreptimeDB** | Rust | Yes (one engine) | The Rust-native challenger; wins metrics-cardinality + object-store economics + auto-scale. |
| **OpenObserve** | Rust | Yes (DataFusion+Parquet+tantivy on S3) | **A competitor *platform*, not an embeddable DB.** You'd query its APIs, not run SQL against a store you own — wrong layer for "Parallax owns the platform." Study it as a rival, not a backend. |
| **Quickwit** | Rust | Logs/traces only (no metrics); sub-second search on S3 | Not unified (no metrics). Datadog-acquired → OSS-sustainability risk. Strong *log/trace search* tech only. |
| **InfluxDB 3 (IOx)** | Rust | Metrics-first (Arrow/Parquet/DataFusion); logs/traces nascent | Metrics-centric; clustering is commercial; weaker unified story than CH/GT. |
| **VictoriaMetrics + VictoriaLogs** | Go | **Two separate products** (metrics; logs); traces weak | Efficient, but not one unified store — two systems to operate; not a single-backend answer. |
| **StarRocks / Apache Doris** | C++ (Doris has a **JVM frontend** → filter risk) | OLAP, used for some obs | Heavier ops, not obs-purpose-built; Doris JVM FE trips the filter. No clear win over ClickHouse. |

**Historical conclusion:** no third candidate clearly beat either comparison
engine. OpenObserve was the most interesting Rust unified platform but the wrong
ownership layer. Current Parallax product work is not choosing among these
engines; it fixes forward on GreptimeDB.

## The data model: metrics + logs + traces + grouped-errors + metadata → a 2–3 store split

Parallax will hold: **metrics, logs, traces**, plus **Sentry-style grouped errors** and **metadata**.
These are not one workload, and forcing them into one engine is the mistake.

- **High-volume append telemetry** (metrics, logs, traces, raw error *events*):
  mandatory GreptimeDB native tables. ClickHouse only demonstrates comparator
  behavior for this workload.
- **Sentry-style grouped errors** (the "issue": fingerprint → first_seen, last_seen, count, status
  resolved/ignored/regressed, assignee, notes): this is **mutable, relational, low-volume OLTP**
  (millions of issues, not billions of events), with frequent UPDATEs and lookups/joins to
  projects/users. **Neither ClickHouse nor GreptimeDB is good at this.** Sentry's own architecture
  proves it: **Postgres holds users/projects/settings + issue metadata; ClickHouse (Snuba) holds the
  event firehose** — and Sentry had to build a special **"replacements consumer"** *because ClickHouse
  cannot do easy row UPDATEs* (merge/unmerge/resolve mutate issues). That pain is the tell.
- **The relational metadata tier is already chosen — and it is the right home for grouped errors.** The
  operator's standing direction is mandatory **Turso** for users, projects,
  issue status, policies, audit records, sessions, invocations, and outcomes.
  Postgres remains useful comparison evidence but is not needed or authorized as
  a product engine.
  Split:
  - **Relational metadata store (Turso)** = issue identity (fingerprint →
    issue), mutable workflow state (status, assignee, snooze, notes), first/last-seen, projects, users,
    DSNs, dashboards, alert rules, config. Transactional, indexed, small.
  - **Columnar store (GreptimeDB)** = the raw event/log/trace/metric firehose + the
    *computed* aggregates. The grouped-error *count/first-seen/last-seen* can be **computed on read**
    (ClickHouse `argMin/argMax/count` is fast) or **materialized** (ClickHouse MV / GreptimeDB Flow,
    see `rollup-and-continuous-aggregation.md`) keyed by fingerprint; the *human/workflow* fields live
    in the relational store.
  - **Object storage (S3/R2)** = cold tier for the columnar firehose (already in `retention-cost-model.md`).

This 3-tier split (relational OLTP metadata + columnar telemetry + object cold tier) is the **proven
Sentry/SigNoz shape**, and it is backend-agnostic: it works whether the columnar store is ClickHouse or
GreptimeDB. **Do not try to keep grouped-issue mutable state in the columnar engine** — that is fighting
the tool (Sentry's replacements-consumer scar tissue is the warning). Note Turso (libSQL/SQLite
lineage, C + a Rust rewrite) passes the language filter; this is consistent with the existing decision,
not a new dependency.

## Is "ClickHouse wins build-on-top" a SQL-capability gap or an ecosystem gap? (live, Run 156)

The re-score weights "build-on-top ecosystem" as a central ClickHouse win. Tested whether that's a
**query-capability** gap (can GreptimeDB even express Parallax's core analytical patterns?) or an
**ecosystem/maturity** gap. Live, via `docker exec`, both engines on the same prior-run data:

- **Grouped-error rollup** (the Sentry-style aggregate — fingerprint → count, first_seen, last_seen,
  latest message): **parity, identical results.** ClickHouse uses `argMax(message, ts)`; GreptimeDB uses
  `last_value(message ORDER BY ts)` — different dialect, same answer (both returned `fp-135`, count 21,
  matching first/last timestamps).
- **Evidence-bundle window ranking** (`row_number() OVER (PARTITION BY trace_id ORDER BY duration_ms
  DESC)` — "slowest spans in this trace"): **parity, identical results** (both ranked 59.46→1,
  48.583→2, 43.545→3).

**Conclusion: the build-on-top advantage is ecosystem/maturity, NOT a SQL-capability gap for Parallax's
queries.** Both columnar engines express the grouped-error rollup *and* the evidence-bundle window
queries correctly. So ClickHouse's "build on top" edge is the *ecosystem* (de-facto obs backend,
integrations, MV maturity, the SigNoz/HyperDX/ClickStack network effect, exotic function breadth), not
that GreptimeDB can't run the queries. **Two decision consequences:** (1) GreptimeDB is **not
capability-blocked** for the new grouped-error requirement — the rollup computes fine on it; (2) the
columnar choice (CH vs GT) is therefore decided by **retrieval speed + ecosystem + cost**, not by
whether Parallax's analytical queries are expressible (they are, on both). The grouped-error
*aggregate* runs on either columnar engine; only the *mutable workflow state* needs the relational
store (above).

## Current Product Consequence

1. GreptimeDB remains the mandatory telemetry engine; benchmark disadvantages
   create risk disclosures or fix-forward work.
2. Turso remains the mandatory mutable metadata store; do not add Postgres.
3. ClickHouse and other systems remain research comparators.
4. No implementation begins from this note. Contract corrections are plan 093,
   and any server topology is plan 115.

## Historical flip conditions (research only)

- Parallax's product centre of gravity turns out to be **metrics/PromQL dashboards over huge
  cardinality**, and pre-aggregation on ClickHouse proves too lossy/awkward.
- The deployment model is **self-hosted HA at large scale** where 1× vs N× S3 (and no Keeper) is a
  decisive cost/ops win, and ClickHouse Cloud is not an option.
- The **startups→big auto-rebalance** requirement is hard and operator-managed ClickHouse resharding is
  unacceptable.
These conditions explain the old re-score. They have no current stack-selection
authority.

## Sources / evidence

- Internal: the full `greptimedb-vs-clickhouse/` record — `verdict-which-to-choose.md` (axes),
  `query-execution-engine.md` (retrieval-speed gap), `compression-and-cost.md` (cost ~tie),
  `distributed-and-scaling.md` (object-store economics, auto-rebalance), `metric-cardinality.md`
  (metric-engine), `write-path-and-ingestion.md` (the now-neutralized native-ingest trio),
  `rollup-and-continuous-aggregation.md` (rollups for grouped-error aggregates).
- External (2026): ClickHouse as de-facto obs backend — [ClickStack](https://clickhouse.com/clickstack),
  [SigNoz on ClickHouse](https://clickhouse.com/blog/signoz-observability-solution-with-clickhouse-and-open-telemetry),
  [best open-source observability 2026](https://clickhouse.com/resources/engineering/best-open-source-observability-solutions);
  Sentry split — [Snuba architecture](https://getsentry.github.io/snuba/architecture/overview.html),
  [Sentry self-hosted data flow](https://develop.sentry.dev/self-hosted/data-flow/);
  [OpenObserve](https://github.com/openobserve/openobserve) (Rust unified platform);
  GreptimeDB pitch (treat as vendor) — [GreptimeDB as a ClickHouse alternative](https://greptime.com/tech-content/2026-04-17-clickhouse-alternative-greptimedb).

## Run 212 (2026-07-17) — alternatives scan still holds (post re-pin)

No third system re-entered the backend shortlist after the v1.1.3 / 26.6 re-pin
cycle (Runs 173–211). Still true:

- **OpenObserve** — competitor *platform*, not embeddable DB backend.
- **Quickwit** — logs/traces search only.
- **InfluxDB 3 / VictoriaMetrics** — split products / different product surface.
- **StarRocks/Doris** — JVM-FE filter risk for this repo.

**Backend shortlist remains GT (product) vs CH (comparator).** Managed-cloud
framework (Run 175) and engine re-verifies do not reopen the field.
