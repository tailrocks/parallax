#!/usr/bin/env bash
set -euo pipefail
# Same C locale as test-oxfmt-contract.sh so file-list hashes match Ubuntu CI.
export LC_ALL=C

root=$(git rev-parse --show-toplevel)
ui="$root/ui"
package="$ui/package.json"
lock="$ui/bun.lock"

[[ $(jq -r '.devDependencies.typescript' "$package") == 7.0.2 ]]
[[ $(jq -r '.devDependencies.oxlint' "$package") == 1.73.0 ]]
[[ $(jq -r '.devDependencies["oxlint-tsgolint"]' "$package") == 0.24.0 ]]
compiler=$(cd "$ui" && bun -e 'import getExePath from "./node_modules/typescript/lib/getExePath.js"; console.log(getExePath())')
platform=$(cd "$ui" && bun -e 'console.log(process.platform + "-" + process.arch)')
if [[ "$compiler" != *"/@typescript/typescript-$platform/lib/tsc" || ! -x "$compiler" ]]; then
  printf 'typescript compiler path=%s platform=%s\n' "$compiler" "$platform" >&2
  exit 1
fi
[[ $(jq -r '.options.typeCheck' "$ui/.oxlintrc.jsonc") == false ]]
[[ $(jq -r 'has("jsPlugins")' "$ui/.oxlintrc.jsonc") == false ]]
[[ $(jq -r '.categories.nursery // "absent"' "$ui/.oxlintrc.jsonc") == absent ]]

if rg -n '(@tanstack/eslint-config|typescript-eslint|@typescript-eslint|eslint-plugin|@oxlint/migrate)' "$package" "$lock" ||
  rg -n '"typescript": \["typescript@6' "$lock"; then
  printf 'legacy TypeScript/ESLint graph is reachable from the final package graph\n' >&2
  exit 1
fi
[[ ! -e "$ui/eslint.config.js" ]]
[[ ! -e "$ui/eslint.config.mjs" ]]
forbidden=$(git -C "$root" ls-files '*.js' '*.jsx' '*.mjs' '*.cjs' '*.mts' '*.cts' | while read -r path; do
  [[ ! -e "$root/$path" ]] || printf '%s\n' "$path"
done)
if [[ -n "$forbidden" ]]; then
  printf '%s\n' "$forbidden"
  printf 'tracked JavaScript source/config is forbidden; use strict TypeScript\n' >&2
  exit 1
fi

hash_stream() {
  shasum -a 256 | awk '{print $1}'
}

# Sort so the pin is independent of filesystem readdir order (Darwin ≠ Linux).
selected=$(cd "$ui" && bun ./node_modules/oxlint/bin/oxlint --debug=files . | LC_ALL=C sort)
selected_count=$(printf '%s\n' "$selected" | wc -l | tr -d ' ')
selected_hash=$(printf '%s\n' "$selected" | hash_stream)
if [[ "$selected_count" != 532 || "$selected_hash" != 8989e34a54a9dcb66599245633c2bdc8035c3751ea5c0a987240a202734b9dd2 ]]; then
  printf 'oxlint selected files: count=%s hash=%s (want 532 / 8989e34a…)\n' \
    "$selected_count" "$selected_hash" >&2
  exit 1
fi

config=$(cd "$ui" && bun ./node_modules/oxlint/bin/oxlint --print-config)
config_hash=$(printf '%s\n' "$config" | hash_stream)
if [[ "$config_hash" != f1796585c8362b98be550755de4b4bb27bfb6aba286e0f041ebfbb0e7410cf7e ]]; then
  printf 'oxlint --print-config hash=%s\n' "$config_hash" >&2
  exit 1
fi
# Do not hash raw --showConfig: the files[] order follows readdir (Darwin ≠ Linux).
ts_config=$(cd "$ui" && bun ./node_modules/typescript/bin/tsc --showConfig)
opts_hash=$(jq -cS '.compilerOptions' <<<"$ts_config" | hash_stream)
files_count=$(jq '.files | length' <<<"$ts_config")
files_hash=$(jq -r '.files[]' <<<"$ts_config" | sed 's#^\./##' | LC_ALL=C sort | hash_stream)
if [[ "$opts_hash" != 3885db28b54bf8f8208f90505464e9b313369d7d6332bf61bc975b98054eaae9 ||
  "$files_count" != 533 ||
  "$files_hash" != 73aba17652fa4ef514bc01ea00a352be2bab759d598405e75beb380baffb7355 ]]; then
  printf 'tsc --showConfig: opts=%s files=%s hash=%s\n' \
    "$opts_hash" "$files_count" "$files_hash" >&2
  exit 1
fi
[[ $(jq -r '.compilerOptions.noPropertyAccessFromIndexSignature' <<<"$ts_config") == true ]]
[[ $(jq -r '.compilerOptions.strict' <<<"$ts_config") == true ]]
[[ $(jq -r '.compilerOptions.allowJs' <<<"$ts_config") == false ]]
[[ $(jq -r '.compilerOptions.checkJs' <<<"$ts_config") == false ]]
[[ $(jq -r '.compilerOptions.isolatedModules' <<<"$ts_config") == true ]]
[[ $(jq -r '.compilerOptions.moduleDetection' <<<"$ts_config") == force ]]
[[ $(jq -r '.compilerOptions.erasableSyntaxOnly' <<<"$ts_config") == true ]]

