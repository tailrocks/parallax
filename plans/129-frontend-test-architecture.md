# Plan 129: Validate the deterministic Vitest foundation cross-platform

> **Executor instructions**: The Linux implementation is complete. Do not move
> production features or absorb Playwright/runtime-schema scope. Retire only
> after Plan 128 closes and the same forced-Bun contract passes on macOS at the
> exact branch head.

## Status

- **Priority**: P1
- **Effort**: S remaining
- **Risk**: MEDIUM
- **Depends on**: 094, 101, 128; supported macOS validation
- **Category**: TypeScript / React / unit and integration testing
- **Planned at**: `e3e7997`, revised 2026-07-15
- **Status**: BLOCKED — hard dependency Plan 128 is upstream-blocked and this
  Linux arm64 host cannot produce the required macOS process evidence

## Completed Contract

`ui/test-matrix.json` owns every current test and all 21 required product,
platform, capability, and shared surfaces with stable IDs, meaningful risks,
exact final/legacy topology, private-route symbol handoffs, and shrink-only
ratchets. The executable `ui.tests` policy rejects stale IDs, missing surfaces,
unowned files, broadened handoffs, duplicated browser shims, raw router growth,
unjustified `fireEvent`, sleeps/timers, snapshots, skipped/focused tests,
diagnostic allowlists, and missing stable native Vitest rules.

The deterministic harness owns cleanup, exact console/page/rejection
diagnostics, network escape, UTC/time/IDs, GraphQL/SSE fixtures, router/history,
browser shims, reset hooks, and opt-in timer tracking. Normal interactions use
`userEvent`; all seven remaining `fireEvent` calls are exact chart pointer
mechanics. Test bodies are separated from production/harness code, and every
legacy path/private import has one expiring feature-plan owner.

At root commit `2b4fd44`, Linux arm64 passes:

- `cd ui && bun run --bun test:ci` — 45 files, 188 tests, zero unexpected
  diagnostics;
- TypeScript, native and type-aware Oxlint, and Oxfmt checks;
- `cargo xtask policy --only ui.tests` and both policy fixtures;
- strict all-target/all-feature xtask clippy and Rust formatting.
- `cargo xtask ci --fast`, including the production UI build.

The Plan 129-owned and repository-wide structural findings are zero.

## Durable Outputs

- `ui/test-matrix.json`
- `ui/src/test/**` and `ui/tests/harness/**`
- `crates/parallax-xtask/src/policy/ui_tests.rs` and its scanner/tests
- the exact Vitest rules in `ui/.oxlintrc.jsonc`

These remain after plan retirement and are consumed by Plans 100, 132-153.

## Remaining Work

1. Close Plan 128 without `skipLibCheck`, compiler weakening, declaration
   patches, or application escape hatches.
2. At the same pushed head on supported macOS, run the forced-Bun suite twice,
   prove Bun process ancestry with no Node descendant, and exercise the
   missing-runtime/lock, implicit-install, injected-Node, and zero-selection
   negative gates.
3. Preserve exact-head evidence, then delete this plan and its index row.

## STOP Conditions

- Do not claim macOS from Linux or remote configuration inspection.
- Do not bypass Plan 128, weaken declarations, allow Node, or exclude tests.
- Do not absorb feature moves, runtime decoding, or browser automation.

## Remove When

Delete this plan and index row when Plan 128 is complete, the exact-head macOS
forced-Bun/negative evidence passes, and the fast aggregate is green.
