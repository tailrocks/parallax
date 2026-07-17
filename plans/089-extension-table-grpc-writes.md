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
- **Status**: BLOCKED on upstream release — contribution submitted
  (2026-07-17): [greptimedb-ingester-rust#58](https://github.com/GreptimeTeam/greptimedb-ingester-rust/pull/58)
  makes rustls-backed TLS an opt-in `tls-ring` feature; the plaintext build
  was verified rustls-free (`cargo tree -i rustls` matches nothing) with all
  tests green on both feature states. Step 0 still waits for a released
  crate version carrying the split (or maintainer-preferred variant).
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

### Upstream contribution packet (2026-07-17)

The operator unblock directive converts the dependency wait into a fix-forward
upstream contribution. A fresh read-only scan found no existing upstream issue
or pull request and narrowed the first contribution to a plaintext feature
split:

- crates.io and upstream `main` (`86aaa15d0ebd152b46f4db581461c1e78968eb24`)
  remain at `greptimedb-ingester 0.18.0`; its manifest unconditionally enables
  tonic 0.14 `tls-ring`.
- Tonic 0.14.6 `tls-native-roots` is not native TLS: it still enables
  `tokio-rustls` and only changes root loading. Parallax must not use it.
- The minimal upstream patch makes TLS opt-in, compiles tonic's transport,
  codegen, gzip, and zstd surfaces without a TLS feature by default, and
  cfg-gates `Certificate`, `Identity`, `ClientTlsConfig`, `ClientTlsOption`,
  `ChannelManager::with_tls_config`, `Client::with_tls_and_urls`, and their
  TLS-only errors/tests/docs. The existing `http://` lazy-channel path then
  works without a behavioral rewrite.
- Required upstream gates are `cargo check` and tests with
  `--no-default-features`, plus a feature-graph assertion that rustls is
  absent. If upstream retains TLS compatibility, test it in a separate opt-in
  job; Parallax enables no TLS feature on the trusted local hop.
- A true native-TLS connector is separate follow-up work because tonic exposes
  no native-tls backend. It would use `native-tls` + `tokio-native-tls` through
  `Endpoint::connect_with_connector_lazy`, with hostname/SNI, system roots,
  client identity, custom CA, and platform CI coverage.
- Required row semantics already exist upstream: nanosecond timestamp values
  and JSON string values are supported by the row API.

This packet is preliminary implementation guidance for the upstream executor;
the executor must verify the current upstream head and extend the tests before
submission. It does not satisfy Step 0 until a released dependency graph used
by Parallax contains no active rustls backend.

- 2026-07-17 recheck: `cargo search greptimedb-ingester --limit 5` still reports
  **0.18.0** as latest. Published crate still hard-enables tonic `tls-ring`
  (rustls path). Step 0 conditions fail; plan remains blocked pending upstream
  native-TLS/plaintext feature. No product-code change.
- 2026-07-17 later recheck: crates.io still **0.18.0**. Upstream PR
  [greptimedb-ingester-rust#58](https://github.com/GreptimeTeam/greptimedb-ingester-rust/pull/58)
  remains **OPEN** (title check green; not merged). Step 0 still fails.


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
