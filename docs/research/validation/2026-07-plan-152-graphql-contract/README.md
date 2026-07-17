# Plan 152 — GraphQL contract foundation

**Recorded:** 2026-07-17  
**Scope:** Deterministic SDL export, Bun GraphQL Code Generator pipeline,
unknown-first platform client, static probe, bounded dashboard widget-series
AST/alias contract. Does not migrate product static operations (134-143,
149-150).

## Delivered

1. **Authoritative SDL** — `parallax_api::export_schema_sdl()` +
   `cargo xtask ui graphql export|check` writing/checking
   `ui/graphql/schema.graphql` (byte-stable, LF, one trailing newline).
2. **Bun codegen** — exact packages under `ui/package.json`;
   `bun run graphql:generate` (`bunx --bun --no-install graphql-codegen`);
   checked-in `schema-types.generated.ts` + probe
   `static-probe.graphql|.generated.ts` with Zod v4 operation result schema.
3. **Platform client** — `executeGraphqlOperation` /
   `executeCachedGraphqlOperation` accept TypedDocumentNode + Zod schema +
   variables; send `{ operationName, query, variables }`; decode envelope then
   data once; secret-safe `GraphqlBoundaryError`.
4. **Legacy transport preserved** — raw `graphql`/`graphqlCached`/`gqlString`
   remain for feature-migration handoffs.
5. **Dynamic dashboard exception** —
   `features/dashboards/api/widget-series-*` AST builder (≤24 aliases,
   `series_<ordinal>`, variables-only), alias-set decode, route seam
   delegated.

## Verification

```bash
cargo xtask ui graphql export
cargo xtask ui graphql check
cd ui && bun run graphql:generate
cd ui && bun run typecheck && bun run lint && bun run check
cd ui && bun run --bun test:ci -- src/platform/graphql src/features/dashboards/tests/api src/routes/__tests__/-dashboards.test.tsx
cargo xtask policy --only ui.architecture
cargo xtask policy --only ui.ratchets
```

**Verified SHA:** `c8b1a42fb71aeb2884bb2cc523e1f4da051d0a5c` on `main`.
