# Prompts

Reusable research and agent prompts for Parallax. Each file here is a
self-contained brief you can hand to an AI coding agent (Claude Code, Codex, Amp,
OpenCode) so a piece of work runs the same way every time.

## Available prompts

| File | Purpose |
| --- | --- |
| `deep-research-parallax.md` | Deep, critical research brief that validates, re-verifies, and extends the Parallax direction indefinitely; every prior finding is treated as a hypothesis until current evidence supports it. |
| `greptimedb-vs-clickhouse-internals.md` | Never-ending `/goal` or Claude Code `/loop` brief for the under-the-hood GreptimeDB vs ClickHouse comparison: read the source, explain which design decisions make each fast or slow per signal, re-verify every claim against the live Docker stack with production-realistic, no-tricks, reproducible benchmarks, check each system's native out-of-the-box metrics/logs/traces structure (adopt-native vs custom), decide which to build Parallax on, and (when one wins but lacks features) map what the winner must implement to close the gap. Writes to `docs/research/greptimedb-vs-clickhouse/`. |

## Current preferred mode

Run `prompts/deep-research-parallax.md` as an indefinite re-verification loop.
The goal is not one final report. The goal is to keep improving the quality,
trustworthiness, and completeness of the research record.

Each pass should treat existing `docs/research/` findings as theories:

1. Re-read the prompt and current research notes.
2. Pick the weakest, stalest, least-proven, most strategically important, or
   most suspicious claim.
3. Re-check current primary sources and current project docs.
4. Reconsider whether the claim still makes sense for Parallax's actual goal.
5. Update or replace the note, add missing important research, commit, push, and
   continue to the next gap.

Routine deep-research passes should not benchmark storage or infrastructure
performance differences. A separate benchmark-focused agent can own those runs.
This prompt should maintain benchmark standards, scrutinize benchmark artifacts
when they exist, and mark benchmark-dependent claims as unproven until measured.

## How to run a prompt

### 1. As a one-off

Open the agent and hand it the file as the task — paste the contents, attach the
file, or reference the path (for example `@prompts/deep-research-parallax.md` in
Claude Code). Use this for a targeted pass only. For the main Parallax research
program, prefer the indefinite modes below.

### 2. `/goal` — indefinite

Use `/goal` in Codex or Claude Code with an explicit never-finished research
instruction. `/goal` is the cross-tool long-running command; the stop condition
for the ordinary research program is operator intervention:

For Codex specifically, current official Codex CLI and app docs list `/goal` for
persistent long-running goals. They do not list `/run` or `/go` as the documented
slash command for this workflow. If `/goal` is hidden in Codex, enable the
feature first:

```sh
codex features enable goals
```

```text
/goal Follow prompts/deep-research-parallax.md as the active indefinite
research brief. Keep improving the research record until I explicitly stop or
replace the research program.

Treat every existing docs/research finding as a theory, not a settled fact.
Each pass: re-read the prompt and current docs/research state; pick the weakest,
stalest, least-proven, most strategically important, or most suspicious claim;
re-research it from current primary sources; reconsider whether it still serves
Parallax's goal as an AI-native runtime evidence/context engine; update or
replace the focused Markdown note; add important missing research; update durable
prompt/README/PROJECT_STRUCTURE docs when needed; commit and push; then continue
to the next highest-value gap.

Focus on research quality, source trustworthiness, current versions, explicit
uncertainty, and falsification criteria. Do not spend this run benchmarking
storage or infrastructure performance differences unless explicitly asked; use
the separate benchmark agent's artifacts when they exist and mark unmeasured
claims as unproven.

Never declare the research complete on your own. Keep going until the operator
explicitly stops the goal, replaces it, or says the research program is complete.
```

### 3. Claude Code `/loop` — scheduled indefinite

Use Claude Code `/loop` with a fixed interval for the never-stop behavior.
`/loop` is Claude Code-only. The interval is a re-trigger beat, not a freshness
requirement. If the scheduler coalesces while a pass is running, `5m` keeps the
loop close to continuous; if overlapping work or cost becomes a problem, raise it
to `15m` or `30m`.

