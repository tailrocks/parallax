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

| Gate | Result |
| --- | --- |
| `cargo nextest run --workspace --all-features --no-fail-fast` | PASS — 251/251, 6 skipped |
| `cargo nextest run -p parallax-server --all-features --run-ignored all --no-capture --no-fail-fast` | PASS — 36/36, including managed GreptimeDB restart/conformance |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | PASS |
| `cargo check --release --workspace --all-features` | PASS |
| `cargo xtask dependencies --all` | PASS — audit, deny, shear, feature powerset, TLS trees, Bun audit/trust |
| `cargo xtask policy` / `cargo xtask facade check` | PASS |
| GraphQL SDL snapshot and model serde contract | PASS |

The initial workspace closure run exposed one stale integration fixture using a
non-hex trace placeholder and one reused occurrence identity. Commit `f13a373`
changed only those fixture identities; the focused test and complete rerun pass.
