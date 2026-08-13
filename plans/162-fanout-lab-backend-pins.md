# Plan 162: Pin every fan-out-lab and playground infra image at current latest stable

> **Executor instructions**: Follow this plan step by step. Run every
> verification command and confirm the expected result before moving on. If
> anything in "STOP conditions" occurs, stop and report — do not improvise.
> When done, update this plan's row in `plans/README.md`.
>
> **Drift check (run first)**:
> `git diff --stat f6208070..HEAD -- bench/otlp-fanout/ docs/research/validation/otlp-fanout-comparison-lab.md docs/research/reference/feature-inventory-and-playground-verification.md`
> (parallax repo) and
> `git -C ../parallax-telemetry-playground diff --stat 6e0a0d5..HEAD -- deploy/docker-compose.yml README.md`.
> If any in-scope file changed since planning, compare the "Current state"
> excerpts against live code first; on mismatch, STOP.

## Status

- **Priority**: P1
- **Effort**: M
- **Risk**: MED (backend upgrades can break ingest paths that previously verified)
- **Depends on**: none
- **Category**: migration
- **Planned at**: parallax `f6208070`, playground `6e0a0d5`, 2026-08-13

## Why this matters

The comparison lab's credibility depends on comparing against *current*
backends. Today Maple is pinned at v0.0.12 (latest v0.0.18), OpenObserve,
Rotel, telemetrygen, Redpanda, flagd, and k6 all run unpinned `:latest`
(non-reproducible), SigNoz is vendor-pinned at v0.129.0 (latest v0.137.0),
Sentry is vendor-pinned at 26.6.0 (latest self-hosted 26.7.2), and postgres
is one major behind (17 vs 18). Reproducible pins at latest stable are the foundation for every
verification and comparison run in plans 163–166: an unpinned lab cannot
produce evidence that stays true.

## Current state

Two repositories are involved:

- **parallax** (this repo): the fan-out lab under `bench/otlp-fanout/` —
  `compose.yml` (Rotel + OpenObserve + telemetrygen), `compose.maple.yml`
  (Maple overlay), `compose.signoz.yml` (SigNoz overlay), `setup-vendor.sh`
  (vendors SigNoz/Sentry composes), `rotel.env.example`, `README.md`.
- **playground** (`../parallax-telemetry-playground`, sibling checkout,
  github.com/tailrocks/parallax-telemetry-playground): app infra images in
  `deploy/docker-compose.yml`.

Excerpts as of the planned-at SHAs:

`bench/otlp-fanout/compose.yml:14` — `# Pin every image tag at implementation; :latest here is a starting point.`
`bench/otlp-fanout/compose.yml:21` — `image: streamfold/rotel:latest`
`bench/otlp-fanout/compose.yml:36` — `image: public.ecr.aws/zinclabs/openobserve:latest`
`bench/otlp-fanout/compose.yml:54` — `image: ghcr.io/open-telemetry/opentelemetry-collector-contrib/telemetrygen:latest`

`bench/otlp-fanout/compose.maple.yml:14` — `MAPLE_VERSION: "v0.0.12"` (build
arg; pinned because the installer's unauthenticated `releases/latest` GitHub
API call is rate-limited inside `docker build`).

`bench/otlp-fanout/setup-vendor.sh:14` — `SIGNOZ_REF="${SIGNOZ_REF:-v0.129.0}"`
(SigNoz vendor pin; the script clones ONLY SigNoz and **skips silently when
the vendor tree already exists** — lines 15–20).
`bench/otlp-fanout/sentry/setup.sh:16` — `SENTRY_REF="${SENTRY_REF:-26.6.0}"`
(Sentry vendor pin lives in this separate script, same skip-if-present
behavior at lines 38–42).

