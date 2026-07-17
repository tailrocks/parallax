# Claude Code session capture adapter (plan 120)

**Status:** preliminary approved shape; fixture proof required before product claim  
**Decision date:** 2026-07-17  
**Approver:** operator unblock directive (plan 120 first adapter = Claude Code)  
**Owner:** Plan 120

## Decision

Parallax's first coding-agent session adapter targets **Claude Code** only.

| Field | Contract |
| --- | --- |
| Tool | Claude Code CLI |
| Version range (claim floor) | `2.1.150`–`2.1.212` (local probe 2026-07-17: `2.1.212`) |
| First capture surface | Print-mode **stream-json** NDJSON (`-p --output-format stream-json`) |
| Secondary surface (same crate, separate claim) | Documented **hook stdin JSON** (command/HTTP hook payloads) |
| Deferred | Interactive native OTel export; transcript file scrape; plugin reverse-engineering |
| Consent | Explicit operator config / CLI import only — **never** auto-enable from checkout |
| Content default | Structural events only; prompt/tool bodies are hashes or redacted refs |

## Why stream-json first

Research (`docs/research/capture/agent-cli-tracing.md`) ranks Claude Code native
OTel as the strongest interactive surface, but that path needs a live OTel
exporter and operator telemetry opt-in. Stream-json is:

- Documented for non-interactive / fixture automation
- Version-probeable without scraping private state
- Able to include hook lifecycle events via `--include-hook-events`
- Compatible with explicit consent (`parallax` import/capture command later)

OTel remains the preferred interactive claim and lands as a separate fixture-
gated slice.

## Supported stream-json event types (v1)

| `type` | Normalized kind | Notes |
| --- | --- | --- |
| `system` + `subtype=init` | `session_start` | Session id, cwd, model when present |
| `assistant` | `model_turn` | Token usage if present; message text redacted by default |
| `user` | `user_turn` | Prompt text redacted by default |
| `result` | `session_end` | Success/error, cost, duration, turn count |
| `hook` / hook-shaped | `hook_event` | When `--include-hook-events` emits rows |
| unknown | `unknown` + lossiness | Fail closed for sensitive fields |

Hook stdin JSON (separate input path) maps documented events
`SessionStart`, `SessionEnd`, `PreToolUse`, `PostToolUse`, `PostToolUseFailure`,
`PermissionRequest`, `Stop`, `StopFailure` into the same normalized action kinds.

## Consent and install

- Capture is off unless the operator enables an explicit adapter config or runs
  an import command with a path/stream.
- Repository `.claude/settings.json` must not auto-start Parallax capture.
- Hook installation that posts to Parallax is operator-owned; Parallax never
  injects project hooks from a checkout by default.

## Lossiness

Every normalized session records:

- `source_tool`, `source_version`, `capture_surface`
- `lossiness[]` reasons (e.g. `prompt_body_redacted`, `tool_input_redacted`,
  `bare_mode_unknown`, `hook_events_absent`)
- Stable `session_id` from the source when present

## Primary sources

- [Claude Code hooks](https://code.claude.com/docs/en/hooks)
- [Claude Code monitoring](https://code.claude.com/docs/en/monitoring-usage)
- [Claude Code CLI / headless](https://code.claude.com/docs/en/cli-usage)
- Local `claude --version` probe: `2.1.212` (2026-07-17)

## Open gates

1. Sanitized real success-path stream-json fixtures from a logged-in Claude Code
   (current environment returned auth-error shaped `result` only).
2. Hook payload fixtures for Pre/PostToolUse with redacted tool input.
3. Storage/API/UI projection and bundle inclusion.
4. Overhead/loss measurement ledger rows.
