#!/usr/bin/env bash
set -euo pipefail

root=$(git rev-parse --show-toplevel)
ui="$root/ui"
package="$ui/package.json"
lock="$ui/bun.lock"

[[ $(jq -r '.devDependencies.typescript' "$package") == 7.0.2 ]]
[[ $(jq -r '.devDependencies.oxlint' "$package") == 1.73.0 ]]
[[ $(jq -r '.devDependencies["oxlint-tsgolint"]' "$package") == 0.24.0 ]]
compiler=$(cd "$ui" && bun -e 'import getExePath from "./node_modules/typescript/lib/getExePath.js"; console.log(getExePath())')
platform=$(cd "$ui" && bun -e 'console.log(`${process.platform}-${process.arch}`)')
[[ "$compiler" == *"/@typescript/typescript-$platform/lib/tsc" && -x "$compiler" ]]
[[ $(jq -r '.options.typeCheck' "$ui/.oxlintrc.jsonc") == false ]]
[[ $(jq -r 'has("jsPlugins")' "$ui/.oxlintrc.jsonc") == false ]]
[[ $(jq -r '.categories.nursery // "absent"' "$ui/.oxlintrc.jsonc") == absent ]]

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
[[ $(cd "$ui" && bun ./node_modules/typescript/bin/tsc --showConfig | hash_stream) == 7a27bbb55ed1ef1a6dd1c9d0b70ccd851f3fa78f18d31e284eff6520b412407e ]]

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

expect_rule_failure() {
  local rule=$1
  local source=$2
  printf '%s\n' "$source" >"$probe"
  output=$(cd "$ui" && bun ./node_modules/oxlint/bin/oxlint --type-aware -A all -D "$rule" "$probe" 2>&1) && {
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

printf 'TypeScript/Oxlint contract passed (151 selected files, 19 rule fixtures)\n'
