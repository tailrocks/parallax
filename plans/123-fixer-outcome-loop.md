# Plan 123: Fixer and outcome-feedback loop residual

> **Executor instructions**: Fixer is separate from Parallax core. Offline
> evaluation first. Never count opened PR as success; no auto-merge/deploy;
> no production mutation.

## Status

- **Priority**: P3
- **Effort**: XL
- **Risk**: HIGH
- **Depends on**: 104, 111 (done); 120, 121 residual gates
- **Category**: future autonomous fixing / outcome evidence
- **Status**: BLOCKED
- **Blocker**: Plans 120/121 residual capture adapters incomplete; offline
  evaluation harness and operator fixer/provider selection not yet landed.
  Scope opened by unblock directive only for after those gates.

## Residual only (after 120/121)

1. Operator-approved fixer/provider, task classes, autonomy ≤L3 draft-PR,
   budgets, success rule.
2. Versioned request/outcome state machine + offline multi-arm harness.
3. Append-only Turso outcomes with immutable hashes; no success without
   review/runtime recurrence evidence.
4. Optional least-privilege draft-PR adapter only after offline gates pass.
5. Read-only feedback surface; no automatic policy learning in this plan.

## Done Criteria

- [ ] Parallax core has no checkout/patch/PR/merge/deploy ownership.
- [ ] Offline measured gates pass before any draft-PR adapter.
- [ ] Failure/unmerged/revert/recurrence preserved; no optimistic success.

## STOP / Remove When

STOP if A1/A2/A3/redaction prerequisites stale or autonomy would exceed L3.
Delete when offline/L3 scope ships or operator rejects fixer direction.
