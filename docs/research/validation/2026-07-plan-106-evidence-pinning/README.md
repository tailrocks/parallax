# Plan 106 — Evidence pinning spike (GO)

**Status:** DONE (2026-07-17)

## Decision

GO — pin sanitized `bundle-v2` JSON in Turso.
See [evidence-pinning.md](../../decisions/evidence-pinning.md).

## Implementation

- Table `evidence_pins` (anchor, schema_version, hash, payload, bounds, expiry)
- `evidence_pin_upsert` / `evidence_pin` / `evidence_pins_for_anchor` / `evidence_pin_delete`
- Soft max 512 KiB; oversize refused
- Unit test: round-trip, idempotent upsert, delete, bound

## Verify

```sh
cargo nextest run --locked -p parallax-metadata -E 'test(/pin/)'
```

## Live Greptime

`SHOW CREATE TABLE opentelemetry_logs` on the QA stack shows native TTL-capable
table options; pin storage intentionally does not copy those rows.
