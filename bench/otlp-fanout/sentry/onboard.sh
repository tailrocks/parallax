#!/usr/bin/env bash
# Onboard the running Sentry: create the admin (idempotent) and print the project
# DSN + the exact rotel.env / SENTRY_DSN exports to wire the fan-out.
#
# Run after ./sentry/setup.sh reports the stack is up.
set -euo pipefail
cd "$(dirname "$0")"
VENDOR="$(cd ../vendor/sentry && pwd)"
cd "${VENDOR}"

EMAIL="${SENTRY_ADMIN_EMAIL:-admin@parallax.lab}"
PASSWORD="${SENTRY_ADMIN_PASSWORD:-Complexpass#123}"

echo "==> creating admin ${EMAIL} (ok if it already exists) ..."
docker compose run --rm -T web createuser \
  --email "${EMAIL}" --password "${PASSWORD}" --superuser --no-input 2>&1 \
  | grep -iE "User created|already exists" || true

echo "==> reading the internal project's DSN key ..."
read -r PROJECT_ID PUBLIC_KEY < <(
  printf '%s\n' \
    "from sentry.models.project import Project" \
    "from sentry.models.projectkey import ProjectKey" \
    "p=Project.objects.get(slug='internal')" \
    "k=ProjectKey.objects.filter(project=p).first()" \
    "print('KEYLINE', p.id, k.public_key)" \
  | docker compose run --rm -T web shell 2>/dev/null \
  | awk '/^KEYLINE/{print $2, $3; exit}'
)

if [ -z "${PUBLIC_KEY:-}" ]; then
  echo "ERROR: could not read the project key — is the stack up? (./sentry/setup.sh)" >&2
  exit 1
fi

DSN="http://${PUBLIC_KEY}@localhost:9000/${PROJECT_ID}"
cat <<EOF

Sentry onboarded.
  project id : ${PROJECT_ID}
  public key : ${PUBLIC_KEY}
  DSN        : ${DSN}
  Web UI     : http://localhost:9000  (login: ${EMAIL} / ${PASSWORD})

# --- Wire the Rotel fan-out to Sentry (paste into rotel.env, then add 'sentry'
#     to ROTEL_EXPORTERS + the per-signal lists; traces+logs only, no metrics) ---
ROTEL_EXPORTER_SENTRY_ENDPOINT=http://host.docker.internal:9000/api/${PROJECT_ID}/integration/otlp
ROTEL_EXPORTER_SENTRY_PROTOCOL=http
ROTEL_EXPORTER_SENTRY_CUSTOM_HEADERS=x-sentry-auth=sentry sentry_key=${PUBLIC_KEY}

# --- Point the playground services' Sentry SDK at this DSN (envelope path) ---
export SENTRY_DSN=${DSN}

Verify: ./sentry/verify.sh ${DSN}
EOF
