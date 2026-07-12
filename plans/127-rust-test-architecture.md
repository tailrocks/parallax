# Plan 127: Separate Rust tests and enforce test ownership

> **Executor instructions**: Move tests without rewriting their assertions in
> the same step. Preserve private access through external child modules, keep
> integration tests on public facades, and split by scenario rather than
> creating a single oversized `tests.rs`.

## Status

- **Priority**: P1
- **Effort**: L
- **Risk**: MEDIUM
- **Depends on**: 095
- **Category**: Rust / test architecture
- **Planned at**: `a1d8bf82`, 2026-07-12
- **Status**: IN PROGRESS

## Why

Thirty-one production source files contain inline test bodies, 32 declare a
`#[cfg(test)]` surface, major bodies sit at the bottom of the largest
implementation files, and one legacy `mod.rs` remains. Existing plans say tests
should move with responsibilities but
do not define a workspace-wide final layout or prevent new inline bodies. A
rigid single-test-file rule would merely move the hotspot; tests need the same
responsibility boundaries as production code.

## Target Layout

For `src/foo.rs`:

```text
src/foo.rs                    production body + `#[cfg(test)] mod tests;`
src/foo/tests.rs              focused tests or small scenario index
src/foo/tests/parsing.rs      optional concern-specific scenario module
src/foo/tests/recovery.rs     optional concern-specific scenario module
```

Use `<crate>/tests/` only for supported public crate contracts. Group related
integration scenarios into a small number of test crates to control compile
cost. Shared builders/fakes/conformance move to `parallax-test-support` in plan
097, but private unit tests remain beside their owning source module.

## Scope

- Syntax-aware inventory and zero-new gates for inline test bodies and
  `mod.rs`.
- Mechanical extraction of unit tests to external child modules.
- Integration/public-contract, doctest, conformance, and real-engine ownership.
- Deterministic test rules, size ratchets, and compile/runtime evidence.

Out of scope:

- Moving reusable fixtures/fakes to the new crate before plan 097.
- Adding property/fuzz/performance targets, owned by plan 103.
- Rewriting behavior or increasing coverage unrelated to safe extraction.
- One `tests.rs` per production file regardless of size.

## Steps

### Step 1: Record topology and behavior

Parse every Rust target and record inline body location/size, integration test
crate, ignored test, fixture owner, global resource, real-engine dependency,
and runtime. Run the current suite. Preserve exact fully qualified test IDs for
every CI selector, quarantine, evidence link, or operator command; for all other
tests record an old-to-new ID map and update every stored selector atomically.
Submodule extraction naturally changes fully qualified names and must not be
blocked by an impossible blanket equality rule. Add zero-new inline-body and `mod.rs` rules before
the migration; existing rows are presence ratchets that may only disappear.

### Step 2: Extract low-risk unit modules

For each touched production module, leave only `#[cfg(test)] mod tests;` and
move test bodies verbatim to the external child. Fix paths/imports minimally,
run the exact selected tests, then full crate tests. Do not combine extraction
with production refactoring or assertion cleanup.

### Step 3: Split hotspot scenarios

For storage, CLI, API, bundle, worker, and supervisor hotspots, make `tests.rs`
a small index and group scenario files by responsibility such as bootstrap,
ingest, query, recovery, rendering, or failure stage. Apply the 600-line new
test-file target and shrink-only baselines from `ENGINEERING-STANDARDS.md`.

### Step 4: Correct public versus private tests

Private algorithms remain child unit tests. `<crate>/tests/` targets import
only reviewed crate-root paths; they must not keep implementation modules
public. Move controlled engine seeding behind a supported test-support or
adapter test API instead of exposing fields. Keep doctests public and compile
them in their separate required partition.

### Step 5: Remove nondeterministic harness patterns

As each file moves, replace ambient environment mutation with injected config,
allocate ports/directories through owned guards, use fixed clocks where time is
not the subject, and replace real sleeps with synchronization. Do not hide a
race with retries or serialization. Inventory named exclusive engine resources
and their required future nextest groups in plan 101's incoming handoff; this
plan does not configure profiles it precedes. Use the current deterministic
selection mechanism during extraction.

### Step 6: Hand off shared support

Populate the schema-checked `Incoming Handoff From 127` tables in plans 097 and
101 with stable IDs. The plan 097 rows name every builder/fake/conformance item,
current owner, consumers, and intended public test-support API. The plan 101 rows
name every exclusive resource, current selector, proposed group, timeout need,
and owner. Zero rows may remain pending or unowned. Do not create the new support
crate or nextest profiles here; the successors consume these handoffs.

## Test Plan

- Parser fixtures distinguishing a marker from an inline test body, macros,
  generated code, malformed syntax, and legacy `mod.rs`.
- Before/after selected-test parity plus fully qualified ID parity for referenced
  selectors and complete old-to-new mappings for unreferenced tests.
- Public integration compile-fail fixture for a private implementation path.
- Nextest/default/doctest/real-engine selection inventory with zero-test
  failures.
- Determinism fixtures for ports, temp dirs, time, environment, and resource
  serialization.
- Size/presence ratchet shrink, growth, stale, and malformed cases.

## Done Criteria

- [ ] No handwritten production file contains a test body.
- [ ] No Rust module uses `mod.rs`.
- [ ] Unit tests retain private access through external child modules.
- [ ] Integration tests use reviewed public facades only.
- [ ] No new or restructured test file exceeds its target without a reasoned
  expiring exception.
- [ ] Ambient environment mutation, arbitrary sleeps, and unmanaged resources
  are absent from migrated tests.
- [ ] Referenced test IDs remain stable; every other renamed ID has a complete
  mapping and all selectors/evidence are updated without behavior loss.
- [ ] The plans 097/101 handoffs are schema-valid, have stable IDs, identify
  every shared fixture/conformance/resource owner, and contain no pending row.

## STOP Conditions

- Extraction changes production behavior, test semantics, or a durable test ID.
- A test can compile only by broadening a production public surface.
- A scenario needs shared test support that would create a dependency cycle;
  leave the exact case for plan 097 and continue independent files.
- Determinism requires masking a real race with retries or blanket serialization.

## Remove When

Delete this plan and row when external test layout, public/private ownership,
determinism rules, and zero-new structural gates are green workspace-wide.
