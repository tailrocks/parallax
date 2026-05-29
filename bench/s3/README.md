# Object-storage benchmark stack

Reproducible MinIO + GreptimeDB(S3) + ClickHouse(S3) stack for the object-storage
path of the GreptimeDB-vs-ClickHouse comparison (cost axis: retained object bytes,
object count, cold-read GET/PUT/LIST). Proven in
`../../docs/research/storage/greptimedb-vs-clickhouse/local-benchmark-results.md` Runs 8–9
(GreptimeDB 4 objects / 37 MiB vs ClickHouse 74 objects / 63 MiB for 1M spans).

```bash
./run-s3-stack.sh up      # MinIO + bucket + GreptimeDB(S3) + ClickHouse(S3)
# ... load data + query via docker exec (see the script's printed hints) ...
./run-s3-stack.sh down    # tear down (ephemeral; nothing persisted)
```

Files:

- `run-s3-stack.sh` — orchestrator (docker create+cp+start; bind mounts don't reach
  the orbstack daemon, so configs are copied in).
- `greptimedb-standalone-s3.toml` — GreptimeDB `[storage] type="S3"` against MinIO.
- `clickhouse-s3-disk.xml` — ClickHouse `s3` disk + `storage_policy='s3only'`.

Credentials are throwaway local MinIO (`minioadmin`) — **not secrets**, local
benchmark only. Pinned image tags match the version pins in the comparison README;
bump them together. This is the home of the remaining object-store cases:
**B10** (request-count instrumentation via `mc admin trace` / MinIO audit) and
**B12** (JSONBench-style cold object-store reads) in
`../../docs/research/storage/greptimedb-vs-clickhouse/benchmarking-the-differences.md`.
