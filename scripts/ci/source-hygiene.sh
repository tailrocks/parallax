#!/usr/bin/env bash
set -euo pipefail

mode=${1:?mode is required}
case "$mode" in
  ci)
    event=${2:?event is required}
    base=${3:-}
    head=${4:?head SHA is required}
    zero=0000000000000000000000000000000000000000
    git cat-file -e "$head^{commit}" 2>/dev/null || {
      printf 'required head commit is unavailable: %s\n' "$head" >&2
      exit 1
    }
    if [[ "$event" == push && ( -z "$base" || "$base" == "$zero" ) ]]; then
      empty_tree=$(git hash-object -t tree /dev/null)
      git diff --check "$empty_tree" "$head"
    else
      [[ "$event" == pull_request || "$event" == push ]] || {
        printf 'unsupported source-hygiene event: %s\n' "$event" >&2
        exit 1
      }
      git cat-file -e "$base^{commit}" 2>/dev/null || {
        printf 'required base commit is unavailable: %s\n' "$base" >&2
        exit 1
      }
      git diff --check "$base" "$head"
    fi
    ;;
  local)
    git diff --check
    git diff --cached --check
    ;;
  *)
    printf 'unknown source-hygiene mode: %s\n' "$mode" >&2
    exit 1
    ;;
esac
