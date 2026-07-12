+++
schema_version = 1
package = "parallax-core"
class = "product"
tier = 1
dependencies = ["parallax-proto", "parallax-storage"]
facade_roots = ["lib.rs"]
+++

# parallax-core

Migration facade for normalization, analysis, fingerprints, stories, and
evidence bundles. Plan 097 removes its temporary storage dependency.
