# Semantic Convention Registry Design

Checked: 2026-07-09

> **Status (2026-07-17): implemented without Weaver in the generation path.**
> The Weaver investigation and candidate layout below are historical evidence.
> Current source of truth is `telemetry/semconv/contract.yaml`; custom deterministic
> generation via `cargo xtask semconv generate` emits Rust, TypeScript, and Java
> constants. `parallax-semconv` owns Rust constants. No active plan owns the
> already-shipped generator.

## Current Weaver State

OpenTelemetry Weaver latest release is `v0.24.2`, published 2026-06-23:
https://github.com/open-telemetry/weaver/releases/tag/v0.24.2

Weaver manages semantic-convention registries through the `weaver registry`
command family: check, generate, diff, documentation, example-signal emission,
live checking, schema inference, packaging, and MCP serving:
https://github.com/open-telemetry/weaver/blob/main/docs/usage.md

Registry files are YAML. The v2 syntax supports groups for spans, events,
metrics, entities, and attribute groups, with `imports` blocks that pull
selected definitions from other registries:
https://github.com/open-telemetry/weaver/blob/main/schemas/semconv-syntax.v2.md

Custom registry validation is already a first-class command:

```bash
weaver registry check -r <path-to-registry>
```

Source: https://github.com/open-telemetry/weaver/blob/main/docs/define-your-own-telemetry-schema.md

Code generation exists through templates:

```bash
weaver registry generate -r my_registry --template code/java
```

The default-template spec names Java codegen packages, markdown docs, and
policy packages. Weaver's own live-check dogfood uses the `otel/weaver:v0.24.2`
Docker image with custom templates to generate Rust constants. Template roots
are directory based and include examples for `go`, `html`, `markdown`, and
`rust`; TypeScript would be a custom template today.

Sources:
- https://github.com/open-telemetry/weaver/blob/main/docs/specs/default-templates/default_templates.md
- https://github.com/open-telemetry/weaver/blob/main/crates/weaver_live_check/docs/dog-fooding.md
- https://github.com/open-telemetry/weaver/blob/main/crates/weaver_forge/README.md

## Candidate Registry Layout

The research evaluated one repository-owned registry, probably under
`telemetry/semconv/registry`, with these files:

```text
registry/
  attributes/parallax.yaml
  events/playground.yaml
  metrics/playground.yaml
  spans/execution-stack.yaml
  registry_manifest.yaml
templates/
  rust/
  java/
  typescript/
```

Registry content:

- Import standard OTel resource attributes: `service.*`,
  `deployment.environment.name`, `exception.*`, `event.name`, `session.id`.
- Define Parallax overlay attributes: `parallax.run.id`,
  `parallax.session.id`, `parallax.execution.layer`, `parallax.agent.id`.
- Define agent/tool attributes: `gen_ai.operation.name`, `tool.name`,
  `shell.command`.
- Define playground event names: `web.checkout.submitted`,
  `catalog.products.served`, `payment.authorized`.
- Define UI/browser attributes: `app.screen.name`, `app.widget.name`,
  `telemetry.propagation.disabled`, `web_vital.*`.
- Define metric names: `catalog.product.queries`, Tokio runtime gauges, and
  service overview lookup candidates.

## Historical Codegen Integration Estimate

Rust:

- Generate a dependency-free `parallax-semconv` leaf crate plus the companion
  `libs/playground-telemetry/src/semconv.rs` output. The historical
  `parallax-proto/src/semconv.rs` compatibility re-export was removed after all
  Rust consumers migrated directly to the leaf crate.
- Use `build.rs` or a checked-in generated file. Checked-in output is safer for
  this repo because CI and local dev should not need Docker or Weaver installed.
- Tracing macro field names like `otel.kind` and quoted dotted fields still need
  literal syntax at macro definition sites unless wrapped by local macros.

Java:

- Generate package-local `Semconv.java` files for services, or one small shared
  Java source set. Today's plan uses package-local classes; Weaver can remove
  that duplication.

TypeScript:

- Generate `web/src/semconv.ts` from a custom template. No default TypeScript
  target was evident in current Weaver docs, so this needs a local template.

Validation:

- Add `weaver registry check -r telemetry/semconv/registry` after registry
  exists.
- Keep existing Rust/TS/Java freeze tests until generated files are proven.

## Decision And Implemented Contract

The recommendation was implemented on 2026-07-15. The repository-owned
`telemetry/semconv/contract.yaml` is the single input to a deterministic xtask
generator. It emits the dependency-free `parallax-semconv` crate and the
checked-in TypeScript, companion Rust, and companion Java constants; ordinary
product builds consume those files without Weaver, Docker, or network access.

`cargo xtask semconv check` validates the overlay, exercises
invalid-schema fixtures, regenerates into temporary storage, compares every
artifact byte-for-byte, rejects legacy Rust ownership and mutable playground
wire literals, and supports the versioned JSON diagnostic envelope. The shared
wire-contract fixture is executed by TypeScript and Java consumers, while a
Rust OTLP protobuf round trip freezes representative emitted names and values.
Registry changes route to both Rust and UI CI lanes and run the same read-only
check without requiring Weaver. The companion
repository's native Rust, Bun, and Gradle gates compile and test the checked-in
outputs independently of generation.

The implementation intentionally uses repository-owned renderers instead of
Weaver: the small local
renderers give all three target languages one reviewed, deterministic contract
without coupling product builds to an external generator lifecycle.
