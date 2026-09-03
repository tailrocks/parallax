# OTLP Fan-Out Comparison Lab

Feed **one** OpenTelemetry stream to several observability backends at once and
compare how each renders identical data. Design + rationale:
[`docs/research/validation/otlp-fanout-comparison-lab.md`](../../docs/research/validation/otlp-fanout-comparison-lab.md).

**Topology:** only **Parallax** runs on the host (Homebrew). Everything else runs
in Compose or its required own Compose stack. **Rotel** is the single shared OTLP
endpoint published on host `4317/4318`; it fans each applicable signal out to
ready backends and back to host Parallax via `host.docker.internal:14317`.

```
emitters ─► localhost:4317 (Rotel) ─┬─► openobserve:5081        (compose)
                                     ├─► host.docker.internal:4321 (SigNoz v0.140.0 Foundry ingester)
                                     ├─► maple:4318              (overlay, chDB)
                                     ├─► grafana-lgtm:4317       (overlay, Loki/Tempo/Prometheus)
                                     ├─► hyperdx:4317            (probe only; .37 listener blocked in this run)
                                     ├─► host.docker.internal:9000 (Sentry nginx, own stack)
                                     └─► host.docker.internal:14317 ─► Parallax (host)
# rustrak is Sentry-envelope only (host :18081), not a Rotel exporter.
# highlight hobby self-host: last live attempt BLOCKED (2026-08-16).
```

## Status

- ✅ **Core (Rotel + OpenObserve)** — implemented and **verified end-to-end**
  (re-verified live 2026-09-04 on the current playground and OpenObserve
  `v0.92.2`; historical 2026-06-23 counts remain below):
  the playground's four Rust services emit OTLP → Rotel fans out → OpenObserve,
  and a search returns the multi-service trace by service: `checkout=30,
  pricing=6, inventory=6, recommendation=6` spans. The OpenObserve search path is
  `/api/{org}/_search` (stream in the SQL `FROM`, with `from`/`size`) — `smoke.sh`
  was corrected to match. The Parallax exporter targets the host; it simply
  retries until Parallax is up (note: Rotel fan-out is **sequential**, so list a
  down host-Parallax sink *after* the others or it back-pressures them).
- ✅ **SigNoz** — current supported deployment uses **Foundry v0.2.17** to
  generate the v0.140.0 Compose stack. `setup-vendor.sh` verifies the Foundry
  checksum and writes `vendor/signoz-pours`; `compose.signoz.yml` includes that
  generated output. First UI registration is required because the ingester is
  OpAMP-managed. Current live run: UI/API `http://localhost:3301` returned 200;
  traces, logs, and metrics were visible after onboarding. The overlay remaps
  OTLP/HTTP to host `:4321` so Rotel keeps `:4317/4318`.
