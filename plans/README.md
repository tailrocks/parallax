# Active Implementation Plans

Execution rules for the leftover program live in
[`IMPLEMENTATION.md`](IMPLEMENTATION.md). The self-contained Rust/TypeScript
target lives in [`ENGINEERING-STANDARDS.md`](ENGINEERING-STANDARDS.md). Numbered
plans contain all information required for implementation and never require an
executor to reconstruct how a decision was originally researched. The optional
[`OXC-IMPLEMENTATION.md`](OXC-IMPLEMENTATION.md) contains only official
implementation lookups for refreshing Oxc/TypeScript component status at
execution time. The compact [`GOAL.md`](GOAL.md) brief drives the leftovers
without duplicating the numbered plans.

Run the leftover program with:

```text
/goal Follow plans/GOAL.md until its Done condition is mechanically proven.
```

`plans/` is the only home for active Parallax implementation plans. It contains
unfinished work only. Completed, rejected, or superseded work belongs in Git
history and, when durable evidence is useful, under `docs/research/validation/`.

## Lifecycle

1. Use a unique, never-reused numeric ID and a flat
   `plans/NNN-kebab-case.md` path.
2. List only `TODO`, `IN PROGRESS`, or `BLOCKED` files in this index.
3. A plan file contains status metadata, current-state rationale, scope, ordered
   steps, tests, machine-checkable done criteria, STOP conditions, and a
   `Remove When` section.
4. When a plan becomes terminal, record durable evidence if needed, then delete
   its file and index row in the same commit. Do not keep a DONE archive here.
5. Work directly on the single active branch from `AGENTS.md`; commit with DCO
   and exactly one agent-product trailer, then push each durable update.
6. `GOAL.md` is an orchestration brief, not another plan or source of
   architecture. Plan 107 deletes it in the final mechanical closure commit.

## Program Constraints

Every plan must preserve these non-negotiable Parallax constraints:

- GreptimeDB + Turso only; no product fallback engine.
- GreptimeDB native raw-signal tables.
- Native TLS only; never an active rustls backend.
- Bun only for JavaScript/TypeScript.
- Decode once and move ownership on the ingest hot path.
- Apache-2.0 throughout.
- One active branch; no per-plan or per-agent branches.

## Execution Preflight

Facts an executor may rely on without re-deriving; re-verify only on failure:

- **Host**: operator's macOS arm64 machine. Docker running; `mise`, cargo, bun,
  and cargo-nextest present.
- **Push rights**: the `gh`-authenticated account has admin on
  `tailrocks/parallax`, `tailrocks/parallax-telemetry-playground`, and
  `tailrocks/homebrew-parallax`; direct pushes to `main` succeed (parallax's
  ruleset is bypassed by admin).
- **Delivery model**: leftover execution lands as direct commits to `main`.
  Do not open a second PR for leftover implementation.
- **Live-engine test lanes**: real-GreptimeDB tests download and cache the
  engine themselves (`target/greptime-test-bin/`) and are gated behind
  `#[ignore]` — run them with `cargo nextest run --run-ignored all -E
  'binary(/greptime/)'` (or the per-test command in each test header).
- **Browser verification**: `agent-browser` at `/opt/homebrew/bin/agent-browser`
  is the designated tool when a leftover step requires a browser. Chrome
  DevTools MCP is the fallback only if the CLI is unavailable.

## Operator Unblock (leftover-relevant)

Hard limits: no destructive history rewrites, no rustls, no engine
substitutions, no gate weakening, no fabricated evidence. A plan may be marked
`BLOCKED` only for a hard external fact (upstream bug, unreachable service),
with fresh reproducible evidence.

Binding leftover decision:

- **Plan 089**: rescoped to a fix-forward upstream contribution (native-TLS /
  plaintext feature in `greptimedb-ingester`). Do not fork, enable rustls, or
  weaken native-TLS policy.

Completed-program evidence (not current work) lives under
[`docs/research/validation/`](../docs/research/validation/).

## Active Plans

Unfinished leftovers only.

