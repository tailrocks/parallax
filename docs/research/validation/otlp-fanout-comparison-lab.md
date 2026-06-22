# OTLP Fan-Out Comparison Lab

Research date: 2026-06-22
Status: design proposal (no code yet)

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

```
                         ┌──────────────┐
  app / SDK / parallax   │              │──► Parallax    (OTLP in)
  ──── OTLP ───────────► │    Rotel     │──► Maple       (OTLP in)
   (one endpoint)        │  (fan-out)   │──► SigNoz      (OTLP in)
                         │              │──► OpenObserve (OTLP in)
                         │              │──► Sentry      (OTLP in *)
                         └──────────────┘
                * Sentry OTLP support is partial — see Risks.
```

Everything lives in one `docker compose` stack on non-conflicting host ports,
plus a Parallax `start` flag that swaps the OTLP endpoint it injects from "itself"
to "Rotel", so Parallax's own data and any app it launches also fan out.

## Why Rotel is the right hub (verified)

Rotel is a Rust-native OTLP collector — on-thesis with Parallax, already tracked
in [`docs/research/capture/otlp.md`](../capture/otlp.md). The capability this
whole idea depends on is **multiple exporters with fan-out**, which Rotel
supports natively (verified 2026-06-22 against the streamfold/rotel README):

- Declare exporters: `ROTEL_EXPORTERS=name:type,name:type,...`
  (CLI: `--exporters name:type,...`).
- Configure each: `ROTEL_EXPORTER_{NAME}_{PARAMETER}`
  (e.g. `ROTEL_EXPORTER_PARALLAX_ENDPOINT=http://parallax:4317`).
- Fan-out per signal: `ROTEL_EXPORTERS_TRACES=a,b,c`,
  `ROTEL_EXPORTERS_METRICS=...`, `ROTEL_EXPORTERS_LOGS=...` — list several
  comma-separated exporters and each gets a copy.
- Exporter types available: OTLP (gRPC/HTTP), ClickHouse, Datadog, AWS X-Ray,
  AWS EMF, Kafka, File, Blackhole.
- Receivers: OTLP/gRPC, OTLP/HTTP, OTLP/HTTP-JSON, Kafka. Default receiver ports
  `4317` (gRPC) / `4318` (HTTP).

So Rotel listens on the standard `4317/4318`, and every backend is just another
OTLP exporter in the fan-out list. Per-signal routing means we can also do
asymmetric experiments (e.g. send logs only to OpenObserve, traces to all).

> **Hub is Rotel, full stop** (operator, 2026-06-22). No OTel Collector Contrib
> substitution. The lab needs a *simple* fan-out, not exotic processing — Rotel
> at `v0.2.2` is fast enough and on-thesis (Rust, matches Parallax). If an
> exporter detail is missing, fix forward in config or upstream, consistent with
> the repo's no-fallback-engine ethos. Collector Contrib is not part of this lab.

## Backends in scope

