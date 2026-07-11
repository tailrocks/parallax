# Advisor Plans Implementation — Exhaustive Closure of 069–090

Implement **every** active item in `advisor-plans/` until the index shows zero
open work and a second full re-audit finds nothing left to do. The mission is
not "mostly done," not "good enough," and not "the high-value plans landed."
The mission is **complete, criteria-true closure** of the active program:
every plan file's steps, every machine-checkable done criterion, every
verification gate, every STOP-condition honesty rule, and every residual item
the index still names after plans land.

This brief is the durable operator intent for a long-running `/goal` run. Plans
are the source of truth for *what* to change; this file is the source of truth
for *how the run behaves* until the stop condition is met.

## Authoritative sources (read, do not invent)

1. `advisor-plans/README.md` — program index, dependency order, status rows,
   verified fact base, rejected/deferred findings, open operator questions.
2. **Every** plan file under `advisor-plans/` whose status is not `DONE`:
   - Codebase audit program: **069–083**
   - Performance & GreptimeDB-integration program: **084–090**
3. Completed plans **019–034** (telemetry-quality + observability program): do
   **not** re-implement. Confirm their status rows remain honest. If a later
   plan or regression suite shows a DONE plan is no longer true, reopen its
   row (`TODO` or `BLOCKED` with reason) and fix to the plan's original done
   criteria before continuing.
4. Repo law: `AGENTS.md`, `COMMITS.md`, `BRANCHING.md` (this run works on the
   implementation branch named by the operator; otherwise stay on the current
   implementation branch), `PROJECT_STRUCTURE.md`, crate/`ui` AGENTS files,
   `docs/research/architecture/v1-implementation-spec.md` for contracts.
5. Primary engine truth when a plan requires it: live GreptimeDB (managed
   binary / embedded path), docs.greptime.com, and the engine version the
   supervisor pins — never marketing claims alone.

If a plan's "Current state" excerpts drift from HEAD, run that plan's drift
check first. On material mismatch: treat as that plan's STOP condition —
re-derive against live code by **function name and contract**, not by stale
line numbers; do not invent a different product design.

## Absolute completion bar (non-negotiable)

You may **never** declare success with language like:

- "mostly achieved"
- "looks good enough"
- "remaining items are minor"
- "the important plans are done"
- "deferred for later" (unless the plan itself marks a sub-item deferred, or
  the operator answers an open question that forces deferral)

A plan is complete **only** when **all** of the following are true:

1. Every **Step** in that plan file was executed in order (or an explicit STOP
   was hit and reported, with the plan status set to `BLOCKED` and a one-line
   reason — not silently skipped).
2. Every checkbox under that plan's **Done criteria** is machine-true on the
   current tree (run the greps/commands; do not eyeball).
3. That plan's verification commands pass with the expected results.
4. That plan's **STOP conditions** were not violated by improvisation.
5. The status row in `advisor-plans/README.md` is updated to `DONE` (or
   `BLOCKED` / `REJECTED` / `SUPERSEDED` with a one-line rationale that matches
   reality).
6. Changes are committed (Conventional Commits, DCO signoff, single agent
   `Co-authored-by` trailer per `COMMITS.md` / `AGENTS.md`) and pushed on the
   implementation branch.

A program is complete **only** when **all** of the following are true:

1. Every plan **069–090** is `DONE`, or legitimately `BLOCKED`/`REJECTED` with
   operator-visible reason and no remaining executor-actionable work.
2. Spikes (**083**, **090**) produced their required reports/notes; every `GO`
   verdict spawned a new plan file under `advisor-plans/` **and that new plan
   is also executed to DONE** before this goal may stop (or is explicitly
   blocked on an operator decision recorded in the index).
3. Every "Additional findings recorded, not separately planned" and every
   "Open questions for the operator" item in `advisor-plans/README.md` has a
   disposition: **addressed in a plan**, **new plan created and DONE**,
   **BLOCKED on operator**, or **reaffirmed deferred with evidence it is still
   correctly out of scope** (not hand-waved).
4. A **closure re-audit** (see below) has been run after the last `DONE` flip
   and found zero remaining actionable gaps. Then run the closure re-audit
   **again**. Only after two consecutive clean re-audits may you stop.

If anything is still greppable as unfinished, still TODO in the index, still
failing a done-criterion command, or still an un-dispositioned residual — the
goal is **not** complete. Keep iterating.

