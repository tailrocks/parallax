+++
schema_version = 1
package = "parallax-storage"
class = "product"
tier = 1
dependencies = ["parallax-model", "parallax-proto"]
facade_roots = ["lib.rs"]
+++

# parallax-storage

Owns query-neutral telemetry and metadata capability contracts plus their pure
shared selection and aggregation rules. Concrete engines live in adapter crates.
