# Active Implementation Plans

`plans/` holds only future product or engineering work we still intend to
implement or improve. Completed work lives in Git history and, when useful,
under `docs/research/validation/`. Do not keep plan history here.

## Active

| Plan | Title | Status |
|------|-------|--------|
| [089](089-extension-table-grpc-writes.md) | Move derived extension-table writes to GreptimeDB's row API | BLOCKED — crates.io `greptimedb-ingester` still 0.18.0; upstream [PR #58](https://github.com/GreptimeTeam/greptimedb-ingester-rust/pull/58) OPEN (recheck 2026-07-17T17:18Z); HTTP SQL path remains |
| [114](114-retire-legacy-spool-reader.md) | Retire the legacy NDJSON spool reader | BLOCKED — only rolling `preview` tag; need one stable raw-frame release cycle + expired legacy segments (recheck 2026-07-17T17:18Z) |

## Constraints

These leftovers must preserve:

- GreptimeDB + Turso only; no product fallback engine.
- GreptimeDB native raw-signal tables.
- Native TLS only; never an active rustls backend.
- Bun only for JavaScript/TypeScript.
- Decode once and move ownership on the ingest hot path.
- Apache-2.0 throughout.

## Lifecycle

1. Numbered file `plans/NNN-kebab-case.md` plus one index row.
2. File states residual, blocker or next step, done criteria, and STOP.
3. When the work ships or is dropped, delete the file and this row in the same
   commit. Do not archive DONE plans here.
