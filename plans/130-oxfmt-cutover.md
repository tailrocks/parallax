# Plan 130: Replace Prettier with Oxfmt

> **Executor instructions**: Consume plan 101's exact operator-approved Oxfmt
> Beta exception; do not call the tool stable or broaden the exception. Recheck
> official status and capabilities first. Preserve the required current
> formatter until one differential migration passes; finish with Oxfmt alone,
> never a permanent dual stack.

## Status

- **Priority**: P1
- **Effort**: M
- **Risk**: MEDIUM
- **Depends on**: 094, 101
- **Category**: Oxc / TypeScript formatting / tooling
- **Planned at**: `a1d8bf82`, 2026-07-12
- **Status**: TODO

## Why

Oxfmt is officially Beta, but its current guide recommends it as the dedicated
formatter, it passes 100% of Prettier's JavaScript/TypeScript conformance suite,
and its built-in Tailwind sorter replaces Parallax's only Prettier plugin. It
supports the current JavaScript/TypeScript surface and broader frontend formats,
parses TypeScript independently of the compiler, and works unchanged with plan
131's TypeScript 7 toolchain. The 2026-07-12 Oxc-only operator direction makes
Oxfmt non-negotiable before program closure and authorizes its exact narrow Beta
exception through plan 101. This does not relabel `oxfmt@0.58.0` stable or grant
a general pre-stable-tool exception.

