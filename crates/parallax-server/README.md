+++
schema_version = 1
package = "parallax-server"
class = "product"
tier = 4
dependencies = ["parallax-api", "parallax-core", "parallax-proto", "parallax-storage"]
facade_roots = ["lib.rs"]
+++

# parallax-server

Composes mandatory GreptimeDB and Turso engines, OTLP receivers, workers,
GraphQL hosting, engine supervision, and optional self-telemetry export.
