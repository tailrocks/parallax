# Plan 164: Extend the playground until every Parallax feature has a scripted scenario

> **Executor instructions**: Follow this plan step by step. Run every
> verification command and confirm the expected result before moving on. If
> anything in "STOP conditions" occurs, stop and report — do not improvise.
> When done, update this plan's row in `plans/README.md` (parallax repo).
>
> **Drift check (run first)**:
> `git -C ../parallax-telemetry-playground diff --stat 6e0a0d5..HEAD -- scenarios/ cli/ fixtures/ VERIFICATION.md TOUR.md docs/`
> and `git diff --stat f6208070..HEAD -- docs/research/reference/feature-inventory-and-playground-verification.md bench/otlp-fanout/README.md`.
> On mismatch with "Current state", STOP.

## Status

- **Priority**: P1
- **Effort**: L
- **Risk**: MED (new scenarios exercise Parallax surfaces that may expose real bugs — that is the point; the risk is scope creep into fixing them here)
- **Depends on**: plans/162-fanout-lab-backend-pins.md, plans/163-playground-example-upgrades.md
- **Category**: tests
- **Planned at**: parallax `f6208070`, playground `6e0a0d5`, 2026-08-13

## Why this matters

The verification program's exit criterion is "every feature in the inventory
exercised by at least one scripted scenario" (see
`docs/research/reference/feature-inventory-and-playground-verification.md`,
Workstream 3 and 5). The playground's ~51 scenario scripts cover telemetry
generation richly, but several Parallax product surfaces have **no scripted
scenario at all**: evidence bundles/MCP, alerting end-to-end, dashboards/
investigations/saved-views/SQL journeys, GitHub deploy+CI ingest, Claude Code
session import, live tail, prune/doctor, `--otlp-forward` compare mode,
self-telemetry, and redaction assertions on every egress surface. Until those
are scripted, "verified" means "someone clicked around once".

## Current state

- Playground scenarios live in `../parallax-telemetry-playground/scenarios/`:
  48 driver scripts + `run.sh` (the dispatcher) + `corner-cases.sh` (drives
  the `t-*`/`l-*`/`m-*`/`e-*`/`p-*`/`j-*`/`eco-*` corner-case IDs). Naming:
  `aN-*` feature proofs, `b*` chaos, plus a-prefixed breach helpers
  (`a-breach-error-rate.sh`, `a-breach-p95.sh`, `a-recover.sh`) that drive
  alert-shaped conditions — but no scenario asserts Parallax *alert rule →
  incident → delivery*.
  **`run.sh` dispatches ONLY hardcoded IDs**: a `catalog()` heredoc plus a
  `scenario()` case statement; an unregistered ID prints "Unknown scenario"
  and exits 2. `scenarios/README.md:3-5` states every new scenario must be
  added "here and in `run.sh`". Every c-series script in this plan MUST be
  registered in BOTH (catalog row + case arm) or its verify fails.
- Machine acceptance: `VERIFICATION.md` + playground CLI `test-verify`
  (JUnit→OTLP bridge; "90/90 ci nextest + w4 flaky fixtures + test-report"
  is the recorded PASS shape).
