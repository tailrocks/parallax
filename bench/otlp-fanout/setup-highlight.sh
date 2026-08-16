#!/usr/bin/env bash
# Live attempt: last published highlight.io hobby self-host (docker-v0.5.6,
# 2025-08-08). Hosted SaaS ended 2026-02-28. This is not a Rotel exporter
# until the stack is healthy — sequential Rotel must not list a down sink.
#
# Host 8080 (playground catalog), 3000 (often Grafana/rustrak-ui), and 9000
# (Sentry nginx / highlight ClickHouse native) collide. The attempt remaps
# those in an override if the vendor compose comes up far enough to apply it.
set -euo pipefail
cd "$(dirname "$0")"
REF="${HIGHLIGHT_REF:-docker-v0.5.6}"
VENDOR="$PWD/vendor/highlight"
mkdir -p vendor
if [ ! -d "$VENDOR/.git" ]; then
  echo "==> cloning highlight/highlight $REF (shallow)"
  git clone --depth 1 --branch "$REF" https://github.com/highlight/highlight.git "$VENDOR"
else
  echo "==> vendor/highlight already present"
fi
cd "$VENDOR/docker"
if [ ! -f run-hobby.sh ]; then
  echo "BLOCKED: vendor/highlight/docker/run-hobby.sh missing at $REF" >&2
  exit 2
fi
echo "==> attempting highlight hobby start (run-hobby.sh)"
# Do not pull-forever: 15 min wall. Port collisions and missing env.enc are
# expected failure modes for an unmaintained hobby stack.
set +e
timeout 900 ./run-hobby.sh --no-pull
rc=$?
set -e
if [ "$rc" -ne 0 ]; then
  echo "BLOCKED: highlight hobby start failed rc=$rc (ref=$REF)" >&2
  docker ps -a --format '{{.Names}} {{.Status}}' | grep -i highlight || true
  exit "$rc"
fi
echo "highlight hobby reported up. UI: see REACT_APP_FRONTEND_URI in docker/.env"