| Plan | Title | Priority | Effort | Depends on | Status |
|------|-------|----------|--------|------------|--------|
| [089](089-extension-table-grpc-writes.md) | Move derived extension-table writes to GreptimeDB's row API | P2 | M | upstream `greptimedb-ingester` native-TLS/plaintext feature fix | BLOCKED — crates.io still 0.18.0; upstream PR #58 OPEN not merged (recheck 2026-07-17T17:18Z); HTTP SQL path remains |
| [114](114-retire-legacy-spool-reader.md) | Retire the legacy NDJSON spool reader | P2 | S | Qualifying stable raw-frame release cycle + expired legacy segments | BLOCKED — only rolling `preview` tag (recheck 2026-07-17T17:18Z) |
| [107](107-program-closure-audits.md) | Run independent source audits and verify the mechanical closure commit | P1 | M | Every other leftover; all blockers freshly rechecked | IN PROGRESS — audit round 1 complete 2026-07-18; C0 `0e5392a2` Step 4 (Auditor B CLEAN); C0 freeze next when quiet window + CI |

## Dependency Order

```text
089 (BLOCKED, upstream ingester) ─┐
114 (BLOCKED, stable release)    ─┼─► 107 last
```

Do not implement 089 or 114 while their exact external triggers still fail.
Recheck the trigger, refresh evidence, and leave the file `BLOCKED`. Plan 107
is last and must not impersonate closure while 089/114 remain unfinished
blockers. C0 freeze requires every other leftover to be a minimal BLOCKED file
whose exact condition was freshly reproduced.

## Shared Verification

Each leftover has narrower commands. The final program baseline is:

```text
git diff --check
git diff --cached --check
cargo fmt --all --check
cargo check --locked --workspace --all-targets
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo xtask dependencies --all
cargo xtask ui graphql check
cargo xtask policy --only ui.runtime-boundaries
cargo nextest run --locked --workspace --all-targets --profile ci
cargo nextest run --locked --workspace --all-targets
cargo xtask ci --full
cd ui && bun ci
cd ui && bun run check
cd ui && bun run lint
cd ui && bun run typecheck
cd ui && bun run --bun test:ci
cd ui && bun run build
cd ui && bun run test:browser
cd ui && bun run test:browser:cross
cd ui && bun run test:browser:a11y
cd ui && bun run test:browser:visual
cd ui && bun run test:browser:full
cd ui && bun run perf:live
cargo xtask ui-bundle analyze
cargo xtask ui-bundle build-twice
mise exec -- actionlint
```

The default Rust commands must work from a clean checkout without `ui/dist`.
The dedicated embed partition builds the UI before compiling `embed-ui`.
Long-running engine commands must narrate progress and finish with the required
ready banner.

## Trigger Ledger Without A Plan

These observations are not currently executable work. Reopen them as numbered
plans only when the trigger becomes true:

| Observation | Reopen trigger |
|-------------|----------------|
| `is_missing_table` / `is_missing_column` use substring matching | A GreptimeDB upgrade breaks conformance or exposes structured errors |
| Pre-commit hooks / `.editorconfig` are absent | Operator selects a repository-wide local-hook policy |
| Bench compose pins lag current engines | The next required four-build benchmark run |
| Native log schema may drift | Every GreptimeDB engine upgrade; compare a fresh native `SHOW CREATE TABLE` before release |
| Old native-table indexes need backfill | A supported old install shows unindexed SST query regression; live-test `ADMIN build_index_table` first |
| Trace native table defaults to 16 partitions | A supported at-scale profile exists; rerun the 1-vs-16 partition harness before changing fresh-table hints |
| Raw forward leg is HTTP-only | GreptimeDB restores a supported native OTLP gRPC ingest endpoint with a native-TLS-compatible client path |
| Profiles are not ingested | GreptimeDB ships a native OTLP profile table/path and the operator opens profile scope |
| ExponentialHistogram support | The signal appears in supported SDK traffic and Greptime native handling is verified |
| Broader newtype rollout | The single ID pilot proves value without wire/persistence churn |
| Stable Homebrew formula mutation | Stable-release readiness is explicitly opened |
| External broker / Iggy | A supported server profile proves the current spool/in-process design cannot meet an approved replay/isolation SLO and the operator opens broker scope |

Accepted decisions such as native tables, no rustls, no Node, no docs site,
and no automatic update branches are repository policy, not unfinished plans.
