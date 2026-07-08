# Plan 045: Playground runtime lanes — Tokio metrics, JVM pressure, container limits, working Micrometer/exemplar knobs

> **Executor instructions**: Targets the **playground repository**
> (`parallax-telemetry-playground`). Follow step by step; run every
> verification. On any STOP condition, stop and report. When done, update the
> status row in the Parallax repo's `plans/README.md`.
>
> **Drift check (run first)**: in the playground repo,
> `git diff --stat ed1f975..HEAD -- libs/playground-telemetry services deploy scenarios`
> On mismatch with the excerpts below, STOP.

## Status

- **Priority**: P2
- **Effort**: L
- **Risk**: MED (a GC-pressure endpoint can destabilize a service if
  unbounded; container limits can kill services if set too tight)
- **Depends on**: plan 036 (shared-lib structure); pairs with Parallax plan
  044 (runtimeSnapshot renders what this emits)
- **Category**: direction
- **Planned at**: commit `408be17` (Parallax) / `ed1f975` (playground), 2026-07-07

## Why this matters

The brief's runtime lane (section G: "explain CPU, memory, GC, Tokio
starvation") has chaos **causes** in the playground (busy-loop, leak, lock)
but no runtime **measurements**: no Tokio runtime metrics, no JVM
GC/heap-pressure driver, no container limits — so a slow span can never be
explained by queue depth, GC pause, or throttling in any backend. Two Java
config knobs are also miswired: the custom Micrometer counter is likely
never exported (agent's Micrometer bridge not enabled) and
`management.tracing.exemplars` is a Micrometer-Tracing property that the
OTel-agent setup ignores — meaning the JVM tier, billed as "the playground's
exemplar source" (Rust SDK lacks exemplars, `libs/playground-telemetry/src/lib.rs:17-19`),
probably emits none.

## Current state

Verified at playground commit `ed1f975`.

- Rust metrics come only from `tracing` fields via `MetricsLayer`
  (`libs/playground-telemetry/src/lib.rs:79-88`); repo-wide grep for
  `tokio-metrics`/`RuntimeMonitor` is empty. Chaos causes exist:
  busy-loop `services/checkout/src/main.rs:106-112` (`?cpu_ms`), lock
  contention (`?lock`, fields at `:49-52`), leak
  `services/recommendation/src/main.rs:2-12` (`?leak=<n>` grows a
  process-held buffer).
- Java: `services/catalog/.../CatalogApplication.java:12-13,53` registers a
  Micrometer `Counter` (`catalog.product.queries`) on the Spring
  `MeterRegistry`, but nothing sets
  `OTEL_INSTRUMENTATION_MICROMETER_ENABLED` (grep in `deploy/` finds no
  micrometer env) and no OTLP Micrometer registry dependency exists — the
  counter likely never leaves the process.
- `services/catalog/src/main/resources/application.yml` and
  `services/payment/.../application.yml` both end with:

  ```yaml
  management:
    tracing:
      exemplars:
        include: all
  ```

  This is a Micrometer-Tracing/Actuator property; tracing here is done by
  the OTel **agent** (`deploy/Dockerfile.java` uses the upstream agent), so
  the property is inert. Agent-native exemplars are controlled by
  `OTEL_METRICS_EXEMPLAR_FILTER` (default `trace_based` in recent agents —
  verify against the agent version pinned in `deploy/Dockerfile.java`).
  `services/fulfillment/.../application.yml` lacks the block entirely
  (drift).
- `deploy/docker-compose.yml` sets no `mem_limit`/cpus on any service
  (`:30-148`), so container-limit/throttling scenarios can't fire.
- JVM services have no heap-pressure/GC-storm endpoint (checked catalog +
  payment controllers).

## Commands you will need

| Purpose | Command (playground root) | Expected |
|---------|---------------------------|----------|
| Rust build | `rtk cargo build` | exit 0 |
| Rust lint | `rtk cargo clippy --all-targets -- -D warnings` | exit 0 |
| Java build | the repo's gradle wrapper per service (check for a root `gradlew`) | exit 0 |
| Compose | `docker compose -f deploy/docker-compose.yml config` | exit 0 |

## Scope

**In scope** (playground repo):
- `libs/playground-telemetry/` (+workspace `Cargo.toml`): `tokio-metrics` →
  OTel gauges module
- `services/checkout` or one representative Rust service to host the Tokio
  saturation scenario (blocking-pool flood knob)
- `services/catalog` (heap-pressure endpoint; Micrometer bridge env),
  `services/payment` (env only), `services/fulfillment` (yml alignment)
- `deploy/docker-compose.yml` (Java env: micrometer/exemplar; one
  memory-limited service variant), `deploy/Dockerfile.java` only if the
  agent version must be read (no changes expected)
- `scenarios/`: `a22-tokio-saturation.sh`, `b19-jvm-gc-pressure.sh`,
  `b20-container-oom.sh` + catalog rows (plan 037 format)

**Out of scope**:
- Rust exemplars (SDK-blocked — advisor-plans/033 handles the Parallax side
  with JVM/synthetic data).
- Continuous profiling (Sentry profile config already present; OTel
  profiles are future per the brief).
- Parallax UI (plan 044).

## Git workflow

- Playground repo, `main`, Conventional Commits, `git commit -s`, one

## Steps

### Step 1: Tokio runtime metrics → OTel gauges

Add `tokio-metrics` (latest stable) to the workspace; in
`libs/playground-telemetry` add `pub fn spawn_runtime_metrics()` (called
from `init` or opt-in per service — prefer automatic in `init` behind a
default-on env `PLAYGROUND_TOKIO_METRICS=1`): a task that samples
`tokio_metrics::RuntimeMonitor` every 5s and records OTel gauges named per
the brief's hook list (`tokio.runtime.workers_count`,
`tokio.runtime.alive_tasks`, `tokio.runtime.blocking_pool_depth`,
`tokio.runtime.budget_forced_yield_count`, plus poll/schedule durations as
histograms where the monitor exposes them). NOTE: `RuntimeMonitor` needs the
runtime handle — take `tokio::runtime::Handle::current()` inside the task.
Check tokio-metrics' current API for which metrics need the (unstable)
`tokio_unstable` cfg: emit ONLY the ones available on stable — list the
skipped ones in a code comment; do NOT enable `tokio_unstable` build flags.

