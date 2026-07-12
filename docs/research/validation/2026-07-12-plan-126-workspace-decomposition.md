# Plan 126 workspace-decomposition closure

**Validated:** 2026-07-12
**Final implementation range:** `01f955e..54676c2`

## Result

The compatibility shell is gone and Cargo metadata matches the intended
domain/port/adapter/composition layering. All workspace packages inherit Rust
1.97.0, edition 2024, Apache-2.0, repository/authors/workspace lints, are
`publish = false`, and use version `0.1.0-dev`.

| Tier | Package | Normal internal dependencies |
|---|---|---|
| T0 | `parallax-proto` | — |
| T0 | `parallax-model` | — |
| T1 | `parallax-ingest` | model, proto |
| T1 | `parallax-analysis` | model, proto |
| T1 | `parallax-storage` | model, proto |
| T2 | `parallax-evidence` | analysis, model |
| T2 | `parallax-greptime` | model, proto, storage |
| T2 | `parallax-metadata` | model, proto, storage |
| T2 | `parallax-spool` | — |
| T3 | `parallax-api` | analysis, evidence, storage |
| T4 | `parallax-server` | analysis, API, Greptime, ingest, metadata, proto, spool, storage |
| T5 | `parallax-cli` | server |

`parallax-semconv` remains intentionally absent: Plan 119 owns its generated
registry and Plan 126 explicitly excluded creating it. Temporary constants are
kept with analysis/proto consumers without restoring an umbrella crate.

## Boundary proof

- `parallax-storage` contains capability traits and query-neutral contracts/
  rules only. Its normal external graph has no Arrow, reqwest, Turso, or spool
  implementation.
- API, analysis, ingest, and evidence normal dependency trees contain no Turso,
  reqwest, Arrow, Greptime adapter, metadata adapter, or spool crate.
- Server is the only production composition root constructing
  `GreptimeStore`, `TursoMetadataStore`, and `Spool`. API and workers receive
  `Arc<dyn TelemetryStore>` / `Arc<dyn MetadataStore>` capabilities.
- CLI's default and all-feature normal trees contain no test-support, xtask, or
  MCP-spike package. No rustls or webpki backend is active; default release uses
  native TLS and the cross-release feature uses vendored OpenSSL.
- Repository search finds no live `parallax-core`, old storage-adapter path, or
  old adapter import under `crates/`, `schema/`, or `PROJECT_STRUCTURE.md`.
- Facade manifests expose one reviewed adapter root each. Every moved hotspot
  retained its exact shrink-only bound under the new path; the metadata port
  implementation was split from the transferred Turso file instead of raising
  its Plan 098-owned ceiling.

## Compatibility and gate evidence

Commands below ran from the final worktree and exited zero:

```text
cargo xtask policy
cargo xtask facade check
cargo xtask dependencies --rust
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo nextest run --workspace
  238 passed, 6 intentionally ignored real-engine tests
cargo nextest run -p parallax-server --run-ignored only --no-capture
  6 passed: managed roundtrip, native-table inventory, metrics, M5 gates,
  restart conformance, and metric-exemplar migration
cargo build --release -p parallax-cli --locked
cargo build --release -p parallax-cli --all-features --locked
```

The full test selection covers the moved normalization clone/serde contract,
analysis invariants, evidence schema/redaction/bounding/hash goldens, spool
framing/rotation/recovery, Turso migration/transaction behavior, GraphQL/CLI
contracts, SQL/Arrow goldens, and architecture negative fixtures for missing
tier, upward/build/aux edges, mixed cycles, feature edges, release
reachability, and stale exceptions.

The six-test real-engine run used managed GreptimeDB and also proved restart
against the same data directory. Its M5 measurements were: warm start 406 ms,
ingest-to-queryable p50 11 ms / p95 54 ms, panic-to-grouped-issue 30 ms, and
warm bundle p50 12 ms / p95 16 ms.

An isolated target directory measured the workspace all-target check at
51.36 seconds clean and 0.74 seconds incremental. The release default build
took 4m02s from its then-cold release cache; the all-feature native-TLS-vendored
increment took 1m35s. Decomposition adds independently cacheable leaves; it did
not hide a regression with a changed cache key or relaxed gate.
