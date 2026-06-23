#!/usr/bin/env bash
# Clone the heavy backends that ship as their own repos (SigNoz) into ./vendor.
# Maple builds from source via compose.maple.yml (no clone needed here).
set -euo pipefail
cd "$(dirname "$0")"
mkdir -p vendor

# Pin a release tag, NOT main (verified 2026-06-23: main and v0.129.0 both bring
# the whole stack up, but SigNoz's otel-collector is OpAMP-managed by the SigNoz
# server — its OTLP :4317 receiver only binds after the server pushes a config,
# which needs the server fully onboarded; `docker compose up` alone leaves 4317
# closed. See README "SigNoz" for the run finding.)
SIGNOZ_REF="${SIGNOZ_REF:-v0.129.0}"
if [ ! -d vendor/signoz ]; then
  echo "cloning SigNoz ($SIGNOZ_REF) into vendor/signoz ..."
  git clone --depth 1 --branch "$SIGNOZ_REF" https://github.com/SigNoz/signoz.git vendor/signoz
else
  echo "vendor/signoz already present — skipping"
fi
echo "done. SigNoz compose: vendor/signoz/deploy/docker/docker-compose.yaml"
