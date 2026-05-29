# Storage — Streaming / Ingest Log (evidence)

Evidence for the append-only ingest log and optional stream layer. The stack-level decision is
rolled up in [../../decisions/stack-decision.md](../../decisions/stack-decision.md).

- [messaging-and-ingestion-layer.md](messaging-and-ingestion-layer.md) — stream/ingest-layer evaluation: Apache Iggy, Redpanda, NATS JetStream, and brokerless startup deployments.
- [ingest-log-replay-and-backpressure-gate.md](ingest-log-replay-and-backpressure-gate.md) — proof gate for the append-only ingest log: local WAL vs Iggy/NATS/Redpanda replay, backpressure, durability modes, fault tests, pass/fail.
