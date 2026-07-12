#!/usr/bin/env bash
set -euo pipefail

root=$(git rev-parse --show-toplevel)
checker="$root/scripts/ci/source-hygiene.sh"
fixture=$(mktemp -d)
trap 'rm -rf "$fixture"' EXIT

git -C "$fixture" init -q
git -C "$fixture" config user.name fixture
git -C "$fixture" config user.email fixture@example.invalid
printf 'clean\n' > "$fixture/file.txt"
git -C "$fixture" add .
git -C "$fixture" commit -qm initial
initial=$(git -C "$fixture" rev-parse HEAD)
printf 'also clean\n' >> "$fixture/file.txt"
git -C "$fixture" commit -qam clean
head=$(git -C "$fixture" rev-parse HEAD)

(cd "$fixture" && "$checker" ci pull_request "$initial" "$head")
(cd "$fixture" && "$checker" ci push "$initial" "$head")
(cd "$fixture" && "$checker" ci push 0000000000000000000000000000000000000000 "$initial")

printf 'bad trailing space \n' >> "$fixture/file.txt"
if (cd "$fixture" && "$checker" local) >/dev/null 2>&1; then
  printf 'FAIL unstaged whitespace unexpectedly passed\n' >&2
  exit 1
fi
git -C "$fixture" add file.txt
if (cd "$fixture" && "$checker" local) >/dev/null 2>&1; then
  printf 'FAIL staged whitespace unexpectedly passed\n' >&2
  exit 1
fi
if (cd "$fixture" && "$checker" ci push deadbeef "$head") >/dev/null 2>&1; then
  printf 'FAIL missing base unexpectedly passed\n' >&2
  exit 1
fi
printf 'source-hygiene fixtures passed (6 cases)\n'
