# Active Plans Execution Contract

Execute the unfinished work indexed in [`README.md`](README.md). This file and
all numbered plan files are one program contract; live source, tests, operator
rules, and current upstream documentation override stale implementation detail.

## Mission

Restructure Parallax using the proven, applicable mechanisms audited from
Jackin and PR #759 while preserving Parallax's product constraints. Complete
every actionable plan, keep genuinely blocked work minimal and current, prove
closure independently, and leave `plans/` containing unfinished work only.

## Operating Rules

1. Work on the single active branch named in `AGENTS.md`. Never create another
   branch or pull request. Parallel agents use disjoint write sets on that same
   branch and receive the same branch restriction.
2. Before a plan starts, read `AGENTS.md`, the index, its dependencies, its
   full file, and the live code. Recheck assumptions/versions and record drift.
3. Mark only one plan `IN PROGRESS` per overlapping write set. Update its
   status and evidence as durable commits land; never claim completion from
   prose, a checkbox, file existence, or grep alone.
4. Follow the dependency graph in the index. Plan 092 may run beside 093;
   disjoint UI/dependency/storage work may run in parallel after prerequisites.
5. Keep changes focused. Run targeted checks before each durable commit and
   the plan's complete gate before retirement. Commit with DCO, exactly one
   agent-product trailer, and push every durable update.
6. Apply current stable ecosystem documentation at execution time. Bun is the
   only JS/TS runtime/package manager. Native TLS, GreptimeDB + Turso, native
   raw-signal tables, zero-copy ingest, Apache-2.0, and progress visibility are
   non-negotiable.
7. STOP exactly when a plan's STOP condition fires. Capture reproducible
   evidence, shrink the file to current blocked work, set `BLOCKED`, and move
   to independent actionable work. Do not invent a bypass.
8. When a plan is terminal, store only durable decision/validation evidence
   outside `plans/` if it has lasting value, then delete the plan file and
   index row in the same commit. Never retain DONE/REJECTED/SUPERSEDED files.
9. New actionable work discovered during execution receives a unique new
   numbered file in `plans/`, dependencies, tests, done/STOP/remove criteria,
   and an index row before implementation. No plan, checklist, prompt, or
   active item may live elsewhere.
10. Run plan 107 last. Two independent auditors must inspect the same pushed
    implementation candidate from separate clean detached checkouts and agree
    after remediation. They then attest the exact mechanically limited staged
    cleanup tree in commit metadata; a repository-owned required check validates
    that tree/diff and full baseline at the final pushed commit.

## Completion State

The program is complete only when:

- every actionable numbered plan has passed its source and command gates and
  has been deleted with its index row;
- every remaining BLOCKED file has a fresh, exact external/operator trigger
  check and contains no completed implementation history;
- both full closure packets agree at the pushed implementation candidate, and
  the closure commit embeds two independent exact-tree attestations while its
  required `closure-final` check verifies the mechanical diff and full baseline;
- `JACKIN-REFERENCE.md` and this execution contract are deleted because no
  Jackin-alignment action remains; and
- repository search proves all remaining active plan material exists only in
  `plans/`.
