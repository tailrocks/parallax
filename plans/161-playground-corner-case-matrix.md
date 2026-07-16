# Plan 161: Build the playground corner-case corpus — one scripted scenario per UI rendering risk

> **Executor instructions**: This plan edits the companion repository
> `tailrocks/parallax-telemetry-playground` on its
> `main` branch (direct-to-main delivery model), after plan 158. Follow step by
> step; every scenario gets a stable id, a runner script, and a documented
> expected rendering. Honor STOP conditions; update this plan's status row in
> `plans/README.md` (Parallax repo) when done.
>
> **Drift check (run first, in the playground checkout)**: plan 158's commits
> must be present on the branch (`git log --oneline -15`); the neutral-key
> emitters are this plan's substrate. If absent, STOP.

## Status

- **Priority**: P1
- **Effort**: M
- **Risk**: LOW (emitter-side additions only)
- **Depends on**: plans/158-playground-unified-cli-contract.md
- **Category**: tests / corpus
- **Planned at**: Parallax `39f172c` (+ same-day restructuring commits), 2026-07-17
- **Operator directive (2026-07-17)**: the playground must contain enough
  corner cases that every Parallax UI surface can be *proven* correct or
  *shown* broken from real data — parts of the current UI render incorrectly
  (span display inside traces is the known-worst area) and gaps only become
  visible when a scenario exercises them. Lands as direct commits to the
  playground's `main` beside plan 158's changes (operator delivery model:
  no branches, no pull requests).

## Why this matters

Plans 160 (UI defect audit & repair) and 159 (live acceptance) verify Parallax
surface by surface. Verification is only as strong as the corpus: a waterfall
that never sees a 500-span trace, an orphan span, or a clock-skewed pair will
pass every walk while staying broken for real users. This plan turns known
rendering risks into deterministic, replayable scenarios with stable ids, so
"the UI displays everything correctly" becomes a checkable matrix instead of
an impression.

## Current state

