# Plan 127 Rust test topology and parity

Date: 2026-07-12

Baseline: `0af3078` (immediately before extraction). Extraction commit:
`106d962`.

## Inline-body inventory

The syntax inventory found 43 handwritten inline `#[cfg(test)] mod tests`
bodies. Locations and span lengths below come from `syn` spans over the
baseline tree, not text matching.

| Production owner | Baseline line | Module lines | External child |
|---|---:|---:|---|
| `crates/parallax-api/src/lib.rs` | 386 | 26 | `crates/parallax-api/src/tests.rs` |
| `crates/parallax-api/src/resolvers/fields.rs` | 245 | 252 | `crates/parallax-api/src/resolvers/fields/tests.rs` |
| `crates/parallax-api/src/resolvers/investigations.rs` | 180 | 215 | `crates/parallax-api/src/resolvers/investigations/tests.rs` |
| `crates/parallax-api/src/resolvers/logs.rs` | 215 | 144 | `crates/parallax-api/src/resolvers/logs/tests.rs` |
| `crates/parallax-api/src/resolvers/metrics.rs` | 257 | 232 | `crates/parallax-api/src/resolvers/metrics/tests.rs` |
| `crates/parallax-api/src/resolvers/runs.rs` | 272 | 145 | `crates/parallax-api/src/resolvers/runs/tests.rs` |
| `crates/parallax-api/src/resolvers/services.rs` | 605 | 297 | `crates/parallax-api/src/resolvers/services/tests.rs` |
| `crates/parallax-api/src/resolvers/sql.rs` | 75 | 63 | `crates/parallax-api/src/resolvers/sql/tests.rs` |
| `crates/parallax-api/src/resolvers/story.rs` | 163 | 209 | `crates/parallax-api/src/resolvers/story/tests.rs` |
| `crates/parallax-api/src/resolvers/traces.rs` | 582 | 400 | `crates/parallax-api/src/resolvers/traces/tests.rs` |
| `crates/parallax-cli/src/commands.rs` | 1100 | 211 | `crates/parallax-cli/src/commands/tests.rs` |
| `crates/parallax-cli/src/doctor.rs` | 304 | 71 | `crates/parallax-cli/src/doctor/tests.rs` |
| `crates/parallax-core/src/agent_session.rs` | 200 | 100 | `crates/parallax-core/src/agent_session/tests.rs` |
| `crates/parallax-core/src/bundle.rs` | 994 | 377 | `crates/parallax-core/src/bundle/tests.rs` |
| `crates/parallax-core/src/derive.rs` | 194 | 139 | `crates/parallax-core/src/derive/tests.rs` |
| `crates/parallax-core/src/fingerprint.rs` | 131 | 103 | `crates/parallax-core/src/fingerprint/tests.rs` |
| `crates/parallax-core/src/gaps.rs` | 115 | 94 | `crates/parallax-core/src/gaps/tests.rs` |
| `crates/parallax-core/src/normalize.rs` | 459 | 173 | `crates/parallax-core/src/normalize/tests.rs` |
| `crates/parallax-core/src/semconv.rs` | 5 | 21 | `crates/parallax-core/src/semconv/tests.rs` |
| `crates/parallax-core/src/span_events.rs` | 208 | 144 | `crates/parallax-core/src/span_events/tests.rs` |
| `crates/parallax-core/src/story.rs` | 305 | 124 | `crates/parallax-core/src/story/tests.rs` |
| `crates/parallax-core/src/trace_analysis.rs` | 406 | 329 | `crates/parallax-core/src/trace_analysis/tests.rs` |
| `crates/parallax-server/src/config.rs` | 162 | 25 | `crates/parallax-server/src/config/tests.rs` |
| `crates/parallax-server/src/greptime_supervisor.rs` | 471 | 34 | `crates/parallax-server/src/greptime_supervisor/tests.rs` |
| `crates/parallax-server/src/live.rs` | 135 | 27 | `crates/parallax-server/src/live/tests.rs` |
| `crates/parallax-server/src/self_telemetry.rs` | 147 | 50 | `crates/parallax-server/src/self_telemetry/tests.rs` |
| `crates/parallax-server/src/worker.rs` | 320 | 366 | `crates/parallax-server/src/worker/tests.rs` |
| `crates/parallax-storage/src/arrow_sql.rs` | 292 | 104 | `crates/parallax-storage/src/arrow_sql/tests.rs` |
| `crates/parallax-storage/src/greptime.rs` | 3501 | 294 | `crates/parallax-storage/src/greptime/tests.rs` |
| `crates/parallax-storage/src/memory.rs` | 1645 | 928 | `crates/parallax-storage/src/memory/tests.rs` |
| `crates/parallax-storage/src/metadata.rs` | 846 | 371 | `crates/parallax-storage/src/metadata/tests.rs` |
| `crates/parallax-storage/src/spool.rs` | 383 | 164 | `crates/parallax-storage/src/spool/tests.rs` |
| `crates/parallax-xtask/src/cli.rs` | 58 | 34 | `crates/parallax-xtask/src/cli/tests.rs` |
| `crates/parallax-xtask/src/command.rs` | 107 | 26 | `crates/parallax-xtask/src/command/tests.rs` |
| `crates/parallax-xtask/src/diagnostic.rs` | 117 | 53 | `crates/parallax-xtask/src/diagnostic/tests.rs` |
| `crates/parallax-xtask/src/facade.rs` | 136 | 37 | `crates/parallax-xtask/src/facade/tests.rs` |
| `crates/parallax-xtask/src/policy/architecture.rs` | 336 | 160 | `crates/parallax-xtask/src/policy/architecture/tests.rs` |
| `crates/parallax-xtask/src/policy/config.rs` | 110 | 23 | `crates/parallax-xtask/src/policy/config/tests.rs` |
| `crates/parallax-xtask/src/policy/docs.rs` | 103 | 12 | `crates/parallax-xtask/src/policy/docs/tests.rs` |
| `crates/parallax-xtask/src/policy/product.rs` | 394 | 50 | `crates/parallax-xtask/src/policy/product/tests.rs` |
| `crates/parallax-xtask/src/policy/rust.rs` | 337 | 19 | `crates/parallax-xtask/src/policy/rust/tests.rs` |
| `crates/parallax-xtask/src/policy/structural.rs` | 228 | 34 | `crates/parallax-xtask/src/policy/structural/tests.rs` |
| `crates/parallax-xtask/src/policy/typescript.rs` | 695 | 252 | `crates/parallax-xtask/src/policy/typescript/tests.rs` |

