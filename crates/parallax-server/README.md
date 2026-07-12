+++
schema_version = 1
package = "parallax-server"
class = "product"
tier = 4
dependencies = ["parallax-analysis", "parallax-api", "parallax-ingest", "parallax-proto", "parallax-storage", "parallax-test-support"]
facade_roots = ["lib.rs"]
+++

# parallax-server

Composes mandatory GreptimeDB and Turso engines, OTLP receivers, workers,
GraphQL hosting, engine supervision, and optional self-telemetry export.
