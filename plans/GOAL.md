# Close The Remaining Leftover Plans

## Authorization And Objective

Retired program waves are not current work. Their evidence lives under
`docs/research/validation/`. Do not execute retired numbered plans.

The leftover unfinished work is only:

- Plan 089 — BLOCKED on the upstream `greptimedb-ingester` native-TLS /
  plaintext feature. Recheck the crates.io / GitHub / PR #58 trigger; leave
  the plan BLOCKED if it still fails. Do not enable rustls, fork the crate,
  or weaken native-TLS policy.
- Plan 114 — BLOCKED until a qualifying stable raw-frame release cycle and
  expired legacy segments exist. Recheck published tags; leave BLOCKED while
  the only tag is rolling `preview`.
- Plan 107 — IN PROGRESS, last. Independent source audits and the mechanical
  program-close commit. Do not retire this plan, do not impersonate its
  closure commit, and do not delete still-binding contracts
  (`IMPLEMENTATION.md`, `ENGINEERING-STANDARDS.md`, `OXC-IMPLEMENTATION.md`,
  this file) until 107's mechanical allowlist says so.

Primary objective: honor the two external blockers, keep their residuals and
done criteria intact, and continue plan 107 only when its C0 freeze criteria
are honestly met. Do not invent actionable product work from retired waves.

## Sources And Scope

Before editing and before each leftover step, read `AGENTS.md`,
`BRANCHING.md`, `COMMITS.md`, `PROJECT_STRUCTURE.md`, `plans/README.md`, this
file, `plans/IMPLEMENTATION.md`, `plans/ENGINEERING-STANDARDS.md`, the
selected leftover plan, and the live source those leftovers name. Recheck
version-sensitive details in current official documentation. Use live
evidence to correct stale mechanics; never silently change operator
decisions, architecture, scope, or gates.

Preserve all non-negotiable repository constraints: GreptimeDB plus Turso
only, GreptimeDB native raw-signal tables, native TLS only, Bun only for
JavaScript and TypeScript, decode once and move ownership on the ingest hot
path, Apache-2.0, and progress narration for long-running commands.

## Continuous Execution

For each leftover: run its drift / trigger check, keep 089 and 114 `BLOCKED`
while their exact external conditions still hold, and only mark 107
`IN PROGRESS` toward C0 when every other leftover is a minimal freshly
rechecked BLOCKED file. Implement complete leftover scope when a trigger
clears. Run every stated verification and done criterion, then retire the
plan per the lifecycle (delete file + index row in the same commit,
preserving durable evidence). Commit with Conventional Commits, DCO
sign-off, and exactly one agent-product trailer, and push each durable
update to `main`. A STOP condition blocks only that plan: preserve
reproducible evidence, shrink the file to unfinished work, mark `BLOCKED`,
continue independent ready leftover work. Never stub a gate, never claim an
unrun check passed, never invent an operator decision.

## Verification And Done

There are no leftover-implementation pull requests: `main` is the
integration line. Compensating discipline is therefore mandatory — never
push a slice whose targeted checks are red, and run each leftover's full
gate set before retiring it.

The overall program's completion state remains
`plans/IMPLEMENTATION.md#completion-state`. Plan 107 runs last and deletes
this file in the final mechanical closure commit.

## STOP

Stop globally only when the leftovers are proven complete, the operator
replaces this goal, or no honest progress remains without an
operator/external decision. In that case finish and push every independent
ready leftover, report exact reproducible evidence and the required
decision, and do not claim completion.