- Parallax surfaces to target (all shipped on `main`, inventory:
  `docs/research/reference/feature-inventory-and-playground-verification.md`):
  - CLI: `parallax issue list|context|resolve`, `invocation start|finish|
    inspect|bundle|agent|list|watch`, `logs`/`traces` `--follow --for`,
    `metrics --invocation`, `sql`, `doctor`, `prune` (dry-run default,
    `--execute --yes`), `import-claude <ndjson>`, `context add|use|…`.
  - GraphQL `POST 127.0.0.1:4000/graphql`: 76 queries / 14 mutations incl.
    `bundle(fingerprint|invocationId|traceId)`, `story`, `evidenceGaps`,
    `alertRuleSave`, `alertDestinationSave`, `dashboardSave`,
    `investigationSave`, `savedViewSave`, `sql`.
  - SSE live tail: `GET /v1/logs/stream`, `/v1/traces/stream`.
  - MCP: `parallax-mcp` stdio (not installed with `parallax`; run via
    `cargo run -p parallax-mcp --`), tools `parallax_issue_context`,
    `parallax_agent_session_show`. `serve` refuses to start without
    `--allow-local-stdio` (`crates/parallax-mcp/src/main.rs:62-64`);
    `check` requires `--fingerprint <fp>` and/or `--invocation-id <id>`
    (`crates/parallax-mcp/src/check.rs:40`) and proves MCP↔CLI↔GraphQL
    projection equivalence.
  - Sentry envelope ingest `POST /api/{project_id}/envelope/` (also without
    trailing slash; default `project_id` is `"1"` —
    `crates/parallax-server/src/sentry_http.rs:40-42`,
    `crates/parallax-server/src/config.rs:45`) and GitHub webhooks
    `POST /webhooks/github` — all **disabled by default**. Config blocks in
    `~/.parallax/config.toml`: `[sentry]` (envelope), and TWO separate GitHub
    blocks — `[github_deploy]` and `[github_actions]` — each with its own
    `enabled` + `webhook_secret` (`crates/parallax-server/src/config.rs:18-22`).
  - Alerting: rules (error_rate, p95/p99, throughput, log_count, metric),
    webhook + `slack_webhook` destinations, incidents, `alertChecks` audit.
  - Redaction: engine `redaction-lite-v3`, 20 detectors; playground already
    ships the canary corpus scenario `a18-canary.sh`.
- Parallax repo helpers: `bench/otlp-fanout/lab.env` (compare mode:
  `PARALLAX_OTLP_FORWARD`, `PARALLAX_SELF_OTLP`).

Scenario-script conventions (match them): each script is a small `bash -eu`
curl/CLI driver against the compose stack with a header comment naming the
scenario id, what it proves, and where to check; registered as a row in
`scenarios/README.md`; destructive ones warn first (see the OOM script).

## Commands you will need

| Purpose | Command | Expected on success |
|---|---|---|
| Stack up | plan-162/163 procedure (lab + compose) | all up |
| Run one scenario | `cd ../parallax-telemetry-playground/scenarios && ./run.sh <id>` | script exit 0 |
| Parallax GraphQL probe | `curl -s localhost:4000/graphql -H 'content-type: application/json' -d '{"query":"{ health }"}'` | `{"data":{"health":...}}` |
| SSE probe | `curl -N "localhost:4000/v1/logs/stream?service=<svc>" & sleep 5; kill %1` | streamed rows during traffic |
| MCP equivalence | `cargo run -p parallax-mcp -- check --fingerprint <fp> --invocation-id <id>` (run from the parallax repo; at least one anchor flag is required) | exit 0, all equivalence cases pass |
| Webhook capture | `python3 -m http.server 9099` (or a tiny sink script in the scenario) | delivery POST logged |
| Scenario registration check | `cd scenarios && ./run.sh c1` after registering | dispatches (no "Unknown scenario") |

## Scope

**In scope** (playground repo): new `scenarios/c<N>-*.sh` scripts (new `c`
series = "Parallax product-surface journeys" — keep `a`/`b` semantics
untouched; note the pre-existing `scenarios/corner-cases.sh` is NOT part of
the series — never glob it as `c*`), `scenarios/run.sh` (catalog rows + case
arms for every c-scenario), `scenarios/README.md` catalog rows, `fixtures/`
additions (GitHub webhook payloads, Claude Code NDJSON sample),
`VERIFICATION.md` new sections, `TOUR.md` pointers,
`docs/corner-case-matrix.md` rows.
**In scope** (parallax repo): `docs/research/reference/feature-inventory-and-playground-verification.md`
(tick coverage), `bench/otlp-fanout/README.md` (mention c-series), and — if a
scenario needs it — a documented `~/.parallax/config.toml` example block in
playground docs (NOT in parallax config code).

**Out of scope**:
- Fixing any Parallax bug a scenario exposes (that is plan 166 — file the
  discrepancy, keep the scenario red-flagged in VERIFICATION.md).
