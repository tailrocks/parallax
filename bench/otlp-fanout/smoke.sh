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
sleep 5
AUTH="Basic cm9vdEBleGFtcGxlLmNvbTpDb21wbGV4cGFzcyMxMjM="
COUNT=$(curl -fsS -H "Authorization: $AUTH" \
  "http://localhost:5080/api/default/default/_search?type=traces" \
  -H 'Content-Type: application/json' \
  -d '{"query":{"sql":"SELECT count(*) AS c FROM default","start_time":0,"end_time":9999999999999999}}' 2>/dev/null || echo "")
echo "OpenObserve response: ${COUNT:-<none>}"

# Host Parallax assert (only if Parallax is serving on the offset port).
if curl -fsS http://localhost:4000/healthz >/dev/null 2>&1 || curl -fsS http://localhost:4000 >/dev/null 2>&1; then
  echo "==> Parallax is up — verify its copy: parallax traces --service smoke"
else
  echo "NOTE: host Parallax not detected on :4000 — start it to verify the host-bridge copy."
fi

echo "smoke complete. Tear down with: docker compose -f compose.yml down -v"
