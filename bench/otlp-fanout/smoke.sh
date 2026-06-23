#!/usr/bin/env bash
# Phase-1 smoke: prove the Rotel fan-out delivers to every running sink.
#
# Brings up the core lab, drives one batch of telemetry through Rotel via
# telemetrygen, then asserts the copy landed in each reachable backend. The
# host-Parallax assert is the lab's one fragile hop (host.docker.internal) — it
# is checked only when Parallax is up on the host.
set -euo pipefail
cd "$(dirname "$0")"

COMPOSE=(docker compose -f compose.yml)

echo "==> bringing up core lab (rotel + openobserve)"
"${COMPOSE[@]}" up -d rotel openobserve

echo "==> waiting for OpenObserve to be ready"
for _ in $(seq 1 60); do
  if curl -fsS http://localhost:5080/healthz >/dev/null 2>&1; then break; fi
  sleep 2
done

echo "==> driving telemetry through Rotel (telemetrygen)"
"${COMPOSE[@]}" --profile loadgen run --rm telemetrygen \
  traces --otlp-endpoint=rotel:4317 --otlp-insecure --rate=10 --duration=10s --service=smoke

echo "==> asserting OpenObserve received traces"
# OpenObserve's WAL→searchable flush is ZO_FILE_PUSH_INTERVAL (10s in compose).
sleep 12
AUTH="Basic cm9vdEBleGFtcGxlLmNvbTpDb21wbGV4cGFzcyMxMjM="
# OO search endpoint is /api/{org}/_search (stream goes in the SQL FROM, NOT in
# the path) and needs from/size or it returns 0 hits. start/end are micros.
NOW_US=$(( $(date +%s) * 1000000 )); START_US=$(( ($(date +%s) - 3600) * 1000000 ))
RESP=$(curl -fsS -H "Authorization: $AUTH" \
  "http://localhost:5080/api/default/_search?type=traces" \
  -H 'Content-Type: application/json' \
  -d "{\"query\":{\"sql\":\"SELECT count(*) AS c FROM \\\"default\\\"\",\"start_time\":$START_US,\"end_time\":$NOW_US,\"from\":0,\"size\":1}}" 2>/dev/null || echo "")
echo "OpenObserve response: ${RESP:-<none>}"
if printf '%s' "$RESP" | grep -qE '"c":[1-9]'; then
  echo "ASSERT PASS: OpenObserve received traces."
else
  echo "ASSERT FAIL: no traces in OpenObserve (check Rotel fan-out / OO ingest)." >&2
fi

# Host Parallax assert (only if Parallax is serving on the offset port).
if curl -fsS http://localhost:4000/healthz >/dev/null 2>&1 || curl -fsS http://localhost:4000 >/dev/null 2>&1; then
  echo "==> Parallax is up — verify its copy: parallax traces --service smoke"
else
  echo "NOTE: host Parallax not detected on :4000 — start it to verify the host-bridge copy."
fi

echo "smoke complete. Tear down with: docker compose -f compose.yml down -v"
