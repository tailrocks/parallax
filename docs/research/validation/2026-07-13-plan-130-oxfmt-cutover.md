# Plan 130 Oxfmt cutover validation

Validation date: 2026-07-13  
Implementation range: `ec5f61f..f0ec0e2`  
Final validation candidate: `f0ec0e2`

## Reviewed compatibility unit

Oxfmt remains officially Beta. Parallax therefore consumes the operator-approved
Plan 101 exception without describing the formatter as stable or broadening the
exception to another pre-stable Oxc surface. The current registry release is
exact `oxfmt@0.58.0`, with integrity
`sha512-8feG/7NVEHDVwc1OUpP6Pks+TnaDFUw2jLLFIMi5bcmmwxAX2wBQvjSzj62RRTYBf2Op1Wt8xbkmagmPTR5ETg==`.
The lock-local package and reviewed native platform package form the executable
unit. Frozen Bun installation runs no untrusted lifecycle script.

The official usage, migration, configuration, and unsupported-feature guides
were rechecked before migration. They document the standalone JSON/JSONC
configuration, `--migrate prettier`, `--check`, `--list-different`, built-in
Tailwind sorting, and broad frontend language support. Import sorting remains
off, while the package-JSON sorter is explicitly disabled rather than silently
accepting its default.

## Differential migration and ownership

The official migrator was run in an isolated graph through Bun and its result
was inspected against the frozen Plan 094 contract. The live
`.oxfmtrc.jsonc` preserves LF endings, no semicolons, double quotes, two-space
indentation, ES5 trailing commas, width 80, `src/styles.css`, and the `cn` and
`cva` Tailwind functions. It excludes generator-owned `routeTree.gen.ts` and
keeps both import and package sorting out of this cutover.

Oxfmt selects exactly 151 repository-owned JavaScript/TypeScript-family files.
The selected manifest SHA-256 is
`ebb965980822201e59b37286bdef0e3933901795ad6a77e5fd1c5c6d22ed1bbe`; the
configuration SHA-256 is
`6804a8af65081087d786f925b09c6656dc4a14c645b73cddbb90a98a546e0d89`.
Eligible broader JSON/JSONC/CSS/Markdown/GraphQL classes remain intentionally
outside this atomic migration: package manifests and lock data have stronger
package-manager ownership, generated files retain generator ownership, styles
retain CSS/Tailwind build ownership, and research/contract documents are not
silently reformatted. Rust, TOML, YAML, generated data, vendored content, and
whitespace-semantic fixtures likewise retain their existing owner or exclusion.

The formatter's isolated mechanical baseline affected ten files and lowered all
affected structural ratchets; no ceiling was raised. Two consecutive writes
produced the same source-diff SHA-256,
`835cf28bf87e104d920f6279c105f31f429d5ba2f70dfbc6d5a5286cc61318d4`,
and the second run was clean. The Tailwind golden is:

```tsx
const node = <div className="flex bg-red-500 p-4" />
const joined = cn("flex bg-red-500 p-4")
const variant = cva("flex bg-red-500 p-4")
```

Direct Prettier, `prettier-plugin-tailwindcss`, `.prettierrc`,
`.prettierignore`, scripts, and configuration ownership are absent. The only
remaining lock node is an internal `@tanstack/router-generator` dependency;
Parallax never invokes it as a formatter.

## Executable contracts

`scripts/ci/test-oxfmt-contract.sh` requires the exact lock-local wrapper under
Bun's `--bun` override, derives and verifies the native binding, and follows the
mise-shim process tree to the Bun executable. It rejects Node ownership,
installer/download behavior, foreign lockfiles, wrong or missing bindings,
different or zero file selection, inclusion of the generated route tree, stale
Prettier ownership, and Tailwind/config drift. The generated route, shadcn, line
ending, and semantic-fixture ownership remains covered by the formatter and UI
contract suites.

The required `ui-formatter-platform` CI matrix performs a frozen Bun install,
runs the full Oxfmt contract, formats twice, and rejects any checkout diff on
both `ubuntu-latest` and `macos-15`. Its cache key includes platform, lock,
configuration, contract, and selected sources. The stable `ci-required`
aggregate cannot pass without the matrix.

## Final gates

- `scripts/ci/test-oxfmt-contract.sh`: passed with 151 selected files, exact
  config/manifest fingerprints, Tailwind goldens, negative fixtures, native
  binding, and Bun process ownership.
- `scripts/ci/test-typescript-oxlint-contract.sh`: passed with 151 files and 19
  lint-rule fixtures after the mechanical baseline.
- `cargo clippy --workspace --all-targets --locked -- -D warnings`: passed on
  Rust 1.97 after replacing newly diagnosed single-element test loops.
- `cargo xtask policy`: passed with lowered, never raised, structural ratchets.
- `cargo xtask ui`: formatting, TypeScript 7, both lint lanes, 41 Vitest files
  and 175 tests, client/SSR production build, and route-tree drift passed.
- GitHub Actions run
  [29221668189](https://github.com/tailrocks/parallax/actions/runs/29221668189)
  passed on the exact candidate after retrying one transient GitHub cache-service
  DNS failure. Both `ui-formatter-platform (ubuntu-latest)` and
  `ui-formatter-platform (macos-15)` passed, every other required job passed,
  and the stable `ci-required` aggregate concluded success. The preceding push
  run [29221540495](https://github.com/tailrocks/parallax/actions/runs/29221540495)
  also passed all path-selected jobs, including the embed job affected by that
  transient dispatch failure.

No research prompt changed: Plan 130 executed the already-current Oxc
implementation contract and did not change research direction, evaluation
criteria, or product decisions.
