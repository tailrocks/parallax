# Plan 042: Make release attribution and feature flags real in the playground — versioned resources, live flagd wiring, honest A13

> **Executor instructions**: Targets the **playground repository**
> (`parallax-telemetry-playground`). Follow step by step; run every
> verification. On any STOP condition, stop and report. When done, update the
> status row in the Parallax repo's `plans/README.md`.
>
> **Drift check (run first)**: in the playground repo,
> `git diff --stat ed1f975..HEAD -- libs/playground-telemetry services flags scenarios deploy`
> On mismatch with the excerpts below, STOP.

## Status

- **Priority**: P1 (feeds Parallax plan 041's demo)
- **Effort**: M
- **Risk**: LOW
- **Depends on**: plan 036 (shared-lib helpers; resource plumbing) — land 036
  first
- **Category**: direction
- **Planned at**: commit `408be17` (Parallax) / `ed1f975` (playground), 2026-07-07

## Why this matters

The playground's change-attribution demos are currently fake in three ways.
(1) The telemetry `release`/`service.version` is hardcoded to the crate
version, so the A13 "release-attributed regression" emits identical version
attrs for v1 and v2 — no backend can attribute the regression. (2) The a13
script never even sets `RELEASE`; it drives `?fail=1` and prints a claim
about markers that are not emitted. (3) All five flagd flags are inert: no
service reads them (the checkout `flag()` helper reads **env vars**), and the
only OpenFeature consumer reads an undefined flag and gates nothing
(`promo ? CATALOG : CATALOG`). Fixing this gives Parallax a true
flip-a-flag-and-watch / deploy-regression story — the brief's domain H — and
gives plan 041's releases lane real data.

## Current state

Verified at playground commit `ed1f975`.

- Hardcoded version/release —
  `libs/playground-telemetry/src/lib.rs:64` (resource):
  `KeyValue::new(SERVICE_VERSION, env!("CARGO_PKG_VERSION"))`;
  `lib.rs:115` (Sentry): `release: Some(env!("CARGO_PKG_VERSION").into())`.
  `RELEASE` env is read only for behavior branching:
  `services/checkout/src/main.rs:135-137`:

  ```rust
  // B12: release-attributed regression — RELEASE=v2 fails (vs v1 clean).
  let release_regressed = std::env::var("RELEASE").as_deref() == Ok("v2");
  if p.fail || flag("PAYMENT_FAILURE") || release_regressed {
  ```

- `flag()` is env, not flagd — `services/checkout/src/main.rs:86-88`:

  ```rust
  fn flag(name: &str) -> bool {
      std::env::var(name).is_ok_and(|v| v == "1" || v.eq_ignore_ascii_case("true"))
  }
  ```

- flagd defines 5 unconsumed flags — `flags/flagd.json`: `paymentFailure`,
  `slowQuery`, `cacheLeak`, `poisonMessage`, `canaryFailure` (all
  boolean, default off). flagd runs in compose with its gRPC port 8013
  published (`deploy/docker-compose.yml:85-90`).

- The one OpenFeature consumer is a no-op on an undefined flag —
  `services/catalog/.../CatalogApplication.java:62-63`:

  ```java
  boolean promo = flags.getBooleanValue("catalogPromo", false);
  return promo ? CATALOG : CATALOG;
  ```

  (`catalogPromo` is absent from flagd.json.) No OpenTelemetry OpenFeature
  hook is registered (no `addHooks` call; `build.gradle.kts` declares only
  the OpenFeature SDK + flagd provider — verify at `:23-24` region), so
  evaluations emit no `feature_flag.*` span events.

- a13 doesn't exercise releases — `scenarios/a13-deploy-regression.sh`:

  ```bash
  echo "v1 (clean):";    curl -sS "$BASE/checkout" -o /dev/null -w " [%{http_code}]\n"
  echo "v2 (regressed):"; curl -sS "$BASE/checkout?fail=1" -o /dev/null -w " [%{http_code}]\n"
  echo "deploy/release markers + commit sha are emitted as resource attrs (RELEASE env)."
  ```

  The last line is aspirational — nothing emits those attrs today.

- Environment is a single value everywhere: `PARALLAX_ENV: "playground"` +
  `deployment.environment.name=playground` in the compose `x-otlp` anchor
  (`deploy/docker-compose.yml:27-28`).

- `releases/README.md` is a stub promising v1/v2 artifacts.

## Commands you will need

| Purpose | Command (playground root) | Expected |
|---------|---------------------------|----------|
| Build | `rtk cargo build` | exit 0 |
| Lint | `rtk cargo clippy --all-targets -- -D warnings` | exit 0 |
| Java build | `cd services/catalog && ./gradlew build -x test` (check the wrapper path — a root `gradlew` may serve all Java services) | exit 0 |
| Compose validate | `docker compose -f deploy/docker-compose.yml config` | exit 0 |

## Scope

**In scope** (playground repo):
- `libs/playground-telemetry/src/lib.rs` (version/release/env sourcing)
- `services/checkout/src/main.rs` (flagd client for the failure flags OR a
  documented env fallback — see Step 3)
- `services/catalog/` (define + actually gate on `catalogPromo`; OTel
  OpenFeature hook; `build.gradle.kts`)
- `flags/flagd.json` (add `catalogPromo`)
- `scenarios/a13-deploy-regression.sh` (make honest), one new
  `scenarios/a14-flag-flip.sh`
- `deploy/docker-compose.yml` (+xlang overlay): `RELEASE` env plumbing, a
  second environment name option
- `releases/README.md` (reflect reality)
- `scenarios/run.sh` + `scenarios/README.md` rows (plan 037's catalog)

**Out of scope**:
- Deploy webhook ingest into Parallax (041's deferred item).
- Rust exemplars, sampling (plan 054), GraphQL family (plan 047).
- A full OpenFeature Rust SDK adoption if it turns out heavyweight — the
  STOP condition covers this.

## Git workflow

- Playground repo, `main`, Conventional Commits, `git commit -s`, one
  `Co-authored-by: Claude <noreply@anthropic.com>` trailer. Push when done.

## Steps

### Step 1: Source version/release/environment from env in the shared lib

In `libs/playground-telemetry/src/lib.rs`:
- `service.version` resource attr = `RELEASE` env if set, else
  `env!("CARGO_PKG_VERSION")` (one helper: `fn release() -> String`).
- Sentry `release` = the same value (`lib.rs:115`).
- Add `vcs.ref.head.revision` resource attr from `GIT_SHA` env if set
  (compose can pass it; absent → omit).
- (deployment.environment.name handling already lands with plan 036 Step 1 —
  if 036 not yet landed, add it here and note the overlap.)

**Verify**: `rtk cargo build && rtk cargo clippy --all-targets -- -D warnings`
→ exit 0.

### Step 2: Honest A13 — two releases with distinct resource attrs

1. `deploy/docker-compose.yml`: checkout service env gets
   `RELEASE: "${RELEASE:-v1}"` (and `GIT_SHA: "${GIT_SHA:-}"`).
2. Rewrite `scenarios/a13-deploy-regression.sh`:
   - Phase 1: ensure stack runs with `RELEASE=v1`, drive N clean checkouts.
   - Phase 2: `RELEASE=v2 docker compose ... up -d checkout` (recreate just
     the checkout container with the new env), drive N checkouts — the
     existing `release_regressed` branch (`main.rs:136`) makes v2 fail on
     its own, so drop the `?fail=1` crutch.
   - Print what to check: "Parallax → Issues: error spike attributed to
     service.version=v2; Services → checkout: release strip shows v1→v2"
     (the strip is Parallax plan 041).
   - End by restoring `RELEASE=v1` (leave the stack clean).
3. Update `releases/README.md` to describe this mechanism (env-driven
   release identity; no fake artifact promises).

**Verify**: `bash -n scenarios/a13-deploy-regression.sh` → exit 0; with a
running stack, phase 2 spans carry `service.version=v2` (check via Parallax
`sql` surface or the OTLP debug of your choice; record the check).

### Step 3: Wire the failure flags to flagd (Rust side)

Decide by inspection: the OpenFeature Rust SDK + flagd provider crate
maturity (check crates.io for `open-feature` and a flagd provider at latest
stable). 
- If a stable pair exists: add to checkout only (not all services), evaluate
  `paymentFailure` (and `slowQuery` if trivial) per request instead of the
  `PAYMENT_FAILURE` env var; keep `fn flag()` env override as the fallback
  (`flagd value || env`), because scenario scripts use env today.
- If NOT stable: keep env vars as the mechanism, but make flagd the source
  by adding a tiny poller in checkout that reads flagd's HTTP/gRPC eval
  endpoint every 10s into an `AtomicBool` — only if this stays under ~50
  lines with no heavyweight deps. Otherwise STOP (see conditions).
Either way, emit a `feature_flag.evaluation` **span event** (fields:
`feature_flag.key`, `feature_flag.provider_name="flagd"`,
`feature_flag.variant`) at each evaluation on the active span, so the
Parallax story/trace views can show flag context.

**Verify**: `rtk cargo build` + clippy → exit 0; with the stack up, flipping
`paymentFailure` in `flags/flagd.json` (flagd watches the file) flips
checkout failures within ~10s — record the check. Add
`scenarios/a14-flag-flip.sh` that automates: healthy burst → flip flag on
(edit flagd.json via `sed` or curl flagd's admin if available — file edit is
fine for the lab) → error burst → flip back. Register it in
`scenarios/run.sh` + README (plan 037 catalog).

### Step 4: Make `catalogPromo` real (Java side)

1. Add `catalogPromo` to `flags/flagd.json` (boolean, default off).
2. In `CatalogApplication.java`, make the flag gate something observable:
   e.g. `promo` → returned products get a `promoPrice` field or the list is
   reordered/discounted — any deterministic, visible-in-span-attrs change
   (add `catalog.promo=true` span attribute at minimum).
3. Register the OpenTelemetry OpenFeature hook so evaluations emit
   `feature_flag.*` span events: add the
   `io.opentelemetry.instrumentation:opentelemetry-openfeature-*` hook
   artifact if one exists for the installed agent version, else register a
   ~15-line custom `Hook` that adds the span event via the agent's bound
   `Span.current()`. Verify which the current OTel Java agent supports
   before choosing (read the agent docs for its OpenFeature support).

**Verify**: Java build exit 0; with the stack up, a `products` GraphQL query
(curl the catalog port with a raw GraphQL POST) while flipping
`catalogPromo` shows the changed behavior and the span event — record it.

### Step 5: Second environment name (cheap, completes domain H)

In compose, parameterize the anchor: 
`OTEL_RESOURCE_ATTRIBUTES: "deployment.environment.name=${PLAYGROUND_ENV:-playground}"`
and `PARALLAX_ENV: "${PLAYGROUND_ENV:-playground}"`, so
`PLAYGROUND_ENV=staging docker compose up` produces a second environment.
Document in `.env.example` (plan 037 file — append if it exists, note if
not).

**Verify**: `docker compose -f deploy/docker-compose.yml config` with and
without `PLAYGROUND_ENV=staging` renders the respective values.

## Test plan

- Rust: a unit test in `libs/playground-telemetry` for the release-sourcing
  helper (env set → env value; unset → crate version).
- The scenario scripts are the integration tests: a13 (release
  attribution), a14 (flag flip) — each records its Parallax-visible outcome
  per plan 037's catalog format.
- Java: existing build gate; no new unit-test harness required (note if the
  repo has none).

## Done criteria

- [ ] `rtk cargo build` + clippy zero warnings; Java services build
- [ ] `RELEASE=v2` run emits `service.version=v2` on checkout spans
      (recorded check)
- [ ] `scenarios/a13-deploy-regression.sh` drives two real releases and no
      longer prints unimplemented claims
- [ ] Flipping `paymentFailure` in flagd changes checkout behavior without
      restarts (recorded); `feature_flag.evaluation` span events emitted
- [ ] `catalogPromo` defined in flagd.json, consumed, and observable;
      OpenFeature evaluations visible as span events on catalog spans
- [ ] `PLAYGROUND_ENV` switches the emitted environment name
- [ ] a13/a14 rows present in `scenarios/run.sh` + `scenarios/README.md`
- [ ] Status row updated in Parallax repo `plans/README.md`

## STOP conditions

- No stable OpenFeature Rust SDK/flagd provider AND the minimal poller
  exceeds ~50 lines or needs a new heavyweight dep — STOP; ship Steps 1/2/4/5
  and report the Rust-flag gap with the version matrix you found.
- The OTel Java agent version in `deploy/Dockerfile.java` has no OpenFeature
  hook support and the custom Hook can't reach the agent's Span — report
  with the agent version; don't downgrade the agent silently.
- Recreating a single compose service with different env proves flaky in the
  a13 script — simplify to full stack restart and note the tradeoff.

## Maintenance notes

- Parallax plan 041 (releases lane) is the consumer — after both land, the
  a13 scenario is the demo for its release strip + regression badge.
- Plan 047 (GraphQL family) touches the same catalog service — land this
  first or coordinate.
- Reviewer: flag evaluations must be span **events** (not logs) per the
  brief's semconv table; check `feature_flag.key` cardinality stays low.
- Deferred: `deployment.id`/`deployment.name` semconv attrs and a real
  deploy-marker event stream — revisit once Parallax ingests deploy events
  (041's deferred webhook).
