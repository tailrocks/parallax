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
- ⚠️ **SigNoz** — overlay `compose.signoz.yml` (vendored clone via
  `setup-vendor.sh`, pinned `v0.137.0`). **v0.137.0 removed
  `deploy/docker/docker-compose.yaml` (Foundry-only); overlay cannot start.**
  **Verified end-to-end live 2026-06-23 on v0.129.0:**
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
- ✅ **Maple** — overlay `compose.maple.yml` (`maple/Dockerfile`). **Verified
  end-to-end live 2026-06-23:** Rotel → `maple:4318` → embedded chDB, `maple
  traces` returns 6 `maple-fanout` spans. Two findings, both handled in the
  Dockerfile/entrypoint:
  1. Maple **does** ship prebuilt Linux bundles (`maple.dev/cli/install` → GitHub
     Releases: `maple` + `libchdb.so`), so we install that **instead of building
     from source** (the old scaffold's assumption was wrong).
  2. `maple start` binds OTLP + query API + dashboard to **127.0.0.1 only** (no
     `--host` flag), so a `socat` forwarder fronts it on `0.0.0.0:4318` to make
     `maple:4318` reachable from Rotel on the lab network. Dashboard/query API is
     published on host `:8081`. (Rotel logs a cosmetic protobuf-response-decode
     warning — Maple's OTLP/HTTP *response* body isn't protobuf — but ingestion
     succeeds and spans land in chDB.)
- ✅ **Sentry** — runnable, **verified end-to-end live 2026-06-23 on v26.6.0**; vendor pin is `26.7.2` (2026-08-14).
  Self-hosted Sentry is ~72 services bootstrapped by its own `install.sh` (not a
  clean `include:` target), so it runs as its **own vendored Compose stack**
  under `vendor/sentry` and Rotel reaches it over the **host bridge**
  (`host.docker.internal:9000` → nginx → relay) — no network-join needed. Three
  scripts drive it:
  1. `sentry/setup.sh` — vendor `getsentry/self-hosted` (pinned `SENTRY_REF`,
     default `26.7.2` ≥ native-OTLP `25.8.0`), run `install.sh` non-interactively
     (needs bash ≥ 4.4 — `brew install bash` on macOS), `docker compose up`.
  2. `sentry/onboard.sh` — create the admin (idempotent), read the internal
     project DSN, and print the exact `rotel.env` exports + `SENTRY_DSN`.
  3. `sentry/verify.sh <DSN>` — assert **A1** (native OTLP trace ingest → 200),
     **A15** (N identical errors group into one issue), **A16** (issue
     `times_seen` rises). Verified: OTLP ingest 200 + grouped issue.

  Paste the printed exports into `rotel.env`, add `sentry` to `ROTEL_EXPORTERS`
  + the traces/logs lists (omit from `ROTEL_EXPORTERS_METRICS` — Sentry has no
  OTLP metrics), and restart Rotel.

## Pinned versions (2026-08-14)

| Component | Pin |
| --- | --- |
| Rotel | `streamfold/rotel:v0.2.5` |
| OpenObserve | `public.ecr.aws/zinclabs/openobserve:v0.92.0` |
| telemetrygen | `ghcr.io/open-telemetry/opentelemetry-collector-contrib/telemetrygen:v0.158.0` |
| Maple | build arg `MAPLE_VERSION=v0.0.18` |
| SigNoz vendor | `SIGNOZ_REF=v0.137.0` |
| Sentry vendor | `SENTRY_REF=26.7.2` |

Playground infra pins (sibling repo `deploy/docker-compose.yml`): `postgres:18`, `redpandadata/redpanda:v26.2.1`, `ghcr.io/open-feature/flagd:v0.16.1`, `grafana/k6:2.2.0`. Bump via the plan-162 procedure. Existing playground `postgres` volumes must be dropped (`docker compose down -v`) when moving 17→18.

## Quick start (core)

```bash
cd bench/otlp-fanout
cp rotel.env.example rotel.env   # local lab credentials — never commit rotel.env
# Fill Authorization (OpenObserve) and optional Sentry headers; compose defaults:
#   root@example.com / Complexpass#123 → base64 in ROTEL_EXPORTER_OPENOBSERVE_CUSTOM_HEADERS
docker compose -f compose.yml up -d rotel openobserve   # OpenObserve UI: http://localhost:5080
./smoke.sh                                               # drive + assert fan-out
docker compose -f compose.yml down -v                    # teardown
```

OpenObserve default login: `root@example.com` / `Complexpass#123` (change in
`compose.yml` + the base64 `Authorization` in your local `rotel.env`).

## Parallax (host) — the one host sink

```bash
# ~/.parallax/config.toml:  bind = "0.0.0.0"  otlp_grpc_port = 14317  otlp_http_port = 14318
brew install tailrocks/parallax/parallax-preview   # or run from a local checkout
parallax serve --config ~/.parallax/config.toml    # UI http://localhost:4000
```

Rotel reaches it at `host.docker.internal:14317`. **Bind `0.0.0.0`** — a
loopback-only bind is unreachable from the container (the lab's one fragile hop).

## Compare mode — `parallax invocation start`

```bash
source bench/otlp-fanout/lab.env          # sets PARALLAX_OTLP_FORWARD=http://localhost:4317
parallax invocation start -- <your-otel-app>     # child telemetry → Rotel → every backend incl. Parallax
parallax invocation start --otlp-forward off -- <app>   # one-off: straight to Parallax
```

Implemented in `crates/parallax-cli` (env + flag; config-file deferred).

## Adding backends

```bash
./setup-vendor.sh                                   # clone SigNoz into vendor/
docker compose -f compose.yml -f compose.signoz.yml -f compose.maple.yml up -d
```

Then uncomment `maple`/`signoz` in `rotel.env` (`ROTEL_EXPORTERS` + the per-signal
lists). SigNoz UI → `http://localhost:3301`, Maple UI → `http://localhost:8081`.

Sentry is its own stack (not an overlay):

```bash
./sentry/setup.sh     # vendor + install.sh + up (20-40 min first run; needs bash >= 4.4)
./sentry/onboard.sh   # create admin, print the DSN + rotel.env exports
./sentry/verify.sh <DSN>   # assert OTLP ingest + issue grouping (A1/A15/A16)
```

Paste the printed `ROTEL_EXPORTER_SENTRY_*` exports into `rotel.env`, add
`sentry` to `ROTEL_EXPORTERS` + the traces/logs lists, restart Rotel. Sentry UI →
`http://localhost:9000`.

## Files

| File | Purpose |
|---|---|
| `compose.yml` | core: Rotel + OpenObserve + telemetrygen (loadgen profile) |
| `rotel.env` | Rotel fan-out config (exporters, per-signal lists, auth headers) |
| `lab.env` | `source` it to put the shell in compare mode |
| `compose.signoz.yml` / `compose.maple.yml` | backend overlays |
| `maple/Dockerfile` | Maple chDB local-mode build (best-effort) |
| `setup-vendor.sh` | clone SigNoz into `vendor/` |
| `sentry/setup.sh` | vendor + install self-hosted Sentry as its own Compose stack |
| `sentry/onboard.sh` | create admin, print DSN + `rotel.env` exports |
| `sentry/verify.sh` | assert Sentry OTLP ingest + issue grouping (A1/A15/A16) |
| `smoke.sh` | bring up core, drive load, assert delivery |

Pin every image tag at implementation; `:latest` here is a starting point.
