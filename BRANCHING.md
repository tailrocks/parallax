# Branching

Parallax currently iterates directly on `main`.

## Current Policy

- Use `main` for routine research, documentation, and early structure updates.
- Do not open pull requests unless the operator explicitly asks for one.
- Commit focused changes directly to `main`.
- Push after each durable finding or repository-structure update.

This differs from the later expected project workflow. Pull requests are useful
once implementation work, CI, review, and release discipline matter. They are
unnecessary overhead while the repository is mostly product research and market
analysis.

## When to Revisit

Reconsider feature branches and pull requests when one of these becomes true:

- The repo has production code that can regress.
- More than one human collaborator starts committing regularly.
- CI checks become meaningful merge gates.
- The project starts publishing releases or hosted artifacts.

Until then, keep the process simple and optimize for fast research iteration.

## History Rewrites

Because this is a private draft repository, history rewrites are acceptable when
the operator explicitly asks for them. When rewriting published history, use:

```sh
git push --force-with-lease
```

Do not rewrite history silently. Name the reason before doing it.
