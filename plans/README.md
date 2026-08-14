# Active Implementation Plans

`plans/` holds only future product or engineering work we still intend to
implement or improve. Completed work lives in Git history and, when useful,
under `docs/research/validation/`. Do not keep plan history here.

## Active

No numbered plans are active. External or operator-gated residuals live in
research, not here:

- row API / `greptimedb-ingester` — [docs/research/decisions/native-otel-tables.md](../docs/research/decisions/native-otel-tables.md)
- legacy NDJSON spool reader — [docs/research/decisions/native-otel-tables.md](../docs/research/decisions/native-otel-tables.md)
- competitor-informed lease (f2) — [docs/research/reference/feature-inventory-and-playground-verification.md](../docs/research/reference/feature-inventory-and-playground-verification.md)
- fingerprint-rules (Step 4) — [docs/research/decisions/fingerprint-rules.md](../docs/research/decisions/fingerprint-rules.md)

A numbered file returns here only when an operator or upstream gate lifts.

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
