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

## Usage

Requires a live `parallax serve` (default `http://127.0.0.1:4000`).

```bash
# stdio MCP server (default)
cargo run -p parallax-mcp-spike

# projection-equivalence proof (issue + optional invocation anchor)
cargo run -p parallax-mcp-spike -- check \
  --fingerprint <fp> \
  --invocation-id <invocation_id>   # optional second anchor
```

No CI wiring: needs a live server and seeded telemetry. Manual only.

## SDK / TLS

`rmcp` 2.2.0 (latest stable rechecked 2026-07-17) with
`default-features = false` and only `server`, `transport-io`,
`macros`. No `reqwest`/`rustls` feature of the SDK is enabled. HTTP to GraphQL
uses the workspace `reqwest` with `native-tls-vendored`.

## Owned concerns

Isolated proof of read-only MCP projection equivalence; never product packaging.
Tool results expose bounded text plus `structuredContent`; comparison-only raw
canonical JSON metadata was removed before product graduation.

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
