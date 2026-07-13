#!/usr/bin/env bash
set -euo pipefail

root=$(git rev-parse --show-toplevel)
classifier="$root/scripts/ci/classify-paths.sh"
failures=0

assert_case() {
  local label=$1
  local expected=$2
  shift 2
  local actual
  actual=$($classifier "$@" | tr '\n' ' ' | sed 's/ $//')
  if [[ "$actual" != "$expected" ]]; then
    printf 'FAIL %s\n  expected: %s\n  actual:   %s\n' "$label" "$expected" "$actual" >&2
    failures=$((failures + 1))
  fi
}

assert_case "docs only" 'rust=false ui=false workflows=false advisory=false release=false security=false docs=true' docs/research/note.md
assert_case "Rust only" 'rust=true ui=false workflows=false advisory=false release=true security=false docs=false' crates/parallax-cli/src/main.rs
assert_case "UI only" 'rust=false ui=true workflows=false advisory=false release=true security=false docs=false' ui/src/main.tsx
assert_case "shared toolchain" 'rust=true ui=true workflows=true advisory=true release=true security=false docs=false' mise.toml
assert_case "shared CI" 'rust=true ui=true workflows=true advisory=true release=false security=true docs=true' .github/workflows/ci.yml
assert_case "shared ratchet" 'rust=true ui=true workflows=false advisory=false release=false security=false docs=false' ratchet.toml
assert_case "release only" 'rust=false ui=false workflows=true advisory=false release=true security=true docs=false' .github/workflows/preview.yml
assert_case "deleted path" 'rust=false ui=true workflows=false advisory=false release=true security=false docs=false' ui/src/deleted.ts
assert_case "rename paths" 'rust=true ui=true workflows=false advisory=false release=true security=false docs=false' crates/old.rs ui/src/new.ts
assert_case "mixed paths" 'rust=true ui=true workflows=true advisory=true release=true security=true docs=true' Cargo.lock ui/package.json SECURITY.md scripts/release.sh

if ((failures)); then
  exit 1
fi
printf 'classifier fixtures passed (10 cases)\n'
