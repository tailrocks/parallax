#!/usr/bin/env bash
set -euo pipefail

root=$(git rev-parse --show-toplevel)
ui="$root/ui"
package="$ui/package.json"
config="$ui/.oxfmtrc.jsonc"

[[ $(jq -r '.devDependencies.oxfmt' "$package") == 0.58.0 ]]
[[ $(jq -r '.devDependencies | has("prettier")' "$package") == false ]]
[[ $(jq -r '.devDependencies | has("prettier-plugin-tailwindcss")' "$package") == false ]]
[[ ! -e "$ui/.prettierrc" && ! -e "$ui/.prettierignore" ]]

[[ $(jq -r '.endOfLine' "$config") == lf ]]
[[ $(jq -r '.semi' "$config") == false ]]
[[ $(jq -r '.singleQuote' "$config") == false ]]
[[ $(jq -r '.tabWidth' "$config") == 2 ]]
[[ $(jq -r '.trailingComma' "$config") == es5 ]]
[[ $(jq -r '.printWidth' "$config") == 80 ]]
[[ $(jq -r '.sortImports' "$config") == false ]]
[[ $(jq -r '.sortPackageJson' "$config") == false ]]
[[ $(jq -r '.sortTailwindcss.stylesheet' "$config") == src/styles.css ]]
[[ $(jq -c '.sortTailwindcss.functions' "$config") == '["cn","cva"]' ]]
[[ $(jq -c '.ignorePatterns' "$config") == '["src/routeTree.gen.ts"]' ]]
[[ $(shasum -a 256 "$config" | awk '{print $1}') == 6804a8af65081087d786f925b09c6656dc4a14c645b73cddbb90a98a546e0d89 ]]

files=$(git -C "$root" ls-files 'ui/*.ts' 'ui/*.tsx' 'ui/*.js' 'ui/*.jsx' 'ui/**/*.ts' 'ui/**/*.tsx' 'ui/**/*.js' 'ui/**/*.jsx' | sed 's#^ui/##' | rg -v '^src/routeTree\.gen\.ts$' | sort)
[[ $(printf '%s\n' "$files" | wc -l | tr -d ' ') == 151 ]]
[[ $(printf '%s\n' "$files" | shasum -a 256 | awk '{print $1}') == ebb965980822201e59b37286bdef0e3933901795ad6a77e5fd1c5c6d22ed1bbe ]]

platform=$(cd "$ui" && bun -e 'console.log(process.platform + "-" + process.arch)')
case "$platform" in
  darwin-*) binding_pattern="$ui/node_modules/@oxfmt/binding-$platform/oxfmt.*.node" ;;
  linux-*) binding_pattern="$ui/node_modules/@oxfmt/binding-$platform-*/oxfmt.*.node" ;;
  win32-*) binding_pattern="$ui/node_modules/@oxfmt/binding-$platform-msvc/oxfmt.*.node" ;;
  *) printf 'unsupported Oxfmt contract platform: %s\n' "$platform" >&2; exit 1 ;;
esac
compgen -G "$binding_pattern" >/dev/null || {
  printf 'missing Oxfmt native binding for %s\n' "$platform" >&2
  exit 1
}

tailwind_input=$'const node = <div className="p-4 flex bg-red-500" />\nconst joined = cn("p-4 flex bg-red-500")\nconst variant = cva("p-4 flex bg-red-500")'
tailwind_expected=$'const node = <div className="flex bg-red-500 p-4" />\nconst joined = cn("flex bg-red-500 p-4")\nconst variant = cva("flex bg-red-500 p-4")'
tailwind_actual=$(printf '%s\n' "$tailwind_input" | (cd "$ui" && bun ./node_modules/oxfmt/bin/oxfmt --stdin-filepath plan130-tailwind.tsx))
[[ "$tailwind_actual" == "$tailwind_expected" ]]

probe="$ui/plan130-format-probe.ts"
trap 'rm -f "$probe"' EXIT
printf '%s\n' 'const probe={value:1}' >"$probe"
if (cd "$ui" && bun ./node_modules/oxfmt/bin/oxfmt --check "$probe" >/dev/null 2>&1); then
  printf 'Oxfmt check negative fixture unexpectedly passed\n' >&2
  exit 1
fi
different=$(cd "$ui" && bun ./node_modules/oxfmt/bin/oxfmt --list-different "$probe" || true)
rg -Fx 'plan130-format-probe.ts' <<<"$different" >/dev/null
if (cd "$ui" && bun ./node_modules/oxfmt/bin/oxfmt --check 'plan130-no-files-*.ts' >/dev/null 2>&1); then
  printf 'Oxfmt zero-file fixture unexpectedly passed\n' >&2
  exit 1
fi
if (cd "$ui" && bun ./node_modules/oxfmt/bin/oxfmt --check src/routeTree.gen.ts >/dev/null 2>&1); then
  printf 'generator-owned route unexpectedly entered formatter selection\n' >&2
  exit 1
fi
rm -f "$probe"

if [[ -d /proc ]]; then
  caller_directory=$PWD
  cd "$ui"
  bun run check >/dev/null &
  check_pid=$!
  cd "$caller_directory"
  runtime=''
  while kill -0 "$check_pid" 2>/dev/null; do
    runtime=$(readlink "/proc/$check_pid/exe" 2>/dev/null || true)
    [[ -z "$runtime" ]] || break
  done
  wait "$check_pid"
  [[ "$runtime" == */bun ]] || {
    printf 'Oxfmt check was not launched by Bun: %s\n' "$runtime" >&2
    exit 1
  }
fi

check_output=$(cd "$ui" && bun run check)
rg -F '151 files' <<<"$check_output" >/dev/null
[[ -z $(cd "$ui" && bun run --silent format:list) ]]

printf 'Oxfmt contract passed (151 files, %s)\n' "$platform"
