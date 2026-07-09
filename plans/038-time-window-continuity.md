# Plan 038 Remaining: Time-Window Continuity

## Audit Verdict

Core URL-state implementation is landed. Shared helpers now make preset ranges
win over stale absolute bounds, and current route code preserves ranges across
the known drilldowns. Remaining work is route-level evidence for custom ranges
on the surfaces that only have helper/source proof.

## Remaining Work

- [ ] Add or record rendered custom/absolute range drilldown proof for Issues,
  Traces, Services, Logs, and Dashboards.
- [ ] Add or record rendered Dashboard link/create proof for preset and custom
  ranges.
- [ ] Record route-level proof that preset navigation has `range=<preset>` and
  no stale `from`/`to` where only helper proof exists.

## Remove When

- Route-level preset and custom range propagation proof is recorded for every
  named surface.
