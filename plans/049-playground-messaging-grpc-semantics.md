# Plan 049 Remaining: Playground Messaging And gRPC Semantics

## Audit Verdict

Implementation is landed. Batch drain and batch-flagged poison handling have
unit coverage. Remaining work is live scenario evidence.

## Remaining Work

- [ ] Record one evidence artifact for batch fan-in, lag, orphan, deadline,
  stream, and poison/dead-letter scenarios.
- [ ] Include exact commands, date, environment, and repo SHAs.
- [ ] Include native GreptimeDB queries and row output for
  `opentelemetry_traces`, `opentelemetry_logs`, and `messaging.queue.depth`.
- [ ] Include Parallax UI or GraphQL proof for `typedLinks`, `linkedTraces`,
  `traceEvents`, batch links, and failed spans.

## Remove When

- Live messaging and gRPC evidence is recorded.
