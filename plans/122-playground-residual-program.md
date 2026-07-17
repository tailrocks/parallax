# Plan 122: Playground residual program

> **Executor instructions**: Companion `parallax-telemetry-playground` only for
> unresolved scenarios needed by current Parallax contracts. No branches/PRs.
> Never replay completed historical phases.

## Status

- **Priority**: P3
- **Effort**: XL residual
- **Risk**: HIGH
- **Depends on**: 105 (open), 111 (DONE), 119 (DONE registry), 151 (open)
- **Category**: cross-repository playground / validation
- **Status**: BLOCKED
- **Blocker**: Plans 105 and 151 still unfinished product contracts. Operator
  cross-repo authorization exists; dependency order still binds.

## Residual only (after 105 + 151)

1. Commit-pinned two-repo disposition table (shipped / obsolete / research /
   actionable) — no completed phase replayed.
2. Only retained telemetry-shape scenarios with exact Greptime/Turso + API/UI
   contracts and fixtures.
3. Deterministic one-command start/progress/ready + failure/redaction behavior.
4. Fan-out lab remains comparative research only (no product fallback backend).
5. Re-audit both repos; leave protocols in research, not engineering queues.

## Done Criteria

- [ ] Historical rows classified; retained scenarios fixture-gated.
- [ ] Progress/reset/failure/redaction deterministic; no comparator product mode.
- [ ] Required Rust/Java/Bun + Parallax integration gates pass.

## STOP / Remove When

STOP if companion path/baseline unavailable or scenario has no Parallax
consumer. Delete when residuals ship or operator leaves only research protocols.
