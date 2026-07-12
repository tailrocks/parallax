# Plan 095 quality control plane validation

Date: 2026-07-12

Plan 095 established one Rust-native local/CI quality control plane. The
implementation is rooted in `parallax-xtask`, with one typed `ratchet.toml`,
Cargo-metadata architecture policy, Oxc-backed TypeScript policy, syntax-aware
Rust and TypeScript health providers, product-law checks, facade manifests,
semantic crate documentation, and equivalent human/JSON/GitHub diagnostics.

## Closure evidence

- `mise exec -- cargo xtask ci --full` passed on `main` at `a35c4a9`.
  - strict workspace formatting and Clippy passed with zero warnings;
  - Bun install, formatting, typecheck, lint, 41 Vitest files / 175 tests, and
    client plus SSR production builds passed;
  - cargo-nextest ran 224 tests across 26 binaries: 224 passed and 6 skipped;
  - workspace doctests and the RustSec audit passed.
- `mise exec -- cargo test -p parallax-xtask` passed 29 policy/fixture tests.
- `mise exec -- cargo xtask policy --output json` returned `[]`.
- `mise exec -- cargo xtask facade check` passed.
- `mise exec -- cargo audit -q` passed independently after the full gate.
- Hosted [CI run 29202168820](https://github.com/tailrocks/parallax/actions/runs/29202168820)
  passed for `a35c4a9`, including the required `policy` job and aggregate
  `ci-required` job.

The initial graph and exact ratchets are green on `main`. No product crate
depends on xtask, and both local policy commands and the hosted policy job call
the same Rust implementation. The fixture corpus includes Cargo edge kinds,
diagnostic equivalence, facade failures, product laws, ratchet failure modes,
and TypeScript alias, package-export, dynamic/type-only/reexport, cycle,
layer, test, and server/client cases.

The production chunk-size warning remains owned by Plan 148 and did not affect
this plan's acceptance criteria.
