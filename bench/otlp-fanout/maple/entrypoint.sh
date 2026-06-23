#!/usr/bin/env bash
# Run Maple local-mode on an internal loopback port and expose it on the lab
# network via socat. Maple binds OTLP/UI/query to 127.0.0.1:<port> only (no bind
# flag), so the forwarder is what makes `maple:4318` reachable from Rotel.
set -euo pipefail

PORT="${MAPLE_INTERNAL_PORT:-9000}"
DATA="${MAPLE_DATA_DIR:-/data}"

echo "[maple-entrypoint] starting maple on 127.0.0.1:${PORT} (data=${DATA})"
maple start --offline --port "${PORT}" --data-dir "${DATA}" &
MAPLE_PID=$!

# Wait for Maple's loopback listener before exposing it.
for _ in $(seq 1 60); do
  if (exec 3<>"/dev/tcp/127.0.0.1/${PORT}") 2>/dev/null; then
    exec 3>&- 3<&-
    echo "[maple-entrypoint] maple up on 127.0.0.1:${PORT}; forwarding 0.0.0.0:4318 -> 127.0.0.1:${PORT}"
    break
  fi
  if ! kill -0 "${MAPLE_PID}" 2>/dev/null; then
    echo "[maple-entrypoint] maple exited during startup" >&2
    exit 1
  fi
  sleep 1
done

# Forward the lab-facing OTLP/HTTP port to Maple's single loopback listener
# (Rotel → maple:4318). Maple serves OTLP ingest + query API + dashboard all on
# the one internal port; the dashboard is reachable in-container at
# 127.0.0.1:${PORT} (forward another lab port to it if you need it on the host).
socat TCP-LISTEN:4318,fork,reuseaddr "TCP:127.0.0.1:${PORT}" &
SOCAT_PID=$!

# If either Maple or the forwarder dies, stop the container.
wait -n "${MAPLE_PID}" "${SOCAT_PID}"
exit $?
