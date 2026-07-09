# Plan 040 Remaining: UI Performance At Scale

## Audit Verdict

Implementation is mostly landed. `LogsTable` now avoids virtualizer overhead
for 100 rows or fewer and tests cover the 100/101 row boundary. Remaining
work is seeded/manual performance evidence.

## Remaining Work

- [ ] Run seeded large-log and large-waterfall UI checks.
- [ ] Record proof that logs stay stable at and above the virtualization
  threshold and that trace waterfall/minimap behavior remains usable at scale.
## Remove When

- Seeded performance evidence is recorded.
