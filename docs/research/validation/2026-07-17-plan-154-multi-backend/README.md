# Plan 154 — live multi-backend fan-out acceptance (2026-07-17)

Research date: 2026-07-17T15:00Z UTC.
Host: macOS aarch64, 64 GiB RAM, Docker (OrbStack), one self-hosted external
backend at a time (operator residual rule; plan text assumed 16 GiB).
Parallax host sink: `parallax serve` with
`/tmp/parallax-fanout-lab/config.toml` —
`bind = 0.0.0.0`, `otlp_grpc_port = 14317`, `otlp_http_port = 14318`,
`api_token = plan154-lab-token`, data under `/tmp/parallax-fanout-lab/data`.
Lab root: `bench/otlp-fanout/`. Rotel on host `:4317/:4318`.
Parallax preview binary at start of session: `0.1.0-preview.958+4e8edfa`.
Repo head during run: see `git rev-parse HEAD` at commit time.

Parallax-backend product arm already DONE via plan 159
(`docs/research/validation/2026-07-unified-cli-observability/`). This packet is
the **external multi-backend residual** only.

## Method

For each backend:

1. Stop other externals (one-at-a-time).
2. Point `rotel.env` at **Parallax + that one backend** only.
3. Drive `telemetrygen` through Rotel (`service=<backend-tag>`).
4. Assert the backend copy and the Parallax SQL copy.

## Results

| Backend | Topology | Assert | Evidence |
|---|---|---|---|
| **OpenObserve** | `compose.yml` rotel+openobserve | **PASS** — OO search `count(*)=102` for traces; Parallax `service_name=smoke` → **102** rows | `./smoke.sh` green; OO UI `:5080`; healthz ok |
| **Maple** v0.0.12 | `compose.maple.yml`, OO stopped | **PASS** — `docker exec pfanout-maple maple traces` shows `serviceName=maple-fanout` spans; Parallax SQL **102** rows for `maple-fanout` | image build from `maple/Dockerfile`; host UI `:8081` |
| **SigNoz** v0.129.0 | `compose.signoz.yml` + vendor clone; first-org register | **PASS** — ClickHouse `distributed_signoz_index_v3`: `signoz-smoke=102`, `signoz-smoke2=82`; Parallax SQL `signoz-smoke=102` | UI `:3301`; admin `admin@parallax.lab` via `/api/v1/register`; OpAMP opens collector OTLP `:4317` only after register |
| **Sentry** self-hosted v26.6.0 | `sentry/setup.sh` + own compose stack | **PASS** — `verify.sh` A1 OTLP 200 + A15/A16 `times_seen=5` on `PaymentError: payment failure (chaos)`; DSN `http://14685f…@localhost:9000/1` | setup ~20 min; onboard admin; Rotel HTTP OTLP to `host.docker.internal:9000/api/1/integration/otlp` |

### OpenObserve detail

- Auth: `root@example.com` / `Complexpass#123` (compose default; Basic header in local `rotel.env`, gitignored).
- Search path: `POST /api/default/_search?type=traces` with stream in SQL `FROM`, `from`/`size` set (matches `smoke.sh`).
- Response snippet: `"hits":[{"c":102}]`.

### Maple detail

- Prebuilt Linux aarch64 bundle `v0.0.12` + socat front on `0.0.0.0:4318` (Maple binds loopback-only).
- Rotel protocol: HTTP to `maple:4318`.
- Query: in-container `maple traces` JSON includes `serviceName: "maple-fanout"`.

### SigNoz detail

- Vendor: `setup-vendor.sh` → `vendor/signoz` @ `v0.129.0` (gitignored).
- **Onboarding gate re-confirmed**: collector OTLP receiver stays closed until first org/admin; after register, logs show `Starting GRPC server … endpoint=[::]:4317`.
- Transient Rotel `unable to connect` during OpAMP race; retry after collector ready succeeds (second service `signoz-smoke2`).

### Sentry detail

- Script path: `bench/otlp-fanout/sentry/{setup,onboard,verify}.sh`, pin `SENTRY_REF=26.6.0`.
- Setup completed this session (~20 min install + compose `--wait`); onboard
  created `admin@parallax.lab`; public key `14685f5828032726db98ad5933e1bcbe`.
- `./sentry/verify.sh <DSN>` → **ASSERT PASS**: OTLP ingest HTTP 200; five
  identical errors grouped into one issue with `times_seen=5`.
- Rotel env (local, gitignored): `sentry` on traces+logs only; metrics stay
  Parallax-only for this arm.

## Residual closure (2026-07-17, second packet)

1. **Rust collector-backed acceptance + test-verify** — PASS on host Parallax
   lab (`:4610` / OTLP gRPC `:14317`, bearer `plan154-lab-token`):
   invocation `1969ff68-0ebc-4bc0-afd5-5c7226b2662e` →
   `test-verify … rust` reported **3 traces, 95 test attempts, 2 app
   descendants**. Wrapper script path:
   `scripts/observable-test-session.sh rust --acceptance` (ran via cargo on
   PATH when mise GitHub rate-limit blocked).
2. **W5 disposition rows** updated in playground `VERIFICATION.md`
   (histogram table + PaymentError table).
3. **Workflow** remains playground `.github/workflows/ci.yml` on `main`.
4. **Plan 122** already DONE earlier same day.

## STOP check

No mock backends, no screenshots-as-proof, no product fallback engine. All
PASS rows above are live Docker + SQL/CLI query evidence.

## Reproduce

```bash
# Host Parallax (offset OTLP + non-loopback bind + token)
parallax serve --config /tmp/parallax-fanout-lab/config.toml

cd bench/otlp-fanout
# OpenObserve-only: rotel.env exporters = parallax,openobserve
docker compose -f compose.yml up -d rotel openobserve && ./smoke.sh

# Maple-only
docker compose -f compose.yml -f compose.maple.yml up -d rotel maple
# (rotel.env: maple exporters) + telemetrygen --service=maple-fanout

# SigNoz-only
./setup-vendor.sh
docker compose -f compose.yml -f compose.signoz.yml up -d
# register first admin, then rotel.env signoz exporters + telemetrygen

# Sentry-only (own stack)
./sentry/setup.sh && ./sentry/onboard.sh && ./sentry/verify.sh <DSN>
```
