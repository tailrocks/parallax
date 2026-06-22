#!/usr/bin/env bash
# Clone the heavy backends that ship as their own repos (SigNoz) into ./vendor.
# Maple builds from source via compose.maple.yml (no clone needed here).
set -euo pipefail
cd "$(dirname "$0")"
mkdir -p vendor

SIGNOZ_REF="${SIGNOZ_REF:-main}" # pin a tag at implementation
if [ ! -d vendor/signoz ]; then
  echo "cloning SigNoz ($SIGNOZ_REF) into vendor/signoz ..."
  git clone --depth 1 --branch "$SIGNOZ_REF" https://github.com/SigNoz/signoz.git vendor/signoz
else
  echo "vendor/signoz already present — skipping"
fi
echo "done. SigNoz compose: vendor/signoz/deploy/docker/docker-compose.yaml"
