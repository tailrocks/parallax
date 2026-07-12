# Semantic Convention Registry Design

Checked: 2026-07-09

> **Status (2026-07-12): design evidence, not an active plan or build
> checklist.** The Weaver investigation and candidate registry shape are retained
> here for reference. Numbered plan 119 in [`plans/`](../../../plans/) exclusively
> owns any registry, generated constants, or CI enforcement. Do not implement
> from this file.

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

- Generate a constants module matching
  `libs/playground-telemetry/src/semconv.rs` and
  `crates/parallax-proto/src/semconv.rs`.
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

## Decision And Implementation Ownership

The research recommendation was GO, subject to measured scope and repository
policy. Plan 119 is the sole execution queue for the YAML registry, checked-in
Rust/Java/TypeScript constants, regeneration/diff CI, and any later Java
deduplication. The shapes above are inputs to that plan, not authorization.
