#!/usr/bin/env bash
set -euo pipefail

event=${1:?event name is required}
base=${2:-}
head=${3:-HEAD}
zero=0000000000000000000000000000000000000000

require_commit() {
  git cat-file -e "$1^{commit}" 2>/dev/null || {
    printf 'required commit is unavailable: %s\n' "$1" >&2
    exit 1
  }
}

require_commit "$head"
case "$event" in
  pull_request)
    [[ -n "$base" && "$base" != "$zero" ]] || {
      printf 'pull_request requires a non-zero base SHA\n' >&2
      exit 1
    }
    require_commit "$base"
    git diff --name-only "$base" "$head"
    ;;
  push)
    if [[ -z "$base" || "$base" == "$zero" ]]; then
      git diff-tree --root --no-commit-id --name-only -r "$head"
    else
      require_commit "$base"
      git diff --name-only "$base" "$head"
    fi
    ;;
  *)
    printf 'unsupported event for changed-path resolution: %s\n' "$event" >&2
    exit 1
    ;;
esac
