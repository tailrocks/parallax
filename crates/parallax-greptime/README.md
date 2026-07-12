+++
schema_version = 1
package = "parallax-greptime"
class = "product"
tier = 2
dependencies = ["parallax-model", "parallax-proto", "parallax-storage"]
facade_roots = ["lib.rs"]
+++

# parallax-greptime

Owns GreptimeDB HTTP/Arrow transport, native OTLP table SQL, migrations, row
mapping, and the concrete implementation of telemetry storage capabilities.
