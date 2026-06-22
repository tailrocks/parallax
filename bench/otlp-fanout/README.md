# OTLP Fan-Out Comparison Lab

Feed **one** OpenTelemetry stream to several observability backends at once and
compare how each renders identical data. Design + rationale:
[`docs/research/validation/otlp-fanout-comparison-lab.md`](../../docs/research/validation/otlp-fanout-comparison-lab.md).

**Topology:** only **Parallax** runs on the host (Homebrew). Everything else runs
in Compose. **Rotel** is the single shared OTLP endpoint published on host
`4317/4318`; it fans every signal out to each backend AND back to host Parallax
via `host.docker.internal:14317`.

```
emitters ─► localhost:4317 (Rotel) ─┬─► openobserve:5081        (compose)
                                     ├─► otel-collector:4317     (SigNoz, overlay)
                                     ├─► maple:4318              (overlay, chDB)
                                     └─► host.docker.internal:14317 ─► Parallax (host)
```

## Status

- ✅ **Core (Rotel + OpenObserve)** — implemented and verified: a trace POSTed to
  Rotel's OTLP/HTTP endpoint fans out and lands in OpenObserve (stream + WAL
  write confirmed). The Parallax exporter targets the host; it simply retries
  until Parallax is up (a down sink never blocks the others).
- 🟡 **SigNoz** — overlay `compose.signoz.yml` (vendored clone via
  `setup-vendor.sh`); verify port-override + network at run.
- 🟡 **Maple** — overlay `compose.maple.yml` builds the chDB local binary from
  source (`maple/Dockerfile`); finalize the build/CMD per Maple's
  `docs/local-mode.md` (no official Linux image exists).
- ⏭ **Sentry** — deferred (heaviest: ~72-service `install.sh` stack + DSN
  bootstrap). Wire its exporter in `rotel.env` once stood up.

## Quick start (core)

```bash
cd bench/otlp-fanout
docker compose -f compose.yml up -d rotel openobserve   # OpenObserve UI: http://localhost:5080
./smoke.sh                                               # drive + assert fan-out
docker compose -f compose.yml down -v                    # teardown
```

OpenObserve default login: `root@example.com` / `Complexpass#123` (change in
`compose.yml` + the base64 `Authorization` in `rotel.env`).

## Parallax (host) — the one host sink

```bash
# ~/.parallax/config.toml:  bind = "0.0.0.0"  otlp_grpc_port = 14317  otlp_http_port = 14318
brew install tailrocks/parallax/parallax-preview   # or run from a local checkout
parallax serve --config ~/.parallax/config.toml    # UI http://localhost:4000
```

Rotel reaches it at `host.docker.internal:14317`. **Bind `0.0.0.0`** — a
loopback-only bind is unreachable from the container (the lab's one fragile hop).

## Compare mode — `parallax run start`

```bash
source bench/otlp-fanout/lab.env          # sets PARALLAX_OTLP_FORWARD=http://localhost:4317
parallax run start -- <your-otel-app>     # child telemetry → Rotel → every backend incl. Parallax
parallax run start --otlp-forward off -- <app>   # one-off: straight to Parallax
```

Implemented in `crates/parallax-cli` (env + flag; config-file deferred).

## Adding backends

```bash
./setup-vendor.sh                                   # clone SigNoz into vendor/
docker compose -f compose.yml -f compose.signoz.yml -f compose.maple.yml up -d
```

Then uncomment `maple`/`signoz` in `rotel.env` (`ROTEL_EXPORTERS` + the per-signal
lists). SigNoz UI → `http://localhost:3301`, Maple UI → `http://localhost:8081`.

## Files

| File | Purpose |
|---|---|
| `compose.yml` | core: Rotel + OpenObserve + telemetrygen (loadgen profile) |
| `rotel.env` | Rotel fan-out config (exporters, per-signal lists, auth headers) |
| `lab.env` | `source` it to put the shell in compare mode |
| `compose.signoz.yml` / `compose.maple.yml` | backend overlays |
| `maple/Dockerfile` | Maple chDB local-mode build (best-effort) |
| `setup-vendor.sh` | clone SigNoz into `vendor/` |
| `smoke.sh` | bring up core, drive load, assert delivery |

Pin every image tag at implementation; `:latest` here is a starting point.