- ✅ **Maple** — overlay `compose.maple.yml` (`maple/Dockerfile`). **Verified
  end-to-end live 2026-09-04 on official `MapleTechLabs/maple v0.0.21`: Rotel →
  `maple:4318` → embedded chDB, `maple
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
- ✅ **Sentry** — runnable, **verified end-to-end live 2026-09-04 on v26.8.0**.
  Self-hosted Sentry is ~72 services bootstrapped by its own `install.sh` (not a
  clean `include:` target), so it runs as its **own vendored Compose stack**
  under `vendor/sentry` and Rotel reaches it over the **host bridge**
  (`host.docker.internal:9000` → nginx → relay) — no network-join needed. Three
  scripts drive it:
  1. `sentry/setup.sh` — vendor `getsentry/self-hosted` (pinned `SENTRY_REF`,
     default `26.8.0` ≥ native-OTLP `25.8.0`), run `install.sh` non-interactively
     (needs bash ≥ 4.4 — `brew install bash` on macOS), `docker compose up`.
  2. `sentry/onboard.sh` — create the admin (idempotent), read the internal
     project DSN, and print the exact `rotel.env` exports + `SENTRY_DSN`.
  3. `sentry/verify.sh <DSN>` — assert **A1** (native OTLP trace ingest → 200),
     **A15** (N identical errors group into one issue), **A16** (issue
     `times_seen` rises). Verified: OTLP ingest 200 + grouped issue.

  Paste the printed exports into `rotel.env`, add `sentry` to `ROTEL_EXPORTERS`
  + the traces/logs lists (omit from `ROTEL_EXPORTERS_METRICS` — Sentry has no
  OTLP metrics), and restart Rotel.

## Current verification (2026-09-04)

Latest supported releases were used for the live run. Stable-only policy excludes
OpenObserve `v1.0.0-rc2`; latest GA is `v0.92.2`. OTel Contrib is `v0.160.0`,
but its telemetrygen image is not published; the newest runnable image is
`v0.159.0`.

| Component | Current release / image |
| --- | --- |
| Rotel | `streamfold/rotel:v0.2.5` |
| OpenObserve | `public.ecr.aws/zinclabs/openobserve:v0.92.2` |
| telemetrygen | `ghcr.io/open-telemetry/opentelemetry-collector-contrib/telemetrygen:v0.159.0` |
| Maple | `v0.0.21` |
| SigNoz | `v0.140.0`; Foundry `v0.2.17`; collector `v0.144.9` |
| Sentry | `26.8.0` |
| Grafana LGTM | `grafana/otel-lgtm:0.32.0` |
| HyperDX / ClickStack | `clickhouse/clickstack-all-in-one:2.37.0` |
| Rustrak | `v0.14.11` |

Evidence and feature matrix: [canonical 2026-09-04 report](../../docs/research/validation/2026-09-04-parallax-main-competitor-verification.md).

## Historical 4-sink re-verify (2026-08-14)

Playground coverage program (plans 162–167 minus SigNoz Foundry rewrite)
re-ran the live Rotel `v0.2.5` fan-out after lockstep SDK upgrades:

| Sink | Result |
| --- | --- |
| OpenObserve v0.92.0 | checkout/catalog/payment/inventory/recommendation/pricing spans present |
| Maple v0.0.18 | `services --since 2h` lists the same six names |
| Parallax host (scratch OTLP 14317 via loopback+TCP bridge) | GraphQL + UI walk of every coverage-matrix surface |
| Sentry 26.7.2 | `verify.sh` A1 OTLP=200, A15/A16 grouping; no OTLP metrics |

Java agent gRPC→Rotel PASS (2.30.0). Per-concept dispositions (Maple/OO
win some cells) are in the playground `VERIFICATION.md`. SigNoz overlay
stays **blocked** — do not invent Foundry. Coverage spine:
playground `docs/coverage-matrix.md`.

## Historical pinned versions (2026-08-14)

| Component | Pin |
| --- | --- |
| Rotel | `streamfold/rotel:v0.2.5` |
| OpenObserve | `public.ecr.aws/zinclabs/openobserve:v0.92.0` |
| telemetrygen | `ghcr.io/open-telemetry/opentelemetry-collector-contrib/telemetrygen:v0.158.0` |
| Maple | build arg `MAPLE_VERSION=v0.0.18` |
| SigNoz vendor | `SIGNOZ_REF=v0.129.0` (last bootable community compose; v0.137.0+ Foundry-only) |
| Sentry vendor | `SENTRY_REF=26.7.2` |
| Grafana LGTM | `grafana/otel-lgtm:0.30.2` (UI host 3300) |
| HyperDX ClickStack | `clickhouse/clickstack-all-in-one:2.35.0` (UI host 18080) |
| rustrak | `rustrak/rustrak-server:v0.14.4` + `rustrak/rustrak-ui:v0.14.4` (18081/18082) |

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
./setup-vendor.sh                                   # Foundry -> vendor/signoz-pours/
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
| `compose.signoz.yml` / `compose.maple.yml` | SigNoz / Maple overlays |
| `compose.grafana.yml` / `compose.hyperdx.yml` / `compose.rustrak.yml` | Grafana LGTM / HyperDX / rustrak overlays |
| `exporters-reachable.sh` | parse `rotel.env` + TCP-probe listed exporters (`--parse-only` for CI) |
| `maple/Dockerfile` | Maple chDB local-mode build (best-effort) |
| `setup-vendor.sh` | generate current SigNoz Compose with Foundry (default `v0.2.17`) |
| `setup-highlight.sh` | last highlight.io hobby attempt (expected BLOCKED) |
| `sentry/setup.sh` | vendor + install self-hosted Sentry as its own Compose stack |
| `sentry/onboard.sh` | create admin, print DSN + `rotel.env` exports |
| `sentry/verify.sh` | assert Sentry OTLP ingest + issue grouping (A1/A15/A16) |
| `smoke.sh` | bring up core, drive load, assert delivery |

Pin every image tag at implementation. The current run uses the exact versions
listed above; `:latest` is not accepted as comparison evidence.