- Parallax product code `crates/`, `ui/` — zero edits.
- Comparison scoring across backends (plan 165).
- New playground *services*; the execution-stack expansion idea in
  `docs/research/architecture/full-observability-ui-and-playground-research.md`
  stays historical brainstorming.

## Git workflow

PR-only `main`, one branch + one PR per repo, `git commit -s`, Conventional
Commits, agent trailer per `COMMITS.md`.

## Steps

Each step = one new scenario (or tight cluster). Every script must: run
against the live stack, drive the condition, then **assert the Parallax
surface via CLI or GraphQL** (machine check, not prose), print a
`Check in <backend> UI` line per backend for plan 165, and exit non-zero on
assertion failure. Register each in **three places in the same commit**:
`scenarios/README.md` row, `scenarios/run.sh` `catalog()` row, and
`scenarios/run.sh` `scenario()` case arm — an unregistered ID makes
`./run.sh cN` exit 2 with "Unknown scenario".

### Step 1: `c1-issue-context.sh` — evidence bundle + agent handoff

Trigger a known error (reuse `a-breach-error-rate.sh` machinery), wait for
the issue, then obtain the fingerprint via GraphQL (the CLI's `issue list`
has NO `--format` flag — only `--status`/`--invocation`,
`crates/parallax-cli/src/main.rs:265-274`):
`curl -s localhost:4000/graphql -H 'content-type: application/json' -d '{"query":"{ issues(status:\"open\", limit:1){ fingerprint } }"}'`
(adjust arg names to `ui/graphql/schema.graphql` if they differ — STOP
condition 2 applies, don't guess beyond the SDL). Then
`parallax issue context <fp> --format json` → assert `bundle-v1` schema
fields (`schema`, canonical hash, `trace`, `logs`, `metric_windows`,
`missing_evidence`), then GraphQL `bundle(fingerprint:…)` returns the same
canonical hash, then `parallax issue resolve <fp>` flips status.

**Verify**: `./run.sh c1` → exit 0.

### Step 2: `c2-invocation-lifecycle.sh` — wrapper, bundle, watch, metrics

`parallax invocation start -- <playground cli command>` (wrapped), then
`invocation list/inspect/bundle/agent`, `metrics --invocation <id>`, and a
bounded `invocation watch <id> --for 30s` during traffic. Assert exit-code
propagation (run a failing child too) and that `bundle` anchors on the
invocation.

**Verify**: `./run.sh c2` → exit 0.

### Step 3: `c3-live-tail.sh` — SSE + CLI follow

During `a1-checkout.sh` traffic: `parallax logs --service checkout --follow
--for 20s` captures ≥1 matching row; `curl -N /v1/logs/stream` and
`/v1/traces/stream` each deliver ≥1 row; a filtered stream
(`?service=nonexistent`) delivers 0.

**Verify**: `./run.sh c3` → exit 0.

### Step 4: `c4-alerting.sh` — rule → incident → delivery

Start a local webhook sink (inline python in the script, port 9099). Via
GraphQL: `alertDestinationSave` (webhook → sink), `alertRuleSave` (error_rate
rule scoped to `checkout`, small window, low threshold). Drive
`a-breach-error-rate.sh`; poll `alertIncidents` until open; assert sink
received a delivery; drive `a-recover.sh`; poll incident resolved; check
`alertChecks` audit rows exist. Clean up rule+destination via delete
mutations.

**Verify**: `./run.sh c4` → exit 0.

### Step 5: `c5-saved-state.sh` — dashboards, investigations, saved views, SQL

GraphQL journey: `dashboardSave` (widget on a playground metric) → query
`dashboard` returns it; `investigationSave` + pin a trace + note → read back;
`savedViewSave` for a logs view → read back; `sql("SELECT count(*) FROM
opentelemetry_logs")` returns a count; snippets survive server restart
(restart `parallax serve`, re-read). Delete everything created.

**Verify**: `./run.sh c5` → exit 0.

### Step 6: `c6-github-ingest.sh` — deploy + CI webhooks (fixtures)

Add `fixtures/github/` sample payloads (`deployment`, `deployment_status`,
`workflow_job` — model on GitHub's documented payload shapes) with a shared
HMAC secret (fixture-only value). Script: enable BOTH `[github_deploy]` and
`[github_actions]` blocks (`enabled = true`, `webhook_secret = "<fixture
secret>"` each — `crates/parallax-server/src/config.rs:18-22`) in the
scratch config, restart serve, POST fixtures with correct
`X-Hub-Signature-256`, assert 2xx + idempotent re-POST. Then assert via
GraphQL `releases(service:"checkout", fromNanos:"<window start>",
toNanos:"<window end>")` (all three args required —
`ui/graphql/schema.graphql:623`) that the deploy window reflects the
fixture. A bad-signature POST must be rejected (non-2xx).

**Verify**: `./run.sh c6` → exit 0.

### Step 7: `c7-agent-session.sh` — Claude Code import + MCP

Add `fixtures/claude-code/session.ndjson` (small synthetic stream-json
session — structural events only, no real prompts). Script:
`parallax import-claude fixtures/claude-code/session.ndjson --json` →
capture the returned JSON and extract the invocation/session id field it
contains (STOP if the output carries no id). GraphQL
`agentSession(invocationId:"<id>")` returns the timeline
(`ui/graphql/schema.graphql:673`). MCP equivalence:
`cargo run -p parallax-mcp -- check --invocation-id <id>` → exit 0 (this
exercises `parallax_agent_session_show` and proves MCP output ≡ CLI ≡
GraphQL — no hand-rolled stdio client needed).

**Verify**: `./run.sh c7` → exit 0.

### Step 8: `c8-sentry-envelope.sh` — envelope ingest parity

Enable `[sentry]` mapping in the scratch config (default `project_id` is
`"1"` — `crates/parallax-server/src/config.rs:45`); point ONE service's
`SENTRY_DSN` at Parallax using the DSN form
`http://<public-key>@host.docker.internal:4000/1` (the SDK derives
`POST /api/1/envelope/` from it — route at
`crates/parallax-server/src/sentry_http.rs:40-42`; the public key must match
the `[sentry]` mapping) instead of real Sentry; trigger an error; assert the
issue appears via the GraphQL `issues` query with the same `error.type` as
the OTLP-derived issue (documents the multi-SDK ledger: run once each for a
Rust, Java, and web DSN).

**Verify**: `./run.sh c8` → exit 0.

### Step 9: `c9-lifecycle-ops.sh` — doctor, prune, forward, self-telemetry

`parallax doctor` asserts healthy (exit 0); `parallax prune` (dry-run)
prints a plan and mutates nothing (issue counts unchanged). Destructive
prune ONLY against an isolated data dir: the CLI has no `--data-dir` flag
(`crates/parallax-cli/src/main.rs:160-169` — Prune takes only
`--execute/--yes/--json`) and the data dir comes from `[storage] data_dir`
(default `~/.parallax`, `crates/parallax-server/src/config.rs:221`), so run
the whole isolated pass under an overridden HOME:
`export HOME=$(mktemp -d)` in a subshell → `parallax serve` there, seed a
little telemetry + one pinned bundle (pin via investigation as in c5), stop,
then `HOME=<same> parallax prune --execute --yes` → expired classes removed,
the pinned bundle survives. NEVER run `--execute` with the real HOME in this
scenario.
`source bench/otlp-fanout/lab.env && parallax invocation start -- <cmd>`
fans the child's telemetry to the lab (assert arrival at ≥2 backends);
`PARALLAX_SELF_OTLP` makes `service.name=parallax` spans appear in the lab.

**Verify**: `./run.sh c9` → exit 0.

### Step 10: `c10-redaction-egress.sh` — canary corpus on every egress

Extend `a18-canary.sh`'s corpus use: read that script and extract the exact
fake canary values it seeds (fake email/token/card/jwt corpus — its shell
variables define them; they are detector-shaped fakes, never real secrets).
Store them in an array in the new script. After seeding, capture four egress
outputs to temp files: `parallax issue context <fp> --format markdown` and
`--format json`, the GraphQL `bundle(fingerprint:…)` response, the MCP
`check` output for the same fingerprint, and 10s of
`curl -N /v1/logs/stream`. Assert per canary value per file:
`grep -cF "<canary>" <file>` → `0`. Any non-zero = redaction leak, script
exits 1 listing surface + detector class (never echo the canary itself into
the failure message beyond its corpus label).

**Verify**: `./run.sh c10` → exit 0.

### Step 11: Catalog + docs

`scenarios/README.md`: new c-series section with one row per script (id,
proves, Parallax assert, "Check in UI" per backend). `scenarios/run.sh`:
catalog row + case arm per script (done incrementally in Steps 1–10 — audit
completeness here). `VERIFICATION.md`: new "Parallax product-surface
journeys (c-series)" section with the run record. `TOUR.md`: one pointer
line. parallax inventory doc Workstream 3 list: annotate each bullet with
its scenario id. `docs/corner-case-matrix.md`: rows for any UI-rendering
risks the new scenarios revealed.

**Verify**: parallax `cargo xtask docs links` → passes; every c-series
script is registered in README and run.sh (glob `c[0-9]*` deliberately
excludes `corner-cases.sh`; no scenario is executed by this check):
`for f in scenarios/c[0-9]*.sh; do id=$(basename $f .sh | cut -d- -f1); grep -q "$id" scenarios/README.md || echo MISSING-README $f; grep -q "$(basename $f)" scenarios/run.sh || echo MISSING-RUNSH $f; done`
→ no `MISSING-*` lines.

## Test plan

The scenarios ARE the tests. Full sweep: `for id in c1 c2 c3 c4 c5 c6 c7 c8
c9 c10; do ./run.sh $id || echo FAIL $id; done` → no FAIL lines against the
plan-163 stack. Failures caused by Parallax defects (not script bugs) are
*expected output*, not plan failure: record each in `VERIFICATION.md` with a
`DISCREPANCY` marker and keep the script's assert honest (red) — plan 166
consumes these.

## Done criteria

- [ ] Ten c-series scripts exist, are executable, registered in
      `scenarios/README.md` AND `scenarios/run.sh` (catalog + case arm), and
      each asserts a Parallax surface machine-checkably (`./run.sh c1` …
      `./run.sh c10` all dispatch — none exits 2).
- [ ] Fixtures added: `fixtures/github/*.json`, `fixtures/claude-code/session.ndjson`
      (synthetic, secret-free).
- [ ] Full c-sweep run recorded in `VERIFICATION.md` (PASS or DISCREPANCY per
      script — no UNTESTED).
- [ ] Every Workstream-3 bullet in the parallax inventory doc carries a
      scenario id.
- [ ] `cargo xtask docs links` passes (parallax).
- [ ] `plans/README.md` row updated.

## STOP conditions

1. Drift check fails.
2. A Parallax surface documented in the inventory doesn't exist at the live
   version (e.g. a mutation name differs from `ui/graphql/schema.graphql`) —
   report the mismatch; do not guess an alternative API.
3. Enabling `[sentry]`/`[github_deploy]`/`[github_actions]` requires config
   keys not present in `crates/parallax-server/src/config.rs` — report,
   don't reverse-engineer.
4. You find yourself editing `crates/` or `ui/` to make a scenario pass —
   that is plan 166's work; record the discrepancy instead.
5. Any fixture would need real credentials or non-synthetic session data.

## Maintenance notes

- c-series scripts double as regression tests for plan 166 fixes: re-run the
  failing script after each fix.
- Reviewer: check every script cleans up what it creates (rules,
  destinations, dashboards, scratch config) — leftover state poisons the
  next comparison run.
- Deferred: automated cross-backend scoring (kept manual by design — see
  playground README); browser-session/RUM product scenarios beyond a5/a28
  (Parallax has no browser-session product surface yet — inventory "gaps").
