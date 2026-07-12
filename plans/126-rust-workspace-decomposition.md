# Plan 126: Decompose Rust into domain, port, adapter, and composition crates

> **Executor instructions**: Follow the exact target graph in
> `ENGINEERING-STANDARDS.md`. Move one vertical responsibility with its
> compatibility tests at a time. Do not create an empty crate, duplicate a
> model, expose an adapter through a port, or clone telemetry to cross a new
> boundary.

## Status

- **Priority**: P1
- **Effort**: XL
- **Risk**: HIGH
- **Depends on**: 097, 101
- **Category**: Rust / workspace architecture
- **Planned at**: `a1d8bf82`, 2026-07-12
- **Status**: TODO

## Why

Plan 097 fixes the immediate `core -> storage` inversion and creates model,
ports, and test support. That is necessary but not the final architecture.
Current core and storage still combine pure domain analysis, evidence building,
OTLP normalization, Greptime transport, Turso metadata, spool durability, and
query contracts. Private modules alone would leave every consumer compiling
against those unrelated dependencies.

The target is a deliberate multi-crate workspace, not a copy of another
project's crate count. Each new crate isolates a named invariant, external
dependency, or independent verification surface.

## Target Graph

```text
T0  parallax-proto       OTLP wire/service types
T0  parallax-model       normalized domain/value types
T0  parallax-semconv     generated registry output (created by plan 119)

T1  parallax-ingest      decode/normalize hot path
T1  parallax-analysis    error derivation/fingerprint/trace/span analysis
T1  parallax-storage     capability ports/query-neutral contracts

T2  parallax-evidence    bundle/story/gap/redaction/bounding/ranking/hash
T2  parallax-greptime    GreptimeDB adapter/native-table ownership
T2  parallax-metadata    Turso adapter/migrations
T2  parallax-spool       raw-frame durability/recovery

T3  parallax-api
T4  parallax-server
T5  parallax-cli

Aux parallax-test-support, parallax-xtask, parallax-mcp-spike
```

Production edges point downward only. `api` sees storage contracts, never
Greptime/Turso clients. `server` owns concrete composition. `parallax-core` is
deleted after its temporary facade has no consumers.

## Scope

In scope:

- New ingest, analysis, evidence, Greptime, metadata, and spool
  crates; final ownership of the port crate.
- Compile-driven dependency migration and deletion of the `parallax-core`
  compatibility shell.
- Workspace tier classification, facade manifests, metadata inheritance,
  publish policy, feature ownership, and dependency-cost evidence.
- Preservation of serde, OTLP, GraphQL, SQL, Arrow, bundle, CLI, persistence,
  zero-copy, and release behavior.

Out of scope:

- Creating `parallax-semconv`, owned by plan 119.
- Product contract, schema, storage-engine, or raw native-table changes.
- Typed error/ID redesign beyond the boundaries owned by plan 099.
- New generic `common`, `utils`, `helpers`, or crate-per-file packages.

## Steps

### Step 1: Freeze the final graph and slice ledger

Use Cargo metadata plus plan 097's landed graph to record every package, normal/
build/dev edge, feature, external dependency, compile cost, public root item,
and current owner. For each proposed crate record the two or more extraction
criteria it satisfies from `ENGINEERING-STANDARDS.md`, its facade, consumers,
forbidden dependencies, compatibility oracles, and deletion source.

Reject a proposed crate that has no real implementation or merely renames a
directory. Confirm all internal packages are `publish = false` and inherit
version, exact Rust version, edition, license, repository, and workspace lints.

A byte-identical mechanical move of a legacy oversized file may transfer its
exact shrink-only bound to the new path, but the row names plan 098 as owner,
keeps the original measured bound, and expires in that plan's module wave. A
move cannot reset, raise, or declare the file a new compliant baseline.

### Step 2: Extract pure business leaves

Move decode/normalization into `parallax-ingest`; error derivation,
fingerprinting, trace analysis, and span-event logic into `parallax-analysis`;
and bundle/story/gap/redaction/bounding/ranking/hash logic into
`parallax-evidence`. Evidence may depend downward on analysis; ingest and
analysis must not depend on one another as same-tier peers. Shared pure value
types live in `parallax-model` only when both crates genuinely consume them;
evidence-only projections such as metric windows stay in evidence. Keep
transport, database rows, GraphQL types, and CLI rendering out of these crates.