Sources: [current Oxfmt recommendation](https://oxc.rs/docs/guide/usage/formatter),
[unsupported features](https://oxc.rs/docs/guide/usage/formatter/unsupported-features),
and the [Oxfmt Beta announcement](https://oxc.rs/blog/2026-02-24-oxfmt-beta).

## Readiness Check

Before any dependency/config change, record the official Oxc status, latest
release, release notes, supported platforms, package integrity, language matrix,
unsupported features, and plan 101 policy result. If Oxfmt is still Beta, the
durable allowlist must contain exactly its reviewed release/platform unit and
the separately justified `oxlint-tsgolint` entry. If Oxfmt becomes stable,
remove its exception before cutover. STOP on a missing/broadened exception or a
regression in required Tailwind, TS/TSX, generated-owner, or platform behavior.

## Scope

- Exact lock-local Oxfmt executable/platform packages and Bun-only invocation.
- `.oxfmtrc.jsonc` preserving current format/Tailwind behavior.
- Differential file/output/idempotence and generated-owner migration.
- Required local/CI diagnostics and removal of direct Prettier ownership.

Out of scope:

- Import/package sorting bundled with the formatter cutover.
- Product refactoring or generated route edits by hand.
- Node, another package manager, or a second permanent formatter.
- Direct Oxc transform/minify/build changes.

## Steps

### Step 1: Freeze exact current behavior

Consume plan 094's exact current selected-file manifest and config behavior.
Record LF, `semi:false`, `singleQuote:false`, tab width 2,
`trailingComma:"es5"`, print width 80, Tailwind stylesheet
`src/styles.css`, functions `cn`/`cva`, `routeTree.gen.ts` exclusion, shadcn
policy, whitespace-semantic fixtures, and current two-run idempotence.

### Step 2: Prove the package/runtime contract

Use plan 101 policy to pin exact Oxfmt and platform packages. Oxc wrappers carry
a Node shebang, so scripts invoke the exact lock-local wrapper through Bun's
`--bun` override with installation disabled. Process-tree fixtures fail Node,
runtime download, missing binding, unsupported platform, foreign lockfile, or
unreviewed lifecycle execution. Prove clean installs on supported macOS/Linux
architectures.

### Step 3: Build the parity configuration

Run the official `--migrate prettier` command through the forced-Bun locked
wrapper and inspect, rather than trust, its result. Create `.oxfmtrc.jsonc` with
the current contract and built-in:

```text
sortTailwindcss.stylesheet = "src/styles.css"
sortTailwindcss.functions = ["cn", "cva"]
sortImports = false
sortPackageJson = false
```

Validate exact option spellings against the pinned official config reference.
Inventory every Oxfmt-supported repository file class and record an explicit
disposition: adopt, generator-owned, whitespace-semantic fixture, vendored, or
owned by a stronger ecosystem formatter. The first cutover covers the current
JS/TS family only; then add eligible frontend JSON/JSONC/CSS/Markdown/GraphQL
classes in separate mechanical commits with per-class differential fixtures.
Do not let default discovery rewrite unrelated research, Cargo TOML, YAML
workflows, generated data, or fixtures before their disposition is approved.

### Step 4: Run differential and generator proof

Compare Prettier and Oxfmt on the exact current file manifest. Fixture Tailwind
classes in `className`, `cn`, and `cva`; line endings; long JSX/TS expressions;
comments; generated route output; shadcn regeneration; and ignored semantic
fixtures. Run Oxfmt twice from clean macOS/Linux checkouts. Any intentional delta
lands as one reviewable non-functional baseline, separate from config or product
work.

Keep `routeTree.gen.ts` excluded from manual formatting and generator-drift
checked. Apply the plan 094 shadcn decision exactly. Repository-owned generated
output remains format-clean; third-party generators may use only narrow,
fixture-proven exclusions.

### Step 5: Cut over atomically

Make write/check/list-different scripts and xtask/CI invoke only the exact Oxfmt
path. Require the check in the stable aggregate and fail zero/unexpected file
selection. Delete direct Prettier, `prettier-plugin-tailwindcss`, `.prettierrc`,
`.prettierignore`, scripts, cache keys, and config references in the same
cutover. A framework-internal transitive Prettier may remain but Parallax never
invokes it as a formatter.

Do not enable import or package sorting during this cutover. Each may receive a
later isolated plan only if its mechanical benefit and ownership are proven.

## Test Plan

- Exact narrow-exception, stable-expiry, and no-broader-pre-stable fixtures.
- Exact locked-wrapper, `--bun`, no-install, no-Node process ancestry tests.
- Config migration and unsupported-option report.
- Exact file-manifest/no-file/unexpected-file tests.
- Prettier/Oxfmt differential goldens plus two-run idempotence on macOS/Linux.
- Tailwind `className`/`cn`/`cva`, line ending, route generator, shadcn, and
  whitespace-semantic fixtures.
- `oxfmt --check`/list-different failure and stale-Prettier search fixtures.

## Done Criteria

- [ ] The exact Oxfmt exception is durable and executable while Beta, expires at
  stable, and does not authorize any other pre-stable Oxc surface.
- [ ] Oxfmt is exact, lock-local, Bun-forced, platform-complete, and spawns no
  Node or installer/download runtime.
- [ ] `.oxfmtrc.jsonc` preserves the current format/Tailwind contract and
  explicitly controls sort defaults; every supported repository file class has
  an adopted or fixture-proven exclusion/other-owner disposition.
- [ ] Differential and two-run macOS/Linux evidence is deterministic.
- [ ] Generated route, shadcn, and semantic-fixture ownership is exact and tested.
- [ ] Oxfmt is the only direct/manual formatter and its required gate cannot pass
  with zero or unexpected files.
- [ ] Direct Prettier/plugin/config/script/cache ownership is absent.

## STOP Conditions

- Plan 101's exact Oxfmt exception is absent, broader than intended, or cannot be
  retired when Oxfmt becomes stable.
- The wrapper needs Node, a foreign manager, lifecycle download, or an unsupported
  native binding.
- Output/file selection differs across clean runs or platforms without an
  understood isolated baseline.
- Tailwind, generator, shadcn, line-ending, or whitespace-semantic behavior
  cannot be preserved.
- Cutover would leave two required formatters or combine product changes.

## Remove When

Delete this plan and index row when Oxfmt is the sole required formatter, all
parity/runtime/generator/policy gates pass, and direct Prettier ownership is
removed.