probe="$ui/plan131-negative-probe.tsx"
cycle_a="$ui/plan131-cycle-a.ts"
cycle_b="$ui/plan131-cycle-b.ts"
trap 'rm -f "$probe" "$cycle_a" "$cycle_b"' EXIT
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

expect_compiler_failure() {
  local diagnostic=$1
  local source=$2
  printf '%s\n' "$source" >"$probe"
  output=$(cd "$ui" && bun ./node_modules/typescript/bin/tsc --noEmit --pretty false 2>&1) && {
    printf 'compiler negative fixture unexpectedly passed: %s\n' "$diagnostic" >&2
    exit 1
  }
  rg -F "$diagnostic" <<<"$output" >/dev/null || {
    printf 'compiler negative fixture did not report %s\n' "$diagnostic" >&2
    exit 1
  }
}

expect_compiler_failure 'TS4111' $'declare const record: Record<string, unknown>\nvoid record.value'
expect_compiler_failure 'TS2748' $'declare const enum Mode { Active }\nvoid Mode.Active'
expect_compiler_failure 'TS1294' $'enum RuntimeMode { Active }\nvoid RuntimeMode.Active'

expect_rule_failure() {
  local rule=$1
  local source=$2
  local mode=()
  if [[ "$rule" == typescript/* ]]; then
    mode=(--type-aware)
  fi
  printf '%s\n' "$source" >"$probe"
  output=$(cd "$ui" && bun ./node_modules/oxlint/bin/oxlint "${mode[@]}" -A all -D "$rule" "$probe" 2>&1) && {
    printf 'negative fixture unexpectedly passed: %s\n' "$rule" >&2
    exit 1
  }
  rg -F "(${rule#*/})" <<<"$output" >/dev/null || {
    printf 'negative fixture did not report its owned rule: %s\n' "$rule" >&2
    exit 1
  }
}

expect_rule_failure 'eslint/no-control-regex' 'const control = /[\x00]/'
expect_rule_failure 'import/no-duplicates' $'import { useMemo } from "react"\nimport { useState } from "react"\nvoid useMemo\nvoid useState'
expect_rule_failure 'react/rules-of-hooks' $'import { useState } from "react"\nexport function Probe({ enabled }: { enabled: boolean }) {\n  if (enabled) useState(0)\n  return null\n}'
expect_rule_failure 'react/exhaustive-deps' $'import { useEffect } from "react"\nexport function Probe({ value }: { value: string }) {\n  useEffect(() => console.log(value), [])\n  return null\n}'
expect_rule_failure 'typescript/consistent-type-imports' $'import { CSSProperties } from "react"\nconst style: CSSProperties = {}\nvoid style'
expect_rule_failure 'typescript/no-floating-promises' 'Promise.resolve(1)'
expect_rule_failure 'typescript/no-misused-promises' 'const button = <button onClick={async () => Promise.resolve()} />; void button'
expect_rule_failure 'typescript/no-unsafe-argument' $'declare const value: any\nfunction takesString(input: string) { return input }\ntakesString(value)'
expect_rule_failure 'typescript/no-unsafe-assignment' $'declare const value: any\nconst text: string = value\nvoid text'
expect_rule_failure 'typescript/no-unsafe-call' $'declare const value: any\nvalue()'
expect_rule_failure 'typescript/no-unsafe-member-access' $'declare const value: any\nvoid value.member'
expect_rule_failure 'typescript/no-unsafe-return' 'function text(): string { const value: any = 1; return value }; void text'
expect_rule_failure 'typescript/only-throw-error' 'throw "not an error"'
expect_rule_failure 'typescript/restrict-plus-operands' $'declare const left: number\ndeclare const right: {}\nvoid (left + right)'
expect_rule_failure 'typescript/restrict-template-expressions' $'const value = { field: 1 }\nvoid `${value}`'
expect_rule_failure 'typescript/return-await' 'async function value() { try { return Promise.resolve(1) } catch { return 0 } }; void value'
expect_rule_failure 'typescript/switch-exhaustiveness-check' $'declare const value: "a" | "b"\nswitch (value) { case "a": break }'
expect_rule_failure 'typescript/use-unknown-in-catch-callback-variable' 'Promise.reject(new Error()).catch((error) => String(error))'

printf '%s\n' 'import "./plan131-cycle-b"' >"$cycle_a"
printf '%s\n' 'import "./plan131-cycle-a"' >"$cycle_b"
cycle_output=$(cd "$ui" && bun ./node_modules/oxlint/bin/oxlint -A all -D import/no-cycle "$cycle_a" "$cycle_b" 2>&1) && {
  printf 'negative fixture unexpectedly passed: import/no-cycle\n' >&2
  exit 1
}
rg -F 'import(no-cycle)' <<<"$cycle_output" >/dev/null

printf 'TypeScript/Oxlint contract passed (532 selected files, 19 rule fixtures)\n'
