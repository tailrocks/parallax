# Prompts

Reusable research and agent prompts for Parallax. Each file here is a
self-contained brief you can hand to an AI coding agent (Claude Code, Codex, Amp,
OpenCode) so a piece of work runs the same way every time.

## Available prompts

| File | Purpose |
| --- | --- |
| `deep-research-parallax.md` | Deep, critical research brief that validates the Parallax direction and must end in a concrete technical implementation concept (which system, which storage, how to build it). |

## How to run a prompt

### 1. As a one-off

Open the agent and hand it the file as the task — paste the contents, attach the
file, or reference the path (for example `@prompts/deep-research-parallax.md` in
Claude Code). The agent works the brief and writes findings to `docs/research/`.

### 2. As a goal (`/goal`)

When you want the agent to treat the prompt as its objective and drive toward it
autonomously, run it through your goal command and point it at the file:

```text
/goal prompts/deep-research-parallax.md
```

The agent adopts the prompt as the goal, does the research, and produces the
deliverables the prompt asks for (research notes plus the implementation
concept).

### 3. On a loop (`/loop`)

`/loop` runs a prompt or command repeatedly so long research can iterate without
re-invoking it by hand:

```text
# self-paced: the agent decides when to continue each iteration
/loop /goal prompts/deep-research-parallax.md

# fixed interval: re-run every 30 minutes
/loop 30m /goal prompts/deep-research-parallax.md
```

Omit the interval to let the model self-pace; pass one (for example `30m`) to run
on a fixed schedule. Use this for research that benefits from multiple passes —
the agent extends and sharpens the notes under `docs/research/` over time.

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