## Active inventory (must all close)

Status at authoring of this brief: all of **069–090** are `TODO` in the index.
Treat the live `advisor-plans/README.md` tables as authoritative; if a row is
already DONE when you start, re-verify its done criteria once, then move on.

### Codebase audit program (069–083)

| Plan | Title (short) | Priority | Depends on |
|------|---------------|----------|------------|
| 069 | CI verification gates (UI test/lint, embed-ui, nightly real-engine, timeouts) | P1 | — |
| 070 | Rust correctness batch (requestRate table, ingest panic, first_seen, stuck runs) | P1 | — |
| 071 | UI correctness batch (cycle guard, stale paging, bucket race, gqlString) | P1 | 069 soft |
| 072 | Redaction bypasses + agent-trust delimiting in bundles | P1 | — |
| 073 | Spool durability truth (retry, shutdown drain, honest spool docs) | P1 | 070 |
| 074 | GreptimeDB SQL testability (golden SQL, escape tests, conformance) | P1 | 070; 069 soft |
| 075 | Read-path performance (traces_search window, fan-out, table cache) | P2 | 074 |
| 076 | Ingest hot path (spool locks/IO, batched upserts, normalize churn) | P2 | 073, 070 |
| 077 | Shared SSE hook + real stream health in Live badges | P2 | 071 |
| 078 | Split parallax-api lib.rs into domain modules | P2 | 069; after 072/073/075; **after 086** |
| 079 | UI query/type dedup + dependency hygiene + rotel.env template | P2 | 069; after 071/077 |
| 080 | Onboarding docs (dev setup, ui/README, PROJECT_STRUCTURE, cli.md) | P3 | — |
| 081 | `--format json` on bundle commands + agent-session CLI verb | P2 | — |
| 082 | Publish bundle-v1 JSON Schema + conformance test | P3 | 072, 081 |
| 083 | MCP read-only adapter SPIKE (projection equivalence; do not ship product) | P3 | 072, 081; 082 soft |

### Performance & GreptimeDB-integration program (084–090)

| Plan | Title (short) | Priority | Depends on |
|------|---------------|----------|------------|
| 084 | GreptimeDB integration corrections (matches_term, indexes, log schema, TTL, timeouts, version upgrades) | P1 | 070; 074 soft |
| 085 | Read-path SQL rewrites (window scans, SQL aggregates, round-trip collapse, uncast timestamps) | P1 | 074, 075, 084 |
| 086 | API request memoization + batching (must land **before** 078) | P1 | 070; **BEFORE 078** |
| 087 | Ingest pipeline restructure (gzip OTLP/HTTP, per-signal workers, raw-bytes spool, bounds) | P1/P2 | 073, 076 |
| 088 | UI data layer (cache, dashboard fan-out, run scan bound, visibility gating, issues table) | P2 | 071, 077, 079 |
| 089 | Extension-table writes via gRPC ingester + metric_exemplars PK fix | P2 | 084, 070 |
| 090 | SPIKE: measure read transport + partition defaults (no product code under `crates/`/`ui/`) | P3 | 084; 085 recommended |

### Hard sequencing rules (do not reorder away)

- **069 early** — baseline CI/UI gates so later UI/Rust work is enforced.
- **070 before 073, 074, 076, 084, 086, 089** — correctness fixes those plans assume.
- **074 before 075 before 085** — golden net first; 075 changes semantics after
  pin; 085 builds on both plus 084.
- **073 before 076 before 087** — durability/worker/spool shapes stack.
- **071 → 077 → 079 → 088** on shared UI routes (`logs`, `traces`, `runs`).
- **086 before 078** — memo/context rewires resolvers; splitting first doubles
  churn. Re-check the index before starting 078.
- **072 before 082 and 083** — never demonstrate the weaker redactor on an
  agent surface.
- **081 before 082** (CLI JSON is the projection reference); 083 compares
  byte-equality against 081's JSON where specified.
- **084 before 089 and before 090**; 089 after 084 on the same bootstrap region.
- **087 Step 1 (gzip OTLP/HTTP)** may be cherry-picked ahead of the rest of 087
  if 073/076 are slow — it is a standalone P1 interop/data-loss fix.
- **090** writes no product code under `crates/`/`ui/`; any GO becomes a new
  plan that must also close under this goal.
