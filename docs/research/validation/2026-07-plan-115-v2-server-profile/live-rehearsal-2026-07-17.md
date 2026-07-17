# Plan 115 — live single-node rehearsal (2026-07-17)

Contract: [`docs/research/decisions/v2-server-profile.md`](../../decisions/v2-server-profile.md)  
Host: operator macOS arm64, Docker available, GreptimeDB managed `v1.1.2`.

## Profile under test

Live process (PID at capture) running:

```text
./target/release/parallax serve --config /tmp/parallax-fanout-lab/config.toml
```

Config (redacted token value only for narrative; live token is local lab-only):

```toml
[server]
api_port = 4000
bind = "0.0.0.0"
otlp_grpc_port = 14317
otlp_http_port = 14318
api_token = "plan154-lab-token"

[storage]
mode = "managed"
data_dir = "/tmp/parallax-fanout-lab/data"
```

Ready banner (from serve log):

```text
Parallax ready — Ctrl-C to stop
  OTLP/gRPC  0.0.0.0:14317
  OTLP/HTTP  0.0.0.0:14318
  auth       bearer-token
```

Storage stack: GreptimeDB + Turso only; native TLS policy unchanged (plaintext
trusted local hop to managed Greptime; remote API behind bearer).

## Non-loopback + auth

| Check | Result |
| --- | --- |
| `GET /health` | `200 ok` (open) |
| GraphQL without bearer | `401 unauthorized` |
| GraphQL `Authorization: Bearer …` | `200` `{ health, version, otlpGrpcPort, otlpHttpPort }` |
| OTLP HTTP open on lab ports | accepts protobuf (empty body → `400` decode error, not auth wall) |

## Remote CLI dogfood

Isolated `HOME=/tmp/parallax-plan115-home`:

```text
parallax context add plan115 --url http://127.0.0.1:4000 --token plan154-lab-token
parallax context use plan115
parallax --context plan115 issue list   # empty set OK on lab
parallax --context plan115 traces       # empty/matching set after load
```

Context show masks the token. API token ≥16 bytes satisfies non-loopback bind
validation (plan 109 contract).

## Offline file-level backup snapshot

Coherent copy taken while process alive (operator stop→copy→start remains the
supported restore path):

```text
SNAP=/tmp/parallax-plan115-backup-20260717T230025
  config.toml
  data/   (~468 MiB including managed Greptime dirs + meta.db)
```

Restore procedure (not executed against the live lab): stop serve → replace
`data_dir` + config → start → `parallax doctor` until ready banner names all
surfaces.

## Retention / prune dry-run

`parallax prune --json` against the default install data dir produced a
contract-version-1 plan (`plan_id` hash present) with Turso class estimates and
protection snapshot generations. No execute path claimed here (plan 106 pin
protection still gates destructive metadata prune).

## Workload micro-packets (not a plan 110 trigger)

### Authenticated GraphQL

```text
n=200 concurrent workers=20
graphql_auth_rps≈2125  p50≈3.9ms  p95≈50ms  wall≈0.09s
```

API path is not saturated; this does **not** prove ingest-worker bottleneck.

### CLI `invocation start` (lab Rotel fan-out path)

Thirty sequential `parallax invocation start -- /bin/echo plan115-load-N`
against the lab's Rotel compare endpoint (`http://127.0.0.1:14318`):

```text
invocation_start_n=30 wall_s=1.21 rps=24.89 p50_ms=27.2 p95_ms=86.9
```

This measures CLI+export wall time under the multi-backend lab topology, not a
pure single-worker Parallax ingest isolation. No worker-stage saturation
packet is claimed.

**Plan 110 gate remains closed:** no evidence that the single ingest worker is
the bottleneck vs disk/network/Greptime. Single-worker stays mandatory.

## Still residual for full plan 115 retirement

- Four-target release artifact install dogfood (plan 102 pipeline archives)
- Operator TLS edge (reverse-proxy cert terminate) with remote WAN bind
- Measured upgrade/rollback binary swap log on the same data dir
- Disk-pressure injection + prune reclaim log under free-space stress
- Load packet that isolates worker stage costs for plan 110

## Commands to reproduce

```sh
# serve (ports must not collide with another managed Greptime on 24000–24003)
./target/release/parallax serve --config /path/to/config.toml

curl -sS http://127.0.0.1:4000/health
curl -sS -o /dev/null -w '%{http_code}\n' -X POST http://127.0.0.1:4000/graphql \
  -H 'content-type: application/json' -d '{"query":"{ health }"}'   # 401
curl -sS -X POST http://127.0.0.1:4000/graphql \
  -H 'content-type: application/json' \
  -H 'authorization: Bearer <token>' \
  -d '{"query":"{ health version otlpGrpcPort otlpHttpPort }"}'

parallax context add plan115 --url http://127.0.0.1:4000 --token <token>
parallax --context plan115 issue list
parallax prune --json
```
