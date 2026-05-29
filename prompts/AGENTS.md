# AGENTS.md — `prompts/`

Authoring rules for prompt files in this folder. Linked by `prompts/CLAUDE.md`. These
complement the root [`AGENTS.md`](../AGENTS.md): the root file governs prompt *intent* (keep
prompts aligned with current operator decisions); this file governs prompt *form*.

## Prompt files contain only the goal

Every `*.md` prompt in this folder must be **pure mission content that can be passed directly
to `/goal` or `/loop`** (or pasted into an agent as a one-off task). A prompt states the
objective, the context, what to achieve, the constraints, and the stop condition — and
nothing about itself.

A prompt file must **not** contain:

- "How to use this file" / "what this file is for" preambles;
- run-mechanics — which command runs it, `/goal` vs `/loop`, intervals, scheduling;
- meta about where the file sits in the repo or how it relates to other prompts.

All of that lives in [`README.md`](README.md) **only**. The README is the single place that
explains what each prompt is for and how to run it; the prompt files stay runnable as-is.

Nuance: "how to run the *work*" — for example, how to stand up a benchmark stack or what
artifacts to produce — is mission content and belongs in the prompt. The rule is only about
how to run *the prompt itself*.

## When you add or change a prompt

1. Keep the prompt file goal-only. If you wrote any usage or run note, move it to
   [`README.md`](README.md).
2. Add or update the prompt's row and run notes in [`README.md`](README.md).
3. Update [`../PROJECT_STRUCTURE.md`](../PROJECT_STRUCTURE.md) in the same change.
