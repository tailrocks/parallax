# Plan 089: Move derived extension-table writes to GreptimeDB's row API

> **Executor instructions**: Recheck Step 0 first. Do not add or enable rustls,
> fork the dependency, change the storage stack, or weaken native-TLS policy.
> If the upstream blocker remains, refresh the evidence and leave this plan
> BLOCKED. Update `plans/README.md` with any state change.

## Status

- **Priority**: P2
- **Effort**: M
- **Risk**: MEDIUM
- **Depends on**: upstream `greptimedb-ingester` native-TLS/plaintext feature fix
- **Category**: storage / ingest performance
- **Planned at**: `e3e7997`, re-based from historical plan 089 on 2026-07-12
- **Status**: BLOCKED
- **Blocker**: Published `greptimedb-ingester` versions through 0.18.0
  hard-enable tonic `tls-ring`; no native-TLS/plaintext feature split exists.

## Why

Parallax writes derived `error_events`, `run_metric_points`, and
`metric_exemplars` rows through text SQL. GreptimeDB's row API is a better fit
for bounded typed batches, but every published `greptimedb-ingester` version
checked through 0.18.0 hard-enabled tonic `tls-ring`. That introduces rustls
even when Parallax uses a plaintext trusted local hop, violating repository
policy.

The independently actionable exemplar primary-key correction was split into
[Plan 092 closure](https://github.com/tailrocks/parallax/commit/953409b). This plan owns transport only.

## Current Evidence

- `crates/parallax-storage/src/greptime.rs` builds SQL `INSERT ... VALUES`
  batches for all three derived tables.
- The managed engine exposes a local gRPC port, while external configuration
  currently carries only the HTTP endpoint.
- Published `greptimedb-ingester 0.18.0` declares tonic with `tls-ring`, `gzip`,
  and `zstd` as required features.
- `cargo tree -i rustls` with that dependency resolves through tonic and
  tokio-rustls. Cargo cannot remove a dependency's required feature.
- 2026-07-14 fresh upstream check: `mise exec -- cargo search
  greptimedb-ingester --limit 5` still reports 0.18.0 as latest. Its published
  manifest declares `tonic = "0.14"` with required `tls-ring`, `gzip`, and
  `zstd` features; Tonic 0.14.6 maps `tls-ring` to `tokio-rustls/ring`.
  Therefore the native-TLS/plaintext feature split required by Step 0 is still
  absent and implementation cannot begin without violating policy.
- 2026-07-15 fresh upstream check at branch head `d82023a`: crates.io still
  reports `greptimedb-ingester 0.18.0` as latest. `cargo info` exposes no
  alternate transport feature, and the downloaded published manifest still
  requires tonic 0.14 with `tls-ring`; tonic 0.14.6 maps that feature directly
  to `tokio-rustls/ring`. The plaintext/native-TLS split remains absent, so
  Step 0 still stops before any manifest or product-code change.

## Scope

In scope after the blocker clears:

- Root and storage crate manifests.
- GreptimeStore connection and derived-row writers.
- Managed/external gRPC endpoint configuration.
- Bounded row batches, compression, and SQL parity tests.
- Documentation of the temporary SQL compatibility path.

Out of scope:

- Raw OTLP forwarding or custom raw-signal tables.
- Replacing GreptimeDB/Turso.
- rustls, a crate fork, or a second TLS trust policy.
- Exemplar schema correction, owned by plan 092.
- Bulk/streaming adoption before the row path is measured.

## Steps

### Step 0: Recheck upstream

Resolve the latest stable `greptimedb-ingester` and inspect its full feature
graph and source manifest. Required conditions:

1. Tonic is mutually compatible with the workspace.
2. Plaintext use does not activate rustls/ring/tokio-rustls.
3. Any TLS option uses native roots/native TLS, not rustls.
4. Row values support nanosecond timestamps and the `attributes` JSON shape,
   or a documented compatible value representation.

If any condition fails, record the exact version, manifest edge, upstream issue
or request, and recheck date in this plan/index, then STOP.

### Step 1: Add endpoint composition

- Managed mode derives the trusted local gRPC endpoint from the supervised
  engine ports.
- External mode requires an explicit gRPC endpoint when row writes are active.
- Product startup validates the endpoint and announces it in the ready banner.
- The client connects lazily so engine startup ordering remains observable.

### Step 2: Add typed bounded writers

- Route the three derived table writers through row batches.
- Keep ownership moving forward; do not clone decoded telemetry to satisfy the
  new API.
- Bound rows per call and total encoded bytes.
- Preserve timestamps, nullable values, JSON, and table names exactly.
- Retain a documented SQL compatibility switch for one release cycle only;
  this plan remains unfinished while that switch exists.

### Step 3: Prove parity and measure

Run the same real-engine fixtures through row and SQL modes. Assert table row
counts and values match for error bursts, run metric points, and exemplars.
Record batch latency/throughput as implementation evidence, not a product claim.

### Step 4: Remove the compatibility path

After one released row-path cycle has clean production/dogfood evidence, remove
the SQL switch and its text-row builder. Re-run real-engine failure, parity,
upgrade, and rollback procedures before deleting the path. Do not retire this
plan by replacing removal with a reminder or unnumbered follow-up.

## Test Plan

- Unit tests for row conversion and chunk boundaries.
- Managed and external endpoint validation tests.
- Gated real-Greptime parity for all three derived tables in both modes.
- Failure injection for unavailable gRPC with actionable error context.
- Full storage conformance and workspace tests.

## Done Criteria

- [ ] Latest ingester graph contains no active rustls backend.
- [ ] Derived row writes use the typed row API by default.
- [ ] Row batches are bounded by count and encoded size.
- [ ] SQL and row modes are value-identical in real-engine tests.
- [ ] Default Rust, strict Clippy, nextest, and storage integration gates pass.
- [ ] Ready output names the active gRPC endpoint without leaking credentials.
- [ ] The one-cycle SQL compatibility switch and text-row path are removed.

## STOP Conditions

- Any required dependency or feature activates rustls.
- Tonic major duplication or native-TLS policy cannot be resolved upstream.
- Row conversion cannot preserve timestamp or JSON semantics.
- Row-mode values diverge from the SQL oracle.
- The implementation requires a custom raw-signal table.

## Remove When

Delete this file and its index row after the row transport is green, its
one-cycle SQL compatibility path is removed, and durable evidence is recorded;
or after an operator decision permanently rejects the transport with SQL
reaffirmed as the supported path. A still-active upstream blocker is not a
terminal state.
