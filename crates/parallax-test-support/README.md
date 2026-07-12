+++
schema_version = 1
package = "parallax-test-support"
class = "test-support"
dependencies = ["parallax-model", "parallax-proto", "parallax-storage"]
facade_roots = ["lib.rs"]
+++

# parallax-test-support

Owns reusable in-memory telemetry fakes, typed fixture builders, and shared
storage conformance scenarios. Product crates may consume it only as a dev
dependency; it is unreachable from release roots.
