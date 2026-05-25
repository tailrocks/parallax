# Messaging and Ingestion Layer

<!-- markdownlint-disable MD013 -->

Research date: 2026-05-25

## Executive Summary

Parallax should **not** require a message broker in the smallest deployment.
The tiny self-hosted version should start with:

```text
parallax-server
  -> local durable WAL / outbox
  -> storage writer
  -> GreptimeDB or ClickHouse
```

That is the fastest way to beat self-hosted Sentry on operational simplicity.
A stream only becomes mandatory when Parallax needs independent processors,
burst absorption, replay, backfill, or scale-out ingestion.

For the durable and scale-out profiles, **Apache Iggy is the first stream to
prototype** because it is Rust-native, append-only, partitioned, single-binary,
low-latency, and architecturally close to Parallax's goal. But it should be
treated as a high-conviction prototype, not a locked dependency, until Parallax
proves crash durability, fsync behavior, cluster replication, operational
recovery, and mixed ingest/query processor workloads.

Current recommendation:

| Deployment stage | Stream decision |
| --- | --- |
| Tiny single server | No external stream. Use a local WAL/outbox and direct writes. |
| Durable single server | Prototype Apache Iggy as the durable replay log. |
| Scale-out | Prefer Iggy if clustering and durability tests pass; otherwise fall back to Redpanda or NATS JetStream depending on which failure mode matters more. |

The important design move is to define Parallax's internal ingest contract now:
raw accepted telemetry is append-only, idempotent, replayable, and processor
agnostic. The first implementation can be a local WAL. Iggy can replace that WAL
when the product actually needs a broker.

## What The Stream Layer Must Do

The stream is not a generic "Kafka because observability" component. It earns
its place only if it gives Parallax concrete capabilities:

| Requirement | Why it matters for Parallax |
| --- | --- |
| Durable raw ingest | Do not lose error events while storage is down or processors restart. |
| Replay | Re-run normalizers, grouping, symbolication, and correlation after bugs or schema changes. |
| Processor fan-out | Let storage, grouping, context indexing, and alerting consume independently. |
| Backpressure | Protect storage and processors from telemetry spikes without losing accepted data. |
| Partitioned ordering | Keep per-project, per-issue, or per-trace ordering where correlation needs it. |
| Idempotent writes | Retried SDK/OTLP/Sentry-envelope delivery must not duplicate downstream state. |
| Short raw retention | Keep raw envelopes and OTLP batches long enough for parser fixes and reprocessing. |
| Simple operations | The broker cannot recreate Sentry's Kafka-shaped burden. |

The minimum internal topics/streams look like:

```text
sentry.raw_envelopes
otel.raw_traces
otel.raw_logs
otel.raw_metrics
parallax.raw_cli_invocations
parallax.raw_agent_sessions
parallax.normalized_errors
parallax.group_updates
parallax.context_jobs
parallax.dead_letters
```

Partition keys should be chosen by the correlation unit:

| Signal | Recommended key |
| --- | --- |
| Sentry-style error event | `project_id + fingerprint_candidate` |
| OTLP trace span | `project_id + trace_id` |
| OTLP log | `project_id + trace_id` when present, otherwise `project_id + service_name` |
| Metric sample | `project_id + metric_name + service_name` |
| CI/test event | `repository + run_id` or `repository + test_signature` |
| CLI invocation | `repository + command + invocation_id` |
| Coding-agent session | `repository + agent_session_id` |

This keeps related data close enough for deterministic processors while still
allowing horizontal scaling.

## Do We Need Kafka-Scale Complexity?

Not at first.

For the initial Rust/Sentry-compatible observability system, the operational
truth is:

1. The user is likely a small team trying to escape self-hosted Sentry
   complexity.
2. Most early value comes from accepting errors, grouping them, and fetching
   nearby context.
3. If storage is local and processors are in-process, an external stream adds
   another service, disk, retention policy, auth surface, and backup story.

The smallest credible design is:

```text
HTTP ingest endpoint
  -> validate auth / size / redaction
  -> append accepted payload to local WAL
  -> enqueue in-process job
  -> write normalized data to storage
  -> mark WAL offset processed
```