```text
/loop 5m Follow prompts/deep-research-parallax.md as the active indefinite
research brief.

Treat every existing docs/research finding as a theory, not a settled fact.
Each pass: re-read the prompt and current docs/research state; pick the weakest,
stalest, least-proven, most strategically important, or most suspicious claim;
re-research it from current primary sources; reconsider whether it still serves
Parallax's goal as an AI-native runtime evidence/context engine; update or
replace the focused Markdown note; add important missing research; update durable
prompt/README/PROJECT_STRUCTURE docs when needed; commit and push; then continue
to the next highest-value gap.

Focus on research quality, source trustworthiness, current versions, explicit
uncertainty, and falsification criteria. Do not spend this loop benchmarking
storage or infrastructure performance differences unless explicitly asked; use
the separate benchmark agent's artifacts when they exist and mark unmeasured
claims as unproven.

Never declare the research complete on your own. Keep going until the operator
explicitly stops the loop, replaces it, or says the research program is complete.
```

Notes:

- `/loop` tasks auto-expire after about 7 days and stop if the session closes
  (they resume on `--resume` while unexpired). Relaunch for a longer program.
- Do not wrap `/goal` inside `/loop` for this open-ended research. Pick one
  runner: `/goal` in Codex or Claude Code, or Claude Code `/loop`.

### 4. Bounded `/goal` — fixed deliverables only

Use a bounded `/goal` only when the desired stop condition is explicit, such as
creating the first verdict or refreshing one named research area:

```text
/goal Execute prompts/deep-research-parallax.md. Phase 1 first: deliver a
GO / NO-GO verdict to docs/research/verdict.md (real problem? solves it?
competitors? market sense?). Phase 2, only if GO: complete the implementation
blueprint — API standard (OpenTelemetry vs Sentry: what to support, store, how),
the Parallax-stores / separate-agent-fixes boundary with the CLI-vs-MCP decision,
three scaling tiers (simple / scalable / very scalable), and a named stack per
layer. Research deeply across many passes; commit and push each durable section;
do not stop shallow or early.
```

A vague bounded goal like "research Parallax" lets an agent declare victory
early. For indefinite improvement, use the explicit indefinite `/goal` or Claude
Code `/loop` examples above.

### Running the GreptimeDB vs ClickHouse internals comparison

`greptimedb-vs-clickhouse-internals.md` is an indefinite, never-converging brief,
so run it with `/goal` in Codex or Claude Code, or with Claude Code `/loop`. Use
a clear, explicit instruction so the run keeps going:

```text
/goal Follow prompts/greptimedb-vs-clickhouse-internals.md as the active
never-ending research brief. Keep researching the GreptimeDB vs ClickHouse
internals pass after pass until I explicitly stop or replace this goal. Each pass:
re-pin versions and re-verify the load-bearing comparison statements against the
live Docker containers (every claim is a theory; correct what no longer
reproduces), deepen one subsystem against the source code, verify performance
claims with production-realistic, no-tricks benchmarks logged under the
reproducibility contract (so I can re-run them by hand), verify each system's
native out-of-the-box metrics/logs/traces structure and the adopt-native-vs-custom
decision, write or update one focused note under
docs/research/greptimedb-vs-clickhouse/, commit and push it, then continue to the
next gap. Do not declare the comparison done.
```

For Claude Code scheduled repetition, use `/loop` with the same never-stop
wording:

```text
/loop Follow prompts/greptimedb-vs-clickhouse-internals.md as the active research
brief. Never stop on your own. Each pass: re-pin versions and re-verify the
load-bearing comparison statements against the live Docker containers (treat every
claim as a theory; correct anything that no longer reproduces), deepen one subsystem
against the source code, verify performance claims with production-realistic,
no-tricks benchmarks logged under the reproducibility contract (so I can re-run them
by hand), verify each system's native out-of-the-box metrics/logs/traces structure
and the adopt-native-vs-custom decision, write or update one focused note under
docs/research/greptimedb-vs-clickhouse/, commit and push it, then continue to the
next gap. Do not declare the comparison done; keep going until I stop you by hand.
```