Decided set (operator, 2026-06-22): **Parallax + Maple + SigNoz + OpenObserve +
Sentry**. ("Cygnus" was the operator's codename for **SigNoz**.) Coroot and
Gonzo are out for now; both are easy to add later as extra exporter targets.

| Backend | What it is | OTLP-native? | Local deploy | Already researched |
|---|---|---|---|---|
| **Parallax** | this project | yes (target) | Cargo binary `parallax serve` | — |
| **Maple** | OTLP-native, ClickHouse, near-identical stack (TanStack/Bun/Turso, MCP) | yes | Docker Compose **or** single Bun binary (`libchdb`) | [maple-deep-research.md](../market/maple-deep-research.md) |
| **SigNoz** | OTLP-native full-stack obs, ClickHouse + bundled otel-collector | yes | Docker Compose | [signoz-deep-research.md](../market/signoz-deep-research.md) |
| **OpenObserve** | OTLP-native logs/metrics/traces, Rust, single binary | yes | Docker single container | [openobserve-deep-research.md](../market/openobserve-deep-research.md) |
| **Sentry** | error tracking + tracing | yes, OTLP **traces + logs** (no metrics) | `getsentry/self-hosted` (heavy compose) | [sentry-deep-research.md](../market/sentry-deep-research.md) |

Maple is the highest-signal comparison — it is the closest thing to Parallax in
the market (OTLP-native, ClickHouse, MCP, same UI stack), so identical input →
side-by-side view is the most directly instructive.

## Port allocation (no conflicts)

The central problem: **every OTLP backend wants `4317`/`4318`**. We solve it by
giving *Rotel* the only `4317/4318` the host exposes; every backend's OTLP
receiver stays on the **container network only** (not published to host), reached
by service name (`http://signoz-otel-collector:4317`, etc.). Only the **UIs** and
debugging endpoints get distinct host ports.

| Service | Host port(s) published | Purpose |
|---|---|---|
| **Rotel (hub)** | `4317` (gRPC), `4318` (HTTP) | the one endpoint everything points at |
| Parallax UI | `8080` | dashboard (+ GraphQL, per serve banner) |
| Parallax OTLP | container-only `4317/4318` | fan-out target |
| Maple UI | `8081` | dashboard |
| Maple OTLP | container-only `4318` | fan-out target |
| SigNoz UI | `3301` | dashboard |
| SigNoz OTLP collector | container-only `4317/4318` | fan-out target |
| OpenObserve UI | `5080` | dashboard |
| OpenObserve OTLP | container-only `5081` (gRPC) / `5080` (HTTP) | fan-out target |
| Sentry web | `9000` | dashboard |
| Sentry ingest (relay) | container-only | fan-out target (partial OTLP) |

Ports above are the published defaults of each project — **treat as a starting
map, verify and lock at implementation time**. The invariant that matters:
*Rotel owns host `4317/4318`; everyone else is reached internally by service
name; UIs get unique host ports.*

## Parallax `start` → Rotel env injection

Today's behavior (per spec / serve banner work): Parallax self-instruments and a
launched app exports OTLP to Parallax's own receiver. The operator's ask is a
switch so that, when the lab is on, Parallax injects **Rotel's** OTLP endpoint
instead of its own — so Parallax's self-telemetry *and* any app it starts flow
into Rotel, which then fans back out to Parallax plus everyone else.

Concretely, a flag/env on `parallax start` (names TBD at impl):

```
parallax start --otlp-forward rotel        # or: PARALLAX_OTLP_FORWARD=http://localhost:4317
```

When set, instead of injecting its own endpoint into the child process env,
Parallax injects the **standard OTEL env pointing at Rotel**:

```
OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317
OTEL_EXPORTER_OTLP_PROTOCOL=grpc
OTEL_SERVICE_NAME=<unchanged>
OTEL_RESOURCE_ATTRIBUTES=<unchanged + parallax.lab=1>
```

Key design points:

1. **Rotel must include Parallax in its fan-out list**, so swapping the endpoint
   to Rotel does not cut Parallax off — Parallax still receives everything, just
   via the hub. (`ROTEL_EXPORTERS_TRACES=parallax,maple,signoz,openobserve,sentry`.)
2. **Use only standard `OTEL_EXPORTER_OTLP_*` env**, nothing Parallax-proprietary
   — that is precisely what makes the same toggle work for *any* SDK/app, not
   just Parallax-aware ones.
3. **Off by default.** Normal `parallax start` keeps pointing at Parallax. The
   forward mode is a lab/dev affordance, gated behind the flag/env.
4. **Tag the stream** (`parallax.lab=1`, run id) so each backend's copy is
   identifiable and we can line up the same trace across all five UIs.

## Docker Compose setup (what to build)

Rotel-only hub. One repo folder (e.g. `lab/otlp-fanout/`) holding a
`docker-compose.yml`, a `rotel.env`, and per-backend config. Bring it up with
`docker compose up`; Sentry stays behind a profile.

### Services

| Service | Image / build | Host ports | Profile | Notes |
|---|---|---|---|---|
| `rotel` | `rotel-dev/rotel` (pin tag) | `4317`, `4318` | default | the only published OTLP ports; all config via `rotel.env` |
| `parallax` | build from this repo (`parallax serve`) | `8080` (UI/GraphQL) | default | OTLP `4317/4318` **internal only**; data dir volume |
| `maple` | Maple self-host image **or** single Bun binary container | `8081` | default | OTLP `4318` internal; chDB/ClickHouse volume |
| `signoz` | SigNoz stack (clickhouse + query-service + otel-collector + frontend) | `3301` | default | its collector OTLP `4317/4318` internal; pull SigNoz's own compose via `include:` |
| `openobserve` | `openobserve/openobserve` (pin tag) | `5080` | default | OTLP `5081` gRPC / `5080` HTTP internal; data volume; set root user/pass env |
| `sentry-*` | `getsentry/self-hosted` (many services) | `9000` (web) | `sentry` | huge; vendor via `include:` + profile; pin version ≥ native-OTLP release |

Hard rule: **only `rotel` publishes `4317/4318` to the host.** Every backend's
OTLP receiver stays on the compose network, reached by service name. UIs get
unique host ports. Multi-service backends (SigNoz, Sentry) are pulled in with
Compose `include:` rather than hand-recopying their service graphs.

### Rotel fan-out config (`rotel.env`)

```dotenv
# Receivers: standard OTLP in (the one endpoint apps point at)
ROTEL_OTLP_GRPC_ENDPOINT=0.0.0.0:4317
ROTEL_OTLP_HTTP_ENDPOINT=0.0.0.0:4318

# Declare every backend as an OTLP exporter
ROTEL_EXPORTERS=parallax:otlp,maple:otlp,signoz:otlp,openobserve:otlp,sentry:otlp

# Per-exporter endpoints (internal service names, not host ports)
ROTEL_EXPORTER_PARALLAX_ENDPOINT=http://parallax:4317
ROTEL_EXPORTER_PARALLAX_PROTOCOL=grpc
ROTEL_EXPORTER_MAPLE_ENDPOINT=http://maple:4318
ROTEL_EXPORTER_MAPLE_PROTOCOL=http
ROTEL_EXPORTER_SIGNOZ_ENDPOINT=http://signoz-otel-collector:4317
ROTEL_EXPORTER_SIGNOZ_PROTOCOL=grpc
ROTEL_EXPORTER_OPENOBSERVE_ENDPOINT=http://openobserve:5081
ROTEL_EXPORTER_OPENOBSERVE_PROTOCOL=grpc
# Sentry: HTTP only, non-standard base path, DSN-derived auth header
ROTEL_EXPORTER_SENTRY_ENDPOINT=http://sentry-web:9000/api/1/integration/otlp
ROTEL_EXPORTER_SENTRY_PROTOCOL=http
ROTEL_EXPORTER_SENTRY_CUSTOM_HEADERS=X-Sentry-Auth=<from DSN>   # verify exact env at impl

# Per-signal fan-out. Sentry omitted from metrics (no OTLP metrics).
ROTEL_EXPORTERS_TRACES=parallax,maple,signoz,openobserve,sentry
ROTEL_EXPORTERS_LOGS=parallax,maple,signoz,openobserve,sentry
ROTEL_EXPORTERS_METRICS=parallax,maple,signoz,openobserve
```

Exact env names (`ROTEL_OTLP_*`, `ROTEL_EXPORTER_*_CUSTOM_HEADERS`,
OpenObserve's required auth params) must be **re-verified against the pinned
Rotel/OpenObserve versions** at implementation — Rotel is pre-1.0 and moves.

### Wiring rules

- **No host OTLP port except Rotel's.** Removes the 4317/4318 collision that
  otherwise kills the whole stack.
- **Pin every image tag** (no `latest` ambiguity in a comparison lab — but follow
  the repo version policy: resolve newest mutually-compatible stable and record
  the tags in the compose).
- **Volumes per backend** so data survives `down`/`up` for repeat inspection.
- **`depends_on` + healthchecks** so Rotel starts after sinks are listening
  (otherwise early spans drop).
- **Profiles:** default = light lab (4 backends). `--profile sentry` adds Sentry.
  `--profile loadgen` adds the fixture generator.
- **One `.env`** at the lab root for shared knobs (image tags, root creds,
  Sentry DSN/version).

## Comparison workflow

1. `docker compose up` the lab (hub + 5 backends).
2. `parallax start --otlp-forward rotel <demo-app>` (or point any OTel SDK at
   `localhost:4317`). Optionally a scripted load generator emitting a fixed,
   versioned fixture set (reuse the OTLP conformance fixtures referenced in
   [`otlp.md`](../capture/otlp.md)).
3. Open all five UIs (`8080/8081/3301/5080/9000`).
4. For the *same* trace/error/log, record per backend: what fields survived,
   how errors were grouped, trace waterfall fidelity, log↔trace correlation,
   metrics rollups, query ergonomics, and MCP/agent surface (Maple & Parallax).
5. Feed findings back into the market matrices
   ([competitive-comparison-matrix.md](../market/competitive-comparison-matrix.md),
   [observability-feature-matrix.md](../market/observability-feature-matrix.md))
   and into Parallax capture/UI work.

This also doubles as **OTLP conformance evidence** for the L4 "Rotel
equivalence" gate already defined in `otlp.md`: same fixtures, Rotel hop,
equivalent normalized rows.

## Sentry OTLP — how it actually works (verified 2026-06-22)

Sentry *does* speak OTLP now; the lab can treat it as a near-first-class target.

- **Native OTLP ingest, open beta.** Sentry exposes a real OTLP HTTP ingest path
  (not just an SDK that wraps OTel). Two pieces exist: (a) SDK-side OTel
  integration (SpanProcessor/Propagator, "POTEL") that maps OTel spans to Sentry
  data and links errors/logs to traces via an `external_propagation_context`;
  and (b) a server **OTLP ingest endpoint** that accepts raw OTLP from any SDK or
  collector. For the lab we only care about (b).
- **Signals: traces + logs. No metrics** ("Sentry does not support OTLP metrics
  at this time"). This is the one asymmetry in the fan-out — handled cleanly by
  Rotel per-signal routing (exclude `sentry` from `ROTEL_EXPORTERS_METRICS`).
- **Transport: OTLP/HTTP.** Endpoint paths are *non-standard*:
  `/api/{PROJECT_ID}/integration/otlp/v1/traces` and `.../v1/logs`. Point an
  exporter at the base `…/integration/otlp` and it appends `/v1/traces|logs` —
  matching Rotel's OTLP/HTTP behavior. (gRPC not relied upon; use HTTP.)
- **Auth: `X-Sentry-Auth` header derived from the project DSN** (or, when fronted
  by a collector, the collector handles auth). Rotel's OTLP exporter must send a
  custom header → verify Rotel's custom-header env at impl.
- **Self-hosted has it.** `getsentry/self-hosted` issue #3830 ("Add Native OTLP
  Ingestion") is **closed**; native OTLP shipped in self-hosted around `25.8.0`/
  `25.10.0`. So our self-hosted Sentry can be a direct Rotel exporter target —
  pin a self-hosted version ≥ that. Confirm exact version + whether Relay or a
  bundled collector terminates OTLP when wiring.

## Risks / open questions

- **Sentry quirks, not blockers.** Non-standard OTLP path + `X-Sentry-Auth`
  header + no OTLP metrics + open-beta status. All handled (base-path exporter,
  custom header, per-signal routing). Pin a self-hosted version with native OTLP.
  Sentry is still the **heaviest** stack (~20+ containers) → keep it behind a
  compose **profile** so the light lab (Parallax+Maple+SigNoz+OpenObserve) runs
  without it.
- **Rotel is pre-1.0 (`v0.2.2`) — accepted** (operator, 2026-06-22). Simple
  fan-out only; no Collector fallback. Re-verify Rotel's exporter/header set at
  impl — it moves fast.
- **Resource weight.** Five backends, several with their own ClickHouse, on one
  laptop is heavy. Mirror the benchmarking rule's two-tier idea: light default
  profile on the laptop, full set (incl. Sentry) on a server.
- **Fan-out is not load testing.** Rotel duplicates payloads; this lab is for
  *behavioral/feature* comparison, not throughput numbers. Keep perf claims in
  the four-build benchmark track, not here.
- **Clock/ID alignment.** To diff "the same event" across UIs, pin trace/span
  ids and timestamps from a fixture generator rather than live random data.
- **Port defaults drift.** The table is a starting map; lock real published
  ports per backend version at implementation.

## Suggested phasing

1. **Hub-only smoke** — Rotel + Parallax + one backend (Maple). Prove fan-out
   reaches two sinks from one endpoint.
2. **Light lab** — add SigNoz + OpenObserve. Lock the port map; add the
   `parallax start --otlp-forward` flag.
3. **Full lab** — add Sentry behind a compose profile; resolve its OTLP/relay
   path.
4. **Fixture + diff harness** — versioned OTLP fixtures + a script that pulls the
   same trace from each backend's API and tabulates field survival → feeds the
   market matrices.
5. **Server tier** — move the full set to a server for sustained comparison runs.

## Sources

- [Rotel](https://rotel.dev) · [streamfold/rotel README](https://github.com/streamfold/rotel)
  (multiple exporters / fan-out / per-signal routing, verified 2026-06-22)
- [maple.dev](https://maple.dev/) · [Makisuo/maple](https://github.com/Makisuo/maple)
- [SigNoz](https://signoz.io/) · [OpenObserve](https://openobserve.ai/) · [Sentry self-hosted](https://github.com/getsentry/self-hosted)
- Sentry OTLP: [docs.sentry.io/concepts/otlp](https://docs.sentry.io/concepts/otlp/) ·
  [develop.sentry.dev OTLP integration](https://develop.sentry.dev/sdk/telemetry/traces/otlp/) ·
  [self-hosted #3830 native OTLP (closed)](https://github.com/getsentry/self-hosted/issues/3830)
- Internal: [`docs/research/capture/otlp.md`](../capture/otlp.md),
  [`maple-deep-research.md`](../market/maple-deep-research.md),
  [`signoz-deep-research.md`](../market/signoz-deep-research.md),
  [`openobserve-deep-research.md`](../market/openobserve-deep-research.md),
  [`sentry-deep-research.md`](../market/sentry-deep-research.md)
</content>
</invoke>
