+++
schema_version = 1
package = "parallax-spool"
class = "product"
tier = 2
dependencies = []
facade_roots = ["lib.rs"]
+++

# parallax-spool

Owns raw OTLP frame append, framing, rotation, retention, and crash recovery.
It is an ingest durability boundary, never a fallback database.
