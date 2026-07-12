#!/usr/bin/env bash
set -euo pipefail

root=$(git rev-parse --show-toplevel)
workflows="$root/.github/workflows"
failures=0

while IFS=: read -r file line value; do
  value=${value#*uses: }
  value=${value%% *}
  case "$value" in
    ./*|docker://*) continue ;;
  esac
  if [[ ! "$value" =~ @[0-9a-f]{40}$ ]]; then
    printf 'mutable action reference: %s:%s: %s\n' "$file" "$line" "$value" >&2
    failures=$((failures + 1))
  fi
done < <(rg -n --no-heading '^\s*- uses:' "$workflows")

rg -U '^  clippy:\n    needs: changes$' "$workflows/ci.yml" >/dev/null || {
  printf 'Clippy is not a sibling of check\n' >&2
  failures=$((failures + 1))
}
rg '^    needs: \[changes, actionlint, source-hygiene, policy, fmt, audit, check, clippy, test, ui, embed\]$' "$workflows/ci.yml" >/dev/null || {
  printf 'ci-required does not explicitly aggregate every required lane\n' >&2
  failures=$((failures + 1))
}
if rg -n '^\s*permissions:\s*write-all' "$workflows"; then
  printf 'write-all workflow permission found\n' >&2
  failures=$((failures + 1))
fi

if ((failures)); then
  exit 1
fi
printf 'workflow policy fixtures passed\n'
