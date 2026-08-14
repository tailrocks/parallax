# Active Implementation Plans

`plans/` holds only future product or engineering work we still intend to
implement or improve. Completed work lives in Git history and, when useful,
under `docs/research/validation/`. Do not keep plan history here.

## Active

| Plan | Title | Status |
|------|-------|--------|
| [089](089-extension-table-grpc-writes.md) | Move derived extension-table writes to GreptimeDB's row API | BLOCKED — crates.io `greptimedb-ingester` still 0.18.0; upstream [PR #58](https://github.com/GreptimeTeam/greptimedb-ingester-rust/pull/58) OPEN; HTTP SQL path remains |
| [114](114-retire-legacy-spool-reader.md) | Retire the legacy NDJSON spool reader | BLOCKED — only rolling `preview` tag; need one stable raw-frame release cycle + expired legacy segments |
| [171](171-competitor-informed-feature-uplift.md) | Preview-before-save, agent issue lease, MCP evals, instrumented onboarding | OPERATOR-GATED — f1 preview shipped; f2 lease catalog proposal awaiting operator; f3 evals label-gated; f4 snippets landed |
| [176](176-grouping-transparency.md) | Grouping transparency — explain why events grouped; steering spec gated | OPERATOR-GATED — Steps 1–3 shipped; Step 4 fingerprint-rules proposal awaits operator |

Shipped 2026-08-15 (deleted from this folder): 162–167 playground verification
(SigNoz Foundry residue only; 4-sink + c-series + coverage matrix + W5
in-repo display fixes on `main`); 168–170 QA waves; 172 design system;
173–175 evidence alerts / durable upgrade / footprint.

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
