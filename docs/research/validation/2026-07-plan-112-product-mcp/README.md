# Plan 112 evidence — product MCP local-stdio (2026-07-17)

## Product decision

- **GO for local stdio only** (operator unblock directive 2026-07-17 + durable
  note in `docs/research/decisions/agent-access-surface.md`).
- **NO remote MCP** until Plan 109 bearer + TLS edge are integrated into the
  MCP adapter (Plan 109 minimal bearer now exists on GraphQL/SSE; MCP still
  loopback-only by construction).
- Catalog: exact two read-only tools
  (`parallax_issue_context`, `parallax_agent_session_show`).
- Resources, mutating tools, shell/SQL/management: permanently denied for this
  profile.
- Spike disposition: **graduated** to product crate `parallax-mcp` (binary
  `parallax-mcp`); compatibility bin alias `parallax-mcp-spike` retained for
  one migration cycle.

## Verification (2026-07-17)

```text
cargo test -p parallax-mcp
# 35 passed

# Live claimed-client discovery (temporary registration, then removed):
codex mcp add parallax-live-probe -- $PWD/target/debug/parallax-mcp --allow-local-stdio
codex mcp list   # shows parallax-live-probe enabled (stdio)
codex mcp remove parallax-live-probe

claude mcp add --scope user parallax-live-probe -- $PWD/target/debug/parallax-mcp --allow-local-stdio
claude mcp list  # parallax-live-probe ✔ Connected
claude mcp remove --scope user parallax-live-probe
```

Hardening already on `main` includes: `--allow-local-stdio` trust, loopback-only
API origin, closed tool catalog, protocol pin `2025-11-25`, hash/schema
fail-closed, secret-free errors, 128 KiB result ceiling, oversized → bounded
summary + `parallax://evidence/…` resource refs, per-call audit row +
`parallax.mcp.audit` tracing span (Layer-capture verified in unit tests), no rustls.

## Claimed-client install sketches

### Codex (`~/.codex/config.toml`)

```toml
[mcp_servers.parallax]
command = "parallax-mcp"
args = ["--allow-local-stdio"]
# optional: env = { PARALLAX_URL = "http://127.0.0.1:4000" }
```

### Claude Code (user scope)

```bash
claude mcp add --scope user parallax -- parallax-mcp --allow-local-stdio
```

Do **not** put project-scoped auto-enable config in the Parallax repository.

## Residual / out of residual

| Gate | Status |
| --- | --- |
| Live Codex/Claude discovery | **passed** 2026-07-17 (add/list/remove probe) |
| Oversized → bounded summary + approved resource refs | landed |
| Per-call audit row + OTel span | landed (row log + Layer-capture tests) |
| Graduate spike → product binary | **graduated** `parallax-mcp` |
| Negative tool/capability/protocol fixtures | permanent fail-closed (crate tests) |
| Remote transport | out of residual until deliberate 109 integration |

## Plan 109 dependency

Remote MCP remains out of scope. Local GraphQL may use optional bearer via
Plan 109; MCP does not inject tokens because loopback open-mode remains the
supported product profile.
