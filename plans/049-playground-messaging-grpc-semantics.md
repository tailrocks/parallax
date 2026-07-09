# Plan 049 Remaining: Playground Messaging And gRPC Semantics

## Audit Verdict

Implementation is mostly landed. Batch drain behavior has partial unit coverage,
but poison handling during drain is not fully proven. Remaining work is targeted
test coverage plus live scenario evidence.

## Remaining Work

- [ ] Add or record proof that poison handling still works during batch drain,
  including the intended behavior for batch-flagged poison messages.
- [ ] Run batch fan-in, lag, orphan, deadline, stream, and poison/dead-letter
  scenarios.
- [ ] Record native GreptimeDB trace/log/metric evidence for each behavior.
- [ ] Confirm batch links and failed-message handling are visible in Parallax.

## Remove When

- Live messaging and gRPC evidence is recorded.
