# parallax-mcp-spike

**SPIKE only — not a product surface.** May be deleted after the MCP ship/no-ship
decision. Do not package, do not enable by default, do not document in the user
guide.

Proves that a thin stdio MCP server over the existing GraphQL API can reproduce
the canonical evidence bundle byte-for-byte (CLI ↔ HTTP ↔ MCP projection
equivalence). See:

- Plan: [`advisor-plans/083-mcp-adapter-spike.md`](../../advisor-plans/083-mcp-adapter-spike.md)
- Findings: [`docs/research/validation/2026-07-11-mcp-spike-projection-equivalence.md`](../../docs/research/validation/2026-07-11-mcp-spike-projection-equivalence.md)
- Design: [`docs/research/decisions/agent-access-surface.md`](../../docs/research/decisions/agent-access-surface.md)

## Tools (read-only catalog, spike subset)

| Tool | Args | Source |
| --- | --- | --- |
| `parallax_issue_context` | `fingerprint: string` | GraphQL `bundle(fingerprint:)` |
| `parallax_agent_session_show` | `run_id: string` | GraphQL `agentSession(runId:)` |

No shell, SQL, deploy, rollback, or management tools exist in this binary.

## Usage

Requires a live `parallax serve` (default `http://127.0.0.1:4000`).

```bash
# stdio MCP server (default)
cargo run -p parallax-mcp-spike

# projection-equivalence proof (issue + optional run anchor)
cargo run -p parallax-mcp-spike -- check \
  --fingerprint <fp> \
  --run-id <run_id>   # optional second anchor
```

No CI wiring: needs a live server and seeded telemetry. Manual only.

## SDK / TLS

`rmcp` 2.2.0 with `default-features = false` and only `server`, `transport-io`,
`macros`. No `reqwest`/`rustls` feature of the SDK is enabled. HTTP to GraphQL
uses the workspace `reqwest` with `native-tls-vendored`.
