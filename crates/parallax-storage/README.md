+++
schema_version = 1
package = "parallax-storage"
class = "product"
tier = 1
dependencies = ["parallax-model", "parallax-proto"]
facade_roots = ["lib.rs"]
+++

# parallax-storage

Owns telemetry and metadata capability contracts plus the current GreptimeDB,
Turso, spool, and test-only in-memory adapters during staged decomposition.
