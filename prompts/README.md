# Prompts

Reusable research and agent prompts for Parallax. Each file here is a
self-contained brief you can hand to an AI coding agent (Claude Code, Codex, Amp,
OpenCode) so a piece of work runs the same way every time.

## Available prompts

| File | Purpose |
| --- | --- |
| `deep-research-parallax.md` | Deep, critical research brief that validates the Parallax direction and must end in a concrete technical implementation concept (which system, which storage, how to build it). |
| `greptimedb-vs-clickhouse-internals.md` | Never-ending `/loop` brief for the under-the-hood GreptimeDB vs ClickHouse comparison: read the source, explain which design decisions make each fast or slow per signal, decide which to build Parallax on, and (when one wins but lacks features) map what the winner must implement to close the gap. Writes to `docs/research/greptimedb-vs-clickhouse/`. |

## How to run a prompt

### 1. As a one-off

Open the agent and hand it the file as the task — paste the contents, attach the
file, or reference the path (for example `@prompts/deep-research-parallax.md` in
Claude Code). The agent works the brief and writes findings to `docs/research/`.

### 2. As a goal (`/goal`) — recommended

`/goal` makes the agent treat the prompt as an objective and keep working across
turns until a completion condition is met. This is the recommended way to run a
long research session: give it a high completion bar and it keeps going on its
own until the bar is reached, then stops — no wasted runs afterward.

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

To get MORE results rather than fewer, the lever is the completion condition, not
the command: a sharp, demanding bar ("done only when the verdict exists and, if
GO, the blueprint covers API, boundary + MCP decision, all three tiers, and a
per-layer stack") keeps the agent grinding through many passes. A vague goal
("research Parallax") lets it declare victory early — avoid that.

For an ongoing research program that should keep working after the verdict and
blueprint exist, use an open-ended completion condition:

```text
/goal Continue deep research on prompts/deep-research-parallax.md indefinitely.

Keep treating the prompt as the active research brief. Do not stop after one
synthesis or one document. Work in repeated passes: re-read the prompt and
current docs/research state, identify the weakest or stalest claim, research
current primary sources, update focused Markdown notes, update durable prompt or
repo-shape docs when needed, commit and push each durable section, then continue
to the next highest-value research gap.

Do not mark the goal complete merely because docs/research/verdict.md or
docs/research/technical-implementation-concept.md exists. Keep the goal active
until I explicitly stop it, replace it, or say the research program is complete.
```

### 3. On a loop (`/loop`)

`/loop` re-runs a prompt on an interval (or self-paced) and never self-completes —
you stop it by hand when the picture is clear enough. Use it when you want
endless deepening passes rather than convergence on a fixed deliverable:

```text
# self-paced: the agent keeps starting fresh passes until you stop it
/loop prompts/deep-research-parallax.md

# fixed interval: re-run every 30 minutes (only if you want pauses between passes)
/loop 30m prompts/deep-research-parallax.md
```

Notes:

- Omit the interval to let the model self-pace (back-to-back passes, no idle
  waits — right for research, since nothing external is being watched). Pass an
  interval only if you want breathing room to review between passes.
- `/loop` tasks auto-expire after about 7 days, and stop if the session closes
  (they resume on `--resume` while unexpired).
- Do NOT wrap `/goal` inside `/loop` for this open-ended research. `/goal`
  already self-completes, so `/loop /goal ...` just restarts a finished goal —
  redundant. Pick one: `/goal` to converge on the verdict + blueprint and stop,
  or `/loop` for endless passes you end by hand.

### Running the GreptimeDB vs ClickHouse internals comparison

`greptimedb-vs-clickhouse-internals.md` is an indefinite, never-converging brief,
so run it with `/loop` (not `/goal`). Use a clear, explicit instruction so the run
keeps going:

```text
/loop Follow prompts/greptimedb-vs-clickhouse-internals.md as the active research
brief. Never stop on your own — keep researching the GreptimeDB vs ClickHouse
internals pass after pass, each pass deepening one subsystem against the source
code and verifying performance claims, write or update one focused note under
docs/research/greptimedb-vs-clickhouse/, commit and push it, then continue to the
next gap. Do not declare the comparison done; keep going until I stop you by hand.
```

The bare path also works (`/loop prompts/greptimedb-vs-clickhouse-internals.md`),
since the brief is self-contained — the explicit wording above just makes the
never-stop intent unmistakable. Do not use `/goal` here, because the brief is
designed to keep going rather than reach a fixed deliverable.

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
research brief. Never stop — each pass, deepen one subsystem against the source
code, verify performance claims (local Docker run where useful), write or update
one focused note under docs/research/greptimedb-vs-clickhouse/, commit and push
it, then continue to the next gap. Do not declare the comparison done.
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
`docs/research/`, following the repo conventions in `AGENTS.md`. The deep-research
prompt specifically must end in a technical implementation concept: which system
per layer, which default storage and why, and the best way to build it, with a
startup-first deployment and a horizontal scale-out path.

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