Each slice lands with moved external test modules, existing goldens, and a
dependency assertion. Ingest accepts decoded ownership and moves model values
forward; it may not add `clone`/serialization round trips to make a facade easy.
Evidence assembly receives typed inputs and remains independent of storage
ports/adapters; an API/server orchestration owner performs reads before calling
it.

### Step 3: Extract concrete adapters

Transform `parallax-storage` into capability traits and query-neutral contract
types only. Move Greptime HTTP/Arrow/native-table behavior to
`parallax-greptime` and Turso connection/migrations/row mapping to
`parallax-metadata`. Their external client crates must disappear from API and
pure-domain dependency trees.

Create `parallax-spool`: raw-frame framing, append, crash recovery, replay,
limits, and compatibility form an independent durability/security invariant
with dedicated fuzz/conformance needs and server/doctor/prune consumers. It may
depend on low-level model/config contracts but not API or engine adapters. The
spool never becomes a fallback database.

### Step 4: Move composition upward

Make server startup the composition root for mandatory GreptimeDB + Turso,
spool, ingest, API, and workers. API contexts receive capability objects and
domain services. CLI remains an edge over the API client plus the explicit
embedded-serve path; it does not construct storage behind commands.

Verify default, `embed-ui`, `conformance`, test, and release graphs separately.
No feature may invert tiers or make a product graph reach test support/xtask/
MCP spike.

### Step 5: Retire compatibility shells

Migrate one consumer at a time through reviewed crate-root facades. Delete
reexports as soon as their last consumer moves. Delete `parallax-core` when
Cargo metadata and repository search prove no supported consumer or documented
public path remains. Do not keep a permanent umbrella crate for convenience.

### Step 6: Lock the architecture

Update the xtask tier map, negative fixtures, facade manifests, workspace map,
crate READMEs, and structural ratchets in the same slices. Measure clean and
incremental compile effects; a regression does not automatically block the
architecture, but it must be explained rather than hidden by cache state.

## Test Plan

- Cargo-metadata fixtures for missing tier, upward/same-tier edge, normal/dev
  cycle, forbidden aux reachability, stale exception, and feature-only edge.
- Compile-contract tests for every new facade and forbidden adapter import.
- Existing normalization, analysis, bundle, SQL, Arrow, GraphQL, CLI, and
  persistence goldens at every slice.
- Memory/live Greptime conformance and Turso migration/restart tests.
- Allocation/clone checks on representative OTLP batches.
- Default, all supported feature, test, release, and clean incremental builds.
- Release dependency tree proving all internal crates are non-publishable and
  no test-support/xtask/MCP package is reachable from release roots through
  normal/build edges or present in binaries/SBOM dependency graphs.

## Incoming Handoff From 097

Plan 097 replaces this placeholder before retirement. The table is
schema-validated; stable IDs are never reused and every row has a target owner,
compatibility oracle, and terminal extraction status before plan 126 retires.

| Stable ID | Current owner/surface | Target crate/facade | Consumers/oracles | Status |
|-----------|-----------------------|---------------------|-------------------|--------|
| `097-extraction-pending` | Populate during plan 097 | Populate during plan 097 | Populate during plan 097 | PENDING |

## Done Criteria

- [ ] Cargo metadata exactly matches the target graph for every created owner.
- [ ] API cannot reach Greptime, Turso, Arrow transport, or spool implementation.
- [ ] Server is the only concrete production composition root.
- [ ] Core responsibilities live in ingest/analysis/evidence and
  `parallax-core` is deleted.
- [ ] Storage contains ports/query-neutral contracts only.
- [ ] Every crate satisfies at least two extraction criteria and has a reviewed
  facade, tests, docs, and metadata inheritance.
- [ ] Every transferred legacy hotspot retains its exact bound and an expiring
  plan 098 split owner; no move resets a structural baseline.
- [ ] Every internal package is `publish = false`.
- [ ] No move changes wire, schema, persistence, CLI, allocation, or product
  behavior.
- [ ] Default and supported-feature Clippy/test/release graphs pass.

## STOP Conditions

- A crate would be empty, generic, single-file ceremony, or have no stable
  facade/consumer.
- A boundary requires telemetry cloning, serialization, or database fields in
  domain types.
- API or a pure crate needs a concrete engine client.
- A feature creates a hidden upward edge or normal/build path to test support.
- A compatibility shell cannot be removed because an undocumented public
  contract is discovered; characterize and settle that contract first.
- The work requires a custom raw-signal table or product storage substitution.

## Remove When

Delete this plan and index row when the final workspace graph, composition
root, crate metadata/facades, compatibility tests, and deletion of the core
shell are enforced and green.
