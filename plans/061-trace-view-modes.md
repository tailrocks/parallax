# Plan 061 Remaining: Trace View Modes

## Audit Verdict

Implementation is mostly landed. Minimap now uses visible rows, not all rows,
and the errors-mode test covers that. Remaining item is route/deep-link/skew
evidence.

## Remaining Work

- [ ] Verify errors-only, service-lanes, and minimap modes through routed UI.
- [ ] Verify deep links preserve selected mode and range.
- [ ] Record proof for skew banner behavior with skewed trace data.

## Remove When

- Route-level mode/deep-link/skew evidence is recorded.
