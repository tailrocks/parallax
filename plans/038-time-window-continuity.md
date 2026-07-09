# Plan 038 Remaining: Time-Window Continuity

## Audit Verdict

Core URL-state implementation is landed. A regression test now pins preset
links clearing stale absolute bounds. Remaining work is route-level evidence.

## Remaining Work

- [ ] Exercise representative drilldowns from Overview, Issues, Traces, Runs,
  Services, Logs, and Dashboards with preset and absolute ranges.
- [ ] Record proof that navigation keeps the intended range and does not leave
  stale `from`/`to` query params when a preset is chosen.

## Remove When

- Route-level range propagation proof is recorded.
