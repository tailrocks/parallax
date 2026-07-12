# Plan 126 workspace slice ledger

**Recorded:** 2026-07-12 at `ae8109b`

## Baseline

Cargo metadata reports ten internal packages. All are `publish = false` and
inherit workspace version `0.1.0-dev`, Rust `1.97.0`, edition 2024,
Apache-2.0, repository, authors, and lints. The product graph still has these
temporary broad edges:

- API depends on `parallax-core` and `parallax-storage`.
- Server depends on core/storage and is the existing concrete composition root.
- Core owns normalization, analysis, and evidence with regex, serde/json, SHA-2,
  model, and proto dependencies.
- Storage owns ports plus Greptime HTTP/Arrow, Turso, and spool durability; its
  normal graph includes Arrow, reqwest, Turso, Tokio, bytes, futures, model, and
  proto.

Largest transfer files retain their current shrink-only measurements:
`greptime.rs` 3,095 logical lines, `bundle.rs` 905, `metadata.rs` 752,
`adapter.rs` 595, and `normalize.rs` 421. Mechanical moves keep
the exact applicable ratchet and name Plan 098 as the split owner.

## Extraction ledger

| Stable ID | Target facade and criteria | Consumers | Forbidden dependencies | Compatibility oracle / deletion source |
|---|---|---|---|---|
| `126-ingest` | `parallax-ingest`; hot-path ownership + removes proto decoding from domain consumers | server worker | storage/API/database clients, evidence, analysis | normalization tests, serde contract, clone floor; delete `core::normalize` |
| `126-analysis` | `parallax-analysis`; stable error/trace invariants + multi-consumer compilation boundary | API, server, evidence | proto decode, storage/adapters, GraphQL | derivation/fingerprint/span/trace tests; delete four core modules |
| `126-evidence` | `parallax-evidence`; security/bounding invariant + stable API/CLI consumers | API, server, CLI projections | storage/adapters, GraphQL, transport | bundle schema/redaction/hash/story/gap tests; delete four core modules |
| `126-greptime` | `parallax-greptime`; removes Arrow/HTTP clients from ports + native-table conformance surface | server composition, adapter conformance | API, Turso, spool | SQL/Arrow goldens and serialized live suite; delete three storage modules |
| `126-metadata` | `parallax-metadata`; mutable persistence invariant + removes Turso from ports | server composition, API through port object | API/GraphQL, Greptime, spool | migration/transaction/restart tests; delete storage metadata module |
| `126-spool` | `parallax-spool`; crash/durability boundary + independent recovery/fuzz surface | server receivers/worker, CLI doctor | API, Greptime, Turso | frame/rotation/recovery/allocation tests; delete storage spool module |

`parallax-storage` remains the T1 capability facade and query-neutral contract
owner. Server is the only production crate permitted to instantiate the three
concrete adapters. Evidence orchestration receives typed, already-read inputs.

## Slice order

1. Ingest, analysis, evidence leaves with their external test modules.
2. Greptime, metadata, and spool adapters, preserving port conformance.
3. Move construction to server, migrate consumers, and delete compatibility
   reexports immediately after the last use.
4. Keep the temporary semantic-convention constants with pure analysis until
   Plan 119 creates generated `parallax-semconv`.

Every slice updates Cargo metadata, architecture tiers, facade manifests,
crate README, structural ratchets, `PROJECT_STRUCTURE.md`, and behavior tests
in the same commit.

## Execution state

- `126-ingest`: complete at `01f955e`; server owns the direct normalization
  edge and the hot-path clone floor remains enforced.
- `126-analysis`: complete at `8872100`; API, server, and evidence consume the
  pure analysis facade directly.
- `126-evidence`: complete in the following commit; schema, redaction,
  bounding, canonical-hash, story, gap, and baseline tests moved with the
  facade. The final compatibility-shell consumer moved and `parallax-core` was
  deleted.
