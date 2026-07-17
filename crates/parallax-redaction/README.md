+++
schema_version = 1
package = "parallax-redaction"
class = "product"
tier = 1
dependencies = []
facade_roots = ["lib.rs"]
+++

# parallax-redaction

Owns the versioned secret-detector engine and the default-deny source
policy every agent-visible or persisted text passes through. Extracted from
`parallax-evidence` so persistence-tier crates sanitize at write time
without inverting the dependency graph.

## Owned concerns

Detector rules and replacement markers, control-character stripping, the
`RedactionReport` hit accounting, and the typed evidence-field source
policy (`decide`/`project_text`/`sanitize_text`).

## Source map

- [src/lib.rs](src/lib.rs)
- [Reviewed facade manifest](facade.toml)

## Public surface

The supported `lib.rs` paths are the exports recorded in the
[reviewed facade manifest](facade.toml). Detector rule identifiers are wire
values inside redaction markers and must never be renamed casually.

## Verification

Run `cargo test -p parallax-redaction` for the narrow crate gate and
`cargo xtask facade check` for root-surface drift.