`../parallax-telemetry-playground/deploy/docker-compose.yml:97` — `image: postgres:17`
`../parallax-telemetry-playground/deploy/docker-compose.yml:108` — `image: redpandadata/redpanda:latest`
`../parallax-telemetry-playground/deploy/docker-compose.yml:118` — `image: ghcr.io/open-feature/flagd:latest`
`../parallax-telemetry-playground/deploy/docker-compose.yml:210` — `image: grafana/k6:latest`

Latest stable versions as researched 2026-08-13 (re-resolve at execution —
the version policy is "latest stable at implementation time", these are
floors, not freezes):

| Component | Deployed | Latest stable 2026-08-13 | Where to check |
| --- | --- | --- | --- |
| Maple | v0.0.12 | v0.0.18 | github.com/Makisuo/maple/releases |
| OpenObserve | `:latest` | v0.92.0 | github.com/openobserve/openobserve/releases |
| SigNoz | vendored, unpinned | v0.137.0 | github.com/SigNoz/signoz/releases |
| Sentry self-hosted | verified 26.6.0 | 26.7.2 | github.com/getsentry/self-hosted/releases |
| Rotel | `:latest` | pin current tag | hub.docker.com/r/streamfold/rotel/tags |
| telemetrygen | `:latest` | pin current tag | ghcr.io otel-collector-contrib releases (collector v0.158.0 line) |
| postgres | 17 | 18 | hub.docker.com/_/postgres |
| Redpanda | `:latest` | pin current tag | hub.docker.com/r/redpandadata/redpanda/tags |
| flagd | `:latest` | pin current tag | github.com/open-feature/flagd/releases |
| k6 | `:latest` | pin current tag | hub.docker.com/r/grafana/k6/tags |

Repo constraints that bind this plan (from `AGENTS.md`): GreptimeDB+Turso are
the only Parallax engines (backends here are comparators, not alternatives);
version tables in docs are known-compatible floors — update them in the same
commit when versions move.

## Commands you will need

| Purpose | Command | Expected on success |
|---|---|---|
| Resolve a GitHub latest release | `gh api repos/<owner>/<repo>/releases/latest --jq .tag_name` | tag string |
| Lab core up | `cd bench/otlp-fanout && cp rotel.env.example rotel.env && docker compose -f compose.yml up -d` | rotel + openobserve running |
| Full lab up | `docker compose -f compose.yml -f compose.signoz.yml -f compose.maple.yml up -d` | all containers healthy |
| OpenObserve smoke | `bench/otlp-fanout/smoke.sh` | output contains the PASS line for the OpenObserve search count — **read the output**; the script echoes failures and still exits 0 (`smoke.sh:25-42`), so exit code alone proves nothing |
| Maple assert | `docker exec pfanout-maple maple traces 2>/dev/null \| head` (or the Maple UI on host :8081) | trace rows from the smoke batch |
| SigNoz assert | ClickHouse query per playground `VERIFICATION.md` "Multi-backend fan-out residual" idiom | smoke batch count |
| Sentry assert | `bench/otlp-fanout/sentry/verify.sh` | exit 0 (A15/A16 asserts) |
| Playground compose valid | `cd ../parallax-telemetry-playground && docker compose -f deploy/docker-compose.yml config -q` | exit 0 |
| Docker Hub tag listing | `curl -s "https://hub.docker.com/v2/repositories/<ns>/<repo>/tags?page_size=25" \| python3 -c "import json,sys; [print(t['name']) for t in json.load(sys.stdin)['results']]"` | tag list to choose from |
| Parallax docs links | `cargo xtask docs links` | `documentation links passed` |

## Scope

**In scope**:
- parallax: `bench/otlp-fanout/compose.yml`, `compose.maple.yml`,
  `compose.signoz.yml`, `setup-vendor.sh`, `sentry/setup.sh`,
  `rotel.env.example`, `bench/otlp-fanout/README.md`,
  `docs/research/validation/otlp-fanout-comparison-lab.md` (version rows only),
  `docs/research/reference/feature-inventory-and-playground-verification.md`
  (Workstream 2 table "Deployed today" column only).
