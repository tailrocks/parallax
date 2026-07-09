# Plan 056 Remaining: Typed Events And Structured Logs

## Audit Verdict

Implementation and docs use native `opentelemetry_logs`; typed names are stored
in `log_attributes['event.name']` and exposed by Parallax as `event_name`. No
custom fallback table remains. Remaining item is live A29 evidence.

## Remaining Work

- [ ] Run A29 typed-events scenario.
- [ ] Query native `opentelemetry_logs` for
  `json_get_string(log_attributes, 'event.name')`, body, and log attributes.
- [ ] Record proof that Java/web/Rust events appear with the expected names.

## Remove When

- Live native-log event evidence is recorded.
