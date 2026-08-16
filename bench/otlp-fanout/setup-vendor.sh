#!/usr/bin/env bash
# Clone the heavy backends that ship as their own repos (SigNoz) into ./vendor.
# Maple builds from source via compose.maple.yml (no clone needed here).
set -euo pipefail
cd "$(dirname "$0")"
mkdir -p vendor

# Last bootable community compose is v0.129.0 (verified 2026-06-23: Rotel ->
# SigNoz collector -> ClickHouse). v0.137.0+ removed
# deploy/docker/docker-compose.yaml (Foundry-only). Do not invent Foundry.
# SigNoz otel-collector is OpAMP-managed: OTLP :4317 binds only after the
# server pushes a config, which happens only after the FIRST org/admin is
# created. After `compose up` register once (see README); compose alone
# leaves :4317 closed.
SIGNOZ_REF="${SIGNOZ_REF:-v0.129.0}"
COMPOSE_PATH="vendor/signoz/deploy/docker/docker-compose.yaml"
if [ -d vendor/signoz ] && [ ! -f "$COMPOSE_PATH" ]; then
  echo "vendor/signoz has no community compose (Foundry-only tree) — moving aside"
  mv vendor/signoz "vendor/signoz-foundry-$(date +%Y%m%dT%H%M%S)"
fi
if [ ! -d vendor/signoz ]; then
  echo "cloning SigNoz ($SIGNOZ_REF) into vendor/signoz ..."
  git clone --depth 1 --branch "$SIGNOZ_REF" https://github.com/SigNoz/signoz.git vendor/signoz
else
  echo "vendor/signoz already present — skipping"
fi
if [ -f "$COMPOSE_PATH" ]; then
  echo "done. SigNoz compose: $COMPOSE_PATH"
else
  echo "ERROR: $COMPOSE_PATH missing after clone of $SIGNOZ_REF (Foundry-only?)" >&2
  exit 1
fi
