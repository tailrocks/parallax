# Plan 120 — Claude Code session capture residual (closed 2026-07-17)

## Scope closed

| Residual | Landed surface |
| --- | --- |
| Success stream-json fixture | prior `success-stream-json.ndjson` |
| Pre/PostToolUse sanitized fixtures | `pre-tool-use-hook.json`, `post-tool-use-hook.json` |
| Explicit-ID restart/redelivery | prior normalizer + loss counters |
| Consent CLI import | `parallax import-claude <path> [--json]` |
| Durable storage | Turso `agent_session_imports` (idempotent import_id + payload hash) |
| Loss ledger bounds | `MAX_LINE_BYTES` / `MAX_EVENTS` + unit gate |
| Doctor inventory | `agent-session imports` count |
| API/UI projection | Existing GraphQL `agentSession` for OTel-derived agent spans; import surface is CLI + Turso ledger (no raw transcript default) |

## Consent

- Never auto-enabled from checkout.
- Operator must run `import-claude` with an explicit path.

## Verification

```text
cargo test -p parallax-evidence --lib post_and_pre_tool
cargo test -p parallax-evidence --lib success_path_fixture
cargo test -p parallax-metadata --lib agent_session_import
cargo test -p parallax-cli --bin parallax
cargo xtask policy --only structural
```
