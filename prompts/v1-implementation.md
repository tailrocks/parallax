# V1 Implementation Brief — Parallax Local-First Release

Implement Parallax V1: the self-sufficient local-machine observability tool, exactly as the
repository's V1 documents specify. The mission is complete when the acceptance scenarios in
`docs/research/architecture/v1-scope.md` §1–§2 pass on a real machine — not when code merely
exists.

## Authoritative documents (read in this order, follow them)

1. `docs/research/architecture/v1-scope.md` — what V1 is; the acceptance scenarios are the stop
   condition.
2. `docs/research/architecture/v1-build-plan.md` — the milestone order: M0 (OTLP ingest
   skeleton) → M1 (managed GreptimeDB + Turso + error derivation/grouping) → M2 (GraphQL API +
   bundles + run model + CLI) → M2.5-UI (the web UI) → packaging/self-sufficiency.
3. `docs/research/architecture/v1-implementation-spec.md` — the concrete contracts: workspace
   conventions, pinned dependencies, ports (Parallax :4000/:4317/:4318; managed GreptimeDB child
   shifted to 24000–24003), `config.toml` keys, GreptimeDB DDL, Turso DDL, the OTLP→column
   mapping, the GraphQL SDL, the CLI output contract, and the GreptimeDB supervision contract.
   Contract changes go to this file first, then code.
4. `docs/research/architecture/simple-ui-v2.md` — the V1 UI specification (TanStack Start +
   shadcn/ui on Base UI, default theme as-is, shadcn charts/blocks; Sentry-grade issues;
   predefined + user-defined dashboards; trace lookup by trace_id and run_id; the
   interactivity rule).
5. `docs/research/capture/rust-stack-instrumentation.md` — what telemetry arrives and how;
   drives the quickstart docs and the SDK-driven integration tests.

Domain semantics (derivation, fingerprinting, bundle assembly, bounding, redaction-lite,
hypotheses) graduate from `poc/evidence-loop/` per
`docs/research/architecture/poc-evidence-loop-coverage.md` — copy-and-adapt the logic with
tests; leave `poc/` itself frozen.

## Version policy (operator, 2026-06-12)

Always use the **latest stable versions everywhere** — crates, the GreptimeDB engine, TanStack
Start, shadcn components, toolchain. The implementation spec's dependency table is a
known-compatible floor, not a freeze: resolve the latest **mutually-compatible** stable set at
start (the OTel ecosystem moves in lockstep release trains — never mix trains), update the
spec's table to the resolved set in the same commit, and repeat the resolution whenever
dependencies are touched.

## Constraints

- One canonical API: CLI and UI consume the GraphQL/HTTP API only; nothing but
  `parallax-storage` adapters touches GreptimeDB or Turso.
- Engine-specific SQL lives only inside the GreptimeDB adapter; the in-memory adapter backs
  fast tests.
- No authentication in V1 (loopback by default). No server/cloud profiles, no MCP, no Sentry
  endpoint, no trigger/dispatch machinery — V2+ per the scope's out-table.
- Apache-2.0 headers/metadata; company name Tailrocks; Conventional Commits with the agent
  attribution trailer per `COMMITS.md`; commit and push after each coherent step.
- Update `PROJECT_STRUCTURE.md` when `crates/` and `ui/` appear; keep claim discipline — no
  product claims beyond what the gate ledgers allow; measured numbers (setup time, freshness,
  bundle latency) go into the gate documents when collected.
- When a needed decision is missing from the spec, make the smallest reasonable choice,
  implement it, and record it in `v1-implementation-spec.md` in the same commit.

## Per-pass proof of progress

Each working session: state the milestone being advanced, land compiling code with passing
tests (`cargo clippy -D warnings`, `cargo fmt --check`, `cargo test --workspace`), commit, push,
and name the next concrete gap. Integration tests emit telemetry through the real
tracing/opentelemetry-otlp stack against the in-process server.

## Stop condition

All v1-scope acceptance scenarios pass end-to-end on macOS (arm64 first): the cold-start
quickstart (<15 min to first evidence), the panic→grouped-issue→bundle→agent flow, the six
stack-shaped scenarios (cross-service gRPC trace; tokio-postgres and clickhouse span
visibility; manual GraphQL spans rendered; visual→agent handoff via trace/run ID; a saved
custom dashboard rendering a user metric; a TUI run captured with OTLP-only logging), and
`parallax doctor`/`prune`/`uninstall --purge` behave as specified. Then stop and report what
was measured against the M5 gate targets.