That design still preserves the architectural seam. Once ingestion volume,
replay, or processor isolation matters, the WAL becomes:

```text
HTTP ingest endpoint
  -> append accepted payload to Iggy
  -> independent processors consume by group
  -> storage writer / grouping / context indexer
```

This is the startup-first, big-company-later trajectory from the prompt: do not
force distributed infrastructure early, but do not design a write path that has
to be rewritten later.

## Candidate Comparison

Version freshness rule: future stream benchmarks must compare the latest
reasonably available stable/public version of each candidate as of the benchmark
date. Older Jepsen reports, vendor benchmarks, or release notes are useful
signals, but they must be labeled historical if newer releases have materially
changed durability, clustering, performance, or licensing.

| Candidate | Runtime | License / source posture | Strong fit | Main concern | Current role |
| --- | --- | --- | --- | --- | --- |
| Apache Iggy | Rust | Apache project, incubating | Best architectural fit: Rust, append-only log, partitions, consumer groups, single binary, low-latency design. | Clustering is still being hardened; public performance evidence is young; defaults may not fit tiny deployments. | First durable-stream prototype. |
| Redpanda | C++ / Seastar | Source-available BSL community; enterprise features | Strong Kafka-compatible operational baseline with Raft, mature partition model, high performance. | Not open-source in the Parallax sense; tiered storage is Enterprise for self-hosted; Kafka compatibility is not a requirement. | Baseline/fallback if Iggy durability or clustering fails. |
| NATS JetStream | Go | Apache-2.0 | Mature, single-binary, simple messaging plus persistence, good edge/cloud deployment model. | Not as naturally Kafka-like for partitioned telemetry replay; durability defaults need careful testing and config. | Fallback when simplicity and OSS matter more than log-centric design. |
| Liftbridge | Go | Apache-2.0 | Kafka-lite semantics on NATS; partitioned replicated streams; simple Go stack. | Smaller ecosystem, old docs, and overlaps with JetStream while adding another layer. | Watch-list only. |
| Kafka / Pulsar | JVM | Open source | Baseline-to-beat for durable stream architecture and ecosystem. | Excluded by the language/runtime filter and operational profile. | Reference only, not deployable candidates. |

## Apache Iggy Evaluation

### Why It Fits

Iggy is unusually close to the stream Parallax would design from scratch:

- Rust-native, no JVM or GC runtime.
- Persistent append-only message streaming.
- Streams contain topics; topics contain partitions; partitions store messages
  in segment files.
- Partitions are the unit of parallelism, offset replay, and horizontal
  consumer scaling.
- Thread-per-core shared-nothing architecture.
- `io_uring` / completion-based I/O path through `compio`.
- Multiple transports: TCP, QUIC, WebSocket, and HTTP.
- Consumer groups for horizontal processing.
- Topic retention, compression, and maximum-size limits.
- Built-in OpenTelemetry logs/traces and Prometheus metrics for the broker
  itself.
- Single-binary operational shape.

Sources:

