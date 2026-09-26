+++
schema_version = 1
package = "parallax-sentry-proxy"
class = "aux"
dependencies = ["parallax-ingest"]
facade_roots = ["lib.rs", "main.rs"]
+++

# parallax-sentry-proxy

Optional Sentry-compatible ingress proxy for Selene. It validates envelope
framing, routes configured ingress projects to enabled destinations, and sends
each destination kind through a shared bounded worker queue. The HTTP surface
accepts `POST /api/{project_id}/envelope/` and
`POST /api/{project_id}/envelope`, plus legacy
`POST /api/{project_id}/store/`. This remains a scaffold; durable delivery,
circuit breaking, and rate-limit handling are not provided.

## Owned concerns

- Load and validate the TOML proxy configuration and ingress DSNs.
- Accept envelope and legacy store requests, and expose `/health`.
- Fan out envelopes to configured Sentry-compatible destinations.

## Source map

- [src/lib.rs](src/lib.rs)
- [src/main.rs](src/main.rs)
- [facade.toml](facade.toml)
- [Example configuration](config.toml.example)
- [Container build](Dockerfile)

## Public surface

The supported library roots and binary entry point are recorded in the
[reviewed facade manifest](facade.toml). Implementation modules are not a
compatibility surface.

## Verification

Run `cargo nextest run -p parallax-sentry-proxy --all-features --locked` for
the crate gate and `cargo xtask facade check` for root-surface drift.