**Verify**: `rtk cargo build` + clippy → exit 0; run any Rust service
locally and confirm `tokio.runtime.*` gauges arrive at the OTLP target
(Parallax `metricNames(prefix: "tokio.")` or debug exporter). Record it.

### Step 2: Tokio saturation scenario

Add a `?block_ms=<n>&block_n=<m>` knob to checkout (or inventory — pick the
service where the busy-loop already lives, keep chaos co-located): spawns
`m` `tokio::task::spawn_blocking` sleeps of `n` ms to flood the blocking
pool. Write `scenarios/a22-tokio-saturation.sh`: baseline traffic, then the
flood + concurrent checkouts — outcome: `tokio.runtime.blocking_pool_depth`
spikes align with slow checkout spans. Add the catalog row ("Check in
Parallax: Services → checkout → Runtime lane (plan 044): blocking-pool
spike; Traces: slow spans in the same window").

**Verify**: `bash -n` the script; live run recorded (gauge spike visible).

### Step 3: JVM pressure endpoint + GC scenario

In catalog, add a bounded chaos endpoint
(`GET /chaos/heap?mb=<n>&holdMs=<m>`, cap `mb` at e.g. 256 and `holdMs` at
30s — enforce the caps server-side): allocates `n` MiB in chunks, holds,
releases. The OTel Java agent already exports `jvm.*` runtime metrics by
default (verify one arrives — `jvm.memory.used`). Write
`scenarios/b19-jvm-gc-pressure.sh` driving repeated allocations while
querying `products` — outcome: `jvm.gc.*`/`jvm.memory.used` climb aligns
with slower GraphQL spans.

**Verify**: Java build exit 0; live check recorded.

### Step 4: Fix the Micrometer + exemplar knobs

1. Add to the Java compose env (all three services):
   `OTEL_INSTRUMENTATION_MICROMETER_ENABLED: "true"` and
   `OTEL_METRICS_EXEMPLAR_FILTER: "trace_based"`.
2. Remove the inert `management.tracing.exemplars` block from catalog +
   payment yml (replace with a comment pointing at the env var); align
   fulfillment's yml logging pattern with the other two while there (it
   lacks the trace-correlated pattern — copy catalog's `logging.pattern`
   block).
3. Confirm `catalog.product.queries` now reaches the OTLP target after a
   few `products` queries.

**Verify**: compose config exit 0; live check: the custom counter and one
exemplar-bearing histogram visible at the backend (exemplars visible via
Parallax once advisor-plans/033 lands — until then verify via the agent
debug/logging exporter or note the limitation honestly).

### Step 5: Container-limit scenario

In compose, give ONE service a demo-profile memory limit — recommendation
(its `?leak` knob makes it the natural OOM candidate):

```yaml
  recommendation:
    ...
    mem_limit: 128m
```

Gate it behind the `demo` profile via an override block or a
`docker-compose.limits.yml` overlay (decide by what compose version
supports cleanly; an overlay file keeps the default stack unlimited).
Write `scenarios/b20-container-oom.sh`: repeated `?leak=` calls until the
container OOM-kills and restarts — outcome: container exit + gap in
telemetry + (post plan 036) ERROR/absence visible; the script must print a
warning that it kills a container and require `--yes`.

**Verify**: `docker compose ... config` with the overlay renders the limit;
live run recorded (container restarts; stack self-heals).

## Test plan

- Rust: unit test for the gauge-name constants (they match the brief's
  naming — cheap string test guards typos); build/clippy gates.
- Scenario scripts: `bash -n` + live runs recorded per plan 037's catalog
  discipline.
- Java: build gate; no unit harness exists (state it).

## Done criteria

- [ ] `tokio.runtime.*` gauges emitted by every Rust service (opt-out env)
- [ ] a22/b19/b20 scenarios exist, indexed in `scenarios/run.sh` + README,
      each with recorded live outcome
- [ ] `OTEL_INSTRUMENTATION_MICROMETER_ENABLED` + `OTEL_METRICS_EXEMPLAR_FILTER`
      set for all Java services; inert yml exemplar blocks removed;
      fulfillment yml aligned
- [ ] `catalog.product.queries` visible at the OTLP target (recorded)
- [ ] Rust + Java builds green; compose config green
- [ ] Status row updated in Parallax repo `plans/README.md`

## STOP conditions

- tokio-metrics' stable-API surface yields fewer than 3 usable runtime
  metrics without `tokio_unstable` — report what's available before
  shrinking the scenario.
- The pinned OTel Java agent version doesn't support the Micrometer bridge
  env var (check its docs for the exact property name/casing) — report the
  version + correct knob rather than guessing.
- The heap-pressure endpoint destabilizes catalog even with caps —
  reduce caps; if still unstable, STOP and report.

## Maintenance notes

- Parallax plan 044's runtimeSnapshot renders these families; advisor-plans
  /033 consumes the JVM exemplars this plan switches on — coordinate
  verification once both land.
- The b20 OOM scenario intentionally kills a container — keep it out of any
  future CI loop.
- Reviewer: gauge sampling task must not keep the process alive on shutdown
  (abort on `Telemetry::shutdown`); heap-endpoint caps enforced server-side,
  not just documented.
