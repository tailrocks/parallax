+++
schema_version = 1
package = "parallax-analysis"
class = "product"
tier = 1
dependencies = ["parallax-model", "parallax-proto"]
facade_roots = ["lib.rs"]
+++

# parallax-analysis

Owns pure error derivation, deterministic fingerprints, span-event parsing,
trace comparison, and critical-path analysis. It has no ingest, storage,
transport, API, or runtime dependency.
