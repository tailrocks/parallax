# Plan 037: One-command playground demo against `parallax serve` — baseline traffic, scenario runner, env contract, doc accuracy

> **Executor instructions**: This plan targets the **playground repository**
> (`parallax-telemetry-playground`). Follow step by step; run every
> verification. On any STOP condition, stop and report. When done, update the
> status row in the Parallax repo's `plans/README.md`.
>
> **Drift check (run first)**: in the playground repo,
> `git diff --stat ed1f975..HEAD -- README.md scenarios loadgen deploy VERIFICATION.md cli`
> On any in-scope drift vs the excerpts below, STOP.

## Status

- **Priority**: P1
- **Effort**: M
- **Risk**: LOW
- **Depends on**: none (better after plan 036 lands, but independent)
- **Category**: dx
- **Planned at**: commit `408be17` (Parallax) / `ed1f975` (playground), 2026-07-07

## Why this matters

The playground is supposed to be the sample ecosystem a reviewer runs to see
every Parallax capability. Today that requires 2-3 repos and ~5-8 cross-repo
commands, the README never mentions starting Parallax at all, nothing
generates continuous traffic (the UI opens to flat charts that go dead a
minute after a scenario), the scenario index is stale, and the env contract is
scattered across source files. The decisive fact: **no wiring change is
needed** — the compose already exports OTLP to `host.docker.internal:4317/4318`,
and `parallax serve` binds exactly `4317` (gRPC) / `4318` (HTTP) with its UI
on `:4000` (Parallax repo `crates/parallax-server/src/config.rs:76-78`). The
gap is a demo entrypoint, a baseline load generator, and honest docs.

## Current state

All excerpts verified at playground commit `ed1f975`.

- `deploy/docker-compose.yml:16-28` — shared env anchor:

  ```yaml
  x-otlp: &otlp
    OTEL_EXPORTER_OTLP_ENDPOINT: "http://host.docker.internal:4317"
    OTEL_EXPORTER_OTLP_PROTOCOL: "grpc"
    ...
    OTEL_RESOURCE_ATTRIBUTES: "deployment.environment.name=playground"
    PARALLAX_ENV: "playground"
  ```

  Java services override to `http://host.docker.internal:4318` +
  `http/protobuf` (`:99-100` etc.); web proxies browser OTLP to
  `ROTEL_OTLP_HTTP_ENDPOINT: "http://host.docker.internal:4318"` (`:144`).
  All comments say "the lab's Rotel" — but these are also exactly
  `parallax serve`'s ports. **No endpoint change is needed for a Parallax
  demo; only the docs/entrypoint are missing.**

- `deploy/docker-compose.yml:70-73` — Postgres runs with a committed default
  password (credential type: Postgres password) and **no service uses it**
  (repo-wide grep for `DATABASE_URL`/`sqlx`/`jdbc`/`5432` in `services/`,
  `libs/`, `cli/` is empty). Plan 048 wires it; here only parameterize the
  password via env so the literal leaves the file.

- `loadgen/checkout.js` — one-shot k6, not in compose:

  ```js
  // loadgen/checkout.js:5-9
  export const options = { vus: 5, duration: "1m" };
  const BASE = __ENV.CHECKOUT_URL || "http://localhost:8088";
  export default function () {
    http.get(`${BASE}/checkout?sku=WIDGET-1&quantity=${1 + Math.floor(Math.random() * 5)}`);
  ```

  No `loadgen` service exists in `deploy/docker-compose.yml` (services:
  checkout, pricing, inventory, recommendation, notifications, orders,
  postgres, broker, flagd, catalog, payment, fulfillment, web).

- `scenarios/README.md:4-6` — stale: claims only "A1 … A12 … Implemented.
  The rest … follow", while `scenarios/` actually holds 11 scripts:
  `a1-checkout.sh`, `a3-async.sh`, `a4-reverse.sh`, `a12-cli-run.sh`,
  `a13-deploy-regression.sh`, `a18-canary.sh`, `b-async-chaos.sh`,
  `b-chaos.sh`, `b-checkout-chaos.sh`, `b-degradation.sh`, `b17-cron.sh`.
  Each script's intended check lives only in ad-hoc trailing `echo` lines.

