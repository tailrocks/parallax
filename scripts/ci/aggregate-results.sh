#!/usr/bin/env bash
set -euo pipefail

needs=${1:?needs JSON is required}
label=${2:-workflow}
printf '%s\n' "$needs"
if jq -e 'to_entries | map(.value.result) | any(. == "failure" or . == "cancelled")' <<<"$needs" >/dev/null; then
  printf 'one or more gated %s jobs failed or were cancelled\n' "$label" >&2
  exit 1
fi
