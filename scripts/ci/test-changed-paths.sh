#!/usr/bin/env bash
set -euo pipefail

root=$(git rev-parse --show-toplevel)
resolver="$root/scripts/ci/changed-paths.sh"
fixture=$(mktemp -d)
trap 'rm -rf "$fixture"' EXIT

git -C "$fixture" init -q
git -C "$fixture" config user.name fixture
git -C "$fixture" config user.email fixture@example.invalid
mkdir -p "$fixture/ui/src"
printf 'old\n' > "$fixture/ui/src/old.ts"
git -C "$fixture" add .
git -C "$fixture" commit -qm initial
initial=$(git -C "$fixture" rev-parse HEAD)
git -C "$fixture" mv ui/src/old.ts ui/src/new.ts
printf 'new\n' > "$fixture/ui/src/new.ts"
git -C "$fixture" commit -qam rename
head=$(git -C "$fixture" rev-parse HEAD)

assert_output() {
  local label=$1
  local expected=$2
  shift 2
  local actual
  actual=$(cd "$fixture" && "$resolver" "$@")
  if [[ "$actual" != "$expected" ]]; then
    printf 'FAIL %s\n  expected: %s\n  actual:   %s\n' "$label" "$expected" "$actual" >&2
    exit 1
  fi
}

renamed_paths=$'ui/src/new.ts\nui/src/old.ts'
assert_output "pull request range" "$renamed_paths" pull_request "$initial" "$head"
assert_output "push range" "$renamed_paths" push "$initial" "$head"
assert_output "initial push" 'ui/src/old.ts' push 0000000000000000000000000000000000000000 "$initial"

if (cd "$fixture" && "$resolver" push deadbeef "$head") >/dev/null 2>&1; then
  printf 'FAIL missing base unexpectedly succeeded\n' >&2
  exit 1
fi
printf 'changed-range fixtures passed (4 cases)\n'
