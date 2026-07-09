# Plan 045 Remaining: Playground Runtime Scenarios

## Audit Verdict

Implementation is landed. B20 no longer auto-passes `--yes`, and stale
exemplar docs were corrected. Remaining item is live scenario evidence.

## Remaining Work

- [ ] Record one evidence artifact covering A22 Tokio saturation, B19 JVM GC
  pressure, B20 container limit/OOM, and A2 exemplar configuration.
- [ ] Include exact commands, timestamp, environment, and repo SHAs.
- [ ] Include native GreptimeDB SQL queries and row output from metric tables,
  `opentelemetry_traces`, and `opentelemetry_logs` for each scenario.
- [ ] Include trace IDs and B20 Docker OOM/restart evidence.

## Remove When

- Live runtime-scenario evidence is recorded.
