# Plan 174: The durable-upgrade and no-silent-loss contract — proven in CI, stated in docs

> **Executor instructions**: Follow this plan step by step. Run every
> verification command. On any "STOP conditions" item, stop and report.
>
> **Drift check (run first)**: `git diff --stat 7418bc9..HEAD -- crates/parallax-metadata/src/turso/ crates/parallax-spool/ crates/parallax-server/src/greptime_supervisor.rs crates/parallax-server/src/ingest_health.rs crates/parallax-server/src/worker/ crates/parallax-cli/src/doctor.rs .github/workflows/ docs/guide/`
> — on mismatch with the excerpts below, STOP.
>
> **Ratchet gate**: exact-match `ratchet.toml` rows per touched Rust file,
> updated in the same commit; `cargo xtask policy --only structural` green.
> Determinism policy: injected clocks/paths in tests, child-process env
> only.

## Status

- **Priority**: P1
- **Effort**: L (multi-PR: harness, then per-store guarantees)
- **Risk**: MED (CI harness downloads released binaries; guarantees may
  expose real defects — that is the point)
- **Depends on**: plans/169-rust-parity-and-structural-tests.md Step 4
  (versioned Turso migrations) for the metadata leg; the harness and the
  other legs can land first
- **Category**: tests
- **Planned at**: parallax `7418bc9`, 2026-08-14
- **Evidence base**: `docs/research/market/competitor-pain-points.md` —
  upgrade breakage is a top-reacted issue class for EVERY researched
  competitor (getsentry/self-hosted's most-reacted issues are migration
  failures 2020→2026; SigNoz 0.57 broke dashboards; OpenObserve
  schema-version mismatch blocks start; Maple's only local recovery is
  wiping telemetry; Uptrace v2 fails out of box). Silent data loss:
  OpenObserve compact-merge metadata loss + corrupt Parquet; Sentry SaaS
  drops events at quota unrecoverably.

## Why this matters

Every researched competitor ships upgrade breakage as a recurring
top-complaint, and two ship silent data loss. Parallax's product claim is
"self-host without the ops tax" — an upgrade that can strand a data dir,
or an ingest path that can drop telemetry without saying so, contradicts
that claim directly. Correctness framing: the code paths mostly exist
(spool forensics, `/health` degradation, bootstrap repair `ALTER`s,
checksum-verified engine download); what does not exist is the *contract*
— a stated guarantee, proven by CI on every release, that (a) a data dir
written by release N opens losslessly under release N+1, (b) telemetry is
never dropped silently: every drop path increments a visible counter and
degrades `/health`. Root-cause: the architecture never encoded
"upgrade-compatibility" or "loss-visibility" as testable properties, so
nothing stops a future change from violating them — this plan adds the
structural gate.

## Current state (verified)

- Turso: single `SCHEMA` const + ad-hoc PRAGMA-sniffed column adds
  (`crates/parallax-metadata/src/turso.rs:45`,
  `turso/connection.rs:12-20`); no version marker (plan 169 Step 4 adds
  `PRAGMA user_version` + fixture tests — this plan's harness consumes it).
- GreptimeDB: managed child pinned by checksum-verified download
  (`crates/parallax-server/src/greptime_supervisor.rs`); bootstrap =
  create-if-not-exists + repair `ALTER`s + TTL reconcile
  (`crates/parallax-greptime/src/greptime/lifecycle.rs`). An engine-version
  bump changes the child binary — nothing tests that an old engine data
  dir survives the pinned-version bump.
- Spool: PSPL1 frames + legacy `.ndjson` readable until reaped
  (`crates/parallax-spool/`); plan 114 (blocked) owns legacy retirement.
- Ingest health: `/health` returns 503 `degraded: <reason>`
  (`crates/parallax-server/src/ingest_health.rs`); per-signal workers with
  bounded queue (`worker/queue.rs`). Whether every drop/reject path feeds a
  counter + the degradation reason is UNVERIFIED — enumerating those paths
  is Step 3's job.
- Doctor: reports spool frame counts, engine health, delivery counts
  (`crates/parallax-cli/src/doctor.rs`).
- Release identity: `xtask release-*` gates exist; preview channel is the
  only published tag today (this constrains "previous release" — see
  Step 1).

## Commands you will need

| Purpose | Command | Expected |
|---|---|---|
| Gates | `cargo xtask ci --fast && cargo xtask lint && cargo xtask test && cargo xtask arch && cargo xtask policy --only structural` | green |
| Real engine | `cargo xtask integration` (after plan 168 Step 1) | green |
| Harness locally | `cargo nextest run -p parallax-server -E 'test(/upgrade/)' --run-ignored only` (harness tests are `#[ignore]`d — they download a released binary) | pass |
| Docs links | `cargo xtask docs links` | pass |

## Scope

**In scope**: a new upgrade-harness test file under
`crates/parallax-server/tests/` (+ helpers in parallax-test-support),
loss-visibility counters + tests in `crates/parallax-server/src/worker/`
+ `ingest_health.rs` + their doctor surfacing, a CI workflow job, the
contract statement in `docs/guide/` (new `upgrade-and-durability.md`) and
a pointer from `releases.md`.

**Out of scope**: plan 169's migration mechanism itself (consume, don't
duplicate); plan 114's legacy spool retirement; GreptimeDB engine-version
bumps themselves (the harness TESTS them when they happen); any backup
tooling (recorded direction only).

