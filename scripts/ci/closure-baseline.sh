#!/usr/bin/env bash
set -euo pipefail

root=$(git rev-parse --show-toplevel)
cd "$root"

git diff --check
git diff --cached --check
cargo fmt --all --check
cargo check --locked --workspace --all-targets
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo xtask dependencies --all
cargo xtask ui graphql check
cargo xtask policy --only ui.runtime-boundaries
cargo nextest run --locked --workspace --all-targets --profile ci
cargo xtask ci --full
(
  cd ui
  bun ci
  bun run check
  bun run lint
  bun run typecheck
  bun run --bun test:ci
  bun run build
  bun run test:browser
  bun run test:browser:cross
  bun run test:browser:a11y
  bun run test:browser:visual
  bun run test:browser:full
  bun run perf:live
)
cargo xtask ui-bundle analyze
cargo xtask ui-bundle build-twice
mise exec -- actionlint
