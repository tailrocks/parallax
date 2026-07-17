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
marker. The current `parallax-mcp` is class `aux` (local-stdio product surface,
plan 112 DONE) and depends only on `parallax-evidence`; remote MCP still needs
Plan 109 protected transport before it may claim a broader product safety
posture.

## Prompt-injection model

Telemetry strings are untrusted evidence, not instructions. Agent projections
must preserve them only inside the canonical schema after redaction and output
budgeting. Transport descriptions, tool results, resource contents, and error
messages must never promote log/span text into system or tool instructions.
Raw GraphQL (`/graphql`), trace/log SSE, SQL, raw storage traits, and source
envelopes are therefore outside the agent context domain even on localhost.

This is a containment boundary, not a claim that redaction alone detects every
prompt injection. Local-stdio MCP graduated plan 112 with closed tools, loopback
origin, and redaction/hash fail-closed paths; broader multi-client, remote, and
adversarial evaluation claims remain ledger-gated (not a re-open of plan 112).

## Enforcement evidence

`parallax-xtask` includes a negative architecture fixture proving that an
`agent_context` product can depend on `parallax-evidence` but fails when it
depends on another product capability. Removing the marker or adding a raw
workspace dependency makes `cargo xtask policy` fail.

The broader transport and claim-evidence contract remains in
[agent-access-surface.md](agent-access-surface.md).