The 928-line memory-store body is a 104-line index plus 457-line
`fields_metrics.rs` and 365-line `traces_services.rs` scenario includes. The
`include!` layout keeps every test directly inside `memory::tests`, so it does
not change a fully qualified ID. No new or restructured scenario exceeds 600
lines.

## Target, resource, and selector inventory

- Extraction parity: the original 224 tests still pass with identical IDs.
  Three new policy fixtures bring the final default inventory to 227 tests
  across 26 binaries; all 227 pass.
- Ignored real-engine targets (six): `m1_greptime`,
  `m1_table_inventory_greptime`, `m2_metrics_greptime`, `m5_gates`,
  `m6_conformance_greptime`, and `m7_metric_exemplar_migration_greptime`.
  Each owns a temporary data directory but shares the cached GreptimeDB binary
  and supervisor ports. Their exact selectors, proposed `greptime-engine`
  group, timeout, and Plan 101 owner are recorded with stable IDs in Plan 101.
- Memory integration harness: `tests/support/harness.rs` is path-included by
  seven integration targets and is not an empty standalone test binary.
- Shared fakes/builders/conformance: five owned stable-ID rows in Plan 097
  cover MemoryStore, API resolver fixtures, the server memory harness, storage
  conformance, and telemetry builders.
- Doctests remain a separate partition. The API private-resolver compile-fail
  doctest proves integration consumers cannot import implementation modules.
- Temporary files/directories in migrated fixtures use drop guards. The worker
  isolation test uses a oneshot completion signal instead of polling sleep, and
  self-telemetry endpoint tests inject environment values instead of mutating
  process state.

The schema policy rejects malformed, duplicate, placeholder, pending, or
unowned Plan 097/101 handoff rows. Syntax-aware determinism metrics reject new
environment mutation, sleep, listener-bind, wall-clock, or temp-root scopes;
existing integration boundaries are exact and shrink-only.

## ID parity

External child modules retain the name `tests`, so every one of the 224 test
IDs printed before extraction is identical afterward. This includes all IDs
referenced by the Plan 093 baseline/defect ledgers (`traces_page`, spool
rotation, bundle redaction, removed storage fallback, and worker replay) and all
real-engine selectors. Therefore the old-to-new mapping is the identity mapping
for every test; there are zero renamed IDs and no selector or quarantine update
is required.

## Closure gates

- `mise exec -- cargo xtask ci --full` passed at `8727d05`: strict Rust
  formatting/Clippy, Bun formatting/typecheck/lint, 41 Vitest files / 175 UI
  tests, client and SSR builds, 227 nextest tests across 26 binaries, the
  compile-fail doctest partition, and RustSec audit.
- `cargo nextest list -p parallax-server --run-ignored=only --message-format
  json` selected exactly the six owned real-engine tests.
- `cargo xtask policy --output json` returned `[]`; repository scans found no
  inline test body, `mod.rs`, or test scenario over 600 lines.
- Hosted [CI run 29203280582](https://github.com/tailrocks/parallax/actions/runs/29203280582)
  passed at `8727d05`, including `policy`, `clippy`, `test`, and `ci-required`.
