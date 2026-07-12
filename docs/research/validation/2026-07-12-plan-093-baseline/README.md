# Plan 093 behavioral baseline

- **Captured:** 2026-07-12
- **Clean source commit:** `1812e20ed1dd33080f3c76180a45a37e4c74bc7b`
- **Characterization commit:** `22434df`
- **Schema:** `parallax.plan-093-baseline/v1`

This packet freezes the pre-restructuring product contracts. `baseline.json`
is the machine-readable index; `graphql-schema.graphql` is the exact Juniper
SDL; `defect-ledger.json` links escaped defects to exact tests and gates. The
two JSON documents validate against the adjacent Draft 2020-12 schemas through
`parallax-core`'s `plan_093_baseline_artifacts_validate` test.

## Reproduction

```sh
mise exec -- rustc --version
mise exec -- cargo --version
mise exec -- bun --version
mise exec -- cargo metadata --format-version 1 --no-deps
mise exec -- cargo nextest list --workspace --message-format json
mise exec -- cargo run -q -p parallax-api --example schema_sdl
mise exec -- cargo run -q -p parallax-api --example schema_sdl | sha256sum
mise exec -- cargo test -p parallax-core plan_093_baseline_artifacts_validate
mise exec -- cargo nextest run --workspace --no-fail-fast
cd ui && mise exec -- bun run --bun --no-install test:ci
```

Every generated artifact names its schema/version and generating command in
`baseline.json`. The SDL exporter is intentionally tiny and reusable so future
plans compare exact output rather than a hand-maintained schema copy.

## Acceptance reconciliation

The previous acceptance prose promised ClickHouse-shaped query and DataLoader
batch spans but had only PostgreSQL and resolver evidence. Commit `22434df`
extends `m3_scenarios` with both missing shapes and exact attribute assertions;
the contract remains intact rather than being narrowed.

The V1 claims corrected in commit `1812e20` are intentionally not restored:

- JSON and Markdown are the bundle projections; Markdown is terminal-facing.
- OTLP profiles are not a V1 signal until a native GreptimeDB path and operator
  scope exist.
- a distinct metrics CLI is owned by Plan 105.
- truthful retention and physical `prune` behavior are owned by Plan 116.
- MemoryStore is test support, never a product/config mode.

## UI characterization

The forced-Bun lane initially exposed invalid externalization of Zod's
conditional ESM exports (`z` was undefined in 17 suites). Vitest's documented
last-resort dependency-inlining control now inlines `zod`; all 41 files and 175
tests pass. Four jsdom `Window.scrollTo()` not-implemented warnings remain
explicit Plan 129 characterization work. The risk rows in `baseline.json` map
route/search, GraphQL data, cache, SSE, and search owners to current tests and
named follow-up plans.

## Worker replay oracle

The current worker retries the whole combined operation. The characterization
test proves, without changing semantics:

| Failure after | Broadcast batches | Stored spans | Issue occurrences | Error rows | Run rows |
| --- | ---: | ---: | ---: | ---: | ---: |
| registration | 1 | 1 | 1 | 1 | 1 |
| live broadcast | 2 | 1 | 1 | 1 | 1 |
| telemetry storage | 2 | 2 | 1 | 1 | 1 |
| issue recording | 2 | 2 | 2 | 2 | 1 |

Plan 099 owns typed errors and idempotency changes. Until then, this table is an
oracle: restructuring may not silently alter it.
