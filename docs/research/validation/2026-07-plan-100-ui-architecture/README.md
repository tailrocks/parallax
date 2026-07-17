# Plan 100 — UI feature architecture foundation

**Recorded:** 2026-07-17  
**Scope:** TypeScript layer graph, ownership ledger, provisional platform/domain
foundations, placement policy. Does not migrate product features (134-143,
149-150) or harden runtime decode (152/153).

## Delivered

1. **Ownership ledger** — every handwritten file under `ui/src`, `ui/tests`, and
   `ui/scripts` has exactly one `[[ui.ownership]]` row in `ratchet.toml` with
   `current_owner`, `target_owner`, `migration_plan`, and `kind`. Unclassified
   or stale paths fail `ui.architecture.ownership.*`.
2. **Policy families**
   - `cargo xtask policy --only ui.architecture` — Oxc layer/facade/cycle graph
     + ownership ledger + feature-edge/layer-exception contracts.
   - `cargo xtask policy --only ui.ratchets` — TypeScript over-budget ceilings
     (shrink-only) from live health measurements.
3. **Provisional lower layers**
   - `ui/src/platform/graphql/transport.ts` — GraphQL fetch/cache/`gqlString`
     (Plan 152 hardens decode/codegen).
   - `ui/src/platform/sse/use-live-stream.ts` — EventSource consumer
     (Plan 153 hardens frame decode/diagnostics).
   - `ui/src/platform/browser/use-page-visible.ts`, `use-is-mobile.ts`
     (Plan 153 hardens environment edges).
   - `ui/src/domain/time-range/range.ts` — pure range values
     (Plan 149 owns RangePicker; Plan 153 hardens URL search).
4. **Compatibility reexports** (`kind = "compatibility-reexport"`) at the old
   `lib/`/`hooks/` paths so feature routes keep working until their migration
   plans switch imports. Removal plans are named on each row.
5. **Durable placement rules** in `ui/AGENTS.md` (rules 25-32) and
   `PROJECT_STRUCTURE.md`.

## Handoffs

| Concern | Owner after Plan 100 |
|---------|----------------------|
| Generated GraphQL documents/schemas/decoded transport | Plan 152 |
| SSE/search/storage/environment unknown-first decode | Plan 153 |
| Route-less capabilities (runtime-metrics, story, time-range UI, page-header) | Plan 149 |
| Product feature moves | Plans 134-142, 150 |
| App/layout/shell | Plan 143 |
| Final graph/zero-compatibility proof | Plan 151 |

## Verification (this commit)

```bash
cargo xtask policy --only ui.architecture
cargo xtask policy --only ui.ratchets
cargo xtask policy --only ui.tests
cargo xtask policy --only typescript
cd ui && bun run --bun test:ci -- src/platform src/domain/time-range
cd ui && bun run typecheck
```

Exact command output and SHA are recorded in the retirement commit message.
