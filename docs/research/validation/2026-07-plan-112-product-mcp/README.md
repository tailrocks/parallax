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
- Spike disposition: **graduated** to product crate `parallax-mcp` (local-stdio only);
  compatibility bin alias `parallax-mcp` retained for one migration cycle.

## Spike verification (2026-07-17)

```text
cargo test -p parallax-mcp
# 30 passed
```

Hardening already on `main` includes: `--allow-local-stdio` trust, loopback-only
API origin, closed tool catalog, protocol pin `2025-11-25`, hash/schema
fail-closed, secret-free errors, 128 KiB result ceiling, no rustls.

## Claimed-client install sketches (not live fixtures)

These are the **supported config shapes** for local stdio. They are not
repository auto-trust files. Operators paste into user-scope config after an
explicit install decision.

### Codex (`~/.codex/config.toml`)

```toml
[mcp_servers.parallax]
command = "parallax-mcp"
args = ["--allow-local-stdio"]
# optional: env = { PARALLAX_URL = "http://127.0.0.1:4000" }
```

Do **not** put project-scoped auto-enable config in the Parallax repository.
Prefer user scope; require workspace trust before enabling in a project.

### Claude Code (user scope)

```bash
claude mcp add --scope user parallax -- parallax-mcp --allow-local-stdio
```

Notes from local client docs:

- `claude mcp get`/`list` may spawn stdio servers for health without the full
  workspace trust dialog — treat that as a residual client-retention risk and
  keep evidence free of secrets (already enforced by the adapter).
- Tool Search may defer MCP tools; tool names/descriptions must remain
  discoverable (locked by wire `tools/list` fixtures).

## Residual gates (do not retire Plan 112)

| Gate | Status |
| --- | --- |
| Live Codex/Claude discovery + invocation fixtures | unfinished |
| Oversized → bounded summary + approved resource refs | fail-closed only |
| Per-call audit row + OTel span | **audit row landed 2026-07-17** (`parallax-mcp/src/audit.rs`): secret-free in-process rows (tool/principal/scopes/status/result_bytes/duration); no anchors/evidence; 1024-row cap; wired on both tools. OTel span still unfinished (spike still installs no tracing subscriber by design). |
| Client retention matrix (memory / attachments) | documented residual only |
| Graduate spike → product binary / package | not started |
| Remote transport | blocked on deliberate 109 integration into MCP |

## Plan 109 dependency

Remote MCP remains out of scope. Local GraphQL now supports optional bearer via
Plan 109; MCP does not yet inject tokens because loopback open-mode remains the
supported product profile for the spike.
