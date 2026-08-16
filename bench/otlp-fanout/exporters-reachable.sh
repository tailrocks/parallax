#!/usr/bin/env bash
# Assert Rotel exporter config is internally consistent, then (unless
# --parse-only) TCP-probe every listed exporter so a down sink cannot sit
# silently on the sequential fan-out list.
#
# Usage:
#   ./exporters-reachable.sh                 # parse rotel.env + probe
#   ./exporters-reachable.sh --parse-only    # parse only (CI / no live lab)
#   ./exporters-reachable.sh --env FILE
set -euo pipefail
cd "$(dirname "$0")"

PARSE_ONLY=0
ENV_FILE="rotel.env"
while [ $# -gt 0 ]; do
  case "$1" in
    --parse-only) PARSE_ONLY=1 ;;
    --env) ENV_FILE="$2"; shift ;;
    *) echo "unknown arg: $1" >&2; exit 2 ;;
  esac
  shift
done

if [ ! -f "$ENV_FILE" ]; then
  echo "missing $ENV_FILE" >&2
  exit 1
fi

# shellcheck disable=SC1090
set -a
# rotel.env is KEY=value; ignore comments / blanks
# shellcheck source=/dev/null
. /dev/null
set +a

declare_line="$(grep -E '^ROTEL_EXPORTERS=' "$ENV_FILE" | tail -1 || true)"
if [ -z "$declare_line" ]; then
  echo "no ROTEL_EXPORTERS= in $ENV_FILE" >&2
  exit 1
fi
raw="${declare_line#ROTEL_EXPORTERS=}"
IFS=',' read -r -a specs <<< "$raw"

names=()
for spec in "${specs[@]}"; do
  spec="${spec//$'\r'/}"
  spec="$(printf '%s' "$spec" | tr -d '[:space:]')"
  [ -z "$spec" ] && continue
  name="${spec%%:*}"
  names+=("$name")
done

if [ "${#names[@]}" -eq 0 ]; then
  echo "ROTEL_EXPORTERS is empty" >&2
  exit 1
fi

fail=0
for name in "${names[@]}"; do
  key="ROTEL_EXPORTER_$(printf '%s' "$name" | tr '[:lower:]' '[:upper:]')_ENDPOINT"
  line="$(grep -E "^${key}=" "$ENV_FILE" | tail -1 || true)"
  if [ -z "$line" ]; then
    echo "FAIL parse: $name listed but $key missing"
    fail=1
    continue
  fi
  endpoint="${line#*=}"
  echo "parse ok  $name -> $endpoint"
  if [ "$PARSE_ONLY" -eq 1 ]; then
    continue
  fi
  hostport="${endpoint#http://}"
  hostport="${hostport#https://}"
  hostport="${hostport%%/*}"
  host="${hostport%%:*}"
  port="${hostport##*:}"
  if [ "$host" = "$port" ]; then
    port=80
  fi
  probe_host="$host"
  if [ "$host" = "host.docker.internal" ]; then
    probe_host="127.0.0.1"
  fi
  if [ "$host" = "127.0.0.1" ] || [ "$host" = "localhost" ] || [ "$host" = "host.docker.internal" ]; then
    if nc -z -G 2 "$probe_host" "$port" >/dev/null 2>&1; then
      echo "probe ok  $name $probe_host:$port"
    else
      echo "FAIL probe: $name $probe_host:$port unreachable — remove from ROTEL_EXPORTERS"
      fail=1
    fi
    continue
  fi
  # Compose-network service: probe from a lab-network container.
  if ! docker network inspect parallax-otlp-fanout >/dev/null 2>&1; then
    echo "FAIL probe: $name $host:$port (lab network missing)"
    fail=1
    continue
  fi
  if docker run --rm --network parallax-otlp-fanout busybox:1.37.0 \
      /bin/sh -c "nc -z -w 3 $host $port" >/dev/null 2>&1; then
    echo "probe ok  $name $host:$port (lab net)"
  else
    echo "FAIL probe: $name $host:$port unreachable on lab net — remove from ROTEL_EXPORTERS"
    fail=1
  fi
done

if [ "$fail" -ne 0 ]; then
  exit 1
fi
echo "exporters-reachable: ${#names[@]} ok (${ENV_FILE})"
