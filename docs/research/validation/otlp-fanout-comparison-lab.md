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

> **Fallback if Rotel falls short.** Rotel is pre-1.0 (`v0.2.2`). If a specific
> backend needs a quirky exporter Rotel lacks, drop in the **OpenTelemetry
> Collector Contrib** as the hub instead (its pipelines fan out to multiple
> exporters by design). Keep the lab hub swappable behind one compose service so
> Rotel-vs-Collector is itself a comparison we get for free.

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
| **Sentry** | error tracking + tracing | **partial** OTLP | `getsentry/self-hosted` (heavy compose) | [sentry-deep-research.md](../market/sentry-deep-research.md) |

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

## Risks / open questions

- **Sentry is not cleanly OTLP-native.** Self-hosted Sentry ingests via its own
  DSN/relay; OTLP trace support exists but is partial and evolving, and there is
  no first-class OTLP logs/metrics path comparable to the others. Expect an
  adapter/relay step, or accept Sentry as a *traces+errors-only* target. Confirm
  current state before wiring. It is also the heaviest stack (~20+ containers) —
  consider a compose **profile** so Sentry is opt-in and the light lab
  (Parallax+Maple+SigNoz+OpenObserve) runs without it.
- **Rotel is pre-1.0 (`v0.2.2`).** Fan-out is documented and on-thesis, but if
  any exporter misbehaves, fall back to OTel Collector Contrib as the hub (kept
  swappable). Re-verify Rotel's exporter set at impl — it moves fast.
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
- Internal: [`docs/research/capture/otlp.md`](../capture/otlp.md),
  [`maple-deep-research.md`](../market/maple-deep-research.md),
  [`signoz-deep-research.md`](../market/signoz-deep-research.md),
  [`openobserve-deep-research.md`](../market/openobserve-deep-research.md),
  [`sentry-deep-research.md`](../market/sentry-deep-research.md)
</content>
</invoke>
