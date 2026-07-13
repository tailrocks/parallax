#!/usr/bin/env bash
set -euo pipefail

root=$(git rev-parse --show-toplevel)
ui="$root/ui"

rg -U '^\[run\]\nbun = true\n\n\[install\]\nauto = "disable"$' "$ui/bunfig.toml" >/dev/null

while IFS=$'\t' read -r name command; do
  case "$name" in
    dev|build|preview|test|test:ci|lint|format|check|typecheck)
      [[ "$command" == "bunx --bun --no-install "* ||
          "$command" == "bun ./node_modules/"* ||
          "$command" == "bun run "* ]] || {
        printf 'script does not enforce lock-local Bun execution: %s\n' "$name" >&2
        exit 1
      }
      if [[ "$command" == "bunx --bun --no-install "* ]]; then
        executable=${command#bunx --bun --no-install }
        executable=${executable%% *}
        [[ -x "$ui/node_modules/.bin/$executable" ]] || {
          printf 'script executable is not installed locally: %s\n' "$executable" >&2
          exit 1
        }
      elif [[ "$command" == "bun ./node_modules/"* ]]; then
        executable=${command#bun }
        executable=${executable%% *}
        [[ -f "$ui/${executable#./}" ]] || {
          printf 'script executable is not installed locally: %s\n' "$executable" >&2
          exit 1
        }
      fi
      ;;
  esac
done < <(jq -r '.scripts | to_entries[] | [.key, .value] | @tsv' "$ui/package.json")

if rg -n 'bunx[^\n]*@latest' "$ui/package.json" "$ui/README.md" "$ui/AGENTS.md"; then
  printf 'mutable package command found in active UI metadata\n' >&2
  exit 1
fi
if jq -e '.scripts[] | test("(^|[;&|]\\s*)(npm|npx|pnpm|yarn)(\\s|$)")' "$ui/package.json" >/dev/null; then
  printf 'foreign package-manager command found in package scripts\n' >&2
  exit 1
fi

for forbidden in package-lock.json pnpm-lock.yaml yarn.lock pnpm-workspace.yaml .npmrc; do
  [[ ! -e "$root/$forbidden" && ! -e "$ui/$forbidden" ]] || {
    printf 'foreign package-manager artifact found: %s\n' "$forbidden" >&2
    exit 1
  }
done

probe=$(cd "$ui" && bun run --silent scripts/runtime-probe.ts)
[[ "$probe" == bun-runtime=* ]]
script_count=$(jq '.scripts | length' "$ui/package.json")
printf 'Bun execution contract fixtures passed (%s scripts)\n' "$script_count"
