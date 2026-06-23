# OTLP Fan-Out Comparison Lab

Research date: 2026-06-22
Status: design proposal (no code yet)
Topology: Parallax runs on the host (Homebrew); Rotel + competitor backends run
in Docker Compose; Rotel fans out across the host↔container boundary.
Deep review: 2026-06-22 (two passes) — every external claim verified against live
sources and every Parallax-side claim checked against `crates/`; corrections
folded in. Items needing unbuilt Parallax features are marked
**[NOT YET IMPLEMENTED]**.
Updates 2026-06-23: added the compare-mode `parallax run start` DevEx design;
**only Parallax runs on the host — Rotel, Maple, SigNoz, OpenObserve, Sentry all
run in Docker Compose.** Maple runs fully local as a **chDB-binary container (not
Tinybird)**. The only Parallax-side support needed is forwarding to the collector;
Rotel reaches the single host sink (Parallax) via `host.docker.internal:14317`.
**All five backends are now implemented and verified live (2026-06-23):**
OpenObserve, SigNoz, Maple (chDB), and Sentry (v26.6.0, its own vendored stack
reached over the host bridge `host.docker.internal:9000`). Sentry is no longer
deferred — see `bench/otlp-fanout/sentry/`.

## Goal

Run several observability backends side by side on one machine, feed them **the
exact same OpenTelemetry stream**, and compare — for identical input — how each
one *ingests*, *stores*, *views*, and *exposes* the data. The purpose is to
sharpen Parallax: see what competitors capture that we drop, how they group and
present errors/traces/logs, and where Parallax can do better.

