# Plan 101 dependency and cache baseline

**Research date:** 2026-07-12

## Enforced live baseline

- `cargo audit`, `cargo deny check`, `cargo shear`, the supported
  `cargo hack` powerset, and `bun audit --json` are clean through
  `cargo xtask dependencies --all`.
- Bun's implicit trusted-package list is disabled with
  `trustedDependencies: []`. `unrs-resolver@1.12.2` remains untrusted. Its
  blocked `postinstall` calls `napi-postinstall` to locate or prepare an
  optional native binding; Parallax does not permit its download path. The
  exact lock entry has integrity
  `sha512-dmlRxBJJayXjqTwC+JtF1HhJmgf3ftQ3YejFcZrf4+KKtJv0qDsK1pjqaaVjG7wJ5NJ6UVP1OqRMQ71Z4C3rxQ==`.
  Frozen ignore-scripts install, typecheck, production build, and all 175 UI
  tests pass using the lock-provisioned Darwin binding.
- The live `undici` advisory path was cleared with the exact compatible
  `7.28.0` override. The fresh Bun audit result is `{}`.
- Oxc resolver-backed reachability removed four unused direct dependencies:
  `@tanstack/react-devtools`, `@tanstack/react-router-devtools`,
  `@tanstack/react-router-ssr-query`, and `@tanstack/router-plugin`. TanStack
  Start owns the router plugin transitively. CSS, compiler-type, peer-runtime,
  test-environment, and maintainer-tool exceptions are explicit in
  `dependency-policy.toml`.

## Oxc and TypeScript handoff

The checked-in Rust analyzer uses one compatible family:
`oxc_parser`/`oxc_ast`/`oxc_semantic`/`oxc_span` `0.139.0` and
`oxc_resolver` `11.24.2`. Live registry discovery recorded:

| Owner | Exact candidate | Constraint handed forward |
|---|---:|---|
| Plan 130 | `oxfmt@0.58.0` | Sole authorized Beta formatter; expires at stable |
| Plan 131 | `oxlint@1.73.0` + `oxlint-tsgolint@0.24.0` | Group upgrade; no JS plugins or runtime downloads |
| Plan 131 | `typescript@7.0.2` | Do not adopt until the ESLint/typescript-eslint path is removed atomically |
| Plan 129 | `@testing-library/user-event@14.6.1` | Add only with its first owned tests |
| Plan 132 | `@playwright/test@1.61.1` | Sole direct runner; explicit browser provisioning under Bun |

The tsgolint package embeds TypeScript Go revision `c080da62`, from before the
TypeScript 7 GA revision. Plan 131 must use a GA-or-newer embedded revision or
check in project/diagnostic parity proof. The current blocking compiler API
path is direct `eslint@9.39.4` and `@tanstack/eslint-config@0.4.0`, which owns
`typescript-eslint@8.61.0` and its `typescript >=4.8.4 <6.1.0` peers. Plan 131
must distinguish direct removal from transitive disappearance with `bun why`.

The executable pins, owners, expiry rules, and Playwright predicate live in
`dependency-policy.toml`; `cargo xtask dependencies --ui` fails if the two
pre-stable exceptions broaden or the handoff versions drift.

Registry metadata also records SHA-512 integrity for each wrapper and the
exact Darwin arm64/x64 and Linux GNU arm64/x64 native package set. Oxlint and
Oxfmt expose no install lifecycle; their wrappers select optional native
packages. `oxlint-tsgolint` has the same four supported platform packages and
a Bun-forced JS wrapper. Playwright's predicate locks runner, `playwright`, and
`playwright-core` to `1.61.1`; browser download is a later explicit provisioning
step and never an install lifecycle side effect.

Primary status sources: [Oxc releases](https://github.com/oxc-project/oxc/releases),
[Oxlint releases](https://github.com/oxc-project/oxc/releases),
[TypeScript releases](https://github.com/microsoft/TypeScript/releases), and
[Playwright releases](https://github.com/microsoft/playwright/releases).

## Cold/warm cache measurement

Measurement used `sccache 0.16.0`, a deleted local sccache directory, a fully
clean Cargo target for each run, `CARGO_INCREMENTAL=0`, and
`cargo check -p parallax-xtask --locked` on the same checkout.

| Run | Wall time | Executed cacheable requests | Hits | Result |
|---|---:|---:|---:|---|
| Cold | 11.500 s | 131 | 0 | 0% |
| Immediate warm after `cargo clean` | 4.975 s | 131 | 131 | 100% for the second run |

The cumulative sccache display reports 50% because it contains both samples.
There were no cache read/write errors. This supports retaining the existing
registry, target, and sccache layers; CI now uploads Cargo timings and sccache
statistics so future extraction or backend changes require measured evidence.