- playground: `deploy/docker-compose.yml` (image tags only), `README.md`
  (version mentions only).

**Out of scope**:
- Adding new backends (Grafana LGTM, HyperDX, Uptrace) — deferred, see
  `plans/README.md` rejected/deferred notes.
- Any Parallax product code under `crates/` or `ui/`.
- App-level dependency upgrades (OTel SDKs, Sentry SDKs, Boot) — that is
  plan 163.
- Scenario scripts — plan 164.

## Git workflow

Both repos: `main` rejects direct pushes (PR-only ruleset with required
checks + DCO). One short-lived branch per repo, one PR per repo, never a
second parallel PR in the same repo. Commits: Conventional Commits, signed
off (`git commit -s`), trailer `Co-authored-by: Claude <noreply@anthropic.com>`
(or the executing agent's canonical trailer per `COMMITS.md`).

## Steps

### Step 1: Resolve the current latest tags

For GitHub-released tools use
`gh api repos/<owner>/<repo>/releases/latest --jq .tag_name`
(openobserve/openobserve, SigNoz/signoz, getsentry/self-hosted,
Makisuo/maple, open-feature/flagd). For Docker-Hub-only images (`rotel`,
`redpanda`, `k6`, `postgres`, telemetrygen on ghcr) use the tag-listing curl
from the commands table (ghcr: `gh api "orgs/open-telemetry/packages/container/opentelemetry-collector-contrib%2Ftelemetrygen/versions" --jq '.[].metadata.container.tags[]' | head`).
Selection rule: newest tag matching `^v?[0-9]+(\.[0-9]+)*$` (no `-rc`,
`-nightly`, `-beta`, `head`). Record every choice in a scratch table.

**Verify**: every version-table row has a concrete tag matching the selection
regex; `grep -c latest <scratch table>` → 0.

### Step 2: Pin the lab core (`bench/otlp-fanout/compose.yml`)

Replace the three `:latest` images with resolved tags (`streamfold/rotel:<tag>`,
`public.ecr.aws/zinclabs/openobserve:<tag>`, telemetrygen `:<tag>`). Delete
the now-false comment line 14 (`# Pin every image tag at implementation…`)
and replace with the pin date: `# Pinned <YYYY-MM-DD>; bump via plan-162 procedure.`

**Verify**: `grep -n "latest" bench/otlp-fanout/compose.yml` → no image tags
(only prose, if any).

### Step 3: Bump Maple to the resolved tag

`compose.maple.yml` build arg `MAPLE_VERSION: "v0.0.12"` → resolved tag
(v0.0.18 or newer). Rebuild: `docker compose -f compose.yml -f compose.maple.yml build maple`.

**Verify**: build exits 0. If the Maple installer layout changed and the
build fails → STOP condition 3.

### Step 4: Bump the SigNoz and Sentry vendor pins

Two separate scripts hold the pins: `bench/otlp-fanout/setup-vendor.sh:14`
(`SIGNOZ_REF`, currently v0.129.0) and `bench/otlp-fanout/sentry/setup.sh:16`
(`SENTRY_REF`, currently 26.6.0). Edit both defaults to the resolved tags.
Both scripts **skip when the vendor tree exists**, so refresh explicitly:
delete the existing vendored checkouts (their paths are named inside each
script; they are untracked per `PROJECT_STRUCTURE.md`) and re-run both
scripts.

**Verify**: `grep -n "SIGNOZ_REF" bench/otlp-fanout/setup-vendor.sh` shows
the new tag; `grep -n "SENTRY_REF" bench/otlp-fanout/sentry/setup.sh` shows
the new tag; both scripts exit 0 on a fresh run (vendor trees absent first).

### Step 5: Pin playground infra images

In `../parallax-telemetry-playground/deploy/docker-compose.yml`: `postgres:17`
→ `postgres:18`, and pin `redpandadata/redpanda`, `ghcr.io/open-feature/flagd`,
`grafana/k6` to resolved tags. In the header comment (line 12), replace only
the sentence "Pin image tags at a real deploy." with "Image tags pinned
<YYYY-MM-DD> (plan 162)." — keep the rest of that comment line (it also
points at `deploy/Dockerfile.web`).

