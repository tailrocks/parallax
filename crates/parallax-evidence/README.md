+++
schema_version = 1
package = "parallax-evidence"
class = "product"
tier = 2
dependencies = ["parallax-analysis", "parallax-model", "parallax-redaction"]
facade_roots = ["lib.rs"]
+++

# parallax-evidence

Pure evidence-domain logic: agent-session and story projection, evidence-gap
detection, and bounded, redacted evidence-bundle assembly.

## Owned concerns

Bounded evidence assembly, ranking, redaction, hashing, and projections.

## Source map

- [src/lib.rs](src/lib.rs)
- [src/bundle.rs](src/bundle.rs)
- [src/story.rs](src/story.rs)
- [Reviewed facade manifest](facade.toml)

## Public surface

The supported `lib.rs` paths are the exports recorded in the
[reviewed facade manifest](facade.toml); implementation modules are not a
compatibility surface.

## Verification

Run `cargo test -p parallax-evidence --all-features` for the narrow crate gate and `cargo xtask facade check` for
root-surface drift.
