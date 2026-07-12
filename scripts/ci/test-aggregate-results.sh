#!/usr/bin/env bash
set -euo pipefail

root=$(git rev-parse --show-toplevel)
aggregate="$root/scripts/ci/aggregate-results.sh"

pass() {
  "$aggregate" "$2" fixture >/dev/null || {
    printf 'FAIL expected success: %s\n' "$1" >&2
    exit 1
  }
}

fail() {
  if "$aggregate" "$2" fixture >/dev/null 2>&1; then
    printf 'FAIL expected failure: %s\n' "$1" >&2
    exit 1
  fi
}

pass success '{"check":{"result":"success"}}'
pass skipped '{"check":{"result":"skipped"},"ui":{"result":"success"}}'
fail failure '{"check":{"result":"failure"},"ui":{"result":"success"}}'
fail cancelled '{"check":{"result":"cancelled"},"ui":{"result":"skipped"}}'
printf 'aggregate fixtures passed (4 cases)\n'