postgres 17→18 note: the playground schema is created fresh on `up` (no
migration of persisted volumes is supported); document in the commit message
that existing `postgres` volumes must be dropped (`docker compose down -v`).

**Verify**: `docker compose -f deploy/docker-compose.yml config -q` → exit 0;
`grep -n ":latest" deploy/docker-compose.yml` → no matches.

### Step 6: Live smoke of the pinned lab

Start host `parallax serve` (Homebrew preview or `cargo run -p parallax-cli --
serve`), bring up the full lab (core + signoz + maple overlays), start the
Sentry vendored stack + onboarding (`bench/otlp-fanout/sentry/onboard.sh`)
if the Sentry arm is in the run, then assert **each backend separately**
(the historical idiom — playground `VERIFICATION.md` §"Multi-backend fan-out
residual" — is one assert per backend, not one script):
OpenObserve via `smoke.sh` (read its PASS/FAIL output text — it exits 0
either way), Maple via its trace list/UI, SigNoz via its ClickHouse count,
Sentry via `sentry/verify.sh`, Parallax via
`parallax sql "SELECT count(*) FROM opentelemetry_traces WHERE service_name='smoke'"`.

**Verify**: every enabled backend shows the smoke batch; any backend at 0 →
STOP condition 2.

### Step 7: Update the paper trail

- `bench/otlp-fanout/README.md`: add/refresh a "Pinned versions (YYYY-MM-DD)"
  table listing every image tag.
- `docs/research/validation/otlp-fanout-comparison-lab.md`: refresh version
  mentions.
- `docs/research/reference/feature-inventory-and-playground-verification.md`
  Workstream 2 table: set "Deployed today" to the new pins.
- Playground `README.md`: refresh any version mentions.

**Verify**: `cargo xtask docs links` → `documentation links passed`.

## Test plan

No unit tests — infrastructure pins. The live smoke (Step 6) is the test.
Record its output (per-backend counts + versions) in the PR description.

## Done criteria

- [ ] No `:latest` image tag remains in `bench/otlp-fanout/*.yml` or
      playground `deploy/docker-compose.yml`.
- [ ] Maple build arg ≥ v0.0.18; postgres:18; SigNoz/Sentry vendor pins ≥
      v0.137.0 / 26.7.2 (or newer stable resolved at execution).
- [ ] Per-backend Step-6 asserts all show the smoke batch at the pinned
      versions (smoke.sh PASS text for OpenObserve, not just exit 0).
- [ ] Version tables updated in the four docs listed in Step 7.
- [ ] `cargo xtask docs links` passes.
- [ ] `plans/README.md` row updated.

## STOP conditions

1. Current-state excerpts don't match (drift since f6208070 / 6e0a0d5).
2. A pinned backend stops accepting the fan-out (smoke count 0) and one
   reasonable config fix (auth header, endpoint path, port) doesn't restore
   it — report the exact failing backend+version instead of downgrading
   silently.
3. Maple's container build breaks at the new tag (its installer contract is
   pre-1.0 and moves) — report; do not vendor a fork.
4. Rotel's env-var contract changed at the pinned tag (exporter env names in
   `rotel.env.example` rejected) — report with the failing variable names.

## Maintenance notes

- Renovate is configured in the playground repo and will propose future image
  bumps; the lab compose files in parallax are not Renovate-covered — recheck
  pins whenever plan-165 comparison runs restart.
- Reviewer: confirm no engine-substitution smell — backends are comparators;
  Parallax stays GreptimeDB+Turso only.
- Deferred: roster additions (Grafana LGTM v13.x, HyperDX, Uptrace v2.1) —
  decision recorded in `plans/README.md`.
