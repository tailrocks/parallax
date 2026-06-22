# OTLP Fan-Out Comparison Lab

Research date: 2026-06-22
Status: design proposal (no code yet)
Topology: Parallax runs on the host (Homebrew); Rotel + competitor backends run
in Docker Compose; Rotel fans out across the host↔container boundary.
Deep review: 2026-06-22 — every external claim verified against live sources and
every Parallax-side claim checked against `crates/`; corrections folded in. Items
that depend on unbuilt Parallax features are marked **[NOT YET IMPLEMENTED]**.

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
        HOST (macOS, Homebrew)                 DOCKER COMPOSE (lab stack)
  ┌───────────────────────────┐          ┌──────────────────────────────────┐
  │ parallax (binary)         │          │  │  Rotel   │── maple:4318         │
  │  UI/API :4000  (browser)  │          │  │ fan-out  │── otel-collector:4317│ (SigNoz)
  │  OTLP   :14317/:14318      │          │  │          │── openobserve:5081   │
  │  greptime child 24000-24003│          │  │ :4317/   │── nginx:80 → relay   │ (Sentry)
  │            ▲              │  host.    │  │ :4318    │                      │
  │ host apps / SDKs ─────────┼─►docker.──┼─►│          │                      │
  │ parallax run start child ─┼─internal──┼─►│          │                      │
  └────────────┼──────────────┘ (publish  │  └────┬─────┘                     │
   emit → localhost:4317        to host)   │       └─► host.docker.internal:14317
               └───────────────────────────────────────────────┘ → parallax OTLP
                                           └──────────────────────────────────┘
```

So: **every emitter sends to one shared host address — Rotel at
`localhost:4317/4318`**. Rotel fans out to the four backends *inside* Compose by
service name, and back *out* to host-native Parallax via
`host.docker.internal:14317`. Parallax is both an emitter (into Rotel) and a sink
(out of Rotel) — it just lives on the host instead of in a container.

## Parallax-side prerequisites (mostly NOT YET IMPLEMENTED)

The lab assumes Parallax behaviors that the committed code does **not** ship yet
(verified against `crates/parallax-cli` + `crates/parallax-server`,
2026-06-22). Treat these as work to do before the lab can run as written, not as
current behavior:

| Need | Current reality (`crates/`) | Gap |
|---|---|---|
| Offset OTLP ports `14317/14318` | ports come from `config.toml` keys `otlp_grpc_port` / `otlp_http_port` (default `4317/4318`); `parallax serve` takes only `--config` | **no CLI flags** — set the ports in `~/.parallax/config.toml`, not via `--otlp-grpc`/`--otlp-http` |
| Bind OTLP on `0.0.0.0` | confirm the receiver bind address is configurable / defaults to a host-reachable interface | verify; may need a config addition |
| Forward child telemetry to Rotel | `parallax run start` injects a **hardcoded** `http://127.0.0.1:4317` into the child env (`commands.rs`) | a `--otlp-forward` switch is **[NOT YET IMPLEMENTED]**. *Lucky accident:* the hardcoded `127.0.0.1:4317` already equals Rotel's published port, so child apps reach Rotel today with zero changes |
| Parallax self-telemetry into the lab | `parallax serve` only **receives** OTLP; it does not emit its own spans/logs anywhere | self-instrumentation is **[NOT YET IMPLEMENTED]**; until built, Parallax's *own* internal traces cannot fan out |
| Install via Homebrew | repo policy: stable formula is **disabled** pre-release; only a rolling `parallax-preview` exists | use `brew install tailrocks/parallax/parallax-preview` (or run from a local checkout), **not** `brew install parallax` |

The correct invocations given today's code:

