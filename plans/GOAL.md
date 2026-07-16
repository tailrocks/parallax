# Ship Unified CLI Observability In One PR, Then Close The Remaining Program

## Authorization And Objective

The previous authorization (branch `codex/active-plan-closure-7f3c`, sole PR
#20) completed and merged on 2026-07-15; that program is closed.

The operator (2026-07-17) authorizes exactly one new implementation branch,
`feature/unified-cli-observability`, in the Parallax repository, with exactly
one pull request from it to `main` — plus exactly one linked pull request on a
branch of the same name in `tailrocks/parallax-telemetry-playground`. Create
and use only these branches. Do not create any other branch, worktree branch,
remote branch, or pull request. Plan/index/document maintenance commits
continue to land directly on `main` as before.

Primary objective: execute plans 156, 157, 158, 161, 160, and 159 (the
Unified CLI Observability vertical in `plans/README.md`) **completely and
one-shot, without operator questions** — every decision needed is fixed
inside those plans. The vertical removes `parallax.run.id` support entirely
and replaces the runs surface with generic CLI-application observability
(`cli.invocation.id`, `session.id`, `app.mode`, `ui.*` events,
`background.cycle`, jobs, `gen_ai.*`, bounded `outcome`/`error.type` —
generic attributes only, application-specific keys are display-only opaque
data), builds the invocation hub UI with per-page real-time toggles and the
session journey view, extends the playground with the corner-case corpus,
audits and fixes every UI display defect (span rendering inside traces
foremost) with browser verification after every implemented feature, and
proves the whole coverage matrix live on the operator's Docker host with
GraphQL assertions and browser evidence (plan 159) before the PR opens.

Secondary objective: after the vertical merges, execute Wave 2 (plans
162-168, the Maple-informed UI evolution — see its section in
`plans/README.md`) on its own single authorized branch and PR pair, then
continue every remaining actionable plan under the same rules, honoring
genuine blockers.

## Sources And Scope

Before editing and before each new plan, read `AGENTS.md`, `BRANCHING.md`,
`COMMITS.md`, `PROJECT_STRUCTURE.md`, `plans/README.md`, this file,
`plans/IMPLEMENTATION.md`, `plans/ENGINEERING-STANDARDS.md`, the selected
plan and its dependencies, and the relevant source, tests, CI, manifests, and
configuration. Read `ui/AGENTS.md` before UI work. Recheck version-sensitive
details in current official documentation via Context7. Use live evidence to
correct stale mechanics; never silently change operator decisions,
architecture, scope, or gates.

Preserve all non-negotiable repository constraints: GreptimeDB plus Turso
only, GreptimeDB native raw-signal tables, native TLS only, Bun only for
JavaScript and TypeScript, decode once and move ownership on the ingest hot
path, Apache-2.0, progress narration for long-running commands, and the UI
rules. The contract-reconciliation note in the Unified CLI Observability
section of `plans/README.md` binds every executor of plans 105, 140, 141,
142, 147, 154, and 155.

## Continuous Execution

For each ready plan: run its drift check, mark it `IN PROGRESS`, implement
its complete scope, run every stated verification and done criterion, then
retire it per the lifecycle (delete file + index row in the same commit,
preserving durable evidence). Commit green durable slices with Conventional
Commits, DCO sign-off, and exactly one agent-product trailer; push after
every durable commit. A STOP condition blocks only that plan: preserve
reproducible evidence, shrink the file to unfinished work, mark `BLOCKED`,
continue independent ready work. Never stub a gate, never claim an unrun
check passed, never invent an operator decision.

## Verification And Done

The vertical's PR opens only after plan 159's evidence bundle is complete and
the shared verification baseline in `plans/README.md#shared-verification`
passes for the commands that exist at that head (UI browser lanes that plans
132/144-146 have not yet created are not yet required). Merge when required
CI is green, then verify merged `main` from a clean checkout and report PR
URL, merged SHA, completed plans, and remaining blockers.

The overall program's completion state remains
`plans/IMPLEMENTATION.md#completion-state`. Plan 107 runs last and deletes
this file in the final mechanical closure commit.

## STOP

Stop globally only when the objectives are proven, the operator replaces this
goal, or no honest progress remains without an operator/external decision. In
that case finish and push every independent ready plan, report exact
reproducible evidence and the required decision, and do not claim completion.
