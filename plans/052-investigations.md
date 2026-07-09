# Plan 052 Remaining: Investigations

## Audit Verdict

Implementation and API validation are landed. Rust validation now uses the
typed `InvestigationState` mirror. Remaining item is manual save/restore proof.

## Remaining Work

- [ ] Save an investigation with window, pins, and notes.
- [ ] Restore it and verify URL/UI state round-trips.
- [ ] Record proof that invalid state is rejected safely.

## Remove When

- Manual investigation save/restore proof is recorded.
