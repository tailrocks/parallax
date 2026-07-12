# Plan 095: Add the Parallax quality control plane

> **Executor instructions**: Build one Rust-native implementation used by local
> and CI commands. Use one typed ratchet configuration from day one. Start noisy
> metrics report-only and promote only deterministic rules. Never add hollow
> placeholder subcommands. Implement the common architecture, budget, and
> exception schema from `ENGINEERING-STANDARDS.md` for both Rust and TypeScript.
> Use Oxc Rust crates as the single TypeScript parser/resolver implementation.

## Status

- **Priority**: P1
- **Effort**: L
- **Risk**: MEDIUM
- **Depends on**: 094
- **Category**: tooling / architecture policy
- **Planned at**: `a1d8bf82`, revised 2026-07-12
- **Status**: TODO

## Why

Parallax quality policy is spread across workflow YAML, prose, and ad hoc
commands. There is no Cargo-metadata architecture gate, structural ratchet,
machine-readable diagnostics, semantic facade/doc gate, or one local command
that matches CI.

## Scope

In scope:

- `crates/parallax-xtask`, `.cargo/config.toml`, and one root `ratchet.toml`.
- Human, JSON, and GitHub diagnostic renderers.
- Rust/TypeScript architecture, facade, product-policy, health, and CI
  orchestration.
- Required path-filtered policy CI job.
- Policy for self-telemetry loop prevention.

Out of scope:

- Dependency/release subcommands before plans 101/102 implement them.
- Product crate/module moves, owned by 097/126/098.
- Exact strict lint rollout, owned by 096.
- Automatic ratchet-update PRs.

## Steps

### Step 1: Scaffold commands and diagnostics

Add real implementations for:

- `cargo xtask ci --fast|--full`;
- `lint`, `test`, `ui`, `integration`;
- `policy [--only <rule>]`;
- `arch`, `facade refresh|check`, and `health`.

Every finding contains schema version, rule ID, severity, file/line, reason,
remediation, and rerun command. Human/JSON/GitHub outputs report the same full
set before exiting. Parser/dispatch/golden tests cover every subcommand.

### Step 2: Enforce the staged architecture graph

Derive workspace packages and normal/build/dev edges from Cargo metadata.
Classify every current member. Reject:

- missing classifications;
- upward or same-tier product edges;
- production and dev cycles;
- stale/unknown exceptions;
- any product dependency on xtask/MCP spike;
- normal/build reachability from release roots to test support, or a normal/dev
  cycle involving it.

Allow acyclic product-to-test-support dev edges because plan 097 requires them.
Fixture normal, build, dev, target-specific, optional-feature, and mixed-cycle
edges separately. Initially allow only the exact measured `core -> storage`
normal edge with reason, owner, removal plan 097, and expiry. Do not require
crates that do not exist yet; classify them in their creation commit. Plan 126
removes the compatibility core and completes the final tier graph.

### Step 3: Add one typed ratchet source

Use one `ratchet.toml` for Rust and TypeScript file/function/component size,
complexity, suppressions/assertions, public root/feature surface, test layout,
module style, import layers/cycles, dependency exceptions, unsafe blocks,
generated ownership, agent-doc bytes, and hot-path clone floors. Parse Rust and
TypeScript syntax; do not count with regex. Numeric and presence providers
reject growth, scope broadening, and stale rows. Values and ceilings come from
`ENGINEERING-STANDARDS.md`. There are no legacy budget files/readers.

Pin one compatible Rust Oxc family and build the TypeScript providers from
`oxc_parser`, `oxc_ast`, `oxc_semantic`, and `oxc_resolver`. The import provider
resolves tsconfig aliases, package exports, type-only edges, barrels/reexports,
dynamic imports, generated route composition, `.server`/`.client` conditions,
and feature public entry points. The AST provider measures functions,
components, hooks, assertions, directives, and exports so moving a 400-line
component out of a route cannot claim success. Parse or resolution failure is a
finding, never an absent edge. This graph is authoritative; Oxlint's native
import/cycle rules may provide fast supplemental diagnostics but may not create
a second architecture truth. Publish the alias, barrel, type-only, dynamic,
server/client, and cycle cases as a tool-neutral downstream parity corpus. This
plan does not install or invoke Oxlint before plan 131 owns that live toolchain;
plans 131 and 100 consume the corpus and prove supplemental agreement later.