## Git workflow

PR-only `main`; ~3 PRs (harness, loss-visibility, docs+CI wiring);
`git commit -s`; Conventional Commits; agent trailer per `COMMITS.md`.

## Steps

### Step 1: Cross-release upgrade harness

New `#[ignore]`d integration test: download the latest published release
binary (preview channel today — resolve via the same mechanism
`docs/guide/releases.md` documents; if only `preview` exists, the harness
pins "previous = current preview at merge time" and upgrades to the
workspace build), run it against a temp HOME to seed a known dataset
(OTLP batch + one issue + one dashboard + one pinned bundle + spool
frames), stop it, then open the SAME data dir with the workspace-built
binary and assert: server starts; issue/dashboard/pin readable via
GraphQL; spool frame count unchanged; engine tables queryable; Turso
`user_version` advanced (once 169 landed). Also the reverse guard:
workspace-written data dir opened by the OLD binary must fail CLOSED with
a clear error, not corrupt (skip if the old binary predates the version
marker — assert it at least does not crash the data dir: reopen with new
binary still works).

**Verify**: harness passes locally via the run-ignored command; seeded
values byte-checked where hashes exist (pinned bundle hash identical).

### Step 2: CI wiring

New workflow job (path-aware per repo CI conventions —
`.github/workflows/`): runs the harness on PRs touching
`crates/parallax-metadata`, `crates/parallax-spool`,
`crates/parallax-greptime`, `crates/parallax-server`, and on every release
workflow run before publish. Cache the downloaded release binary.

**Verify**: workflow triggers on a test PR touching a listed path; release
rehearsal (`cargo xtask release-rehearse`, if runnable locally) documents
the new gate.

### Step 3: Loss-visibility audit + counters

Enumerate every path where telemetry can be dropped or rejected after
receipt: OTLP validation rejects, queue-full drops, normalize errors,
engine write failures, exponential-histogram/summary drops
(`crates/parallax-ingest/src/metrics.rs:84-87` — the comment already
promises "surfaced through doctor counters later"; this step delivers
that), spool write failures, broadcast lag (live tail — documented lossy,
still counted). For each: a counter (per signal, per reason), surfaced in
`parallax doctor` and used to set `/health` degradation where the class is
loss (not for the documented-lossy live tail). Tests per path: induce the
condition, assert counter increment + health state.

**Verify**: `cargo nextest run -p parallax-server -p parallax-ingest -E 'test(/drop|loss|counter/)'`
→ one test per enumerated path (list them in the PR description);
`parallax doctor` output shows the counters (CLI test extends plan 168
Step 12's suite if landed).

### Step 4: State the contract

`docs/guide/upgrade-and-durability.md`: the guarantee in plain words —
what is versioned, what the upgrade test proves per release, what
"degraded" means, where the counters live, what the spool does and does
not promise (forensic trail, not WAL — existing stance), and the explicit
non-promises (no downgrade support beyond fail-closed; live-tail is lossy
by design with lag counted). Link from `releases.md` and the README's
guide list.

**Verify**: `cargo xtask docs links` pass; every guarantee sentence maps
to a named test (list the mapping in the doc — guarantee → test name).

## Test plan

The harness IS the test (Step 1); Step 3 adds one failing-condition test
per drop path. No guarantee sentence without a named test.

## Done criteria

- [ ] Upgrade harness green locally + wired in CI (path-aware + release
      gate).
- [ ] Old-binary-on-new-data fails closed; new-binary-on-old-data lossless
      (seeded checks incl. pinned-bundle hash).
- [ ] Every enumerated drop path has counter + test + doctor surfacing;
      exp-histogram drop is now visible (the plan-166 decision then chooses
      model-vs-drop with visibility already in place).
- [ ] `docs/guide/upgrade-and-durability.md` ships with guarantee→test
      mapping.
- [ ] All gates green.
- [ ] `plans/README.md` row updated.

## STOP conditions

1. Drift check fails.
2. The harness reveals an ACTUAL lossy upgrade today — record as
   `DISCREPANCY:` (plan 166 pipeline), fix root-cause there or here per its
   triage, but do not ship the contract doc claiming what the test
   disproves.
3. A drop path cannot be counted without touching the zero-copy hot path's
   ownership discipline — report the design conflict (counter placement
   options) instead of cloning telemetry.
4. No previous release binary is resolvable in CI (network/tag constraints)
   — report; the harness design (not its existence) is the negotiable part.

## Maintenance notes

- Every future schema/format change (Turso step, spool framing, greptime
  pinned-version bump, bundle schema) must extend the seeded dataset in
  the harness — reviewers block on harness-untouched format changes.
- The guarantee→test mapping in the doc is the audit surface: a guarantee
  without a green test is a doc bug.
- Recorded direction (not planned here): `parallax backup`/restore
  tooling; multi-node story. Both belong to V2 profile decisions.
