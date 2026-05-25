#!/usr/bin/env bash
# Stand up the object-store benchmark stack: MinIO + GreptimeDB(S3) + ClickHouse(S3),
# all on an isolated docker network. Proven in local-benchmark-results.md Runs 8-9.
#
# Why docker create + cp (not `-v` bind mounts or a compose file): on the orbstack
# daemon used here, host bind mounts do not reach the container ("config not found"),
# so configs are injected with `docker create` -> `docker cp` -> `docker start`.
# Queries run via `docker exec` (no host port mapping -> no clash with the main
# bench/compose.yml stack on 4000/8123).
#
# Usage:  ./run-s3-stack.sh up      # bring up + create bucket
#         ./run-s3-stack.sh down    # tear everything down
#
# Pinned versions (keep in sync with bench README / version pins):
GT_IMG="greptime/greptimedb:v1.0.2"
CH_IMG="clickhouse/clickhouse-server:26.5.1.882"
MINIO_IMG="minio/minio:latest"
MC_IMG="minio/mc:latest"
NET="pbench-s3"
HERE="$(cd "$(dirname "$0")" && pwd)"
set -euo pipefail

wait_healthy() { # $1 = predicate command; bounded ~40s
  for _ in $(seq 1 20); do if eval "$1" >/dev/null 2>&1; then return 0; fi; sleep 2; done
  echo "timeout waiting: $1" >&2; return 1
}

up() {
  docker network create "$NET" 2>/dev/null || true
  docker rm -f pbench-minio pbench-gt-s3 pbench-ch-s3 2>/dev/null || true

  docker run -d --name pbench-minio --network "$NET" \
    -e MINIO_ROOT_USER=minioadmin -e MINIO_ROOT_PASSWORD=minioadmin \
    "$MINIO_IMG" server /data >/dev/null
  sleep 3
  docker run --rm --network "$NET" --entrypoint sh "$MC_IMG" -c \
    "mc alias set m http://pbench-minio:9000 minioadmin minioadmin && mc mb -p m/greptimedb"

  # GreptimeDB on S3 (config injected via create+cp+start)
  docker create --name pbench-gt-s3 --network "$NET" "$GT_IMG" \
    standalone start -c /s3.toml --http-addr 0.0.0.0:4000 --rpc-bind-addr 0.0.0.0:4001 \
    --mysql-addr 0.0.0.0:4002 >/dev/null
  docker cp "$HERE/greptimedb-standalone-s3.toml" pbench-gt-s3:/s3.toml
  docker start pbench-gt-s3 >/dev/null
  wait_healthy "docker exec pbench-gt-s3 curl -sf http://localhost:4000/health"
  echo "GreptimeDB(S3) ready  ->  docker exec pbench-gt-s3 curl -s -X POST --data-urlencode 'sql=...' http://localhost:4000/v1/sql?db=public"

  # ClickHouse on S3 (config.d injected via create+cp+start)
  docker create --name pbench-ch-s3 --network "$NET" --ulimit nofile=262144:262144 "$CH_IMG" >/dev/null
  docker cp "$HERE/clickhouse-s3-disk.xml" pbench-ch-s3:/etc/clickhouse-server/config.d/s3.xml
  docker start pbench-ch-s3 >/dev/null
  wait_healthy "docker exec pbench-ch-s3 clickhouse-client --query 'SELECT 1'"
  echo "ClickHouse(S3) ready  ->  docker exec pbench-ch-s3 clickhouse-client --query '... SETTINGS storage_policy=\"s3only\"'"
  echo "MinIO objects         ->  docker run --rm --network $NET --entrypoint sh $MC_IMG -c 'mc alias set m http://pbench-minio:9000 minioadmin minioadmin && mc du m/greptimedb'"
}

down() {
  docker rm -f pbench-ch-s3 pbench-gt-s3 pbench-minio 2>/dev/null || true
  docker network rm "$NET" 2>/dev/null || true
  echo "torn down"
}

case "${1:-up}" in
  up) up ;;
  down) down ;;
  *) echo "usage: $0 {up|down}" >&2; exit 1 ;;
esac
