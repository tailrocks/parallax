# Ingest Log Replay And Backpressure Gate

<!-- markdownlint-disable MD013 -->

Research date: 2026-05-25

## Purpose

This note tightens proof gate 4 from
[Strategic verdict and research coverage](strategic-verdict-and-research-coverage.md):

> Iggy replay and backpressure behavior versus local WAL and NATS/Redpanda.

The current architecture already says "local WAL first, Apache Iggy later." This
note turns that into a falsifiable gate. The decision must be based on accepted
message durability, replay speed, backpressure behavior, recovery complexity,
and operational burden under faults, not on the attractiveness of any broker's
architecture.

Decision: **the tiny profile stays local-WAL-first until a stream proves a
specific replay or processor-isolation need.** Apache Iggy is the first durable
single-node stream to prototype. NATS JetStream and Redpanda are the scale-out
fallbacks until Iggy ships and proves production clustering.

## Current Primary-Source Checks

| Source | What matters for Parallax |
| --- | --- |
| [Apache Iggy architecture](https://iggy.apache.org/docs/introduction/architecture/) | Iggy is a persistent append-only message-streaming system with streams, topics, partitions, segment files, retention policies, compression, consumer groups, and partition rebalancing. This matches Parallax's raw ingest replay model closely. |
| [Apache Iggy 0.8.0 release](https://iggy.apache.org/blogs/2026/04/22/release-0.8.0/) | The 0.8.0 release is focused on internal restructuring toward clustering: VSR foundations, shard routing, persistent WAL metadata journal, WAL-backed client table, and `iggy-server-ng`. That is progress, but still points to clustering as future work. |
| [Iggy clustering status issue](https://github.com/apache/iggy/issues/2562) | The public issue asks for explicit documentation of whether Iggy supports multi-node clustering or is single-node today. Parallax should not treat Iggy clustering as production-ready until docs, release notes, and fault tests prove it. |
| [Apache Iggy incubation status](https://incubator.apache.org/projects/iggy.html) | Iggy entered the Apache Incubator on 2025-02-04 and is still an incubating project. This is compatible with prototyping, but not with making it a load-bearing Tier 3 HA dependency. |
| [NATS JetStream concepts](https://docs.nats.io/nats-concepts/jetstream) | JetStream supports complete replay, original-rate replay, replication factors, source/mirror streams, and file-based persistence. Its docs also state that default file persistence flushes to the OS but does not immediately `fsync`, with a default `sync_interval` of 2 minutes. |
| [NATS JetStream consumers](https://docs.nats.io/nats-concepts/jetstream/consumers) | JetStream consumers track delivery and acknowledgements and can provide at-least-once delivery. This maps well to Parallax worker restarts and replay, but duplicate handling remains Parallax's responsibility. |
| [Jepsen NATS 2.12.1](https://jepsen.io/analyses/nats-2.12.1) | Jepsen found acknowledged-write loss under file truncation/corruption and coordinated power-failure scenarios, tied partly to the default 2-minute fsync interval. JetStream remains a strong fallback, but Parallax must test exact settings such as `sync_interval: always`. |
| [Redpanda architecture](https://docs.redpanda.com/current/get-started/architecture/) | Redpanda is a serious clustered log fallback and its tiered storage architecture can offload log segments to object storage while preserving API access to historical offsets. |
| [Redpanda tiered storage docs](https://docs.redpanda.com/24.2/manage/tiered-storage/) | Self-managed Tiered Storage requires an Enterprise license, and docs warn about mixed-version upload stalls and re-enabling risks. This conflicts with Parallax's open self-hosted cheap-retention posture if Redpanda becomes more than a short replay buffer. |

## IngestLog Contract

Parallax should implement an internal `IngestLog` interface before committing to
any broker:

```rust
#[async_trait]
pub trait IngestLog {
    async fn append(&self, topic: Topic, key: PartitionKey, payload: Bytes) -> Result<LogAck>;
    async fn subscribe(&self, topic: Topic, group: ConsumerGroup) -> Result<LogConsumer>;
    async fn ack(&self, cursor: LogCursor) -> Result<()>;
    async fn nack(&self, cursor: LogCursor, reason: NackReason) -> Result<()>;
    async fn replay(&self, topic: Topic, range: ReplayRange) -> Result<LogConsumer>;
    async fn lag(&self, topic: Topic, group: ConsumerGroup) -> Result<LagReport>;
    async fn backpressure(&self) -> Result<BackpressureState>;
}
```

The contract must preserve these invariants across local WAL, Iggy, NATS, and
Redpanda:

| Invariant | Required behavior |
| --- | --- |
| Accepted means replayable | Once ingest returns success, the raw payload can be recovered or replayed according to that profile's documented durability mode. |
| Idempotent downstream writes | Every payload has `event_id`, `ingest_id`, source idempotency key, and `raw_ref` so retries do not duplicate normalized events. |
| Explicit durability mode | The ack path records whether it was `process-durable`, `fsync-durable`, `quorum-durable`, or weaker. Hidden best-effort acks are not allowed. |
| Bounded local disk | Each topic has retention by age and bytes plus a hard stop/backpressure policy before disk-full corruption. |
| Backpressure before loss | When storage or processors fall behind, producers receive retryable pressure before accepted data is dropped. |
| Replay by stable cursor | Normalizers, grouping, symbolication, and context builders can replay by offset/time/window without reading from object storage manually. |
| Lossiness report | Any dropped, dead-lettered, expired, or unparseable payload is counted with topic, reason, and policy version. |

## Candidate Modes To Test

| Mode | Role | What must be proven |
| --- | --- | --- |
| `local-wal:batch-fsync` | Tiny default. Append accepted payloads to segment files, fsync per batch or time window, process in-process or through a local worker queue. | Process crash safety, restart recovery, bounded disk behavior, simple replay, and no extra service. |
| `local-wal:strict-fsync` | Sensitive-mode baseline. Fsync before acknowledging high-value error envelopes. | Stronger durability cost for low-volume Sentry/error events. |
| `iggy:standalone` | Durable single-server profile. | Durable ack semantics, replay throughput, consumer group behavior, retention, memory tuning, and recovery after segment faults. |
| `nats:jetstream-r3` | OSS clustered fallback. | Replicated stream behavior, pull consumer scaling, replay policy, duplicate rate, and explicit `sync_interval` durability tradeoff. |
| `redpanda:r3` | Kafka-like clustered fallback. | Raft partition durability, consumer group behavior, replay throughput, operational cost, and whether lack of open self-managed tiering matters when used only as a short replay buffer. |

Do not test only happy-path throughput. The selection criterion is "which
profile keeps accepted evidence recoverable and replayable while staying simpler
than self-hosted Sentry?"

## Workload Matrix

Use the same payload classes across all modes:

| Workload | Payload shape |
| --- | --- |
| Sentry envelope small | 2 KB Rust error event with stacktrace and trace context. |
| Sentry envelope large | 200 KB event with breadcrumbs, context, and bounded attachment metadata. |
| OTLP traces | Batches of 100, 1,000, and 10,000 spans. |
| OTLP logs | Small structured logs plus large stack-trace log records. |
| Metrics | Prometheus/OTLP metric batches with high-cardinality labels. |
| CLI invocation | Sanitized command metadata plus stdout/stderr raw refs. |
| Agent session | Tool/action timeline events plus raw transcript refs. |
| Mixed ingest | 80% logs, 15% spans, 4% metrics, 1% errors, plus low-rate CLI/agent events. |

Partition keys must match the correlation unit already defined in
[Messaging and ingestion layer](messaging-and-ingestion-layer.md): `project_id +
trace_id` for spans, `project_id + fingerprint_candidate` for errors,
`repository + agent_session_id` for agent sessions, and equivalent keys for CLI
and CI runs.

## Fault Matrix

Every mode must run these tests:

| Fault | Required measurement |
| --- | --- |
| Producer process kill | Last acknowledged event, last replayable event, duplicate count. |
| Ingest server kill | Accepted payload loss, restart recovery time, WAL/stream cursor correctness. |
| Broker/process kill | Acknowledged payload loss, uncommitted payload handling, recovery time. |
| OS crash / power-cut simulation | Difference between process-durable and fsync/quorum-durable modes. |
| Disk full | Whether backpressure starts before corruption or partial writes. |
| Segment truncation/corruption | Detection, quarantine, replay gap reporting, and operator visibility. |
| Storage writer outage for 10 minutes | Producer ack latency, spool growth, backpressure state, replay catch-up time. |
| Consumer restart/rebalance | Duplicate rate, missing-message rate, group lag, per-partition ordering. |
| Backfill while ingesting | Replay throughput and impact on live ingest p95/p99. |
| Upgrade/restart | Whether new binary reads old segments/topics and preserves cursors. |

For NATS, run at least two durability configurations:

- default or near-default file persistence;
- `sync_interval: always` or the nearest documented strongest setting.

For local WAL, run at least two configurations:

- batch fsync with a bounded interval;
- strict fsync for error-event topics.

For Redpanda, record whether any object-storage or long-retention behavior used
in the test requires an Enterprise license; Parallax should not hide a license
dependency inside the fallback path.

## Metrics And Initial Budgets

These are initial gates for a small self-hosted deployment, not production
claims:

| Metric | Tiny local WAL target | Durable stream target |
| --- | --- | --- |
| Accepted-message loss after process kill | 0 for acknowledged payloads in the configured durability mode. | 0 for acknowledged payloads in the configured durability mode. |
| Acknowledged-message loss after OS crash | Must match the declared durability mode; strict mode should lose 0 acknowledged error events. | Must match documented fsync/quorum settings; any loss must be reported as mode-specific, not silent. |
| Duplicate normalized events | 0 after idempotency keys; duplicate raw deliveries are allowed and counted. | Same. |
| Replay speed | Reprocess 24 hours of expected startup raw ingest at >= 10x realtime on the target small VPS profile. | Same, plus consumer lag catch-up under live ingest. |
| Producer p95 ack latency | <= 10 ms for small Sentry events in batch mode; strict mode reports its cost separately. | <= 25 ms for small events in single-node durable mode; clustered mode reports quorum cost separately. |
| Backpressure | Retryable pressure starts before 80% disk usage or before configured lag limit. | Same, plus per-consumer-group lag visibility. |
| Recovery time | <= 30 s after process kill for local WAL at startup tier. | <= 60 s for single-node stream; clustered mode reports leader/follower recovery separately. |
| Memory | <= 128 MB incremental memory for local WAL. | Must fit the durable single-server profile without violating the small VPS target. |

If a target is too strict under real measurements, revise the number in the
benchmark result, not the product claim. The gate exists to prevent accidental
"fast but lossy" ingestion.

## Pass/Fail Decision

### Local WAL Remains The Tiny Default If

1. It loses zero acknowledged payloads under process-kill tests in batch mode.
2. Strict mode loses zero acknowledged error events under OS-crash simulation.
3. It can replay 24 hours of expected startup raw ingest at >= 10x realtime.
4. It applies backpressure before disk-full and emits a clear lag/disk report.
5. It requires materially less operator setup than any external stream.

### Iggy Becomes The Durable Single-Server Default If

1. Standalone durability behavior is explicit and passes process-kill,
   restart, segment-truncation, and disk-full tests.
2. Consumer groups rebalance without missing acknowledged payloads.
3. Replay/backfill under live ingest does not starve current writes.
4. Memory can be tuned low enough for the durable single-server profile.
5. Operational setup remains simpler than NATS/Redpanda.

Iggy must **not** become the Tier 3 clustered stream until production clustering
is released, documented, and passes the same fault matrix under multi-node
failure.

### NATS JetStream Becomes The OSS Clustered Fallback If

1. The strongest acceptable `sync_interval`/replica configuration passes the
   acknowledged-message fault tests.
2. Pull consumers and replay behavior satisfy Parallax worker restart/backfill
   needs.
3. Operator complexity stays lower than Redpanda for the same durability class.
4. Jepsen-relevant hazards are either fixed upstream or explicitly mitigated in
   Parallax's required config.

### Redpanda Becomes The Clustered Fallback If

1. NATS cannot satisfy durability/backpressure/replay requirements.
2. Kafka-like partition logs materially reduce implementation risk.
3. The licensing posture is acceptable because Parallax uses Redpanda only as a
   short-retention replay buffer, not as the cheap long-term evidence archive.
4. Operational complexity is justified by measured fault behavior.

## Product Implication

The architecture should make these claims, and no stronger ones:

- "Parallax can run without an external broker in the tiny profile."
- "Parallax uses an append-only ingest-log interface so raw evidence can be
  replayed after parser, grouping, and correlation fixes."
- "Apache Iggy is the preferred single-node durable stream candidate, not yet a
  proven clustered dependency."
- "NATS JetStream and Redpanda remain measured fallbacks for clustered
  deployments."
- "Accepted evidence durability is reported by mode; Parallax does not hide
  best-effort acks behind durable-sounding language."

## Relationship To Other Research

- [Messaging and ingestion layer](messaging-and-ingestion-layer.md) is the broad
  stream-layer evaluation this gate operationalizes.
- [Technical implementation concept](technical-implementation-concept.md) should
  keep local WAL in the tiny profile and Iggy behind `IngestLog`.
- [Sentry-compatible ingestion](sentry-compatible-ingestion.md) depends on this
  gate for raw-envelope durability and replay.
- [OpenTelemetry protocol and context layer](opentelemetry-protocol-and-context-layer.md)
  depends on this gate for OTLP batch replay and backpressure.
- [Storage benchmark prototype](storage-benchmark-prototype.md) is separate:
  databases own long retention; this gate owns short raw replay and processor
  fan-out.
- [A5 stack decision ledger](a5-stack-decision-ledger.md) consumes this gate's
  local-WAL/Iggy/NATS/Redpanda rows before any ingest-log result can become a
  tiny, durable-single, or clustered stack default.

## Bottom Line

Parallax should build the ingest contract now and keep the runtime choice
swappable. The local WAL is the right tiny default. Iggy is the best Rust-shaped
single-node stream to test. NATS JetStream is the strongest open clustered
fallback if configured for explicit durability. Redpanda is the strongest
Kafka-like fallback if operational and licensing tradeoffs are acceptable. The
winner is the first mode that keeps acknowledged evidence replayable under
faults without making Parallax harder to run than the systems it is trying to
replace.