- **083** is a spike: prove projection equivalence and write findings; do not
  ship product MCP as the definition of done.

Recommended backbone (parallel only when dependencies and file ownership allow):

1. **069, 070, 072, 080, 081** (independent / early)
2. **071** after 069 soft; **073** after 070; **074** after 070
3. **075** after 074; **076** after 073; **077** after 071
4. **084** (after 070; soft 074); **086** (after 070; **before 078**)
5. **085** after 074+075+084; **079** after 071+077; **082** after 072+081
6. **087** after 073+076; **088** after 071+077+079; **089** after 084
7. **078** only after 072/073/075 **and 086**
8. **083** after 072+081; **090** after 084 (ideally after 085)
9. Disposition residuals → second and third closure re-audits

Inside each plan: **read the full plan file first**, honor its in-scope /
out-of-scope lists, run its drift check, execute every step, run every verify
command, check every done criterion, update the index row, commit, push.

## Per-plan execution protocol

For **each** plan, in dependency-safe order:

### A. Enter

1. Read the entire plan file end-to-end (not just the title).
2. Run the plan's **Drift check**.
3. Confirm dependencies in `advisor-plans/README.md` are `DONE` (or soft-deps
   acknowledged). If a hard dependency is not DONE, do that plan first.
4. Set the plan's status row to `IN PROGRESS`.

### B. Execute

1. Follow **Steps** in order. Do not skip a step because it looks small.
2. After each step, run that step's **Verify** commands when the plan names
   them; otherwise run the plan's command table gates before the next step.
3. Stay inside **In scope**. Out-of-scope paths are forbidden even when related.
4. On any **STOP condition**: stop that plan, set status `BLOCKED` with a
   one-line reason, commit the report (index + short note under
   `docs/research/validation/` if the finding is durable), push, and continue
   with other unblocked plans. Do not improvise around a STOP.
5. Spikes (083, 090): produce the plan's required artifact (findings report /
   measurement note / poc harness). If the verdict is GO for product work,
   author a new numbered plan under `advisor-plans/`, add a README row, and
   schedule it in this same goal until DONE.

### C. Exit

1. Run **every** Done-criteria command; all must match the plan's expected
   results.
2. Run workspace gates relevant to the change (see Verification baseline).
3. Update the status row to `DONE`.
4. Commit with a subject that matches the plan's suggested Conventional Commit
   style; include DCO (`git commit -s`) and the correct single agent trailer.
5. Push the implementation branch.
6. Surface in the transcript: plan id, what landed, gates run, next plan id.

Never batch "half of plan A + half of plan B" into one vague commit that makes
done criteria un-auditable. One plan may be multiple commits (as the plan
suggests); the plan is not DONE until its full criteria pass.

## Residual inventory (must be dispositioned, not ignored)

These are first-class goal items even when they lack their own plan number:

1. **README "Additional findings recorded, not separately planned" (084–090
   section)** — each bullet gets a disposition.
2. **README "Findings considered and rejected / deferred" (069–083 and
   022–034 sections)** — reaffirm or reopen; do not re-litigate settled
   rejects unless code drift invalidates the rejection.
3. **README "Open questions for the operator"** — if unanswered and blocking a
   plan STOP, set that plan `BLOCKED` and continue other work; do not invent
   the product decision. If a question is non-blocking, record the assumption
   the plan already states and keep going.
4. **Maintenance notes inside each plan** that name follow-ups required for
   correctness of the landed work (not distant V2 dreams) — either fold into
   the plan before DONE, open a new plan, or document as intentionally out of
   scope with evidence.
5. **New defects discovered while executing** that are in-scope for the
   active programs — fix under the current plan if the plan's scope allows;
   otherwise add a plan file + README row and execute it before closure.
6. **Regressions of DONE 019–034** — fix immediately; reopen status if needed.

"Deferred" is only valid when the index already deferred it **and** current
evidence still supports that choice. "I did not have time" is never a valid
disposition.

## Verification baseline (repo reality)

Use the repo's real commands (and `rtk` when the environment provides it):

**Rust (repo root):**

- `cargo build --workspace`
- `cargo clippy --workspace --all-targets` → zero warnings (`-D warnings` in CI)
- `cargo fmt --all` / `cargo fmt --check`
- `cargo nextest run --workspace` (and `--all-targets` when matching CI)
- Gated real-engine: `cargo nextest run --workspace --run-ignored only` when a
  plan requires it (074/075/084/089/069 nightly path)

