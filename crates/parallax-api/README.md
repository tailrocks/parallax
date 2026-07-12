+++
schema_version = 1
package = "parallax-api"
class = "product"
tier = 3
dependencies = ["parallax-core", "parallax-storage", "parallax-test-support"]
facade_roots = ["lib.rs"]
+++

# parallax-api

Owns the GraphQL schema, resolvers, batching boundaries, and API error
projection over domain and storage contracts.
