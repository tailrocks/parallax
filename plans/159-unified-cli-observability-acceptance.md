# Plan 159: Prove the unified CLI observability slice live — playground fan-in, GraphQL assertions, browser evidence

> **Executor instructions**: Run this on a Docker-capable host (the operator's
> machine qualifies; Docker was confirmed available 2026-07-17). Follow step
> by step; every claim needs a command, a captured output, or a screenshot.
> Honor STOP conditions. When done, update this plan's status row in
> `plans/README.md` and store the evidence bundle under
> `docs/research/validation/2026-07-unified-cli-observability/`.
>
> **Drift check (run first)**: plans 156, 157, and 158 must be implemented on
> branch `feature/unified-cli-observability` (both repos) before this plan
> starts. `git log --oneline -10` on both branches must show their commits;
> otherwise STOP.

## Status

- **Priority**: P1
- **Effort**: M
- **Risk**: LOW (verification only; no product code changes except evidence
  docs)
- **Depends on**: plans/156-unified-cli-observability-contract.md,
  plans/157-cli-invocation-observability-ui.md,
  plans/158-playground-unified-cli-contract.md
- **Category**: tests / acceptance
- **Planned at**: commit `39f172c`, 2026-07-17

## Why this matters

The one-PR program (operator directive 2026-07-17) is done only when the
whole loop is observed working: playground microservices + browser + CLI
modes emit the neutral contract → Parallax ingests into GreptimeDB native
tables → GraphQL serves invocations/sessions/screens/actions/cycles/jobs →
the UI renders every surface live. Plan 154's earlier acceptance was blocked
on a Docker-less host; this plan covers the Parallax-backend arm of that
sweep on the operator's Docker host (the other four backends stay with plan
154). Green CI is evidence only for what tests exercise — this plan is the
live proof.

## Current state

- Parallax serve defaults: GraphQL/UI `127.0.0.1:4000`, OTLP gRPC `:4317`,
  OTLP HTTP `:4318` (`crates/parallax-server/src/config.rs:79-89`) — exactly
  the endpoints the playground's compose anchor expects via
  `host.docker.internal` (`deploy/docker-compose.yml:16-26`).
- Playground stack: 14 always-on containers + k6 demo profile; launch
  `docker compose -f deploy/docker-compose.yml up --build -d`; scenarios via
  `scenarios/run.sh <id>`; CLI sims `./target/debug/playground
  [drive|cron|daemon|console]` (plan 158 adds `console`).
- Long-running command rule (AGENTS.md): every long CLI step narrates
  progress; serve ends with a ready banner naming every surface — the
  banner is itself an asserted artifact here.
- Browser automation: the operator authorizes driving the UI with the
  available browser-automation tooling (Chrome DevTools MCP / agent browser)
  to capture evidence; screenshots are the durable artifact.

## Commands you will need

| Purpose | Command | Expected on success |
|---|---|---|
| Backend up | `cargo run -p parallax-cli -- serve` (verify the real serve invocation in `crates/parallax-cli` first) | ready banner lists UI/GraphQL/OTLP surfaces |
| Stack up | `docker compose -f deploy/docker-compose.yml up --build -d` (playground) | all containers healthy (`docker compose ps`) |
| Scenario | `scenarios/run.sh a1` (list ids with no args) | exit 0 |
| CLI sims | `./target/debug/playground`, `… cron`, `… daemon`, `… console --seconds 30` | exit 0 each |
| Browser session | open `http://localhost:5173`, exercise checkout | web spans arrive |
| Observable tests | `CLI_INVOCATION_ID=$(uuidgen) scripts/observable-test-session.sh rust` (plan-158 shape) | exit 0 |
| GraphQL probe | `curl -s http://127.0.0.1:4000/graphql -H 'content-type: application/json' -d '{"query":"…"}' \| jq` | asserted fields non-empty |
| Teardown | `docker compose -f deploy/docker-compose.yml --profile demo down` | clean |

## Scope

**In scope:** evidence scripts + captured outputs + screenshots + one
validation write-up under
`docs/research/validation/2026-07-unified-cli-observability/`; updates to the
playground `VERIFICATION.md` (its branch); status-row updates; small test-only
fixes discovered here are routed back to plans 156-158's owners (same
branch), each as its own commit.

**Out of scope:** performance benchmarking (four-build rule does not apply —
this is not a GreptimeDB-vs-ClickHouse benchmark), multi-backend fan-out
(plan 154), retention/prune, release gates (plan 102).

## Steps

### Step 1: Bring-up

Start `parallax serve` (fresh data dir; capture the ready banner text), then
the playground stack. Capture `docker compose ps` output showing healthy
containers.

