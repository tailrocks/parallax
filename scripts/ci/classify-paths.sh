#!/usr/bin/env bash
set -euo pipefail

rust=false
ui=false
workflows=false
advisory=false
release=false
security=false
docs=false

classify() {
  local path=$1

  case "$path" in
    *.md|Cargo.toml|Cargo.lock|rust-toolchain.toml|crates/parallax-xtask/Cargo.toml|crates/parallax-xtask/src/docs_links.rs|crates/parallax-xtask/src/docs_links/*|crates/parallax-xtask/src/cli.rs|crates/parallax-xtask/src/command.rs|crates/parallax-xtask/src/lib.rs|.github/workflows/ci.yml|scripts/ci/classify-paths.sh|scripts/ci/test-classify-paths.sh|scripts/ci/test-workflow-policy.sh)
      docs=true
      ;;
  esac
  case "$path" in
    .cargo/*|Cargo.toml|Cargo.lock|rust-toolchain.toml|ratchet.toml|telemetry/semconv/*|crates/*|poc/*)
      rust=true
      ;;
  esac
  case "$path" in
    ratchet.toml|telemetry/semconv/*|ui/*)
      ui=true
      ;;
  esac
  case "$path" in
    .github/workflows/*|.github/actions/*|scripts/*)
      workflows=true
      ;;
  esac
  case "$path" in
    Cargo.toml|Cargo.lock|mise.toml)
      advisory=true
      ;;
  esac
  case "$path" in
    .github/workflows/release.yml|.github/workflows/preview.yml|.github/actions/sign-and-attest-archive/*|scripts/release.sh|Cargo.toml|Cargo.lock|rust-toolchain.toml|mise.toml|crates/*|ui/*)
      release=true
      ;;
  esac
  case "$path" in
    SECURITY.md|CONTRIBUTING.md|REPOSITORY_PROTECTION.md|BRANCHING.md|COMMITS.md|AGENTS.md|.github/workflows/*|.github/actions/*)
      security=true
      ;;
  esac
  case "$path" in
    mise.toml)
      rust=true
      ui=true
      workflows=true
      ;;
    .github/workflows/ci.yml)
      rust=true
      ui=true
      advisory=true
      ;;
  esac
}

if (($#)); then
  for path in "$@"; do
    classify "$path"
  done
else
  while IFS= read -r path; do
    [[ -n "$path" ]] && classify "$path"
  done
fi

printf 'rust=%s\n' "$rust"
printf 'ui=%s\n' "$ui"
printf 'workflows=%s\n' "$workflows"
printf 'advisory=%s\n' "$advisory"
printf 'release=%s\n' "$release"
printf 'security=%s\n' "$security"
printf 'docs=%s\n' "$docs"
