+++
schema_version = 1
package = "parallax-mcp-spike"
class = "proof"
dependencies = []
facade_roots = ["main.rs"]
+++

# parallax-mcp-spike

**SPIKE only — not a product surface.** May be deleted after the MCP ship/no-ship
decision. Do not package, do not enable by default, do not document in the user
guide.

Proves that a thin stdio MCP server over the existing GraphQL API can reproduce
the canonical evidence bundle byte-for-byte (CLI ↔ HTTP ↔ MCP projection
equivalence). See:

- Findings: [`docs/research/validation/2026-07-11-mcp-spike-projection-equivalence.md`](../../docs/research/validation/2026-07-11-mcp-spike-projection-equivalence.md)
- Design: [`docs/research/decisions/agent-access-surface.md`](../../docs/research/decisions/agent-access-surface.md)
- Active ship/no-ship work: [`plans/112-product-mcp-ship-gates.md`](../../plans/112-product-mcp-ship-gates.md)

## Tools (read-only catalog, spike subset)

| Tool | Args | Source |
| --- | --- | --- |
| `parallax_issue_context` | `fingerprint: string` | GraphQL `bundle(fingerprint:)` |
| `parallax_agent_session_show` | `invocation_id: string` | GraphQL `agentSession(invocationId:)` |

No shell, SQL, deploy, rollback, or management tools exist in this binary.
Both tools advertise MCP annotations as read-only, non-destructive, idempotent,
and closed-world.
Anchor schemas and runtime validation require 1–256 UTF-8 bytes; invalid-input
errors never echo the supplied anchor.
Missing bundles and sessions return stable MCP resource-not-found errors;
transport and malformed-response failures remain secret-free internal errors.

## Usage

Requires a live `parallax serve` (default `http://127.0.0.1:4000`).

```bash
# stdio MCP server (explicit local trust required)
cargo run -p parallax-mcp-spike -- --allow-local-stdio

# projection-equivalence proof (issue + optional invocation anchor)
cargo run -p parallax-mcp-spike -- check \
  --fingerprint <fp> \
  --invocation-id <invocation_id>   # optional second anchor
```

No CI wiring: needs a live server and seeded telemetry. Manual only.
The stdio server fails closed unless `--allow-local-stdio` appears on the
process command line; no environment variable or repository file can provide
that trust decision.
Unit coverage includes wire-level MCP initialization and `tools/list` over an
in-memory stdio-equivalent duplex transport. Live Codex/Claude fixtures remain
manual and unfinished.
Wire fixtures require empty, terminal prompt/resource/template catalogs and
method-level denial for prompt/resource reads; those capabilities remain
disabled.
The API origin is restricted to credential-free plaintext HTTP on literal
loopback IPs; hostnames are rejected so DNS/hosts configuration cannot escape
the local boundary.
Authenticated remote transport remains deferred to Plan 109.
HTTP redirects are disabled so loopback cannot bounce a request to a remote
origin; connects time out after 5 seconds and calls after 30 seconds.
The spike installs no tracing subscriber, so `RUST_LOG` cannot enable MCP
protocol/result logging or persist anchors and evidence through dependency
diagnostics.

## SDK / TLS

`rmcp` 2.2.0 (latest stable rechecked 2026-07-17) with
`default-features = false` and only `server`, `transport-io`,
`macros`. No `reqwest`/`rustls` feature of the SDK is enabled. HTTP to GraphQL
uses the workspace `reqwest` with `native-tls-vendored`.
The server explicitly advertises stable MCP `2025-11-25`; it does not inherit
an SDK `LATEST` value that could silently opt into a newer protocol revision.
Initialization accepts only reviewed revisions from `2024-11-05` through
`2025-11-25`; SDK-known future and unknown revisions fail closed.

## Owned concerns

Isolated proof of read-only MCP projection equivalence; never product packaging.
Tool results expose bounded text plus `structuredContent`; comparison-only raw
canonical JSON metadata was removed before product graduation.
Bundle assembly is explicitly capped at 4,000 tokens for MCP because each call
returns both the canonical structured object and compatibility text.
The adapter streams GraphQL responses through a hard 1 MiB pre-parse ceiling,
including chunked responses and agent-session projections.
Final structured content plus compatibility text has a combined 128 KiB
ceiling. Oversized results fail closed until resource-reference delivery lands.
The issue-context tool advertises the checked-in bundle-v2 JSON Schema as its
MCP `outputSchema`.
Agent-session responses decode into a closed typed projection and advertise its
generated MCP `outputSchema`; arbitrary GraphQL fields cannot pass through.

## Source map

- [src/main.rs](src/main.rs)
- [Reviewed facade manifest](facade.toml)

## Public surface

The supported `main.rs` paths are the exports recorded in the
[reviewed facade manifest](facade.toml); implementation modules are not a
compatibility surface.

## Verification

Run `cargo check -p parallax-mcp-spike` for the narrow crate gate and `cargo xtask facade check` for
root-surface drift.
