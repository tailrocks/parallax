# Ship Unified CLI Observability Directly On Main, Then Close The Remaining Program

## Authorization And Objective

The previous authorization (branch `codex/active-plan-closure-7f3c`, sole PR
#20) completed and merged on 2026-07-15; that program is closed.

The operator (2026-07-17, superseding the same-day branch authorization)
directs: **the entire program is implemented directly on `main`** in BOTH
repositories — `tailrocks/parallax` and
`tailrocks/parallax-telemetry-playground`. Do not create any branch,
worktree branch, remote branch, or pull request, in either repository, for
any part of this program. Every durable green slice is committed to `main`
(DCO sign-off, exactly one agent trailer, Conventional Commits) and pushed
immediately. The parallax `main` ruleset is bypassed by the operator's
admin rights — the "Bypassed rule violations" notice on push is expected,
not an error. Plan/index/document maintenance commits land on `main` the
same way.

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
GraphQL assertions and browser evidence (plan 159) before Wave 1 is
declared complete.

Secondary objective: after plan 159's evidence completes Wave 1, execute
Wave 2 (plans 162-168, the Maple-informed UI evolution — see its section in
`plans/README.md`) the same way — directly on `main` in both repositories —
then continue every remaining actionable plan under the same rules,
honoring genuine blockers.

Before the first plan, read `plans/README.md#execution-preflight-verified-live-2026-07-17`
— it records the verified host facts (Docker, toolchains, push rights,
live-engine test invocation shape, browser-tooling requirement) and the
only operator-gated leftovers. Do not re-derive or second-guess those
facts unless a command contradicts them.

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

There are no pull requests: `main` is the integration line in both
repositories. Compensating discipline is therefore mandatory — never push a
slice whose targeted checks are red, and run each plan's full gate set
before retiring it. Wave 1 is complete only when plan 159's evidence bundle
exists and the shared verification baseline in
`plans/README.md#shared-verification` passes for the commands that exist at
that head (UI browser lanes that plans 132/144-146 have not yet created are
not yet required), verified from a clean checkout of `main`; report the
verified SHA, completed plans, and remaining blockers. Wave 2 closes the
same way at its final `main` SHA.

The overall program's completion state remains
`plans/IMPLEMENTATION.md#completion-state`. Plan 107 runs last and deletes
this file in the final mechanical closure commit.

## STOP

Stop globally only when the objectives are proven, the operator replaces this
goal, or no honest progress remains without an operator/external decision. In
that case finish and push every independent ready plan, report exact
reproducible evidence and the required decision, and do not claim completion.