**UI (from `ui/`):**

- `bun install --frozen-lockfile` when deps change
- `bun run typecheck`
- `bun run lint`
- `bun run test` / `bun run test:ci` once 069 lands
- `bun run build`

**Workflows / misc when touched:**

- `actionlint` on workflow edits
- Plan-local greps listed under each Done criteria

Do not invent alternate green bars. Do not weaken existing test assertions to
make a plan pass (plans call that out as STOP).

## Engineering constraints (always on)

- **Stack**: GreptimeDB (telemetry) + Turso (metadata) only — no fallback
  engines, no rusqlite product path, no Postgres swap.
- **Native OTel tables** for raw signals always (`opentelemetry_logs`,
  `opentelemetry_traces`, native metrics tables). No hand-rolled raw-signal
  tables; extension tables only when justified and documented.
- **Ingest zero-copy**: decode once, move ownership forward; no new hot-path
  telemetry clones on the first-attempt path (retries are the documented
  exception where plans allow).
- **TLS**: native TLS only, never rustls (including dependency feature choices).
- **JS/TS**: Bun only (runtime + package manager); `bun.lock` only.
- **Versions**: latest mutually-compatible stable; update pins/docs in the same
  change when you bump.
- **API boundary**: CLI and UI talk to GraphQL/HTTP only; only storage adapters
  touch GreptimeDB/Turso.
- **Claim discipline**: no product claims beyond what tests/gates prove.
- **Apache-2.0 / Tailrocks** attribution in new artifacts that declare license
  or company metadata.
- **Progress visibility**: long-running local steps narrate what they are doing.

## Per-pass proof of progress

Each working session / pass must leave a transcript-visible trail:

1. Which plan id is in progress.
2. Drift-check result (clean / adapted / STOP).
3. Steps completed this pass.
4. Commands run and results (pass/fail).
5. Files changed.
6. Commit hash(es) and push confirmation.
7. Index status transitions.
8. Next concrete plan id (or "closure re-audit").

Use the goal progress tool when available to mark progress; never claim
`completed` until the global stop condition below is truly met.

## Closure re-audit (mandatory, twice)

After the last plan row first reaches a terminal status, run a full closure
pass:

1. Re-read `advisor-plans/README.md` tables — every 069–090 row terminal.
2. For every `DONE` plan, re-run its Done-criteria commands on HEAD.
3. Re-run full workspace gates (Rust + UI).
4. Scan for orphan TODOs: `TODO`/`FIXME` introduced by this program that the
   plans required to be gone; unfinished spike follow-ups; GO verdicts without
   child plans; child plans not DONE.
5. Scan residuals list above for any missing disposition.
6. Write a short closure ledger under
   `docs/research/validation/YYYY-MM-DD-advisor-plans-closure.md` listing each
   plan id → DONE/BLOCKED/REJECTED, gate results, residual dispositions, and
   "items remaining: none" **only if true**.
7. Commit and push the ledger.

Then **repeat the entire closure re-audit from step 1**. Only if the second
pass independently finds **items remaining: none** may the goal complete.

If the second pass finds anything, fix it, and require **two consecutive clean
re-audits** again.

## Global stop condition

Stop **only** when all are true:

1. `advisor-plans/README.md`: plans **069–090** are all terminal (`DONE` /
   legitimate `BLOCKED` / `REJECTED` / `SUPERSEDED`), with no `TODO` or
   `IN PROGRESS` left among them.
2. Every `DONE` plan's Done criteria still pass on HEAD (verified in the latest
   closure re-audit).
3. Every spike GO child plan is also terminal.
4. Every residual/open-question item has an explicit disposition.
5. Two consecutive closure re-audits report **items remaining: none**.
6. Workspace verification baseline is green for the landed surface.
7. Implementation branch is pushed with the closure ledger committed.

Until then: pick the highest-priority unblocked unfinished plan (or the
blocking dependency of one), and continue. Do not stop early. Do not ask the
operator whether "this is enough" when executable work remains. Do not redefine
done criteria downward.

If every remaining item is truly `BLOCKED` on operator decisions (not on
executor effort), stop only after: all such BLOCKED rows are recorded, the
closure ledger lists each blocker, two re-audits agree no executor-actionable
work remains, and the transcript states the exact operator questions that
unblock the rest.
