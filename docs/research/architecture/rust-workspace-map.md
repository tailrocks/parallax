# Rust Workspace Map

Research date: 2026-07-12; re-verified 2026-07-17 against source (17 members).

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
Tier 2  evidence   concrete adapters (GreptimeDB, Turso)
        \          /
Tier 1  analysis, ingest, redaction, storage capability ports
          \       /
Tier 0    model, proto, semconv
```

Dependencies point downward. `parallax-test-support` is dev-only,
`parallax-xtask` is repository-only, and `parallax-mcp` is an isolated
proof. `parallax-spool`, `parallax-redaction`, and `parallax-semconv` are true
leaves (no internal deps); `parallax-spool` sits conceptually at Tier 2
(raw-frame durability) despite having no internal crate dependency today.

## Product crates

| Tier | Crate | Owned responsibility | Facade |
| ---: | --- | --- | --- |
| 0 | [parallax-model](../../../crates/parallax-model/README.md) | Query-neutral normalized domain rows + stable value types (traces/logs/metrics/errors/issues/invocations/dashboards/investigations/saved views/test reporting) | `lib.rs` |
| 0 | [parallax-proto](../../../crates/parallax-proto/README.md) | Single pinned OTLP protocol surface — re-exports `opentelemetry-proto` tonic types | `lib.rs` |
| 0 | [parallax-semconv](../../../crates/parallax-semconv/README.md) | Generated semantic-convention name constants (producers + consumers); source `telemetry/semconv/contract.yaml`, regen `cargo xtask semconv generate` | `lib.rs` |
| 1 | [parallax-analysis](../../../crates/parallax-analysis/README.md) | Fingerprints, error derivation (traces/logs), span events, Drain log patterns, trace critical-path/compare, JUnit/nextest adaptation, test flakiness/reporting | `lib.rs` |
| 1 | [parallax-ingest](../../../crates/parallax-ingest/README.md) | OTLP normalization boundary (traces/logs/metrics) + Sentry envelope framing; `cli.invocation.id` signal-then-resource resolution | `lib.rs` |
| 1 | [parallax-redaction](../../../crates/parallax-redaction/README.md) | Secret-detector engine (`redaction-lite-v3`, 20 detectors) and default-deny source policy; applied at metadata/bundle projection, not on the ingest hot path | `lib.rs` |
| 1 | [parallax-storage](../../../crates/parallax-storage/README.md) | `TelemetryStore` + `MetadataStore` capability contracts, projections, deterministic prune planning | `lib.rs` |
| 2 | [parallax-evidence](../../../crates/parallax-evidence/README.md) | Bounded evidence-bundle assembly (`bundle-v1`, envelope v2), gap detection, story/agent-session/CI/deploy normalizers, source redaction policy | `lib.rs` |
| 2 | [parallax-greptime](../../../crates/parallax-greptime/README.md) | GreptimeDB native-table telemetry adapter (`GreptimeStore`): OTLP-forward write, SQL/arrow read, bootstrap/TTL reconcile | `lib.rs` |
| 2 | [parallax-metadata](../../../crates/parallax-metadata/README.md) | Turso (`TursoMetadataStore`) mutable metadata + migrations: issues/invocations/dashboards/investigations/alerts/test reporting/evidence pins/deploys/sentry acks/prune journal | `lib.rs` |
| 2 | [parallax-spool](../../../crates/parallax-spool/README.md) | Raw OTLP/Sentry-frame ingest durability — forensic trail (PSPL1 framing), rotation, retention, reclaim | `lib.rs` |
| 3 | [parallax-api](../../../crates/parallax-api/README.md) | GraphQL surface (Juniper, code-first; 76 queries / 14 mutations / 0 subscriptions), resolvers, request memo/limits | `lib.rs` |
| 4 | [parallax-server](../../../crates/parallax-server/README.md) | OTLP ingest (gRPC `:4317` / HTTP), GraphQL host, GreptimeDB supervision, staged workers, live SSE, alerting, self-telemetry, UI mount | `lib.rs` |
| 5 | [parallax-cli](../../../crates/parallax-cli/README.md) | Installed `parallax` binary; embeds server (`serve`) + thin API client for all subcommands; `embed-ui`, `cross-release-vendored` features | `main.rs` |

## Non-product crates

| Class | Crate | Boundary |
| --- | --- | --- |
| Test support | [parallax-test-support](../../../crates/parallax-test-support/README.md) | Reusable fakes/builders/conformance; unreachable from release roots |
| Auxiliary | [parallax-xtask](../../../crates/parallax-xtask/README.md) | Repository policy and CI control plane |
| Proof | [parallax-mcp](../../../crates/parallax-mcp/README.md) | MCP projection experiment; not packaged |

## Machine-owned contracts

- `cargo xtask facade check` checks every root against its
  `crates/*/facade.toml`.
- `cargo xtask policy` checks Cargo class/tier/dependencies, README source
  links and roots, architectural direction, and structural ratchets.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
  checks the sealed visibility surface, including `unreachable_pub`.