(playground `main` after PR #6 + plan 158 on the branch)

- Scenario infrastructure exists: `scenarios/run.sh <id>` fires stories
  (listing with no args); chaos flags in `flags/flagd.json` (paymentFailure,
  slowQuery, cacheLeak, poisonMessage, canaryFailure, catalogPromo); orders
  poison/dead-letter, batch fan-in with many links, orphan (linkless)
  variant already exist (`services/orders/src/main.rs:70-188`); k6 demo
  profile; limits overlay for OOM (`deploy/docker-compose.limits.yml`).
- Plan 158 added: CLI modes emitting `cli.invocation.id`/`app.mode`/
  `cli.command.name`/`outcome`, `console` interactive sim (`session.start/end`,
  screen visits, `ui.action` roots), daemon `background.cycle` loop,
  `job.id`/`job.type` on orders + fulfillment Kafka legs, richer `gen_ai.*`.
- Cross-language exception shapes exist (Rust anyhow-style, Java stack
  traces, TS/browser) but are not systematically triggered per scenario.

## The scenario matrix (build exactly these; ids are stable API)

Grouped by the Parallax surface they exercise. Each scenario is a
`scenarios/run.sh <id>` entry (or a documented CLI-sim flag) that runs
against the standing stack, is deterministic in SHAPE (counts/structure; ids
and timestamps vary), and completes in < 60 s unless noted.

**Trace anatomy (waterfall / span tree / minimap — the operator's known-bad
area):**
- `t-deep` — one trace, linear chain depth ≥ 12 across ≥ 3 services.
- `t-wide` — one trace with ≥ 500 spans (fan-out burst) — virtualization,
  minimap sampling, "whole trace" row.
- `t-multiroot` — one trace id containing 2 root spans (legal OTel; renderer
  must not lose either).
- `t-orphan` — spans whose parent span id never arrives (broken tree; must
  render as detached, not vanish).
- `t-skew` — CLIENT/SERVER pair where the child starts before the parent
  (clock skew; skew banner + non-negative bars).
- `t-zero` — zero-duration spans and spans with identical start/end at
  microsecond resolution.
- `t-links` — span links across two traces (link navigation both ways) plus
  the existing orders batch fan-in (many links on one span).
- `t-longnames` — span/attribute names and values at 1–4 KiB, unicode + emoji
  (layout, truncation, copy).
- `t-events` — a span with ≥ 50 span events including multi-line
  `exception.stacktrace` in Rust, Java, and browser-JS shapes.

**Protocol reconstruction:**
- `p-grpc-err` — gRPC leg returning each of: OK, INVALID_ARGUMENT,
  DEADLINE_EXCEEDED (client timeout), UNAVAILABLE (server down mid-call).
- `p-grpc-stream` — streaming RPC with per-message events (rpc-stream panel).
- `p-graphql-err` — storefront GraphQL: field error with partial data, and a
  request-level error (graphql-operation panel + errors surface).
- `p-kafka-lag` — fulfillment consumer paused N seconds (PRODUCER/CONSUMER
  gap rendering, job attempt latency), then dead-letter path.

**Logs:**
- `l-burst` — ≥ 5k logs in 30 s across severities (live tail caps, histogram).
- `l-bodies` — structured JSON bodies, 32 KiB body, ANSI escapes, blank body,
  identical-timestamp runs (ordering stability).

**Metrics:**
- `m-shapes` — counter reset mid-window, gauge with gaps, exponential-vs-
  explicit histogram, exemplar-bearing histogram (exemplar deep-link).

**Errors / issues:**
- `e-burst` — one error type at 100×/min for grouping/trend; plus 5 distinct
  `error.type` values in one invocation (errors-tab breakdown).
- `e-multi-lang` — same logical failure surfacing in Rust, Java, and browser
  fingerprints (issue list disambiguation).

**CLI journey (feeds the plan-157 journey view):**
- `j-happy` — console sim: home → cart → checkout, all actions succeed.
- `j-error` — console sim where the `checkout.submit` action fails
  server-side: error must attribute to the `checkout` screen and the
  submitting widget (`app.widget.*` set).
- `j-outside` — an error fired between screen visits (unattributed bucket).
- `j-reattach` — session end → new session with `session.previous_id` chain
  (≥ 3 links).
- `j-parallel` — 3 concurrent invocations of the same CLI binary (distinct
  invocation ids, interleaved signals; list + hub isolation) while the daemon
  sim also runs (4 concurrent correlation domains).

**Ecosystem:**
- `eco-full` — one pass touching every edge: browser → checkout → pricing/
  inventory/recommendation, storefront → catalog/payment, fulfillment Kafka →
  payment/notifications, CLI → checkout (kinds cli/browser/service all
  present).

## Commands you will need

| Purpose | Command | Expected on success |
|---|---|---|
| List scenarios | `scenarios/run.sh` | prints every id above with one-line description |
| Run one | `scenarios/run.sh t-wide` | exit 0; narrated progress; prints the invocation/trace ids it minted |
| Corpus sweep | `scenarios/run.sh --all-corner-cases` (new) | runs every id sequentially, exit 0, prints a summary table |
| Tests | `cargo nextest run --workspace --profile ci --no-tests=fail` | pass |

## Scope

**In scope (playground repo):** `scenarios/**` (new runner entries + the
`--all-corner-cases` sweep + per-scenario docs), small emitter hooks needed
by scenarios (a fault flag, a delay flag, a span-shape generator under
`libs/playground-telemetry` or per-service), `flags/flagd.json` additions,
`docs/corner-case-matrix.md` (the scenario→surface→expected-rendering table),
CLI sim flags (`console --fail-at checkout.submit`, `--reattach`,
`--parallel N`).

**Out of scope:** Parallax-side changes (160 owns fixes), new services/infra,
load testing beyond `l-burst`/`t-wide` shapes, multi-backend concerns.

## Git workflow

- Work directly on the playground's `main` (no branches, no PRs);
  Conventional Commits, DCO `-s`, one agent trailer, push per durable green
  commit. Suggested:
  `feat(scenarios): corner-case corpus for parallax ui verification`.

## Steps

### Step 1: Matrix doc first

Write `docs/corner-case-matrix.md`: one row per scenario id — trigger
command, signals emitted (shape counts), target Parallax surface, expected
rendering, "known-broken?" column (filled by plan 160). This doc is the
contract plans 159/160 execute against.

**Verify**: every id above has a row; doc lints (repo markdown checks if any).

### Step 2: Trace-anatomy + protocol scenarios

Implement `t-*` and `p-*`. Prefer real service paths (flags/delays) over
synthetic span factories; where a shape cannot come from real flow
(`t-multiroot`, `t-skew`, `t-zero`), add a small `span-shapes` generator in
the CLI sim that emits hand-built OTLP via the shared lib — clearly labeled
`service.name=playground-shapes`.

**Verify**: each scenario exit 0 and prints its trace ids; a nextest per
generator asserting the emitted structure (root count, span count, link
count, skew sign).

### Step 3: Logs, metrics, errors scenarios

Implement `l-*`, `m-*`, `e-*` via flags + generators.

**Verify**: scenario exit 0; generator tests assert counts/shapes.

### Step 4: Journey + ecosystem scenarios

Implement `j-*` (console-sim flags) and `eco-full`.

**Verify**: `j-error` test asserts the error event timestamp falls inside the
checkout screen visit and carries `app.widget.*`; `j-parallel` asserts 3
distinct invocation ids; `eco-full` asserts one signal per expected edge.

### Step 5: Sweep runner + docs closure

`--all-corner-cases` sequential runner with summary table; README section
pointing at the matrix doc.

**Verify**: full sweep exit 0 against a live Parallax on the Docker host;
matrix doc row count equals runner id count.

## Test plan

Per-generator structural tests (steps 2-4); the sweep run itself is the
integration test; plan 159 consumes the ids for acceptance assertions and
plan 160 walks each id's target surface.

## Done criteria

- [ ] `scenarios/run.sh` lists every matrix id; `--all-corner-cases` exits 0.
- [ ] `docs/corner-case-matrix.md` complete (id → trigger → signals →
  surface → expected rendering).
- [ ] Generator/structural tests green in the workspace suite.
- [ ] `j-error` attribution shape proven by test (timestamp inside visit +
  widget attrs).
- [ ] `plans/README.md` (Parallax) status row updated.

## STOP conditions

Stop and report back (do not improvise) if:
- A shape cannot be emitted through the OTel SDKs at all (e.g. the Rust SDK
  refuses multi-root export) — document the SDK limitation in the matrix doc
  instead of faking it post-export.
- A scenario needs Parallax-side changes to be observable — that belongs to
  plan 156/160; record the gap in the matrix "known-broken?" column.
- `t-wide`/`l-burst` destabilize the stack on the dev host — halve the counts
  and record the effective numbers in the matrix; do not delete the scenario.

## Maintenance notes

- Every future UI rendering bug gets a scenario id here BEFORE its fix lands
  in Parallax (regression corpus discipline).
- Scenario ids are stable API for plans 159/160 and future CI — never rename;
  add new ids instead.