- `README.md:71-83` — the Run block never mentions Parallax; step 1 is
  "start the lab (parallax repo: bench/otlp-fanout)". The architecture
  diagram (`:16-23`) lists `loadgen (k6)` and `Postgres` as if active.

- `cli/src/main.rs:1-24` — the `playground` binary (`drive` default, `cron`
  mode), flushes before exit. `scenarios/a12-cli-run.sh` and `b17-cron.sh`
  `exec target/debug/playground …`, which requires a prior `cargo build`
  the README never states, and wrap it in `parallax run start` from the
  Parallax repo without noting it's optional.

- No `.env.example` exists (`.gitignore` lists `.env`). Env knobs scattered:
  `OTEL_EXPORTER_OTLP_ENDPOINT`, `PARALLAX_ENV` (compose), `SENTRY_DSN`,
  `VITE_SENTRY_DSN` (VERIFICATION.md), `RELEASE`
  (`services/checkout/src/main.rs:136`), `CHECKOUT_URL`/`ORDERS_URL`/
  `FULFILLMENT_URL` (scenario scripts), `FLAGD_HOST` (compose `:103`).

- Parallax-side facts to state in docs (verified in the Parallax repo at
  `408be17`): `parallax serve` defaults — UI/GraphQL `:4000`, OTLP gRPC
  `:4317`, OTLP HTTP `:4318` (`crates/parallax-server/src/config.rs:76-78`).

## Commands you will need

| Purpose | Command (playground root) | Expected |
|---------|---------------------------|----------|
| Compose validate | `docker compose -f deploy/docker-compose.yml config` | exit 0 |
| Compose up (demo) | `docker compose -f deploy/docker-compose.yml --profile demo up --build -d` | services start |
| Rust build | `rtk cargo build` | exit 0 |
| Script lint | `bash -n <script>` | exit 0 |

## Scope

**In scope** (playground repo):
- `demo.sh` (create, repo root)
- `deploy/docker-compose.yml` (add `loadgen` service under a `demo` profile;
  parameterize `POSTGRES_PASSWORD`; fix stale comments)
- `loadgen/demo.js` (create), `loadgen/checkout.js` (leave as-is)
- `scenarios/run.sh` (create), `scenarios/README.md` (rewrite)
- `README.md` (Run section + architecture notes)
- `.env.example` (create)
- `VERIFICATION.md` (only the pointer to the new demo path; full tour doc is
  plan 054)

**Out of scope**:
- TOUR.md / guided tour beats — plan 054.
- Wiring Postgres to a service — plan 048.
- Removing the fan-out-lab path — keep it working; the demo path is additive.
- Any Parallax-repo change beyond the plans/README.md status row.
- New scenarios (plans 042/047/048/049/050/054 add them); `run.sh` must make
  adding entries trivial.

## Git workflow

- Playground repo, `main`, Conventional Commits, `git commit -s`, one
  `Co-authored-by: Claude <noreply@anthropic.com>` trailer. Push when done.

## Steps

### Step 1: `loadgen/demo.js` — continuous, story-rich baseline

Create a k6 script for ambient demo traffic (NOT the one-shot benchmark):
- `options = { vus: 2, duration: "24h" }` (the compose service is stopped
  with the stack; long duration ≈ "runs until stopped").
- Each iteration: weighted mix — ~80% healthy checkout (varied `sku` from a
  small list, `quantity` 1-5), ~10% `?slow=250..1500`, ~5% `?fail=1`,
  ~5% `/quote-stream?quantity=4`; hit `ORDERS_URL /order` and
  `FULFILLMENT_URL /publish` every ~10th iteration so async/messaging lanes
  stay alive. Sleep 1-3s randomized. Read base URLs from `__ENV` with
  in-network defaults (`http://checkout:8088`, `http://orders:8092`,
  `http://fulfillment:8080`).
  (Check `scenarios/a3-async.sh`/`a4-reverse.sh` for the exact orders/
  fulfillment paths and ports before hardcoding — fulfillment is published on
  host `:8093` but in-network it is `:8080`.)