**Verify**: banner names UI URL, GraphQL, both OTLP ports, storage mode,
data dir; 14 containers healthy.

### Step 2: Generate the corpus

Run, capturing exit codes: k6/demo baseline for ~2 minutes OR
`scenarios/run.sh` ids covering happy path + one failure scenario; each CLI
mode once (`drive`, `cron`, `daemon` for ≥60 s so cycles fire, `console
--seconds 30`); one real browser session on `:5173` (add to cart → checkout);
one observable test session (nextest bridge).

**Verify**: every command exit 0; note each minted invocation id (the CLI
prints it — plan 158 keeps ids visible in progress output).

### Step 3: Machine assertions over GraphQL

Write `docs/research/validation/2026-07-unified-cli-observability/assert.sh`
(curl+jq, kept in the evidence dir) asserting, with recorded JSON outputs:
1. `invocations` returns entries for all four CLI modes with correct
   `appMode`, `commandName`, and terminal `outcome`/`exitCode` for the
   finished ones; the `daemon` one shows `running` while alive.
2. `invocation(invocationId:)` for the console run: `sessions` has one
   closed pair; `screenVisits` ≥2 with strictly increasing
   `navigationSequence`; `uiActions` ≥2 incl. `checkout.submit` whose trace
   (via `tracesByInvocation`) crosses into `checkout` service spans.
3. `backgroundCycles(invocationId:)` for the daemon run: ≥1 cycle name with
   count ≥1.
4. `jobs(...)`: ≥1 `order_dispatch` and ≥1 `fulfillment_shipment` job whose
   producer and consumer share `jobId` across process boundaries.
5. `conversations(invocationId:)` for the daemon run: ≥1 conversation with
   agent + provider names.
6. `logsByInvocation`/`tracesByInvocation` non-empty for the drive run;
   `serviceMap` includes nodes of kind `cli`, `browser`, and `service`.
7. Negative: a query filtering the legacy field name fails schema validation
   (`run(runId:)` unknown field), and no ingested resource for the CLI runs
   carries `parallax.run.id` (probe via the `sql` field over
   `opentelemetry_traces`).

**Verify**: `bash assert.sh` exits 0; JSON outputs stored beside it.

### Step 4: Browser evidence

Drive the UI (browser automation or manual): capture screenshots of
(a) `/invocations` list with the running daemon pulsing and mode badges;
(b) console-run hub Overview; (c) Traces tab streaming with Live ON (two
captures ≥10 s apart showing growth); (d) Logs tab live tail; (e) Errors tab
after the failure scenario; (f) Sessions & UI tab with the screen-visit lane;
(g) Jobs & Cycles tab; (h) `/ecosystem` with cli/browser/service kinds;
(i) a trace detail reached from the hub with the invocation back-link.
Confirm zero browser-console errors during the walk (capture the console).

**Verify**: nine named PNGs in the evidence dir; console log capture clean.

### Step 5: Write-up + closure

Write `docs/research/validation/2026-07-unified-cli-observability/README.md`:
date, branch SHAs (both repos), what ran, assertion results table,
screenshot index, deviations found and where they were fixed. Update the
playground `VERIFICATION.md`. Tear down containers. If everything is green,
this is the PR-readiness evidence for the one Parallax PR + linked
playground PR.

**Verify**: evidence dir complete; `docker compose ps` empty; both branches
pushed.

## Done criteria

- [ ] `assert.sh` exit 0 with stored outputs (all 7 assertion groups).
- [ ] Nine UI screenshots + clean browser console captured.
- [ ] Ready banner + compose health captured.
- [ ] Evidence README written; playground `VERIFICATION.md` updated.
- [ ] Discovered defects fixed under plans 156-158 (own commits) and
  re-asserted — no known-red assertion at closure.
- [ ] `plans/README.md` status row updated.

## STOP conditions

Stop and report back (do not improvise) if:
- OTLP data arrives but an assertion fails for a CONTRACT reason (wrong
  key priority, missing column promotion) — that is a plan-156 defect; fix
  there, never patch the assertion.
- `host.docker.internal` is unreachable from containers on this host —
  capture the failing curl from inside a container and report the network
  shape rather than editing compose bindings ad hoc.
- The UI needs a data shape the API cannot serve (plan 157 STOP condition
  redux) — route to 156.
- Resource pressure makes the 14-container stack unstable on this machine —
  reduce to the minimal service set needed per assertion (document which),
  do not fake outputs.

## Maintenance notes

- Re-run this acceptance whenever the contract registry changes or a
  GreptimeDB engine upgrade lands (native-table promotion is the fragile
  seam).
- Plan 154 (multi-backend sweep) remains separately blocked on its five-
  backend matrix; this plan's evidence covers the Parallax backend only —
  do not mark 154 done from here.
