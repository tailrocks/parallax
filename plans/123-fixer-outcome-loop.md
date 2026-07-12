# Plan 123: Prove the separate fixer and outcome-feedback loop

> **Executor instructions**: Parallax remains the evidence/context engine; the
> fixer is a separate component. Begin with an offline evaluation harness and
> append-only measured outcomes. Never count an opened PR as success, auto-merge
> or deploy, mutate production, or let provider/agent output redefine policy.

## Status

- **Priority**: P3
- **Effort**: XL
- **Risk**: HIGH
- **Depends on**: 104, 111, 120, 121
- **Category**: future autonomous fixing / outcome evidence / safety
- **Planned at**: `eefa4617`, 2026-07-12
- **Status**: BLOCKED
- **Blocker**: Autonomous fixing is not open product scope. The operator has not
  selected a fixer/provider, and the A1/A2/A3/redaction evidence gates required
  before an outcome-loop implementation have not all passed.

## Why

The fixer-boundary decision defines a strong separation and a versioned outcome
model, but its historical implementation order remained outside `plans/`. PR
creation is commodity; the useful Parallax asset would be a reproducible chain
from evidence bundle through agent session, patch, validation, review,
merge/revert, and runtime recurrence. That claim must be earned offline before
any draft-PR integration or autonomy level advances.

## Scope

In scope after the blocker clears:

- A separate fixer adapter process/component consuming canonical redacted bundles.
- Versioned request, budget, repository/permission, provider, session, patch,
  validation, review, merge/revert, recurrence, and evidence-citation records.
- An offline multi-arm evaluation harness at L0-L2 before optional L3 draft PRs.
- Explicit task-class/evidence/patch-size/file-count/time/cost safety budgets.
- Append-only outcomes in Turso with immutable artifact hashes and safe refs.
- One least-privilege repository provider adapter only after offline gates pass.
- Feedback queries that measure outcome/citation/recurrence without automatically
  changing production policy or evidence selection.

Out of scope:

- Putting checkout/edit/test/PR logic in Parallax core.
- Auto-merge, deployment, rollback, production mutation, generic write MCP tools,
  or L4/L5 autonomy.
- Treating provider task completion, a patch, green local tests, or an opened PR
  as a successful fix without review/runtime outcomes.
- Hidden prompts, raw sensitive bundles, unrestricted repository/network access.

## Steps

1. Reproduce every gate and record the operator-approved fixer/provider, task
   classes, autonomy ceiling, repository permissions, budgets, and success rule.
2. Finalize the versioned request/outcome schema and state machine. Model denied,
   abandoned, failed, unmerged, reverted, and recurred outcomes as first-class.
3. Build an offline harness using frozen canonical bundles/tasks and approved
   agent-session adapters. Run no-provider, diagnosis-only, and patch arms with
   deterministic artifact/session hashes and equal evidence budgets.
4. Persist append-only outcome records and validation evidence through typed
   APIs. Keep raw repository/agent artifacts outside default bundles and apply
   plan 111 redaction/source-field policy before any projection.
5. Measure fix quality, regression rate, cost/time, evidence citation, human
   review, and recurrence. Do not advance autonomy unless predeclared gates hold.
6. If earned, add one least-privilege draft-PR provider adapter with protected
   branch/review/CI behavior. It may propose only; humans retain merge authority.
7. Add a read-only feedback analysis surface. Any automated policy-learning step
   requires a separate future plan and operator decision.

## Test Plan

- State-machine/property tests for every terminal/interrupted/retry outcome.
- Frozen task/bundle evaluation fixtures with deterministic hashes and budgets.
- Seeded malicious repository, issue, telemetry, and agent output proving policy
  separation, redaction, permission, and network/tool denial.
- Provider sandbox fixtures for auth, branch protection, CI approval, review,
  close/reopen, merge, revert, and recurrence linkage.
- Replay/idempotency tests across crash and duplicate provider notifications.
- Outcome-query tests that cannot upgrade incomplete evidence to fix success.

## Done Criteria

- [ ] Operator/gate/provider/autonomy/budget decisions are current and explicit.
- [ ] Parallax core has no checkout, patch, branch, PR, merge, or deploy ownership.
- [ ] Every run has canonical input, session, patch, validation, and outcome hashes.
- [ ] Failure/unmerged/revert/recurrence states are preserved without optimistic collapse.
- [ ] Offline measured gates pass before any draft-PR adapter is enabled.
- [ ] Provider access is least privilege and cannot merge/deploy/mutate production.
- [ ] No success or moat claim exceeds dated review/runtime outcome evidence.

## STOP Conditions

- Autonomous-fixer scope or the first provider is not explicitly opened.
- A1/A2/A3/redaction prerequisites are stale, failed, or unmeasured.
- Implementation would place fixer behavior in Parallax core, expose raw sensitive
  evidence, weaken branch/review rules, or exceed L3 draft-PR authority.
- Success cannot be linked through review and recurrence with stable evidence.

## Remove When

Delete this plan and row when the approved offline/L3 scope is shipped with
measured outcome evidence, or when the operator rejects the fixer direction and
no actionable outcome-loop work remains.
