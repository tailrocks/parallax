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
rg '^    needs: \[changes, actionlint, source-hygiene, security-hygiene, policy, fmt, docs-links, audit, check, clippy, test, ui, ui-formatter-platform, embed, browser-contracts, browser-full-stack, browser-breadth, closure-final, fuzz-bench\]$' "$workflows/ci.yml" >/dev/null || {
  printf 'ci-required does not explicitly aggregate every required lane\n' >&2
  failures=$((failures + 1))
}
if rg -n '^\s*permissions:\s*write-all' "$workflows"; then
  printf 'write-all workflow permission found\n' >&2
  failures=$((failures + 1))
fi
rg '^      - run: cargo xtask semconv check$' "$workflows/ci.yml" >/dev/null || {
  printf 'policy lane does not enforce generated semantic conventions\n' >&2
  failures=$((failures + 1))
}
rg -U '^  closure-final:\n(?:.*\n)*?    permissions:\n      contents: read\n(?:.*\n)*?          install_args: "rust bun actionlint"\n(?:.*\n)*?          install_args: "cargo-binstall aqua:nextest-rs/nextest/cargo-nextest cargo:cargo-audit cargo:cargo-deny cargo:cargo-hack cargo:cargo-shear"\n(?:.*\n)*?            scripts/ci/closure-baseline.sh\n            cargo xtask closure-final$' "$workflows/ci.yml" >/dev/null || {
  printf 'closure-final does not run its full read-only baseline with required tools\n' >&2
  failures=$((failures + 1))
}
baseline="$root/scripts/ci/closure-baseline.sh"
if [[ ! -x "$baseline" ]]; then
  printf 'closure baseline is not executable\n' >&2
  failures=$((failures + 1))
fi
for command in \
  'git diff --check' \
  'git diff --cached --check' \
  'cargo fmt --all --check' \
  'cargo check --locked --workspace --all-targets' \
  'cargo clippy --locked --workspace --all-targets -- -D warnings' \
  'cargo xtask dependencies --all' \
  'cargo xtask ui graphql check' \
  'cargo xtask policy --only ui.runtime-boundaries' \
  'cargo nextest run --locked --workspace --all-targets --profile ci' \
  'cargo xtask ci --full' \
  'bun ci' \
  'bun run check' \
  'bun run lint' \
  'bun run typecheck' \
  'bun run --bun test:ci' \
  'bun run build' \
  'bun run test:browser' \
  'bun run test:browser:cross' \
  'bun run test:browser:a11y' \
  'bun run test:browser:visual' \
  'bun run test:browser:full' \
  'bun run perf:live' \
  'cargo xtask ui-bundle analyze' \
  'cargo xtask ui-bundle build-twice' \
  'mise exec -- actionlint'; do
  sed 's/^[[:space:]]*//' "$baseline" | rg -F -x "$command" >/dev/null || {
    printf 'closure baseline omits command: %s\n' "$command" >&2
    failures=$((failures + 1))
  }
done

if ((failures)); then
  exit 1
fi
printf 'workflow policy fixtures passed\n'