The bare path also works for Claude Code
(`/loop prompts/greptimedb-vs-clickhouse-internals.md`), since the brief is
self-contained. The explicit wording above just makes the never-stop intent
unmistakable.

#### How often to trigger it

Use a **fixed interval** for this loop. Despite the general self-paced guidance
above, self-paced (`/loop` with no interval) has been observed to stop after a
while: self-paced mode relies on the running agent re-scheduling itself each pass,
so if a pass judges its work "done" it may not continue. A fixed interval re-fires
on the harness timer regardless of that judgment — that is what actually delivers
never-stop behavior, so prefer it here.

The interval is not about data freshness. This loop watches nothing external — the
source and releases of GreptimeDB and ClickHouse move on a weeks horizon, not
minute to minute — so no interval makes the data newer. The interval is purely the
re-trigger beat. The configured cadence for this loop is `5m`, chosen to keep it
as continuous as possible and to re-fire quickly if a pass ever stops early:

```text
/loop 5m Follow prompts/greptimedb-vs-clickhouse-internals.md as the active
research brief. Never stop. Each pass: re-pin versions and re-verify the
load-bearing comparison statements against the live Docker containers (every
"X is faster/abler" claim is a theory — rotate the slice so the whole record stays
re-verified; correct anything that no longer reproduces). Deepen one subsystem
against the source code; verify claims with production-realistic, no-tricks
benchmarks (model the often-run single-user queries, fair footing on both sides,
experimental features count as stable) and log each run to
local-benchmark-results.md under the reproducibility contract so I can re-run it by
hand. Also verify each system's native out-of-the-box metrics/logs/traces structure
and the adopt-native-vs-custom decision. Write or update one focused note under
docs/research/greptimedb-vs-clickhouse/, commit and push, continue to the next gap.
Do not declare the comparison done.
```

- Because one deep pass — read source, verify claims, optional local Docker run,
  write and commit a note — outlasts five minutes, the extra fires coalesce: the
  loop runs effectively back-to-back, not literally every five minutes. The `5m`
  setting mainly guarantees a prompt re-fire after any early stop.
- Caveat to watch: this assumes the scheduler coalesces fires while a pass is
  still running. If you see overlapping passes (racing commits, two Docker stacks
  on the same ports), raise the interval to `15m`–`30m`. Token spend is also higher
  at `5m`; raise it to spread cost.
- Each fire re-reads the brief and the current `docs/research/greptimedb-vs-clickhouse/`
  state and continues from the next gap, so a fresh re-trigger loses no progress.
- Re-pin versions and re-check public claims about every 1–2 weeks (or when either
  project ships a new stable release); that is the only cadence on which the
  underlying facts actually move.

Note `/loop` runs auto-expire after about 7 days and stop when the session closes
(they resume with `--resume` while unexpired); relaunch for a longer program.

## Output

Running these prompts should produce source-linked Markdown under
`docs/research/`, following the repo conventions in `AGENTS.md`. The
deep-research prompt still has gated deliverables — the GO / NO-GO verdict and,
if GO, the technical implementation concept — but the ongoing research program
does not end when those files exist. Later passes should keep re-verifying,
narrowing, replacing, and extending the research record.

## Prompt maintenance

Prompts are the durable source of operator intent for `/goal` and `/loop` runs.
When the operator clarifies the research target, confirms a direction, changes
evaluation criteria, or names tools that must be compared, update the relevant
prompt file in the same change if future runs would otherwise use stale
instructions.

## Adding a prompt

1. Add a new `*.md` file in this folder.
2. Add a row to the table above.
3. Update `PROJECT_STRUCTURE.md` in the same change.
