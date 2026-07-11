# Portable evidence-bundle schemas

This directory holds the **shipped**, machine-readable contract for Parallax
evidence bundles — the canonical JSON bytes returned by:

- GraphQL `BundleOut.json` (`serde_json::to_string_pretty` over
  `parallax_core::bundle::Bundle`)
- `parallax issue context --format json` / `parallax run bundle --format json`
  (verbatim `json` field from the same GraphQL projection)

Frozen PoC schemas under `poc/evidence-loop/schema/` are concept references
only and are **not** this contract.

## Files

| File | Governs |
| --- | --- |
| [`evidence-bundle.v1.schema.json`](evidence-bundle.v1.schema.json) | `schema_version: "bundle-v1"` — the production Serialize shape of `crates/parallax-core/src/bundle.rs` |

Dialect: JSON Schema Draft 2020-12. The schema is **self-contained** (no remote
`$ref` resolution) so CI/conformance tests stay offline.

## Versioning policy

- **`bundle-v1` is additive-only.** New *optional* fields (or new keys inside
  open maps such as `redaction.redacted_counts`) are allowed without a major
  bump. Consumers must ignore unknown properties.
- **Renames, removals, type changes, or required-field additions require
  `bundle-v2`.** Ship a new file (`evidence-bundle.v2.schema.json`) beside v1;
  keep the v1 file forever so old consumers keep validating.
- The Rust constant `parallax_core::bundle::SCHEMA_VERSION` must match the
  schema's `schema_version` `const`.
- Enforcement: the conformance tests in `crates/parallax-core/src/bundle.rs`
  assemble representative bundles, serialize them the same way the API does,
  and validate against this file with the `jsonschema` crate. Every
  bundle-shape PR must update the schema and fixtures in the same commit.

## `additionalProperties` choice

Every object in the v1 schema leaves `additionalProperties` at the draft
default (**open** / allowed). Rationale:

- The research prose promised a closed envelope in places, but the *shipped*
  shape is still evolving under the additive-only rule above.
- Open objects + required field lists give consumers a floor without freezing
  additive keys mid-v1.
- Closed maps are expressed only where the type *is* a free-form map of known
  value type: `redaction.redacted_counts` uses
  `additionalProperties: { type: integer }` for its values, not a closed key
  set (rule names evolve with the redactor).

If a future major version wants a closed shape for a sub-object, set
`additionalProperties: false` on that `$defs` entry and document it here.

## Related

- Prose / research draft (not the validator source of truth):
  [`docs/research/architecture/evidence-bundle-schema.md`](../docs/research/architecture/evidence-bundle-schema.md)
- Assembly / hash / redaction implementation:
  [`crates/parallax-core/src/bundle.rs`](../crates/parallax-core/src/bundle.rs)
- PoC frozen schemas:
  [`poc/evidence-loop/schema/`](../poc/evidence-loop/schema/)
