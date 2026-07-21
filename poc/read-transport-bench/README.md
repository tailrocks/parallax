# read-transport-bench (PoC — plan 090)

Standalone measurement harness for Parallax read-transport choices against
GreptimeDB. **Not product code.** Supports no product claims.

## What it measures

| Transport | Endpoint | Decode path |
|-----------|----------|-------------|
| HTTP `greptimedb_v1` (current) | `POST /v1/sql?format=greptimedb_v1` | `serde_json` → row count |
| HTTP `arrow` | `POST /v1/sql?format=arrow` | `arrow-ipc` stream → row count |
| HTTP `arrow+zstd` | `POST /v1/sql?format=arrow&compression=zstd` | same + zstd |
| MySQL prepared | `127.0.0.1:24002` plaintext | `mysql_async` (no TLS features) |

## TLS rule

- `reqwest` uses `native-tls` only (`default-features = false`).
- `mysql_async` uses **no** TLS features (plaintext localhost). Verify with
  `cargo tree -i rustls` inside this crate — it must print nothing.

## Run

```bash
export GREPTIME_HTTP=http://127.0.0.1:24000
export GREPTIME_MYSQL=mysql://127.0.0.1:24002/public

cargo run --release -- seed --n 100000
cargo run --release -- bench --reps 50
cargo run --release -- partition-bench --reps 50
cargo run --release -- range-check
```

## Tests

```bash
cargo nextest run          # offline unit + optional live parity when GREPTIME_HTTP set
```

Outputs are JSON suitable for pasting into
`docs/research/storage/read-transport-and-engine-defaults.md`.