```
# host: offset Parallax OTLP ports via config, then serve
#   ~/.parallax/config.toml:  otlp_grpc_port = 14317 ; otlp_http_port = 14318
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
| **Maple** | OTLP-native, ClickHouse, near-identical stack (TanStack/Bun/Turso, MCP) | yes | **build-from-source** Compose (`--build`, needs a Tinybird endpoint) **or** single Bun binary (`libchdb`/chDB) | [maple-deep-research.md](../market/maple-deep-research.md) |
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
2. **Parallax (host) offsets its OTLP receiver to `14317/14318`** via
   `config.toml` (`4317/4318` now belong to Rotel). Nothing addresses Parallax's
   OTLP directly except Rotel, so the offset is invisible to users. Parallax
   UI/API stays on **`4000`** (its real default), and its managed GreptimeDB
   child uses host `127.0.0.1:24000-24003` — leave those free.
3. **Every competitor backend keeps OTLP on the Compose network only**, reached
   by service name. Only their **UIs** publish to host, on distinct ports.

The cross-boundary hop: Rotel reaches host-native Parallax via
**`host.docker.internal:14317`** (Docker Desktop macOS/Windows built-in; Linux
add `extra_hosts: ["host.docker.internal:host-gateway"]`).

> **Hard rule — Parallax MUST bind its OTLP listener on `0.0.0.0`.** This is the
> single point of failure for the whole lab. A `127.0.0.1`-only bind is
> definitively unreachable from a container on Linux, and unreliable on Docker
> Desktop Mac (version-dependent). The failure is silent and asymmetric: **every
> backend gets the trace except Parallax**, because only Parallax is reached
> across the host bridge. Phase-1 must assert the Parallax copy arrived.

| Where | Component | Address used by others | Notes |
|---|---|---|---|
| **host** | **Rotel receiver (shared)** | `localhost:4317` / `localhost:4318` | the one endpoint every emitter points at |
| **host** | Parallax UI / API / GraphQL | `localhost:4000` | dashboard (default `api_port`) |
| **host** | Parallax OTLP (sink) | `host.docker.internal:14317` (from Rotel) | offset via `config.toml`; **bind `0.0.0.0`** |
| **host** | Parallax GreptimeDB child | `127.0.0.1:24000-24003` | managed by `serve`; keep free |
| compose | Maple UI | `localhost:8081` → container `:80` | native host port is `3471`; remap to avoid clashes |
| compose | Maple OTLP | `maple:4318` HTTP (or `maple:4317` gRPC) internal | otel-collector receiver, **no auth** |
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

## `parallax run start` → Rotel env injection **[NOT YET IMPLEMENTED]**

Today (`crates/parallax-cli`): `parallax run start -- <cmd>` injects a
**hardcoded** `http://127.0.0.1:4317` into the child's `OTEL_EXPORTER_OTLP_*`
env. In this lab that constant already points at Rotel's published port, so child
apps fan out today with no code change. What's missing is making the target
**configurable** so the same toggle works against any Rotel host/port and any
SDK.

Proposed switch (names TBD at impl):

```
parallax run start --otlp-forward rotel -- <demo-app>
# or: PARALLAX_OTLP_FORWARD=http://localhost:4317
```

When set, inject the **standard OTEL env pointing at Rotel**:

```
OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317
OTEL_EXPORTER_OTLP_PROTOCOL=grpc
OTEL_SERVICE_NAME=<unchanged>
OTEL_RESOURCE_ATTRIBUTES=<unchanged + parallax.lab=1>
```

Design points:

1. **Rotel includes Parallax in its fan-out list** so the data still reaches
   Parallax via the hub (`ROTEL_EXPORTERS_TRACES=parallax,maple,signoz,openobserve,sentry`;
   the `parallax` exporter targets `host.docker.internal:14317`).
2. **Use only standard `OTEL_EXPORTER_OTLP_*` env** — that is what makes the
   toggle work for any SDK/app, not just Parallax-aware ones.
3. **Off by default** — lab/dev affordance, gated behind the flag/env.
4. **Tag the stream** (`parallax.lab=1`, run id) so each backend's copy is
   identifiable across UIs. (Tagging is for *alignment*, not loop prevention —
   see below.)
5. **[NOT YET IMPLEMENTED] Parallax self-telemetry** — once Parallax emits its
   own spans, route them to Rotel too. **Loop hazard:** Parallax self-telemetry →
   Rotel → back into Parallax, whose ingest path then emits more spans → loop /
   inflated counts. Tagging does **not** break this; suppress Parallax's own
   ingest-path spans from the self-telemetry exporter, or don't route Parallax
   self-telemetry back to itself.

## Docker Compose setup (what to build)

Rotel-only hub. **Parallax is NOT a Compose service** — host via Homebrew. Put
the lab under **`bench/otlp-fanout/`** (the repo already uses `bench/` for
compose-based smoke stacks; `lab/` is not a registered top-level dir — if you
prefer `lab/`, add it to `PROJECT_STRUCTURE.md` in the same change). The folder
holds `docker-compose.yml`, `rotel.env`, and per-backend config. `docker compose
up`; Sentry behind a profile. Start Parallax separately on the host (see
prerequisites).

### Services (Compose only — Parallax is on the host)

| Service | Image / build | Host ports | Profile | Notes |
|---|---|---|---|---|
| `rotel` | `streamfold/rotel` (Docker Hub, pin tag) | `4317`, `4318` | default | the only published OTLP ports; config via `rotel.env`; `extra_hosts` on Linux to reach host Parallax |
| `maple` | **build-from-source** (`build:` per `apps/*/Dockerfile`); needs a **Tinybird endpoint** in compose mode | `8081`→`80` | default | otel-collector OTLP `4318`/`4317` internal, **no receiver auth**; UI native `3471`; metadata SQLite/libSQL |
| `signoz` | `include:` SigNoz `deploy/docker` (signoz + otel-collector + clickhouse + zookeeper) | `3301`→`8080` | default | **override to unpublish its host `4317/4318`** (see Hard rule); collector service `otel-collector` |
| `openobserve` | `public.ecr.aws/zinclabs/openobserve` (pin tag) | `5080` | default | OTLP `5081` gRPC internal; set `ZO_ROOT_USER_EMAIL`/`ZO_ROOT_USER_PASSWORD`; **ingest needs auth headers** (see `rotel.env`) |
| `sentry-*` | `getsentry/self-hosted` (**~72 services**, `install.sh`) | `9000` (nginx) | `sentry` | **not a clean `include:` target** — run as its own stack + join Rotel to its network. OTLP via `nginx:80` → `relay:3000`. Needs feature flags + re-run `install.sh`. Pin ≥ native-OTLP (`~25.8.0`) |
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

