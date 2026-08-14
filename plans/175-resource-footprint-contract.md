# Plan 175: The resource-footprint contract — measured, published, regression-gated

> **Executor instructions**: Follow this plan step by step. Run every
> verification command. On any "STOP conditions" item, stop and report.
>
> **Drift check (run first)**: `git diff --stat 7418bc9..HEAD -- bench/ crates/parallax-server/src/config.rs crates/parallax-server/src/greptime_supervisor.rs .github/workflows/ docs/guide/ README.md`
> — on mismatch with the excerpts below, STOP.

## Status

- **Priority**: P2
- **Effort**: M
- **Risk**: LOW-MED (measurement infra; the risk is unstable CI numbers —
  handled with generous ceilings + same-runner-class pinning)
- **Depends on**: none
- **Category**: perf
- **Planned at**: parallax `7418bc9`, 2026-08-14
- **Evidence base**: `docs/research/market/competitor-pain-points.md` —
  idle/runaway resource hunger is a top OSS-challenger complaint class:
  SigNoz forces a ClickHouse cluster + ~800MB-idle ZooKeeper on single
  node (SigNoz#8784, #7002; HN 45293788 "sloppily built"), 9GB collector
  RAM (#6128), OOMKills (#9306), UI-open saturating ClickHouse (#10590);
  Coroot OOM loop (#18); Sentry self-host needs 16GB minimum (186-pt HN
  thread); Datadog agent overhead "leading to outages" (datadog-agent#3793).

## Why this matters

"Runs on the box you have" is a load-bearing part of Parallax's claim
(local-first single binary, dev-machine profile), and resource hunger is
one of the most-cited reasons users abandon each researched competitor.
Today Parallax makes the claim with zero measured numbers and zero
regression protection — nothing stops a dependency bump or a new
background worker from doubling idle RSS silently. Correctness framing:
an unmeasured claim is an unproven claim (the repo's own research rules
mark benchmark-dependent cells ⚪ unproven until measured). This plan
makes footprint a measured, published, regression-gated contract — the
same move plan 174 makes for durability.

## Current state (verified)

- Process model: `parallax serve` = one Rust binary + one supervised
  GreptimeDB child (`crates/parallax-server/src/greptime_supervisor.rs`,
  ports 24000–24003, checksum-pinned download); Turso is in-process. So
  "footprint" = RSS(parallax) + RSS(greptime child) + disk(data dir).
- Existing bench infra: `bench/` holds the fan-out lab and comparison
  scaffolding (`PROJECT_STRUCTURE.md`: "Reproducible local benchmark and
  comparison-lab scaffolding"); the GreptimeDB-vs-ClickHouse program has
  its own protocol (AGENTS.md four-build rule) — THIS plan is not that
  program: no cross-engine comparisons here, only Parallax's own numbers.
- No footprint measurement, contract file, or CI gate exists
  (`grep -rn "rss\|footprint" bench/ .github/workflows/` — verify at
  execution; expected: no product hits).
- Progress-visibility rule (AGENTS.md): long-running CLI steps narrate —
  the measurement script must too.

## Commands you will need

| Purpose | Command | Expected |
|---|---|---|
| Gates | `cargo xtask ci --fast && cargo xtask lint` | green |
| Build release binary | `cargo build --release -p parallax-cli` | binary at `target/release/parallax` |
| Run measurement | `bench/footprint/measure.sh` (created by this plan) | JSON report printed + written |
| Docs links | `cargo xtask docs links` | pass |

## Scope

**In scope**: new `bench/footprint/` (measure script + scenario driver +
`contract.toml` ceilings + README), a CI workflow job, publication of the
measured numbers in `docs/guide/` (quickstart or a new
`footprint.md`) and the README claim line, `PROJECT_STRUCTURE.md` note if
`bench/` gains the subdir (structure table already covers `bench/`).

**Out of scope**: any performance OPTIMIZATION (findings become
`DISCREPANCY:`/plan-166 items or new plans); the GreptimeDB-vs-ClickHouse
four-build benchmark program (separate protocol); load/throughput
benchmarking beyond the light steady scenario below.

## Git workflow

PR-only `main`; one branch, one PR (or two: harness, then CI+docs);
`git commit -s`; Conventional Commits; agent trailer per `COMMITS.md`.

## Steps

### Step 1: Measurement harness

`bench/footprint/measure.sh` (portable bash, Linux + macOS): builds
release binary, starts `parallax serve` with a scratch HOME (child env,
never mutating the operator's), waits for the ready banner, then samples
at three phases: (a) **idle-after-start** (60s post-ready, no traffic),
(b) **light steady** (a fixed telemetrygen-style OTLP feed — reuse the
lab's telemetrygen invocation shape from `bench/otlp-fanout/compose.yml`
at a pinned small rate for 120s), (c) **post-ingest idle** (60s after
stopping traffic). Per phase record: RSS of parallax process, RSS of the
greptime child (find via the supervisor's pidfile in the data dir), CPU%,
data-dir bytes. Output: human table (narrated) + `report.json`. Samples
via `ps`/`/proc` on Linux and `ps` on macOS — no new dependencies.

**Verify**: script runs locally end-to-end, exits 0, report.json has all
3 phases × 4 metrics, and re-running twice yields RSS within ±15%
(record both runs).

### Step 2: Contract ceilings

`bench/footprint/contract.toml`: ceilings per phase/metric, set from the
Step-1 measurement plus generous headroom (2× the observed value initially
— the gate's job is catching regressions-by-integer-factor, not noise;
tighten later deliberately). A small checker (`check.sh` or a `--check`
mode) compares report.json to the contract, exit 1 on breach, printing
metric, ceiling, observed.

**Verify**: `measure.sh && check.sh` → exit 0; tamper the contract to an
impossible ceiling → exit 1 naming the metric.

### Step 3: CI job

Path-aware workflow job on a pinned runner class (`ubuntu-latest` — note
runner variance risk in the job's comment): runs measure+check on PRs
touching `crates/` or `Cargo.lock`. Non-blocking (warn) for its first two
weeks of history, then flip to required — record the flip date in the
workflow comment; the operator flips it.

**Verify**: job runs on a test PR; report.json uploaded as artifact.

### Step 4: Publish the numbers

- `docs/guide/footprint.md` (or a section in quickstart): the measured
  table (idle/steady RSS for binary + engine, disk growth per the fixed
  feed), the hardware/runner it was measured on, the date, and the
  contract ceilings — presented per the repo's honesty rules (numbers are
  claims only with source + date; unmeasured stays unmeasured).
- README "Using It" line gains the headline number with a link.
- Refresh `docs/research/market/competitor-pain-points.md` mapping row
  (footprint → measured, plan delivered).

**Verify**: `cargo xtask docs links` pass; every published number matches
report.json committed alongside (or linked CI artifact).

## Test plan

The harness is self-testing (Step 1 repeatability check, Step 2 breach
check). No unit tests beyond the checker's own breach case.

## Done criteria

- [x] `bench/footprint/` harness runs on Linux + macOS, narrated, scratch
      HOME only.
- [x] `contract.toml` ceilings + checker; tampered ceiling fails.
- [x] CI job wired path-aware with artifact upload; flip-to-required date
      recorded.
- [x] Numbers published with date + hardware in docs/guide + README line.
- [x] Targeted gates green (docs links + lint). Full `ci --fast` waits
      for 162–176 close-out.
- [x] `plans/README.md` row updated.

## STOP conditions

1. Drift check fails.
2. CI runner variance exceeds ±30% across three runs — report with data;
   the contract may need a self-hosted runner decision (operator call),
   not silently looser ceilings.
3. The measurement reveals footprint an integer factor above the product
   claim's spirit (e.g. multi-GB idle) — that's a real finding: record
   `DISCREPANCY:`, report, and do NOT publish marketing numbers until
   triaged.
4. macOS sampling can't attribute the greptime child reliably — report
   the pid-discovery gap; Linux-only CI with macOS manual is acceptable
   only as a documented interim.

## Maintenance notes

- Dependency bumps and new background workers are exactly what the gate
  exists to catch — a breach on such a PR is a finding, not an excuse to
  raise the ceiling; ceiling raises need the measured justification in
  the PR.
- When the V2 server profile lands, add a server-profile scenario row —
  same harness, second contract section.
- The four-build storage benchmark program remains separate; never merge
  the two (different questions: own-footprint vs engine comparison).
