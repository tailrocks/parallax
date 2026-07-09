# Plan 049 Remaining: Playground Messaging And gRPC Semantics

## Audit Verdict

Implementation is mostly landed. Batch drain now preserves poison handling by
reusing single-message consumption. Remaining item is live scenario evidence.

## Remaining Work

- [ ] Run batch fan-in, lag, orphan, deadline, stream, and poison/dead-letter
  scenarios.
- [ ] Record native GreptimeDB trace/log/metric evidence for each behavior.
- [ ] Confirm batch links and failed-message handling are visible in Parallax.

## Remove When

- Live messaging and gRPC evidence is recorded.
