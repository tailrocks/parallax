# Plan 099 boundary, idempotency, and agent-safety evidence

Validation date: 2026-07-13

## Typed and sanitized boundaries

- Storage and metadata ports expose classified `thiserror` results with source
  chains; configuration and server startup expose typed public errors.
- GraphQL maps failures centrally to stable `INVALID_INPUT`, `NOT_FOUND`,
  `CONFLICT`, `UNAVAILABLE`, `TIMEOUT`, and `INTERNAL` codes. Negative tests
  prove SQL, paths, URLs, and private identifiers do not enter client messages.
- Residual library `anyhow` is limited to private adapter composition,
  top-level lifecycle composition, and erased sources inside typed errors.
  Every approved file has an exact ceiling and non-empty reason in
  `product.anyhow_edges`; an unlisted occurrence or changed count fails policy.

## Effects and issue identity

- The ingest retry oracle injects failure after registration, broadcast,
  telemetry persistence, and issue recording. A monotonic per-item checkpoint
  proves every completed effect executes exactly once after retry.
- Structured span exception/status and log exception/severity views select
  `error.type`, then `exception.type`, then a channel fallback. A golden test
  proves one fingerprint across all four views while source evidence remains
  distinct and a different top frame remains separate.
- Turso owns the durable occurrence ledger and atomically claims identity before
  updating issue counters, buckets, and tags. Reopen, eight-way concurrent
  delivery, distinct-identity, and 30-day pruning checks pass on real local
  Turso. GreptimeDB retains derived rows and owns no mutable dedup state.

## Typed ID and agent trust

- The boundary-first `TraceId` pilot validates GraphQL, CLI, and OTLP inputs
  while preserving transparent lowercase text on wire and disk. Detailed
  frontier evidence is in
  [the TraceId pilot note](2026-07-13-plan-099-trace-id-pilot.md).
- Product agent/MCP packages must declare `agent_context = true` and may depend
  only on `parallax-evidence` and `parallax-model`. A negative architecture
  fixture rejects a raw product dependency. The current MCP spike remains a
  non-shipping proof.

## Gates

Focused crate tests, strict all-target/all-feature Clippy, structural/product
policy, facade drift, model serde, GraphQL SDL, and real Turso gates pass. Full
workspace and real GreptimeDB results are recorded when the plan closes.
