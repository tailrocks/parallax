#!/usr/bin/env bash
# Verify the running Sentry stack end-to-end on the local machine:
#   1. A1   — native OTLP trace ingest (Rotel's fan-out path) returns 200.
#   2. A15  — N identical errors group into ONE Sentry issue.
#   3. A16  — that issue's event count rises with each occurrence (times_seen).
#
# Usage: ./sentry/verify.sh [DSN]
#   DSN defaults to the internal project: http://<key>@localhost:9000/1
# Reads the key from the DSN, drives events, then asserts via the Group model.
set -euo pipefail
cd "$(dirname "$0")"
VENDOR="$(cd ../vendor/sentry && pwd)"

DSN="${1:-${SENTRY_DSN:-}}"
if [ -z "${DSN}" ]; then
  echo "ERROR: pass a DSN (or set SENTRY_DSN). Get it from ./sentry/onboard.sh" >&2
  exit 1
fi
# parse http://<key>@<host:port>/<projid>
KEY="$(printf '%s' "${DSN}" | sed -E 's#^https?://([^@]+)@.*#\1#')"
HOSTPORT="$(printf '%s' "${DSN}" | sed -E 's#^https?://[^@]+@([^/]+)/.*#\1#')"
PROJ="$(printf '%s' "${DSN}" | sed -E 's#.*/([0-9]+)$#\1#')"
BASE="http://${HOSTPORT}"
N="${SENTRY_VERIFY_COUNT:-5}"

echo "==> A1: native OTLP trace ingest → ${BASE}/api/${PROJ}/integration/otlp/v1/traces"
NOW="$(python3 -c 'import time;print(int(time.time()*1e9))')"
END="$(python3 -c 'import time;print(int(time.time()*1e9)+5000000)')"
TID="$(python3 -c 'import os;print(os.urandom(16).hex())')"
SID="$(python3 -c 'import os;print(os.urandom(8).hex())')"
OTLP_CODE="$(curl -s -o /dev/null -w '%{http_code}' \
  -X POST "${BASE}/api/${PROJ}/integration/otlp/v1/traces" \
  -H "x-sentry-auth: sentry sentry_key=${KEY}" -H 'content-type: application/json' \
  -d "{\"resourceSpans\":[{\"resource\":{\"attributes\":[{\"key\":\"service.name\",\"value\":{\"stringValue\":\"sentry-verify\"}}]},\"scopeSpans\":[{\"spans\":[{\"traceId\":\"${TID}\",\"spanId\":\"${SID}\",\"name\":\"verify-span\",\"kind\":2,\"startTimeUnixNano\":\"${NOW}\",\"endTimeUnixNano\":\"${END}\"}]}]}]}")"
echo "    OTLP ingest HTTP ${OTLP_CODE}"

echo "==> A15/A16: posting ${N} identical errors (same fingerprint) ..."
for _ in $(seq 1 "${N}"); do
  EID="$(python3 -c 'import os;print(os.urandom(16).hex())')"
  TS="$(python3 -c 'import time;print(int(time.time()))')"
  curl -s -o /dev/null -X POST "${BASE}/api/${PROJ}/store/" \
    -H "X-Sentry-Auth: Sentry sentry_version=7, sentry_key=${KEY}" \
    -H 'content-type: application/json' \
    -d "{\"event_id\":\"${EID}\",\"timestamp\":${TS},\"platform\":\"native\",\"level\":\"error\",\"logger\":\"checkout\",\"exception\":{\"values\":[{\"type\":\"PaymentError\",\"value\":\"payment failure (chaos)\"}]},\"fingerprint\":[\"payment-failure-chaos\"]}"
done

echo "==> waiting for ingest/processing ..."
sleep 12

echo "==> asserting one grouped issue with times_seen >= ${N} ..."
RESULT="$(
  printf '%s\n' \
    "from sentry.models.group import Group" \
    "g=Group.objects.filter(message__icontains='payment failure').order_by('-times_seen').first()" \
    "print('VERIFY %s times_seen=%d' % ((g.title if g else 'NONE'), (g.times_seen if g else 0)))" \
  | (cd "${VENDOR}" && docker compose run --rm -T web shell 2>/dev/null) \
  | awk '/^VERIFY /{ $1=""; print substr($0,2); exit }'
)"
echo "    ${RESULT}"
SEEN="$(printf '%s' "${RESULT}" | sed -E 's/.*times_seen=([0-9]+).*/\1/')"
if [ "${OTLP_CODE}" = "200" ] && [ "${SEEN:-0}" -ge "${N}" ] 2>/dev/null; then
  echo "ASSERT PASS: OTLP ingest=200; ${N} errors grouped into one issue (times_seen=${SEEN})."
else
  echo "ASSERT FAIL: OTLP=${OTLP_CODE}, times_seen=${SEEN:-0} (expected >= ${N})." >&2
  exit 1
fi
