# Plan 098 closure: facades, modules, and nested-read batching

Date: 2026-07-12

Plan 098 is complete. The final workspace has checked crate facades, semantic
crate orientation, responsibility-focused Rust modules, and constant-call
nested issue reads.

## Delivered evidence

- Every workspace crate has a syntax-derived `facade.toml`; server lifecycle
  paths also have compile-pass and representative compile-fail contracts.
- Workspace `unreachable_pub` is enabled and strict clippy is green.
- GreptimeDB, Turso, spool, ingest, evidence, CLI, API services/traces, server
  HTTP assembly, memory test capabilities, and TypeScript policy internals are
  split by owned responsibility.
- Crate roots and all restructured production files meet the structural file
  targets. The pre-existing trace-search function exception moved to its owned
  helper and tightened from 108 to 107 logical lines; it remains shrink-only.
- `Issue.latestEvent` and ranged `Issue.events` share a request-local cohort
  memo and a partitioned storage batch query. The API call-count test proves
  exactly two storage reads at both one item and the maximum page size.
- Shared adapter conformance now reads a persisted fingerprint plus a missing
  fingerprint through the batch contract. It passes in memory and against
  managed GreptimeDB, including restart persistence.
- All crate READMEs name owned concerns, source modules, reviewed roots, public
  surface, and narrow verification. Policy checks their Cargo metadata, tier,
  dependencies, source links, facade roots, and gate commands.
- The plain-Markdown [Rust workspace map](../architecture/rust-workspace-map.md)
  owns crate/tier navigation; `PROJECT_STRUCTURE.md` is limited to stable
  top-level ownership.

## Compatibility evidence

- The GraphQL SDL snapshot is unchanged.
- Bundle JSON/Markdown and CLI output characterization tests are unchanged and
  green.
- Greptime native raw-signal table policy and SQL golden tests are green.
- No product storage mode, TLS backend, JS runtime, or package manager changed.

## Verification

| Gate | Result |
| --- | --- |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | PASS |
| `cargo nextest run --workspace --all-features` | PASS — 240 run, 240 passed, 6 ignored |
| `cargo nextest run -p parallax-server --all-features --run-ignored all --no-capture` | PASS — 33/33, including six real GreptimeDB gates |
| Targeted real `greptime_conformance_scenarios` after adding the batch assertion | PASS before and after managed-engine restart |
| `cargo check --release --workspace --all-features` | PASS |
| `cargo xtask dependencies --all` | PASS — audit, deny, shear, feature powerset, TLS trees, Bun audit/trust |
| `cargo xtask policy` | PASS |
| `cargo xtask facade check` | PASS |

## Focused implementation history

- `878103c` seals the server lifecycle facade.
- `938c6ec` batches nested issue events and proves constant storage calls.
- `5d492e8`, `34e481f`, `192356e`, `22fb6b4`, and `dc06afd` split the concrete
  adapters and pure ingest/evidence owners.
- `84ffa6d`, `4fd6031`, `3aee486`, and `0f6bec3` split CLI, API, and server
  orchestration.
- `c70d968` and `d05e0a1` split test-support and repository-policy hotspots.
- `09ebb96` adds semantic crate documentation and the workspace map.
- `f371116` adds memory and real-Greptime batch conformance.
