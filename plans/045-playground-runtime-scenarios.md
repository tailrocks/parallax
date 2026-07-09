# Plan 045 Remaining: Playground Runtime Scenarios

## Audit Verdict

Implementation is mostly landed. B20 no longer auto-passes `--yes`, and
stale exemplar docs were corrected. Remaining item is live scenario evidence.

## Remaining Work

- [ ] Run runtime scenarios that cover Tokio gauges, JVM pressure, container
  limits, and exemplar configuration.
- [ ] Record native GreptimeDB metric/log/trace evidence for each scenario.
- [ ] Confirm dangerous scenarios still require explicit operator opt-in.

## Remove When

- Live runtime-scenario evidence is recorded.
