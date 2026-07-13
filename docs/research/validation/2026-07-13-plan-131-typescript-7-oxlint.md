# Plan 131 TypeScript 7 and Oxlint validation

Validation date: 2026-07-13  
Implementation range: `a3bb168..91cbbf5`  
Final validation candidate: `91cbbf5`

## Final compatibility unit

The UI has one compiler and one linter family:

| Component | Exact version | Authority |
| --- | --- | --- |
| TypeScript | 7.0.2 | independent `tsc --noEmit` compiler gate |
| Oxlint | 1.73.0 | stable native lint gate |
| `oxlint-tsgolint` | 0.24.0 | exact Plan 101 exception for typed rules |

The final Bun lock has no direct or transitive ESLint, TanStack ESLint config,
typescript-eslint, `@typescript-eslint/*`, ESLint plugin/parser/config, migration
utility, TypeScript 6 alias, or `unrs-resolver` lifecycle-script path. Frozen
installation reports zero untrusted dependencies with scripts. The exact
pre-stable entry remains owned by Plan 131 in `dependency-policy.toml`, expires
at the first stable `oxlint-tsgolint` release, and authorizes no JavaScript
plugin, nursery rule, transformer, minifier, or experimental type-check
authority.

## Compiler and project parity

The stored pre-cutover characterization is
[`2026-07-13-plan-131-ts6-eslint-baseline.md`](2026-07-13-plan-131-ts6-eslint-baseline.md).
TypeScript 6.0.3 and the isolated TypeScript 7.0.2 candidate both produced zero
application diagnostics and selected the same 152 Parallax source/config/test
classes. The final configuration removes only the deleted ESLint config from
the compiler include set; it preserves ES2022, ESNext, bundler resolution,
explicit Vite types, JSX, no-emit, and every strictness option. Final
`tsc --showConfig` SHA-256 is
`f6b94e460cb728ea095b0a7138c731f3ddf9f89a8faa10d0d18480c8933b8083`.

`scripts/ci/test-typescript-oxlint-contract.sh` derives the native compiler
package from Bun's `process.platform` and `process.arch`, requires its
lock-provisioned executable, and rejects a missing or wrong platform binding.
The current Linux arm64 run selected
`@typescript/typescript-linux-arm64@7.0.2/lib/tsc`. The wrapper is run by Bun
from its exact lock-local path and execs the platform-native compiler; no
repository package uses the unavailable TypeScript 7 programmatic API.

## Lint selection and typed parity

The final native and type-aware commands are separate CI steps and both fail on
warnings, unused directives, unmatched paths, parse failures, or resolution
failures. Their frozen observations are:

| Observation | Value |
| --- | --- |
| Selected files | 151, nonzero |
| Selected-file SHA-256 | `ebb965980822201e59b37286bdef0e3933901795ad6a77e5fd1c5c6d22ed1bbe` |
| Effective Oxlint config SHA-256 | `ebcc47b1b91ce91f0e19cc0f260992e656cbddd6a0a1d9d1ab73aa9b837fc04d` |
| Native full-project duration | approximately 0.5 seconds |
| Type-aware full-project duration | 1.30 seconds characterized; 1–2 seconds repeated |
| Type-aware sampled peak process-tree RSS | 927,772 KiB |

The 0.24.0 companion embeds pre-GA TypeScript Go revision `c080da62`, so the
package version was not accepted as GA proof. The checked compatibility script
instead runs a GA TypeScript 7 project/config/selection oracle and 19 exact
rule-negative fixtures. Every deny rule in `.oxlintrc.jsonc` owns a failing
fixture: control regex, duplicate imports, cycles including type edges, both
React hook rules, type-only imports, floating/misused promises, five unsafe
value boundaries, thrown-error discipline, plus operands, template
interpolation, return-await, switch exhaustiveness, and catch-callback unknown.
The full project is the positive fixture. Adding a rule, changing selection,
changing effective config, or changing the exact compiler/linter unit fails the
stored fingerprints or fixture inventory.

A live Linux process-tree sample resolved the apparent
`node .../.bin/tsgolint headless` label to
`/home/agent/.local/share/mise/installs/bun/1.3.14/bin/bun` through
`/proc/<pid>/exe`; Bun then launched the reviewed native
`@oxlint-tsgolint/linux-arm64/tsgolint` binary. No Node executable owned the
process. Oxc `--type-check` remains explicitly false and cannot satisfy the
independent compiler negative fixture.

The only generated-route exception is `src/router.tsx`: TanStack's generated
route tree necessarily registers the root route back through the router type.
The authoritative Oxc Rust architecture graph remains required across the
whole UI, while native `import/no-cycle` stays unlimited-depth with type edges
included and unsafe dynamic cycles disabled everywhere else. The generated
`routeTree.gen.ts` retains its generator-owned `eslint-disable` banner, but is
ignored and `respectEslintDisableDirectives` is false; no ESLint process reads
it.

## Final gates

- `cargo nextest run --workspace --all-targets --profile ci --no-tests=fail`:
  253 passed, 6 intentionally skipped.
- `cargo test --workspace --doc --locked`: all doctests passed.
- `cargo clippy --workspace --all-targets --locked -- -D warnings`: passed.
- `cargo xtask policy`: passed with lowered, never raised, TypeScript ratchets.
- `cargo xtask dependencies --all`: cargo audit/deny/shear/hack/tree plus
  frozen Bun install, audit, license/integrity, unused dependency, and lifecycle
  checks passed.
- `scripts/ci/test-bun-contract.sh`: 11 scripts passed Bun ownership fixtures.
- `scripts/ci/test-typescript-oxlint-contract.sh`: 151 files and 19 rule
  fixtures passed.
- `cargo xtask ui`: formatting, TypeScript 7, both lint lanes, 41 Vitest files
  and 175 tests, client/SSR Vite production build, and route-tree drift passed.
- Production preview with a Bun-only exact-shape GraphQL smoke endpoint returned
  HTTP 200 and a 52,611-byte Parallax document containing the product title and
  built asset references.

The first backend-less preview probe returned the expected 500 because
isomorphic overview loaders target the product GraphQL endpoint at port 4000.
It was not treated as passing evidence; the controlled Bun GraphQL smoke above
proved the production output with its required dependency present.

No research prompt changed: Plan 131 executed an already-current implementation
contract and did not alter product research direction or evaluation criteria.