The mechanism is a single **fan-out hop**: every emitter points at one endpoint
(a [Rotel](https://rotel.dev) collector); Rotel duplicates each trace/metric/log
to *all* backends at once, Parallax included. One input, N synchronized copies,
zero per-backend re-instrumentation.

**Topology decision (operator, 2026-06-22): Parallax runs on the host via
Homebrew, NOT inside Compose.** Compose holds only Rotel + the competitor
backends. This fits the repo's Homebrew packaging policy and lets you
develop/run the real `parallax` binary on macOS while the lab stays a disposable
container stack. Two consequences fall out of it, both handled below: (1) Rotel
(in a container) must reach **back to the host** to deliver Parallax's copy — via
`host.docker.internal`; (2) host-native Parallax and host-published Rotel can't
both own `4317/4318` — so Parallax's OTLP receiver is offset to `14317/14318`.

```
   HOST (macOS) — Parallax only        DOCKER COMPOSE (everything else)
  ┌──────────────────────────┐    ┌────────────────────────────────────────────┐
  │ parallax                 │◄─┐ │  ┌──────────┐                               │
  │  UI/API :4000            │  │ │  │  Rotel   │── maple:4318  (chDB binary)   │
  │  OTLP   :14317/:14318    │  │ │  │ fan-out  │── otel-collector:4317         │ (SigNoz)
  │  greptime 24000-24003    │  │ │  │ :4317/   │── openobserve:5081            │
  │            ▲             │host│ │  │ :4318    │── nginx:80 → relay            │ (Sentry)
  │ parallax run start child ┼─►│ │ │  └────┬─────┘                               │
  └────────────┼────────────┘docker└───────┼ host.docker.internal:14317 ─────────┘
   forward → localhost:4317  internal       └─► parallax  (the only host sink)
```

**Topology (operator, 2026-06-23): only Parallax runs on the host; everything
else — Rotel, Maple, SigNoz, OpenObserve, Sentry — runs in Docker Compose.** So:
**every emitter sends to one shared host address — Rotel at `localhost:4317/4318`**.
Rotel fans out to every Compose backend by service name and back *out* to the
**single host sink, Parallax** (`host.docker.internal:14317`). Parallax is both an
emitter (forwards into Rotel) and a sink (gets its copy out of Rotel) — it just
lives on the host. Because everything else is in Docker and reachable through the
collector's fan-out, **the only Parallax-side support we need is the ability to
send to the collector — nothing else.** Maple still runs **fully local (its chDB
binary, NOT Tinybird)** — just containerized rather than host-resident.

## Parallax-side prerequisites (what's built vs not)

Some lab assumptions are already config-only (BUILT); two depend on unbuilt
Parallax features (verified against `crates/parallax-cli` +
`crates/parallax-server`, 2026-06-22). The port offset and `0.0.0.0` bind are
config edits you can make today — only the child-telemetry forward switch and
self-telemetry are genuinely unbuilt:

| Need | Current reality (`crates/`) | Gap |
|---|---|---|
| Offset OTLP ports `14317/14318` | **BUILT** — `config.toml` keys `otlp_grpc_port` / `otlp_http_port` (default `4317/4318`); `parallax serve` takes only `--config` | config-only edit; **no CLI flags** (`--otlp-grpc`/`--otlp-http` don't exist) — set the ports in `~/.parallax/config.toml` |
| Bind OTLP on `0.0.0.0` | **BUILT** — `config.toml` key `bind` (default `127.0.0.1`); `serve` binds all three listeners (api/otlp_http/otlp_grpc) to it | config-only edit; set `bind = "0.0.0.0"`. No code change needed |
| Forward child telemetry to Rotel | `parallax run start` injects a **hardcoded** `http://127.0.0.1:4317` into the child env (`commands.rs`) | a `--otlp-forward` switch is **[NOT YET IMPLEMENTED]**. *Lucky accident:* the hardcoded `127.0.0.1:4317` already equals Rotel's published port, so child apps reach Rotel today with zero changes |
| Parallax self-telemetry into the lab | `parallax serve` only **receives** OTLP; it does not emit its own spans/logs anywhere | self-instrumentation is **[NOT YET IMPLEMENTED]**; until built, Parallax's *own* internal traces cannot fan out |
| Install via Homebrew | repo policy: stable formula is **disabled** pre-release; only a rolling `parallax-preview` exists | use `brew install tailrocks/parallax/parallax-preview` (or run from a local checkout), **not** `brew install parallax` |

The correct invocations given today's code:

```
# host: offset Parallax OTLP ports + bind 0.0.0.0 via config, then serve
#   ~/.parallax/config.toml:
#     bind = "0.0.0.0"          # default 127.0.0.1 — must be 0.0.0.0 for Rotel to reach it
#     otlp_grpc_port = 14317
#     otlp_http_port = 14318
brew install tailrocks/parallax/parallax-preview
parallax serve --config ~/.parallax/config.toml      # UI/API on :4000

# launch a child app under Parallax (child telemetry → 127.0.0.1:4317 = Rotel today)
parallax run start -- <demo-app>
```

## Why Rotel is the right hub (verified)

Rotel is a Rust-native OTLP collector — on-thesis with Parallax, already tracked
in [`docs/research/capture/otlp.md`](../capture/otlp.md). The capability this
whole idea depends on is **multiple exporters with fan-out**, which Rotel
supports natively (verified 2026-06-22 against `streamfold/rotel-docs`):

- Declare exporters: `ROTEL_EXPORTERS=name:type,name:type,...`
  (CLI: `--exporters name:type,...`). Name optional (defaults to type).
- Configure each: `ROTEL_EXPORTER_{NAME}_{PARAMETER}` (env-only, no CLI form):
  `_ENDPOINT`, `_PROTOCOL` (`grpc`|`http`), `_CUSTOM_HEADERS` (comma-separated
  `key=value`). **There is no `_TLS_INSECURE`** — the skip-verify option is
  `_TLS_SKIP_VERIFY`.
- Fan-out per signal: `ROTEL_EXPORTERS_TRACES=a,b,c`,
  `ROTEL_EXPORTERS_METRICS=...`, `ROTEL_EXPORTERS_LOGS=...` — comma-separated
  list, each listed exporter gets a **copy** (true fan-out, confirmed in docs).
- Exporter types: OTLP (gRPC/HTTP), ClickHouse, Datadog, AWS X-Ray, AWS EMF,
  Kafka, File, Blackhole.
- Receivers: OTLP/gRPC, OTLP/HTTP, OTLP/HTTP-JSON, Kafka. Env
  `ROTEL_OTLP_GRPC_ENDPOINT` / `ROTEL_OTLP_HTTP_ENDPOINT`; defaults bind
  **`localhost`** `4317`/`4318` → must override to `0.0.0.0:4317/4318` so the
  container's published ports are reachable.
- Defaults: batching (`--batch-max-size 8192`, `--batch-timeout 200ms`), retries
  on for 429/timeout (backoff 5s→30s, max-elapsed 300s), `5s` request timeout,
  `gzip`.

> **Fan-out is sequential** (operational caveat). Rotel docs: "telemetry is sent
> sequentially to the sending queues for each exporter in-order." A slow or down
> backend can back-pressure the others. Keep retries/queue on; in a lab this is
> tolerable, but it means one wedged backend can delay every backend's copy.

> **Hub is Rotel, full stop** (operator, 2026-06-22). No OTel Collector Contrib
> substitution. Simple fan-out, not exotic processing — Rotel `v0.2.2` (image
> `streamfold/rotel`) is fast enough and on-thesis (Rust). Fix forward if an
> exporter detail is missing.

## Backends in scope

Decided set (operator, 2026-06-22): **Parallax + Maple + SigNoz + OpenObserve +
Sentry**. ("Cygnus" was the operator's codename for **SigNoz**.) Coroot and
Gonzo are out for now; both are easy to add later as extra exporter targets.

| Backend | What it is | OTLP-native? | Local deploy | Already researched |
|---|---|---|---|---|
| **Parallax** | this project | yes (target) | **host (Homebrew preview tap) `parallax serve`** — not in Compose | — |
| **Maple** | OTLP-native, ClickHouse-engine, near-identical stack (TanStack/Bun/Turso, MCP) | yes | **Compose container running Maple's chDB single-binary (local mode) — NOT Tinybird** (operator, 2026-06-23) | [maple-deep-research.md](../market/maple-deep-research.md) |
| **SigNoz** | OTLP-native full-stack obs, ClickHouse + bundled otel-collector | yes | git clone + `deploy/docker` compose (bundles ClickHouse + ZooKeeper) | [signoz-deep-research.md](../market/signoz-deep-research.md) |
| **OpenObserve** | OTLP-native logs/metrics/traces, Rust, single binary | yes | Docker single container (`public.ecr.aws/zinclabs/openobserve`) | [openobserve-deep-research.md](../market/openobserve-deep-research.md) |
| **Sentry** | error tracking + tracing | yes, OTLP **traces + logs** (no metrics), open beta | `getsentry/self-hosted` (**~72 services**, `install.sh`) | [sentry-deep-research.md](../market/sentry-deep-research.md) |

Maple is the highest-signal comparison — closest to Parallax (OTLP-native,
ClickHouse, MCP, same UI stack), so identical input → side-by-side view is the
most directly instructive.

## Host ↔ Compose topology and port plan

Two networks meet: the **host** (Parallax + your apps) and the **Compose
network** (Rotel + competitor backends). Rules that keep it conflict-free:

1. **Rotel is the single shared emit endpoint, published on host `4317/4318`.**
   Only Rotel publishes OTLP ports to the host. Every emitter sends to
   `localhost:4317` (gRPC) / `:4318` (HTTP). This is "that host address used
   everywhere."
2. **Parallax (the only host process) offsets its OTLP receiver to `14317/14318`**
   via `config.toml` (`4317/4318` belong to Rotel). UI `4000`, greptime
   `24000-24003`. Nothing addresses Parallax's OTLP directly except Rotel, so the
   offset is invisible to users.
3. **Every Compose backend (Maple, SigNoz, OpenObserve, Sentry) keeps OTLP on the
   Compose network only**, reached by service name; only their **UIs** publish to
   host on distinct ports.

The cross-boundary hop (the lab's one fragile hop): Rotel reaches host-resident
Parallax via **`host.docker.internal:14317`** — Docker Desktop macOS/Windows
built-in; Linux add `extra_hosts: ["host.docker.internal:host-gateway"]`.

> **Hard rule — Parallax MUST bind its OTLP listener on `0.0.0.0`.** This is the
> single point of failure for the whole lab. A `127.0.0.1`-only bind is
> definitively unreachable from a container on Linux, and unreliable on Docker
> Desktop Mac (version-dependent). The failure is silent and asymmetric: **every
> Compose backend gets the trace except Parallax**, because only Parallax is
> reached across the host bridge. Phase-1 must assert the Parallax copy arrived.

| Where | Component | Address used by others | Notes |
|---|---|---|---|
| **host** | **Rotel receiver (shared)** | `localhost:4317` / `localhost:4318` | the one endpoint every emitter points at |
| **host** | Parallax UI / API / GraphQL | `localhost:4000` | dashboard (default `api_port`) |
| **host** | Parallax OTLP (sink) | `host.docker.internal:14317` (from Rotel) | offset via `config.toml`; **bind `0.0.0.0`** |
| **host** | Parallax GreptimeDB child | `127.0.0.1:24000-24003` | managed by `serve`; keep free |
| compose | Maple UI | `localhost:8081` → container UI port | container runs the chDB binary (local mode) |
| compose | Maple OTLP | `maple:4318` HTTP (internal) | chDB-binary receiver, **no auth**, chDB data in a volume — no Tinybird |
| compose | SigNoz UI | `localhost:3301` → container `:8080` | SigNoz UI is now `:8080`; republish to `3301` on host |
| compose | SigNoz OTLP collector | `otel-collector:4317` (internal) | **service** name is `otel-collector` (`signoz-otel-collector` is only the container_name) |
| compose | OpenObserve UI | `localhost:5080` | `ZO_HTTP_PORT` |
| compose | OpenObserve OTLP | `openobserve:5081` gRPC (internal) | `ZO_GRPC_PORT`; auth + org/stream headers required |
| compose | Sentry entry (nginx) | `localhost:9000` (`SENTRY_BIND`) | nginx, not a `sentry-web` service |
| compose | Sentry OTLP ingest | `nginx:80/api/<projectId>/integration/otlp` → `relay:3000` | OTLP terminated by **Relay**, routed by nginx |

Ports/service-names are version-dependent — **verify and lock at
implementation**. Invariants: *Rotel owns host `4317/4318`; Parallax-host uses
`14317/14318` + `4000` (+ greptime `24000-24003`); backends expose only UIs on
unique host ports; Rotel reaches Parallax via `host.docker.internal`.*

## Compare mode: `parallax run start` → Rotel fan-out (DevEx design)

This is the operator's headline feature. Restating the goal in the operator's
words: running e.g. `parallax run start -- jackin --debug` normally sends Jackin's
telemetry **only to Parallax**. We want an **ambient setting** that flips this so
Parallax injects **Rotel's** endpoint instead of its own — Rotel then fans the
same stream out to **every backend including Parallax** — so you can open all five
UIs and compare, on identical data, how each renders it and decide what to build
into Parallax.

### What already exists (verified in `crates/parallax-cli/src/commands.rs`)

`parallax run start -- <cmd>` today already does the hard parts:

- mints a `run_id`, records `runStart`/`runFinish`, captures the child exit code;
- injects the **full standard OTel env into the child** — `OTEL_EXPORTER_OTLP_ENDPOINT`
  **plus per-signal** `_TRACES_/_LOGS_/_METRICS_/_PROFILES_ENDPOINT` and matching
  `_PROTOCOL`s, **plus** `OTEL_RESOURCE_ATTRIBUTES=parallax.run.id=<id>`;
- bare mode (no `-- <cmd>`) prints the same as `export` lines to `source`.

The **only** thing hardcoded is the destination: a single const
`OTLP_GRPC_ENDPOINT = "http://127.0.0.1:4317"`. So compare mode is a *small,
surgical change* — make that destination resolvable from ambient config — not a
new subsystem. Everything else (run-id stamping, per-signal env, standard-OTel
approach so it works for any SDK/language, not just Parallax-aware apps) is done.

### The DevEx — how the user turns it on

Design goal: **the command line never has to change.** `parallax run start --
jackin --debug` is identical whether comparing or not; an ambient setting decides
where telemetry goes. Resolution precedence (highest wins):

1. **Per-invocation flag** — `parallax run start --otlp-forward <target> -- …`.
   `<target>` ∈ a URL · `rotel` (the configured lab hub) · `off`/`parallax`
   (force the default even if ambient config says forward). For one-off overrides.
2. **Global env var (the operator's primary surface)** — `PARALLAX_OTLP_FORWARD`.
   Set it once in the shell/profile and *every* `parallax run start` forwards:
   - a URL → use it (`PARALLAX_OTLP_FORWARD=http://localhost:4317`);
   - `1`/`true`/`rotel` → use the configured rotel endpoint;
   - `off` → force default.
   This is exactly the "set a system environment, and Parallax sends to Rotel
   instead of itself" model. **Discoverability:** the lab folder ships a
   `lab.env` (or `docker compose ... config` helper) printing the exact
   `export PARALLAX_OTLP_FORWARD=http://localhost:4317` line to `source`, so the
   endpoint comes straight from the compose, not memory.
3. **Config file** — `~/.parallax/config.toml` `[run].otlp_forward` /
   `[lab].rotel_endpoint`. **DEFERRED (operator, 2026-06-23): v1 ships env + flag
   ONLY** — no new `config.rs` section yet; add the config-file surface later if
   the ambient env proves insufficient.
4. **Respect a pre-existing child OTel endpoint** — if the environment `run start`
   inherits *already* has `OTEL_EXPORTER_OTLP_ENDPOINT` set, **don't clobber it**.
   This is the idiomatic OTel escape hatch: `export
   OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317` alone already forwards to
   Rotel, and Parallax should defer to it.
5. **Default (today's behavior)** — inject Parallax's own receiver
   (`http://127.0.0.1:4317`, or the configured `otlp_grpc_port`).

> One value, two jobs avoided: keep "where to send" (an endpoint) separate from
> "compare mode is on" (presence of forward). Presence of any forward setting =
> compare mode. No separate boolean needed, though `PARALLAX_COMPARE=1` could be
> offered as sugar for "forward to configured rotel."

### What gets injected in compare mode

Same env block as today, with the destination swapped to the forward endpoint and
two extra resource attributes for cross-UI alignment:

```
OTEL_EXPORTER_OTLP_ENDPOINT       = <forward endpoint>      # e.g. http://localhost:4317
OTEL_EXPORTER_OTLP_{TRACES,LOGS,METRICS,PROFILES}_ENDPOINT = <forward endpoint>
OTEL_EXPORTER_OTLP_PROTOCOL (+per-signal) = grpc            # http if endpoint is :4318
OTEL_RESOURCE_ATTRIBUTES = parallax.run.id=<id>,parallax.lab=1,deployment.environment.name=<env>
```

`parallax.run.id` (already injected) + `parallax.lab=1` make the *same run*
findable in every backend's UI — the key to a fair side-by-side. Protocol follows
the endpoint port (`:4317`→grpc, `:4318`→http) or an explicit
`PARALLAX_OTLP_FORWARD_PROTOCOL`.

### Banner — make the mode obvious

`run start` must announce where telemetry is going (repo progress-visibility
rule). Compare mode:

```
Parallax run id: 1a2b3c
command: jackin --debug
telemetry → Rotel (fan-out) http://localhost:4317   [COMPARE MODE]
   ↳ parallax · maple · signoz · openobserve · sentry
live: parallax run watch 1a2b3c   ·   compare UIs: :4000 :8081 :3301 :5080 :9000
```

Default mode just says `telemetry → Parallax http://127.0.0.1:4317`. The contrast
makes it impossible to forget which mode you're in.

### Supporting DevEx details

- **Dry-run / introspection.** `parallax run start --otlp-forward rotel
  --print-env -- jackin` (or bare mode, which already prints exports) prints the
  exact env it *would* inject without running — debug the forward without launching
  the app. Bare mode must honor the same precedence (so
  `PARALLAX_OTLP_FORWARD=… parallax run start` prints Rotel exports).
- **Pre-flight reachability.** In compare mode, TCP-probe the forward endpoint and
  **warn (don't fail)** if Rotel is down: `⚠ Rotel http://localhost:4317
  unreachable — telemetry may be dropped`. Forwarding makes Rotel a dependency for
  *all* backends incl. Parallax, so a silent dead hub = "nothing shows anywhere."
- **Parallax must be in Rotel's fan-out list** so forwarding doesn't cut Parallax
  off: `ROTEL_EXPORTERS_TRACES=parallax,maple,signoz,openobserve,sentry`, the
  `parallax` exporter → `host.docker.internal:14317`. Forward sends the child to
  Rotel; Rotel sends a copy back to Parallax.
- **Standard OTel only.** Inject nothing Parallax-proprietary — that is what makes
  the toggle work for Jackin and any other OTel app/SDK/language unchanged.
- **Off-switch per invocation** — `--otlp-forward off` (or `PARALLAX_OTLP_FORWARD=off`)
  forces the default endpoint even when config enables forward.

### Scope note

This covers **child-process** telemetry (Jackin, demo apps) — which is exactly the
compare use case. Parallax's **own self-telemetry** is separate and **[NOT YET
IMPLEMENTED]**; if/when it ships, route it to Rotel too, but suppress Parallax's
ingest-path spans from its self-telemetry exporter to avoid a
self→Rotel→self feedback loop (tagging identifies copies, it does not break the
loop).

## Docker Compose setup (what to build)

**Parallax is the ONLY host process; everything else runs in Compose** — Rotel,
Maple, SigNoz, OpenObserve, Sentry. Put the lab under **`bench/otlp-fanout/`** (the
repo already uses `bench/` for compose-based smoke stacks; `lab/` is not a
registered top-level dir — if you prefer `lab/`, add it to `PROJECT_STRUCTURE.md`
in the same change). The folder holds `docker-compose.yml`, `rotel.env`, and
per-backend config. `docker compose up`; Sentry behind a profile. Start Parallax
separately on the host (see prerequisites / workflow step 0).

### Services (Compose only — Parallax is on the host)

| Service | Image / build | Host ports | Profile | Notes |
|---|---|---|---|---|
| `rotel` | `streamfold/rotel` (Docker Hub, pin tag) | `4317`, `4318` | default | the only published OTLP ports; config via `rotel.env`; `extra_hosts` on Linux to reach host Parallax |
| `maple` | container running Maple's **chDB single-binary** (local mode) — small image wrapping the binary; **no Tinybird** | `8081`→UI | default | OTLP `4318` HTTP internal, **no auth**; chDB data in a named volume |
| `signoz` | `include:` SigNoz `deploy/docker` (signoz + otel-collector + clickhouse + zookeeper) | `3301`→`8080` | default | **override to unpublish its host `4317/4318`** (see Hard rule); collector service `otel-collector` |
| `openobserve` | `public.ecr.aws/zinclabs/openobserve` (pin tag) | `5080` | default | OTLP `5081` gRPC internal; set `ZO_ROOT_USER_EMAIL`/`ZO_ROOT_USER_PASSWORD`; **ingest needs auth headers** (see `rotel.env`) |
| `sentry-*` | `getsentry/self-hosted` (**~72 services**, `install.sh`) | `9000` (nginx) | own stack | **not a clean `include:` target** — runs as its **own vendored Compose stack** (`bench/otlp-fanout/sentry/setup.sh`); Rotel reaches it over the **host bridge** `host.docker.internal:9000` → nginx → relay (no network-join needed). **IMPLEMENTED + verified live 2026-06-23 on v26.6.0** (A1 OTLP ingest + A15/A16 grouping). Pin ≥ native-OTLP (`~25.8.0`); default `SENTRY_REF=26.6.0` |
| `loadgen` | small OTel SDK / `telemetrygen` container | — | `loadgen` | optional fixed-fixture emitter → `rotel:4317`; pins trace/span ids for cross-UI diffing |

Hard rule: **only `rotel` publishes `4317/4318` to the host.** Every competitor
backend's OTLP receiver stays on the Compose network; UIs get unique host ports.
Parallax (host) is reached *out* of Compose via `host.docker.internal:14317`.

> **`include:` carries upstream port mappings.** SigNoz's stock compose
> **publishes its own `4317/4318` to the host** (verified on `main`), colliding
> with Rotel. When you `include:` it, add a Compose **override** that unpublishes
> those (`ports: []` / drop the host side) so only Rotel keeps host `4317/4318`.
> Same for any UI/ingest port you don't want on the host. SigNoz also now pushes
> a "Foundry" install path; manual compose is the fallback and may add
> PostgreSQL + ClickHouse-Keeper. Sentry self-hosted can't be `include:`d at all
> (see its row) — separate stack joined on a shared network.

### Rotel fan-out config (`rotel.env`)

```dotenv
# Receivers: bind 0.0.0.0 so the container's published ports are reachable
ROTEL_OTLP_GRPC_ENDPOINT=0.0.0.0:4317
ROTEL_OTLP_HTTP_ENDPOINT=0.0.0.0:4318

# Declare every backend as an OTLP exporter
ROTEL_EXPORTERS=parallax:otlp,maple:otlp,signoz:otlp,openobserve:otlp,sentry:otlp

# Parallax is the only HOST sink → host.docker.internal; everything else is a
# Compose service → internal service name.
ROTEL_EXPORTER_PARALLAX_ENDPOINT=http://host.docker.internal:14317
ROTEL_EXPORTER_PARALLAX_PROTOCOL=grpc
ROTEL_EXPORTER_MAPLE_ENDPOINT=http://maple:4318   # chDB-binary container, HTTP-only
ROTEL_EXPORTER_MAPLE_PROTOCOL=http
ROTEL_EXPORTER_SIGNOZ_ENDPOINT=http://otel-collector:4317   # service name, not container_name
ROTEL_EXPORTER_SIGNOZ_PROTOCOL=grpc
ROTEL_EXPORTER_OPENOBSERVE_ENDPOINT=http://openobserve:5081
ROTEL_EXPORTER_OPENOBSERVE_PROTOCOL=grpc
# OpenObserve ingest REQUIRES auth + org/stream routing (else rejected):
ROTEL_EXPORTER_OPENOBSERVE_CUSTOM_HEADERS=Authorization=Basic <b64 email:password>,organization=default,stream-name=default
ROTEL_EXPORTER_OPENOBSERVE_TLS_SKIP_VERIFY=true
# Sentry: HTTP only; OTLP terminated by Relay behind nginx. Project id from DSN.
# 25.10.0+ path is /api/<proj>/integration/otlp ; 25.8.0 used /api/<proj>/otlp (no "integration/").
ROTEL_EXPORTER_SENTRY_ENDPOINT=http://nginx:80/api/<projectId>/integration/otlp
ROTEL_EXPORTER_SENTRY_PROTOCOL=http
ROTEL_EXPORTER_SENTRY_CUSTOM_HEADERS=x-sentry-auth=sentry sentry_key=<DSN public key>

# Per-signal fan-out. Sentry omitted from metrics (no OTLP metrics).
ROTEL_EXPORTERS_TRACES=parallax,maple,signoz,openobserve,sentry
ROTEL_EXPORTERS_LOGS=parallax,maple,signoz,openobserve,sentry
ROTEL_EXPORTERS_METRICS=parallax,maple,signoz,openobserve
```

Exact env spellings were verified against `streamfold/rotel-docs` (2026-06-22);
re-verify at the pinned Rotel version since it is pre-1.0. Maple runs its **chDB
single-binary in local mode inside the container** — its OTLP receiver
(`maple:4318`, HTTP) takes **no auth** (the key-protected ingest gateway `:3474`
is a Tinybird/compose-build concern, not used here), so no Maple header is needed.

### Wiring rules

- **No host OTLP port except Rotel's.** Rotel publishes host `4317/4318`;
  Parallax-host uses offset `14317/14318` (via `config.toml`). Removes the
  collision between the two host-side OTLP listeners.
- **Rotel → host Parallax via `host.docker.internal:14317`.** Docker Desktop
  (macOS/Windows) built-in; Linux add
  `extra_hosts: ["host.docker.internal:host-gateway"]` to the `rotel` service,
  and **allow the docker-bridge subnet through the host firewall** (ufw/firewalld
  often blocks it). Non-Docker-Desktop runtimes differ: **Colima** may refuse the
  connection / need a manual host-IP; **OrbStack** supports it (except in
  host-networking mode); **Podman** uses `host.containers.internal`.
- **Parallax binds OTLP on `0.0.0.0`** (hard rule above).
- **Enable gRPC keepalive** on the Parallax exporter path — NAT/host-gateway can
  silently drop idle gRPC streams.
- **Pin every image tag** (follow repo version policy: newest mutually-compatible
  stable, recorded in the compose).
- **Volumes per backend** so data survives `down`/`up`.
- **Ordering across the host boundary is not enforceable by `depends_on`.**
  `depends_on`/healthchecks only order Compose-internal sinks; the **host
  Parallax sink is invisible to Compose**. Start host Parallax *before* `docker
  compose up`, and rely on Rotel's retry/queue so early Parallax-bound spans
  aren't lost.
- **Profiles:** default = core lab (Parallax host + Maple + SigNoz + OpenObserve).
  `--profile sentry` adds Sentry. `--profile loadgen` adds the fixture generator.
  Consider profile-gating SigNoz too (it carries ClickHouse + ZooKeeper).
- **One `.env`** at the lab root for shared knobs (image tags, root creds, Sentry
  DSN/version).

## Comparison workflow

0. Host (Parallax only): `brew install tailrocks/parallax/parallax-preview`; in
   `~/.parallax/config.toml` set `bind = "0.0.0.0"`, `otlp_grpc_port = 14317`,
   `otlp_http_port = 14318`; `parallax serve --config ~/.parallax/config.toml`
   (UI `:4000`).
1. `docker compose up` the lab (Rotel hub + Maple + SigNoz + OpenObserve;
   Parallax already up on the host; Sentry behind `--profile sentry`).
2. Emit telemetry into Rotel (`localhost:4317`). **Interim payload (operator,
   2026-06-23): OTel `telemetrygen`** — synthetic traces/metrics/logs, zero app to
   build, lights up all backends immediately. Later: `parallax run start
   --otlp-forward rotel -- <demo-app>`, or the full playground (separate repo).
3. Open all five UIs — Parallax `localhost:4000` (host) + Maple `:8081`, SigNoz
   `:3301`, OpenObserve `:5080`, Sentry `:9000` (Compose).
4. For the *same* trace/error/log, **manually** open each backend's UI and
   eyeball: what fields survived, how errors were grouped, trace waterfall
   fidelity, log↔trace correlation, metrics rollups, query ergonomics, MCP/agent
   surface (Maple & Parallax). A field that looks "missing" may be *renamed*, not
   dropped.
5. Feed observations back into the market matrices
   ([competitive-comparison-matrix.md](../market/competitive-comparison-matrix.md),
   [observability-feature-matrix.md](../market/observability-feature-matrix.md))
   and into Parallax capture/UI work.

> **Scored comparison harness is DEFERRED** (operator, 2026-06-23) — no automated
> per-backend extraction / scoring rubric / pinned-id diffing as part of this
> build. Comparison is **manual** (open the UIs). A future harness (per-backend
> read APIs, preserved/renamed/dropped scoring, recorded `semconv_version`) can be
> added if we want quantitative results; it is distinct from otlp.md's L4
> conformance gate.

## Sentry OTLP — how it actually works (verified 2026-06-22)

Sentry speaks OTLP; the lab treats it as a near-first-class target.

- **Native OTLP ingest, open beta.** A real server OTLP HTTP path (not just an
  SDK wrapping OTel). For the lab we use the server endpoint.
- **Signals: traces + logs. No metrics** ("Sentry does not support OTLP metrics
  at this time") — handled by excluding `sentry` from `ROTEL_EXPORTERS_METRICS`.
- **Transport: OTLP/HTTP.** Path `/api/<projectId>/integration/otlp/v1/{traces,logs}`
  (point at base `…/integration/otlp`, signal auto-appended). HTTP only, no gRPC.
- **Auth header value is specific:** `x-sentry-auth: sentry sentry_key=<DSN
  public key>` (not the raw DSN). Project id comes from the DSN. When fronted by
  a collector, omit the header (the collector handles auth).
- **Self-hosted ingest path (corrected):** there is **no `sentry-web` service**.
  `SENTRY_BIND=9000` publishes the **`nginx`** container; OTLP requests
  (`^/api/<id>/...`) are routed by nginx to **`relay:3000`**, which terminates
  OTLP. Point Rotel at `http://nginx:80/api/<projectId>/integration/otlp` (front
  door) or directly at `http://relay:3000/...`.
- **Self-hosted enablement:** `getsentry/self-hosted` #3830 ("Add Native OTLP
  Ingestion") is **closed (2026-05-19)**; native OTLP shipped ~`25.8.0`
  (version-pinned setup guides for `25.8.0` and `25.10.0`). Requires enabling
  Performance Trace Explorer + Event Analytics Platform, adding relay/OTLP
  **feature flags to `sentry.conf.py`**, then **re-running `./install.sh`**. No
  extra service / no bundled collector — Relay owns it. **Path differs by
  version:** `25.8.0` used `/api/<id>/otlp/v1/...` (no `integration/`); the path
  gained `integration/` by `25.10.0`. Pin a version and match the path.
- **Deployment reality:** self-hosted Sentry is **~72 services** installed via
  `install.sh` that generates configs — **not** a clean Compose `include:`
  target. Run it as its own stack. **Realized (2026-06-23):** rather than joining
  Rotel to Sentry's network, Rotel reaches the published nginx front door over
  the **host bridge** (`host.docker.internal:9000`) — same hop Parallax uses —
  which avoids a cross-stack network-join entirely. Implemented in
  `bench/otlp-fanout/sentry/`.

## Risks / open questions

- **Two Parallax-side features are unbuilt.** Port offset and `0.0.0.0` bind are
  already **config-only** — usable today. **Configurable child-telemetry
  forwarding** (`--otlp-forward`/`PARALLAX_OTLP_FORWARD`) and **self-telemetry**
  are genuinely unbuilt. The "lucky accident" (hardcoded `127.0.0.1:4317` happens
  to equal Rotel's published port) lets the lab run **only** in this exact port
  arrangement, and a subtle consequence: because Parallax is offset to `14317`,
  `run start`'s hardcoded `4317` *always* hits Rotel while the lab is up — so you
  **cannot** send child telemetry to Parallax-only without the lab running. The
  configurable forward is therefore not cosmetic: it's what makes compare-mode
  *controllable* (on/off, choose endpoint) and lets `run start` work outside this
  one port layout.
- **Host↔container bridge fragility.** `host.docker.internal` depends on the
  runtime (Docker Desktop built-in; Linux `host-gateway` + firewall;
  Colima/OrbStack/Podman differ) and is the single load-bearing hop for the one
  host sink (Parallax). Misbehavior → "every Compose backend has the trace except
  Parallax." Document per-runtime setup + a phase-1 assert for the Parallax copy.
- **Sentry quirks.** Non-standard path (version-dependent), `x-sentry-auth`,
  no OTLP metrics, open beta, ~72-container `install.sh` stack, OTLP via
  nginx→relay. Keep behind a profile; run as its own stack.
- **Rotel pre-1.0 (`v0.2.2`) + sequential fan-out — accepted.** Re-verify
  exporter/header env at impl; a wedged backend can back-pressure others.
- **Resource weight.** Parallax (+ managed GreptimeDB) on the host, plus Maple
  (chDB/ClickHouse), SigNoz (ClickHouse + ZooKeeper), OpenObserve — multiple
  storage engines on one Mac. The "core lab" is laptop-*tolerable*, not light;
  full set (incl. Sentry) belongs on a server. Mirror the benchmark two-tier rule.
- **Fan-out is not load testing.** Behavioral/feature comparison only; keep perf
  claims in the four-build benchmark track.
- **Clock/ID alignment.** Pin trace/span ids + timestamps from a fixture
  generator so "the same event" is retrievable across all five read APIs.
- **Version/service-name drift.** SigNoz UI/service names, Sentry path, Maple
  ports, OpenObserve image tag — lock per pinned version at implementation.

## Suggested phasing

1. **Host-bridge smoke** — host Parallax (offset ports, `0.0.0.0`) + Rotel +
   Maple (compose). Prove the host↔container bridge: emit to `localhost:4317`,
   **assert** the copy lands in both Parallax (`host.docker.internal:14317`) and
   Maple (`maple:4318`). This single assert guards the lab's one fragile host hop.
2. **Core lab** — add SigNoz + OpenObserve in Compose (with `include:` port
   overrides + auth headers). Lock the port map; build the `parallax run start`
   compare-mode forward (the `--otlp-forward`/`PARALLAX_OTLP_FORWARD` switch).
3. **Full lab (Sentry)** — **DONE (2026-06-23, v26.6.0):** self-hosted Sentry
   runs as its own vendored stack via `sentry/setup.sh`; `sentry/onboard.sh`
   bootstraps the project/DSN and prints the `rotel.env` exports; Rotel reaches
   it over the host bridge (`host.docker.internal:9000`, no network-join);
   `sentry/verify.sh` asserts native OTLP ingest (A1) + issue grouping
   (A15/A16). Sentry is the heaviest/fiddliest piece, so phases 1–2 still stand
   up without it.
4. **Server tier** — move the full set to a server for sustained runs.

*(A scored fixture/diff harness is out of scope for now — comparison is manual,
see §Comparison workflow.)*

## Sources

- [Rotel](https://rotel.dev) · [streamfold/rotel README](https://github.com/streamfold/rotel)
  · [streamfold/rotel-docs](https://github.com/streamfold/rotel-docs) (exporters,
  multiple-exporters, base config — env names verified 2026-06-22)
- [maple.dev](https://maple.dev/) · [Makisuo/maple](https://github.com/Makisuo/maple)
  (compose build-from-source, Tinybird, ports verified)
- [SigNoz docker install](https://signoz.io/docs/install/docker/) ·
  [SigNoz compose @ main](https://github.com/SigNoz/signoz/blob/main/deploy/docker/docker-compose.yaml)
  (UI `:8080`, collector service `otel-collector`, publishes `4317/4318`)
- [OpenObserve OTLP ingestion](https://openobserve.ai/docs/ingestion/logs/otlp/) ·
  [env vars](https://openobserve.ai/docs/environment-variables/) ·
  [zinclabs/openobserve (ECR)](https://gallery.ecr.aws/zinclabs/openobserve)
- Sentry OTLP: [docs.sentry.io/concepts/otlp](https://docs.sentry.io/concepts/otlp/) ·
  [develop.sentry.dev OTLP integration](https://develop.sentry.dev/sdk/telemetry/traces/otlp/) ·
  [self-hosted #3830 (closed)](https://github.com/getsentry/self-hosted/issues/3830)
- Docker networking: [Docker Desktop networking](https://docs.docker.com/desktop/features/networking/)
  (`host.docker.internal`, `host-gateway`, `0.0.0.0` bind)
- Internal: [`docs/research/capture/otlp.md`](../capture/otlp.md),
  [`maple-deep-research.md`](../market/maple-deep-research.md),
  [`signoz-deep-research.md`](../market/signoz-deep-research.md),
  [`openobserve-deep-research.md`](../market/openobserve-deep-research.md),
  [`sentry-deep-research.md`](../market/sentry-deep-research.md);
  Parallax CLI/server: `crates/parallax-cli`, `crates/parallax-server`
</content>
