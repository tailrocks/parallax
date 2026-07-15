# Complete The Active Plans Program In One PR

## Authorization And Objective

The operator explicitly authorizes exactly one implementation branch,
`codex/active-plan-closure-7f3c`, and exactly one pull request from that branch
to `main`. Create and use only that branch for this program. Do not create any
other branch, worktree branch, remote branch, or pull request.

Execute every actionable plan indexed by `plans/README.md`, honor genuine
blockers, and prove the complete program at one validated pull-request head,
then merge that one PR and verify the merged `main` commit. A phase, a green
subset, a partial implementation, a commit, a push, or a status report is not
completion. Continue automatically until the Done condition is mechanically
proven or an exact operator/external decision prevents all honest progress.

## Sources And Scope

Before editing and before each new plan, read `AGENTS.md`, `BRANCHING.md`,
`COMMITS.md`, `PROJECT_STRUCTURE.md`, `plans/README.md`, this file,
`plans/IMPLEMENTATION.md`, `plans/ENGINEERING-STANDARDS.md`,
`plans/OXC-IMPLEMENTATION.md`, the selected plan and its dependencies, and
the relevant source, tests, CI, manifests, and configuration. Read
`ui/AGENTS.md` before UI work. Recheck version-sensitive library, framework,
SDK, API, CLI, and cloud-service details in current official documentation via
Context7. Use live evidence to correct stale mechanics; never silently change
operator decisions, architecture, scope, or gates.

Follow the dependency graph in `plans/README.md`. Parallelize only
write-disjoint ready plans; otherwise serialize. Do not implement a `BLOCKED`
plan before its exact trigger is proven. Preserve all non-negotiable repository
constraints: GreptimeDB plus Turso only, GreptimeDB native raw-signal tables,
native TLS only, Bun only for JavaScript and TypeScript, decode once and move
ownership on the ingest hot path, Apache-2.0, and the UI rules.

## Continuous Execution

For each ready plan, inspect current behavior, mark it `IN PROGRESS`, implement
its complete scope and required durable evidence, and run every stated test,
machine-checkable Done Criterion, and removal gate. Use live manifests and
configuration as command truth. Do not run a planned command before its owner
creates and proves it; after that owner lands, a missing, hollow, or failing
command prevents retirement. Never stub a gate, accept an empty test selection,
replace a named live gate with a mock, or claim an unrun check passed.

When a plan passes, preserve required durable evidence, delete its file and its
index row in the same commit, re-read the index, and continue immediately with
the next ready plan. Put newly discovered actionable work in a numbered plan
before implementing it. Commit and push green durable slices to
`codex/active-plan-closure-7f3c` with Conventional Commits, a separate
`Signed-off-by` trailer, and exactly one trailer:

```text
Co-authored-by: Codex <codex@openai.com>
```

After each push, report the plan/step, checks, SHA, blocker, and next ready
work, then continue without waiting for confirmation. Do not pause for routine
implementation decisions; use the fixed decisions in the plans and current
official documentation.

A STOP condition blocks only that plan. For a genuine upstream, operator,
product-scope, cross-repository, release-cycle, or external blocker, freshly
verify its exact trigger, preserve reproducible evidence, reduce the file to
only unfinished work, mark it `BLOCKED`, commit and push it, and continue every
independent ready plan. Do not invent an operator decision, authorize a
destructive history rewrite, open V2 scope, choose a provider or credential,
or bypass repository policy. Run Plan 107 only after every other actionable
plan is completed or freshly proven blocked.

## Verification And Integration

At closure, run the complete `plans/README.md#shared-verification` command
block from a clean checkout at the PR head. Every command must exit zero; tests
must pass; format, lint, and type checks must emit no warnings; repeatability
checks must agree; and required CI must be green for that exact pushed commit.
Run every required real GreptimeDB/Turso, browser, release, tamper,
performance, and artifact gate. Complete Plan 107's two independent
clean-checkout audits, exact-tree attestations, and `closure-final` validation
without weakening a gate.

Before opening the PR, verify that every remaining blocked plan has fresh exact
trigger evidence and no hidden actionable work. Delete all completed, rejected,
or superseded plan files and index rows. Delete `plans/GOAL.md`,
`plans/IMPLEMENTATION.md`, `plans/ENGINEERING-STANDARDS.md`, and
`plans/OXC-IMPLEMENTATION.md` only when their documented completion conditions
are mechanically satisfied. Verify repository search proves all remaining
active plan material exists only in `plans/`.

Open the sole PR from `codex/active-plan-closure-7f3c` to `main` only after the
PR head passes all required validation. Merge it when required CI is green and
the operator/repository merge policy permits it. Then verify the merged commit
on `main` from a clean checkout and report the PR URL, merged SHA, completed
plans, exact remaining blockers, and all verification results.

## Done

Completion is valid only when every condition in
`plans/IMPLEMENTATION.md#completion-state` is mechanically true at the same
validated PR head and, after merge, on the resulting `main` commit: all
actionable plans passed and were removed with their index rows; each remaining
blocked plan has freshly reproduced trigger evidence and no hidden actionable
work; Plan 107's auditors agree; `closure-final` passes; completed temporary
standards/run-contract files are retired; repository search finds no active
plan material outside `plans/`; and the post-merge worktree is clean with local
`main` equal to `origin/main`.

Never infer success from prose, checkboxes, file existence, grep alone, one
platform, or a subset of tests or plans. Report the commands, commit SHAs, CI
result, attestations, blocked triggers, and searches that prove completion.

## STOP

Stop globally only when Done is proven, the operator explicitly stops or
replaces the goal, or no honest progress remains because an operator/external
decision is required. In the last case, finish and push every independent
ready plan, report exact reproducible evidence and the required decision as
blocked, and do not claim completion. Stop and escalate rather than invent
requirements, cross unopened scope, perform unauthorized destructive history
changes, substitute forbidden technology, weaken a check, suppress a real
failure, or claim an unrun gate passed.
