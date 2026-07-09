# Plan 051 Remaining: traceCriticalPath And traceCompare

## Audit Verdict

Implementation and regression tests are landed. Trace compare now includes
parent structural keys so sibling operations under different parents do not
collide. Remaining item is manual proof on real traces.

## Remaining Work

- [ ] Capture or seed two comparable real traces.
- [ ] Verify critical path and compare UI with the actual trace IDs.
- [ ] Record proof that distinct same-name child spans remain distinct.

## Remove When

- Real-trace compare evidence is recorded.
