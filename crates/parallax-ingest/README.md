+++
schema_version = 1
package = "parallax-ingest"
class = "product"
tier = 1
dependencies = ["parallax-model", "parallax-proto"]
facade_roots = ["lib.rs"]
+++

# parallax-ingest

Owns the zero-copy OTLP-to-domain normalization boundary. It accepts decoded
wire ownership and emits `parallax-model` values without storage, API, or
evidence dependencies.