Fixture TS/TSX/JS/JSX, extension/index lookup, aliases, type-only imports,
`import()`, reexports, package exports, missing targets, cycles, root-layout,
generated route, and server/client cases. Record version/source/license review
for the initial Oxc crates; plan 101 later makes compatible-family upgrades and
supply-chain coverage executable.

`health` begins report-only. Promotion to `policy` requires deterministic
fixture coverage and a documented remediation.

### Step 4: Add product-law rules

Gate:

- mandatory GreptimeDB + Turso release composition;
- native Greptime raw-signal tables;
- no product MemoryStore route;
- no active rustls backend and correct native-TLS release feature;
- Bun-only files/commands;
- Apache-2.0 declarations;
- zero-copy ingest clone floors;
- storage ingest log-quietness or explicit self-telemetry filter coverage;
- Bun-only server/client bundle ownership and no server-only module in the
  browser graph.

Scope checks so historical research and neutral `rustls-pki-types` do not
produce false positives. Prefer Cargo metadata/structured parsers to greps.

### Step 5: Add facade and semantic-doc manifests

`facade refresh|check` parses crate-root `pub mod`, `pub use`, and public root
items into sorted crate-local manifests with `cfg` conditions. Fixture-test
nested reexports and malformed input. This is a structural root-facade oracle,
not full published-semver analysis.

Semantic crate-doc validation compares Cargo tier/dependencies and source
modules/exports with each README, rather than accepting any same-commit touch.

### Step 6: Make local and CI paths identical

CI calls xtask partitions rather than re-encoding their rules. `ci --full`
includes plan 094's real cargo-audit gate. Plans 101/102 extend the command only
when their actual implementations land.

## Test Plan

- Architecture fixtures for every forbidden/stale graph case.
- Ratchet shrink/growth/stale/missing/malformed fixtures.
- Oxc TypeScript/JS parser and resolver fixtures for aliases, type-only/dynamic/
  barrel/package edges, parse/resolution failures, import direction, and AST
  function/component/hook measurement.
- Diagnostic equivalence golden tests.
- Product-policy positive/negative fixtures.
- Facade and crate-doc semantic fixtures.
- CI command inventory test preventing placeholders.

## Done Criteria

- [ ] `cargo xtask ci --fast|--full` execute real documented partitions.
- [ ] Human/JSON/GitHub outputs are schema-valid and equivalent.
- [ ] Architecture fails closed for all listed cases.
- [ ] `ratchet.toml` is the only structural budget source.
- [ ] Rust files/modules/tests and TypeScript files/functions/import layers use
  Oxc-backed syntax/semantic/resolution providers with generated ownership.
- [ ] The Oxc architecture graph is the only authoritative TypeScript import
  graph, and its tool-neutral downstream Oxlint parity corpus is fixture-complete;
  this plan does not create a second graph or require live Oxlint.
- [ ] Product laws are required in `ci-required`.
- [ ] Facade and semantic crate-doc checks have fixtures.
- [ ] No product crate depends on xtask.
- [ ] Local and CI policy invoke the same Rust implementation.

## STOP Conditions

- A rule needs a broad false-positive allowlist or ad hoc text parsing where
  structured metadata is available.
- The initial required policy cannot be green after plan 093.
- A baseline can increase without an explicit separate policy change.
- A parser cannot resolve TypeScript aliases/server-client edges or Rust cfg
  edges and silently treats them as absent.
- A placeholder command reports success.

## Remove When

Delete this plan and row once xtask/policy is required, fixture-complete, and
the initial graph/ratchets are green on `main`.
