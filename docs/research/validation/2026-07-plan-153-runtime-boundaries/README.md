# Plan 153 — Non-GraphQL runtime boundary foundation

**Recorded:** 2026-07-17  
**Scope:** Unknown-first decode/diagnostic primitives; hardened SSE/visibility;
search + versioned storage mechanisms; first-consumer policy for
environment/window-message (fixtures via placement docs, no dead production
modules). Does not migrate product search/storage/SSE schemas (feature plans).

## Delivered

1. **`platform/external-values`** — `RuntimeDecoder`, `BoundaryResult`,
   `BoundaryError`, silent injectable diagnostic sink, `decodeJsonText`.
2. **SSE** — `event-source.client.ts`, `live-stream-controller.ts`, hardened
   `use-live-stream` (legacy `parse` path preserved for 140-142).
3. **Visibility** — `platform/visibility/*` (browser reexport kept).
4. **URL** — `decodeSearchValue` for feature-owned search schemas.
5. **Storage** — `browser-storage` + `versioned-storage-codec` (injectable
   Storage doubles; no silent corrupt rewrite/delete).

## Verification

```bash
cd ui && bun run typecheck && bun run lint && bun run check
cd ui && bun run --bun test:ci -- src/platform
cargo xtask policy --only ui.architecture
cargo xtask policy --only ui.ratchets
```

**Verified SHA:** `c8b1a42fb71aeb2884bb2cc523e1f4da051d0a5c` on `main`.
