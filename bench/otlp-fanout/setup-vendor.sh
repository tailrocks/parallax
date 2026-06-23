#!/usr/bin/env bash
# Clone the heavy backends that ship as their own repos (SigNoz) into ./vendor.
# Maple builds from source via compose.maple.yml (no clone needed here).
set -euo pipefail
cd "$(dirname "$0")"
mkdir -p vendor

# Pin a release tag, NOT main (verified end-to-end 2026-06-23 on v0.129.0:
# Rotel -> SigNoz collector -> ClickHouse, 8 spans). SigNoz's otel-collector is
# OpAMP-managed: its OTLP :4317 receiver binds only after the server pushes a
# config, which the server does only after the FIRST org/admin is created. So
# after `compose up` you must register the first user once (see README "SigNoz"
# for the exact /api/v1/register call); `compose up` alone leaves :4317 closed.
SIGNOZ_REF="${SIGNOZ_REF:-v0.129.0}"
if [ ! -d vendor/signoz ]; then
  echo "cloning SigNoz ($SIGNOZ_REF) into vendor/signoz ..."
  git clone --depth 1 --branch "$SIGNOZ_REF" https://github.com/SigNoz/signoz.git vendor/signoz
else
  echo "vendor/signoz already present — skipping"
fi
echo "done. SigNoz compose: vendor/signoz/deploy/docker/docker-compose.yaml"
