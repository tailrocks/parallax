# Plan 131 TypeScript 6 and ESLint baseline

Capture date: 2026-07-13  
Baseline commit: `14c3219`

## Resolved compatibility unit

Registry resolution under Bun 1.3.14 confirms the plan snapshot remains current:
TypeScript 7.0.2, Oxlint 1.73.0, `oxlint-tsgolint` 0.24.0, and isolated
`@oxlint/migrate` 1.73.0. Context7 official-source checks confirm `tsc
--noEmit` retains semantic diagnostics, TypeScript 7 removes the named legacy
options, and Oxlint exposes JSON/JSONC config, `--type-aware`,
`--report-unused-disable-directives`, `--print-config`, `--debug=files`, and
native plugins without JavaScript plugin packages.

## Compiler characterization

| Measurement | TypeScript 6.0.3 | TypeScript 7.0.2 isolated candidate |
| --- | ---: | ---: |
| `--noEmit` diagnostics | 0 | 0 |
| All files (`--listFilesOnly`) | 1,695 | 1,693 |
| Project source/config/generated classes | 152 | 152 |
| Wall time | 5.87 s | 1.04 s |
| `--showConfig` SHA-256 | `91170243001c59844cab5d3b3a56783994d28ddc1ed3f12e1a3945bb76f7afd9` | `67d71ea6b138702852b728d06c5e534e6162a548aeb05a7f1257e820ebac90cc` |

The total-file delta is explained by the native TS7 platform package and
dependency declaration selection; every included Parallax source, test,
generated route, runtime probe, ESLint config, Prettier config, and Vite config
class remains selected. The existing explicit ES2022/ESNext/bundler, JSX,
types, strictness, no-emit, and safety options are accepted unchanged. Neither
compiler reports an application diagnostic.

## Last ESLint baseline

ESLint 9 through `@tanstack/eslint-config` selected 152 files with 0 errors, 0
warnings, and 5 suppressed diagnostics. Effective-config hashes were:

- `src/main.tsx`: `f389f297728ae2ccab6991bfca766d2df312b52010a550b0465b081e1f80cbd8`
- `src/components/ui/button.tsx`: `b58da2e783e85615403d2fcbbc9685c2ad57f9336bba4df8ba57a2941aaef27f`

The isolated matched migrator produced 64 native/type-aware mappings. It
reported five skipped rules: nursery `no-unnecessary-condition`, JS-plugin-only
`@stylistic/spaced-comment`, unimplemented `naming-convention` and
`node/prefer-node-protocol`, and strict-mode-superseded `no-octal`. No JS plugin
or nursery rule is authorized by this migration.

## Peer and API inventory

`bun why typescript` shows one incompatible branch only:
`@tanstack/eslint-config@0.4.0 → typescript-eslint@8.61.0 →
@typescript-eslint/*`, whose peers cap TypeScript below 6.1. Vite, Vitest,
TanStack Start/Router, React, shadcn, and repository generators do not require
the compiler API. A repository source/config/script scan found no import,
require, `typescript.sys`, or `createProgram` use. Removing the TanStack ESLint
branch before adding TypeScript 7 therefore closes the only known API/peer
conflict.

This is the last authorized execution of repository ESLint. Subsequent parity
work uses this stored inventory and never reloads ESLint after the atomic
compiler/linter cutover.
