# Plan 119: Generate semantic-convention constants from one registry

> **Executor instructions**: Preserve every emitted wire name byte-for-byte.
> Build one repository-owned registry and deterministic checked-in generation;
> do not require Docker or Weaver during normal product builds. Coordinate the
> companion playground repository without creating extra branches or PRs.

## Status

- **Priority**: P2
- **Effort**: M
- **Risk**: MEDIUM
- **Depends on**: 095, 096, 100, 101
- **Category**: schema ownership / code generation / CI
- **Planned at**: `eefa4617`, 2026-07-12
- **Status**: TODO

## Why

Historical plan 066 consolidated load-bearing semantic-convention literals into
Rust/Java/TypeScript constants and produced a Weaver feasibility note. The note
recommended one YAML registry, checked-in generated constants, a temp-directory
regeneration diff in CI, and removal of Java duplication. Leaving those four
steps in the research note created a second active plan. A single registry makes
telemetry field names an explicit cross-language contract while retaining
reviewable generated source and reproducible offline builds.

## Current Evidence

- Parallax has `parallax-proto/src/semconv.rs` and a core facade/freeze test.
- The companion playground maintains Rust, Java, and TypeScript emitters that
  must agree with Parallax consumers.
- `docs/research/architecture/semconv-registry-design.md` records the evaluated
  Weaver registry/template shape and the previous current-version check.
- TypeScript generation required a custom template at the research snapshot;
  recheck current stable Weaver before choosing templates.

## Scope

In scope:

- One versioned YAML semantic-convention registry for the Parallax overlay and
  approved playground spans, events, metrics, and shared attributes.
- Pinned, verified Weaver tooling through mise and deterministic local xtask
  orchestration with human/JSON diagnostics.
- Checked-in generated Rust, Java, and TypeScript constants from reviewed
  repository-owned templates.
- A temp-directory regeneration/diff gate used identically by local and CI
  commands, plus cross-language wire-name fixtures.
- Migration of existing hand-written constant modules without changing emitted
  names, values, types, cardinality, or storage columns.
- Removal of package-local Java duplication after generated output is stable.

Out of scope:

- Renaming an attribute, metric, event, span, or stored column.
- Generating during ordinary Cargo/Bun/Java product builds.
- Requiring Docker, network access, or mutable downloads in normal builds.
- Importing every upstream OTel convention or introducing a second registry.

## Steps

1. Recheck the latest stable Weaver CLI, registry schema, templates, licensing,
   and install path. Inventory every emitted/consumed constant in Parallax and
   the companion playground, then freeze exact wire names and ownership.
2. Define the minimal registry and manifest with explicit standard imports and
   Parallax-owned groups. Add negative fixtures for duplicate IDs, invalid
   stability/type/cardinality, unknown imports, and wire-name drift.
3. Implement reviewed repository-owned templates for Rust, Java, and TypeScript.
   Generate into a temporary directory first and compare exact normalized output;
   checked-in files remain the product build inputs.
4. Add `cargo xtask semconv check|generate` through the plan 095 diagnostic
   control plane. `check` must be read-only, deterministic, and machine-readable;
   `generate` is an explicit maintainer command.
5. Migrate Parallax and playground consumers in small language-specific slices.
   Preserve freeze tests throughout and remove hand-written duplicates only after
   generated output has compiled and emitted identical fixtures.
6. Add path-aware CI for registry/template/generated-source changes. Prove stale,
   malicious, missing, and nondeterministic output fails without network access.

## Test Plan

- Registry schema/import and invalid-definition fixtures.
- Golden generation for Rust, Java, and TypeScript from one frozen registry.
- Two clean generation runs with byte-identical output.
- Cross-repository fixture comparing every shared wire name and representative
  emitted OTLP payloads before/after migration.
- Cargo, Bun, and Java compile/tests using checked-in output with Weaver absent.
- CI routing and machine-readable diagnostic contract tests.

## Done Criteria

- [ ] One registry owns every approved shared Parallax/playground convention.
- [ ] Generated Rust, Java, and TypeScript output is deterministic and checked in.
- [ ] Normal product builds require neither Weaver, Docker, nor network access.
- [ ] Temp regeneration fails on every stale or hand-edited generated artifact.
- [ ] Cross-language OTLP fixtures prove byte-identical wire names and values.
- [ ] Package-local duplication is removed without changing public/storage data.
- [ ] Xtask, path-aware CI, strict Rust, Bun, and companion-language gates pass.

## STOP Conditions

- An existing spelling/type/cardinality conflict would change a live wire or
  stored-column contract; record it as a separate compatibility decision first.
- Current stable Weaver cannot express the required overlay safely or has no
  maintainable deterministic template path for one target language.
- The companion repository or its operator-approved single branch is unavailable.
- Generation would become an implicit network/Docker dependency of product builds.

## Remove When

Delete this plan and index row when the one-registry generation and drift gate
own all approved cross-language conventions with byte-identical compatibility,
or when a durable decision rejects Weaver and closes every actionable follow-up.
