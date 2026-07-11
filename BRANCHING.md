# Branching

Parallax currently iterates directly on `main` for research, with one active
implementation branch when the operator names it.

## Operator override (2026-07-11) — one branch, one PR

While the advisor-plans / v1 implementation program is open:

| Rule | Detail |
|------|--------|
| **Only branch** | `implement/advisor-plans-069-090` |
| **Only PR** | One open PR: that branch → `main` |
| **Never** | Create any new branch; open a second PR; split work across named side branches (`implement/076-…`, `implement/078-…`, per-agent `codex/…` implement branches, etc.) |

### For the main agent and every subagent

1. `git checkout implement/advisor-plans-069-090` (or stay on it).
2. Commit and push **only** to that branch.
3. Do **not** run `git checkout -b`, `git switch -c`, or create remote branches.
4. Do **not** open a new GitHub PR if one already exists for this branch.
5. Subagent prompts must restate: *single branch
   `implement/advisor-plans-069-090`, single PR, never create branches*.

Parallelism is allowed only as concurrent work **on that same branch** (or
worktrees that check out the same branch). Do not invent branch names to
isolate plans.

## Default research policy (when no implement branch is active)

- Use `main` for routine research, documentation, and early structure updates.
- Do not open pull requests unless the operator explicitly asks for one.
- Commit focused changes directly to `main`.
- Push after each durable finding or repository-structure update.

This differs from a heavy multi-PR workflow. Pull requests are useful once
implementation work, CI, review, and release discipline matter. They are
unnecessary overhead while the repository is mostly product research and market
analysis — except when the operator explicitly requests a single PR as above.

## When to Revisit

Reconsider multi-branch stacks when one of these becomes true **and** the
operator lifts the single-branch override:

- The repo has production code that can regress across independent features.
- More than one human collaborator starts committing regularly.
- CI checks become meaningful merge gates for independent PRs.
- The project starts publishing releases or hosted artifacts.

Until then, keep the process simple: one active implement branch when named,
otherwise `main`.

## History Rewrites

Because this is a private draft repository, history rewrites are acceptable when
the operator explicitly asks for them. When rewriting published history, use:

```sh
git push --force-with-lease
```

Do not rewrite history silently. Name the reason before doing it.