**Verify**: `docker run --rm -i grafana/k6:latest inspect - < loadgen/demo.js`
→ exit 0 (script parses). If the k6 image can't be pulled here, `bun x` is
NOT a substitute — mark the check as done-at-compose-up instead.

### Step 2: `loadgen` compose service under the `demo` profile

In `deploy/docker-compose.yml` add:

```yaml
  loadgen:
    image: grafana/k6:latest
    profiles: [demo]
    command: ["run", "/scripts/demo.js"]
    environment:
      CHECKOUT_URL: "http://checkout:8088"
      ORDERS_URL: "http://orders:8092"
      FULFILLMENT_URL: "http://fulfillment:8080"
    volumes: ["../loadgen:/scripts:ro"]
    depends_on: [checkout, orders, fulfillment]
    networks: [pg]
```

Also in this file: change `postgres`'s `POSTGRES_PASSWORD: playground` to
`POSTGRES_PASSWORD: ${POSTGRES_PASSWORD:-playground}` (env-parameterized,
same default so nothing breaks), and update the header comment (`:1-6`) to
say the stack exports to **any OTLP listener on host 4317/4318 — either
`parallax serve` (UI on :4000) or the fan-out lab's Rotel**.

**Verify**: `docker compose -f deploy/docker-compose.yml config` → exit 0;
`docker compose -f deploy/docker-compose.yml config --profiles` lists `demo`;
default (no profile) config does NOT include `loadgen`.

### Step 3: `demo.sh` entrypoint

Create executable `demo.sh` at the playground root:

```bash
#!/usr/bin/env bash
set -euo pipefail
# One-command demo: playground stack + baseline traffic → local parallax serve.
if ! nc -z 127.0.0.1 4317 2>/dev/null; then
  echo "No OTLP listener on 127.0.0.1:4317."
  echo "Start Parallax first (in the parallax repo):  parallax serve"
  echo "(or the fan-out lab if you are comparing backends)"
  exit 1
fi
docker compose -f deploy/docker-compose.yml --profile demo up --build -d
echo "Stack up. Baseline traffic is running (k6 'loadgen', ~1-2 rps)."
echo "Fire a story scenario:   scenarios/run.sh a1"
echo "Open Parallax:           http://localhost:4000"
echo "Stop everything:         docker compose -f deploy/docker-compose.yml --profile demo down"
```

Narration matters (Parallax repo rule: long-running steps announce
themselves) — keep the ready banner naming the UI URL.

**Verify**: `bash -n demo.sh` → exit 0; `chmod +x demo.sh` applied
(`ls -l demo.sh` shows executable bit).

### Step 4: `scenarios/run.sh` + catalog rewrite

1. Create `scenarios/run.sh`:
   - `run.sh` with no args → prints the catalog table (id, one-line what it
     does, what to open in Parallax).
   - `run.sh <id>` → executes `scenarios/<matching>.sh` (map `a1` →
     `a1-checkout.sh`, etc. — derive from filename prefix), then prints the
     "check in Parallax" line for that id.
   - Store the catalog as a simple in-script table (bash case or a
     here-doc) so plans 042-054 append rows trivially.