- [Apache Iggy overview](https://iggy.apache.org/)
- [Apache Iggy architecture](https://iggy.apache.org/docs/introduction/architecture/)
- [Apache Iggy SDK introduction](https://iggy.apache.org/docs/sdk/introduction/)
- [Apache Iggy FAQ](https://iggy.apache.org/docs/faq/faq/)

This lines up with Parallax's ingestion path better than a generic pub/sub
system. Observability ingestion is mostly append-only evidence capture:

```text
accept event
append raw evidence
normalize later
replay if parser/grouping/correlation changes
fan out to processors
retain briefly for recovery
```

Iggy's physical model maps directly to that job.

### Why It May Not Fit Yet

Hard current status (rechecked 2026-05-25): **Iggy runs as a single node today.
It has no production multi-node clustering or replication.** Clustering based on
Viewstamped Replication (VSR) is under active development — the core consensus
protocol, view-change mechanism, deterministic timeouts, and a network simulator
exist — but it is explicitly **not production-ready**, and `v0.8.0` ships only the
*groundwork* for multi-node (TwoHalves buffer, aligned-buffer memory pool,
sans-IO frame codec, VSR type consolidation, persistent WAL journal) that the
server does not yet use for replication. Iggy is also still in the Apache
Incubator (entered 2025-02; latest releases `v0.7.0` 2026-02-24, `v0.8.0`
2026-04-22).

The consequence for Parallax is concrete and changes a prior conclusion: **Tier 3
horizontal durability cannot depend on Iggy yet**, because a single-node stream is
a single point of failure with no replicated partitions or failover. Iggy is a
fine *single-node durable* stream (Tier 2), but the Tier 3 clustered-durable
stream must be NATS JetStream (Go, mature clustering) or Redpanda (C++, Raft) —
or a storage-backed/object-store stream — until Iggy ships and proves VSR
clustering. Keep Iggy behind the `IngestLog` abstraction precisely so this
Tier-3 substitution is a config change, not a rewrite.

Sources:

- [Apache Iggy 0.7.0 release](https://iggy.apache.org/blogs/2026/02/24/release-0.7.0/)
- [Apache Iggy 0.8.0 release](https://iggy.apache.org/blogs/2026/04/22/release-0.8.0/)
- [Iggy clustering status (issue #2562)](https://github.com/apache/iggy/issues/2562)
- [Apache Iggy incubation status](https://incubator.apache.org/projects/iggy.html)

Risks to test before making it a default dependency:

| Risk | Why it matters |
| --- | --- |
| Cluster readiness | Parallax's future scale-out story needs replicated partitions, failover, and recovery that are boring under faults. |
| Durability semantics | Accepted telemetry must survive process crashes, OS crashes, disk pressure, and restart loops according to documented guarantees. |
| Fsync tradeoff | Broker throughput is irrelevant if "acknowledged" does not mean durable enough for debugging evidence. |
| Memory defaults | Iggy's docs describe a default 4 GiB memory pool and a 512 MiB minimum; that can violate the tiny-deployment goal unless tuned. |
| Linux bias | The highest-performance path depends on `io_uring`; laptop and non-Linux dev environments may behave differently. |
| Ecosystem depth | Kafka, Redpanda, and NATS have more operational runbooks and user scars. |

Iggy should therefore be evaluated with adversarial tests, not only vendor
benchmarks:

1. Produce Sentry-envelope-sized and OTLP-batch-sized messages with `fsync`
   settings documented.
2. Kill producers, consumers, and the broker during sustained writes.
3. Restart with partially written segments.
4. Force disk-full behavior.
5. Rebalance consumer groups during active processing.
6. Backfill old offsets while new ingest continues.
7. Measure memory on a tiny VPS profile, not only a benchmark box.

### Iggy Verdict

Iggy is the right first prototype because it is the only candidate that is both
Rust-native and architecturally shaped like Parallax's desired ingest log. If
its durability and clustering pass the Parallax benchmark, it should become the
default durable stream.

But it should not be required in the tiny profile. The first version should keep
Iggy behind an internal `IngestLog` abstraction so Parallax can run with either
`local-wal` or `iggy`.

## Redpanda Evaluation

Redpanda is the strongest non-Rust fallback for a high-throughput durable log.
It is C++/Seastar-based, Kafka API-compatible, uses Raft-based data management,
and appends events to partition log files on disk.

Sources:

- [Redpanda architecture](https://docs.redpanda.com/current/get-started/architecture/)
- [Redpanda topic management](https://docs.redpanda.com/current/develop/manage-topics/config-topics/)
- [Redpanda product overview](https://www.redpanda.com/what-is-redpanda)

Strengths:

- strong Kafka-like data model without the JVM;
- mature partition, consumer group, and replication concepts;
- broad Kafka ecosystem compatibility if users already have Kafka tooling;
- built-in schema registry and HTTP proxy;
- credible performance baseline.

Weaknesses under the Parallax lens:

- Kafka compatibility is not required, so it should not earn extra points.
- Redpanda Community is source-available under BSL, not open source in the
  Parallax sense.
- Self-hosted tiered storage is an Enterprise feature, which directly conflicts
  with Parallax's open-source and cheap-retention bias.
- It is more infrastructure than a tiny Parallax deployment should need.

Source:

- [Redpanda tiered storage docs](https://docs.redpanda.com/24.1/manage/tiered-storage/)
- [Redpanda editions comparison](https://www.redpanda.com/compare-platform-editions)

### Redpanda Verdict

Redpanda is the serious fallback if Iggy cannot prove durability or cluster
readiness. It is not the preferred Parallax default because it is not Rust,
Kafka compatibility is not valuable for this product, and the licensing/tiered
storage posture works against the open-source object-retention goal.

## NATS JetStream Evaluation

NATS JetStream is the strongest "simple operational fallback." It is built into
the NATS server, supports persistent streams, file or memory storage,
at-least-once delivery, message replay, stream retention limits, consumers, and
Raft-based clustering.

Sources:

- [NATS JetStream concepts](https://docs.nats.io/nats-concepts/jetstream)
- [NATS JetStream streams](https://docs.nats.io/nats-concepts/jetstream/streams)
- [NATS JetStream consumers](https://docs.nats.io/nats-concepts/jetstream/consumers)
- [NATS JetStream clustering](https://docs.nats.io/running-a-nats-service/configuration/clustering/jetstream_clustering)
- [NATS server repository](https://github.com/nats-io/nats-server)

Strengths:

- Apache-2.0, Go, single binary, widely deployed.
- Strong pub/sub and request/reply story in addition to persistence.
- Good for edge/cloud hybrid and service-mesh-like messaging.
- Pull consumers can scale processing horizontally.
- Lower conceptual barrier than Kafka-like systems.

Weaknesses for Parallax:

- The data model is subject/stream oriented, not as directly partition-log
  oriented as Kafka/Iggy/Redpanda for telemetry replay.
- Large retained streams require careful storage and indexing choices.
- NATS object store is not a distributed object storage replacement; it does not
  solve cheap long-retention telemetry storage.
- Independent Jepsen testing of NATS 2.12.1 found data-loss risks around file
  corruption, power-failure scenarios, and default fsync behavior. That does
  not disqualify JetStream, but it means Parallax must test exact durability
  settings instead of assuming "acknowledged" means safe enough.

Source:

- [Jepsen analysis of NATS 2.12.1](https://jepsen.io/analyses/nats-2.12.1.pdf)

### NATS Verdict

Use NATS JetStream if Parallax values broad operational maturity and simple OSS
deployment more than a purpose-built append-only telemetry log. It is a better
fallback than Liftbridge because it is the native NATS persistence path, but it
is less architecturally precise than Iggy for Parallax's evidence replay model.

## Liftbridge Evaluation

Liftbridge is a Go "Kafka-lite" message streaming server built on NATS. It
implements durable, replicated, partitioned streams with consumer groups and
aims to bridge the gap between Kafka/Pulsar complexity and simpler cloud-native
messaging.

Sources:

- [Liftbridge overview](https://liftbridge.io/docs/overview.html)
- [Liftbridge consumer groups](https://liftbridge.io/docs/consumer-groups.html)
- [Liftbridge GitHub repository](https://github.com/liftbridge-io/liftbridge)

Strengths:

- Go, no JVM, no ZooKeeper.
- Partitioned streams with leaders/followers.
- Consumer groups with checkpointing and failover.
- Simpler conceptual model than Kafka.

Weaknesses:

- It is not Rust.
- It adds a NATS-based layer while JetStream already exists as NATS's native
  persistence system.
- Documentation still carries old timestamps in several areas.
- Public performance numbers in the README are lower than Iggy's stated target
  class and roughly comparable to JetStream in the same README.

### Liftbridge Verdict

Keep Liftbridge on the watch list only. It is directionally aligned with the
"Kafka-lite, no JVM" instinct, but it is neither Rust-native nor clearly better
than NATS JetStream as the practical fallback.

## Cost and Retention

The stream should not be the long-term telemetry store. GreptimeDB or
ClickHouse should own long retention. The stream owns short raw replay.

Recommended retention:

| Topic class | Retention |
| --- | --- |
| Raw Sentry envelopes | 24-72 hours, longer only for debugging parser rollouts. |
| Raw OTLP batches | 6-24 hours unless users explicitly pay for replay retention. |
| Normalized events | Long retention in storage, not the stream. |
| Group/context jobs | Until processed plus a short retry window. |
| Dead letters | Bounded count and size, with explicit operator visibility. |

This avoids a common observability mistake: retaining huge raw logs in an
expensive broker because it is convenient. The durable stream is a processor
replay buffer, not the evidence archive.

Object storage belongs primarily behind the database/storage layer:

- GreptimeDB object-storage-oriented deployments for retained telemetry.
- ClickHouse object-storage/tiered layouts where self-hosting proves practical.
- Broker tiering only if the broker is also used as a durable historical log,
  which is not the first Parallax requirement.

## Scaling Trajectory

### Profile 1: Tiny Single Server

```text
parallax-server
  - HTTP Sentry envelope endpoint
  - HTTP/gRPC OTLP endpoint
  - local WAL/outbox
  - in-process normalizer/grouping/storage writer
greptimedb standalone or clickhouse
turso metadata
```

Goal: lower operational burden than self-hosted Sentry.

This profile should run on a small VPS. No broker unless benchmarks prove the
local WAL loses data or blocks ingestion too often.

### Profile 2: Durable Single Server

```text
parallax-ingest
apache iggy standalone
parallax-worker
greptimedb standalone
postgres metadata
object storage for database retention/backups
```

Goal: tolerate storage outages, processor restarts, and parser/grouping
replays.

This is the first place Iggy should appear in the default architecture.

### Profile 3: Scale-Out

```text
parallax-ingest x N
clustered durable stream: NATS JetStream or Redpanda
  (NOT an Iggy cluster yet — Iggy has no production clustering as of 2026-05;
   adopt Iggy here only once VSR clustering ships and passes the fault tests)
normalizer workers x N
grouping workers x N
context-index workers x N
greptimedb distributed or clickhouse cluster
postgres metadata
object storage
```

Goal: horizontal scaling without rewriting event contracts. Because the
`IngestLog` abstraction is the same across tiers, swapping the Tier 3 stream
(NATS/Redpanda today, Iggy when its clustering is proven) is a deployment change,
not an event-contract change.

The ingest gateway must remain stateless except for authentication/cache. All
durable state belongs in the stream, observability storage, metadata store, or
object storage.

## Benchmark Plan For The Stream Layer

The storage benchmark already covers GreptimeDB versus ClickHouse. The stream
benchmark should be separate and narrower:

### Dataset

- Sentry envelopes: 2 KB, 20 KB, 200 KB event payloads.
- OTLP trace batches: 100 spans, 1,000 spans, 10,000 spans.
- OTLP log batches: small structured logs and large stack-trace logs.
- Mixed workload: 80% logs, 15% spans, 4% metrics batches, 1% error events.

### Metrics

- producer p50/p95/p99 ack latency;
- ingest throughput by payload class;
- consumer lag under mixed processors;
- replay throughput from old offsets;
- memory and CPU per MB/s;
- disk write amplification;
- restart recovery time;
- data loss or duplication after faults;
- operational setup time.

### Fault Tests

- process kill while producers write;
- OS reboot or VM power cut if available;
- disk full;
- corrupt/truncate one segment;
- consumer group rebalance under load;
- storage writer outage for 10 minutes;
- high-cardinality partition keys;
- broker upgrade/restart during ingest.

### Acceptance Criteria

Iggy becomes the preferred durable stream only if:

1. Single-node durability is clear and configurable.
2. Replay is fast enough to reprocess at least 24 hours of expected startup
   telemetry.
3. Memory can be tuned low enough for a small self-hosted deployment.
4. Consumer groups behave predictably under worker restarts.
5. Cluster mode either works under faults or is not needed for the first public
   release.
6. Operational setup stays simpler than Redpanda and far simpler than Kafka.

## Bottom Line

Parallax should design around an append-only ingestion log, but it should not
prematurely force users to operate one.

- **Now:** implement the interface and local WAL semantics.
- **Prototype:** use Apache Iggy for the durable stream profile.
- **Fallback:** use Redpanda if durability/cluster maturity beats openness and
  Rust purity; use NATS JetStream if OSS simplicity beats log-centric design.
- **Reject:** Kafka and Pulsar as deployable candidates because they violate the
  language/runtime and operational-simplicity constraints.

This keeps the architecture honest: Iggy is the best match for the vision, but
the product wins early by being simpler than Sentry, not by requiring another
piece of infrastructure before users have enough volume to need it.
