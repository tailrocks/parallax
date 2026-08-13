# Plan 164: Extend the playground until every Parallax feature has a scripted scenario

> **Executor instructions**: Follow this plan step by step. Run every
> verification command and confirm the expected result before moving on. If
> anything in "STOP conditions" occurs, stop and report — do not improvise.
> When done, update this plan's row in `plans/README.md` (parallax repo).
>
> **Drift check (run first)**:
> `git -C ../parallax-telemetry-playground diff --stat 6e0a0d5..HEAD -- scenarios/ cli/ VERIFICATION.md TOUR.md docs/`
> and `git diff --stat f6208070..HEAD -- docs/research/reference/feature-inventory-and-playground-verification.md`.
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

- Playground scenarios live in `../parallax-telemetry-playground/scenarios/`
  (51 `*.sh` scripts + `README.md` catalog + `run.sh <id>` runner). Catalog
  README has a "Check in Parallax UI" column per scenario. Naming: `aN-*`
  feature proofs, `b*-`/breach chaos scripts (`a-breach-error-rate.sh`,
  `a-breach-p95.sh`, `a-recover.sh` drive alert-shaped conditions but no
  scenario asserts Parallax *alert rule → incident → delivery*).
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
  - MCP: `parallax-mcp` stdio, tools `parallax_issue_context`,
    `parallax_agent_session_show`; `parallax-mcp check` proves projection
    equivalence.
  - Sentry envelope ingest `POST /api/<project_id>/envelope/` and GitHub
    webhooks `POST /webhooks/github` — both **disabled by default** in
    `crates/parallax-server/src/config.rs` (`[sentry]`, `[github]` blocks);
    scenarios must enable them via `~/.parallax/config.toml`.
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
| MCP equivalence | `parallax-mcp check` | exit 0 |
| Webhook capture | `python3 -m http.server 9099` (or a tiny sink script in the scenario) | delivery POST logged |

## Scope

**In scope** (playground repo): new `scenarios/c*-*.sh` scripts (new `c`
series = "Parallax product-surface journeys" — keep `a`/`b` semantics
untouched), `scenarios/README.md` catalog rows, `fixtures/` additions (GitHub
webhook payloads, Claude Code NDJSON sample), `VERIFICATION.md` new sections,
`TOUR.md` pointers, `docs/corner-case-matrix.md` rows.
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
assertion failure. Register each in `scenarios/README.md` in the same commit.

### Step 1: `c1-issue-context.sh` — evidence bundle + agent handoff

Trigger a known error (reuse `a-breach-error-rate.sh` machinery), wait for
the issue, then: `parallax issue list --format json` → pick fingerprint →
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
HMAC secret. Script: enable `[github]` in a scratch `~/.parallax/config.toml`
(documented inline), restart serve, POST fixtures with correct
`X-Hub-Signature-256`, assert 2xx + idempotent re-POST, then GraphQL
`releases`/deploy-linked queries reflect the deploy. A bad-signature POST
must be rejected.

**Verify**: `./run.sh c6` → exit 0.

### Step 7: `c7-agent-session.sh` — Claude Code import + MCP

Add `fixtures/claude-code/session.ndjson` (small synthetic stream-json
session — structural events only, no real prompts). Script:
`parallax import-claude fixtures/claude-code/session.ndjson --json` →
assert imported id; GraphQL `agentSession` returns the timeline;
`parallax-mcp check` exits 0; call `parallax_agent_session_show` through a
stdio MCP client one-liner and assert sanitized output.

**Verify**: `./run.sh c7` → exit 0.

### Step 8: `c8-sentry-envelope.sh` — envelope ingest parity

Enable `[sentry]` mapping in the scratch config; point ONE service's
`SENTRY_DSN` at Parallax (`http://host.docker.internal:4000/api/<project>/…`)
instead of real Sentry; trigger an error; assert the issue appears in
`parallax issue list` with the same `error.type` as the OTLP-derived issue
(documents the multi-SDK ledger gap: run once each for a Rust, Java, and web
DSN).

**Verify**: `./run.sh c8` → exit 0.

### Step 9: `c9-lifecycle-ops.sh` — doctor, prune, forward, self-telemetry

`parallax doctor` asserts healthy (exit 0); `parallax prune` (dry-run)
prints a plan and mutates nothing (issue counts unchanged); `parallax prune
--execute --yes` on a scratch data dir removes only expired classes; a
pinned evidence bundle survives prune (pin via investigation from c5).
`source bench/otlp-fanout/lab.env && parallax invocation start -- <cmd>`
fans the child's telemetry to the lab (assert arrival at ≥2 backends);
`PARALLAX_SELF_OTLP` makes `service.name=parallax` spans appear in the lab.

**Verify**: `./run.sh c9` → exit 0.

### Step 10: `c10-redaction-egress.sh` — canary corpus on every egress

Extend `a18-canary.sh`'s corpus use: after seeding canary secrets (fake,
detector-shaped values only), assert none of the 20 detector patterns'
*seeded canaries* appear in: `issue context` markdown+json, GraphQL `bundle`,
MCP `parallax_issue_context` output, or the SSE stream. Grep-based assert
using the canary values the corpus defines (never real secrets).

**Verify**: `./run.sh c10` → exit 0.

### Step 11: Catalog + docs

`scenarios/README.md`: new c-series section with one row per script (id,
proves, Parallax assert, "Check in UI" per backend). `VERIFICATION.md`: new
"Parallax product-surface journeys (c-series)" section with the run record.
`TOUR.md`: one pointer line. parallax inventory doc Workstream 3 list:
annotate each bullet with its scenario id. `docs/corner-case-matrix.md`:
rows for any UI-rendering risks the new scenarios revealed.

**Verify**: parallax `cargo xtask docs links` → passes; every c-script has a
README row (`for f in scenarios/c*.sh; do grep -q $(basename $f .sh) scenarios/README.md || echo MISSING $f; done` → no output).

## Test plan

The scenarios ARE the tests. Full sweep: `for id in c1 c2 c3 c4 c5 c6 c7 c8
c9 c10; do ./run.sh $id || echo FAIL $id; done` → no FAIL lines against the
plan-163 stack. Failures caused by Parallax defects (not script bugs) are
*expected output*, not plan failure: record each in `VERIFICATION.md` with a
`DISCREPANCY` marker and keep the script's assert honest (red) — plan 166
consumes these.

## Done criteria

- [ ] Ten c-series scripts exist, are executable, registered in
      `scenarios/README.md`, and each asserts a Parallax surface machine-checkably.
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
3. Enabling `[sentry]`/`[github]` requires config keys not documented in
   `crates/parallax-server/src/config.rs` — report, don't reverse-engineer.
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
