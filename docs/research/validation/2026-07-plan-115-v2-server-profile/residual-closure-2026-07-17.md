# Plan 115 residual closure (2026-07-17)

Contract: [`docs/research/decisions/v2-server-profile.md`](../../decisions/v2-server-profile.md)  
Prior packet: [`live-rehearsal-2026-07-17.md`](live-rehearsal-2026-07-17.md)  
Host: operator macOS arm64. Isolated lab (ports avoid concurrent bench/fan-out):

| Surface | Bind |
| --- | --- |
| API/UI/GraphQL | `0.0.0.0:4500` |
| OTLP gRPC/HTTP | `0.0.0.0:15317` / `15318` |
| TLS edge | `https://127.0.0.1:8443` → `http://127.0.0.1:4500` |
| Token | `plan115-residual-token` (≥16 B; env `PARALLAX_API_TOKEN` unset so config wins) |
| Data | `/tmp/parallax-plan115-residual/data` (managed Greptime + Turso) |

Binary under test: `parallax 0.1.0-dev+983e11e` (workspace release) plus published
`0.1.0-preview.1295+e37a65d` install dogfood.

## Residual gates

### 1. Operator TLS edge (reverse-proxy terminate)

- Self-signed RSA cert (`CN=parallax.lab`, SAN `localhost`/`127.0.0.1`).
- Minimal Python `ThreadingHTTPServer` TLS terminator (lab stand-in for OS /
  reverse-proxy terminate). **Native TLS only** on the Parallax product path;
  product process remains plaintext on the trusted local hop behind the edge.
- Checks:

| Check | Result |
| --- | --- |
| `GET https://127.0.0.1:8443/health` | `ok` |
| GraphQL over TLS without bearer | `401` |
| GraphQL over TLS with bearer | `200` `{ health, version, otlpGrpcPort:15317, otlpHttpPort:15318 }` |
| CLI `context add` + `issue list` via `https://127.0.0.1:8443` | PASS (lab cert trusted in login keychain for Secure Transport) |

Logs: process output captured in session; edge script was lab-local under
`/tmp/parallax-plan115-residual/tls/` (not product code).

### 2. Upgrade / rollback binary swap (same `data_dir`)

Evidence: [`upgrade-rollback.log`](upgrade-rollback.log)

1. Start `parallax-prev` → ready banner names all surfaces; GraphQL 401/200.
2. SIGTERM (exit ~1–2 s).
3. Start `parallax-new` same config/data → ready ~2 s; GraphQL OK.
4. SIGTERM; start `parallax-prev` (rollback) same data → ready ~2 s; GraphQL OK.

Same Turso path and managed Greptime data dir across cutovers. SHA of both
binaries in this lab: `5d3c938a…` (same content rebuild; path exercises
stop→swap→start, not a version skew).

### 3. Disk-pressure ballast + reclaim

Evidence: [`disk-pressure.log`](disk-pressure.log)

| Step | Observation |
| --- | --- |
| Before | `data` ≈ 466 MiB; volume ~635 GiB free |
| Inject | 2 GiB `data/_pressure/ball.bin` → `data` ≈ 2.5 GiB |
| Under pressure | Serve health still `ok`; `prune --json` dry-run still emits contract v1 plan |
| Reclaim | Remove ballast → `data` ≈ 466 MiB; free space restored; GraphQL health OK |

Operator reclaim of non-engine ballast under free-space stress is proven.
Telemetry raw signals remain engine-TTL managed (plan 116); destructive prune
execute still pin-gated.

### 4. Four-target release artifact install dogfood

Evidence: [`install-dogfood.log`](install-dogfood.log) + plan 102 baseline
[`2026-07-13-plan-102-release-baseline.md`](../2026-07-13-plan-102-release-baseline.md).

| Target | HEAD | Host install |
| --- | ---: | --- |
| `aarch64-apple-darwin` | 200 (56,499,635 B) | Extracted + `--version` → `0.1.0-preview.1295+e37a65d`; sha256 `7889417208b4333124ccec60e2d52879922ca2f8f7a073cfacf48134af8fb909` |
| `x86_64-apple-darwin` | 200 (61,091,729 B) | Artifact present (not executed on arm64 host) |
| `aarch64-unknown-linux-gnu` | 200 (68,879,699 B) | Artifact present (cross-target; plan 102 release-verify) |
| `x86_64-unknown-linux-gnu` | 200 (65,973,414 B) | Artifact present (cross-target; plan 102 release-verify) |

Tap formula `parallax-preview` still pins four-target URLs; live archive for
darwin-arm64 matches published preview tag content at dogfood time (sha differs
from older formula pin `aa82a831…` which tracked preview.958 — version table
floor, not freeze).

### 5. Plan 110 load packet (gate remains CLOSED)

Evidence: [`load-packet.log`](load-packet.log),
[`telemetrygen-summary.log`](telemetrygen-summary.log)

| Packet | Result |
| --- | --- |
| GraphQL auth micro | n=200 workers=20 ok=200 **rps≈2765** p50≈3.6 ms p95≈34 ms |
| OTLP gRPC load | `telemetrygen` 10 s × 4 workers → **14,586 traces** (~1.5k traces/s, ~2.9k spans/s) accepted; CLI `traces` lists `service=plan115-load` |
| Stage isolation | **Not available** — no per-stage worker/disk/Greptime counters in packet |
| vs envelope | Profile claims ≤5k spans/s; load under envelope without error |

**Plan 110 trigger stays CLOSED.** This is not evidence that the single ingest
worker is the bottleneck versus disk/network/Greptime. Product decision for the
only supported V2 profile: **retain single-worker** (see plan 110 retirement).

### 6. OTLP ingest tokens

ADR already defers optional project tokens until measured need; default remains
open OTLP on the trusted network behind the operator edge. No remote-public
OTLP exposure claimed; residual not opened.

## Done criteria map

| Criterion | Evidence |
| --- | --- |
| Support contract/ADR implemented | `docs/research/decisions/v2-server-profile.md` + this packet |
| GreptimeDB + Turso; native TLS | managed Greptime + Turso meta; TLS only at edge |
| Auth on remote API/query/management | bearer required on non-loopback GraphQL/CLI |
| Backup/restore/upgrade/rollback | prior backup snapshot + this upgrade/rollback log |
| Verified artifacts on supported targets | four-target HEAD + host install + plan 102 |
| Remote CLI + workload; plan 110 packet | TLS CLI dogfood + load packet (**110 not opened**) |

## STOP check

No alternate DB, no rustls product path, no unauthenticated non-loopback bind,
no multi-worker concurrency without measurement.
