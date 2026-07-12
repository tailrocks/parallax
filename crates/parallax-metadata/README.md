+++
schema_version = 1
package = "parallax-metadata"
class = "product"
tier = 2
dependencies = ["parallax-model", "parallax-proto", "parallax-storage"]
facade_roots = ["lib.rs"]
+++

# parallax-metadata

Owns Turso connection management, schema migrations, transactions, and row
mapping for mutable Parallax product metadata.
