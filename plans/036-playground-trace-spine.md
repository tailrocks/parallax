# Plan 036 Remaining: Playground Trace Spine

## Audit Verdict

Implementation is mostly landed. Keep this item only for missing live proof
and the dependency note around `open-feature-flagd`/`rustls`.

## Remaining Work

- [ ] Run the playground trace-spine smoke against current compose and record
  proof that HTTP/gRPC parent-child continuity, ERROR span status, baggage,
  CORS, and resource identity are visible through native GreptimeDB trace
  tables.
- [ ] Resolve the `rustls` dependency note: either upgrade/remove it if a
  newer `open-feature-flagd` releases without hard tonic TLS, or document that
  `0.2.1` still requires it.

## Remove When

- Live trace-spine evidence is recorded locally or in the PR body.
- Dependency note is resolved with command output.
