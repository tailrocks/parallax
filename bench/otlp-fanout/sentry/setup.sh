#!/usr/bin/env bash
# Stand up self-hosted Sentry as part of the lab — a real Docker Compose stack
# you run and verify locally (operator, 2026-06-23: Sentry is NOT deferred; it is
# a first-class, runnable part of this lab).
#
# getsentry/self-hosted is ~72 services bootstrapped by its own install.sh (it is
# NOT a clean `include:` target), so we vendor it under ./vendor/sentry, run its
# installer non-interactively, then `docker compose up`. Rotel fans out to it over
# the host bridge (host.docker.internal:9000 → nginx → relay), so no network-join
# is needed. Verified end-to-end on v26.6.0 (2026-06-23): native OTLP ingest +
# A15/A16 issue grouping.
set -euo pipefail
cd "$(dirname "$0")"
ROOT="$(cd .. && pwd)"

SENTRY_REF="${SENTRY_REF:-26.6.0}"   # pin a release ≥25.8.0 (native OTLP). Verified: 26.6.0
VENDOR="${ROOT}/vendor/sentry"

# --- install.sh needs bash >= 4.4; macOS ships 3.2. Find a modern bash. ---
need_bash() {
  for b in bash /opt/homebrew/bin/bash /usr/local/bin/bash; do
    v="$("$b" -c 'echo "${BASH_VERSINFO[0]}${BASH_VERSINFO[1]}"' 2>/dev/null || echo 0)"
    if [ "${v:-0}" -ge 44 ] 2>/dev/null; then echo "$b"; return 0; fi
  done
  return 1
}
BASH5="$(need_bash || true)"
if [ -z "${BASH5:-}" ]; then
  echo "ERROR: getsentry/self-hosted install.sh needs bash >= 4.4 (macOS ships 3.2)." >&2
  echo "       Install one:  brew install bash   then re-run this script." >&2
  exit 1
fi
echo "==> using bash: ${BASH5} ($("${BASH5}" --version | head -1))"

# --- vendor the stack ---
mkdir -p "${ROOT}/vendor"
if [ ! -d "${VENDOR}" ]; then
  echo "==> cloning getsentry/self-hosted ${SENTRY_REF} into vendor/sentry ..."
  git clone --depth 1 --branch "${SENTRY_REF}" \
    https://github.com/getsentry/self-hosted.git "${VENDOR}"
else
  echo "==> vendor/sentry already present — skipping clone"
fi

# --- install (non-interactive) + bring up ---
cd "${VENDOR}"
echo "==> running install.sh (non-interactive; pulls images, runs migrations — 20-40 min) ..."
PATH="$(dirname "${BASH5}"):${PATH}" "${BASH5}" ./install.sh \
  --skip-user-creation --no-report-self-hosted-issues --skip-commit-check

echo "==> bringing up the Sentry stack (docker compose up -d --wait) ..."
docker compose up -d --wait

echo
echo "Sentry is up. Web UI: http://localhost:9000  (SENTRY_BIND)"
echo "Next: ./sentry/onboard.sh   # create admin + print the DSN and rotel.env exports"
