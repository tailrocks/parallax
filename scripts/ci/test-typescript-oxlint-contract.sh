#!/usr/bin/env bash
set -euo pipefail

root=$(git rev-parse --show-toplevel)
ui="$root/ui"
package="$ui/package.json"
lock="$ui/bun.lock"

[[ $(jq -r '.devDependencies.typescript' "$package") == 7.0.2 ]]
[[ $(jq -r '.devDependencies.oxlint' "$package") == 1.73.0 ]]
[[ $(jq -r '.devDependencies["oxlint-tsgolint"]' "$package") == 0.24.0 ]]

if rg -n '(@tanstack/eslint-config|typescript-eslint|@typescript-eslint|eslint-plugin|@oxlint/migrate|typescript@6)' "$package" "$lock"; then
  printf 'legacy TypeScript/ESLint graph is reachable from the final package graph\n' >&2
  exit 1
fi
[[ ! -e "$ui/eslint.config.js" ]]
[[ ! -e "$ui/eslint.config.mjs" ]]

hash_stream() {
  shasum -a 256 | awk '{print $1}'
}

selected=$(cd "$ui" && bun ./node_modules/oxlint/bin/oxlint --debug=files .)
[[ $(printf '%s\n' "$selected" | wc -l | tr -d ' ') == 151 ]]
[[ $(printf '%s\n' "$selected" | hash_stream) == ebb965980822201e59b37286bdef0e3933901795ad6a77e5fd1c5c6d22ed1bbe ]]

config=$(cd "$ui" && bun ./node_modules/oxlint/bin/oxlint --print-config)
[[ $(printf '%s\n' "$config" | hash_stream) == ebcc47b1b91ce91f0e19cc0f260992e656cbddd6a0a1d9d1ab73aa9b837fc04d ]]
[[ $(cd "$ui" && bun ./node_modules/typescript/bin/tsc --showConfig | hash_stream) == f6b94e460cb728ea095b0a7138c731f3ddf9f89a8faa10d0d18480c8933b8083 ]]

probe="$ui/plan131-negative-probe.ts"
trap 'rm -f "$probe"' EXIT
printf '%s\n' 'Promise.resolve(1)' >"$probe"
if (cd "$ui" && bun ./node_modules/oxlint/bin/oxlint --type-aware "$probe" >/dev/null 2>&1); then
  printf 'type-aware negative fixture unexpectedly passed\n' >&2
  exit 1
fi
printf '%s\n' 'const plan131Number: number = "not a number"' >"$probe"
if (cd "$ui" && bun ./node_modules/typescript/bin/tsc --noEmit >/dev/null 2>&1); then
  printf 'TypeScript negative fixture unexpectedly passed\n' >&2
  exit 1
fi

printf 'TypeScript/Oxlint contract passed (151 selected files)\n'
