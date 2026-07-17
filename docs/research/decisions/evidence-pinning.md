# Evidence pinning beyond telemetry TTL

- **Status:** Approved (GO)
- **Decision date:** 2026-07-17
- **Approved by:** alexey@chainargos.com (operator unblock directive + plan 104/116 closure)
- **Depends on:** plan 104 Option C, plan 116 lifecycle
- **Implementation status (2026-07-17):** Shipped. Plans 104 and 116 are
  closed historical dependencies.
- **Pass 79 recheck (2026-07-17):** Decision **GO still holds** against code:
  `evidence_pins` Turso path + prune protection (`parallax-metadata` prune
  excludes live pins; `source_state` / TTL do not delete pin rows). No primary
  source found this pass that would force raw OTLP copy or disabling native
  TTL. **Unproven residual:** large-pin economics, multi-operator `pinned_by`
  (plan 109), and cold-read of *native* rows after TTL when only pin remains
  (expected: pin is the surviving artifact).

## Decision (GO)

**V1 pin representation:** store the **sanitized, immutable `bundle-v2` JSON
bytes** (Option C envelope around the v1 dossier) in **Turso**, not a custom
raw-signal table and not a second observability store.

| Question | Answer |
|---|---|
| What is pinned? | Sanitized bundle-v2 JSON + schema_version + canonical_hash |
| Actor | Local operator (`pinned_by=local-operator` until plan 109) |
| Default retention | Optional `expires_at`; null = until explicit delete |
| Size bound | 512 KiB per pin (`EVIDENCE_PIN_MAX_BYTES`) |
| When native TTL expires | Pin remains; `source_state` may be marked `expired` later |
| Delete pin | Removes Turso row only; never mutates Greptime native tables |
| Hash/version | Store envelope `schema_version` and `canonical_hash` for verification |

## Live storage notes (2026-07-17)

- Greptime native tables (e.g. `opentelemetry_logs`) carry TTL via table options
  / `ALTER TABLE … SET 'ttl'`; expiry is asynchronous through compaction
  (plan 116 contract).
- Turso supports atomic metadata+payload upserts for pin rows under the same
  connection lock used by other metadata domains.
- Pinning does **not** disable native TTL and does **not** copy raw OTLP rows.

## Rejected alternatives

| Option | Why rejected |
|---|---|
| Copy raw spans/logs into custom tables | Violates native-table rule |
| Disable Greptime TTL for pinned traces | Silent retention footgun; violates contract |
| Object store without approved stack | Out of scope; needs separate decision |

## Implementation

`evidence_pins` Turso table + `TursoMetadataStore::evidence_pin_*` CRUD with
size bound and idempotent upsert are implemented. GraphQL and CLI expose the
same durable V1 pin contract.
