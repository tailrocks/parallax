# Plan 056 Remaining: Typed Events And Structured Logs

## Audit Verdict

Implementation and docs were corrected to use native `opentelemetry_logs`
with `event_name`; no custom fallback table remains. Remaining item is live
A29 evidence.

## Remaining Work

- [ ] Run A29 typed-events scenario.
- [ ] Query native `opentelemetry_logs` for `event_name`, body, and log
  attributes.
- [ ] Record proof that Java/web/Rust events appear with the expected names.

## Remove When

- Live native-log event evidence is recorded.
