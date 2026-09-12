# OTLP Fan-Out Comparison Lab

Feed **one** OpenTelemetry stream to several observability backends at once and
compare how each renders identical data. Design + rationale:
[`docs/research/validation/otlp-fanout-comparison-lab.md`](../../docs/research/validation/otlp-fanout-comparison-lab.md).

**Topology:** host processes are **Parallax** (from a source build of this
repo) and, optionally, the **Maple** official binary; everything else runs in
Compose. **Rotel** is the single shared OTLP endpoint published on host
`4317/4318`; it fans every signal out to each backend AND back to host Parallax
via `host.docker.internal:14317`.

```
emitters ─► localhost:4317 (Rotel) ─┬─► openobserve:5081            (compose, v1.0.0)
                                     ├─► host.docker.internal:14327  (SigNoz Foundry stack, own compose project)
                                     ├─► host.docker.internal:14341  (Maple host binary; overlay maple:4318 = alt path)
                                     ├─► grafana-lgtm:4317           (overlay, Loki/Tempo/Prometheus)
                                     ├─► hyperdx:4317                (overlay, all-in-one; after OpAMP onboarding + ingest token)
                                     ├─► host.docker.internal:9000   (Sentry nginx, own stack)
                                     └─► host.docker.internal:14317 ─► Parallax (host)
# rustrak is Sentry-envelope only (host :18081), not a Rotel exporter.
# highlight hobby self-host: last live attempt BLOCKED (2026-08-16).
```

**Last full live run: 2026-09-12** — see
[`docs/research/validation/2026-09-12-parallax-main-competitor-verification.md`](../../docs/research/validation/2026-09-12-parallax-main-competitor-verification.md).

- **Parity:** one telemetrygen stream (304 traces / 608 spans / 72k logs / 121k
  metric points) was received by **every** OTLP sink; the same error story as
  Sentry envelopes landed as 4223 EAP items in Sentry.
- **SigNoz:** the deprecated v0.129.0 vendor-clone overlay was replaced by the
  checked-in Foundry-generated `signoz-foundry/` deployment (signoz v0.141.1 +
  collector v0.144.9, ClickHouse/Keeper 25.12.5, postgres 16 metastore; host
  ports 14327/14328/3301). `setup-vendor.sh` is deleted.
- **HyperDX:** OpAMP onboarding gate + team ingest-token header documented; the
  overlay sets `FRONTEND_URL=http://127.0.0.1:18080` so login redirects stop
  bouncing to the playground catalog on :8080.
