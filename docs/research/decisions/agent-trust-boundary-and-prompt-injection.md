# Agent trust boundary and prompt-injection containment

Decision date: 2026-07-13  
Status: accepted architecture constraint

## Decision

Human-local raw surfaces and agent context are separate trust domains. The UI
and an explicitly operated local CLI may use raw GraphQL, SQL, and live SSE.
Product agent/MCP transports may consume only the bounded, redacted,
schema-valid canonical bundle projection owned by `parallax-evidence`.

An agent-visible transport must be classified in `ratchet.toml` with
`agent_context = true`. The architecture gate then permits production
dependencies only on `parallax-evidence` and `parallax-model`; dependencies on
`parallax-api`, `parallax-storage`, `parallax-server`, or an adapter are build
failures. This prevents an agent transport from acquiring raw query, live-tail,
or persistence capabilities through the workspace graph.

The name-based classification check is fail-closed for future product packages
whose names contain `agent` or `mcp`: they cannot enter the graph without the
marker. The current `parallax-mcp` remains class `proof`, is not packaged,
and cannot establish a product safety claim.

## Prompt-injection model

Telemetry strings are untrusted evidence, not instructions. Agent projections
must preserve them only inside the canonical schema after redaction and output
budgeting. Transport descriptions, tool results, resource contents, and error
messages must never promote log/span text into system or tool instructions.
Raw GraphQL (`/graphql`), trace/log SSE, SQL, raw storage traits, and source
envelopes are therefore outside the agent context domain even on localhost.

This is a containment boundary, not a claim that redaction alone detects every
prompt injection. Product MCP remains blocked on its separate ship gates for
schema validation, redaction fixtures, audit, authorization, cross-client
behavior, and adversarial evaluation.

## Enforcement evidence

`parallax-xtask` includes a negative architecture fixture proving that an
`agent_context` product can depend on `parallax-evidence` but fails when it
depends on another product capability. Removing the marker or adding a raw
workspace dependency makes `cargo xtask policy` fail.

The broader transport and claim-evidence contract remains in
[agent-access-surface.md](agent-access-surface.md).
