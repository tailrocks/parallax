# Plan 054: Telemetry-quality scenarios + demo tour — sampling gap, cron semantics, field-spike logs, uncorrelated log, TOUR.md

> **Executor instructions**: Targets the **playground repository**
> (`parallax-telemetry-playground`). Follow step by step; run every
> verification. On any STOP condition, stop and report. When done, update the
> status row in the Parallax repo's `plans/README.md`.
>
> **Drift check (run first)**: in the playground repo,
> `git diff --stat ed1f975..HEAD -- libs/playground-telemetry cli scenarios VERIFICATION.md`
> On mismatch with the excerpts below, STOP.

## Status

- **Priority**: P3
- **Effort**: M
- **Risk**: LOW
- **Depends on**: plan 037 (demo.sh + scenario runner/catalog — the TOUR
  builds on them); plan 036 (span status, for honest cron errors)
- **Category**: direction
- **Planned at**: commit `408be17` (Parallax) / `ed1f975` (playground), 2026-07-07

## Why this matters

Three "telemetry quality" demos from the brief have no data: **sampling
visibility** (everything runs at 100%; the UI can never show a sampled-out
gap), **cron/job semantics** (the weighted-outcome job lacks schedule
attributes and missed/duplicate cases), and **field-explorer/uncorrelated
logs** (the log substrate is attribute-rich, but no scenario produces a
field-dominant spike, and no log is ever emitted outside span context — so
the "log without trace/run id = evidence gap" case can't be shown). And the
guided tour — run scenario X, open Parallax screen Y — still lives only as
prose in VERIFICATION.md. This plan closes the quality-scenario tail and
ships the tour document that strings the whole demo ecosystem together.

## Current state

Verified at playground commit `ed1f975`.

- No sampler configured: `libs/playground-telemetry/src/lib.rs:72-75` —
  `SdkTracerProvider::builder().with_resource(...).with_batch_exporter(...)`
  with no `with_sampler` (SDK default = ParentBased(AlwaysOn)); Java
  `traces-sample-rate: 1.0` (`services/*/application.yml`); web
  `tracesSampleRate: 1.0` (`web/src/instrument.client.ts:23`). 100% is
  the intended lab default — only a **dedicated scenario** should sample.
- Cron: `cli/src/main.rs:38-59` — `cron()` is one INTERNAL span with a
  deterministic bucket (0-89 ok / 90-94 fail exit 1 / 95-99 "stuck" 2s
  sleep); no schedule/job attributes, no missed or duplicate cases; driven
  manually by `scenarios/b17-cron.sh`.
- Logs are structured (typed `tracing` fields → OTLP attrs via the bridge,
  `lib.rs:99-108`) but: no scenario emits a spike where one field value
  dominates (the Field Explorer demo, brief L "Field explorer demo" —
  Parallax plan 046's showcase), and every log is emitted inside a span
  context — there is no uncorrelated-log case (brief B: "one scenario
  intentionally emits uncorrelated logs").
- VERIFICATION.md is a prose runbook with concrete "drive X → observe Y"
  steps (A2 exemplars, A5/B15 RUM, A15/A16 grouping, A17 profiling) but
  is backend-generic, manual, and not Parallax-screen-oriented. Parallax UI
  routes available for the tour (Parallax repo `ui/src/components/nav.ts`):
  Overview, Issues, Traces, Logs, Services, Runs, Dashboards, SQL.
- Scenario infra after plan 037: `scenarios/run.sh` catalog +
  `scenarios/README.md` matrix + `demo.sh`.

## Commands you will need

| Purpose | Command (playground root) | Expected |
|---------|---------------------------|----------|
| Build | `rtk cargo build` | exit 0 |
| Lint | `rtk cargo clippy --all-targets -- -D warnings` | exit 0 |
| Scripts | `bash -n scenarios/<new>.sh` | exit 0 |

## Scope

**In scope** (playground repo):
- `libs/playground-telemetry/src/lib.rs` (env-gated ratio sampler)
- `cli/src/main.rs` (cron job attrs + missed/duplicate modes)
- One Rust service for the log-spike + uncorrelated-log knobs (checkout —
  chaos already lives there)
- `scenarios/`: `b22-sampling-gap.sh`, `b17b-cron-suite.sh`,
  `a9-field-spike.sh`, `b23-uncorrelated-log.sh` + catalog rows
- `TOUR.md` (create, repo root) + a pointer from `README.md` and
  `VERIFICATION.md`

**Out of scope**:
- Changing the 100% default sampling anywhere.
- Parallax-side sampling/quality UI (advisor-plans/032 and the brief's
  telemetryQuality score — future).
- A cron scheduler daemon (the suite script simulates schedule semantics).

## Git workflow

- Playground repo, `main`, Conventional Commits, `git commit -s`, one

## Steps

### Step 1: Env-gated ratio sampler

In `libs/playground-telemetry/src/lib.rs`: when `PLAYGROUND_SAMPLE_RATIO`
is set (0.0-1.0), build the tracer provider with
`with_sampler(Sampler::ParentBased(Box::new(Sampler::TraceIdRatioBased(r))))`
(opentelemetry_sdk 0.32 — verify the exact `Sampler` path/name); unset →
current default (100%). Log the active ratio at init (narration rule).

**Verify**: `rtk cargo build` + clippy exit 0; local run with
`PLAYGROUND_SAMPLE_RATIO=0.1` shows ~10% of drive requests producing traces
(record counts).

### Step 2: Sampling-gap scenario

`scenarios/b22-sampling-gap.sh`: restarts (or docker-compose-runs) checkout
with `PLAYGROUND_SAMPLE_RATIO=0.1`, fires 50 checkouts, restores. Print:
"Check in Parallax: Traces shows ~5 of 50; Logs still shows all 50 request
logs — the missing traces ARE the demo (sampled-out evidence); compare a
log row whose trace link goes nowhere." (That dangling log→trace link is
the evidence-gap teaching moment; honest until advisor-plans/032 renders it
explicitly.)

**Verify**: `bash -n` clean; live run recorded (trace count vs log count).

### Step 3: Cron semantics

1. `cli/src/main.rs` `cron()`: add span attributes
   `cron.job.name="playground-report"`, `cron.schedule="*/1 * * * *"`
   (documented as the *declared* schedule), `parallax.run`-friendly
   `process.exit.code` already flows via exit codes; add two new modes:
   `cron missed` (emits NOTHING — the missed check-in is the absence) and
   `cron duplicate` (runs the job span twice with the same
   `cron.invocation.id` attr).
2. `scenarios/b17b-cron-suite.sh`: runs a timeline —
   ok, ok, fail, stuck, missed (skip), duplicate — with ~5s spacing, each
   invocation wrapped in `parallax run start` when available (mirror
   b17's pattern). Print: "Check in Parallax: Runs — exit codes and
   durations; the missing beat at slot 5; two runs sharing
   cron.invocation.id at slot 6."

**Verify**: build + clippy; `bash -n`; live run recorded.

### Step 4: Field-spike + uncorrelated log

1. In checkout add `?spike=<screen>` — emits a burst of WARN logs (~30)
   with a dominant structured field, e.g.
   `tracing::warn!(app_screen_name = %screen, cart_tier = "free", "slow render observed")`
   (dotted OTel names come from the bridge's field mapping — check how
   existing fields land in OTLP attrs and match: if fields arrive as
   `app_screen_name`, document that; the demo needs a *dominant value*, not
   a perfect name).
2. Add `?rogue_log=1` — spawns a detached task that logs OUTSIDE any span
   (`tokio::spawn` without instrument, `tracing::error!("orphan diagnostic — no trace context")`)
   → an OTLP log with no trace/span id.
3. `scenarios/a9-field-spike.sh`: baseline logs + the spike with
   `screen=workspace-select` → "Check in Parallax: Logs + Field Explorer
   (plan 046): the spike window's `app_screen_name` shows
   `workspace-select` at ~90% coverage." `scenarios/b23-uncorrelated-log.sh`
   → "Check in Parallax: Logs — the error row has no trace chip; that
   absence is the evidence gap."

**Verify**: build + clippy; `bash -n` both; live runs recorded (dominant
field visible via Parallax logs/SQL; rogue log confirmed chipless).

### Step 5: TOUR.md

Write `TOUR.md` (playground root): the guided demo, one beat per section,
each beat = *(scenario command → Parallax route → what you should see →
which product capability it proves)*. Beat order (adjust to what has
landed — check the Parallax plans/README status table and mark
not-yet-landed beats "(after plan NNN)"):
1. `./demo.sh` → Overview: live charts (baseline traffic).
2. `scenarios/run.sh a1` → Traces: stitched checkout waterfall.
3. `run.sh b-chaos` → Issues: grouped errors, ERROR spans.
4. `run.sh a3` → Trace detail: producer→consumer link.
5. `run.sh a13` → Issues + release attribution (plans 041/042).
6. `run.sh a22`/`b19` → Runtime lanes (plans 044/045).
7. `run.sh a25` → DB spans + pool exhaustion (plan 048).
8. `run.sh a6` → GraphQL field tree shapes (plan 047).
9. RUM journey (plan 050) → browser→backend trace + web vitals.
10. `run.sh b22`/`b23`/`b17b`/`a9` → the quality beats (this plan).
11. `run.sh a12` → Runs: run-scoped story; bundle export.
Cross-link from `README.md` ("Guided tour: TOUR.md") and the top of
`VERIFICATION.md` ("Demo tour: TOUR.md; this file remains the
cross-backend verification runbook").

**Verify**: every scenario id named in TOUR.md exists in `scenarios/run.sh`
(`for id in $(...); do scenarios/run.sh $id --help-ish check; done` — or a
simple grep cross-check); `rtk grep -c "TOUR.md" README.md VERIFICATION.md`
→ ≥1 each.

## Test plan

- Rust: sampler env parsing unit test (unset → None; `0.1` → ratio; junk →
  default + warning).
- Scripts: `bash -n` + recorded live outcomes per the catalog discipline.
- TOUR.md cross-check (Step 5 verify) is the doc's test.

## Done criteria

- [ ] `PLAYGROUND_SAMPLE_RATIO` sampler works; default behavior unchanged
      when unset
- [ ] b22 recorded: ~10% traces, 100% logs, dangling log→trace links
- [ ] Cron suite: job attrs + missed + duplicate cases recorded in Runs
- [ ] a9 field-spike dominant value confirmed; b23 rogue log chipless
- [ ] TOUR.md committed; every referenced scenario exists; README +
      VERIFICATION point at it
- [ ] `rtk cargo build` + clippy zero warnings; all new scripts in the
      catalog
- [ ] Status row updated in Parallax repo `plans/README.md`

## STOP conditions

- opentelemetry_sdk 0.32's sampler API differs (renamed/moved) — find the
  current shape; if ParentBased+ratio genuinely unavailable, report.
- The tracing→OTLP bridge drops or renames the structured fields so no
  dominant field survives to Parallax — report the actual mapping before
  renaming fields blindly.
- The missed-cron beat is invisible in Parallax (no scheduled-baseline
  concept exists yet) — expected; the TOUR wording must stay honest ("the
  absence at slot 5 is the point"); do not fake a missed-run marker.

## Maintenance notes

- Parallax-side consumers: plan 046 (field explorer demo), advisor-plans
  /032 (gap detector should eventually flag the rogue log + dangling links
  + duplicate cron ids), future telemetryQuality score.
- TOUR.md is a living doc — every future scenario plan (042/045/047/048/
  049/050) should add its beat when it lands; the plans instruct catalog
  rows, the TOUR gives them narrative order.
- Reviewer: sampler must stay parent-based (children follow the root
  decision) or the "partial trace" demo lies; cron duplicate ids must be
  identical across the two runs.