# Parallax is on the HOST → host.docker.internal; competitors are Compose
# services → internal service names.
ROTEL_EXPORTER_PARALLAX_ENDPOINT=http://host.docker.internal:14317
ROTEL_EXPORTER_PARALLAX_PROTOCOL=grpc
ROTEL_EXPORTER_MAPLE_ENDPOINT=http://maple:4318
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
re-verify at the pinned Rotel version since it is pre-1.0. Maple's `:4318`
collector has **no receiver auth** (its key-protected ingest gateway is `:3474`,
deliberately bypassed here), so no Maple header is needed.

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

0. Host: `brew install tailrocks/parallax/parallax-preview`; set
   `otlp_grpc_port=14317` / `otlp_http_port=14318` in `~/.parallax/config.toml`
   (bind `0.0.0.0`); `parallax serve --config ~/.parallax/config.toml` (UI `:4000`).
1. `docker compose up` the lab (Rotel hub + competitor backends; Parallax already
   up on the host).
2. `parallax run start -- <demo-app>` (child telemetry → `127.0.0.1:4317` = Rotel
   today; with the future `--otlp-forward`, any Rotel endpoint), or point any OTel
   SDK at `localhost:4317`. Optionally a scripted load generator emitting a fixed,
   versioned fixture set (reuse the OTLP conformance fixtures in
   [`otlp.md`](../capture/otlp.md)).
3. Open all five UIs — Parallax `localhost:4000` (host) + Maple `:8081`, SigNoz
   `:3301`, OpenObserve `:5080`, Sentry `:9000` (Compose).
4. For the *same* trace/error/log, record per backend: what fields survived, how
   errors were grouped, trace waterfall fidelity, log↔trace correlation, metrics
   rollups, query ergonomics, and MCP/agent surface (Maple & Parallax). See the
   extraction spec below — "field survival" must be defined per backend because
   each renames/normalizes differently.
5. Feed findings back into the market matrices
   ([competitive-comparison-matrix.md](../market/competitive-comparison-matrix.md),
   [observability-feature-matrix.md](../market/observability-feature-matrix.md))
   and into Parallax capture/UI work.

### Per-backend extraction (phase 4 must specify this)

The "same trace across all UIs" diff is not free: the five backends expose
different read APIs and normalize fields differently (e.g. GreptimeDB renames
metric labels; Sentry maps OTel → its own model). A field that is "missing" may
be *renamed*, not *dropped*. The harness must define, per backend, how to fetch a
known `trace_id` and which canonical field set to compare:

| Backend | Read API for a known trace_id | Normalization to watch |
|---|---|---|
| Parallax | GraphQL (`:4000`) | GreptimeDB column/label renames |
| Maple | GraphQL API (`:3472`) | Tinybird/ClickHouse schema |
| SigNoz | query API / ClickHouse SQL | OTel→ClickHouse span schema |
| OpenObserve | REST search API (`:5080`) | stream-mapped fields |
| Sentry | events/issues API | OTel→Sentry event model (lossy by design) |

This lab produces **behavioral** evidence (what each tool keeps/shows). It is
**not** the same as the OTLP conformance gate: otlp.md's L4 "Rotel equivalence"
requires hash-disciplined normalized-row/bundle/projection equality from a pinned
fixture set. The lab *shares fixtures and exercises the Rotel hop* and so feeds
L4, but does not by itself advance the conformance ledger past `not_measured`.

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
  target. Run it as its own stack and join Rotel to its network.

## Risks / open questions

- **Parallax-side features are unbuilt.** Offset OTLP ports (via config),
  configurable child-telemetry forwarding, `0.0.0.0` bind, and self-telemetry are
  prerequisites (see the prerequisites section). The lab can't run end-to-end
  until at least the port-offset + `0.0.0.0` bind exist.
- **Host↔container bridge fragility.** `host.docker.internal` depends on the
  runtime (Docker Desktop built-in; Linux `host-gateway` + firewall;
  Colima/OrbStack/Podman differ). Misbehavior → "every backend has the trace
  except Parallax." Document per-runtime setup + a phase-1 smoke assert.
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

1. **Hub-only smoke** — host Parallax (offset ports, `0.0.0.0`) + Rotel + one
   backend (Maple). Prove the host↔container bridge: emit to `localhost:4317`,
   **assert** the copy lands in both Parallax (via `host.docker.internal:14317`)
   and Maple. This single assert guards the lab's one fragile hop.
2. **Core lab** — add SigNoz + OpenObserve (with `include:` port overrides + auth
   headers). Lock the port map; build the `parallax run start --otlp-forward`
   switch.
3. **Full lab** — add Sentry as its own stack joined to Rotel's network; resolve
   feature flags + path version.
4. **Fixture + diff harness** — versioned OTLP fixtures + per-backend extraction
   (table above) tabulating field survival → feeds the market matrices.
5. **Server tier** — move the full set to a server for sustained runs.

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