- **Version pins bumped:** OpenObserve v1.0.0, Grafana LGTM 0.33.0, HyperDX
  2.38.0 (new image repo), rustrak 0.14.12, Maple v0.0.22.

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
- ✅ **SigNoz** — separate **Foundry** stack in `signoz-foundry/` (its own
  compose project; it cannot be an overlay of this lab's `compose.yml`).
  SigNoz deprecated the repo-bundled compose in v0.130.0, so the old
  v0.129.0 vendor-clone overlay + `setup-vendor.sh` are gone. The checked-in
  deployment is `foundryctl` v0.2.17-generated with images pinned to
  `signoz/signoz:v0.141.1` + `signoz/signoz-otel-collector:v0.144.9`
  (ClickHouse/Keeper 25.12.5, postgres 16 metastore); host ports are remapped
  to **14327 → 4317** (OTLP gRPC), **14328 → 4318** (OTLP HTTP), **3301 → 8080**
  (UI/API) so Rotel keeps 4317/4318. **Verified live 2026-09-12:** 608 spans in
  `signoz_traces.signoz_index_v3` after the shared telemetrygen stream.

  ```bash
  docker compose -f signoz-foundry/compose.yaml -p signoz up -d
  # Rotel reaches the ingester at host.docker.internal:14327 (see rotel.env),
  # then enable `signoz` in rotel.env (ROTEL_EXPORTERS + per-signal lists).
  ```

  **First-run onboarding:** register the initial org/admin in the UI
  (`http://localhost:3301`) before query asserts. Unlike the old overlay, the
  ingester accepts OTLP without it — only the query APIs are auth-gated until
  an organization exists. Regenerate from upstream when bumping SigNoz
  (`foundryctl gen examples`, then re-pin images and re-map the three ports);
  see the header of `compose.signoz.yml`.
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

  **Live run 2026-09-12 used the official Maple host binary instead**
  (`maple-v0.0.22-aarch64-apple-darwin`, OTLP on host `:14341` — Rotel's
  `ROTEL_EXPORTER_MAPLE_ENDPOINT`): Maple publishes no Docker image, and the
  binary is the vendor-supported path. The containerized overlay above stays
  as the reproducible fallback. 304 traces / facet count 304 via UI + CLI.
- ✅ **Sentry** — runnable, **verified end-to-end live 2026-06-23 on v26.6.0**;
  the 2026-09-12 run used `SENTRY_REF=26.8.0` (`sentry/setup.sh` still defaults
  to `26.7.2`).
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

## 4-sink re-verify (2026-08-14, historical)

Predates the 2026-09-12 run: SigNoz was still blocked here, and the pins are
the pre-bump ones. Playground coverage program (plans 162–167) re-ran the live
Rotel `v0.2.5` fan-out after lockstep SDK upgrades:

| Sink | Result |
| --- | --- |
| OpenObserve v0.92.0 | checkout/catalog/payment/inventory/recommendation/pricing spans present |
| Maple v0.0.18 | `services --since 2h` lists the same six names |
| Parallax host (scratch OTLP 14317 via loopback+TCP bridge) | GraphQL + UI walk of every coverage-matrix surface |
| Sentry 26.7.2 | `verify.sh` A1 OTLP=200, A15/A16 grouping; no OTLP metrics |

Java agent gRPC→Rotel PASS (2.30.0). Per-concept dispositions (Maple/OO
win some cells) are in the playground `VERIFICATION.md`. Coverage spine:
playground `docs/coverage-matrix.md`.

## Pinned versions (2026-09-12)

| Component | Pin |
| --- | --- |
| Rotel | `streamfold/rotel:v0.2.5` |
| OpenObserve | `openobserve/openobserve:v1.0.0` (first GA; supersedes `v0.92.0`) |
| telemetrygen | `ghcr.io/open-telemetry/opentelemetry-collector-contrib/telemetrygen:v0.158.0` |
| Maple | build arg `MAPLE_VERSION=v0.0.22` (overlay); the 2026-09-12 run also ran the official host binary v0.0.22 on `:14341` |
| SigNoz | `signoz-foundry/` (foundryctl v0.2.17 gen): `signoz/signoz:v0.141.1` + `signoz/signoz-otel-collector:v0.144.9`, ClickHouse/Keeper `25.12.5`, postgres `16` metastore; host 14327/14328/3301 |
| Sentry vendor | `SENTRY_REF=26.8.0` (2026-09-12 run; `sentry/setup.sh` default `26.7.2`) |
| Grafana LGTM | `grafana/otel-lgtm:0.33.0` (UI host 3300) |
| HyperDX | `hyperdx/hyperdx-all-in-one:2.38.0` (UI host 18080; repo moved from `clickhouse/clickstack-all-in-one`) |
| rustrak | `rustrak/rustrak-server:v0.14.12` + `rustrak/rustrak-ui:v0.14.12` (18081/18082) |

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

## Parallax (host) — the host sink

Host processes: **Parallax, built from source** (the 2026-09-12 run used
`0.1.0+6b3a92b`; the Homebrew preview install is an older build and was
excluded there), plus the optional Maple official binary.

```bash
cargo build --release
# config.toml (all server keys live under [server]; misnested/unknown keys now
# fail startup instead of silently defaulting):
#   [server]  bind = "0.0.0.0"  otlp_grpc_port = 14317  otlp_http_port = 14318
target/release/parallax serve --config config.toml   # UI http://localhost:4000
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
docker compose -f compose.yml -f compose.maple.yml up -d        # Maple overlay (chDB build)
docker compose -f signoz-foundry/compose.yaml -p signoz up -d   # SigNoz: separate project, not an overlay
```

Then enable `maple`/`signoz` in `rotel.env` (`ROTEL_EXPORTERS` + the per-signal
lists). SigNoz UI → `http://localhost:3301` (register the first org/admin before
query asserts), Maple UI → `http://localhost:8081`.

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
| `compose.signoz.yml` | notes only: SigNoz runs as the separate `signoz-foundry/` project — an overlay cannot express it (and the v0.129.0 vendor clone is gone) |
| `compose.maple.yml` | Maple overlay (chDB build) |
| `compose.grafana.yml` / `compose.hyperdx.yml` / `compose.rustrak.yml` | Grafana LGTM / HyperDX / rustrak overlays |
| `signoz-foundry/` | Foundry-generated SigNoz deployment (signoz v0.141.1 + collector v0.144.9; boot standalone with `-p signoz`) |
| `exporters-reachable.sh` | parse `rotel.env` + TCP-probe listed exporters (`--parse-only` for CI) |
| `maple/Dockerfile` | Maple chDB local-mode build (best-effort) |
| `setup-highlight.sh` | last highlight.io hobby attempt (expected BLOCKED) |
| `sentry/setup.sh` | vendor + install self-hosted Sentry as its own Compose stack |
| `sentry/onboard.sh` | create admin, print DSN + `rotel.env` exports |
| `sentry/verify.sh` | assert Sentry OTLP ingest + issue grouping (A1/A15/A16) |
| `smoke.sh` | bring up core, drive load, assert delivery |

Pin every image tag at implementation; `:latest` here is a starting point.
