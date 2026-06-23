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

- ✅ **Core (Rotel + OpenObserve)** — implemented and **verified end-to-end**
  (re-verified live 2026-06-23 on the upgraded Rust stack, otel 0.32/tonic 0.14):
  the playground's four Rust services emit OTLP → Rotel fans out → OpenObserve,
  and a search returns the multi-service trace by service: `checkout=30,
  pricing=6, inventory=6, recommendation=6` spans. The OpenObserve search path is
  `/api/{org}/_search` (stream in the SQL `FROM`, with `from`/`size`) — `smoke.sh`
  was corrected to match. The Parallax exporter targets the host; it simply
  retries until Parallax is up (note: Rotel fan-out is **sequential**, so list a
  down host-Parallax sink *after* the others or it back-pressures them).
- ✅ **SigNoz** — overlay `compose.signoz.yml` (vendored clone via
  `setup-vendor.sh`, pinned `v0.129.0`). **Verified end-to-end live 2026-06-23:**
  Rotel → SigNoz `otel-collector` → ClickHouse, `signoz-smoke = 8` spans queried
  back from `signoz_traces.distributed_signoz_index_v3`.

  **One-time onboarding is required** (the key run finding): SigNoz's
  `otel-collector` is **OpAMP-managed** by the SigNoz server — its OTLP `:4317`
  receiver is *not* opened by the static `otel-collector-config.yaml`; it binds
  only after the server pushes a config, and the server pushes it only after the
  **first org/admin is created**. On a fresh `compose up` the collector loops
  `opamp/server_client.go` errors and `:4317` stays closed. Create the first user
  once (the SigNoz UI does this, or via API), then the collector starts its OTLP
  receiver and Rotel delivers:

  ```bash
  # after `docker compose -f compose.yml -f compose.signoz.yml up -d` and the
  # signoz server is healthy — register the first org+admin (OpenAccess route):
  docker exec signoz wget -qO- --header='Content-Type: application/json' \
    --post-data='{"name":"Admin","email":"admin@parallax.lab","password":"Complexpass#123","orgDisplayName":"Parallax Lab","orgName":"parallax"}' \
    http://localhost:8080/api/v1/register
  # then enable `signoz` in rotel.env (ROTEL_EXPORTERS + ROTEL_EXPORTERS_TRACES).
  ```

  (Overlay networking verified: Rotel resolves `otel-collector` on the shared
  `lab` network; host `4317/4318` unpublished. Note: the `signoz: ports: !reset`
  override did not publish the UI on host `3301` in this run — reach the API
  in-container as above, or fix the publish — tracked.)
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
