# A5 Stack Decision Ledger

<!-- markdownlint-disable MD013 -->

Research date: 2026-05-25

## Purpose

This note turns assumption A5 from
[Risks and the bear case](risks-and-bear-case.md) into a roll-up claim ledger:

> The chosen stack holds: GreptimeDB speed/cost, Turso reliability, and Iggy
> where used.

Parallax already has focused proof gates for storage speed, storage cost,
metadata, ingest-log replay/backpressure, and self-hosted setup. The missing
piece was a rule for when those component results are allowed to become a stack
decision. This ledger owns that boundary.

The decision rule is deliberately conservative: **no component benchmark creates
an A5 pass by itself.** A5 passes only when the current run manifest proves the
selected profile end to end, records exact versions and settings, and names the
fallback trigger that would change the default.

## Current Primary-Source Checks

| Source | Current read for A5 |
| --- | --- |
| [GreptimeDB standalone docs](https://docs.greptime.com/getting-started/installation/greptimedb-standalone/) | GreptimeDB `v1.0.2` is the current documented standalone binary/Docker example in the 1.0 docs, with one-process local persistence suitable for the tiny-tier storage candidate. |
| [GreptimeDB configuration docs](https://docs.greptime.com/user-guide/deployments/configuration/) | GreptimeDB supports local file storage plus S3-compatible, Azure Blob, Aliyun OSS, and GCS storage, has object-storage cache settings, and exposes WAL durability settings such as `sync_write`. A5 storage claims must pin these settings. |
| [ClickHouse MergeTree docs](https://clickhouse.com/docs/engines/table-engines/mergetree-family/mergetree) | ClickHouse remains the benchmark fallback for high-volume observability data: MergeTree targets high ingest and large data volumes, supports TTL and disk/object-storage movement, and has mature concurrent-read behavior. |
| [ClickHouse insert strategy docs](https://clickhouse.com/docs/best-practices/selecting-an-insert-strategy) | ClickHouse performance claims depend on insert mode: synchronous inserts need client-side batches, while async inserts change acknowledgement/error behavior. A5 cannot compare candidates without recording insert mode. |
| [Turso libSQL docs](https://docs.turso.tech/libsql) | Turso documents libSQL as production-ready and Turso Database as evolving beta. A5 must distinguish libSQL/Turso Database/local file/Cloud usage instead of saying only "Turso." |
| [Turso SDK docs](https://docs.turso.tech/sdk/introduction) | Turso recommends Turso Database packages for new local/embedded work and libSQL packages as production-ready ORM fallbacks. The metadata default must record which package/engine is under test. |
| [Turso local development docs](https://docs.turso.tech/local-development) | Local development can use SQLite files, `turso dev`, or remote Turso; `turso dev --db-file` persists a local file. The tiny profile must not quietly depend on hosted Turso. |
| [Turso Database `v0.6.1` release](https://github.com/tursodatabase/turso/releases/tag/v0.6.1) | Latest checked Turso Database release is `v0.6.1`, published 2026-05-22. Its release notes are still heavy on MVCC correctness fixes, which supports keeping production claims gated. |
| [Apache Iggy architecture docs](https://iggy.apache.org/docs/introduction/architecture/) | Iggy has the right append-only stream shape: streams, topics, partitions, segment files, consumer groups, offsets, and metadata WAL. It fits the durable-stream candidate role. |
| [Apache Iggy 0.8.0 release](https://iggy.apache.org/blogs/2026/04/22/release-0.8.0/) | 0.8.0 hardens the path toward clustering, but the release still frames clustering as upcoming work through `iggy-server-ng`. |
| [Iggy clustering status issue](https://github.com/apache/iggy/issues/2562) | Public docs still need clearer clustering/replication status. A5 must not treat Iggy as a production HA clustered dependency until that changes and fault tests pass. |
| [Apache Iggy incubator status](https://incubator.apache.org/projects/iggy.html) | Iggy entered the Apache Incubator on 2025-02-04. Incubation is acceptable for prototype evaluation, not for unstated Tier-3 HA dependence. |
| [PostgreSQL backup docs](https://www.postgresql.org/docs/current/backup.html) and [concurrency docs](https://www.postgresql.org/docs/current/mvcc.html) | Postgres remains the mature metadata fallback with documented backup/restore and concurrency behavior. It is heavier than the tiny tier, but the production fallback must be real. |

## Ledger Artifacts

Every A5 run should produce a durable, source-linked result bundle:

```text
docs/research/stack-decision-results.md
docs/research/stack-decision-runs/<run_id>/manifest.json
docs/research/stack-decision-runs/<run_id>/storage-speed.jsonl
docs/research/stack-decision-runs/<run_id>/storage-cost.jsonl
docs/research/stack-decision-runs/<run_id>/metadata-store.jsonl
docs/research/stack-decision-runs/<run_id>/ingest-log.jsonl
docs/research/stack-decision-runs/<run_id>/setup-ops.jsonl
docs/research/stack-decision-runs/<run_id>/integration.jsonl
docs/research/stack-decision-runs/<run_id>/decision-ledger.jsonl
docs/research/stack-decision-runs/<run_id>/hashes.sha256
```

`stack-decision-results.md` is the human-readable summary. The JSONL files are
the auditable ledger rows. Large generated datasets, database directories,
object-store buckets, and raw benchmark outputs should remain out of git unless
they are intentionally reduced fixtures.

## Run Manifest

The run manifest must include:

| Field | Required content |
| --- | --- |
| `run_id` | Stable id such as `a5-2026-05-25-small-001`. |
| `research_date` | Date the source check and run started. |
| `profile` | `tiny`, `durable-single`, `scale-out`, or `production-metadata`. |
| `claim_target` | The exact claim being tested, not a broad stack slogan. |
| `operator` | Human or agent that ran the benchmark. |
| `hardware` | Host, CPU, RAM, disk, filesystem, network/object-store locality. |
| `container_or_binary_versions` | Exact image tags, binary versions, commit SHAs, and checksums where practical. |
| `config_hashes` | Hashes for compose files, benchmark configs, DDL, schema migrations, and adapter code. |
| `source_snapshot` | URLs and checked dates for official docs, release notes, pricing pages, and known caveats. |
| `dataset` | Generator seed, tier, duration, signal mix, row counts, raw bytes, and fixture version. |
| `durability_modes` | Storage WAL settings, stream ack mode, fsync/quorum settings, and retry behavior. |
| `redaction_mode` | Redaction fixture version if result rows include bundle projections. |
| `known_invalid_claims` | Explicit statements this run is not allowed to support. |

## Component Rows

Each component result row should contain `run_id`, `component`, `candidate`,
`version`, `profile`, `status`, `evidence_path`, `source_url`, `checked_at`, and
`notes`.

### Storage Speed

Owned by
[Storage freshness and bundle latency gate](storage-freshness-and-bundle-latency-gate.md).

Required fields:

| Field | Meaning |
| --- | --- |
| `candidate` | `greptimedb`, `clickhouse`, or later storage candidate. |
| `schema_variant` | DDL/schema version and indexing choices. |
| `insert_mode` | Sync/async/client-batched/OTLP/native path, with ack semantics. |
| `freshness_p50_ms`, `freshness_p95_ms`, `freshness_p99_ms` | Ingest-to-queryable visibility by signal. |
| `q1_warm_p95_ms`, `q6_warm_p95_ms` | Trace-context and evidence-bundle query latency. |
| `mixed_ingest_penalty` | Query slowdown under concurrent ingest. |
| `stale_bundle_rate` | Bundle rows missing generated references inside the freshness window. |
| `pass_threshold` | The threshold used at run time. |

### Storage Cost

Owned by
[Storage size and object cost gate](storage-size-and-object-cost-gate.md).

Required fields:

| Field | Meaning |
| --- | --- |
| `raw_bytes_by_signal` | Generated uncompressed source bytes by signal. |
| `retained_bytes_by_signal` | Local and object-store retained bytes by signal. |
| `compression_ratio_by_signal` | Retained/raw ratio. |
| `object_count` | Number of objects/parts by age tier and signal. |
| `provider` | Local disk, MinIO, R2, B2, S3, or other provider. |
| `pricing_source_url` | Official pricing source used for the cost projection. |
| `pricing_checked_at` | Date/time pricing was checked. |
| `monthly_cost_projection` | 7/30/90-day retention model at 0/10/20/50 percent reread. |
| `cache_dependency` | Whether passing latency depends on local cache warmth. |

### Metadata Store

Owned by
[Metadata store benchmark plan](metadata-store-benchmark-plan.md) and
[Turso metadata production readiness](turso-metadata-production-readiness.md).

Required fields:

| Field | Meaning |
| --- | --- |
| `engine` | `turso-database`, `libsql`, `sqlite`, `postgres`, or other. |
| `mode` | Embedded file, local server, hosted/cloud, sync, or Postgres fallback. |
| `package` | SDK/package/crate name and exact version. |
| `hot_write_p95_ms` | Agent/session/outcome write latency under target concurrency. |
| `read_p95_ms` | Issue, session, audit, and bundle-reference reads. |
| `crash_recovery_status` | Whether committed state survives process/OS crash tests. |
| `backup_restore_status` | Export, restore, PITR/snapshot status and duration. |
| `migration_rollback_status` | Forward/backward migration safety. |
| `postgres_fallback_status` | Whether logical rows round-trip into Postgres with stable ids. |

### Ingest Log

Owned by
[Ingest log replay and backpressure gate](ingest-log-replay-and-backpressure-gate.md).

Required fields:

| Field | Meaning |
| --- | --- |
| `mode` | `local-wal:batch-fsync`, `local-wal:strict-fsync`, `iggy:standalone`, `nats:jetstream-r3`, `redpanda:r3`, or later mode. |
| `ack_durability` | `process-durable`, `fsync-durable`, `quorum-durable`, or weaker. |
| `accepted_loss_count` | Acknowledged payloads lost under configured durability mode. |
| `duplicate_raw_deliveries` | Raw duplicate delivery count before idempotent writes. |
| `duplicate_normalized_events` | Must be zero after idempotency keys. |
| `replay_speed_x_realtime` | Replay throughput relative to expected startup raw ingest. |
| `producer_ack_p95_ms` | Producer acknowledgement latency by payload class. |
| `backpressure_start` | Disk/lag threshold where retryable pressure starts. |
| `recovery_seconds` | Process/broker restart recovery time. |
| `memory_mb` | Incremental memory footprint for stream/WAL mode. |

### Setup And Operations

Owned by
[Self-hosted simplicity gate](self-hosted-simplicity-gate.md).

Required fields:

| Field | Meaning |
| --- | --- |
| `time_to_first_bundle_minutes` | Fresh VM to first `parallax issue context` result. |
| `long_running_services` | Count and names of required services. |
| `required_resources` | Required and measured CPU/RAM/disk. |
| `ports_and_secrets` | Exposed ports and required/generated secrets. |
| `backup_restore_minutes` | Setup-specific backup/restore proof. |
| `upgrade_steps` | Count and description of documented upgrade steps. |
| `external_dependencies` | Hosted/cloud/broker/object-store requirements. |

### Integration

The integration row prevents single-component green lights from hiding broken
handoffs.

Required fields:

| Field | Meaning |
| --- | --- |
| `sentry_fixture_status` | Whether current SDK fixtures ingest and normalize through the chosen stack. |
| `otlp_fixture_status` | Whether OTLP traces/logs/metrics ingest through the chosen stack. |
| `bundle_projection_status` | Whether the chosen stack returns the generated issue context bundle. |
| `restart_durability_status` | Whether stop/start keeps raw event, issue, trace link, metadata, and bundle refs. |
| `cross_component_backpressure` | Whether storage/metadata/stream outages produce retryable pressure before loss. |
| `operator_visible_failure` | Whether failures surface as health/lag/lossiness reports, not silent gaps. |

## Claim Levels

Use these exact claim levels in `decision-ledger.jsonl`:

| Level | Meaning |
| --- | --- |
| `not_measured` | No current compatible run. No product claim allowed. |
| `smoke_only` | Starts and passes fixture smoke tests only. Suitable for prototype planning, not default claims. |
| `greptime_prototype_default` | GreptimeDB may be named as the first storage prototype for the tested profile. |
| `clickhouse_storage_default` | ClickHouse replaces GreptimeDB as storage default for the tested profile. |
| `dual_storage_open` | Neither storage candidate clearly wins; architecture must keep storage swappable. |
| `turso_prototype_metadata` | Turso/Turso Database/libSQL may be used for tiny/prototype metadata only, with production caveat. |
| `postgres_production_metadata` | Postgres is the production metadata default or fallback for the tested profile. |
| `local_wal_tiny_default` | Local WAL remains the tiny-profile ingest-log default. |
| `iggy_optional_profile` | Iggy may be offered for durable single-node replay, not required tiny-tier operation. |
| `nats_or_redpanda_clustered_fallback` | NATS or Redpanda is the current clustered stream fallback; Iggy is not a clustered default. |
| `phase1_stack_pass` | The tiny profile passes end-to-end for the tested workload and may be used for Phase 1 claims. |
| `claim_expired` | A prior result is stale because versions, docs, pricing, hardware profile, or workload changed materially. |
| `claim_failed` | The tested claim is false for the recorded configuration. |

## Roll-Up Decision Rules

### Storage Default

GreptimeDB may be the tested profile's storage default only if:

1. It passes mixed-load freshness and Q6 bundle latency gates.
2. It passes retained-size/object-cost gates or records an explicit operational
   advantage that justifies a bounded cost premium.
3. The result pins storage mode, WAL settings, cache settings, schema, insert
   mode, and object-store provider.
4. ClickHouse does not beat it by enough to justify the operational cost for
   Parallax's narrow bundle workload.

ClickHouse becomes the storage default if it passes freshness/cost gates and
GreptimeDB fails, or if GreptimeDB passes only through settings that undermine
durability, freshness, or operator simplicity.

If both pass with tradeoffs, record `dual_storage_open` and keep the storage
adapter boundary real.

### Metadata Default

Turso may be the tiny/prototype metadata default only if:

1. The row states whether it tested Turso Database, libSQL, SQLite-file mode,
   local `turso dev`, hosted Turso, or sync.
2. Crash recovery, hot write contention, backup/restore, migration rollback,
   and audit/history invariants pass for the tested workload.
3. Hosted Turso is not required for the tiny self-hosted profile.

Turso must not be called production metadata by default while Turso Database is
still recorded as beta/evolving and production-readiness gates are incomplete.
Use `postgres_production_metadata` when production backup/restore, operational
maturity, or migration safety matters more than tiny-tier simplicity.

### Ingest Log Default

Local WAL remains the tiny default if it passes acknowledged-payload durability,
replay speed, backpressure, recovery, and setup-simplicity targets.

Iggy may become an optional durable single-node profile only if standalone
durability, consumer groups, replay/backfill, memory, and setup burden pass.
Iggy must not become a clustered HA default until clustering is released,
documented, and passes the multi-node fault matrix.

NATS or Redpanda becomes the clustered fallback only after a current run shows
which one meets replay/durability/backpressure needs with acceptable operator
burden and licensing posture.

### Phase 1 Stack Pass

`phase1_stack_pass` requires all of the following in the same run family:

1. Sentry SDK fixture event and OTLP fixture data ingest successfully.
2. Storage returns the generated issue context bundle inside freshness/latency
   budgets.
3. Metadata survives restart and preserves issue, session, audit, and
   bundle-reference state.
4. Ingest log mode preserves acknowledged raw payloads under its declared
   durability mode.
5. Stop/start all services keeps raw event, issue grouping, trace link, and
   bundle references.
6. The tiny setup stays within the self-hosted simplicity gate.
7. Any redaction fixture included in bundle output reports seeded canary status.

If any component fails, A5 records the failure component and fallback trigger
instead of weakening the product claim.

## Fallback Triggers

| Component | Trigger | Default consequence |
| --- | --- | --- |
| Storage speed | Freshness p95 or Q6 p95 misses threshold under mixed ingest. | Switch to ClickHouse if it passes, or narrow MVP signal retention. |
| Storage cost | Retained bytes/object count/provider costs exceed budget without offsetting simplicity. | Change schema/retention or switch object-store/candidate. |
| Metadata | Turso/local metadata fails crash, backup/restore, contention, or migration gates. | Use Postgres for production metadata; keep Turso only for prototype/tiny if safe. |
| Ingest log | Local WAL loses acknowledged data or cannot backpressure before disk risk. | Add Iggy/NATS/Redpanda profile or reject the target workload. |
| Iggy | Standalone stream fails replay/durability/memory/setup gates, or clustering remains unproven for HA. | Keep local WAL tiny default and NATS/Redpanda clustered fallback. |
| Setup | More than three tiny-tier services, >15 minutes to first bundle, or hidden hosted dependency. | Narrow MVP or reject the default stack. |
| Integration | Component green results do not produce one durable issue context bundle after restart. | No A5 pass; fix handoffs before changing claims. |

## Freshness Rules

A5 result claims expire when any of these happen:

- a tested candidate releases a new major/minor version used in the default
  profile;
- official docs change the maturity, deployment, durability, or support posture
  of a load-bearing component;
- object-store pricing or request/egress rules change materially;
- the benchmark workload, schema, hardware class, or deployment profile changes;
- a safety gate such as redaction or production-data access invalidates the
  bundle projection layer;
- 90 days pass without re-running the relevant component gate.

Expired results stay useful as history, but the decision row must move to
`claim_expired` before public or roadmap wording relies on it.

## Relationship To Other Research

- [Risks and bear case](risks-and-bear-case.md) owns assumption A5. This ledger
  supplies the result contract for that assumption.
- [Build roadmap and validation sequence](build-roadmap-and-validation-sequence.md)
  places A5 in Phase 2 after A1/A2 have earned the engineering investment.
- [Storage benchmark prototype](storage-benchmark-prototype.md) owns the shared
  harness and candidate storage adapter model.
- [Storage freshness and bundle latency gate](storage-freshness-and-bundle-latency-gate.md)
  owns speed/freshness rows.
- [Storage size and object cost gate](storage-size-and-object-cost-gate.md)
  owns storage cost rows.
- [Turso metadata production readiness](turso-metadata-production-readiness.md)
  owns metadata maturity and fallback rows.
- [Ingest log replay and backpressure gate](ingest-log-replay-and-backpressure-gate.md)
  owns stream/WAL durability, replay, and backpressure rows.
- [Self-hosted simplicity gate](self-hosted-simplicity-gate.md) owns setup and
  operator-burden rows.
- [Strategic verdict and research coverage](strategic-verdict-and-research-coverage.md)
  should treat this as the A5 stack-proof umbrella, not another component gate.

## Bottom Line

A5 should be reported as a ledger, not a vibe. The current architecture remains
reasonable, but the honest claim is:

> GreptimeDB, Turso/libSQL/Turso Database, local WAL, Iggy, ClickHouse, NATS,
> Redpanda, and Postgres are candidates behind explicit boundaries. Parallax can
> claim a stack default only after current, version-pinned component gates roll
> up into one end-to-end decision row for the exact deployment profile.

Until then, the safe wording is "prototype default" or "fallback candidate,"
not "the stack holds."
