# Plan 119: Generate semantic-convention constants from one registry

> **Executor instructions**: Preserve every emitted wire name byte-for-byte.
> Build one repository-owned registry and deterministic checked-in generation;
> do not require Docker or Weaver during normal product builds. Coordinate the
> companion playground repository without creating extra branches or PRs.

## Status

- **Priority**: P2
- **Effort**: M
- **Risk**: MEDIUM
- **Depends on**: 095, 096, 100, 101, 126
- **Category**: schema ownership / code generation / CI
- **Planned at**: `a1d8bf82`, revised 2026-07-12
- **Status**: IN PROGRESS

## Why

Historical plan 066 consolidated load-bearing semantic-convention literals into
Rust/Java/TypeScript constants and produced a Weaver feasibility note. The note
recommended one YAML registry, checked-in generated constants, a temp-directory
regeneration diff in CI, and removal of Java duplication. Leaving those four
steps in the research note created a second active plan. A single registry makes
telemetry field names an explicit cross-language contract while retaining
reviewable generated source and reproducible offline builds.

## Current Evidence

- At the planning baseline Parallax had `parallax-proto/src/semconv.rs` and a
  core facade/freeze test; the generated T0 crate now owns this surface.
- The companion playground maintains Rust, Java, and TypeScript emitters that
  must agree with Parallax consumers.
- `docs/research/architecture/semconv-registry-design.md` records the evaluated
  Weaver registry/template shape and the previous current-version check.
- TypeScript generation required a custom template at the research snapshot;
  recheck current stable Weaver before choosing templates.
- `7effc39` extends the repository-owned generator's negative contract checks:
  duplicate generated Rust/TypeScript/Java identifiers and empty scalar/list
  wire values now fail before rendering, alongside the existing duplicate-ID
  and cardinality checks. The renderer was flattened without output changes to
  satisfy strict linting. `cargo test --locked -p parallax-xtask` (55 tests),
  strict xtask clippy, Weaver registry validation, and the cross-repository
  deterministic semconv check pass locally.
- `76c773e` completes the ready machine-readable control-plane slice:
  `cargo xtask --output json semconv check` now emits one clean versioned JSON
  document naming every checked-in root/playground artifact. Successful Weaver
  output is captured so it cannot corrupt JSON; failed Weaver output remains
  in the error diagnostic. Xtask tests (55), strict clippy, and the human/JSON
  cross-repository deterministic checks pass locally.
- `6ecb2c1` adds the generator-owned, versioned cross-language wire-contract
  fixture to the companion repository. It names every shared/playground wire
  ID, language identifier, scalar/list value, and owner; a Bun Vitest test
  proves generated TypeScript exports match it exactly, while xtask's artifact
  check proves the Rust and Java generated files match the same registry.
  Xtask tests (55), strict clippy, semconv drift checking, and companion
  Vitest (7) pass locally.
- `6997df1` adds the Java-side consumer of that same fixture. It reflects each
  generated `Semconv` field and proves its scalar/list wire value against the
  versioned contract, completing executable fixture consumers for TypeScript
  and Java. `898eae2` subsequently unblocked the Java host by relocating
  Gradle's native cache; the clean catalog suite now executes this fixture
  consumer successfully alongside the generated TypeScript Vitest consumer.
- The current local implementation makes stale generated-artifact rejection directly unit-testable:
  the xtask creates output from a temporary registry, accepts the untouched
  artifacts, and fails closed after a hand edit without invoking Weaver or the
  network. Formatting, all 56 xtask tests, strict clippy, and the real
  cross-repository semconv check pass locally.
- 2026-07-14: the TypeScript renderer now emits the repository's current Oxfmt
  style (including multiline lists and long declarations) rather than relying
  on a post-generation manual formatter pass. Root and companion generated
  TypeScript outputs were regenerated together. A direct renderer fixture locks
  that formatter-compatible layout. Root xtask tests (58), strict
  clippy, cross-repository drift checking, root UI formatting/tests/build, and
  companion web tests/build pass locally.
- 2026-07-15: added a representative product OTLP protobuf round-trip instead
  of relying only on generated-source equality. `parallax-proto` constructs,
  encodes, and decodes a trace carrying registry-generated resource, run, test,
  and GraphQL names/values, then asserts the exact wire strings survive. The
  locked proto test and strict all-target proto clippy pass locally; the same
  constants remain tied to the Java/TypeScript consumers by the versioned
  cross-repository fixture.
- 2026-07-15: removed the remaining stale analysis-layer ownership indirection.
  `parallax-analysis` now depends on and re-exports the generated T0
  `parallax-semconv` crate directly rather than reaching through
  `parallax-proto` under a "temporary until Plan 119" comment. Its 27 locked
  tests and strict all-target clippy pass locally.
- 2026-07-15: completed the Rust consumer migration and removed the temporary
  `parallax-proto::semconv` compatibility module. Server, storage, Greptime,
  metadata, ingest, test-support, analysis, examples, and the OTLP wire fixture
  now depend directly on `parallax-semconv`; repository search finds no old
  imports. Locked all-target checks and strict clippy pass across all eight
  affected crates.
- 2026-07-15: made the final Rust ownership state part of the read-only
  semconv gate. `cargo xtask semconv check` now rejects recreation of
  `parallax-proto/src/semconv.rs` and any Rust `use` that reaches conventions
  through `parallax_proto::semconv`. Positive and both negative fixtures pass,
  strict xtask clippy is clean, and the real cross-repository deterministic
  check passes at the pushed tree.
- 2026-07-15: closed additional production-consumer ownership gaps found by a
  literal audit. The registry now owns the CLI-emitted `parallax.lab` overlay;
  CLI forwarding, storage sensitivity rules, SQL run navigation, GraphQL trace
  parsing, service exemplars, and runtime metric queries consume generated
  Rust/TypeScript constants without changing wire values. Cross-repository
  regeneration is deterministic; 23 affected Rust tests, strict clippy,
  TypeScript checking, formatting, and 15 focused UI tests pass locally.
- 2026-07-15: companion commit `6a24e80` removes the remaining mutable
  TypeScript producer literal found by the same audit: browser screen spans now
  key `url.path` through the generated constant. Seven Vitest cases and the
  production web build/typecheck pass locally. Rust `tracing` field keys remain
  literal only where the macro grammar requires compile-time field syntax.

## Scope

In scope:

- One versioned YAML semantic-convention registry for the Parallax overlay and
  approved playground spans, events, metrics, and shared attributes.
- Pinned, verified Weaver tooling through mise and deterministic local xtask
  orchestration with human/JSON diagnostics.
- Checked-in generated Rust, Java, and TypeScript constants from reviewed
  repository-owned templates.
- The T0 `parallax-semconv` leaf crate from the final plan 126 graph and
  generated TypeScript under plan 100's `shared` public boundary.
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
   checked-in files remain the product build inputs. Rust output is owned by the
   dependency-free T0 `parallax-semconv` crate; TypeScript output is exposed
   through `ui/src/shared` and never from a feature or route.
4. Add `cargo xtask semconv check|generate` through the plan 095 diagnostic
   control plane. `check` must be read-only, deterministic, and machine-readable;
   `generate` is an explicit maintainer command.
5. Migrate Parallax and playground consumers in small language-specific slices.
   Preserve freeze tests throughout and remove hand-written duplicates only after
   generated output has compiled and emitted identical fixtures.
6. Add path-aware CI for registry/template/generated-source changes. Prove stale,
   malicious, missing, and nondeterministic output fails without network access.
   Generated output is excluded only from manual size/style ownership, not
   compile, format, import direction, license, secret, or drift checks.

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
