# Rust Workspace Map

Research date: 2026-07-12

This is the semantic map of the Cargo workspace. Crate READMEs own the detailed
source map and verification command; `facade.toml` owns the checked root
surface.

## Dependency direction

```text
Tier 5  parallax-cli
           |
Tier 4  parallax-server
           |
Tier 3  parallax-api
        /          \
Tier 2  evidence   concrete adapters (GreptimeDB, Turso, spool)
        \          /
Tier 1  analysis, ingest, redaction, storage capability ports
          \       /
Tier 0    model, proto
```

Dependencies point downward. `parallax-test-support` is dev-only,
`parallax-xtask` is repository-only, and `parallax-mcp-spike` is an isolated
proof.

## Product crates

| Tier | Crate | Owned responsibility | Facade |
| ---: | --- | --- | --- |
| 0 | [parallax-model](../../../crates/parallax-model/README.md) | Query-neutral telemetry records and value types | `lib.rs` |
| 0 | [parallax-proto](../../../crates/parallax-proto/README.md) | OTLP wire/service aliases and semantic conventions | `lib.rs` |
| 1 | [parallax-analysis](../../../crates/parallax-analysis/README.md) | Fingerprints, error derivation, span events, trace comparison/path analysis | `lib.rs` |
| 1 | [parallax-ingest](../../../crates/parallax-ingest/README.md) | Signal-specific zero-copy normalization | `lib.rs` |
| 1 | [parallax-redaction](../../../crates/parallax-redaction/README.md) | Secret-detector engine and default-deny source policy | `lib.rs` |
| 1 | [parallax-storage](../../../crates/parallax-storage/README.md) | Telemetry and metadata capability contracts | `lib.rs` |
| 2 | [parallax-evidence](../../../crates/parallax-evidence/README.md) | Bounded/redacted/ranked evidence and agent projections | `lib.rs` |
| 2 | [parallax-greptime](../../../crates/parallax-greptime/README.md) | GreptimeDB native-table telemetry adapter | `lib.rs` |
| 2 | [parallax-metadata](../../../crates/parallax-metadata/README.md) | Turso metadata adapter | `lib.rs` |
| 2 | [parallax-spool](../../../crates/parallax-spool/README.md) | Raw-frame ingest durability | `lib.rs` |
| 3 | [parallax-api](../../../crates/parallax-api/README.md) | GraphQL schema, resolvers, and request batching | `lib.rs` |
| 4 | [parallax-server](../../../crates/parallax-server/README.md) | Engine/receiver/worker/API/UI runtime composition | `lib.rs` |
| 5 | [parallax-cli](../../../crates/parallax-cli/README.md) | Installed CLI and final binary composition | `main.rs` |

## Non-product crates

| Class | Crate | Boundary |
| --- | --- | --- |
| Test support | [parallax-test-support](../../../crates/parallax-test-support/README.md) | Reusable fakes/builders/conformance; unreachable from release roots |
| Auxiliary | [parallax-xtask](../../../crates/parallax-xtask/README.md) | Repository policy and CI control plane |
| Proof | [parallax-mcp-spike](../../../crates/parallax-mcp-spike/README.md) | MCP projection experiment; not packaged |

## Machine-owned contracts

- `cargo xtask facade check` checks every root against its
  `crates/*/facade.toml`.
- `cargo xtask policy` checks Cargo class/tier/dependencies, README source
  links and roots, architectural direction, and structural ratchets.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
  checks the sealed visibility surface, including `unreachable_pub`.

