# Branching

Parallax uses **one active branch at a time**. Do not create side branches.

## Operator rule (2026-07-11) — single branch always

| Rule | Detail |
|------|--------|
| **Active branch** | `main` (after PR #19 merge; implement branch deleted) |
| **Never** | Create new branches; open a second PR; split work across named side branches |
| **Subagents** | Same branch as the main agent; restate the rule in every spawn prompt |

When the operator later asks for a PR, use **one** named branch only, open
**one** PR to `main`, merge it, delete the branch, and return to `main`.

### For the main agent and every subagent

1. Stay on the active branch (`main` unless the operator says otherwise).
2. Commit and push **only** there.
3. Do **not** run `git checkout -b`, `git switch -c`, or create remote branches.
4. Do **not** open a new GitHub PR unless the operator asked and none exists yet.

Parallelism is allowed only as concurrent work **on that same branch**.

## Default workflow

- Commit focused changes on the active branch.
- Push after each durable finding or repository-structure update.
- Pull requests only when the operator explicitly requests one.

## History Rewrites

Because this is a private draft repository, history rewrites are acceptable when
the operator explicitly asks for them. When rewriting published history, use:

```sh
git push --force-with-lease
```

Do not rewrite history silently. Name the reason before doing it.
