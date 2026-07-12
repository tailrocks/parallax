# Implement the Active Plans Program

## Objective

Execute every actionable plan indexed by `plans/README.md`, honor genuine
blockers, and prove closure at one pushed commit with only unfinished work left
in `plans/`. A phase or green subset is not completion.

## Sources And Scope

Each pass, read `AGENTS.md`, `COMMITS.md`,
`plans/README.md`, `plans/IMPLEMENTATION.md`,
`plans/ENGINEERING-STANDARDS.md`, `plans/OXC-IMPLEMENTATION.md`, the selected
plan/dependencies, and live source/tests/CI/tool config. Recheck version-sensitive
details in current official docs. Use live evidence to correct stale mechanics;
never silently change operator decisions, architecture, scope, or gates.

Use the single active branch; never create a branch or PR. Follow the
dependency graph and parallelize only write-disjoint ready plans. Do not
implement a `BLOCKED` plan before its exact trigger is proven.

For each ready plan, inspect current behavior, mark it `IN PROGRESS`, implement
its full scope and required evidence, then run all gates. Commit and push focused
green slices with a separate `Signed-off-by` and exactly one `Co-authored-by`.
The completion commit must include final evidence and delete the completed,
rejected, or superseded plan plus its index row. Re-read the index and continue.
Put newly discovered actionable work in a numbered plan before implementation.
After each push, report plan/step, checks, SHA, blocker, and next ready work,
then continue.

A plan STOP condition blocks only that plan: preserve reproducible evidence,
reduce the file to exact unfinished work, mark it `BLOCKED`, push, and continue
independent work. A genuine blocker remains minimal and indexed. Run Plan 107
only after every other actionable plan is terminal.

## Verification

For every plan, run its `Test Plan`, machine-checkable `Done Criteria`, and
removal gate with the stated result. Use live manifests and configuration as
command truth. Do not run a planned command before its owner creates and proves
it; after that owner lands, a missing, hollow, or failing command
prevents retirement. Never stub a gate or accept an empty test selection.

At closure, run the `plans/README.md#shared-verification` command block from a
clean checkout. Every command exits zero; tests pass; format, lint, and type
checks emit no warnings; repeatability checks agree; and required CI is green
on the same pushed commit. Run every required real
GreptimeDB/Turso, browser, release, tamper, performance, and artifact gate; do
not replace a named live gate with a mock. Complete Plan 107's two independent
clean-checkout audits, exact-tree attestations, and `closure-final` validation
without weakening a gate.

## Done

Completion is valid only when every condition in
`plans/IMPLEMENTATION.md#completion-state` is mechanically true at the same
pushed commit: all actionable plans passed and were removed with their index
rows; each remaining blocked plan has freshly reproduced trigger evidence and
no hidden actionable work; Plan 107's auditors agree; `closure-final` passes;
completed temporary standards/run-contract files are retired; repository
search finds no active plan material outside `plans/`; the worktree is clean;
and local `main` equals `origin/main`.

Never infer success from prose, checkboxes, file existence, grep alone, one
platform, or a subset of tests/plans. Report commands, commit SHA, CI result,
attestations, blocked triggers, and searches that prove completion.

## STOP

Stop globally only when Done is proven, the operator explicitly stops or
replaces the goal, or no honest progress remains because an operator/external
decision is required. In the last case, report exact evidence and the required
decision as blocked, not complete. Stop and escalate rather than invent
requirements, cross unopened scope, perform unauthorized destructive history
changes, substitute forbidden technology, weaken a check, suppress a real
failure, or claim an unrun gate passed.