2. Rewrite `scenarios/README.md` as the real catalog. One row per existing
   script. Columns: ID | Script | Drives | Check in Parallax UI. Fill "Check
   in Parallax" concretely, e.g.:
   - a1 → Traces: one checkout trace, waterfall with pricing/inventory/
     recommendation children (children stitched only after plan 036 lands —
     say so honestly if 036 hasn't landed).
   - a3/a4 → Trace detail: producer span with link to consumer trace.
   - a12 → Runs: a run row with command + exit code (note: requires
     `cargo build` first and optionally `parallax run start` — document
     both).
   - a13 → Issues: error spike while `RELEASE=v2` (note current limitation:
     release attribution lands with plan 042).
   - a18 → Issues/bundle: redaction canary (FAKE corpus).
   - b-* → Issues/Services: each chaos knob and where it shows.
   - b17 → Runs: cron run outcomes (build prerequisite as a12).
3. Keep every existing scenario script untouched.

**Verify**: `bash -n scenarios/run.sh` → exit 0; `scenarios/run.sh` (no
stack) prints the table and exits 0; `rtk grep -c "Check in Parallax"
scenarios/README.md` ≥ 1 and the README lists all 11 scripts.

### Step 5: `.env.example` + README Run rewrite

1. `.env.example` (names + comments only, no real values): document
   `OTEL_EXPORTER_OTLP_ENDPOINT`, `PARALLAX_ENV`, `SENTRY_DSN`,
   `VITE_SENTRY_DSN`, `RELEASE`, `POSTGRES_PASSWORD`, `CHECKOUT_URL`,
   `ORDERS_URL`, `FULFILLMENT_URL`, `FLAGD_HOST`.
2. README `## Run` becomes two paths:
   - **Demo against Parallax (primary)**: `parallax serve` in the Parallax
     repo → `./demo.sh` → `scenarios/run.sh a1` → open
     `http://localhost:4000`.
   - **Fan-out lab (comparison)**: the existing three-step block, unchanged
     below the demo path.
   Also: note the CLI build prerequisite
   (`cargo build && ./target/debug/playground [cron]`), and mark `loadgen`
   and `postgres` in the architecture diagram as `(demo profile)` and
   `(reserved — unused until DB scenarios land)` respectively.
3. VERIFICATION.md: add a one-line pointer at the top — "For the quick demo
   path see `./demo.sh`; this file is the full cross-backend verification
   runbook."

**Verify**: `rtk grep -n "parallax serve" README.md` ≥ 1;
`rtk grep -n "demo.sh" README.md VERIFICATION.md` ≥ 2; `.env.example` exists
and contains no `=`-assigned secret-looking values (names/comments/defaults
only; `grep -i "dsn=" .env.example` → placeholder only, no real DSN).

### Step 6: Live run-through

With `parallax serve` running locally: `./demo.sh`, wait ~2 minutes, open
`http://localhost:4000` — Overview shows nonzero spans/logs with a moving
chart; Services lists ≥6 services; run `scenarios/run.sh b-chaos` and see an
issue appear. Record observations in the commit message body. If Docker or a
local Parallax is unavailable in your environment, STOP and report which
verification you could not run (do not claim it).

## Test plan

Shell scripts: `bash -n` each new/changed script; compose: `config` gate; the
live Step 6 run is the integration test. No Rust/TS code changes in this
plan, so no unit tests.

## Done criteria

ALL must hold (playground repo):

- [ ] `demo.sh` exists, executable, `bash -n` clean, refuses politely when
      :4317 has no listener
- [ ] `docker compose -f deploy/docker-compose.yml config` exit 0; `loadgen`
      only under `demo` profile
- [ ] `rtk grep -n "POSTGRES_PASSWORD:" deploy/docker-compose.yml` shows the
      `${POSTGRES_PASSWORD:-...}` form, not a bare literal
- [ ] `scenarios/run.sh` lists and dispatches all 11 existing scenarios
- [ ] `scenarios/README.md` has a row per script with a "Check in Parallax
      UI" column
- [ ] `.env.example` committed; README Run section names `parallax serve` and
      `http://localhost:4000`
- [ ] Step 6 live check recorded (or explicitly reported as blocked)
- [ ] Status row updated in Parallax repo `plans/README.md`

## STOP conditions

- The compose OTLP defaults no longer point at host `4317/4318` (drift) —
  re-verify against `parallax serve` ports before writing docs.
- `grafana/k6` image cannot run the script due to k6 API drift — pin a
  working tag and note it; if none works, STOP.
- Step 6 shows services missing from Parallax that emit fine to the lab —
  that is a Parallax ingest bug, not yours: report it with the service names,
  don't work around it in this plan.

## Maintenance notes

- Plans 042/047/048/049/050/054 append rows to `scenarios/run.sh` +
  `scenarios/README.md` — keep the catalog format stable.
- Plan 054 turns VERIFICATION.md's demo beats into a full TOUR doc backed by
  this plan's runner.
- Reviewer: check demo.sh fails gracefully (no half-up stack) and that the
  demo profile never auto-starts for lab users.
- Deferred: seeded/back-dated history window (Parallax repo has
  `crates/parallax-server/examples/seed.rs` for that; wiring the playground
  to it was considered and left out — the continuous loadgen covers the
  "alive charts" need).
