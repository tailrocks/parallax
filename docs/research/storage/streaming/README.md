# Storage — Streaming / Ingest Log (evidence)

Evidence for the append-only ingest log and optional stream layer. The stack-level decision is
rolled up in [../../decisions/stack-decision.md](../../decisions/stack-decision.md).

- [messaging-and-ingestion-layer.md](messaging-and-ingestion-layer.md) — historical stream/ingest-layer candidate evaluation; broker work is dormant until the plans index trigger opens it.
- [ingest-log-replay-and-backpressure-gate.md](ingest-log-replay-and-backpressure-gate.md) — dormant experiment protocol for local-spool versus Iggy/NATS/Redpanda replay and fault behavior; not an implementation queue.
